# import_mapping_declarations

**Feature dueña:** [`data-import-wizard`](../features/data-import-wizard.md)
**Migración:** `migrations/0023_data_import_wizard_mapping_ledger.sql`
**Naturaleza:** APPEND-ONLY (`event_sequence_id UNIQUE`, molde `.agents/knowledge/ledger-pattern.md` — triggers `trg_import_mapping_declarations_no_update`/`trg_import_mapping_declarations_no_delete` que rechazan a nivel de base de datos cualquier UPDATE/DELETE)
**Perfil ADR-0020:** Perfil A (Datos/Ingest) — Grupo I (universal, append-only) + Grupo IV (`node_id`/`process_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` (registro inmutable) |
| `audit_hash` | TEXT | NO | SHA-256 del contenido de la declaración (mapeo + plantilla + vista previa + zona horaria + versión de esquema), calculado en el Núcleo |
| `audit_chain_hash` | TEXT | SÍ | `audit_hash` de la fila anterior; NULL en la fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `node_id` | TEXT | SÍ | Hardware que selló la declaración |
| `process_id` | TEXT | SÍ | PID del proceso que la selló |
| `source_file_name` | TEXT | NO | Nombre del archivo tal como lo entregó el usuario |
| `source_file_hash` | TEXT | NO | SHA-256 del contenido crudo del archivo — clave de idempotencia de reimportación (ADR-0163) |
| `instrument_id` | TEXT | NO | FK → `instrument_master(instrument_id)` — misma cadena de negocio de ADR-0155 Restricción 8 |
| `venue` | TEXT | NO | Bróker/venue donde se opera el instrumento |
| `bar_type` | TEXT | NO | `CHECK IN ('TIME','TICK_COUNT','VOLUME','DOLLAR','RENKO','RANGE')` — catálogo ADR-0155, default `'TIME'` |
| `timeframe_value` | TEXT | NO | Ej. `"1M"` — fijo para toda la corrida de este archivo |
| `mapping_origin` | TEXT | NO | `CHECK IN ('PLATFORM_TEMPLATE','MANUAL_USER')` — decide quién responde por un mapeo erróneo (LDR-0008) |
| `platform_template_code` | TEXT | SÍ | Ej. `"MT5_M1_BAR_V1"` — NULL si `mapping_origin = MANUAL_USER` |
| `platform_template_version` | TEXT | SÍ | Versión de la plantilla aplicada — NULL si `MANUAL_USER` |
| `column_mapping_json` | TEXT | NO | `CHECK (json_valid(...))` — mapeo completo columna origen → campo canónico, tal como se mostró al usuario |
| `transform_engine_version` | TEXT | NO | Versión del binario/motor que ejecutó el mapeo |
| `canonical_schema_version` | TEXT | NO | Versión del esquema canónico (ADR-0155) vigente al declarar |
| `accepted_at` | INTEGER | NO | Nanosegundos UTC del instante de aceptación |
| `preview_transformed_rows_json` | TEXT | NO | `CHECK (json_valid(...))` — filas de DATO YA TRANSFORMADO que el usuario vio al aceptar (LDR-0008 condición técnica 3, nunca solo esquema) |
| `source_timezone_declared` | TEXT | NO | Identificador IANA (ej. `"America/New_York"`) — NUNCA se infiere; una importación sin esta declaración se rechaza antes de llegar a este ledger |
| `real_volume_unit_declared` | TEXT | SÍ | `CHECK (NULL OR IN ('CONTRACTS','LOTS','BASE_ASSET_UNITS','QUOTE_ASSET_UNITS'))` — espejo de `broker_instrument_terms` para esta corrida concreta |
| `spread_reported_unit_declared` | TEXT | NO | Unidad de `spread_reported_raw` (criterio de aceptación #13). Para la plantilla MT5 M1, siempre `"POINTS_MT5"` — un hecho FIJO de la plantilla (Drasus la afirma, LDR-0008), no una declaración variable por bróker como el volumen. NOT NULL: una importación sin esta declaración se rechaza antes de llegar aquí, mismo trato que `source_timezone_declared` |

## Claves

- **PK:** `id`
- **FK salientes:** `instrument_id` → `instrument_master(instrument_id)` `ON DELETE RESTRICT`
- **Únicas:** `event_sequence_id`, `source_file_hash` (un mismo archivo por contenido no genera dos declaraciones — reimportar el mismo tramo idéntico es no-op, ADR-0163)
- **Índices:** `instrument_id` (FK hijo), `event_sequence_id`, `node_id`

## Quién la referencia (FK entrante)

- Ninguna físicamente todavía. Por convención de aplicación (ADR-0156 Restricción 8, NO FK física porque vive en otra feature): `transformation_records.data_snapshot_id` remonta, vía la partición Parquet, hasta el `id` de la fila de este ledger que originó la corrida — cuatro saltos por referencia (`backtest → data_snapshot_id → transformation_records → esta fila`), nunca datos duplicados.

## Notas de diseño

- **No lleva `data_snapshot_id`.** Una sola declaración de mapeo puede cubrir muchas particiones Parquet (un archivo de 25 años produce del orden de 300 particiones mensuales) — la relación de trazabilidad es la inversa de lo habitual en este substrato: no es esta tabla la que apunta a la partición, es la partición (vía `transformation_records`) la que remonta hasta esta fila.
- `mapping_origin` es el campo que LDR-0008 exige como "propio y consultable, no inferido" — decide si un mapeo erróneo lo responde Drasus (`PLATFORM_TEMPLATE`) o el usuario (`MANUAL_USER`).
- `preview_transformed_rows_json` debe contener datos ya transformados (precio, volumen, fecha reales), nunca solo la tabla de correspondencias columna→columna — es la condición técnica 1 de LDR-0008, la que separa una aceptación informada de un clic vacío.
- Sellada **antes** de que exista la primera fila de dato transformado (criterio de aceptación #5 de STORY-068): si la persistencia de datos aguas abajo falla, esta fila ya quedó comprometida y es recuperable.
- Enforzamiento append-only a nivel de base de datos, mismo patrón que `0019_data_portability.sql`/`0021_hybrid_data_transformer.sql` — probado en las cuatro pruebas obligatorias de `.agents/knowledge/ledger-pattern.md`.
