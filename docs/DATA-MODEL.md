# DATA-MODEL.md — Índice del Modelo de Datos Real

> **Propósito:** el mapa de las tablas SQLite que existen HOY (`migrations/*.sql`), quién es su dueña, y cómo se relacionan — sin tener que abrir el `.sql` ni el código. Igual que `docs/ADR.md`/`docs/SAD.md`, este archivo es el **índice**; la ficha completa de cada tabla vive en `docs/data-model/<tabla>.md` (plantilla: [`docs/templates/DATA-MODEL.md`](./templates/DATA-MODEL.md)).
> **Cobertura:** 29 tablas reales en 20 migraciones (`migrations/0001` a `migrations/0020`), verificado 2026-07-12 leyendo el 100% de las migraciones línea por línea — cero tablas inventadas o supuestas.
> **Protocolo de sincronización (OBLIGATORIO):** toda migración nueva/modificada actualiza su ficha correspondiente en el MISMO cambio. Si una migración no tiene ficha, o la ficha no coincide con la migración real, la migración es la fuente de verdad y la ficha está desactualizada — repórtalo.

---

## Patrón dominante del esquema

**No hay tablas pivote M:N todavía.** El 100% de las relaciones hoy son 1:N por fan-out desde `accounts` (la cuenta local es el ancla casi universal — 18 de las 29 tablas tienen `owner_id → accounts(id) ON DELETE RESTRICT`). Las únicas dos relaciones hija→padre que NO son `accounts` son `job_results → jobs` y `operator_assignments → operator_roles`. El resto de referencias entre tablas de dominio (`api_usage_records.credential_id`, `attested_track_records.verified_account_id`, `account_hierarchy.parent_owner_id`, `override_attestations.parent_owner_id`) son **referencias suaves** (sin `FOREIGN KEY` física) — ver cada ficha para el porqué.

**Tres naturalezas de tabla, sin excepción:**
- **APPEND-ONLY** (`event_sequence_id INTEGER UNIQUE` + triggers `BEFORE UPDATE`/`BEFORE DELETE` que abortan) — un hecho histórico permanente, nunca se edita ni se borra.
- **MUTABLE** (`row_version INTEGER`, concurrencia optimista) — el estado cambia en sitio.
- **ESTADO/REFERENCIA** — casos especiales sin ninguno de los dos patrones (`mcp_gateway_config` es clave-valor; `foundation_master_fields` es el ancla conceptual del contrato de 25 campos, sin lógica de app real).

---

## Índice de tablas

| Tabla | Feature dueña | Migración | Naturaleza | FK saliente principal |
|---|---|---|---|---|
| [`foundation_master_fields`](./data-model/foundation_master_fields.md) | Ancla ADR-0020 (sin feature propia) | `0001` | Referencia | — |
| [`audit_events`](./data-model/audit_events.md) | [`audit-log`](./features/audit-log.md) | `0002` | Append-only | `owner_id → accounts` |
| [`jobs`](./data-model/jobs.md) | [`async-job-executor`](./features/async-job-executor.md) | `0003` | Mutable | `owner_id → accounts` |
| [`job_results`](./data-model/job_results.md) | [`async-job-executor`](./features/async-job-executor.md) | `0003` | Append-only | `job_uuid → jobs` |
| [`telemetry_samples`](./data-model/telemetry_samples.md) | [`telemetry`](./features/telemetry.md) | `0004` | Podado por retención | — |
| [`permission_decisions`](./data-model/permission_decisions.md) | [`agentic-mcp-gateway`](./features/agentic-mcp-gateway.md) | `0005` | Append-only | `owner_id → accounts` (nullable) |
| [`mcp_gateway_config`](./data-model/mcp_gateway_config.md) | [`agentic-mcp-gateway`](./features/agentic-mcp-gateway.md) | `0005` | Estado clave-valor | — |
| [`sovereign_download_records`](./data-model/sovereign_download_records.md) | [`sovereign-data-fetcher`](./features/sovereign-data-fetcher.md) | `0006` | Append-only | — |
| [`accounts`](./data-model/accounts.md) | [`central-identity`](./features/central-identity.md) | `0007` | Mutable | — (ancla) |
| [`licenses`](./data-model/licenses.md) | [`licensing-system`](./features/licensing-system.md) | `0008` | Mutable | `owner_id → accounts` |
| [`plans`](./data-model/plans.md) | [`plan-tier-quota`](./features/plan-tier-quota.md) | `0009` | Mutable | — |
| [`usage_records`](./data-model/usage_records.md) | [`usage-metering`](./features/usage-metering.md) | `0010` | Append-only | `owner_id → accounts` |
| [`consent_records`](./data-model/consent_records.md) | [`consent-registry`](./features/consent-registry.md) | `0011` | Append-only | `owner_id → accounts` |
| [`domain_events`](./data-model/domain_events.md) | [`enriched-domain-events`](./features/enriched-domain-events.md) | `0012` | Append-only | `owner_id → accounts` |
| [`generated_reports`](./data-model/generated_reports.md) | [`institutional-report-engine`](./features/institutional-report-engine.md) | `0013` | Append-only | `owner_id → accounts` |
| [`api_credentials`](./data-model/api_credentials.md) | [`third-party-api-gateway`](./features/third-party-api-gateway.md) | `0014` | Mutable | `owner_id → accounts` |
| [`api_usage_records`](./data-model/api_usage_records.md) | [`third-party-api-gateway`](./features/third-party-api-gateway.md) | `0014` | Append-only | `owner_id → accounts` |
| [`aggregated_indexes`](./data-model/aggregated_indexes.md) | [`data-aggregation`](./features/data-aggregation.md) | `0015` | Append-only | — |
| [`verified_accounts`](./data-model/verified_accounts.md) | [`verified-account-registry`](./features/verified-account-registry.md) | `0016` | Mutable | `owner_id → accounts` |
| [`attested_track_records`](./data-model/attested_track_records.md) | [`verified-account-registry`](./features/verified-account-registry.md) | `0016` | Append-only | `owner_id → accounts` |
| [`instance_backups`](./data-model/instance_backups.md) | [`instance-continuity`](./features/instance-continuity.md) | `0017` | Append-only | `owner_id → accounts` |
| [`custody_state`](./data-model/custody_state.md) | [`instance-continuity`](./features/instance-continuity.md) | `0017` | Mutable | `owner_id → accounts` |
| [`account_hierarchy`](./data-model/account_hierarchy.md) | [`master-account-hierarchy`](./features/master-account-hierarchy.md) | `0018` | Mutable | `owner_id → accounts` |
| [`override_attestations`](./data-model/override_attestations.md) | [`master-account-hierarchy`](./features/master-account-hierarchy.md) | `0018` | Append-only | `owner_id → accounts` |
| [`exportable_data_catalog`](./data-model/exportable_data_catalog.md) | [`data-portability`](./features/data-portability.md) | `0019` | Mutable | — |
| [`data_portability_requests`](./data-model/data_portability_requests.md) | [`data-portability`](./features/data-portability.md) | `0019` | Append-only | `owner_id → accounts` |
| [`operator_roles`](./data-model/operator_roles.md) | [`operator-roles`](./features/operator-roles.md) | `0020` | Mutable | `owner_id → accounts` |
| [`operator_assignments`](./data-model/operator_assignments.md) | [`operator-roles`](./features/operator-roles.md) | `0020` | Mutable | `owner_id → accounts`, `role_id → operator_roles` |
| [`operator_role_events`](./data-model/operator_role_events.md) | [`operator-roles`](./features/operator-roles.md) | `0020` | Append-only | `owner_id → accounts` |

---

## `accounts` — el hub (18 tablas dependen de ella)

Toda tabla del substrato de monetización (ADR-0144, cimientos #1–#14) cuelga de `accounts.id` vía `owner_id ON DELETE RESTRICT`: nunca se borra una cuenta con historial asociado en ninguna de las 18. Ver [`accounts.md`](./data-model/accounts.md) para el detalle completo de columnas y la lista completa de quién la referencia.

---

## Referencias suaves (sin FK física — documentadas, no accidentales)

| Columna | Apunta a | Por qué no lleva FK física |
|---|---|---|
| `api_usage_records.credential_id` | `api_credentials.id` | Denormalizado a propósito (el ledger de uso es auto-contenido sin JOIN obligatorio) |
| `attested_track_records.verified_account_id` | `verified_accounts.id` | Mismo criterio del resto del substrato: SQLite STRICT no impone FKs por defecto en este esquema salvo la enmienda `owner_id → accounts` |
| `account_hierarchy.parent_owner_id` | `accounts.id` (nullable) | Fuera del alcance textual de la enmienda ADR-0141 M6 (que solo fija la regla para columnas literalmente nombradas `owner_id`) — **observación abierta reportada al Tech-Lead en la propia migración 0018** |
| `override_attestations.parent_owner_id` | `accounts.id` | Misma observación abierta que `account_hierarchy` |

---

## Deuda de gobernanza documental relacionada

Este índice se creó el 2026-07-12 auditando las 20 migraciones existentes (`ADR-0153`, sesión de diseño del banco de estrategias, extendida a esta segunda tarea). Antes de esta fecha, el conocimiento de relaciones entre tablas vivía roto en tres sitios (comentarios `--` en cada `.sql`, la sección `## Persistencia` genérica de cada feature, y prosa dispersa en ADRs puntuales) — ver `docs/templates/DATA-MODEL.md` para el porqué y la plantilla.
