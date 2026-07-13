# foundation_master_fields

**Feature dueña:** ninguna — ancla conceptual de ADR-0020 (Inundación de Fundaciones), no lógica de aplicación real.
**Migración:** `migrations/0001_foundation_master_fields.sql`
**Naturaleza:** Referencia — no es tabla de negocio, es la implementación de referencia del contrato de 25 campos.
**Perfil ADR-0020:** Los 25 campos completos (Grupos I–V), única tabla del sistema donde eso es correcto — todas las demás toman solo el subset de su Perfil.

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Nanosegundos desde epoch |
| `audit_hash` | TEXT | NO | SHA-256 |
| `audit_chain_hash` | TEXT | SÍ | Enlace blockchain-lite (NULL en fila génesis) |
| `event_sequence_id` | INTEGER | NO | Secuencia de recuperación |
| `owner_id` | TEXT | SÍ | Dueño capital/IP |
| `institutional_tag` | TEXT | SÍ | Entorno |
| `manifest_id` | TEXT | SÍ | Contrato de diseño |
| `access_token_id` | TEXT | SÍ | Auth tracking |
| `version_node_id` | TEXT | SÍ | DAG link |
| `parent_id` | TEXT | SÍ | Puntero genético |
| `logic_hash` | TEXT | SÍ | Commit código/binario |
| `data_snapshot_id` | TEXT | SÍ | PIT market snapshot |
| `transformation_id` | TEXT | SÍ | Raw vs synthetic flag |
| `process_id` | TEXT | SÍ | Job anchor |
| `session_id` | TEXT | SÍ | Runtime grouping |
| `node_id` | TEXT | SÍ | Hardware ID |
| `portfolio_container_id` | TEXT | SÍ | Governance |
| `compliance_status_id` | TEXT | SÍ | Veredicto riesgo |
| `risk_audit_id` | TEXT | SÍ | Ticket detallado riesgo |
| `indicator_state_hash` | TEXT | SÍ | Technical snapshot |
| `execution_latency_ms` | INTEGER | SÍ | Latencia en ms |
| `source_signal_id` | TEXT | SÍ | Signal link |
| `signature_hash` | TEXT | SÍ | HMAC signals |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna
- **Índices:** `event_sequence_id`

## Quién la referencia (FK entrante)

- Ninguna tabla real la referencia por FK — es una tabla de **vocabulario**, no de datos operativos. Cada tabla del sistema embebe el subset de estos 25 campos que su Perfil (ADR-0020) exige, sin apuntar de vuelta aquí.

## Notas de diseño

- Es el "diccionario" físico del Contrato Global de Persistencia (ADR-0020): cualquier duda de "¿qué tipo SQL usa `logic_hash` en todo el sistema?" se resuelve mirando esta tabla, no memorizando 29 esquemas distintos.
- `STRICT` sin cambios de tipo — todas las columnas ya usaban `TEXT`/`INTEGER` canónico desde el origen (ADR-0141 M12).
