# Point-In-Time Data Validator — Prevención de Look-Ahead Bias

**Carpeta:** `./features/pit-data-validator/`
**Estado:** Crítico / Prioritario
**Última actualización:** 2026-07-25 (barrido Etapa 0.4 — cotización cruzada, `volume_ticks=0` con rango real)
**Decisión Arquitectónica Asociada (Gate de Coherencia):** ADR-0162 · ADR-0155 (enmienda 2026-07-25, TTR-004)

---

## ¿Qué es?

Valida que los datos históricos son "Point-In-Time" (PIT-real): información que realmente estaba disponible en ese momento específico, sin contaminación del futuro. Detecta y rechaza datos con look-ahead bias (el error más común en backtesting falso).

**Problema:** Si tu dato "abierto" de mañana contamina tu análisis de hoy, tienes look-ahead bias. Ejemplo: si usas "máximo de la semana" para tomar decisiones hoy, pero ese máximo no ocurre hasta el jueves, estás viendo el futuro (bias). PIT-real significa: "datos que solo existen después de que accionaste".

**User Story:** Como usuario, quiero garantizar que mis backtests no contienen look-ahead bias. El sistema debe validar automáticamente que cada barra usa solo información disponible EN ESA BARRA, no posterior.

---

## Comportamientos Observables

- [ ] Usuario importa datos OHLCV (Open, High, Low, Close, Volume)
  → Sistema valida que Open[t] < High[t] ≤ Max(Close) en período [t-1, t]
  → Si High[t] > todos los precios anteriores, PODRÍA SER PIT válido
  → Si High[t] < todos los anteriores (retroceso fácil), probablemente PIT válido
  → Si High[t] viola lógica temporal, RECHAZADA como look-ahead

- [ ] Datos con survivorship bias (eliminaron valores delisted)
  → Sistema detecta gap inexplicable (precio baja 50% de repente)
  → Alerta: "Posible delisting o error de datos"

- [ ] Close[t] > High[t]
  → IMPOSIBLE PIT-real (close NUNCA puede superar high)
  → RECHAZADA automáticamente

- [ ] **(ADR-0155, enmienda 2026-07-25) Cotización cruzada en `QuoteTick`: `bid > ask`.**
  → Físicamente imposible en un mercado real (el comprador nunca paga menos que lo que el vendedor pide)
  → Puede surgir de un arrastre (`carry-forward`, `data-normalization-layer` TTR-004) que se dejó correr más allá de `QUOTE_SIDE_CARRY_FORWARD_MAX_NS`
  → RECHAZADA automáticamente, con el `bid_source`/`ask_source` de la fila incluidos en el motivo

- [ ] **(ADR-0156 Restricción 6) Rango de precio real con `volume_ticks=0` en `Bars`.** Verificado real: `datasets/EURUSD_M1_....csv`, 2 filas de 9.2 millones (`2019-11-01`, coincide con el cambio de horario DST de EE. UU.) con `open≠close` u `high≠low` pese a `volume_ticks=0`
  → Matemáticamente imposible bajo la propia definición de `volume_ticks` (cuenta cambios de precio/cotización — si hubo movimiento, hubo al menos un cambio)
  → Marcada explícitamente (no descartada en silencio ni persistida como vela ordinaria), motivo "actividad de precio sin contador de actividad"

- [ ] **Gate de Coherencia de Calendario (ADR-0162).** El usuario importa un archivo con una plantilla de sesión declarada (ej. `FOREX_245`) para el instrumento
  → Sistema calcula la huella de huecos **observada** en el archivo (dónde faltan velas)
  → La compara contra la huella **esperada** de la plantilla declarada (`session-catalog`)
  → Si son estructuralmente incompatibles (ej. el archivo no tiene hueco de fin de semana pero la plantilla dice Forex 24/5, o el archivo tiene huecos diarios mientras la plantilla dice Cripto 24/7) → **RECHAZADA con motivo explícito**, en cualquier combinación de clase de activo (comparación simétrica huella-contra-huella, no una lista de pares)
  → Si el archivo es demasiado corto para exhibir su huella (ej. cubre un solo lunes, no puede mostrar el hueco de fin de semana) → veredicto **`INDETERMINADO`**, nunca un `APTO` silencioso

---

## Restricciones

- **NUNCA High[t] < Open[t] o Low[t].** Física: high es máximo, low es mínimo de la barra.
- **NUNCA Close[t] > High[t] o < Low[t].** Close DEBE estar dentro del rango [Low, High].
- **NUNCA volumen < 0.** Volumen es conteo, siempre >= 0.
- **NUNCA precio < 0.** Precios positivos (o cero en caso de suspensión, pero marcado especial).
- **NUNCA timestamp fuera de orden.** Barras deben ser cronológicas.
- **NUNCA información futura en barra presente.** No usar max/min de semana para decisión en lunes.
- **(ADR-0162) NUNCA el Gate de Coherencia de Calendario es una lista de pares prohibidos.** Compara la huella de calendario declarada contra la observada — cualquier clase de activo cruzada con cualquier plantilla, sin caso privilegiado ni combinación no cubierta.
- **(ADR-0162) NUNCA se aprueba en silencio por evidencia insuficiente.** Un archivo demasiado corto para exhibir su huella de calendario recibe `INDETERMINADO`, nunca `APTO`.
- **(ADR-0162) NUNCA la tolerancia del gate es cero por defecto de forma rígida.** Instrumentos con calendarios atípicos (ej. futuros con pausa de mantenimiento corta) necesitan margen configurable — la tolerancia es `CONFIG`, no `FIJO`.
- **(ADR-0155, enmienda 2026-07-25) NUNCA `bid > ask` en `QuoteTick`.** Cotización cruzada — imposible en un mercado real, señal de arrastre (`carry-forward`) mal acotado en `data-normalization-layer`.
- **(ADR-0156 Restricción 6) NUNCA una `Bars` con rango de precio real (`open≠close` u `high≠low`) y `volume_ticks=0` se persiste como vela ordinaria sin marca.** Es matemáticamente imposible bajo la definición de `volume_ticks` — movimiento sin ningún cambio contado.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| OHLC_SANITY_CHECK | true | true/false | Validar OHLC lógica (O<H, L<H, Close ∈ [L,H]) | CONFIG |
| SURVIVORSHIP_CHECK | true | true/false | Detectar gaps que indican delisting/error | CONFIG |
| VOLUME_MINIMUM | 0 | 0-10000 | Volumen mínimo por barra (0 = permitido) | CONFIG |
| MAX_DAILY_MOVE | 0.20 | 0.05-0.50 | Máximo cambio diario permitido (20% = threshold para gap sospechoso) | CONFIG |
| TIMESTAMP_VALIDATION | true | true/false | Verificar orden cronológico de barras | CONFIG |
| FORWARD_BIAS_DETECTION | true | true/false | Detectar información futura inyectada | [FIJO] |
| CALENDAR_COHERENCE_TOLERANCE | 0 huecos atípicos | 0 - N (por definir en Story de implementación) | Cuántos huecos que no encajan exactamente en la huella esperada se admiten antes de que el Gate de Coherencia rechace (ADR-0162) | CONFIG |
| MIN_BARS_FOR_FINGERPRINT | por definir en Story de implementación | — | Cuántas barras mínimas debe cubrir un archivo para que el Gate de Coherencia emita `APTO`/`RECHAZADO` en vez de `INDETERMINADO` | CONFIG |

---

## Ciclo de Vida de la Feature — PIT Data Validator

### Entrada
- Dataset OHLCV bruto (importado de broker, archivo, API)
- Parámetros de validación
- Rango temporal (ej: 2020-2024)

### Proceso
1. **Saneamiento OHLC:** Valida relaciones lógicas Open-High-Low-Close
2. **Detección de gaps:** Busca cambios anormales (>MAX_DAILY_MOVE)
3. **Validación de orden:** Timestamps cronológicos
4. **Detección de contaminación futura:** Verifica que no hay información del futuro
5. **Marca de problemas:** Cada barra etiquetada como VALID / WARNING / REJECT

### Salida
- **CleanedData:** Barras que pasaron validación (VALID)
- **ValidationReport:** Lista de barras rechazadas + razón
- **Quality Score:** Porcentaje de datos válidos (ej: 98.5% válidos, 1.5% rechazados)
- **Contaminationlist:** Qué períodos tienen sospecha de look-ahead

### Contextos de Uso

**Contexto 1: Ingesta de Datos (Módulo Ingest)**
- Entrada: Datos crudos de broker/archivo
- Pregunta: ¿Estos datos son PIT-real?
- Impacto: Rechaza datos contaminados antes de backtesting (previene falsos positivos)

**Contexto 2: Validación Pre-Backtest (Módulo Validate)**
- Entrada: Datos que van a usarse en backtest de estrategia candidata
- Pregunta: ¿Estoy backtestando con datos limpios?
- Impacto: Certifica que backtest será válido

**Contexto 3: Auditoría (Módulo Feedback)**
- Entrada: Histórico de datos usados en estrategia LIVE
- Pregunta: ¿Los datos históricos que alimentaron mi estrategia eran PIT-real?
- Impacto: Valida que decisiones pasadas se basaron en datos limpios

---

---

## Tareas (TTRs)

### **TTR-001: Validación Estructural OHLCV (Sanity Check)**
*   **Descripción:** Verifica relaciones lógicas (O<H, L<H, Close ∈ [L,H]) para evitar datos físicamente imposibles.
*   **Reglas de Negocio:**
    * Los precios deben ser `int64` (centavos/ticks) para consistencia (ADR-0002).
    * Toda barra invalidada DEBE incluir el `audit_hash` del dataset original (ADR-0020).
    * **(ADR-0156 Restricción 6, enmienda 2026-07-25)** Una barra con rango de precio real (`open≠close` u `high≠low`) y `volume_ticks=0` es una imposibilidad física de la propia definición de `volume_ticks` — se marca explícitamente, nunca se persiste como vela ordinaria ni se descarta en silencio.
*   **Entrada:** `ohlcv_data` (Arrow/DataFrame).
*   **Salida:** `is_structurally_sound` (bool), `violation_report`.
*   **Precondición:** Stream de datos cargado en memoria.
*   **Postcondición:** Registro de la auditoría en `pit_audits` con `process_id`.

### **TTR-002: Validación Cronológica Monótona**
*   **Descripción:** Asegura que los timestamps están en orden estricto sin retrocesos ni duplicados.
*   **Reglas de Negocio:**
    * Los gaps temporales > `MAX_GAP` deben marcarse como `DATA_VACUUM`.
    * Cada timestamp debe reconciliarse con el `ntp_sync_offset` (ADR-0013).
*   **Entrada:** `timestamp_series`.
*   **Salida:** `is_chronological` (bool), `gap_indices`.
*   **Precondición:** TTR-001 finalizado.
*   **Postcondición:** Persistencia del `Future-Leakage Score` en los metadatos del dataset.

### **TTR-003: Gate de Coherencia de Calendario (ADR-0162)**
*   **Descripción:** Compara la huella de calendario esperada de la plantilla de sesión declarada (`session-catalog`) contra la huella observada en el archivo importado (dónde hay velas y dónde faltan), en toda combinación de clase de activo — nunca una lista de pares.
*   **Reglas de Negocio:**
    * El patrón declarado-vs-observado reutiliza la misma doctrina que ADR-0154 ya fija para la tabla de capacidades por bróker.
    * Si el archivo es demasiado corto para exhibir su huella, el veredicto es `INDETERMINADO`, nunca `APTO` por omisión.
    * La tolerancia a huecos atípicos es `CONFIG` (`CALENDAR_COHERENCE_TOLERANCE`), nunca cero rígido.
*   **Entrada:** `CalendarFingerprint` esperada (de `session-catalog`), serie de timestamps del archivo importado.
*   **Salida:** `CalendarCoherenceVerdict` (`APTO` / `RECHAZADO` con motivo / `INDETERMINADO`).
*   **Precondición:** El instrumento tiene una plantilla de sesión declarada (`instrument-catalog`/`session-catalog`); si no la tiene, el gate no puede correr y la importación queda bloqueada por falta de declaración, no por un veredicto de coherencia.
*   **Postcondición:** El veredicto queda visible junto al resultado de la importación; un `RECHAZADO` detiene la importación con el motivo estructural exacto (qué hueco sobra o falta).

### **TTR-004: Validación Estructural de `QuoteTick` — Cotización Cruzada (ADR-0155, enmienda 2026-07-25)**
*   **Descripción:** Verifica que ningún `QuoteTick` (observado o completado por arrastre en `data-normalization-layer`) tenga `bid > ask` — físicamente imposible en un mercado real.
*   **Reglas de Negocio:**
    * Aplica a todo `QuoteTick`, sin importar si `bid_source`/`ask_source` son `OBSERVED` o `CARRIED_FORWARD` — el arrastre es la causa más probable de una cruce si `QUOTE_SIDE_CARRY_FORWARD_MAX_NS` se configuró demasiado laxo, pero el chequeo no distingue el origen al aplicar la regla.
*   **Entrada:** `QuoteTick` (stream o batch).
*   **Salida:** `is_crossed` (bool) por fila, `violation_report`.
*   **Precondición:** Ninguna — corre siempre que hay `QuoteTick` que validar, independiente del Gate de Coherencia de Calendario (TTR-003).
*   **Postcondición:** Fila rechazada con motivo explícito si `is_crossed`; registro en `pit_audits`.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `bars_in` | `Bars` | Input | 1 | Dataset OHLCV a validar estructuralmente (TTR-001/TTR-002) |
| `expected_fingerprint_in` | `(interno)` | Input | 0..1 | Huella de calendario esperada, consumida de `session-catalog` cuando el instrumento tiene plantilla declarada — payload de comparación, no cableable de forma independiente |
| `coherence_verdict_out` | `CalendarCoherenceVerdict` | Output | 0..1 | Veredicto del Gate de Coherencia (TTR-003); ausente si no hay plantilla de sesión declarada para el instrumento |
| `quote_ticks_in` | `QuoteTick` | Input | 0..N | `QuoteTick` (observado o completado por arrastre) a validar por cotización cruzada (TTR-004, enmienda 2026-07-25) |

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda auditoría PIT y limpieza registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la auditoría |
| | `created_at` | Timestamp de ejecución |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del veredicto PIT |
| | `audit_chain_hash` | Hash de la cadena de veredictos |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | Ref al dataset externo auditado (PIT) |
| | `transformation_id` | ID del paso de limpieza PIT |
| | `logic_hash` | Hash del algoritmo PIT |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del auditor PIT |

## Gobernanza y Estándares (Fijos)
- **Decisión Arquitectónica Asociada:**
    - ADR-0013: Stack Tecnológico (Nautilus/Polars).
    - ADR-0017: Simulación de Alta Fidelidad (PIT-real).
    - ADR-0020: Inundación de Fundaciones.
    - ADR-0162: Catálogo de Instrumento × Bróker × Sesión — Gate de Coherencia de Calendario (TTR-003).

---

## Dependencias
**Depende de:**
- Ninguna para la validación estructural de `Bars` (TTR-001/TTR-002) — es el filtro de entrada primario.
- [`session-catalog`](../features/session-catalog.md) — para la huella de calendario esperada que consume el Gate de Coherencia (TTR-003); sin plantilla de sesión declarada, ese gate no corre.
- [`data-normalization-layer`](../features/data-normalization-layer.md) — produce el `QuoteTick` (observado o completado por arrastre) que TTR-004 valida por cotización cruzada (enmienda 2026-07-25).

**Consumido por:**
- [`ingest`](../modules/ingest.md) — para filtrado pre-almacenamiento.
- [`backtest-engine`](../features/backtest-engine.md) — para asegurar realismo histórico.
- [`data-import-wizard`](../features/data-import-wizard.md) — corre el Gate de Coherencia antes de sellar una importación con plantilla de sesión declarada.

---

## Nota Crítica

**El look-ahead bias es la razón #1 de estrategias que fracasan en vivo después de verse excelentes en backtesting.** Este feature es no-negociable.
