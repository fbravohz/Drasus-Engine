//! [CORE] Pure hash-chain logic for the Audit Log (`docs/features/audit-log.md`
//! TTR-001, ADR-0015, ADR-0020 V2, ADR-0027).
//!
//! No I/O, no system clock, no unseeded randomness (ADR-0002/0004). The
//! timestamp (`created_at_ns`) and identifier (`id`) are injected by the
//! shell (persistence layer) — the same way the `Clock` port is injected
//! elsewhere — so that, given the same inputs, [`chain_event`] and
//! [`verify_chain`] always produce the exact same output, bit for bit.
//!
//! ## Hash chain ("blockchain-lite", ADR-0020 V2 `audit_chain_hash`)
//!
//! Every [`AuditEvent`] carries:
//! - `audit_hash`: SHA-256 over this event's own content plus the previous
//!   event's `audit_hash` (or [`GENESIS_PREVIOUS_HASH`] for the first
//!   event in the chain).
//! - `audit_chain_hash`: the previous event's `audit_hash` (the actual
//!   link), `None` only for the genesis event.
//!
//! Mutating any historical event's content changes the SHA-256 input for
//! that event, which changes its `audit_hash`. Every subsequent event's
//! `audit_hash` was computed using the *original* `audit_hash` as the
//! `audit_chain_hash` link, so [`verify_chain`] recomputes each hash from
//! the (possibly tampered) content and detects the first link where the
//! recomputed hash no longer matches the stored `audit_chain_hash` of the
//! next event (or the stored `audit_hash` of the tampered event itself).

use sha2::{Digest, Sha256};

/// Placeholder used as the "previous hash" input when hashing the genesis
/// event (the first event ever appended to the chain, `event_sequence_id
/// == 1`). It has no corresponding row — it anchors the chain so the
/// genesis event's `audit_hash` is still derived deterministically from
/// "previous content".
pub const GENESIS_PREVIOUS_HASH: &str = "GENESIS";

/// Caller-supplied content of an audit event (`docs/features/audit-log.md`
/// TTR-001 "Entrada": `action_type`, `entity_type`, `entity_id`,
/// `details_json`, `process_id`; plus the mandatory ADR-0020 V2 fields for
/// the "Ops / Auditoría" profile: `institutional_tag` is required by
/// TTR-001, the rest of the Soberanía/Infraestructura group are optional
/// per event).
///
/// `details_json` is an opaque, already-serialized JSON string — the core
/// does not interpret it, only hashes it (Zero-Trust: validation of its
/// shape happens at the module boundary that produced it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventContent {
    // TTR-001 feature-specific fields (all mandatory:
    // "NUNCA un evento Audit falta campos obligatorios").
    pub action_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details_json: String,

    // ADR-0020 V2 "Ops / Auditoría" profile (Group II: Soberanía).
    pub owner_id: Option<String>,
    /// Mandatory per TTR-001 ("Toda entrada DEBE incluir ... `institutional_tag`").
    pub institutional_tag: String,
    pub manifest_id: Option<String>,
    pub access_token_id: Option<String>,

    // ADR-0020 V2 "Ops / Auditoría" profile (Group IV: Infraestructura / "Hardware").
    /// Mandatory per TTR-001 ("Toda entrada DEBE incluir `process_id` ...").
    pub process_id: String,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
}

/// A fully-chained audit event, ready to be appended to (or already stored
/// in) `audit_events`.
///
/// Field groups follow ADR-0020 V2's "Ops / Auditoría" profile (Group I,
/// universal + Group II Soberanía + Group IV Infraestructura), plus the
/// TTR-001 feature-specific fields carried in [`AuditEventContent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    // I. Identidad & Integridad (universal, ADR-0020 V2).
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    // TTR-001 + ADR-0020 V2 Groups II/IV content.
    pub content: AuditEventContent,
}

/// Builds the deterministic byte representation that gets hashed for a
/// given event. Exposed so [`chain_event`] and [`verify_chain`] compute the
/// exact same digest from the exact same logical inputs.
///
/// The representation concatenates every field that must be tamper-evident
/// — `id`, `created_at_ns`, `event_sequence_id`, all of
/// [`AuditEventContent`], and the previous link — using a field separator
/// (`\u{1F}`, ASCII Unit Separator) that cannot appear in any of the
/// string fields' normal usage, so distinct field combinations cannot
/// collide into the same byte stream.
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

/// Computes the SHA-256 `audit_hash` (hex-encoded, lowercase) for an event
/// with the given identity, sequence position and content, chained after
/// `previous_audit_hash` (use [`GENESIS_PREVIOUS_HASH`] for the first event
/// in the chain).
///
/// Deterministic: the same arguments always produce the same digest
/// (ADR-0002/0004) — no I/O, no clock, no randomness inside this function.
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

    // Lowercase hex encoding.
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Builds the next [`AuditEvent`] in the chain.
///
/// `id` and `created_at_ns` are injected by the caller (shell): `id` comes
/// from a UUID generator, `created_at_ns` from the [`super::clock::Clock`]
/// port. This function performs no I/O and no randomness of its own —
/// given the same `id`, `created_at_ns`, `event_sequence_id`, `content`
/// and `previous` it always returns the same [`AuditEvent`]
/// (ADR-0002/0004).
///
/// `previous` is the most recently appended event in the chain (or `None`
/// for the genesis event, `event_sequence_id == 1`). The new event's
/// `audit_chain_hash` becomes `previous.audit_hash` (or `None` for
/// genesis), and `event_sequence_id` is `previous.event_sequence_id + 1`
/// (or `1` for genesis).
///
/// For an append-only log, `updated_at_ns` always equals `created_at_ns`:
/// the event is never modified after creation.
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

/// Outcome of [`verify_chain`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainVerificationResult {
    /// Every event's `audit_hash` matches its recomputed digest, and every
    /// `audit_chain_hash` correctly links to the previous event's
    /// `audit_hash`. The chain has not been tampered with.
    Valid,
    /// The chain is broken starting at the event with this
    /// `event_sequence_id` — either its own `audit_hash` no longer matches
    /// its (current) content, or its `audit_chain_hash` no longer matches
    /// the previous event's `audit_hash`. This is the first broken link;
    /// everything from this point onward is untrustworthy.
    Broken { event_sequence_id: i64 },
}

/// Recomputes the hash chain over `events` (which MUST be ordered by
/// ascending `event_sequence_id`, starting at `1` with no gaps) and reports
/// whether it is intact.
///
/// For each event:
/// 1. Recompute `audit_hash` from its current (possibly tampered) content
///    using [`compute_audit_hash`] with the *previous event's stored*
///    `audit_hash` as the link. If the recomputed digest differs from the
///    stored `audit_hash`, the event's own content was altered ->
///    [`ChainVerificationResult::Broken`].
/// 2. Check that `audit_chain_hash` equals the previous event's
///    `audit_hash` (or `None` for the genesis event). If it does not, the
///    chain link itself was altered -> [`ChainVerificationResult::Broken`].
///
/// An empty slice is trivially [`ChainVerificationResult::Valid`].
pub fn verify_chain(events: &[AuditEvent]) -> ChainVerificationResult {
    let mut previous_audit_hash = GENESIS_PREVIOUS_HASH.to_string();
    let mut previous_stored_hash: Option<&str> = None;

    for event in events {
        // Check 1: the chain link must point at the previous event's
        // stored audit_hash (None only for the genesis event).
        let expected_chain_hash = previous_stored_hash.map(str::to_string);
        if event.audit_chain_hash != expected_chain_hash {
            return ChainVerificationResult::Broken {
                event_sequence_id: event.event_sequence_id,
            };
        }

        // Check 2: the event's own audit_hash must match its content,
        // hashed together with the previous event's audit_hash.
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

    /// Two independent runs with the same inputs must produce bit-for-bit
    /// identical events (ADR-0002/0004 determinism).
    #[test]
    fn chain_event_is_deterministic_given_same_inputs() {
        let content = sample_content("a");

        let event_a = chain_event("fixed-id".to_string(), 1_000, content.clone(), None);
        let event_b = chain_event("fixed-id".to_string(), 1_000, content, None);

        assert_eq!(event_a, event_b);
    }

    /// Genesis event (first in the chain): `audit_chain_hash` is `None`,
    /// `event_sequence_id` is 1, and `updated_at_ns == created_at_ns`
    /// (append-only: never modified after creation).
    #[test]
    fn genesis_event_has_no_chain_link_and_sequence_one() {
        let content = sample_content("genesis");
        let event = chain_event("id-genesis".to_string(), 1_000, content, None);

        assert_eq!(event.event_sequence_id, 1);
        assert_eq!(event.audit_chain_hash, None);
        assert_eq!(event.updated_at_ns, event.created_at_ns);
        assert!(!event.audit_hash.is_empty());
    }

    /// The second event in the chain links to the first via
    /// `audit_chain_hash == previous.audit_hash`, and its `event_sequence_id`
    /// increments by exactly 1.
    #[test]
    fn second_event_chains_to_first() {
        let genesis = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let second = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&genesis));

        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(second.audit_chain_hash, Some(genesis.audit_hash.clone()));
        assert_ne!(second.audit_hash, genesis.audit_hash);
    }

    /// A 3-event chain, recomputed with [`verify_chain`], reports
    /// [`ChainVerificationResult::Valid`] when nothing was tampered with.
    #[test]
    fn verify_chain_accepts_untampered_chain() {
        let event_1 = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let event_2 = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&event_1));
        let event_3 = chain_event("id-3".to_string(), 3_000, sample_content("3"), Some(&event_2));

        let chain = vec![event_1, event_2, event_3];

        assert_eq!(verify_chain(&chain), ChainVerificationResult::Valid);
    }

    /// CLOSING CRITERION: mutating a historical event's content (here,
    /// `details_json` of the first event in a 3-event chain) is detected
    /// by [`verify_chain`] — it reports
    /// [`ChainVerificationResult::Broken`] at the tampered event's
    /// `event_sequence_id`, even though `audit_chain_hash` pointers were
    /// left untouched.
    #[test]
    fn verify_chain_detects_mutation_of_historical_event() {
        let event_1 = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let event_2 = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&event_1));
        let event_3 = chain_event("id-3".to_string(), 3_000, sample_content("3"), Some(&event_2));

        let mut tampered_chain = vec![event_1, event_2, event_3];

        // Simulate an attacker (or a buggy "edit" path) rewriting the
        // first event's details after the fact, without recomputing any
        // hash.
        tampered_chain[0].content.details_json =
            "{\"from\":\"NEW\",\"to\":\"CANCELLED\",\"seq\":\"1\"}".to_string();

        assert_eq!(
            verify_chain(&tampered_chain),
            ChainVerificationResult::Broken { event_sequence_id: 1 },
            "mutating event #1's content must break the chain at #1"
        );
    }

    /// Tampering with the *link* (`audit_chain_hash`) instead of the
    /// content is also detected, at the event whose link was rewritten.
    #[test]
    fn verify_chain_detects_tampered_chain_link() {
        let event_1 = chain_event("id-1".to_string(), 1_000, sample_content("1"), None);
        let event_2 = chain_event("id-2".to_string(), 2_000, sample_content("2"), Some(&event_1));

        let mut tampered_chain = vec![event_1, event_2];

        // Rewrite event 2's chain link to point at a hash that does not
        // belong to event 1.
        tampered_chain[1].audit_chain_hash = Some("not-a-real-hash".to_string());

        assert_eq!(
            verify_chain(&tampered_chain),
            ChainVerificationResult::Broken { event_sequence_id: 2 }
        );
    }

    /// Swapping the order of two events (replay/reorder attack) is
    /// detected: `event_sequence_id` ordering and chain links no longer
    /// match the recomputed hashes.
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

    /// An empty chain is trivially valid (nothing to verify).
    #[test]
    fn verify_chain_accepts_empty_chain() {
        assert_eq!(verify_chain(&[]), ChainVerificationResult::Valid);
    }
}
