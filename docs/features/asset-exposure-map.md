# Asset Exposure Map — Mapa de Exposición Evento→Activo

**Carpeta:** `./features/asset-exposure-map/`
**Estado:** En Diseño
**Última actualización:** 2026-06-18
**Decisión Arquitectónica Asociada:** ADR-0128 (Relevancia y Normalización por Instrumento)

---

## ¿Qué es?

Es la pieza que decide, de forma **determinista**, **qué evento fundamental aplica a qué activo y con qué peso**. Cada instrumento lleva un **vector de exposición** (emisor, sector, país, divisa, subyacentes correlacionados, eslabones de cadena de suministro) y cada evento lleva **etiquetas** (entidad, alcance, región). La **relevancia** entre un evento y un activo es el solape entre ambos, modulado por el **alcance** del evento.

**Problema:** el sistema corre la misma estrategia sobre varios activos a la vez. Sin un mapa explícito, decidir qué noticia afecta a cuál sería arbitrario. Una declaración de pandemia (alcance global) afecta a casi todo; los resultados de una empresa (alcance emisor) afectan sobre todo a ese emisor.

**Por qué la necesitamos:** resuelve el problema multi-mercado sin lógica ad-hoc por activo, y de forma reproducible.

---

## Comportamientos Observables

- [ ] Un evento de alcance global (pandemia) → relevancia alta en casi todos los activos de riesgo.
- [ ] Un evento de resultados de un emisor → relevancia alta en ese emisor, media en sus proveedores, casi nula en un par de divisas sin relación.
- [ ] Dos activos distintos con la misma estrategia → cada uno recibe su propio conjunto de eventos relevantes según su exposición.
- [ ] Cambio los pesos de solape (ej.: subo el peso del sector) → la relevancia de los eventos sectoriales cambia de forma coherente y reproducible.

---

## Restricciones

- **NUNCA** la relevancia se decide por criterio discrecional caso a caso; siempre por fórmula sobre las etiquetas de exposición.
- **NUNCA** se asigna un valor de relevancia global idéntico a todos los activos.
- Un evento sin etiquetas de alcance se trata como de relevancia mínima hasta que se etiquete (no se infla por defecto).
- El vector de exposición de un instrumento es dato versionado (cambia con el tiempo: una empresa cambia de sector, un país de régimen cambiario).

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| OVERLAP_WEIGHTS | por dimensión | — | Peso de cada dimensión del solape (emisor, sector, país, divisa, correlación, cadena de suministro) | CONFIG |
| MIN_RELEVANCE | configurable | 0-1 | Umbral mínimo para que un evento entre en el indicador de un activo | CONFIG |
| SCOPE_DECAY | por alcance | — | Cómo decae la relevancia del alcance global → país → sector → emisor | CONFIG |

---

## Ciclo de Vida de la Feature — Asset Exposure Map

### Entrada
- Vector de exposición del instrumento (emisor, sector, país, divisa, correlaciones, cadena de suministro).
- Etiquetas del evento (entidad, alcance, región).
- Pesos de solape y umbral de relevancia configurados.

### Proceso
- Calcula el solape determinista entre las etiquetas del evento y el vector del instrumento.
- Modula por el alcance del evento (global más difuso, emisor más concentrado).
- Aplica el umbral mínimo: por debajo, el evento no entra en el indicador de ese activo.

### Salida
- Coeficiente de relevancia (acotado) por par evento-activo.
- Conjunto de eventos relevantes para un activo dado.

### Contextos de Uso
**Contexto 1: Proyección del indicador (Feature fundamental-indicator-projector)**
- Entrada: coeficiente de impacto del evento + relevancia por activo. Impacto: el indicador final de cada activo solo agrega los eventos relevantes para él.

**Contexto 2: Gestión de portafolio (Módulo Manage)**
- Entrada: relevancia de un evento sobre los activos del portafolio. Impacto: pondera exposición/riesgo según la concentración de eventos relevantes.

---

## Tareas (TTRs)

### TTR-001: Cálculo determinista de relevancia por solape
*   **¿Cuál es el problema?** Hay que decidir qué evento aplica a qué activo sin juicio discrecional.
*   **¿Qué tiene que pasar?** La relevancia sale de una fórmula sobre el solape de etiquetas, modulada por el alcance; misma entrada → mismo resultado.
*   **¿Cómo sé que está hecho?**
    - [ ] Un evento global puntúa relevancia alta en muchos activos; uno de emisor, alta solo en ese emisor.
    - [ ] Recalcular con la misma entrada da idéntico resultado.
*   **¿Qué no puede pasar?** Relevancia global idéntica a todos los activos, o decisión caso a caso fuera de la fórmula.

### TTR-002: Versionado del vector de exposición del instrumento
*   **¿Cuál es el problema?** La exposición de un activo cambia con el tiempo (cambio de sector, de régimen cambiario).
*   **¿Qué tiene que pasar?** El vector de exposición es versionado; una relevancia histórica usa el vector vigente en esa fecha.
*   **¿Cómo sé que está hecho?**
    - [ ] Cambio el sector de un activo y la relevancia histórica anterior no se altera.
*   **¿Qué no puede pasar?** Usar el vector actual para puntuar relevancia de un evento pasado (look-ahead de exposición).

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

**Perfil A. Datos / Ingest:** Identidad (I) + Linaje (III) + Hardware (IV).

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del registro de exposición/relevancia |
| | `created_at` | Timestamp de creación |
| | `updated_at` | Timestamp de última modificación |
| | `audit_hash` | Hash del vector de exposición / coeficiente de relevancia |
| | `audit_chain_hash` | Hash encadenado al registro anterior |
| | `event_sequence_id` | Secuencia de recuperación |
| **III. Linaje** | `version_node_id` | Versión del vector de exposición del instrumento |
| | `parent_id` | Puntero a la versión anterior del vector |
| | `data_snapshot_id` | Snapshot del evento etiquetado evaluado |
| | `logic_hash` | Hash de la fórmula de relevancia aplicada |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de relevancia |

**Rastro de Evidencia:** emite a `feedback` qué eventos fueron relevantes para cada activo y con qué peso, para diagnosticar si una decisión se apoyó en un evento mal mapeado.

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** cálculo del solape y modulación por alcance — sin I/O.
- **Shell (Infraestructura):** persistencia versionada de vectores de exposición y etiquetas de evento.
- **Frontera Pública:** contrato para consultar la relevancia de un evento sobre un activo y el conjunto de eventos relevantes de un activo.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% local; los vectores de exposición son dato sensible del usuario.
- **Decisión Arquitectónica Asociada:** ADR-0128 (Relevancia y Normalización por Instrumento), ADR-0020 V2.

---

## Dependencias
**Depende de:**
- [`fundamental-event-store`](./fundamental-event-store.md) — para las etiquetas de los eventos a mapear.

**Consumido por:**
- [`fundamental-indicator-projector`](./fundamental-indicator-projector.md) — para agregar solo eventos relevantes por activo.
- [`manage`](../modules/manage.md) — para ponderar riesgo/exposición por concentración de eventos relevantes.

**Contrato de Integración UI (ADR-0117):**
- **Ventana de Verificación:** Feature consumidora [`fundamental-indicator-projector`](./fundamental-indicator-projector.md). El observable concreto: la lista de eventos relevantes para un activo dado con su coeficiente de relevancia, visible y persistido tras recargar.
