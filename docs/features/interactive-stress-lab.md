# Interactive Stress Lab

**Carpeta:** `./features/interactive-stress-lab/`
**Estado:** En Diseño / Prioritario
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0057 (Glass-Box AI Translator — soberanía humana), ADR-0008 (Configurabilidad Universal)

---

## ¿Qué es?

Panel de control **táctil y reactivo** que permite al analista deformar la curva de capital de una estrategia **en tiempo real** moviendo deslizadores de fricción y de shock macro. El motor matemático ya existe (`monte-carlo-simulator`, `slippage-models`, `adversarial-noise-agent`); esta feature aporta la **capa de interacción humana inmediata**: el humano "siente" con la mano dónde se rompe el sistema, sin esperar simulaciones por lotes.

**Problema que resuelve:** En la herramienta competidora (SQX), el estrés es pasivo y post-mortem — configuras 250 simulaciones, esperas una hora y miras un gráfico de espaguetis o un veredicto de confianza al 95%. Aquí el feedback es instantáneo: muevo un slider y veo la curva deformarse al momento.

**Por qué la necesitamos:** Devuelve el control táctil al analista (filosofía Caja de Cristal). No hay magia de IA opaca; es matemática interactiva que el humano dirige.

---

## Comportamientos Observables

- [ ] El usuario ve la curva de capital al centro y un panel lateral de deslizadores: Slippage, Spread, Ruido OHLC.
  → Al arrastrar el slider de Slippage de 1 a 2 pips, la curva se recalcula y se deforma visiblemente en menos de un parpadeo.
- [ ] Existe un grupo de **Deslizadores Macro** (ej: "Shock de Tipos de Interés", "Cisne Negro de Liquidez").
  → Al moverlos, el sistema reaplica el shock sobre los datos (o sobre el gemelo digital si está activo) y reevalúa el Drawdown en segundos.
- [ ] El panel muestra en vivo las métricas clave que cambian con cada movimiento: Drawdown máximo, Profit Factor, Sharpe.
  → El usuario observa cuál slider rompe primero la estrategia.
- [ ] El usuario puede "fijar" un escenario de estrés (snapshot) para compararlo lado a lado con el escenario base.

---

## Ciclo de Vida de la Feature — Interactive Stress Lab

### Entrada
- Lista de operaciones / curva de capital base de una estrategia ya backtesteada.
- Valores actuales de los deslizadores (fricción y macro).
- Opcional: serie de gemelo digital sintético provista por el moonshot `gans-universos-sinteticos`.

### Proceso
- Aplica las perturbaciones de fricción (slippage, spread) y de ruido OHLC sobre las operaciones base reusando los motores existentes.
- Para shocks macro, deforma la serie de precios según el factor del deslizador y vuelve a ejecutar la valuación de la curva.
- Recalcula métricas de riesgo de forma incremental para mantener la respuesta interactiva.

### Salida
- Curva de capital deformada en vivo.
- Métricas de estrés actualizadas (Drawdown, Profit Factor, Sharpe) por escenario.
- Snapshots comparables de escenarios fijados por el usuario.

### Contextos de Uso
**Contexto 1: Validación (Módulo Validate)**
- El analista estresa manualmente una candidata antes de aprobarla, identificando su punto de quiebre.
**Contexto 2: Retroalimentación (Módulo Feedback)**
- Reconstruye qué nivel de fricción explica una degradación observada en vivo.

---

## Restricciones

- NUNCA modifica los datos históricos originales: el estrés se aplica sobre copias en memoria; la fuente permanece inmutable.
- NUNCA la deformación interactiva sustituye a la simulación completa por lotes; es exploración táctil, no veredicto estadístico final.
- El recálculo interactivo debe sentirse instantáneo; si el conjunto de operaciones es demasiado grande, se degrada con muestreo visual antes que perder reactividad.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| SLIPPAGE_RANGE | 0–3 pips | 0–20 pips | Rango del deslizador de slippage | CONFIG |
| SPREAD_RANGE | 0–2 pips | 0–30 pips | Rango del deslizador de spread | CONFIG |
| OHLC_NOISE_RANGE | 0–50% ATR | 0–100% ATR | Intensidad máxima del ruido OHLC | CONFIG |
| MACRO_SHOCK_FACTORS | tipos, liquidez | lista editable | Catálogo de deslizadores macro disponibles | CONFIG |
| INTERACTIVE_REFRESH_BUDGET | 1 frame | — | Límite de tiempo de recálculo antes de aplicar muestreo visual | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Funciones puras de perturbación de operaciones y revaluación de curva, sin IO.
- **Shell (Infraestructura):** Orquesta la invocación a los motores de Monte Carlo / slippage y emite snapshots.
- **Frontera Pública:** Contrato que recibe curva base + vector de deslizadores y devuelve curva deformada + métricas.

---

## Slice Visual (Flutter / Impeller / FFI)
- Lienzo de curva de capital renderizado con Impeller para repintado a alta tasa de frames.
- Deslizadores nativos cuyo evento de arrastre cruza el puente FFI (`flutter_rust_bridge`) hacia el Core Rust.
- Transporte Zero-Copy de la curva recalculada vía `binary-arrow-transport` para evitar serialización pesada.
- Modo Headless (SaaS): la misma frontera se expone vía gRPC para clientes remotos.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. La reactividad exige cómputo en máquina del usuario.
- **Fidelidad (ADR-0017):** Alta — reusa los modelos de fricción institucionales existentes.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2 · Perfil Ops/Auditoría)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador del snapshot de estrés |
| | `created_at` | Timestamp del escenario fijado |
| | `audit_hash` | Hash del vector de deslizadores aplicado |
| **II. Soberanía** | `owner_id` | Analista que ejecutó el estrés |
| | `manifest_id` | Estrategia sometida a estrés |
| **IV. Hardware** | `node_id` | ID del hardware donde se computó |

- **Rastro de Evidencia:** Emite a `feedback` el vector de deslizadores que primero rompe la estrategia (punto de quiebre táctil).

---

## Dependencias
**Consumido por:** `validate`, `feedback`.
**Depende de:** `monte-carlo-simulator`, `slippage-models`, `adversarial-noise-agent`, `equity-curve-tracker`, `binary-arrow-transport`.
**Bloqueantes:** Ninguno (los motores subyacentes ya existen).

---

## Tareas (TTRs)

### TTR-001: Deslizadores de fricción recalculan la curva en vivo
* **¿Cuál es el problema?** El analista necesita sentir cómo la fricción real (slippage/spread) erosiona sus ganancias sin esperar una hora de simulaciones.
* **¿Qué tiene que pasar?** Al arrastrar el deslizador de slippage o spread, la curva de capital del centro se redibuja al instante con la nueva fricción aplicada.
* **¿Cómo sé que está hecho?**
  - [ ] Muevo el slippage de 1 a 2 pips y veo la curva bajar inmediatamente.
  - [ ] Las métricas (Drawdown, PF) del panel cambian con el movimiento.
* **¿Qué no puede pasar?** No puede modificarse el dataset histórico original; no puede congelarse la UI durante el arrastre.

### TTR-002: Deslizadores de shock macro deforman el escenario
* **¿Cuál es el problema?** El analista quiere simular un cisne negro o un shock de tipos con su propia mano y ver si la estrategia sobrevive.
* **¿Qué tiene que pasar?** Al mover un deslizador macro, el sistema reaplica el shock sobre la serie (o sobre el gemelo digital) y reevalúa el Drawdown en segundos.
* **¿Cómo sé que está hecho?**
  - [ ] Activo "Cisne Negro de Liquidez" y la curva refleja un valle nuevo coherente con el shock.
  - [ ] El catálogo de deslizadores macro es configurable.
* **¿Qué no puede pasar?** No puede presentarse el resultado interactivo como veredicto estadístico definitivo.

### TTR-003: Snapshots comparables de escenarios
* **¿Cuál es el problema?** El analista necesita comparar el escenario base contra un escenario estresado lado a lado.
* **¿Qué tiene que pasar?** El usuario fija un escenario y lo ve superpuesto/contiguo al base, con sus métricas registradas.
* **¿Cómo sé que está hecho?**
  - [ ] Fijo un escenario estresado y aparece junto al base.
  - [ ] El snapshot queda persistido con su vector de deslizadores y hash de auditoría.
* **¿Qué no puede pasar?** No puede perderse el vector exacto de estrés que generó el snapshot (sin trazabilidad = inútil).
