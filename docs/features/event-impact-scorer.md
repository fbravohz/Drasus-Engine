# Event Impact Scorer — Motor Determinista de Coeficiente de Impacto

**Carpeta:** `./features/event-impact-scorer/`
**Estado:** En Diseño
**Última actualización:** 2026-06-18
**Decisión Arquitectónica Asociada:** ADR-0125 (Frontera Determinista — Event Study + Surprise)

---

## ¿Qué es?

Es el corazón determinista de la fusión fundamental: convierte un evento ya estructurado en un **coeficiente de impacto numérico**, por **fórmula**, nunca por opinión. Usa dos métodos canónicos:

1. **Event Study ("cuánto impactó"):** mide el retorno anormal del activo en una ventana alrededor del evento, contra su comportamiento esperado. Aplica a eventos discretos con reacción de precio observable (ej.: la declaración de pandemia y el colapso de mercados que siguió).
2. **Surprise ("cuánto sorprendió"):** para eventos **programados con consenso** (resultados trimestrales, publicaciones macro), calcula la distancia estandarizada entre el dato real y el consenso previo. Es el caso del informe trimestral: el mercado se mueve según lo esperado vs lo publicado.

**Problema:** sin un motor determinista, "el impacto de una noticia" sería un juicio subjetivo irreproducible. Este motor lo vuelve un número auditable.

**Por qué la necesitamos:** es el foso defendible — combina información del mundo con el rigor cuantitativo, sin meter una caja negra subjetiva en el motor.

---

## Comportamientos Observables

- [ ] Recalculo el coeficiente de un evento histórico → obtengo **siempre el mismo número** (determinismo).
- [ ] Un evento con fuerte reacción de precio anormal → coeficiente de impacto alto vía Event Study.
- [ ] Un resultado trimestral muy por encima del consenso → sorpresa alta vía Surprise; uno en línea con el consenso → sorpresa cercana a cero.
- [ ] Puedo auditar **por qué** un evento tiene su coeficiente: la fórmula aplicada y los datos de entrada (precio, consenso) quedan trazados.
- [ ] Un evento sin medida objetiva (solo texto libre, sin reacción ni consenso) → no se puntúa en el núcleo (queda fuera, es R&D).

---

## Restricciones

- **NUNCA** el coeficiente se obtiene por opinión de un modelo; siempre por fórmula (Event Study o Surprise).
- **NUNCA** se usa una cifra revisada o un consenso posterior al evento (look-ahead) — se consume el first-print y el consenso previo del [`fundamental-event-store`](./fundamental-event-store.md).
- **NUNCA** se puntúa un evento sin medida objetiva (sin reacción de precio observable ni consenso).
- El método aplicado (Event Study vs Surprise) se elige por el tipo de evento, de forma determinista, no arbitraria.

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| EVENT_WINDOW | configurable | — | Ventana alrededor del evento para medir retorno anormal | CONFIG |
| EXPECTED_RETURN_MODEL | configurable | — | Modelo de retorno esperado contra el que se mide la anomalía | CONFIG |
| SURPRISE_STANDARDIZER | configurable | — | Cómo se estandariza la distancia real vs consenso (dispersión histórica) | CONFIG |
| SIGNIFICANCE_THRESHOLD | configurable | — | Umbral para considerar el impacto estadísticamente relevante | CONFIG |
| SCORING_METHOD | auto | auto / event_study / surprise | Selección de método; `auto` decide por tipo de evento | CONFIG |

---

## Ciclo de Vida de la Feature — Event Impact Scorer

### Entrada
- Evento estructurado y versionado (first-print, consenso previo) del [`fundamental-event-store`](./fundamental-event-store.md).
- Serie de precio del activo (para Event Study).
- Parámetros de ventana, modelo de retorno esperado y estandarización.

### Proceso
- Selecciona el método según el tipo de evento (reacción de precio → Event Study; consenso programado → Surprise).
- Calcula el retorno anormal acumulado o la sorpresa estandarizada.
- Evalúa la significancia según el umbral configurado.

### Salida
- Coeficiente de impacto (acotado), reproducible y auditable.
- Nivel de significancia del impacto.
- Trazas de la fórmula y datos de entrada usados.

### Contextos de Uso
**Contexto 1: Histórico (Módulo Generate / Validate)**
- Entrada: eventos pasados + precio. Pregunta: ¿cuánto impactó cada evento? Impacto: alimenta backtest y descubrimiento con un indicador fundamental PIT-correcto.

**Contexto 2: Tiempo real (Módulo Execute)**
- Entrada: evento recién publicado + precio actual. Pregunta: ¿cuánto está impactando ahora? Impacto: el indicador se actualiza para la decisión viva.

---

## Tareas (TTRs)

### TTR-001: Motor de Event Study (retorno anormal)
*   **¿Cuál es el problema?** Hay que medir cuánto movió un evento al activo, más allá de su movimiento normal.
*   **¿Qué tiene que pasar?** El sistema calcula el retorno anormal acumulado en la ventana del evento contra el retorno esperado, y lo traduce en un coeficiente.
*   **¿Cómo sé que está hecho?**
    - [ ] Un evento con fuerte reacción anormal produce coeficiente alto; uno sin reacción, cercano a cero.
    - [ ] Recalcular da el mismo número.
*   **¿Qué no puede pasar?** Usar datos de la ventana posteriores al instante de decisión (look-ahead).

### TTR-002: Motor de Surprise (real vs consenso)
*   **¿Cuál es el problema?** En eventos programados, lo que mueve al mercado es la sorpresa frente a lo esperado.
*   **¿Qué tiene que pasar?** El sistema calcula la distancia estandarizada entre el dato real y el consenso previo al evento.
*   **¿Cómo sé que está hecho?**
    - [ ] Un resultado muy por encima del consenso da sorpresa alta; uno en línea, cercana a cero.
    - [ ] Se usa el consenso previo, nunca uno revisado después.
*   **¿Qué no puede pasar?** Estandarizar con datos posteriores al evento.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

**Perfil C. Ops / Hot-Path:** Identidad (I) + Soberanía (II) + Hardware (IV) + Latencia, subset Ejecución (V).

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del cálculo de impacto |
| | `created_at` | Timestamp del cálculo |
| | `updated_at` | Timestamp de última modificación |
| | `audit_hash` | Hash del coeficiente y sus entradas |
| | `audit_chain_hash` | Hash encadenado al cálculo anterior |
| | `event_sequence_id` | Secuencia de recuperación |
| **II. Soberanía** | `owner_id` | Dueño del cálculo |
| | `institutional_tag` | Entorno (PROD/PAPER/CHALLENGE) |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de scoring |
| **V. Forense/Ejecución** | `indicator_state_hash` | Snapshot del coeficiente de impacto resultante |
| | `source_signal_id` | Link al evento origen que generó el coeficiente |
| | `execution_latency_ms` | Latencia del cálculo (relevante en tiempo real) |

**Rastro de Evidencia:** emite a `feedback` el coeficiente, el método aplicado y el evento origen, para diagnosticar si una decisión apoyada en fundamentales tuvo causa real o ruido.

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** cálculo de retorno anormal (Event Study) y de sorpresa estandarizada (Surprise), tests de significancia — sin I/O.
- **Shell (Infraestructura):** carga del evento versionado y la serie de precio; persistencia del coeficiente.
- **Frontera Pública:** contrato para puntuar un evento (histórico o en vivo) y devolver su coeficiente auditable.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% local; el cálculo no sale del nodo.
- **Decisión Arquitectónica Asociada:** ADR-0125 (Frontera Determinista), ADR-0127 (PIT de Eventos), ADR-0020 V2.

---

## Dependencias
**Depende de:**
- [`fundamental-event-store`](./fundamental-event-store.md) — para el evento versionado (first-print, consenso previo).
- [`pit-data-validator`](./pit-data-validator.md) — para garantizar que la ventana de cálculo no contiene look-ahead.

**Consumido por:**
- [`fundamental-indicator-projector`](./fundamental-indicator-projector.md) — combina el coeficiente con la relevancia por activo.

**Contrato de Integración UI (ADR-0117):**
- **Ventana de Verificación:** Feature consumidora [`fundamental-indicator-projector`](./fundamental-indicator-projector.md) (y la superficie de `validate`). El observable concreto: el coeficiente de impacto de un evento con su método aplicado (Event Study / Surprise) y su significancia, visible y persistido tras recargar.
