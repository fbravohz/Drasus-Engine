//! [SHELL] Repositorio de decisiones de permiso del Gateway MCP.
//!
//! Implementa la persistencia de la tabla `permission_decisions`
//! (append-only, forense) y de la tabla de configuración `mcp_gateway_config`
//! (estado mutable del interruptor de producción).
//!
//! ADR-0020, Perfil D (Ops/Auditoría): Grupo I + Grupo II + Grupo IV +
//! 4 campos de dominio propio. Migración: `0005_mcp_gateway.sql`.

use sqlx::{Row, SqlitePool};

use crate::domain::mcp_gateway::PermissionDecision;

// ────────────────────────────────────────────────────────────────────────────
// Tipo de error
// ────────────────────────────────────────────────────────────────────────────

/// Errores del repositorio de decisiones de permiso.
#[derive(Debug, thiserror::Error)]
pub enum McpGatewayError {
    /// Error de la capa SQLite (SQLx).
    #[error("SQLite: {0}")]
    Sqlx(#[from] sqlx::Error),
}

// ────────────────────────────────────────────────────────────────────────────
// Repositorio
// ────────────────────────────────────────────────────────────────────────────

/// Repositorio de decisiones de permiso y del interruptor de producción.
///
/// Todas las operaciones sobre `permission_decisions` son append-only:
/// no existe UPDATE ni DELETE sobre esa tabla. El propósito es forense:
/// una vez registrada, una decisión no puede borrarse ni modificarse.
///
/// `mcp_gateway_config` sí es mutable: el interruptor puede
/// activarse y desactivarse en cualquier momento por el propietario.
pub struct McpGatewayRepository;

impl McpGatewayRepository {
    // ── Decisiones de permiso (append-only) ──────────────────────────────

    /// Inserta una decisión de permiso en la tabla `permission_decisions`.
    ///
    /// La tabla es append-only: este método es la única escritura permitida.
    /// Nunca llames a UPDATE o DELETE sobre `permission_decisions`.
    pub async fn append(
        pool: &SqlitePool,
        d: &PermissionDecision,
    ) -> Result<(), McpGatewayError> {
        let production_override_i64 = if d.production_override_active { 1i64 } else { 0i64 };
        let id_str = d.id.to_string();

        sqlx::query(
            r#"
            INSERT INTO permission_decisions (
                id, created_at, updated_at,
                audit_hash, audit_chain_hash, event_sequence_id,
                owner_id, institutional_tag,
                node_id, process_id,
                agent_session_id, requested_scope,
                permission_outcome, production_override_active
            ) VALUES (
                ?, ?, ?,
                ?, ?, ?,
                ?, ?,
                ?, ?,
                ?, ?,
                ?, ?
            )
            "#,
        )
        .bind(&id_str)
        .bind(d.created_at)
        .bind(d.updated_at)
        .bind(&d.audit_hash)
        .bind(&d.audit_chain_hash)
        .bind(d.event_sequence_id)
        .bind(&d.owner_id)
        .bind(&d.institutional_tag)
        .bind(&d.node_id)
        .bind(d.process_id)
        .bind(&d.agent_session_id)
        .bind(&d.requested_scope)
        .bind(&d.permission_outcome)
        .bind(production_override_i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Devuelve el `audit_hash` y el `event_sequence_id` de la última
    /// decisión de la cadena (el extremo donde encadenar la siguiente).
    ///
    /// Devuelve `None` si la tabla está vacía (la siguiente decisión usará
    /// `"genesis"` como `audit_chain_hash` y `1` como `event_sequence_id`).
    pub async fn chain_tip(
        pool: &SqlitePool,
    ) -> Result<Option<(String, i64)>, McpGatewayError> {
        let row = sqlx::query(
            r#"
            SELECT audit_hash, event_sequence_id
            FROM permission_decisions
            ORDER BY event_sequence_id DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let hash: String = r.get("audit_hash");
            let seq: i64 = r.get("event_sequence_id");
            (hash, seq)
        }))
    }

    // ── Interruptor de producción (estado mutable) ────────────────────────

    /// Lee el estado actual del interruptor de producción desde la BD.
    ///
    /// `true` = activo (el agente puede invocar `execute`/`withdraw`/`manage(Live)`).
    /// `false` = inactivo (comportamiento de fábrica, ADR-0123).
    pub async fn get_production_override(pool: &SqlitePool) -> Result<bool, McpGatewayError> {
        let row = sqlx::query(
            r#"
            SELECT value FROM mcp_gateway_config
            WHERE key = 'production_override_active'
            "#,
        )
        .fetch_one(pool)
        .await?;

        let value: String = row.get("value");
        Ok(value == "1")
    }

    /// Activa o desactiva el interruptor de producción.
    ///
    /// Solo el propietario del despliegue puede llamar a este método.
    /// La API del agente MCP NO expone esta operación como herramienta
    /// (ADR-0123: "El interruptor no puede activarse por una llamada del propio agente").
    pub async fn set_production_override(
        pool: &SqlitePool,
        active: bool,
    ) -> Result<(), McpGatewayError> {
        let value = if active { "1" } else { "0" };

        sqlx::query(
            r#"
            INSERT INTO mcp_gateway_config (key, value)
            VALUES ('production_override_active', ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
        )
        .bind(value)
        .execute(pool)
        .await?;

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Pruebas de integración (Capa 2 — ADR-0133)
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;
    use crate::domain::mcp_gateway::{
        PermissionDecision, PermissionOutcome, PermissionRequest, Pipeline,
        evaluate_permission,
    };
    use crate::persistence::pool::{connect, migrate};

    /// Abre una BD en archivo temporal y aplica las migraciones.
    ///
    /// Usamos archivo temporal, no `:memory:`, porque la Capa 2 de la pirámide
    /// ADR-0133 exige verificar durabilidad/recuperación: la BD debe sobrevivir
    /// al cierre del pool. Una BD `:memory:` desaparece al cerrar el pool,
    /// haciendo inútil cualquier prueba de persistencia real.
    async fn setup_db() -> (SqlitePool, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let url = format!("sqlite://{}", file.path().display());
        let pool = connect(&url).await.unwrap();
        migrate(&pool).await.unwrap();
        (pool, file)
    }

    fn make_decision(
        req: &PermissionRequest,
        outcome: &PermissionOutcome,
        prev_hash: &str,
        seq: i64,
    ) -> PermissionDecision {
        PermissionDecision::build(
            req,
            outcome,
            1_700_000_000_000_000_000i64,
            prev_hash.to_string(),
            seq,
            "node-test".to_string(),
            12345,
        )
    }

    // Criterio 9: una decisión persistida es recuperable vía chain_tip.
    #[tokio::test]
    async fn permission_decision_persists_and_is_retrievable() {
        let (pool, _file) = setup_db().await;

        let req = PermissionRequest {
            pipeline: Pipeline::Ingest,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-abc".into(),
            requested_scope: "ingest.submit_bar".into(),
        };
        let outcome = evaluate_permission(&req);
        let decision = make_decision(&req, &outcome, "genesis", 1);

        McpGatewayRepository::append(&pool, &decision).await.unwrap();

        let tip = McpGatewayRepository::chain_tip(&pool).await.unwrap();
        assert!(tip.is_some(), "chain_tip debe devolver Some tras insertar");
        let (hash, seq) = tip.unwrap();
        assert_eq!(seq, 1);
        // El hash recuperado coincide con el que se insertó.
        assert_eq!(hash, decision.audit_hash);
    }

    // Criterio 10: la cadena de auditoría encadena decisiones secuencialmente.
    #[tokio::test]
    async fn audit_chain_links_sequential_decisions() {
        let (pool, _file) = setup_db().await;

        let req = PermissionRequest {
            pipeline: Pipeline::Ingest,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-chain".into(),
            requested_scope: "ingest.submit_bar".into(),
        };
        let outcome = evaluate_permission(&req);

        // Primera decisión: prev_hash = "genesis", seq = 1.
        let d1 = make_decision(&req, &outcome, "genesis", 1);
        let hash_d1 = d1.audit_hash.clone();
        McpGatewayRepository::append(&pool, &d1).await.unwrap();

        // Segunda decisión: prev_hash = audit_hash de d1, seq = 2.
        let d2 = make_decision(&req, &outcome, &hash_d1, 2);
        // El audit_chain_hash de d2 apunta al audit_hash de d1.
        assert_eq!(d2.audit_chain_hash, hash_d1);
        McpGatewayRepository::append(&pool, &d2).await.unwrap();

        // chain_tip devuelve la última (d2).
        let tip = McpGatewayRepository::chain_tip(&pool).await.unwrap().unwrap();
        assert_eq!(tip.1, 2, "sequence_id debe ser 2");
        assert_eq!(tip.0, d2.audit_hash, "hash del tip debe ser el de d2");
    }

    // Criterio 11: el interruptor de producción persiste en la BD.
    #[tokio::test]
    async fn production_override_toggle_persists() {
        let (pool, _file) = setup_db().await;

        // Valor inicial debe ser false (la migración lo inicializa a '0').
        let initial = McpGatewayRepository::get_production_override(&pool).await.unwrap();
        assert!(!initial, "valor inicial debe ser false");

        // Activar.
        McpGatewayRepository::set_production_override(&pool, true).await.unwrap();
        let after_enable = McpGatewayRepository::get_production_override(&pool).await.unwrap();
        assert!(after_enable, "debe ser true tras activar");

        // Desactivar.
        McpGatewayRepository::set_production_override(&pool, false).await.unwrap();
        let after_disable = McpGatewayRepository::get_production_override(&pool).await.unwrap();
        assert!(!after_disable, "debe ser false tras desactivar");
    }
}
