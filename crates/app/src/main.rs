//! [SHELL] Binario raíz `drasus` — punto de composición del motor.
//!
//! Responsabilidad única: cablear los componentes (CLI → pool → executor)
//! y esperar la señal de cierre del OS. Cero lógica de dominio aquí.
//!
//! ADR-0003: importa SOLO `public_interface` de cada crate; prohibido
//! importar internals (`domain::`, `persistence::`, `orchestrator::`).

use std::collections::HashMap;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use shared::public_interface::{
    create_pool, run_migrations, run_mcp_server,
    ExecutorIdentity, JobExecutor, JobExecutorConfig, SystemClock,
};

// ────────────────────────────────────────────────────────────────────────────
// CLI declarativa (Clap 4 con el macro `derive`)
// ────────────────────────────────────────────────────────────────────────────

/// Motor de trading algorítmico Drasus Engine.
#[derive(Parser)]
#[command(
    name = "drasus",
    about = "Motor de trading algorítmico Drasus Engine",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Subcomandos disponibles en EPIC-0.
/// Los subcomandos de EPIC-1+ (`ingest`, `backtest`…) se añadirán
/// en sus respectivas épicas (ver §8 de STORY-009).
#[derive(Subcommand)]
enum Commands {
    /// Arranca el motor: inicializa la BD, recupera jobs pendientes y espera
    /// señal de cierre (Ctrl+C / SIGTERM).
    Start {
        /// Ruta al archivo SQLite de persistencia.
        /// Se crea automáticamente si no existe (ADR-0107: SQLite en modo WAL).
        #[arg(long, default_value = "drasus.db")]
        db: String,
    },

    /// Imprime la versión del binario y sale.
    Version,
}

// ────────────────────────────────────────────────────────────────────────────
// Punto de entrada principal
// ────────────────────────────────────────────────────────────────────────────

/// El atributo `#[tokio::main]` transforma `async fn main` en una
/// `fn main` síncrona que lanza el runtime multi-hilo de Tokio.
/// La feature `rt-multi-thread` del Cargo.toml habilita el scheduler
/// con múltiples threads del OS (necesario para futures concurrentes).
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            // `env!("CARGO_PKG_VERSION")` se resuelve en tiempo de
            // compilación: Cargo inyecta la versión del Cargo.toml
            // como variable de entorno durante el build.
            println!("drasus v{}", env!("CARGO_PKG_VERSION"));
        }

        Commands::Start { db } => {
            run_start(&db).await;
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Lógica de arranque (Shell puro — no hay lógica de dominio)
// ────────────────────────────────────────────────────────────────────────────

/// Inicializa el pool, aplica migraciones, recupera jobs del crash anterior
/// y bloquea esperando señal de cierre.
///
/// Separo esta función de `main` para que sea testeable de forma aislada
/// en el futuro (convention: `main` solo parsea CLI y delega).
async fn run_start(db_path: &str) {
    // Construye la URL de conexión SQLite que entiende SQLx.
    // El prefijo `sqlite://` es obligatorio en el formato de SQLx.
    let database_url = format!("sqlite://{db_path}");

    // Abre el pool de conexiones con modo WAL (ADR-0107).
    // `create_pool` es un alias de `persistence::pool::connect`
    // re-exportado desde `public_interface`.
    let pool = create_pool(&database_url)
        .await
        .expect("No se pudo abrir la base de datos. Verifica la ruta y permisos.");

    // Aplica migraciones embebidas en el binario (ADR-0006).
    // Idempotente: segunda ejecución contra la misma BD es un no-op.
    run_migrations(&pool)
        .await
        .expect("Las migraciones fallaron. Revisa la integridad de la base de datos.");

    // Construye el reloj de producción (SystemClock: tiempo real del OS).
    // Se inyecta como `Arc<dyn Clock>` para mantener el determinismo del
    // Core — en tests se sustituye por DeterministicClock sin cambiar nada.
    let clock = Arc::new(SystemClock::default());

    // Identidad del proceso: metadatos ADR-0020 V2 del executor.
    // `process_id` identifica esta instancia en el audit log y como
    // `worker_id` en las transiciones de jobs.
    let identity = ExecutorIdentity {
        process_id: format!("drasus-pid-{}", std::process::id()),
        session_id: None,
        node_id: None,
        logic_hash: Some(env!("CARGO_PKG_VERSION").to_string()),
        institutional_tag: "drasus-engine".to_string(),
    };

    // Construye el executor con config por defecto (max 3 jobs concurrentes).
    // En EPIC-0 no se registran handlers reales — TTR-ASYNC-EXECUTOR-007
    // los conectará cuando existan los módulos generate/validate/etc.
    let executor = JobExecutor::new(
        pool.clone(),
        clock,
        identity,
        JobExecutorConfig::default(),
        HashMap::new(), // sin handlers en EPIC-0
    );

    // Recupera jobs que quedaron QUEUED o RUNNING al morir el proceso anterior.
    // QUEUED → re-encola tal cual.
    // RUNNING → resetea a QUEUED (no se sabe si terminaron) y re-encola.
    // Emite un evento `JOB_RECOVERED_AT_STARTUP` en `audit_events` por cada job.
    let recovered = executor
        .recover_at_startup()
        .await
        .expect("La recuperación de startup falló.");

    if !recovered.is_empty() {
        println!(
            "Recuperados {} jobs del crash anterior.",
            recovered.len()
        );
    }

    // Lanza el servidor MCP en background (stdio).
    // SqlitePool es un Arc interno: clonar es barato (incrementa el contador de referencia).
    // tokio::spawn devuelve un JoinHandle; lo ignoramos porque el ciclo de vida
    // del servidor MCP está atado al del proceso principal: cuando el proceso termina,
    // el handle de stdin/stdout se cierra y el loop del servidor finaliza limpiamente.
    let mcp_pool = pool.clone();
    tokio::spawn(async move {
        if let Err(e) = run_mcp_server(mcp_pool).await {
            eprintln!("MCP server error: {e}");
        }
    });
    println!("Servidor MCP activo (stdio).");

    println!("Motor Drasus arrancado. Presiona Ctrl+C para detener.");

    // Espera concurrente de SIGINT (Ctrl+C interactivo) y SIGTERM
    // (señal del OS en producción / systemd / kubectl stop).
    // `tokio::select!` se desbloquea en cuanto CUALQUIERA de las
    // dos señales llega primero — el otro branch se cancela.
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            // SIGINT recibido (Ctrl+C en terminal interactiva).
        }
        _ = sigterm_received() => {
            // SIGTERM recibido (apagado ordenado por el OS).
        }
    }

    println!("Apagado limpio.");
    pool.close().await;

    // Salida con código 0: cierre limpio, sin error.
    std::process::exit(0);
}

/// Envuelve la API de señales de Unix de Tokio en una `async fn` que
/// resuelve cuando llega la primera señal SIGTERM.
///
/// `#[cfg(unix)]` — en Windows no existe SIGTERM; si el proyecto
/// alguna vez se porta a Windows, este branch quedaría inactivo y
/// `select!` solo esperaría SIGINT. Por ahora el target es Linux
/// (SAD §4, ADR-0030: Local-First / Zero-Docker sobre Linux).
#[cfg(unix)]
async fn sigterm_received() {
    use tokio::signal::unix::{signal, SignalKind};
    // `signal(SignalKind::terminate())` registra un listener de SIGTERM.
    // `recv()` bloquea hasta recibir la señal.
    let mut stream = signal(SignalKind::terminate())
        .expect("No se pudo registrar el listener de SIGTERM.");
    stream.recv().await;
}

/// Versión no-Unix: nunca resuelve (en Windows solo SIGINT está disponible).
#[cfg(not(unix))]
async fn sigterm_received() {
    std::future::pending::<()>().await;
}
