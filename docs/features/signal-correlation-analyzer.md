# Signal Correlation Analyzer — Análisis de Diversificación

**Carpeta:** `./features/signal-correlation-analyzer/`
**Estado:** En Diseño
**Última actualización:** 2026-04-12

---

## ¿Qué es?

Calcula matrices de correlación entre señales (señal vs señal) y entre señales y factores de mercado. Proporciona auditoría visual de diversificación: ¿Mis señales son independientes entre sí, o estoy usando clones?

**Problema:** Si tengo 10 señales generadas, ¿cuáles son realmente distintas? Si 8 son prácticamente idénticas (correlación 0.99), solo tengo 2 señales únicas. La matriz de correlaciones expone la duplicación.

**User Story:** Como usuario, quiero ver una matriz que me muestre si mis señales son diversas o si estoy doblándolas en cálculos redundantes. También quiero ver cómo correlacionan mis señales con factores (para detectar contaminación de beta).

---

## Comportamientos Observables

- [ ] Usuario visualiza matriz de correlación de 10 señales
  → Diagonal = 1.0 (cada señal consigo misma)
  → Señal 1 vs Señal 2 = 0.95 (casi idénticas, poco diversas)
  → Señal 1 vs Señal 7 = 0.02 (muy distintas, buena diversificación)

- [ ] Usuario ve matriz señales vs factores
  → Señal 3 vs MKT-RF = 0.85 (contaminada con beta, mala)
  → Señal 5 vs MKT-RF = 0.10 (limpia, buena)
  → Dashboard marca Señal 5 con bandera verde, Señal 3 con bandera roja

- [ ] Cálculo de Diversification Ratio
  → 10 señales: std(correlaciones off-diagonal) muy baja → Ratio alto (5.8 = muy diverso)
  → 10 señales clonadas: correlaciones ~0.98 → Ratio bajo (1.1 = nada diverso)

---

## Restricciones

- **NUNCA una correlación NaN.** Si falta datos, ignorar par de señales.
- **NUNCA matriz no-simétrica.** Corr(A,B) = Corr(B,A).
- **Diagonal siempre = 1.0** (cada señal correlaciona perfectamente consigo misma).
- **Correlación siempre ∈ [-1, 1].**

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| CORRELATION_WINDOW | 252 | 50-1000 | Período histórico para calcular correlaciones | CONFIG |
| MIN_SIGNALS | 2 | 2-100 | Mínimo de señales para generar matriz (si hay menos, error) | CONFIG |
| ROLLING_WINDOW | 0 | 0 o 50-500 | Si > 0, calcula rolling correlations (múltiples períodos) | CONFIG |
| DIVERSIFICATION_THRESHOLD | 2.0 | 1.0-5.0 | Umbral mínimo de diversificación (debajo = poco diverso) | CONFIG |

---

## Ciclo de Vida de la Feature — Signal Correlation Analyzer

### Entrada
- Lista de señales (N señales × T barras cada una)
- Lista de factores (5 factores Fama-French × T barras)
- Parámetros de ventana y umbrales

### Proceso
- Calcula matriz N×N de correlaciones inter-señales (Pearson)
- Calcula matriz N×5 de correlaciones señal-factor
- Computa Diversification Ratio = std(señales) / std(correlaciones off-diagonal)
- Genera rolling correlations si aplica (múltiples períodos históricos)

### Salida
- **CorrelationMatrix:** Matriz de correlaciones señal×señal
- **SignalFactorCorrelation:** Matriz señal×factor
- **DiversificationRatio:** Score [1.0-N] (N=muy diverso)
- **RollingCorrelations:** Historial de matrices por período (si aplica)
- Metadata: timestamps, señales con problemas, factores dominantes

### Contextos de Uso

**Contexto 1: Dashboard de Transparencia (Módulo UI)**
- Entrada: Todas las señales en cartera
- Pregunta: ¿Mis señales son realmente diversas?
- Impacto: Usuario ve heatmap visual de redundancia

**Contexto 2: Validación de Robustez (Módulo Validate)**
- Entrada: Señales de una estrategia candidata en período de validación
- Pregunta: ¿Las correlaciones entre mis señales se mantienen estables, o se degradaron?
- Impacto: Detecta si estrategia contaba con relaciones que desaparecieron en período de validación

**Contexto 3: Feedback Loop (Módulo Feedback)**
- Entrada: Señales de estrategia LIVE actual
- Pregunta: ¿Hay aumento de correlación con factores? ¿Perdió su diversificación?
- Impacto: Alerta si estrategia necesita renovación (sus señales se volvieron redundantes)

---

## Tareas (TTRs)

### TTR-001: Calcular Matriz de Correlaciones Inter-Señales

**Qué hace:** Toma N señales y devuelve matriz N×N de correlaciones Pearson.

**Entrada:**
- N series temporales (señales)
- Período de cálculo (ej: últimas 252 barras)

**Salida:**
- Matriz simétrica N×N de correlaciones
- Pares más correlacionados (top 5)
- Pares menos correlacionados (bottom 5)

**Restricciones:**
- Si alguna señal tiene < 20 puntos válidos, ignorarla (marcar como "insufficient data")
- Si correlación es NaN, marcar como "missing" (no 0)

---

### TTR-002: Calcular Matriz Señales vs Factores

**Qué hace:** Calcula cómo cada señal correlaciona con los 5 factores Fama-French.

**Entrada:**
- N señales
- 5 factores Fama-French (Mercado, Tamaño, Valor, Rentabilidad, Inversión)

**Salida:**
- Matriz N×5 de correlaciones
- Señales "limpias" (correlación < 0.20 con todos factores)
- Señales "contaminadas" (correlación > 0.70 con algún factor)

---

### TTR-003: Calcular Diversification Ratio

**Qué hace:** Métrica única que expresa cuán diversas son las señales (1.0 = nada diverso, N = perfecta diversidad).

**Entrada:**
- Matriz de correlaciones inter-señales (del TTR-001)

**Salida:**
- DiversificationRatio ∈ [1.0, N]
- Interpretación: "X señales únicas en efectivo" (si ratio=4.2 con 10 señales, equivale a 4.2 señales verdaderamente distintas)

**Restricciones:**
- Ratio siempre positivo
- Ratio ≤ número de señales

---

### TTR-004: Calcular Rolling Correlations (Opcional)

**Qué hace:** Divide histórico en ventanas no-solapadas, calcula matriz para cada una, devuelve evolución temporal.

**Entrada:**
- Señales
- Tamaño de ventana rolling (ej: 126 barras)

**Salida:**
- Lista de matrices correlación (1 por ventana)
- Gráfico de evolución temporal: ¿Las correlaciones suben o bajan?
- Volatilidad de correlaciones (std de correlaciones inter-ventanas)

---

## Dependencias

**Depende de:**
- `institutional-metrics` (cálculo base de correlaciones)
- `factor-decomposition` (acceso a factores)

**Depende de ella:**
- `validate` (auditoría de señales en validación)
- `feedback` (monitoreo de degradación)
- UI Dashboard (visualización)

---

## Gobernanza y Estándares

- **Inundación de Fundaciones (ADR-0020):** 

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del reporte |
| | `created_at` | Timestamp de cálculo |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de la matriz resultante |
| | `audit_chain_hash` | Hash del timeline de diversificación |
| | `event_sequence_id` | Secuencia del evento de cálculo |
| **II. Soberanía** | `owner_id` | Autor de las señales |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor analítico |
| | `data_snapshot_id` | Contexto de mercado/señales |
| | `indicator_state_hash` | Diversification Ratio score |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker analítico |

