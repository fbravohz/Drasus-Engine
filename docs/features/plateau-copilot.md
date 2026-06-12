# Plateau Co-Pilot

**Carpeta:** `./features/plateau-copilot/`
**Estado:** En Diseño / Prioritario
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0057 (Glass-Box AI Translator — la IA asiste, el humano dispone), ADR-0008 (Configurabilidad Universal)

---

## ¿Qué es?

Asistente visual de **auditoría topológica manual** de parámetros. El motor de fuerza bruta (no LLM) ya existe (`parameter-optimization`, `hierarchical-parameter-optimization`): mapea todas las variaciones de un par de parámetros. Esta feature aporta el **Mapa de Calor bidimensional** y el flujo de selección manual: la IA sugiere con un recuadro la "zona más segura" (meseta estable), pero **es el humano quien hace clic en el pixel exacto** del mapa para fijar el parámetro que irá a producción.

**Problema que resuelve:** En SQX la optimización entrega gráficos 3D complejos que el usuario debe interpretar a ciegas para elegir el parámetro final, con riesgo de escoger un pico sobre-optimizado (overfitting).

**Por qué la necesitamos:** Visualizar la meseta evita elegir picos frágiles. "La IA asiste la visión, el humano ejecuta el francotirador."

---

## Comportamientos Observables

- [ ] El usuario lanza el barrido de dos parámetros (ej: período de media rápida vs lenta, del 10 al 200).
  → El sistema muestra un Mapa de Calor 2D donde el color codifica la métrica objetivo (ej: Profit Factor o Sharpe).
- [ ] La IA dibuja un recuadro sobre la **zona de meseta más estable** (área amplia de buen desempeño, no un pico aislado).
  → El usuario ve la sugerencia, pero conserva la autoridad final.
- [ ] El usuario hace clic en un pixel exacto del mapa.
  → El sistema fija ("hardcodea") ese par de parámetros como configuración de producción y lo registra.
- [ ] Al pasar el cursor sobre cualquier pixel, un tooltip muestra los valores de parámetro y la métrica exacta de esa celda.

---

## Ciclo de Vida de la Feature — Plateau Co-Pilot

### Entrada
- Resultado del barrido de parámetros (rejilla de combinaciones con su métrica objetivo).
- Métrica objetivo seleccionada por el usuario (configurable).
- Umbral de detección de meseta (qué tan amplia y plana debe ser una región para considerarse segura).

### Proceso
- Renderiza la rejilla como mapa de calor.
- Detecta regiones de meseta: zonas contiguas donde la métrica se mantiene alta y estable frente a perturbaciones de parámetro.
- Resalta la meseta candidata más robusta.
- Captura el clic del humano y lo traduce al par de parámetros elegido.

### Salida
- Mapa de calor interactivo con meseta(s) resaltada(s).
- Par de parámetros fijado por el humano (decisión de producción).
- Registro auditable del pixel elegido vs la sugerencia de la IA.

### Contextos de Uso
**Contexto 1: Validación (Módulo Validate)**
- El analista elige parámetros robustos sobre una meseta antes de aprobar la estrategia.
**Contexto 2: Generación (Módulo Generate)**
- Inspección de la superficie de parámetros de una candidata recién generada.

---

## Restricciones

- NUNCA la IA fija el parámetro de producción por sí sola: la selección de producción siempre requiere el clic humano.
- NUNCA se permite seleccionar un pico aislado sin advertir visualmente que carece de meseta (riesgo de overfitting).
- El mapa de calor debe mantenerse legible aunque la rejilla sea densa (downsampling visual configurable).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| TARGET_METRIC | profit_factor | PF / Sharpe / Ret-DD / etc. | Métrica que colorea el mapa | CONFIG |
| PLATEAU_FLATNESS | 10% | 1%–50% | Variación máxima dentro de una meseta válida | CONFIG |
| PLATEAU_MIN_AREA | 9 celdas | 4–100 celdas | Tamaño mínimo de región para ser "segura" | CONFIG |
| PEAK_WARNING | activado | on/off | Advertir si la selección es un pico sin meseta | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Render del mapa de calor + traducción del clic humano a par de parámetros. La detección geométrica de la meseta se **delega** a `topological-plateau-finder` (no se reimplementa aquí — coherencia §7.2 Cálculo Universal).
- **Shell (Infraestructura):** Recibe la rejilla del optimizador, invoca al finder y persiste la selección humana.
- **Frontera Pública:** Contrato que recibe rejilla + métrica objetivo y devuelve mapa + meseta sugerida.

---

## Slice Visual (Flutter / Impeller / FFI)
- Mapa de calor renderizado con Impeller (acelerado GPU) para rejillas densas.
- Captura de clic y hover con tooltip de coordenadas paramétricas vía FFI hacia el Core.
- Transporte de la rejilla vía `binary-arrow-transport` (Zero-Copy).
- Modo Headless (SaaS): frontera expuesta por gRPC.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** Hereda la del motor de barrido subyacente.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2 · Perfil Ops/Auditoría)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador de la selección de parámetro |
| | `created_at` | Timestamp de la decisión |
| | `audit_hash` | Hash de la rejilla evaluada |
| **II. Soberanía** | `owner_id` | Analista que fijó el parámetro |
| | `manifest_id` | Estrategia afectada |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del par de parámetros elegido |

- **Rastro de Evidencia:** Emite a `feedback` la distancia entre la sugerencia de la IA y la elección humana (sesgo del operador).

---

## Dependencias
**Consumido por:** `validate`, `generate`.
**Depende de:** `topological-plateau-finder` (detección geométrica de meseta), `parameter-optimization`, `hierarchical-parameter-optimization`, `binary-arrow-transport`, `visual-downsampling-service`.
**Bloqueantes:** Ninguno (el motor de barrido y el finder geométrico ya existen).

---

## Tareas (TTRs)

### TTR-001: Renderizar mapa de calor 2D del barrido
* **¿Cuál es el problema?** El analista no puede interpretar gráficos 3D crudos; necesita ver de un vistazo dónde la estrategia es estable.
* **¿Qué tiene que pasar?** Tras el barrido de dos parámetros, aparece un mapa de calor donde el color indica la métrica objetivo de cada combinación.
* **¿Cómo sé que está hecho?**
  - [ ] Lanzo un barrido y veo el mapa coloreado.
  - [ ] Al pasar el cursor, un tooltip muestra los valores y la métrica de esa celda.
* **¿Qué no puede pasar?** No puede volverse ilegible con rejillas densas (debe aplicar downsampling visual).

### TTR-002: La IA resalta la meseta más segura
* **¿Cuál es el problema?** Elegir un pico aislado sobre-optimiza la estrategia; hay que identificar zonas amplias y estables.
* **¿Qué tiene que pasar?** El sistema dibuja un recuadro sobre la región de meseta más robusta según el umbral de planitud y área.
* **¿Cómo sé que está hecho?**
  - [ ] Veo un recuadro sobre una zona amplia de buen desempeño, no sobre un pico.
  - [ ] Cambiar el umbral de planitud cambia la meseta resaltada.
* **¿Qué no puede pasar?** No puede la IA fijar el parámetro automáticamente.

### TTR-003: El humano fija el parámetro con un clic
* **¿Cuál es el problema?** La autoridad final de qué va a producción es del humano (soberanía Glass-Box).
* **¿Qué tiene que pasar?** El usuario hace clic en un pixel y ese par de parámetros queda fijado como configuración de producción, registrado con su hash.
* **¿Cómo sé que está hecho?**
  - [ ] Hago clic y el par queda fijado y persistido.
  - [ ] Si elijo un pico sin meseta y la advertencia está activa, el sistema me avisa antes de confirmar.
* **¿Qué no puede pasar?** No puede perderse el registro de qué eligió el humano vs qué sugirió la IA.
