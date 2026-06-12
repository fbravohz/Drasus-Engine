# Zero-Crossing Filter — Aislamiento de Señales Ortogonales

**Carpeta:** `./features/zero-crossing-filter/`
**Estado:** En Diseño
**Última actualización:** 2026-04-12

---

## ¿Qué es?

Filtra señales de trading para detectar aquellas que son ortogonales (independientes) respecto a factores de mercado conocidos. Detecta los momentos en que una señal cruza cero sin arrastrar exposición pasiva a factores.

**Problema:** Una señal que correlaciona 0.95 con el retorno del mercado no es una señal real, es simplemente "seguir el mercado". Si filtramos por correlación baja con factores, aislamos señales que contienen **alpha puro**, no "factor luck".

**User Story:** Como usuario, quiero filtrar automáticamente mis señales generadas para detectar cuáles son realmente independientes del mercado. El sistema debe rechazar señales que simplemente replican factores conocidos.

---

## Comportamientos Observables

- [ ] Usuario proporciona una señal generada (ej: media móvil adaptativa)
  → Sistema compara contra Fama-French 5 (Mercado, Tamaño, Valor, Rentabilidad, Inversión)
  → Si correlación > umbral (default 0.30) con cualquier factor → RECHAZADA
  → Si correlación < umbral con todos → APROBADA, marcada como "ortogonal"

- [ ] Una señal que es simplemente "SPY returns"
  → Sistema detecta correlación = 1.0 con Mercado
  → Veredicto: RECHAZADA (no es alpha, es beta puro)

- [ ] Una señal con comportamiento oscilante (sin relación con mercado)
  → Sistema detecta cruces de cero consistentes
  → Correlación con factores < 0.15
  → Veredicto: APROBADA (ortogonal)

---

## Restricciones

- **NUNCA aprobar una señal cuya correlación con MKT-RF > 0.50.** (Significa que sigue el mercado)
- **NUNCA procesar señal sin datos válidos.** Si hay NaN o valores faltantes, rechazar.
- **Umbral de ortogonalidad es configurable** pero NUNCA debe ser negativo.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ORTHOGONALITY_THRESHOLD | 0.30 | 0.05-0.50 | Máxima correlación permitida con factores (debajo = ortogonal) | CONFIG |
| MARKET_CORRELATION_MAX | 0.50 | 0.30-0.70 | Límite absoluto de correlación con Mercado (MKT-RF) | CONFIG |
| LOOKBACK_WINDOW | 252 | 50-500 | Período histórico para calcular correlaciones | CONFIG |
| MIN_DATAPOINTS | 20 | 10-100 | Mínimo de puntos de datos para validar una señal | CONFIG |

---

## Ciclo de Vida de la Feature — Zero-Crossing Filter

### Entrada
- Señal bruta (serie temporal de valores -1 a 1, o valores continuos)
- Factores de mercado (Fama-French 5 u otros)
- Parámetros de umbral configurables

### Proceso
- Calcula correlación entre señal y cada factor
- Detecta cruces de cero (cambios de signo)
- Valida ortogonalidad (ninguna correlación > umbral)
- Emite veredicto: APROBADA / RECHAZADA + score de ortogonalidad

### Salida
- Señal filtrada (si aprobada) o lista vacía (si rechazada)
- OrthogonalityReport: correlaciones, score [0-1], veredicto, timestamp
- Metadata: factor más correlacionado, distancia al umbral

### Contextos de Uso

**Contexto 1: Generación de Estrategias (Módulo Generate)**
- Entrada: Ecuaciones simbólicas nativas (ADR-0113), señales NSGA-II candidatas
- Pregunta: ¿Esta señal es ortogonal a factores?
- Impacto: Solo incluye señales alpha-puras en estrategias finales

**Contexto 2: Validación (Módulo Validate)**
- Entrada: Señales que ya están siendo utilizadas por estrategias
- Pregunta: ¿Sigue siendo ortogonal en período de validación (ventana rolling)?
- Impacto: Detecta si signal desapareció o se volvió correlacionada (degradación)

---

## Tareas (TTRs)

### TTR-001: Calcular Correlación con Factores de Mercado

**Qué hace:** Toma una señal y los 5 factores Fama-French, calcula correlación de Pearson para cada uno.

**Entrada:**
- Serie temporal de señal (mínimo 20 puntos)
- Series de factores (precargas locales)

**Salida:**
- Correlaciones individuales [Mercado, Tamaño, Valor, Rentabilidad, Inversión]
- Máxima correlación detectada
- Timestamp del cálculo

**Restricciones:**
- NUNCA correlación = NaN (si ocurre, rechazar señal)
- Correlación siempre ∈ [-1, 1]

---

### TTR-002: Validar Ortogonalidad Contra Umbral

**Qué hace:** Verifica si todas las correlaciones están por debajo del umbral configurado.

**Entrada:**
- Correlaciones individuales (del TTR-001)
- Umbral configurable

**Salida:**
- Booleano: ortogonal=true/false
- Score normalizado [0-1] donde 1 = perfecto ortogonal, 0 = perfecto correlacionado

**Restricciones:**
- Si MKT-RF > MARKET_CORRELATION_MAX → automáticamente RECHAZADA (no importan otros factores)

---

### TTR-003: Detectar Cruces de Cero

**Qué hace:** Identifica los puntos en que la señal cambia de signo (cruza cero).

**Entrada:**
- Serie temporal de señal

**Salida:**
- Lista de índices donde ocurren cruces
- Dirección de cada cruce (positivo→negativo o negativo→positivo)
- Frecuencia de cruces (cruces por período)

**Restricciones:**
- Cruces con diferencia < 1 barra se agrupan en uno solo (evitar ruido)

---

## Dependencias

**Depende de:**
- `factor-decomposition` (acceso a datos de factores Fama-French)
- `institutional-metrics` (cálculos de correlación, si comparte lógica)

**Depende de ella:**
- `generate` (filtra señales para estrategias candidatas)
- `validate` (valida robustez de señales en nuevos períodos)

---

## Gobernanza y Estándares

- **Inundación de Fundaciones (ADR-0020 V2):** 

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del filtrado |
| | `created_at` | Timestamp de validación |
| | `audit_hash` | Hash del veredicto de ortogonalidad |
| | `audit_chain_hash` | Hash de la integridad de la señal |
| **II. Soberanía** | `owner_id` | Dueño de la IP de la señal |
| | `manifest_id` | ID del contrato de diseño |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del modelo de factores (FF-5) |
| | `data_snapshot_id` | Ref a los factores de referencia |
| | `indicator_state_hash` | Snapshot del score de ortogonalidad |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del validador de señales |

