# Robustness Verdict Engine (Veredicto en Lenguaje Natural)

**Carpeta:** `./features/robustness-verdict-engine/`
**Estado:** En Diseño
**Última actualización:** 2026-04-29
**Decisión Arquitectónica Asociada:** ADR-0058 (Política de Scoring Ponderado de Robustez)

---

## ¿Qué es?

El Robustness Verdict Engine es un motor de interpretación que convierte los resultados crudos del guantelete de robustez en un veredicto en lenguaje natural comprensible por cualquier trader. **Por defecto opera de forma determinista por plantilla** (ADR-0115), sin ninguna dependencia de LLM: toma los 5 resultados de tests más el score ponderado final y emite hallazgos accionables de forma reproducible. Un LLM local soberano (vía `candle` embebido) es un realce **opcional** para prosa más rica, nunca un requisito.

**Problema que resuelve:** Los resultados estadísticos (WFE, PBO, VaR) son opacos para el trader retail. Un score de 82 no explica por sí mismo si la estrategia es buena o por qué. Este motor traduce números a insights accionables, identifica puntos de quiebre y genera justificaciones semánticas.

---

## Comportamientos Observables

- [ ] Recibe los 5 resultados individuales de tests más el score ponderado final.
- [ ] Genera, por **plantilla determinista**, un **Veredicto en Lenguaje Humano** resumiendo los hallazgos. Ejemplo: *"La estrategia sobrevive en el 98% de las mutaciones. El parámetro más sensible es el Trailing Stop. Se recomienda fijarlo en el centro del rango estable (45 pips). Listo para revisión"*.
- [ ] Identifica **Puntos de Ruptura**: condiciones específicas donde el sistema colapsa. Ejemplo: *"Falla críticamente si el spread promedio supera los 2.5 pips"*.
- [ ] Genera una **Justificación Semántica del Score**: explica en lenguaje humano por qué la estrategia obtuvo un score determinado.
- [ ] Identifica el **Parámetro Más Sensible** y recomienda su fijación en el centro del rango estable.

---

## Restricciones

- **FIJO:** El motor DEBE producir, por defecto, un reporte estructurado determinista por plantilla, sin dependencia de LLM (ADR-0115). PROHIBIDO exigir Ollama como requisito y PROHIBIDO depender de APIs externas (OpenAI, Anthropic).
- **FIJO:** Si se habilita el realce, el LLM es local soberano (`candle` embebido) y actúa exclusivamente como traductor semántico, no como generador de lógica de trading. No puede modificar el score, parámetros ni emitir señales.
- **FIJO:** El veredicto base (texto por plantilla) es reproducible: mismo input → mismo texto.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| LLM_ENHANCEMENT_ENABLED | false | true/false | Activa el realce opcional vía LLM local soberano (`candle`); por defecto, reporte determinista por plantilla. | CONFIG |
| LLM_MODEL | configurable | - | Modelo local soberano (`candle`) a usar si el realce está habilitado. Nunca Ollama ni API externa. | CONFIG |
| LLM_TIMEOUT_SECONDS | 30 | 10 - 120 | Tiempo máximo de espera del realce LLM opcional. | CONFIG |
| MAX_RETRIES | 2 | 1 - 5 | Reintentos del realce LLM opcional antes de caer al reporte determinista. | CONFIG |

---

## Ciclo de Vida de la Feature — Robustness Verdict Engine

### Entrada
- Resultados crudos del guantelete de robustez:
  - WFA: WFE Index, OSE Stability, Matriz WFA.
  - MC Trades: Confidence Intervals (P5-P95), Ruin Probability.
  - MC Tóxico: Supervivencia ante eventos diarios letales, Prop Firm compliance.
  - CPCV/PBO: Probability of Overfitting, Deflated Sharpe Ratio.
  - Ockham: Degrees of Freedom Ratio, Complexity Penalty Factor.
- Score ponderado final (0-100) + desglose de contribuciones.
- Metadatos de la estrategia (tipo de activo, temporalidad, número de trades).

### Proceso
- Mapea los resultados por test, el score final y el contexto de la estrategia a una **plantilla determinista** que produce: veredicto textual, puntos de ruptura, parámetro más sensible y recomendaciones. Mismo input → mismo texto.
- (Opcional) Si `LLM_ENHANCEMENT_ENABLED`, pasa el veredicto base a un LLM local soberano (`candle`) para enriquecer la prosa, sin alterar los campos estructurados ni el score.
- Si el realce opcional falla o no está disponible, el reporte determinista por plantilla es la salida final (no es un fallback degradado: es el camino por defecto).

### Salida
- `verdict_text`: Veredicto en lenguaje natural (1-3 párrafos).
- `rupture_points`: Lista de condiciones donde la estrategia colapsa.
- `most_sensitive_parameter`: Parámetro con mayor fragilidad detectada.
- `score_explanation`: Justificación semántica del score obtenido.
- `recommendations`: Acciones recomendadas (fijar parámetros, ajustar stops, rechazar).

### Contextos de Uso

**Contexto 1: Veredicto Final de Validación (Validate)**
- Al finalizar todos los tests y el cálculo del score ponderado, el módulo de validación invoca al Verdict Engine.
- El veredicto se presenta al usuario en la interfaz de Strategy Inspector.
- Los puntos de ruptura se inyectan como reglas de protección (Dominant Rules) en incubate y execute.

---

## Tareas (TTRs)

### **TTR-001: Generador de Veredicto Determinista por Plantilla**
* **¿Cuál es el problema?** El trader necesita un veredicto útil y reproducible, sin depender de runtimes externos ni de la estocasticidad de un LLM.
* **¿Qué tiene que pasar?** El sistema mapea los 5 resultados de tests, el score final y el contexto a una plantilla determinista que redacta el veredicto en español técnico-educativo.
* **¿Cómo sé que está hecho?**
  - [ ] La plantilla consume los 5 resultados de tests y el score.
  - [ ] Produce: veredicto, puntos de ruptura, parámetro más sensible, justificación del score.
  - [ ] Mismo input → mismo texto (reproducible).
* **¿Qué no puede pasar?** No puede depender de Ollama ni de APIs externas. No puede modificar parámetros ni el score.

### **TTR-002: Estructuración de Campos del Veredicto**
* **¿Cuál es el problema?** Necesitamos campos estructurados para almacenamiento y consumo por otros módulos.
* **¿Qué tiene que pasar?** El generador emite directamente los campos estructurados (veredicto, rupturas, parámetro sensible, explicación, recomendaciones); el realce LLM opcional, si está activo, solo reescribe la prosa del campo `verdict_text` sin tocar los demás.
* **¿Cómo sé que está hecho?**
  - [ ] El veredicto textual se almacena completo para visualización.
  - [ ] Los puntos de ruptura se emiten como lista estructurada (condición + umbral).
  - [ ] El parámetro más sensible se identifica con nombre y valor recomendado.
  - [ ] El realce LLM opcional nunca altera los campos estructurados ni el score.
* **¿Qué no puede pasar?** El realce opcional no puede ser una dependencia del camino crítico — su ausencia no degrada el veredicto.

### **TTR-003: Inyección de Puntos de Ruptura como Reglas de Protección**
* **¿Cuál es el problema?** Los puntos de ruptura identificados deben convertirse en protecciones activas, no quedarse como texto informativo.
* **¿Qué tiene que pasar?** Los puntos de ruptura detectados (ej: "spread > 2.5 pips") se traducen a Dominant Rules y se inyectan en los metadatos de la estrategia para ser aplicados por los módulos incubate y execute.
* **¿Cómo sé que está hecho?**
  - [ ] La regla "spread > 2.5 pips → no operar" aparece en los metadatos de la estrategia.
  - [ ] El módulo execute rechaza órdenes si la condición de ruptura se cumple.
  - [ ] Las reglas inyectadas son visibles en el Strategy Inspector.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** El veredicto base es determinista por plantilla, sin runtimes externos. El realce LLM opcional opera 100% local vía `candle` embebido (ADR-0115). Cero Ollama, cero servicios externos.
- **Inundación de Fundaciones (ADR-0020):**
  - Perfil: AI / R&D.
  - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
  - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
  - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
  - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
- **Rastro de Evidencia:** Emite `verdict_text`, `rupture_points`, `most_sensitive_parameter`, `score_explanation` y `recommendations` para consumo del módulo de feedback.
- **Determinismo Asistido por LLM (ADR-0051):** El LLM actúa exclusivamente como traductor semántico. No modifica parámetros ni emite señales.

---

## Dependencias

**Consumido por:** `validate`, `feedback`.
**Depende de:** `robustness-score-aggregator`, `walk-forward-analyzer`, `monte-carlo-simulator`, `complexity-penalization`.