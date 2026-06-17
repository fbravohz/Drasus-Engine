//! [CORE] Lógica pura de la cadena de hashes del Audit Log
//! (`docs/features/audit-log.md` TTR-001, ADR-0015, ADR-0020 V2, ADR-0027).
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004). La
//! marca de tiempo (`created_at_ns`) y el identificador (`id`) los inyecta
//! la cáscara (capa de persistencia) — igual que se inyecta el puerto
//! `Clock` en otras partes — para que, con las mismas entradas,
//! [`chain_event`] y [`verify_chain`] siempre produzcan exactamente el
//! mismo resultado, byte por byte.
//!
//! ## Cadena de hashes ("blockchain-lite", ADR-0020 V2 `audit_chain_hash`)
//!
//! Cada [`AuditEvent`] guarda dos hashes:
//! - `audit_hash`: SHA-256 sobre el contenido propio de este evento más el
//!   `audit_hash` del evento anterior (o [`GENESIS_PREVIOUS_HASH`] si es el
//!   primer evento de la cadena).
//! - `audit_chain_hash`: el `audit_hash` del evento anterior — el enlace
//!   real de la cadena. Es `None` únicamente en el evento génesis.
//!
//! Si alguien modifica el contenido de un evento histórico, cambia la
//! entrada del SHA-256 de ese evento y, por lo tanto, cambia su
//! `audit_hash`. Pero el `audit_hash` de TODOS los eventos posteriores se
//! calculó usando el `audit_hash` *original* (sin modificar) como enlace
//! `audit_chain_hash`. Por eso [`verify_chain`] puede detectar la
//! manipulación: recalcula cada hash a partir del contenido (posiblemente
//! alterado) y encuentra el primer punto donde el hash recalculado ya no
//! coincide con el `audit_chain_hash` guardado en el siguiente evento (o
//! con el `audit_hash` guardado del propio evento alterado).

use sha2::{Digest, Sha256};

/// Valor de relleno usado como "hash anterior" al calcular el hash del
/// evento génesis (el primer evento de la cadena, `event_sequence_id ==
/// 1`). No corresponde a ninguna fila real — solo ancla la cadena, para
/// que el `audit_hash` del evento génesis también se derive de forma
/// determinista a partir de un "contenido anterior".
pub const GENESIS_PREVIOUS_HASH: &str = "GENESIS";

/// Contenido de un evento de auditoría, provisto por quien llama
/// (`docs/features/audit-log.md` TTR-001 "Entrada": `action_type`,
/// `entity_type`, `entity_id`, `details_json`, `process_id`; más los
/// campos obligatorios de ADR-0020 V2 para el perfil "Ops / Auditoría":
/// `institutional_tag` es obligatorio según TTR-001; el resto del grupo
/// Soberanía/Infraestructura es opcional, evento por evento).
///
/// `details_json` es una cadena JSON opaca, ya serializada — el núcleo no
/// la interpreta, solo la incluye en el hash (Zero-Trust: la validación de
/// su forma ocurre en el módulo que la produjo, no aquí).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventContent {
    // Campos propios de la feature TTR-001 (todos obligatorios:
    // "NUNCA un evento Audit falta campos obligatorios").
    pub action_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details_json: String,

    // Perfil "Ops / Auditoría" de ADR-0020 V2 (Grupo II: Soberanía).
    pub owner_id: Option<String>,
    /// Obligatorio según TTR-001 ("Toda entrada DEBE incluir ... `institutional_tag`").
    pub institutional_tag: String,
    pub manifest_id: Option<String>,
    pub access_token_id: Option<String>,

    // Perfil "Ops / Auditoría" de ADR-0020 V2 (Grupo IV: Infraestructura / "Hardware").
    /// Obligatorio según TTR-001 ("Toda entrada DEBE incluir `process_id` ...").
    pub process_id: String,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
}

/// Un evento de auditoría ya encadenado, listo para agregarse a
/// `audit_events` (o ya almacenado ahí).
///
/// Los grupos de campos siguen el perfil "Ops / Auditoría" de ADR-0020 V2
/// (Grupo I universal + Grupo II Soberanía + Grupo IV Infraestructura),
/// más los campos propios de la feature TTR-001 que vienen en
/// [`AuditEventContent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    // I. Identidad & Integridad (universal, ADR-0020 V2).
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    // Contenido de TTR-001 + Grupos II/IV de ADR-0020 V2.
    pub content: AuditEventContent,
}

/// Construye la representación determinista en bytes que se hashea para un
/// evento dado. Se expone para que [`chain_event`] y [`verify_chain`]
/// calculen exactamente el mismo digest a partir de las mismas entradas
/// lógicas.
///
/// La representación concatena todos los campos que deben quedar a prueba
/// de manipulación — `id`, `created_at_ns`, `event_sequence_id`, todo
/// [`AuditEventContent`] y el enlace al evento anterior — usando un
/// separador de campo (`\u{1F}`, el carácter ASCII "Unit Separator") que
/// no puede aparecer en el uso normal de ningún campo de texto. Así, dos
/// combinaciones de campos distintas nunca colisionan en el mismo flujo de
/// bytes.
fn canonical_bytes(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    content: &AuditEventContent,
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
    push(&content.action_type);
    push(&content.entity_type);
    push(&content.entity_id);
    push(&content.details_json);
    push(content.owner_id.as_deref().unwrap_or(""));
    push(&content.institutional_tag);
    push(content.manifest_id.as_deref().unwrap_or(""));
    push(content.access_token_id.as_deref().unwrap_or(""));
    push(&content.process_id);
    push(content.session_id.as_deref().unwrap_or(""));
    push(content.node_id.as_deref().unwrap_or(""));
    push(previous_audit_hash);

    buffer.into_bytes()
}

/// Calcula el `audit_hash` SHA-256 (codificado en hexadecimal, en
/// minúsculas) de un evento con la identidad, posición en la secuencia y
/// contenido dados, encadenado después de `previous_audit_hash` (usa
/// [`GENESIS_PREVIOUS_HASH`] para el primer evento de la cadena).
///
/// Determinista: los mismos argumentos siempre producen el mismo digest
/// (ADR-0002/0004) — esta función no hace I/O, no usa el reloj ni azar.
pub fn compute_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    content: &AuditEventContent,
    previous_audit_hash: &str,
) -> String {
    let bytes = canonical_bytes(id, created_at_ns, event_sequence_id, content, previous_audit_hash);

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();

    // Codificación hexadecimal en minúsculas.
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Construye el siguiente [`AuditEvent`] de la cadena.
///
/// `id` y `created_at_ns` los inyecta quien llama (la cáscara): `id` viene
/// de un generador de UUID, `created_at_ns` del puerto
/// [`super::clock::Clock`]. Esta función no hace I/O ni usa azar propio —
/// con el mismo `id`, `created_at_ns`, `event_sequence_id`, `content` y
/// `previous`, siempre devuelve el mismo [`AuditEvent`] (ADR-0002/0004).
///
/// `previous` es el evento agregado más recientemente a la cadena (o
/// `None` si este es el evento génesis, `event_sequence_id == 1`). El
/// nuevo evento toma `audit_chain_hash = previous.audit_hash` (o `None`
/// en el génesis), y `event_sequence_id = previous.event_sequence_id + 1`
/// (o `1` en el génesis).
///
/// Al ser un log de solo-apéndice (append-only), `updated_at_ns` siempre
/// es igual a `created_at_ns`: el evento nunca se modifica después de
/// creado.
pub fn chain_event(
    id: String,
    created_at_ns: i64,
    content: AuditEventContent,
    previous: Option<&AuditEvent>,
) -> AuditEvent {
    let (event_sequence_id, audit_chain_hash, previous_audit_hash) = match previous {
        Some(previous_event) => (
            previous_event.event_sequence_id + 1,
            Some(previous_event.audit_hash.clone()),
            previous_event.audit_hash.clone(),
        ),
        None => (1, None, GENESIS_PREVIOUS_HASH.to_string()),
    };

    let audit_hash = compute_audit_hash(
        &id,
        created_at_ns,
        event_sequence_id,
        &content,
        &previous_audit_hash,
    );

    AuditEvent {
        id,
        created_at_ns,
        updated_at_ns: created_at_ns,
        audit_hash,
        audit_chain_hash,
        event_sequence_id,
        content,
    }
}

/// Resultado de [`verify_chain`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainVerificationResult {
    /// El `audit_hash` de cada evento coincide con su digest recalculado,
    /// y cada `audit_chain_hash` enlaza correctamente con el `audit_hash`
    /// del evento anterior. La cadena no fue manipulada.
    Valid,
    /// La cadena está rota a partir del evento con este
    /// `event_sequence_id` — ya sea porque su propio `audit_hash` ya no
    /// coincide con su contenido (actual), o porque su `audit_chain_hash`
    /// ya no coincide con el `audit_hash` del evento anterior. Este es el
    /// primer enlace roto; todo lo que sigue desde aquí no es confiable.
    Broken { event_sequence_id: i64 },
}

/// Recalcula la cadena de hashes sobre `events` (que DEBE venir ordenado
/// por `event_sequence_id` ascendente, empezando en `1` y sin huecos) e
/// informa si está intacta.
///
/// Para cada evento:
/// 1. Recalcula `audit_hash` a partir de su contenido actual (posiblemente
///    alterado), usando [`compute_audit_hash`] con el `audit_hash`
///    *guardado* del evento anterior como enlace. Si el digest recalculado
///    difiere del `audit_hash` guardado, el contenido propio del evento
///    fue alterado -> [`ChainVerificationResult::Broken`].
/// 2. Verifica que `audit_chain_hash` sea igual al `audit_hash` del evento
///    anterior (o `None` en el evento génesis). Si no lo es, el enlace de
///    la cadena en sí fue alterado -> [`ChainVerificationResult::Broken`].
///
/// Un slice vacío es trivialmente [`ChainVerificationResult::Valid`].
pub fn verify_chain(events: &[AuditEvent]) -> ChainVerificationResult {
    let mut previous_audit_hash = GENESIS_PREVIOUS_HASH.to_string();
    let mut previous_stored_hash: Option<&str> = None;

    for event in events {
        // Verificación 1: el enlace de la cadena debe apuntar al
        // audit_hash guardado del evento anterior (None solo en el evento
        // génesis).
        let expected_chain_hash = previous_stored_hash.map(str::to_string);
        if event.audit_chain_hash != expected_chain_hash {
            return ChainVerificationResult::Broken {
                event_sequence_id: event.event_sequence_id,
            };
        }

        // Verificación 2: el audit_hash propio del evento debe coincidir
        // con su contenido, hasheado junto con el audit_hash del evento
        // anterior.
        let recomputed = compute_audit_hash(
            &event.id,
            event.created_at_ns,
            event.event_sequence_id,
            &event.content,
            &previous_audit_hash,
        );

        if recomputed != event.audit_hash {
            return ChainVerificationResult::Broken {
                event_sequence_id: event.event_sequence_id,
            };
        }

        previous_audit_hash = event.audit_hash.clone();
        previous_stored_hash = Some(&event.audit_hash);
    }

    ChainVerificationResult::Valid
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_content(suffix: &str) -> AuditEventContent {
        AuditEventContent {
            action_type: format!("ORDER_STATE_CHANGE_{suffix}"),
            entity_type: "ORDER".to_string(),
            entity_id: format!("order-{suffix}"),
            details_json: format!("{{\"from\":\"NEW\",\"to\":\"FILLED\",\"seq\":\"{suffix}\"}}"),
            owner_id: Some("owner-1".to_string()),
            institutional_tag: "BACKTEST".to_string(),
            manifest_id: None,
            access_token_id: None,
            process_id: "process-1".to_string(),
            session_id: Some("session-1".to_string()),
            node_id: None,
        }
    }

    /// Dos ejecuciones independientes con las mismas entradas deben
    /// producir eventos idénticos byte a byte (determinismo de
    /// ADR-0002/0004).
    #[test]
    fn chain_event_is_deterministic_given_same_inputs() {
        let content = sample_content("a");

        let event_a = chain_event("fixed-id".to_string(), 1_000, content.clone(), None);
        let event_b = chain_event("fixed-id".to_string(), 1_000, content, None);

        assert_eq!(event_a, event_b);
    }

    /// Evento génesis (el primero de la cadena): `audit_chain_hash` es
    /// `None`, `event_sequence_id` es 1, y `updated_at_ns ==
    /// created_at_ns` (solo-apéndice: nunca se modifica después de
    /// creado).
    #[test]
    fn genesis_event_has_no_chain_link_and_sequence_one() {
        let content = sample_content("genesis");
        let event = chain_event("id-genesis".to_string(), 1_000, content, None);

        assert_eq!(event.event_sequence_id, 1);
        assert_eq!(event.audit_chain_hash, None);
        assert_eq!(event.updated_at_ns, event.created_at_ns);
        assert!(!event.audit_hash.is_empty());
    }

    /// El segundo evento de la cadena enlaza con el primero mediante
    /// `audit_chain_hash == previous.audit_hash`, y su
    /// `event_sequence_id` se incrementa en exactamente 1.
    #[test]
    fn second_event_chains_to_first() {
        let genesis = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let second = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&genesis));

        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(second.audit_chain_hash, Some(genesis.audit_hash.clone()));
        assert_ne!(second.audit_hash, genesis.audit_hash);
    }

    /// Una cadena de 3 eventos, recalculada con [`verify_chain`], reporta
    /// [`ChainVerificationResult::Valid`] cuando nada fue manipulado.
    #[test]
    fn verify_chain_accepts_untampered_chain() {
        let event_1 = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let event_2 = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&event_1));
        let event_3 = chain_event("id-3".to_string(), 3_000, sample_content("3"), Some(&event_2));

        let chain = vec![event_1, event_2, event_3];

        assert_eq!(verify_chain(&chain), ChainVerificationResult::Valid);
    }

    /// CRITERIO DE CIERRE: modificar el contenido de un evento histórico
    /// (aquí, el `details_json` del primer evento de una cadena de 3) lo
    /// detecta [`verify_chain`] — reporta
    /// [`ChainVerificationResult::Broken`] en el `event_sequence_id` del
    /// evento manipulado, aunque los punteros `audit_chain_hash` se hayan
    /// dejado intactos.
    #[test]
    fn verify_chain_detects_mutation_of_historical_event() {
        let event_1 = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let event_2 = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&event_1));
        let event_3 = chain_event("id-3".to_string(), 3_000, sample_content("3"), Some(&event_2));

        let mut tampered_chain = vec![event_1, event_2, event_3];

        // Simula a un atacante (o un camino de "edición" con un defecto)
        // que reescribe los detalles del primer evento después del hecho,
        // sin recalcular ningún hash.
        tampered_chain[0].content.details_json =
            "{\"from\":\"NEW\",\"to\":\"CANCELLED\",\"seq\":\"1\"}".to_string();

        assert_eq!(
            verify_chain(&tampered_chain),
            ChainVerificationResult::Broken { event_sequence_id: 1 },
            "mutating event #1's content must break the chain at #1"
        );
    }

    /// Manipular el *enlace* (`audit_chain_hash`) en vez del contenido
    /// también se detecta, en el evento cuyo enlace fue reescrito.
    #[test]
    fn verify_chain_detects_tampered_chain_link() {
        let event_1 = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let event_2 = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&event_1));

        let mut tampered_chain = vec![event_1, event_2];

        // Reescribe el enlace de la cadena del evento 2 para que apunte a
        // un hash que no le pertenece al evento 1.
        tampered_chain[1].audit_chain_hash = Some("not-a-real-hash".to_string());

        assert_eq!(
            verify_chain(&tampered_chain),
            ChainVerificationResult::Broken { event_sequence_id: 2 }
        );
    }

    /// Intercambiar el orden de dos eventos (ataque de
    /// repetición/reordenamiento) también se detecta: el orden de
    /// `event_sequence_id` y los enlaces de la cadena ya no coinciden con
    /// los hashes recalculados.
    #[test]
    fn verify_chain_detects_reordered_events() {
        let event_1 = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let event_2 = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&event_1));

        let reordered = vec![event_2, event_1];

        match verify_chain(&reordered) {
            ChainVerificationResult::Broken { .. } => {}
            ChainVerificationResult::Valid => panic!("reordered chain must not verify as valid"),
        }
    }

    /// Una cadena vacía es trivialmente válida (no hay nada que
    /// verificar).
    #[test]
    fn verify_chain_accepts_empty_chain() {
        assert_eq!(verify_chain(&[]), ChainVerificationResult::Valid);
    }
}
