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
    create_pool, run_migrations, run_mcp_server, verify_central_identity, verify_licensing_system,
    verify_plan_tier_quota, verify_usage_metering, CentralIdentityVerifyInput, ExecutorIdentity,
    JobExecutor, JobExecutorConfig, LicensingSystemVerifyInput, PlanTierQuotaVerifyInput,
    SystemClock, UsageMeteringVerifyInput,
};
use sovereign_data_fetcher::public_interface::{VerifyInput, verify};

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

/// Subcomandos disponibles.
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

    /// Ejecuta el harness de verificación de una feature y emite el resultado como JSON.
    ///
    /// Ejemplo:
    ///   drasus verify sovereign-data-fetcher --input '{"symbol":"BTCUSDT","interval":"1h"}'
    ///   drasus verify central-identity --input '{"email":"a@b.com"}'
    ///   drasus verify licensing-system --input '{"tier":"SOVEREIGN"}'
    ///   drasus verify plan-tier-quota --input '{"tier":"FREE"}'
    ///   drasus verify usage-metering --input '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'
    ///
    /// La salida JSON va a stdout; los errores van a stderr con exit code != 0.
    Verify {
        /// Identificador de la feature a verificar en kebab-case.
        /// Features soportadas en Fase 1: `sovereign-data-fetcher`, `central-identity`, `licensing-system`, `plan-tier-quota`, `usage-metering`.
        feature_id: String,

        /// Input JSON para la verificación.
        /// Si se omite, se usan los valores por defecto de la feature
        /// (BTCUSDT, intervalo 1h, 1 día hacia atrás).
        #[arg(long)]
        input: Option<String>,
    },
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

        Commands::Verify { feature_id, input } => {
            // Despacha la verificación y delega la presentación del resultado.
            run_verify(&feature_id, input.as_deref()).await;
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Lógica de arranque (Shell puro — no hay lógica de dominio)
// ────────────────────────────────────────────────────────────────────────────

/// Despacha el subcomando `verify` a la feature indicada por `feature_id`.
///
/// Parsea el JSON de entrada (o usa los valores por defecto si `input_json` es None),
/// llama a la función de verificación de la feature y serializa el output a stdout.
/// En caso de error (feature desconocida, JSON malformado o fallo de verificación),
/// escribe el error en stderr y sale con código 1.
async fn run_verify(feature_id: &str, input_json: Option<&str>) {
    match feature_id {
        // ── Sovereign Data Fetcher ────────────────────────────────────────────
        "sovereign-data-fetcher" => {
            // Parsea el input JSON o usa los valores por defecto de la feature.
            let input: VerifyInput = match input_json {
                Some(json) => match serde_json::from_str(json) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error al parsear --input JSON: {e}");
                        std::process::exit(1);
                    }
                },
                // Sin --input: usa los valores por defecto (BTCUSDT, 1h, 1 día).
                None => VerifyInput::default(),
            };

            // Llama a la función de verificación con adaptadores reales.
            // El resultado incluye job_id, record_id, bytes descargados o descripción del error.
            let output = verify(input).await;

            // La salida JSON va siempre a stdout para que el usuario pueda pipear a `jq`.
            let json = serde_json::to_string_pretty(&output)
                // serde_json::to_string_pretty solo falla si el tipo tiene claves Map no-string,
                // lo cual no aplica aquí; el expect documenta que es imposible que falle.
                .expect("VerifyOutput siempre es serializable a JSON");
            println!("{json}");

            // Si la verificación falló, emite el código de salida 1 para que los scripts
            // puedan detectar el fallo (exit code 0 solo en éxito).
            if !output.ok {
                std::process::exit(1);
            }
        }

        // ── Central Identity (STORY-027, vive en `shared` -- ver ADR-0137) ────
        "central-identity" => {
            // Parsea el input JSON. A diferencia de sovereign-data-fetcher,
            // `email` es obligatorio (sin valores por defecto razonables para
            // un correo), así que sin --input no hay nada que verificar.
            let input: CentralIdentityVerifyInput = match input_json {
                Some(json) => match serde_json::from_str(json) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error al parsear --input JSON: {e}");
                        std::process::exit(1);
                    }
                },
                None => {
                    eprintln!(
                        "central-identity requiere --input con al menos {{\"email\":\"...\"}}"
                    );
                    std::process::exit(1);
                }
            };

            let output = verify_central_identity(input).await;

            let json = serde_json::to_string_pretty(&output)
                // serde_json::to_string_pretty solo falla si el tipo tiene claves Map no-string,
                // lo cual no aplica aquí; el expect documenta que es imposible que falle.
                .expect("CentralIdentityVerifyOutput siempre es serializable a JSON");
            println!("{json}");

            if !output.ok {
                std::process::exit(1);
            }
        }

        // ── Licensing System (STORY-028, vive en `shared` -- ver ADR-0137) ────
        "licensing-system" => {
            // A diferencia de central-identity, aquí SÍ hay defaults razonables
            // para todos los campos (tier = SOVEREIGN, correo fijo de humo) --
            // por eso `drasus verify licensing-system` sin --input también es válido.
            let input: LicensingSystemVerifyInput = match input_json {
                Some(json) => match serde_json::from_str(json) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error al parsear --input JSON: {e}");
                        std::process::exit(1);
                    }
                },
                None => match serde_json::from_str("{}") {
                    Ok(v) => v,
                    // "{}" con todos los campos #[serde(default)] siempre parsea.
                    Err(_) => unreachable!("LicensingSystemVerifyInput con defaults debe parsear desde '{{}}'"),
                },
            };

            let output = verify_licensing_system(input).await;

            let json = serde_json::to_string_pretty(&output)
                // serde_json::to_string_pretty solo falla si el tipo tiene claves Map no-string,
                // lo cual no aplica aquí; el expect documenta que es imposible que falle.
                .expect("LicensingSystemVerifyOutput siempre es serializable a JSON");
            println!("{json}");

            if !output.ok {
                std::process::exit(1);
            }
        }

        // ── Plan / Tier / Quota (STORY-029, vive en `shared` -- ver ADR-0137) ──
        "plan-tier-quota" => {
            // A diferencia de central-identity, aquí SÍ hay un default
            // razonable (tier = FREE) -- por eso `drasus verify
            // plan-tier-quota` sin --input también es válido.
            let input: PlanTierQuotaVerifyInput = match input_json {
                Some(json) => match serde_json::from_str(json) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error al parsear --input JSON: {e}");
                        std::process::exit(1);
                    }
                },
                None => match serde_json::from_str("{}") {
                    Ok(v) => v,
                    // "{}" con todos los campos #[serde(default)] siempre parsea.
                    Err(_) => unreachable!("PlanTierQuotaVerifyInput con defaults debe parsear desde '{{}}'"),
                },
            };

            let output = verify_plan_tier_quota(input).await;

            let json = serde_json::to_string_pretty(&output)
                // serde_json::to_string_pretty solo falla si el tipo tiene claves Map no-string,
                // lo cual no aplica aquí; el expect documenta que es imposible que falle.
                .expect("PlanTierQuotaVerifyOutput siempre es serializable a JSON");
            println!("{json}");

            if !output.ok {
                std::process::exit(1);
            }
        }

        // ── Usage Metering (STORY-030, vive en `shared` -- ver ADR-0137) ──────
        "usage-metering" => {
            // A diferencia de plan-tier-quota, aquí `operations` es
            // obligatorio (sin operaciones no hay nada que medir) -- por
            // eso `drasus verify usage-metering` SIN --input no es válido.
            let input: UsageMeteringVerifyInput = match input_json {
                Some(json) => match serde_json::from_str(json) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error al parsear --input JSON: {e}");
                        std::process::exit(1);
                    }
                },
                None => {
                    eprintln!(
                        "usage-metering requiere --input con al menos {{\"operations\":[{{\"size\":...,\"price\":...}}]}}"
                    );
                    std::process::exit(1);
                }
            };

            let output = verify_usage_metering(input).await;

            let json = serde_json::to_string_pretty(&output)
                // serde_json::to_string_pretty solo falla si el tipo tiene claves Map no-string,
                // lo cual no aplica aquí; el expect documenta que es imposible que falle.
                .expect("UsageMeteringVerifyOutput siempre es serializable a JSON");
            println!("{json}");

            if !output.ok {
                std::process::exit(1);
            }
        }

        // ── Feature no reconocida ─────────────────────────────────────────────
        unknown => {
            eprintln!(
                "feature-id no reconocido: '{unknown}'. Features soportadas en Fase 1: sovereign-data-fetcher, central-identity, licensing-system, plan-tier-quota, usage-metering"
            );
            std::process::exit(1);
        }
    }
}

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

// ── Tests unitarios de parseo de argumentos CLI ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Verifica que el subcomando `verify` con `--input` parsea correctamente.
    ///
    /// Simula el comando:
    ///   drasus verify sovereign-data-fetcher --input '{"symbol":"BTCUSDT","interval":"1h"}'
    #[test]
    fn cli_verify_with_input_parses_correctly() {
        let cli = Cli::try_parse_from([
            "drasus",
            "verify",
            "sovereign-data-fetcher",
            "--input",
            r#"{"symbol":"BTCUSDT","interval":"1h"}"#,
        ])
        .expect("el subcomando verify con --input debe parsear sin error");

        match cli.command {
            Commands::Verify { feature_id, input } => {
                assert_eq!(feature_id, "sovereign-data-fetcher", "feature_id incorrecto");
                let input_str = input.expect("--input debe estar presente");
                // El JSON del input debe parsear a VerifyInput correctamente.
                let parsed: VerifyInput = serde_json::from_str(&input_str)
                    .expect("--input debe ser JSON válido de VerifyInput");
                assert_eq!(parsed.symbol, "BTCUSDT");
                assert_eq!(parsed.interval, "1h");
            }
            other => panic!("se esperaba Commands::Verify, se obtuvo {:?}", std::mem::discriminant(&other)),
        }
    }

    /// Verifica que el subcomando `verify` sin `--input` también parsea (usa defaults).
    ///
    /// Simula el comando mínimo: drasus verify sovereign-data-fetcher
    #[test]
    fn cli_verify_without_input_parses_correctly() {
        let cli = Cli::try_parse_from(["drasus", "verify", "sovereign-data-fetcher"])
            .expect("verify sin --input debe parsear sin error");

        // Sin --input el campo input debe ser None; run_verify usa los valores por defecto.
        assert!(
            matches!(cli.command, Commands::Verify { ref feature_id, input: None } if feature_id == "sovereign-data-fetcher"),
            "se esperaba Verify {{ feature_id: sovereign-data-fetcher, input: None }}"
        );
    }

    /// Verifica que `verify` con un feature-id desconocido parsea (el error ocurre en runtime).
    ///
    /// Clap acepta cualquier string como feature_id; la validación del feature-id conocido
    /// ocurre en `run_verify` en tiempo de ejecución, no en el parseo de args.
    #[test]
    fn cli_verify_unknown_feature_parses_at_clap_level() {
        let result = Cli::try_parse_from(["drasus", "verify", "feature-inexistente"]);
        // Clap debe aceptar el argumento; la validación es responsabilidad de run_verify.
        assert!(result.is_ok(), "Clap debe aceptar cualquier feature-id sin validar");
    }
}
