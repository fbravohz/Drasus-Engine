//! [CORE] Lógica pura de construcción de muestras de telemetría
//! (`docs/features/telemetry.md` TTR-001, ADR-0015, ADR-0020).
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004). El
//! `id` y `created_at_ns` los inyecta la cáscara (orquestador), el mismo
//! patrón que [`super::audit_log::chain_event`].
//!
//! A diferencia del Audit Log, esta cadena de hashes vive solo en memoria
//! (la cáscara la siembra una vez al iniciar leyendo la última fila — ver
//! `persistence::telemetry::TelemetryRepository::load_tail` — y nunca vuelve
//! a tocar disco para encadenar una muestra nueva): ningún criterio de
//! aceptación de esta Story exige verificación de cadena, y leer la cola
//! desde SQLite en cada muestra violaría el límite de 50µs.

use sha2::{Digest, Sha256};

use super::audit_log::GENESIS_PREVIOUS_HASH;

/// Contenido de una muestra de telemetría, provisto por quien llama
/// (`docs/features/telemetry.md` "Persistencia").
///
/// `execution_latency_ms` es `Some` solo en una muestra de latencia, y
/// `None` en un heartbeat — el tipo mismo hace irrepresentable, en código
/// bien formado construido vía [`build_sample`], tanto "un heartbeat con
/// valor de latencia" como "una muestra de latencia sin valor".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetrySampleContent {
    // Específico de la feature (columnas propias, fuera del contrato de
    // 25 campos, mismo patrón que action_type/entity_type/entity_id de
    // audit_events).
    pub metric_name: String,
    pub details_json: Option<String>,

    // ADR-0020 Grupo II: Soberanía.
    pub institutional_tag: String,

    // ADR-0020 Grupo III: Pesos/Arquitectura.
    pub logic_hash: Option<String>,
    pub session_id: Option<String>,

    // ADR-0020 Grupo IV: Infraestructura / Hardware.
    pub node_id: Option<String>,
    pub process_id: String,
    pub execution_latency_ms: Option<i64>,
}

/// Una muestra de telemetría ya encadenada, lista para persistir (o ya
/// persistida) en `telemetry_samples`.
///
/// Los grupos de campos siguen el perfil técnico de ADR-0020 (Grupo I,
/// universal + content) — mismo patrón que [`super::audit_log::AuditEvent`]
/// envolviendo [`super::audit_log::AuditEventContent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetrySample {
    // I. Identidad & Integridad (universal, ADR-0020).
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub content: TelemetrySampleContent,
}

/// Construye la representación canónica en bytes que se hashea para una
/// muestra dada. Separada de [`compute_sample_hash`] por la misma razón que
/// `canonical_bytes` en `audit_log.rs`: que [`build_sample`] siempre derive
/// el mismo digest a partir de las mismas entradas lógicas.
///
/// Usa el separador `\u{1F}` (Unit Separator de ASCII) entre campos — un
/// byte que no aparece en uso normal de ninguno de estos campos de texto,
/// así que distintas combinaciones de campos no pueden colisionar en el
/// mismo stream de bytes.
fn canonical_bytes(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    content: &TelemetrySampleContent,
    previous_audit_hash: &str,
) -> Vec<u8> {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(&content.metric_name);
    push(content.details_json.as_deref().unwrap_or(""));
    push(&content.institutional_tag);
    push(content.logic_hash.as_deref().unwrap_or(""));
    push(content.session_id.as_deref().unwrap_or(""));
    push(content.node_id.as_deref().unwrap_or(""));
    push(&content.process_id);
    push(
        &content
            .execution_latency_ms
            .map(|value| value.to_string())
            .unwrap_or_default(),
    );
    push(previous_audit_hash);

    buffer.into_bytes()
}

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una muestra con la
/// identidad, posición de secuencia y contenido dados, encadenada después
/// de `previous_audit_hash` (usa [`GENESIS_PREVIOUS_HASH`] para la primera
/// muestra de la cadena).
///
/// Determinista: los mismos argumentos siempre producen el mismo digest
/// (ADR-0002/0004) — sin I/O, sin reloj, sin azar dentro de esta función.
pub fn compute_sample_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    content: &TelemetrySampleContent,
    previous_audit_hash: &str,
) -> String {
    let bytes = canonical_bytes(id, created_at_ns, event_sequence_id, content, previous_audit_hash);

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Construye la siguiente [`TelemetrySample`] de la cadena (criterios #1 y
/// #2 — sirve tanto para una muestra de latencia como para un heartbeat: lo
/// único que cambia es `content.execution_latency_ms`).
///
/// `id` y `created_at_ns` los inyecta quien llama (la cáscara): `id` de un
/// generador de UUID, `created_at_ns` del puerto [`super::clock::Clock`].
/// Esta función no hace I/O ni azar propio — dados el mismo `id`,
/// `created_at_ns`, `content` y `previous`, siempre devuelve la misma
/// [`TelemetrySample`] (ADR-0002/0004).
///
/// `previous` es la última muestra encadenada conocida (la cáscara la
/// siembra al iniciar leyendo `telemetry_samples` una sola vez — ver el
/// comentario del módulo) o `None` para la primera muestra del proceso.
pub fn build_sample(
    id: String,
    created_at_ns: i64,
    content: TelemetrySampleContent,
    previous: Option<&TelemetrySample>,
) -> TelemetrySample {
    let (event_sequence_id, audit_chain_hash, previous_audit_hash) = match previous {
        Some(previous_sample) => (
            previous_sample.event_sequence_id + 1,
            Some(previous_sample.audit_hash.clone()),
            previous_sample.audit_hash.clone(),
        ),
        None => (1, None, GENESIS_PREVIOUS_HASH.to_string()),
    };

    let audit_hash = compute_sample_hash(&id, created_at_ns, event_sequence_id, &content, &previous_audit_hash);

    TelemetrySample {
        id,
        created_at_ns,
        updated_at_ns: created_at_ns,
        audit_hash,
        audit_chain_hash,
        event_sequence_id,
        content,
    }
}

/// Decide qué muestras quedan fuera de la ventana de retención dado un
/// corte de tiempo ya calculado (`cutoff_ns`) — la "poda" descrita en
/// `docs/features/telemetry.md` ("PODA AUTOMÁTICA").
///
/// Devuelve los `id` de las muestras con `created_at_ns < cutoff_ns`. Pura:
/// no decide qué es "ahora" ni qué es `RETENTION_DAYS` en días — eso lo
/// calcula quien llama (la cáscara, con el `Clock` inyectado) antes de
/// invocar esta función.
pub fn expired_sample_ids(samples: &[TelemetrySample], cutoff_ns: i64) -> Vec<String> {
    samples
        .iter()
        .filter(|sample| sample.created_at_ns < cutoff_ns)
        .map(|sample| sample.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn latency_content(metric_name: &str, execution_latency_ms: i64) -> TelemetrySampleContent {
        TelemetrySampleContent {
            metric_name: metric_name.to_string(),
            details_json: None,
            institutional_tag: "BACKTEST".to_string(),
            logic_hash: Some("shared-v1".to_string()),
            session_id: Some("session-1".to_string()),
            node_id: Some("node-1".to_string()),
            process_id: "process-1".to_string(),
            execution_latency_ms: Some(execution_latency_ms),
        }
    }

    fn heartbeat_content(metric_name: &str) -> TelemetrySampleContent {
        TelemetrySampleContent {
            metric_name: metric_name.to_string(),
            details_json: None,
            institutional_tag: "BACKTEST".to_string(),
            logic_hash: Some("shared-v1".to_string()),
            session_id: Some("session-1".to_string()),
            node_id: Some("node-1".to_string()),
            process_id: "process-1".to_string(),
            execution_latency_ms: None,
        }
    }

    /// Criterio #1: una muestra de latencia se construye en el núcleo puro,
    /// sin tocar reloj real ni disco — `created_at_ns` viene inyectado, no
    /// de `SystemTime::now()`.
    #[test]
    fn build_sample_constructs_a_latency_sample() {
        let content = latency_content("ingest.hot_path_latency", 7);
        let sample = build_sample("id-1".to_string(), 1_000, content, None);

        assert_eq!(sample.event_sequence_id, 1);
        assert_eq!(sample.audit_chain_hash, None);
        assert_eq!(sample.content.execution_latency_ms, Some(7));
        assert_eq!(sample.updated_at_ns, sample.created_at_ns);
    }

    /// Criterio #2: una muestra de heartbeat (sin valor de latencia) se
    /// construye correctamente — `execution_latency_ms` es `None`.
    #[test]
    fn build_sample_constructs_a_heartbeat_sample() {
        let content = heartbeat_content("job_executor.heartbeat");
        let sample = build_sample("id-1".to_string(), 1_000, content, None);

        assert_eq!(sample.content.execution_latency_ms, None);
        assert_eq!(sample.content.metric_name, "job_executor.heartbeat");
    }

    /// Determinismo (ADR-0002/0004): mismas entradas, mismo resultado bit a
    /// bit.
    #[test]
    fn build_sample_is_deterministic_given_same_inputs() {
        let sample_a = build_sample("fixed-id".to_string(), 1_000, latency_content("m", 5), None);
        let sample_b = build_sample("fixed-id".to_string(), 1_000, latency_content("m", 5), None);

        assert_eq!(sample_a, sample_b);
    }

    /// La segunda muestra de la cadena enlaza con la primera vía
    /// `audit_chain_hash == previous.audit_hash`, y el `event_sequence_id`
    /// avanza en exactamente 1.
    #[test]
    fn second_sample_chains_to_first() {
        let first = build_sample("id-1".to_string(), 1_000, heartbeat_content("m"), None);
        let second = build_sample("id-2".to_string(), 2_000, heartbeat_content("m"), Some(&first));

        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()));
        assert_ne!(second.audit_hash, first.audit_hash);
    }

    /// Criterio #6 (parte de núcleo): `expired_sample_ids` conserva las
    /// muestras dentro de la ventana y solo señala las más viejas que el
    /// corte.
    #[test]
    fn expired_sample_ids_returns_only_samples_older_than_cutoff() {
        let old = build_sample("old".to_string(), 1_000, heartbeat_content("m"), None);
        let recent = build_sample("recent".to_string(), 5_000, heartbeat_content("m"), Some(&old));

        let expired = expired_sample_ids(&[old, recent], 3_000);

        assert_eq!(expired, vec!["old".to_string()]);
    }

    /// Un corte que no excluye a nadie devuelve una lista vacía.
    #[test]
    fn expired_sample_ids_returns_empty_when_nothing_is_outside_the_window() {
        let sample = build_sample("id-1".to_string(), 5_000, heartbeat_content("m"), None);

        assert!(expired_sample_ids(&[sample], 1_000).is_empty());
    }
}