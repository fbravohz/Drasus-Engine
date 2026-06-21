//! Funciones FFI para el observable "Auditoría" del Panel Operativo.
//!
//! Expone los últimos N eventos de la cadena de auditoría de Drasus hacia
//! Flutter/Dart. La cadena de auditoría garantiza integridad histórica: cada
//! evento contiene el hash del evento anterior, formando una cadena que
//! detecta cualquier manipulación retroactiva.

use shared::public_interface::{create_pool, run_migrations, AuditLogRepository, SystemClock};

/// Resumen de un evento de auditoría para el Panel Operativo.
///
/// Solo se incluyen los campos necesarios para la visualización:
/// acción, entidad, cuándo ocurrió y el hash de cadena que permite
/// verificación visual rápida.
///
/// ## Tipos en Rust vs Dart
///
/// | Campo               | Rust     | Dart     | Nullable en Dart |
/// |---------------------|----------|----------|-----------------|
/// | `id`                | `String` | `String` | No              |
/// | `action_type`       | `String` | `String` | No              |
/// | `entity_type`       | `String` | `String` | No              |
/// | `created_at`        | `i64`    | `int`    | No              |
/// | `audit_chain_hash`  | `String` | `String` | No              |
///
/// ## Sobre `audit_chain_hash`
/// En el tipo interno [`shared::public_interface::AuditEvent`], `audit_chain_hash`
/// es `Option<String>` — el evento génesis (el primero de la cadena) no tiene
/// predecesor, por lo que su campo es `None`. Al cruzar la frontera FFI se
/// convierte a `String` con `unwrap_or_default()`: el evento génesis muestra
/// una cadena vacía en el Panel, lo cual es correcto (no hay hash anterior
/// que mostrar).
pub struct AuditEventSummary {
    /// Identificador único del evento (UUID v4 como cadena de texto).
    pub id: String,
    /// Tipo de acción registrada (ej. "ORDER_STATE_CHANGE", "USER_VETO",
    /// "CLOCK_NTP_SYNC"). Siempre presente; nunca vacío.
    pub action_type: String,
    /// Tipo de entidad sobre la que ocurrió la acción (ej. "ORDER",
    /// "JOB", "CLOCK"). Siempre presente; nunca vacío.
    pub entity_type: String,
    /// Momento de creación del evento en nanosegundos desde el Unix epoch.
    pub created_at: i64,
    /// Hash SHA-256 encadenado. Es el hash de este evento, que a su vez
    /// incluye el hash del evento anterior en su cálculo — cualquier
    /// modificación retroactiva de un evento rompe todos los hashes
    /// posteriores. El Panel muestra los últimos 8 caracteres para
    /// verificación visual rápida.
    ///
    /// El evento génesis (primero de la cadena) muestra cadena vacía aquí
    /// porque todavía no tiene predecesor que encadenar.
    pub audit_chain_hash: String,
}

/// Retorna los últimos `limit` eventos de auditoría desde la base de datos
/// en `db_path`, ordenados del más reciente al más antiguo.
///
/// ## Argumentos
/// - `db_path`: ruta al archivo SQLite de Drasus.
/// - `limit`: cuántos eventos devolver (el Panel solicita 50 normalmente).
///
/// ## Estrategia de carga
/// `AuditLogRepository::load_chain()` carga todos los eventos ordenados ASC
/// (del más antiguo al más reciente). Se toma el tail con `.rev().take(limit)`
/// y se invierte de vuelta para que el más reciente aparezca primero en el
/// Panel. Esto es eficiente para el volumen de EPIC-0 (decenas a miles de
/// eventos); en EPIC-1+ con millones de eventos se sustituirá por una query
/// paginada directa.
///
/// ## Por qué async (sin #[frb(sync)])
/// Igual que `get_jobs_summary`: hace I/O a SQLite y no debe bloquear el
/// hilo principal de Flutter.
///
/// ## Error handling en la frontera
/// Si la conexión falla o la cadena está vacía, retorna un `Vec` vacío.
/// El Panel muestra "sin eventos" — estado válido en una BD recién creada.
pub async fn get_recent_audit_events(db_path: String, limit: u64) -> Vec<AuditEventSummary> {
    // Construye la URL de conexión SQLx desde la ruta recibida de Dart.
    let url = format!("sqlite://{db_path}");

    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    if run_migrations(&pool).await.is_err() {
        return Vec::new();
    }

    // SystemClock::new() es requerido por AuditLogRepository pero no se
    // usa para leer eventos (solo para append). Se usa la implementación
    // de producción; el reloj no afecta las lecturas.
    let clock = SystemClock::new();
    let repo = AuditLogRepository::new(&pool, &clock);

    // Carga la cadena completa ordenada ASC (génesis primero).
    let chain = match repo.load_chain().await {
        Ok(events) => events,
        Err(_) => return Vec::new(),
    };

    // Toma los últimos `limit` eventos (tail de la cadena) y los invierte
    // para que el Panel muestre el más reciente primero.
    let limit_usize = limit.min(usize::MAX as u64) as usize;
    chain
        .into_iter()
        .rev()
        .take(limit_usize)
        .map(|event| AuditEventSummary {
            id: event.id,
            action_type: event.content.action_type,
            entity_type: event.content.entity_type,
            created_at: event.created_at_ns,
            // audit_chain_hash es Option<String>: None en el evento génesis,
            // Some(hash) en todos los demás. Se serializa como String vacía
            // para el génesis — la UI muestra "(génesis)" o queda en blanco.
            audit_chain_hash: event.audit_chain_hash.unwrap_or_default(),
        })
        .collect()
}
