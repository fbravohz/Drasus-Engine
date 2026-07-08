//! [CORE] Lógica pura de la Jerarquía de Cuenta Maestra
//! (`docs/features/master-account-hierarchy.md`, ADR-0147 -- cimiento #12
//! rector, ADR-0143, ADR-0141, ADR-0093, ADR-0020, ADR-0002, ADR-0137,
//! STORY-040).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Cierra el substrato de monetización (12/12): una
//! cuenta maestra raíz (fondo) agrupa N cuentas maestras hijas, con
//! autoridad de auditoría y override sobre cada una -- pero el mando NUNCA
//! escribe directo en la base de datos de la hija, y todo override exige
//! consentimiento vigente y queda doblemente atestado (fondo + hija).
//!
//! ## Las seis reglas fijas de ADR-0147, y dónde vive cada una en este módulo
//!
//! 1. **Jerarquía central = puntero, no árbol** -- este módulo no modela
//!    ningún tipo "árbol completo"; solo produce el hash de UNA relación
//!    padre-hija a la vez ([`compute_hierarchy_audit_hash`]). El anti-
//!    `tenant_id` se cumple estructuralmente: no existe forma de consultar
//!    "todas las hijas de un fondo" desde este Core.
//! 2. **Canal de mando elevado** -- el adaptador de red del relé es
//!    diferido (`docs/features/master-account-hierarchy.md`); este módulo
//!    solo decide y hashea, nunca transmite.
//! 3. **Consentimiento contractual** -- [`decide_override_authorization`]
//!    es la ÚNICA puerta: `Executed` solo si el `ConsentVerdict` REAL de
//!    `consent-registry` (#5) es `Covered`.
//! 4. **Doble atestación** -- este módulo no decide CUÁNTAS filas se
//!    escriben (eso es la Shell, ver `orchestrator::master_account_hierarchy`),
//!    pero SÍ fija el vocabulario ([`AttestationSide::Issuer`] /
//!    [`AttestationSide::Executor`]) que hace la doble atestación posible
//!    de distinguir.
//! 5. **"Eliminar" = archivar** -- [`apply_local_command_effect`] SOLO
//!    puede devolver [`LocalEffect::Archived`] (una transición de estado)
//!    o [`LocalEffect::NoEffect`]; no existe ninguna variante de borrado
//!    físico en este catálogo.
//! 6. **La hija conserva su Plano de Control** -- esta capa no reemplaza
//!    nada de `central-identity`/`consent-registry`; solo los CONSUME
//!    (import de [`ConsentVerdict`], nunca redefinición).

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::domain::consent_registry::ConsentVerdict;

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `verified_account_registry::encode_hex` /
/// `instance_continuity::sha256_hex`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Catálogo de comandos de override (columna `command_kind`) ──────────────

/// Los comandos que un fondo puede ordenar sobre una hija -- catálogo
/// `OVERRIDE_COMMANDS` (`docs/features/master-account-hierarchy.md`
/// "Parámetros Configurables", ADR-0008: catálogo cerrado y auditable, no
/// una lista libre de texto). Los tres son EXHAUSTIVOS: cualquier mando
/// elevado que el fondo emita cae en uno de estos tres, nunca en un texto
/// arbitrario que la migración no pueda validar con su `CHECK`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverrideCommandKind {
    /// "Eliminar" en el vocabulario del fondo -- SIEMPRE se traduce a
    /// archivar (regla fija #5), nunca a un DELETE físico.
    Archive,
    /// Modificar un parámetro del recurso referenciado por `target_ref`.
    Modify,
    /// Pedir que la hija emita/entregue un reporte de auditoría -- no
    /// muta ningún estado, solo produce evidencia.
    RequestAuditReport,
}

impl OverrideCommandKind {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (command_kind IN (...))` de la migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            OverrideCommandKind::Archive => "ARCHIVE",
            OverrideCommandKind::Modify => "MODIFY",
            OverrideCommandKind::RequestAuditReport => "REQUEST_AUDIT_REPORT",
        }
    }

    /// Reconstruye el comando desde su representación en texto, o `None`
    /// si no es ninguno de los tres reconocidos (integridad de datos, o
    /// input externo del harness CLI).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "ARCHIVE" => Some(OverrideCommandKind::Archive),
            "MODIFY" => Some(OverrideCommandKind::Modify),
            "REQUEST_AUDIT_REPORT" => Some(OverrideCommandKind::RequestAuditReport),
            _ => None,
        }
    }
}

// ── Lado de la atestación (columna `attestation_side`) ──────────────────────

/// Qué extremo de la doble atestación produjo UNA fila de
/// `override_attestations` (regla fija #4, ADR-0147): el fondo encadena
/// "emití esta orden" ([`AttestationSide::Issuer`]), la hija encadena
/// "recibí esta orden firmada por mi padre y la ejecuté (o la rechacé)"
/// ([`AttestationSide::Executor`]). Nunca hay una mutación silenciosa: toda
/// orden produce EXACTAMENTE una fila de cada lado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttestationSide {
    Issuer,
    Executor,
}

impl AttestationSide {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (attestation_side IN (...))` de la migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            AttestationSide::Issuer => "ISSUER",
            AttestationSide::Executor => "EXECUTOR",
        }
    }

    /// Reconstruye el lado desde su representación en texto, o `None` si
    /// no es ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "ISSUER" => Some(AttestationSide::Issuer),
            "EXECUTOR" => Some(AttestationSide::Executor),
            _ => None,
        }
    }
}

// ── Etiqueta persistida del desenlace (columna `outcome`) ───────────────────

/// La etiqueta de dos valores que se persiste en la columna `outcome` --
/// deliberadamente SIN la razón de denegación (esa vive en
/// [`OverrideOutcome::Denied`], nunca en esta columna restringida por el
/// `CHECK` de la migración). Se deriva SIEMPRE de [`OverrideOutcome`] vía
/// [`OverrideOutcomeLabel::from`], nunca se construye por separado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverrideOutcomeLabel {
    Executed,
    Denied,
}

impl OverrideOutcomeLabel {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (outcome IN (...))` de la migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            OverrideOutcomeLabel::Executed => "EXECUTED",
            OverrideOutcomeLabel::Denied => "DENIED",
        }
    }

    /// Reconstruye la etiqueta desde su representación en texto, o `None`
    /// si no es ninguna de las dos reconocidas.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "EXECUTED" => Some(OverrideOutcomeLabel::Executed),
            "DENIED" => Some(OverrideOutcomeLabel::Denied),
            _ => None,
        }
    }
}

impl From<&OverrideOutcome> for OverrideOutcomeLabel {
    /// Deriva SIEMPRE la etiqueta persistida desde el desenlace real del
    /// gate -- nunca hay una etiqueta que pueda desincronizarse del
    /// desenlace que la produjo.
    fn from(outcome: &OverrideOutcome) -> Self {
        match outcome {
            OverrideOutcome::Executed => OverrideOutcomeLabel::Executed,
            OverrideOutcome::Denied(_) => OverrideOutcomeLabel::Denied,
        }
    }
}

// ── Gate de autorización (regla fija #3, EL punto de correctitud legal) ────

/// El desenlace de intentar UN override -- `Executed` (el gate lo permitió)
/// o `Denied(razón)` (el gate lo bloqueó, con la razón exacta que
/// [`ConsentVerdict::NotCovered`] trae, nunca un booleano ciego).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverrideOutcome {
    Executed,
    Denied(String),
}

/// Decide si un override puede ejecutarse -- EL punto de correctitud legal
/// de este cimiento (ADR-0147 regla fija #3): `Executed` **solo si**
/// `consent.is_covered()` es verdadero, usando el `ConsentVerdict` REAL
/// resuelto por quien llama contra `consent-registry` (#5) ANTES de invocar
/// esta función pura -- este módulo NUNCA consulta la base de datos por su
/// cuenta ni asume cobertura por defecto. Sin opt-in vigente, el override
/// se deniega SIEMPRE, sin excepción, y ambos lados (fondo e hija) atestan
/// el intento denegado (nunca se descarta en silencio).
pub fn decide_override_authorization(consent: &ConsentVerdict) -> OverrideOutcome {
    match consent {
        ConsentVerdict::Covered => OverrideOutcome::Executed,
        ConsentVerdict::NotCovered(reason) => OverrideOutcome::Denied(format!("{reason:?}")),
    }
}

// ── "Eliminar" = archivar (regla fija #5, ADR-0141) ─────────────────────────

/// El efecto LOCAL de ejecutar un comando -- deliberadamente un catálogo
/// CERRADO de dos valores, ninguno de los cuales es un borrado físico
/// (regla fija #5, ADR-0147/ADR-0141: "eliminar" SIEMPRE es archivar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalEffect {
    /// El comando `ARCHIVE` se ejecutó: el recurso referenciado por
    /// `target_ref` queda marcado como archivado/desactivado -- una
    /// transición de estado, NUNCA una fila borrada.
    Archived,
    /// Ningún efecto local aplicado -- el comando no era `ARCHIVE`
    /// (`MODIFY`/`REQUEST_AUDIT_REPORT` no archivan nada por sí mismos), o
    /// el gate de consentimiento denegó el override.
    NoEffect,
}

/// Decide el efecto LOCAL de ejecutar `command_kind` dado el `outcome` ya
/// resuelto por [`decide_override_authorization`] -- pura, sin I/O.
/// Estructuralmente IMPOSIBLE que devuelva algo distinto de
/// [`LocalEffect::Archived`] o [`LocalEffect::NoEffect`]: el catálogo de
/// salida no tiene una tercera variante de borrado que un defecto pudiera
/// alcanzar.
pub fn apply_local_command_effect(command_kind: OverrideCommandKind, outcome: &OverrideOutcome) -> LocalEffect {
    match (command_kind, outcome) {
        (OverrideCommandKind::Archive, OverrideOutcome::Executed) => LocalEffect::Archived,
        _ => LocalEffect::NoEffect,
    }
}

// ── Hash de auditoría de `override_attestations` (APPEND-ONLY) ─────────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de UNA fila de
/// `override_attestations`, encadenado al `audit_hash` de la fila anterior
/// en la secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis). Mismo estilo (buffer separado por bytes de
/// control, nunca JSON) que
/// [`crate::domain::verified_account_registry::compute_track_record_audit_hash`]
/// -- protege la integridad DE LA FILA en el ledger append-only.
#[allow(clippy::too_many_arguments)]
pub fn compute_override_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    parent_owner_id: &str,
    node_id: &str,
    attestation_side: &str,
    command_kind: &str,
    target_ref: &str,
    outcome: &str,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(previous_audit_hash);
    push(owner_id);
    push(parent_owner_id);
    push(node_id);
    push(attestation_side);
    push(command_kind);
    push(target_ref);
    push(outcome);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Hash de auditoría de `account_hierarchy` (MUTABLE) ──────────────────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `account_hierarchy`, encadenado a la versión anterior de la MISMA
/// fila (`previous_audit_hash: None` en la versión génesis, `row_version ==
/// 1`). Mismo patrón que
/// [`crate::domain::verified_account_registry::compute_verified_account_audit_hash`]
/// -- tabla MUTABLE, se encadena por versión de fila, no por secuencia
/// global.
#[allow(clippy::too_many_arguments)]
pub fn compute_hierarchy_audit_hash(
    id: &str,
    created_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    parent_owner_id: Option<&str>,
    consent_ref: &str,
    node_id: &str,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(""));
    push(owner_id);
    // NULL (sin padre todavía) se encadena como cadena vacía -- idéntico
    // criterio que `broker_connection_ref` en `verified_account_registry`.
    push(parent_owner_id.unwrap_or(""));
    push(consent_ref);
    push(node_id);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::consent_registry::NotCoveredReason;

    // ── Enums: round-trip de representación en texto ────────────────────────

    #[test]
    fn override_command_kind_round_trips_through_its_string_representation() {
        for variant in [
            OverrideCommandKind::Archive,
            OverrideCommandKind::Modify,
            OverrideCommandKind::RequestAuditReport,
        ] {
            assert_eq!(OverrideCommandKind::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(OverrideCommandKind::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn attestation_side_round_trips_through_its_string_representation() {
        for variant in [AttestationSide::Issuer, AttestationSide::Executor] {
            assert_eq!(AttestationSide::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(AttestationSide::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn override_outcome_label_round_trips_through_its_string_representation() {
        for variant in [OverrideOutcomeLabel::Executed, OverrideOutcomeLabel::Denied] {
            assert_eq!(OverrideOutcomeLabel::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(OverrideOutcomeLabel::from_str_value("UNKNOWN"), None);
    }

    // ── CRITERIO: gate de consentimiento (regla fija #3) ─────────────────────

    /// CRITERIO DE CIERRE: consentimiento cubierto -> SIEMPRE ejecutado.
    #[test]
    fn decide_override_authorization_executes_only_with_covered_consent() {
        let outcome = decide_override_authorization(&ConsentVerdict::Covered);
        assert_eq!(outcome, OverrideOutcome::Executed);
    }

    /// CRITERIO DE CIERRE: sin consentimiento vigente, el override se
    /// deniega SIEMPRE -- nunca se ejecuta "a medias" ni por defecto.
    #[test]
    fn decide_override_authorization_denies_without_covered_consent() {
        let not_covered = ConsentVerdict::NotCovered(NotCoveredReason::NoConsent);
        let outcome = decide_override_authorization(&not_covered);
        assert!(matches!(outcome, OverrideOutcome::Denied(_)), "sin consentimiento vigente, NUNCA debe ejecutar");
    }

    #[test]
    fn decide_override_authorization_denied_reason_reflects_the_real_not_covered_reason() {
        let stale = ConsentVerdict::NotCovered(NotCoveredReason::StaleVersion);
        let outcome = decide_override_authorization(&stale);
        match outcome {
            OverrideOutcome::Denied(reason) => assert!(
                reason.contains("StaleVersion"),
                "la razón denegada debe reflejar el NotCoveredReason real, fue: {reason}"
            ),
            OverrideOutcome::Executed => panic!("StaleVersion nunca debe ejecutar"),
        }
    }

    // ── CRITERIO: "eliminar" = archivar (regla fija #5) ──────────────────────

    /// CRITERIO DE CIERRE: `ARCHIVE` ejecutado -> archiva. Ninguna otra
    /// combinación produce `Archived` -- ni `MODIFY`/`REQUEST_AUDIT_REPORT`
    /// (no archivan nada por definición), ni `ARCHIVE` denegado (el gate lo
    /// bloqueó, no hay efecto local).
    #[test]
    fn apply_local_command_effect_archives_only_for_executed_archive_command() {
        assert_eq!(
            apply_local_command_effect(OverrideCommandKind::Archive, &OverrideOutcome::Executed),
            LocalEffect::Archived
        );
        assert_eq!(
            apply_local_command_effect(OverrideCommandKind::Archive, &OverrideOutcome::Denied("x".to_string())),
            LocalEffect::NoEffect,
            "un ARCHIVE denegado no debe archivar nada"
        );
        assert_eq!(
            apply_local_command_effect(OverrideCommandKind::Modify, &OverrideOutcome::Executed),
            LocalEffect::NoEffect,
            "MODIFY nunca archiva, aunque esté ejecutado"
        );
        assert_eq!(
            apply_local_command_effect(OverrideCommandKind::RequestAuditReport, &OverrideOutcome::Executed),
            LocalEffect::NoEffect,
            "REQUEST_AUDIT_REPORT nunca archiva, aunque esté ejecutado"
        );
    }

    // ── audit_hash: determinismo + sensibilidad a cambios ────────────────────

    #[test]
    fn compute_override_audit_hash_is_deterministic() {
        let hash_a = compute_override_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "child-1", "fund-1", "node-A", "ISSUER", "ARCHIVE", "strategy-42",
            "EXECUTED",
        );
        let hash_b = compute_override_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "child-1", "fund-1", "node-A", "ISSUER", "ARCHIVE", "strategy-42",
            "EXECUTED",
        );
        assert_eq!(hash_a, hash_b, "mismo evento, mismo hash");
    }

    #[test]
    fn compute_override_audit_hash_changes_when_outcome_changes() {
        let executed = compute_override_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "child-1", "fund-1", "node-A", "ISSUER", "ARCHIVE", "strategy-42",
            "EXECUTED",
        );
        let denied = compute_override_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "child-1", "fund-1", "node-A", "ISSUER", "ARCHIVE", "strategy-42",
            "DENIED",
        );
        assert_ne!(executed, denied, "cambiar el outcome debe cambiar el audit_hash");
    }

    #[test]
    fn compute_override_audit_hash_changes_when_attestation_side_changes() {
        let issuer = compute_override_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "child-1", "fund-1", "node-A", "ISSUER", "ARCHIVE", "strategy-42",
            "EXECUTED",
        );
        let executor = compute_override_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "child-1", "fund-1", "node-A", "EXECUTOR", "ARCHIVE", "strategy-42",
            "EXECUTED",
        );
        assert_ne!(issuer, executor, "el lado de la atestación debe formar parte del hash -- ISSUER y EXECUTOR nunca colisionan");
    }

    #[test]
    fn compute_hierarchy_audit_hash_is_deterministic_and_chains_by_row_version() {
        let genesis = compute_hierarchy_audit_hash(
            "id-1", 1_000, 1, None, "child-1", Some("fund-1"), "v1", "node-A",
        );
        let genesis_again = compute_hierarchy_audit_hash(
            "id-1", 1_000, 1, None, "child-1", Some("fund-1"), "v1", "node-A",
        );
        assert_eq!(genesis, genesis_again, "misma versión, mismo contenido -> mismo hash");

        let updated = compute_hierarchy_audit_hash(
            "id-1", 2_000, 2, Some(&genesis), "child-1", Some("fund-2"), "v1", "node-A",
        );
        assert_ne!(updated, genesis, "cambiar parent_owner_id debe cambiar el hash");

        let no_parent = compute_hierarchy_audit_hash("id-1", 1_000, 1, None, "child-1", None, "v1", "node-A");
        assert_ne!(no_parent, genesis, "sin padre (None) debe producir un hash distinto que con padre");
    }
}
