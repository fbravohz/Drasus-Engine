# Point-In-Time Data Validator — Prevención de Look-Ahead Bias

**Carpeta:** `./features/pit-data-validator/`
**Estado:** Crítico / Prioritario
**Última actualización:** 2026-04-12

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

---

## Restricciones

- **NUNCA High[t] < Open[t] o Low[t].** Física: high es máximo, low es mínimo de la barra.
- **NUNCA Close[t] > High[t] o < Low[t].** Close DEBE estar dentro del rango [Low, High].
- **NUNCA volumen < 0.** Volumen es conteo, siempre >= 0.
- **NUNCA precio < 0.** Precios positivos (o cero en caso de suspensión, pero marcado especial).
- **NUNCA timestamp fuera de orden.** Barras deben ser cronológicas.
- **NUNCA información futura en barra presente.** No usar max/min de semana para decisión en lunes.

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

---

## Dependencias
**Depende de:**
- Ninguna. Es el filtro de entrada primario.

**Consumido por:**
- [`ingest`](../modules/ingest.md) — para filtrado pre-almacenamiento.
- [`backtest-engine`](../features/backtest-engine.md) — para asegurar realismo histórico.

---

## Nota Crítica

**El look-ahead bias es la razón #1 de estrategias que fracasan en vivo después de verse excelentes en backtesting.** Este feature es no-negociable.
