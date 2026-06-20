//! Test de integración de caja negra: gate EPIC-0 — job sobrevive a SIGKILL.
//! Solo compila y corre en Unix (SIGKILL/SIGTERM no existen en Windows).
#![cfg(unix)]
//!
//! Demuestra que los jobs persistidos en SQLite sobreviven a un `kill -9`
//! (SIGKILL) y se recuperan al reiniciar el binario `drasus`.
//!
//! Este test opera a nivel de **proceso real**:
//! - Lanza el binario compilado `drasus` con `std::process::Command`.
//! - Le envía SIGKILL (no interceptable — el kernel mata el proceso sin
//!   darle tiempo de correr ningún handler de limpieza).
//! - Verifica que la base de datos SQLite en archivo persiste los jobs y
//!   el evento `JOB_RECOVERED_AT_STARTUP` tras reiniciar.
//!
//! ADR-0003: importamos solo tipos del dominio público de `shared`; nunca
//! internals de ningún crate.

use std::process::Command;
use std::time::Duration;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use sqlx::{Row, SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::str::FromStr;

// ────────────────────────────────────────────────────────────────────────────
// Helpers internos del test
// ────────────────────────────────────────────────────────────────────────────

/// Abre un pool de lectura sobre el archivo de BD temporal.
/// Replicamos `persistence::pool::connect` localmente porque el test
/// no puede importar internals de `shared` — solo `public_interface`.
/// (En este caso usamos sqlx directamente porque el test manipula la BD
/// como "tercero externo", igual que lo haría una herramienta de debug.)
async fn open_pool(db_path: &std::path::Path) -> SqlitePool {
    let url = format!("sqlite://{}", db_path.display());
    let options = SqliteConnectOptions::from_str(&url)
        .expect("URL SQLite válida")
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .expect("abrir pool de verificación")
}

/// Inserta un job en estado QUEUED directamente en la BD.
///
/// El orquestador del binario se encargará de recuperarlo al reiniciar.
/// Insertamos sin pasar por el executor para simular el estado "job ya
/// en disco cuando el motor muere".
async fn insert_queued_job(pool: &SqlitePool, job_id: &str) {
    // Generamos valores mínimos compatibles con el esquema de `jobs`
    // (migración 0003_jobs.sql). Los campos NOT NULL con valor no funcional
    // se rellenan con placeholders — esto es un test de durabilidad, no de
    // lógica de negocio.
    let now_ns: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64;

    sqlx::query(
        "INSERT INTO jobs \
         (id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
          process_id, session_id, node_id, logic_hash, owner_id, access_token_id, \
          user_id, job_type, parameters, state, progress) \
         VALUES (?, ?, ?, ?, NULL, 1, NULL, NULL, NULL, NULL, NULL, NULL, \
                 'test-user', 'BACKTEST', '{}', 'QUEUED', 0)",
    )
    .bind(job_id)
    .bind(now_ns)
    .bind(now_ns)
    .bind(format!("hash-{job_id}")) // audit_hash minimal válido
    .execute(pool)
    .await
    .expect("insertar job en QUEUED");
}

// ────────────────────────────────────────────────────────────────────────────
// Gate EPIC-0: job sobrevive a kill -9 y se recupera al reiniciar
// ────────────────────────────────────────────────────────────────────────────

/// **Gate de cierre EPIC-0** (STORY-009 §5, criterio 5):
///
/// 1. Arranca `drasus start` con una BD temporal en archivo.
/// 2. Espera 300ms para que inicialice y corra migraciones.
/// 3. Inserta un job en QUEUED directamente en la BD.
/// 4. Envía SIGKILL al proceso — sin tiempo de cleanup.
/// 5. Verifica que el proceso murió por señal (no salida limpia).
/// 6. Reinicia `drasus start` con la misma BD.
/// 7. Espera 500ms para que corra la recuperación de startup.
/// 8. Verifica:
///    a) El job sigue en QUEUED (el executor lo encuentra y re-encola).
///    b) Existe al menos un evento `JOB_RECOVERED_AT_STARTUP` en `audit_events`.
/// 9. Apaga el segundo proceso con SIGTERM (cierre limpio).
#[tokio::test]
async fn job_survives_kill9_and_recovers_on_restart() {
    // ── Preparación ─────────────────────────────────────────────────────────
    let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
    let db_path = temp_dir.path().join("drasus_kill9_test.sqlite");

    // Ruta al binario compilado — Cargo la expone como variable de entorno
    // `CARGO_BIN_EXE_drasus` en tiempo de compilación de los tests.
    // Esto es más robusto que hardcodear `./target/debug/drasus`.
    let bin_path = env!("CARGO_BIN_EXE_drasus");

    // ── Primera instancia: arranque + SIGKILL ────────────────────────────────

    // Lanzamos `drasus start --db <ruta>` como subproceso real.
    // `spawn()` retorna inmediatamente (proceso hijo corriendo en paralelo).
    let mut first_process = Command::new(bin_path)
        .args(["start", "--db", db_path.to_str().expect("ruta UTF-8")])
        .spawn()
        .expect("lanzar primera instancia de drasus");

    let pid = first_process.id();

    // Esperamos 300ms para que el motor inicialice y aplique las migraciones.
    // En un test real de alta velocidad se podría sondear la BD, pero 300ms
    // es más que suficiente para SQLite local y mantiene el test simple.
    std::thread::sleep(Duration::from_millis(300));

    // Abrimos un pool directo sobre la BD para insertar el job.
    // La BD ya existe porque `drasus start` la creó y migró.
    let pool = open_pool(&db_path).await;
    let job_id = "test-job-kill9-001";
    insert_queued_job(&pool, job_id).await;
    pool.close().await;

    // SIGKILL (señal 9): el kernel mata el proceso inmediatamente.
    // A diferencia de SIGTERM, SIGKILL no puede ser interceptado ni ignorado.
    // El proceso muere sin ejecutar ningún handler de limpieza — esto es
    // exactamente lo que ocurre en un `kill -9` o en un OOM killer del OS.
    signal::kill(
        Pid::from_raw(pid as i32),
        Signal::SIGKILL,
    )
    .expect("enviar SIGKILL al primer proceso");

    // Esperamos a que el proceso hijo termine (ya lo mató SIGKILL).
    let exit_status = first_process
        .wait()
        .expect("esperar al primer proceso");

    // Verificamos que murió por señal, no por `exit(0)`.
    // En Unix, `status.success()` retorna false si el proceso terminó por señal.
    assert!(
        !exit_status.success(),
        "el proceso debería haber muerto por SIGKILL, no salir limpiamente"
    );

    // ── Segunda instancia: reinicio + verificación de recuperación ───────────

    // Reiniciamos con la MISMA ruta de BD — este es el escenario real de
    // "reinicio tras crash".
    let mut second_process = Command::new(bin_path)
        .args(["start", "--db", db_path.to_str().expect("ruta UTF-8")])
        .spawn()
        .expect("lanzar segunda instancia de drasus");

    let second_pid = second_process.id();

    // 500ms para que el motor corra `recover_at_startup` y emita el evento
    // de auditoría `JOB_RECOVERED_AT_STARTUP` antes de que consultemos.
    std::thread::sleep(Duration::from_millis(500));

    // ── Verificación de postcondiciones ─────────────────────────────────────
    let verify_pool = open_pool(&db_path).await;

    // (a) El job debe seguir en QUEUED.
    // Si el executor lo encontró en QUEUED lo re-encoló sin cambiar el estado.
    // Si lo encontró en RUNNING (no aplica aquí) lo habría reseteado a QUEUED.
    let job_state: String = sqlx::query(
        "SELECT state FROM jobs WHERE id = ?",
    )
    .bind(job_id)
    .fetch_one(&verify_pool)
    .await
    .expect("consultar estado del job")
    .get(0);

    assert_eq!(
        job_state, "QUEUED",
        "el job debe estar en QUEUED tras la recuperación (encontrado: {job_state})"
    );

    // (b) Debe existir al menos un evento `JOB_RECOVERED_AT_STARTUP`
    // en la tabla `audit_events`, emitido durante el reinicio.
    let recovery_event_count: i64 = sqlx::query(
        "SELECT COUNT(*) FROM audit_events WHERE action_type = 'JOB_RECOVERED_AT_STARTUP'",
    )
    .fetch_one(&verify_pool)
    .await
    .expect("consultar audit_events")
    .get(0);

    assert!(
        recovery_event_count >= 1,
        "debe existir al menos un evento JOB_RECOVERED_AT_STARTUP en audit_events \
         (encontrados: {recovery_event_count})"
    );

    verify_pool.close().await;

    // ── Cierre limpio de la segunda instancia ────────────────────────────────
    // Usamos SIGTERM (no SIGKILL): el binario tiene handler de SIGTERM que
    // llama `pool.close()` y sale con código 0 — verificamos que el cierre
    // es limpio.
    signal::kill(
        Pid::from_raw(second_pid as i32),
        Signal::SIGTERM,
    )
    .expect("enviar SIGTERM al segundo proceso");

    let second_exit = second_process
        .wait()
        .expect("esperar al segundo proceso");

    assert!(
        second_exit.success(),
        "el segundo proceso debe cerrarse limpiamente con SIGTERM (código 0)"
    );

    // `temp_dir` se limpia automáticamente al salir del scope (Drop).
}
