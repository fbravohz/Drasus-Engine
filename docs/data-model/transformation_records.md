# transformation_records

**Feature dueña:** [`hybrid-data-transformer`](../features/hybrid-data-transformer.md)
**Migración:** `migrations/0021_hybrid_data_transformer.sql`
**Naturaleza:** APPEND-ONLY (`event_sequence_id UNIQUE`, mismo patrón de repositorio atómico que `sovereign_download_records`, con triggers `trg_transformation_records_no_update`/`trg_transformation_records_no_delete` que rechazan a nivel de base de datos cualquier UPDATE/DELETE — ver Notas de diseño)
**Perfil ADR-0020:** Perfil A (Datos de Mercado) — Grupo I + Grupo III (`data_snapshot_id`, `transformation_id`, `logic_hash`, `parent_id`) + Grupo IV (`node_id`, `process_id`, `session_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` (registro inmutable) |
| `audit_hash` | TEXT | NO | SHA-256 del DataFrame resultante (barras ya ordenadas, calculado en el Núcleo) |
| `audit_chain_hash` | TEXT | SÍ | `audit_hash` de la fila anterior; NULL en la fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `data_snapshot_id` | TEXT | SÍ | Formato `<exchange>_<symbol>_<timeframe>_<year><month>` (`CHECK GLOB '*_*_*_*'`) |
| `transformation_id` | TEXT | NO | ID del paso de transformación dentro del pipeline (ej. `"OHLCV_AGGREGATION"`) |
| `logic_hash` | TEXT | NO | Hash reproducible del plan de transformación (intervalo destino + umbral) |
| `parent_id` | TEXT | SÍ | Transformación previa encadenada, si el pipeline tiene varios pasos |
| `node_id` | TEXT | SÍ | Hardware donde corrió la transformación |
| `process_id` | TEXT | SÍ | PID del proceso que ejecutó la transformación |
| `session_id` | TEXT | SÍ | Sesión de ingesta que originó la transformación |
| `transform_duration_ns` | INTEGER | NO | Duración real del proceso (nanosegundos de pared) |
| `rows_in` | INTEGER | NO | Filas de entrada (ticks recibidos) |
| `rows_out` | INTEGER | NO | Filas de salida (barras producidas) |
| `schema_version` | TEXT | NO | Versión semver del esquema de salida (ADR-0141 F3, ej. `"1.0"`) |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna
- **Referencias no-FK (soft, informativas):** `parent_id` → `transformation_records.id` — encadena pasos de un pipeline multi-transformación; sin FK física por el mismo criterio que `account_hierarchy.parent_owner_id` (autorreferencia dentro de la misma tabla, fuera del alcance textual de la enmienda ADR-0141 M6, que solo fija la regla para columnas literalmente nombradas `owner_id`)
- **Índices:** `event_sequence_id`, `node_id`, `parent_id`

## Quién la referencia (FK entrante)

- Ninguna todavía. El sanitizador y el validador PIT (aguas abajo en el pipeline `ingest`) son candidatos naturales a referenciar `id` como su propio `parent_id` cuando se construyan.

## Notas de diseño

- `audit_hash`/`audit_chain_hash` tienen una semántica distinta de `sovereign_download_records`: aquí `audit_hash` es el hash del CONTENIDO resultante (las barras ya ordenadas, ADR-0141 criterio de reproducibilidad — el mismo dataset y plan producen el mismo hash sin importar el número de hilos de Polars usados), no un hash de los campos de la fila. La cadena (`audit_chain_hash`) protege la integridad histórica del pipeline ETL completo.
- Sin credenciales (Grupo II omitido — esta feature no toca brokers ni secretos).
- `transform_duration_ns`/`rows_in`/`rows_out`/`schema_version` son campos propios fuera del catálogo de 25 (excepción documentada en `docs/features/hybrid-data-transformer.md` "Persistencia"): sustituyen a `execution_latency_ms` (Grupo V, no autorizado en Perfil A).
- **Corrección Tech-Lead 2026-07-18:** la migración original dejaba la inmutabilidad de este ledger como una convención del repositorio (nadie emite `UPDATE`/`DELETE`), sin que la base de datos la exigiera. Se añadieron los triggers `trg_transformation_records_no_update`/`trg_transformation_records_no_delete` (mismo patrón que `0019_data_portability.sql`), que rechazan con `RAISE(ABORT, ...)` cualquier intento de modificar o borrar una fila ya persistida — probado en `tests/integration_tests.rs` (`transformation_records_rejects_update_and_delete_at_database_level`).
