# generated_reports

**Feature dueña:** [`institutional-report-engine`](../features/institutional-report-engine.md)
**Migración:** `migrations/0013_generated_reports.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id` (IV) + subset V (`signature_hash`, `compliance_status_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Instante de PERSISTENCIA |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | Integridad de ESTA FILA en el ledger |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona GLOBAL, `UNIQUE` |
| `owner_id` | TEXT | NO | Dueño del reporte |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que generó el reporte |
| `signature_hash` | TEXT | NO | Firma REPRODUCIBLE del CONTENIDO (distinta de `audit_hash`) |
| `compliance_status_id` | TEXT | SÍ | Veredicto vigente al generar (nullable) |
| `report_type` | TEXT | NO | `VALIDATION`\|`BACKTEST`\|`EXECUTION`\|`STRESS_TEST`\|`MODEL_VALIDATION`\|`BACKTEST_CERTIFICATION`\|`DRAWDOWN_FORENSICS` |
| `source_result_ref` | TEXT | SÍ | Referencia libre al resultado fuente (nullable) |
| `source_event_refs` | TEXT | NO | JSON lista de ids de `domain_events`/audit-log citados |
| `report_body` | TEXT | NO | JSON canónico completo — lo que `signature_hash` hashea |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `report_type`, `owner_id`
- **Triggers:** `trg_generated_reports_no_update`, `trg_generated_reports_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- `signature_hash` ≠ `audit_hash`: el primero es integridad REPRODUCIBLE del contenido del reporte, el segundo es integridad de la fila en el ledger — dos firmas con propósitos distintos, no redundantes.
- Tres tipos ya en producción (`VALIDATION`/`BACKTEST`/`EXECUTION`), cuatro anticipados por ADR-0144 con adaptador de negocio diferido pero catálogo ya sembrado (Inundación de Fundaciones).
