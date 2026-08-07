# DEBT-041 · El puente FFI no reconoce fecha y hora declaradas en columnas separadas — el Núcleo sí

| Campo | Valor |
|---|---|
| **ID** | DEBT-041 |
| **Severidad** | 🔴 Alta |
| **Estado** | Abierta |
| **Abierta** | 2026-07-31 (Flutter-Engineer, Etapa 4 de [STORY-073](../execution/STORY-073-asistente-importacion-pantalla.md), antes de construir el Paso 2 del asistente) |
| **Origen** | Contraste directo entre `crates/features/data/data-import-wizard/src/domain.rs` (Etapa 2) y `crates/bridge/src/api/data_import_wizard.rs` (Etapa 3) de la misma Story |
| **Disparador de pago** | Antes de dar por cumplido el criterio de aceptación #13 de STORY-073 (importar el archivo real de 9,2 M de filas de punta a punta) |

---

## El hecho

`domain.rs` define `CanonicalBarField` con **diez** variantes: `Timestamp` (columna combinada) y, como alternativa mutuamente excluyente, `TimestampDate` + `TimestampTime` (columnas separadas). El comentario del propio enum lo fecha hoy: *"el archivo real de MetaTrader del propietario: `<DATE>` TAB `<TIME>`, la forma más común de la industria — corrección del Tech-Lead 2026-07-31 sobre la medición contra ese archivo real"*. `resolve_manual_mapping` ya sabe combinar ambas columnas, y ya rechaza declarar solo una de las dos sin la otra.

El puente FFI que Flutter consume (`crates/bridge/src/api/data_import_wizard.rs::parse_canonical_bar_field`, líneas 298-313) **no se actualizó tras esa corrección**: su vocabulario reconocido sigue siendo el de los ocho campos anteriores (`TIMESTAMP, OPEN, HIGH, LOW, CLOSE, VOLUME_TICKS, VOLUME_TRADED, SPREAD_REPORTED`). Enviar `"TIMESTAMP_DATE"` o `"TIMESTAMP_TIME"` desde Dart hoy produce el error explícito `"canonical_field '...' no reconocido"` — nunca un dato mal interpretado, pero sí un rechazo total de la vía.

## Por qué es Alta y no Media

Porque el archivo que bloquea **es el mismo que cierra la Story**. El criterio de aceptación #13 de STORY-073 exige importar desde la interfaz, de punta a punta, `EURUSD_M1_2000-01-01_2025-05-01.csv` — el archivo real del propietario, con columnas `<DATE>` TAB `<TIME>` separadas. Sin este puente, el Paso 2 del asistente no tiene forma sintáctica de declarar ese archivo: cualquier intento de mapear las dos columnas temporales termina en rechazo del backend, y el criterio 13 no se puede ejecutar contra el archivo real, solo contra un fixture de columna combinada que no representa el caso que motivó la Story.

## Causa raíz

Secuenciación entre Etapas de la misma Story: la Etapa 2 (Núcleo) corrigió el modelo de campos **después** de que la Etapa 3 (Puente) ya hubiera cerrado su propia superficie, y nadie propagó el cambio de vocabulario hacia el Bridge antes de despachar la Etapa 4 (Interfaz).

## Qué hace falta — acotado y verificado contra el código real

En `crates/bridge/src/api/data_import_wizard.rs::parse_canonical_bar_field` (función interna, ~15 líneas), añadir dos brazos al `match`:

```rust
"TIMESTAMP_DATE" => Ok(CanonicalBarField::TimestampDate),
"TIMESTAMP_TIME" => Ok(CanonicalBarField::TimestampTime),
```

Y actualizar el mensaje de error y el doc-comment del vocabulario fijo (hoy dicen "ocho campos"; pasan a diez). No hace falta tocar ningún DTO ni regenerar bindings: `ManualColumnMappingDto.canonicalField` ya viaja como `String` libre — el único cambio es qué acepta el `match` del lado Rust.

## Qué se hizo mientras tanto (Etapa 4, sin tocar Rust)

El Paso 2 del Inspector Panel (`ui/lib/tabs/data_import_wizard/`) se construyó **hacia adelante**: deja elegir entre columna combinada o columna de fecha + columna de hora separadas, y emite exactamente `"TIMESTAMP_DATE"`/`"TIMESTAMP_TIME"` (mismo vocabulario que `CanonicalBarField::label()`) cuando el usuario declara la vía separada. Hoy, contra el archivo real, esa vía falla con el mensaje literal del backend (comportamiento correcto de ADR-0156 — rechazo explícito, nunca importación parcial) hasta que esta deuda se pague. La vía de columna combinada funciona de punta a punta hoy mismo.

## Relación con otras deudas

- Comparte Story y bloqueo de criterio con [`DEBT-039`](./DEBT-039.md) (rango declarado vs. contenido real) y [`DEBT-040`](./DEBT-040.md) (memoria de la confirmación) — las tres tocan el mismo criterio 13, por razones distintas.
