//! Funciones FFI para el observable "Trabajos" del Panel Operativo.
//!
//! Expone una consulta de los últimos 20 trabajos de la base de datos de
//! Drasus, en cualquier estado, hacia Flutter/Dart.

use shared::public_interface::{create_pool, run_migrations};
use sqlx::Row;

/// Resumen de un trabajo, con solo los campos necesarios para el Panel
/// Operativo. Todos los campos son primitivos o `String` — tipos que
/// flutter_rust_bridge puede transportar a través de la frontera FFI sin
/// serialización adicional.
///
/// ## Tipos en Rust vs Dart
///
/// | Campo      | Rust     | Dart     | Nullable en Dart |
/// |------------|----------|----------|-----------------|
/// | `id`       | `String` | `String` | No              |
/// | `job_type` | `String` | `String` | No              |
/// | `state`    | `String` | `String` | No              |
/// | `created_at` | `i64`  | `int`    | No              |
///
/// `created_at` es un timestamp en nanosegundos desde el Unix epoch (i64).
/// Dart lo representa como `int` (entero de 64 bits con signo), compatible.
pub struct JobSummary {
    /// Identificador único del trabajo (UUID v4 como cadena de texto).
    /// Propietario: Rust genera el UUID; Dart lo recibe de solo lectura.
    pub id: String,
    /// Tipo de trabajo registrado al momento del envío (ej. "BACKTEST").
    pub job_type: String,
    /// Estado actual del trabajo: "QUEUED", "RUNNING", "COMPLETED", "FAILED"
    /// o "CANCELLED". Se almacena y transmite como String para no acoplar
    /// el Panel al enum interno de Rust.
    pub state: String,
    /// Momento de creación del trabajo en nanosegundos desde el Unix epoch.
    /// Se usa i64 porque SQLite almacena el campo como INTEGER con signo,
    /// y se mantiene el mismo tipo en toda la pila para evitar truncamientos.
    pub created_at: i64,
}

/// Retorna los últimos 20 trabajos (en cualquier estado) desde la base de
/// datos en `db_path`, ordenados del más reciente al más antiguo.
///
/// ## Argumentos
/// - `db_path`: ruta al archivo SQLite de Drasus (ej. `/home/user/drasus.db`).
///   Se convierte internamente a la URL de conexión que SQLx entiende.
///
/// ## Ownership al cruzar la frontera FFI
/// `Vec<JobSummary>` se serializa por flutter_rust_bridge en una lista Dart.
/// Rust libera el vector; Dart recibe una copia de los datos en su propio
/// heap. No hay memoria compartida — cada lado gestiona la suya.
///
/// ## Por qué async (sin #[frb(sync)])
/// Esta función hace I/O (abre la base de datos, ejecuta una query SQLite).
/// Si fuera síncrona bloquearía el hilo principal de Flutter durante el acceso
/// al disco, causando frames caídos. flutter_rust_bridge la convierte en un
/// `Future<List<JobSummary>>` en Dart: Flutter llama `await getJobsSummary()`
/// y el motor de UI sigue dibujando mientras espera.
///
/// ## Error handling en la frontera
/// Si la conexión a la BD falla o la query falla, se retorna un `Vec` vacío.
/// El Panel muestra "sin datos" en vez de colapsar — adecuado para un Panel
/// de diagnóstico donde un error de BD es observable en sí mismo.
pub async fn get_jobs_summary(db_path: String) -> Vec<JobSummary> {
    // Construye la URL de conexión SQLx desde la ruta recibida de Dart.
    let url = format!("sqlite://{db_path}");

    // Abre el pool de conexiones SQLite. Falla silenciosamente si la ruta
    // no existe o no es una base de datos Drasus válida.
    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    // Aplica las migraciones si la BD es nueva o está desactualizada.
    // Idempotente: no hace nada si las migraciones ya están aplicadas.
    if run_migrations(&pool).await.is_err() {
        return Vec::new();
    }

    // Consulta los 20 trabajos más recientes en cualquier estado.
    // JobRepository no expone un método de listado general (solo filtra
    // por estado), por lo que la Shell consulta directamente vía sqlx —
    // este es el único punto donde el Bridge toca sqlx directamente.
    let rows = match sqlx::query(
        "SELECT id, job_type, state, created_at \
         FROM jobs \
         ORDER BY created_at DESC \
         LIMIT 20",
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return Vec::new(),
    };

    // Mapea cada fila al struct FFI-safe. El estado viene como String desde
    // la BD (ej. "QUEUED") — no se necesita parsear al enum Rust porque
    // la UI solo lo muestra, no toma decisiones con él.
    rows.into_iter()
        .map(|row| JobSummary {
            id: row.get("id"),
            job_type: row.get("job_type"),
            state: row.get("state"),
            created_at: row.get("created_at"),
        })
        .collect()
}
