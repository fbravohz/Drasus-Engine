# Barrido de Coherencia Etapa 0.4 — Lector de Archivos Reales de Mercado (MT5)

**Rol:** Architect. **Insumo obligatorio:** `.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md` (Quant-Engineer, veredicto APTO-CON-CONDICIONES). **Método:** confirmar/corregir/ampliar el insumo del Quant; verificación independiente sobre los archivos reales de `datasets/` (muestreo, nunca carga completa); recorrido de ADR-0154/0155/0156/0114/0162/0163, `docs/features/data-import-wizard.md` y features de ingesta relacionadas, `docs/DATA-MODEL.md`.

**Verificación independiente del hallazgo de mayor impacto (§3.1 del Quant):** confirmado bit a bit sobre `datasets/EURUSD_TICK.csv` real. La primera fila del archivo (`2025.04.01 13:00:00.009`) trae solo `BID`, `FLAGS=2`. Conteo de `FLAGS`: `2`→1,499,738 · `4`→1,538,462 · `6`→1,959,998, suma exacta 4,998,198 = total de filas de dato. El 60.9% del archivo es unilateral. El hallazgo del Quant se canoniza sin corrección. También verifiqué de forma independiente §3.2 (las dos filas `TICKVOL=0` con rango de precio real, líneas 7187218/7187221, coinciden exactamente).

---

## Preguntas resueltas antes de implementar

*(sección lista para pegar en la Orden de Trabajo del lector MT5)*

### 1. ¿Cómo se reconstruye un `QuoteTick` (NOT NULL en `bid`/`ask`) cuando la fuente entrega actualizaciones unilaterales (MT5: cada tick trae solo un lado)?

**Respuesta:** por arrastre acotado (`carry-forward`), nunca inventando un valor. `data-normalization-layer` mantiene el último valor real de cada lado por `(instrument_id, venue)` y completa cada fila; cada lado queda marcado `OBSERVED` o `CARRIED_FORWARD` en dos columnas de auditoría nuevas. El arrastre nunca excede `QUOTE_SIDE_CARRY_FORWARD_MAX_NS` (más allá del umbral, la fila no se emite — arrastrar sin límite permitiría un lado congelado que cruce físicamente al otro). Reutiliza el mismo mecanismo ya vetado por el Quant y sellado por el propietario para `spread_reported` (ADR-0114 regla (c), sub-cláusula 2026-07-23) — no es estadística nueva, es la misma doctrina aplicada a un campo distinto.

**Documento y línea:** `docs/adr/ADR-0155.md`, Restricción 3, sub-cláusula "Sub-cláusula de reconstrucción unilateral (enmienda 2026-07-25)" — **canonizado en este barrido**. Esquema `QuoteTick` gana columnas `bid_source`/`ask_source`. Mecanismo técnico: `docs/features/data-normalization-layer.md` TTR-004.

### 2. ¿Qué pasa con el primer tick de un archivo/sesión, cuando ningún lado tiene semilla previa de la que arrastrar?

**Respuesta:** no se emite `QuoteTick` hasta que ambos lados hayan sido observados al menos una vez desde la apertura del archivo (en el archivo real, esto son los primeros ~5 ticks de 5 millones — pérdida despreciable). No es una pérdida silenciosa: los ticks de apertura descartados por esta razón se cuentan y quedan reportados en `transformation_records` (infraestructura ya existente, ADR-0141/ADR-0155). Es una limitación estructural de punto-en-el-tiempo (no puede existir una cotización de dos lados antes de que el mercado haya mostrado los dos lados), no una decisión de descarte arbitrario.

**Documento y línea:** `docs/adr/ADR-0155.md`, misma sub-cláusula de Restricción 3 — **canonizado en este barrido**.

### 3. ¿Quién declara la convención de unidad de `<VOL>` (volumen real de MT5) por bróker?

**Respuesta:** `broker_instrument_terms.real_volume_unit_declared` (nuevo campo, `NULL` = "no declarado", nunca "unidades base por defecto"). Si no está declarado, `data-normalization-layer` persiste el valor crudo (nunca lo descarta) pero marca la corrida de transformación como no verificada en vez de afirmar una unidad.

**Documento y línea:** `docs/adr/ADR-0162.md`, tabla de esquema `broker_instrument_terms`, fila "`real_volume_unit_declared` (enmienda 2026-07-25)" — **canonizado en este barrido**. Restricción reflejada en `docs/features/instrument-catalog.md`.

**Aclaración de alcance — esto NO responde qué convención usa el archivo concreto que se va a importar.** Ver "Bloqueos para el propietario" abajo.

### 4. ¿El Gate de Coherencia de Calendario admite la huella real observada (viernes 23:56 → lunes 00:00, sin barras dominicales) para `FOREX_245`?

**Respuesta: ya resuelto, no requiere canon nueva.** `session_template_window` es editable por el usuario y overrideable por bróker (`broker_instrument_terms.session_template_id_override`). El `SESSION_TEMPLATE_SEED_SET` es explícitamente "representativo, editable" — no una verdad universal. Si la huella real de este archivo no coincide con el ejemplo semilla, el Gate rechaza con motivo explícito (comportamiento ya diseñado, nunca silencioso) y el usuario ajusta o crea el template correcto para ese bróker antes de reimportar — el mecanismo para hacerlo ya existe, no hace falta una plantilla fija nueva tipo `FOREX_245_NO_SUNDAY`.

**Documento y línea:** `docs/features/session-catalog.md` restricciones ("NUNCA una plantilla de sesión se copia..."; tabla de parámetros, `SESSION_TEMPLATE_SEED_SET`); `docs/adr/ADR-0162.md` Restricción 2 y 5.

### 5. ¿`pit-data-validator` enumera "rango de precio real con `volume_ticks=0`" y "cotización cruzada (`bid > ask`)" como chequeos de imposibilidad?

**Respuesta: ahora sí, ambos canonizados en este barrido.** Ninguno de los dos aparecía documentado antes de este barrido; el archivo real trae dos casos concretos del primero (2019-11-01, coincide con cambio de DST de EE. UU.) y el segundo es una consecuencia directa de acotar mal el arrastre de la pregunta 1.

**Documento y línea:** `docs/features/pit-data-validator.md`, TTR-001 (regla de negocio `volume_ticks=0`) y TTR-004 nuevo (cotización cruzada, enmienda 2026-07-25).

### 6. ¿Quién ejecuta la validación de semántica exacta de `spread_reported` (open/close/promedio), ahora que el mes solapado que la desbloquea ya está en el repo?

**Respuesta: es opcional y no bloqueante — no requiere decisión de arquitectura nueva.** TASK-064 §4 ya especifica el procedimiento completo (comparación tick-a-tick del residual, criterio de aceptación, diagnóstico de sesgo) y declara explícitamente que el sistema no depende de él: "si el propietario borra el dataset de abril 2025, nada de lo canonizado aquí se rompe". Si esta comprobación se agenda dentro de la misma Story del lector o aparte es una decisión de secuenciación de trabajo, no de arquitectura — la remito al Tech-Lead sin tomarla por él.

**Documento y línea:** `docs/execution/TASK-064-politica-spread-fill-model-dst.md` §4.1–4.4 (particularmente el banner de corrección 2026-07-23).

### 7. ¿El parser debe descartar el `\r` (CRLF de Windows) antes de convertir `SPREAD`/`FLAGS` a entero?

**Respuesta: sí — este es un criterio de aceptación de implementación, no una decisión de arquitectura.** El propio informe del Quant (condición 4) ya lo especifica como test de oráculo verificable; no hay ambigüedad de diseño que resolver, solo un caso de prueba que el ingeniero debe satisfacer.

**Documento y línea:** `.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md`, condición 4.

### 8. ¿Es correcto que el lector de M1/tick no calcule ningún indicador ni agregado durante la ingesta (para no repetir el error de STORY-051)?

**Respuesta: sí, confirmado por diseño existente.** `Bars` es un primitivo de persistencia (mapea `open`/`high`/`low`/`close`/`TICKVOL`/`VOL`/`SPREAD` directo del archivo, sin cómputo), no un cálculo derivado — el principio "guarda el hecho, calcula la interpretación" ya excluye explícitamente `spread` derivado, `mid_price`, `vwap` calculado por Drasus, etc., de la capa de persistencia. Nada en el alcance de este lector calcula un promedio ponderado ni ningún indicador; toda transformación es mapeo de campo + las políticas de integridad ya ancladas (carry-forward de spread/bid/ask, piso de spread, sanitización). Esta pregunta se incluye explícitamente porque es la clase exacta de error que motivó la creación de esta Etapa 0.4.

**Documento y línea:** `docs/adr/ADR-0155.md` Restricción 1 y sección "Qué NO se persiste".

### 9. ¿En qué orden deben existir el instrumento/plantilla de sesión respecto a la importación del archivo?

**Respuesta: ya resuelto.** El instrumento y su plantilla de sesión deben existir en el catálogo ANTES de importar — si no hay plantilla de sesión declarada para el instrumento, el Gate de Coherencia no puede correr y la importación queda bloqueada por falta de declaración, nunca por un veredicto de coherencia (no es lo mismo "no declarado" que "declarado e incoherente").

**Documento y línea:** `docs/features/pit-data-validator.md`, TTR-003, "Precondición".

### 10. ¿Los tres archivos `.dat` (Dukascopy/IC Markets) entran en el alcance de esta Story?

**Respuesta: no. Fuera de alcance por instrucción explícita de este encargo** — su formato no está identificado (cabecera `4.2 D ABCDEFGH`/`SnRbTs` sin coincidencia con MT4 `.hst` ni `.bi5` oficial) y solo el propietario puede identificar la herramienta que los generó; ya se le preguntó. No se intenta deducir su formato en este barrido ni en la Story derivada.

---

## Qué canonicé y dónde (rutas exactas)

| Documento | Qué cambió |
|---|---|
| `docs/adr/ADR-0155.md` | Banner de enmienda 2026-07-25; sub-cláusula de reconstrucción unilateral en Restricción 3; columnas `bid_source`/`ask_source` en el esquema de `QuoteTick`; entrada de Trazabilidad. |
| `docs/adr/ADR-0162.md` | Banner de enmienda 2026-07-25; fila `real_volume_unit_declared` en el esquema de `broker_instrument_terms`; entrada de Trazabilidad. |
| `docs/ADR.md` | Índice actualizado para ADR-0155 y ADR-0162 (nota de enmienda 2026-07-25 en ambas filas). |
| `docs/features/data-normalization-layer.md` | Nuevo comportamiento observable + 4 restricciones + parámetro `QUOTE_SIDE_CARRY_FORWARD_MAX_NS` + TTR-004 completo (reconstrucción BBO). Cabecera actualizada. |
| `docs/features/pit-data-validator.md` | 2 comportamientos observables + 2 restricciones + regla de negocio en TTR-001 + TTR-004 nuevo (cotización cruzada) + puerto `quote_ticks_in` + dependencia de `data-normalization-layer`. Cabecera actualizada. |
| `docs/features/instrument-catalog.md` | 1 restricción nueva + nota de estado "pendiente de retrofit en migración" + cabecera actualizada. |

**Nada de esto tocó código de producción ni migraciones.** `crates/shared/src/types/mod.rs` confirma que `QuoteTick`/`TradeTick` siguen siendo marcadores de tipo de puerto (sin campos reales) y `migrations/0022` no tiene aún `real_volume_unit_declared` ni `bid_source`/`ask_source` — el retrofit lo ejecuta la Story del lector MT5, no este barrido. Es el momento más barato posible para esta canonización: cero código que retroceder.

---

## Contradicciones encontradas entre documentos vigentes

**Ninguna.** No encontré ningún par de documentos vigentes que se contradigan sobre este dominio. Los vacíos que sí encontré (preguntas 1–3, 5 arriba) eran ausencias de doctrina, no doctrinas en conflicto — se canonizaron arriba. La pregunta 4 (huella de `FOREX_245`) parecía a primera vista una discrepancia entre el ejemplo del `session-catalog.md` y el archivo real, pero al verificar el diseño completo confirmé que es una plantilla-semilla explícitamente editable, no una especificación rígida — no hay contradicción, solo un ejemplo ilustrativo que no cubre este bróker concreto.

---

## Bloqueos duros para el propietario (no improvisados, no resueltos aquí)

1. **Identidad de herramienta de los tres `.dat`** — ya preguntado, fuera de mi alcance (instrucción explícita del encargo).
2. **Aprobación del offset/zona horaria declarada para el CSV M1 de 25 años** — ya exigido por TASK-064 §3.5 antes de este barrido, lo reafirmo porque aplica directamente a esta Story: el mecanismo de declaración ya existe (selector IANA + bandera DST), pero el valor concreto que se declare para este archivo de 25 años debe ser confirmado explícitamente por el propietario (puede conocer el bróker/exportador de origen) antes de canonizarse como hecho — no es una inferencia que el ingeniero deba resolver por su cuenta.
3. **Convención de unidad real de `<VOL>` para el archivo concreto `EURUSD_M1_2000-01-01_2025-05-01.csv`** — la arquitectura ya tiene dónde persistir esta declaración (`real_volume_unit_declared`, canonizado arriba), pero el valor específico (¿lotes? ¿unidades del activo base? ¿contratos?) depende de qué terminal/bróker exportó este archivo — un hecho de negocio que solo el propietario conoce, no una decisión de arquitectura. **No bloquea construir el lector** (el campo es nullable y el sistema ya sabe marcar "no verificado" en vez de asumir) — sí bloquea usar `volume_traded` de este archivo con confianza plena en un backtest hasta que se declare.

Ninguno de los tres bloquea el arranque de la Story del lector MT5 para los dos archivos CSV (M1 y tick) — los tres son preguntas que el propietario puede responder en paralelo sin detener la implementación, y el sistema ya está diseñado para operar de forma explícita y auditable mientras la respuesta no llega (nunca silenciosa, nunca inventada).

---

## Lo ya especificado que nadie debía tener que redescubrir

- La zona horaria de origen del CSV **nunca se infiere de volatilidad ni de noticia** — ya descartado explícitamente por el propietario (ADR-0162 Restricción 7, TASK-064 §3.3). Un ingeniero que proponga anclar por Nóminas repetiría un método ya rechazado.
- El demultiplexado cotización/ejecución **es de `data-normalization-layer`, nunca del wizard** — ya fijado en `data-import-wizard.md` desde ADR-0155. La reconstrucción bid/ask unilateral que este barrido canoniza hereda la misma frontera: tampoco es del wizard.
- El piso de spread **nunca es una constante global** — vive por instrumento (`default_spread`, ADR-0162) más la distribución empírica del propio archivo (TASK-064 §1.2). Ya cerrado, no se re-abre por esta Story.
- El modelo de fill del motor (`bid` al vender, `ask` al comprar, nunca punto medio) **es FIJO desde ADR-0114 regla (c)** — el lector no toca esa doctrina, solo alimenta los campos que ya la satisfacen.

---

**Trazabilidad de este barrido:** `.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md` (insumo), `docs/adr/ADR-0155.md`, `docs/adr/ADR-0162.md`, `docs/adr/ADR-0156.md`, `docs/adr/ADR-0114.md`, `docs/adr/ADR-0163.md`, `docs/features/data-import-wizard.md`, `docs/features/data-normalization-layer.md`, `docs/features/pit-data-validator.md`, `docs/features/instrument-catalog.md`, `docs/features/session-catalog.md`, `docs/features/data-sanitizer-pipeline.md`, `docs/execution/TASK-064-politica-spread-fill-model-dst.md`, `docs/DATA-MODEL.md`, verificación directa sobre `datasets/EURUSD_TICK.csv` y `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv`.
