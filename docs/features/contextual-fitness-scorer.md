# Contextual Fitness Scorer

**Carpeta:** `./features/contextual-fitness-scorer/`
**Estado:** En Diseño / Prioritario
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0057 (Glass-Box — el humano define las prioridades por régimen), ADR-0008 (Configurabilidad Universal)

---

## ¿Qué es?

Motor de **fitness contextual multi-régimen**. En lugar de un único número estático de calidad (como el "Weighted Fitness" de SQX, donde fijas un peso al Drawdown y otro al beneficio para siempre), esta feature evalúa la curva de capital **diseccionada por régimen de mercado** y asigna un *score multidimensional*. El humano configura visualmente qué métrica priorizar en cada régimen; el resultado se presenta como un gráfico de radar interactivo que revela en qué tipo de mercado la estrategia es vulnerable.

**Problema que resuelve:** Una estrategia no debe juzgarse con la misma vara en un mercado alcista que en una crisis. El fitness estático esconde fragilidades específicas de régimen.

**Por qué la necesitamos:** Reusa el catálogo de métricas existente (`institutional-metrics`, `robustness-score-aggregator`) pero las pondera dinámicamente según el régimen, dando al analista una lectura táctica del riesgo.

---

## Comportamientos Observables

- [ ] El usuario define en la UI reglas de prioridad por régimen (ej: "si la volatilidad es alta, priorizar el ratio de Sharpe; si es baja, priorizar el beneficio neto").
  → Esas prioridades quedan activas para el scoring.
- [ ] El sistema disecciona la curva de capital por régimen (provisto por la detección automática o por las zonas etiquetadas manualmente).
  → Calcula métricas por separado en cada régimen.
- [ ] La UI muestra un gráfico de radar (spider chart) con un eje por dimensión/régimen.
  → El analista ve de un vistazo dónde la estrategia se hunde.
- [ ] El score final es multidimensional, no un solo número: se puede expandir cada eje para ver su composición.

---

## Ciclo de Vida de la Feature — Contextual Fitness Scorer

### Entrada
- Curva de capital / operaciones de la estrategia.
- Clasificación de régimen por barra (de `hmm-regime-detection` o de `manual-regime-tagger`).
- Reglas de prioridad por régimen definidas por el humano.

### Proceso
- Particiona la curva según el régimen de cada tramo.
- Calcula las métricas base por régimen reusando el motor de métricas existente.
- Pondera cada métrica según la prioridad configurada para ese régimen.
- Consolida en un score multidimensional y proyecta los ejes del radar.

### Salida
- Score multidimensional por régimen.
- Gráfico de radar interactivo.
- Señalamiento del régimen donde la estrategia es más vulnerable.

### Contextos de Uso
**Contexto 1: Validación (Módulo Validate)**
- Reemplaza el fitness plano por una evaluación sensible al régimen en el veredicto.
**Contexto 2: Gestión de Portafolio (Módulo Manage)**
- Selecciona estrategias que cubren los regímenes débiles del portafolio actual.

---

## Restricciones

- NUNCA colapsa el resultado a un único número que oculte la vulnerabilidad por régimen; el desglose siempre está disponible.
- NUNCA inventa un régimen no provisto por la fuente de clasificación; depende de una clasificación válida de entrada.
- Las prioridades por régimen son configurables, pero la lista de métricas base proviene del catálogo institucional (coherencia total).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| REGIME_PRIORITY_MAP | Sharpe en vol alta, NetProfit en vol baja | mapa editable | Qué métrica pesa más en cada régimen | CONFIG |
| REGIME_SOURCE | automático | automático / manual | De dónde viene la clasificación de régimen | CONFIG |
| RADAR_AXES | un eje por régimen | lista editable | Ejes mostrados en el radar | CONFIG |
| METRIC_CATALOG | institucional | — | Métricas base disponibles | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Partición por régimen + ponderación de métricas, sin IO.
- **Shell (Infraestructura):** Recupera la clasificación de régimen y persiste el score multidimensional.
- **Frontera Pública:** Contrato que recibe curva + régimen por barra + mapa de prioridades y devuelve score por dimensión.

---

## Slice Visual (Flutter / Impeller / FFI)
- Gráfico de radar interactivo renderizado con Impeller; cada eje expandible al hacer clic.
- Editor visual del mapa de prioridades por régimen; eventos vía FFI hacia el Core Rust.
- Transporte del score multidimensional vía `binary-arrow-transport`.
- Modo Headless (SaaS): frontera por gRPC.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** Hereda la del cálculo de métricas base.

## Persistencia (Inundación de Fundamentos — ADR-0020 · Perfil B IA/R&D)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador del score contextual |
| | `created_at` | Timestamp del cálculo |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del mapa de prioridades aplicado |
| | `audit_chain_hash` | Hash encadenado de la secuencia de scores |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Analista que configuró las prioridades |
| | `manifest_id` | Estrategia evaluada |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la partición por régimen |
| | `version_node_id` | Versión del scorer aplicado |

- **Rastro de Evidencia:** Emite a `feedback` el régimen donde la estrategia obtuvo el peor sub-score (vulnerabilidad dominante).

---

## Dependencias
**Consumido por:** `validate`, `manage`.
**Depende de:** `institutional-metrics`, `robustness-score-aggregator`, `hmm-regime-detection`, `manual-regime-tagger`, `binary-arrow-transport`.
**Bloqueantes:** Ninguno.

---

## Tareas (TTRs)

### TTR-001: Disección de la curva por régimen
* **¿Cuál es el problema?** Evaluar la estrategia en promedio esconde que en crisis se hunde; hay que separar por régimen.
* **¿Qué tiene que pasar?** El sistema parte la curva según el régimen de cada tramo y calcula las métricas por separado en cada uno.
* **¿Cómo sé que está hecho?**
  - [ ] Veo métricas distintas para "vol alta" y "vol baja" de la misma estrategia.
  - [ ] La fuente de régimen (auto o manual) es seleccionable.
* **¿Qué no puede pasar?** No puede mezclar regímenes en un solo promedio que oculte la vulnerabilidad.

### TTR-002: Prioridades de métrica por régimen configurables
* **¿Cuál es el problema?** El analista quiere que en crisis pese la estabilidad (Sharpe) y en calma pese el beneficio.
* **¿Qué tiene que pasar?** El usuario define qué métrica prioriza en cada régimen y el score las pondera en consecuencia.
* **¿Cómo sé que está hecho?**
  - [ ] Configuro "Sharpe 90% en vol>30" y el score refleja esa ponderación.
  - [ ] Cambiar la prioridad cambia el score resultante.
* **¿Qué no puede pasar?** No puede usarse una métrica fuera del catálogo institucional.

### TTR-003: Gráfico de radar multidimensional
* **¿Cuál es el problema?** Un solo número de fitness no dice en qué mercado falla la estrategia.
* **¿Qué tiene que pasar?** La UI muestra un radar con un eje por régimen/dimensión, y cada eje se puede expandir para ver su composición.
* **¿Cómo sé que está hecho?**
  - [ ] Veo el radar y detecto visualmente el régimen más débil.
  - [ ] El régimen más débil se emite a feedback.
* **¿Qué no puede pasar?** No puede ocultarse el desglose detrás de un único número.
