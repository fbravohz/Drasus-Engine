# Lienzo de Fórmulas

**Carpeta:** `./features/formula-canvas/`
**Estado:** En Diseño
**Última actualización:** 2026-07-21
**Decisión Arquitectónica Asociada:** ADR-0161 (Motor de Expresiones Compartido y Lienzo Matemático Libre), ADR-0113 (AST tipado)

---

## ¿Qué es esta feature?

Un **motor de expresiones único** que permite al usuario crear artefactos mediante fórmulas matemáticas libres, desde nodos del canvas. El mismo motor sirve a cuatro clases de artefacto:

- **Símbolos sintéticos** (`synthetic-symbols.md`, ADR-0159).
- **Indicadores custom** (el siguiente RSI/MACD/Bandas de Bollinger inventado por el usuario).
- **Señales de entrada/salida** de las estrategias.
- **Fórmulas de gestión de riesgo.**

**Problema que resuelve:** el usuario quiere un lienzo matemático **libre**, no plantillas básicas — aplicar una cuadrática con tres símbolos, una transformada, una integral discreta o una fórmula de econometría, y que el sistema la calcule de forma determinista. Construir un motor por cada caso triplicaría infraestructura; reconocer que es el **mismo motor** reutilizado por dominio lo evita. Empodera la capacidad más difícil del oficio: inventar la matemática del edge.

---

## Narrativa de Producto

**Capacidad:** `CAP-03` — Búsqueda automática en el espacio de estrategias

**Historia de usuario:** Como investigador de estrategias, quiero escribir mi propia fórmula matemática libre —no solo elegir entre indicadores ya hechos— para inventar el indicador, la señal o la regla de riesgo que a nadie más se le ha ocurrido.

**El problema (técnico):** un catálogo cerrado de indicadores predefinidos limita al investigador a combinar piezas que ya existen. Pero la parte más difícil y valiosa del oficio es inventar la matemática propia: una relación entre varios símbolos, una transformada, una fórmula de econometría que no está en ningún catálogo. Construir un motor separado para cada tipo de fórmula —una para símbolos, otra para indicadores, otra para señales— triplicaría el trabajo sin necesidad.

**La solución (técnica):** un único motor de fórmulas libres que el usuario teclea desde nodos del lienzo, capaz de producir cuatro tipos de resultado según el contexto: un símbolo sintético, un indicador con una o varias series (como un MACD, que produce tres), una señal de entrada/salida, o una regla de gestión de riesgo. El sistema no juzga si la fórmula tiene sentido económico —una fórmula sin sentido simplemente produce un resultado visible en la gráfica que el usuario descarta—, pero sí garantiza que ninguna operación de la fórmula pueda usar un dato que todavía no existía en el instante de la decisión.

**Explicado sin tecnicismos:** es como si en vez de elegir entre las recetas de un libro de cocina fijo, te dieran una alacena completa de ingredientes y la libertad de inventar tu propio platillo desde cero, combinándolos como se te ocurra.

Esta pieza te da esa alacena para las matemáticas del mercado: en vez de conformarte con los indicadores de siempre, escribes tu propia fórmula y el sistema la convierte en un instrumento real que puedes ver graficado. Si la fórmula no tiene sentido, simplemente verás un resultado sin utilidad —la libertad es tuya, y también la responsabilidad de que la fórmula diga algo real—. Lo único que el sistema nunca permite es que la fórmula, sin que te des cuenta, haga trampa mirando información del futuro.

---

## Comportamientos Observables

- [ ] Desde nodos de canvas, el usuario teclea una fórmula y obtiene un símbolo sintético, un indicador multi-serie, una señal o una regla de riesgo — con la misma herramienta.
- [ ] El catálogo de operaciones es amplio (funciones trascendentes, transformadas, integrales/derivadas discretas, estadística), no solo aritmética básica.
- [ ] Un indicador puede producir **varias series nombradas** (ej. MACD = EMA corta + EMA larga + diferencial); un símbolo produce una salida con forma OHLC.
- [ ] Una fórmula sin sentido produce un artefacto basura, visible en la gráfica — el sistema no juzga el sentido económico de lo que el usuario teclea.
- [ ] Toda operación con ventana (transformada, media, integral) evaluada sobre la barra N solo ve `1..N`.

---

## Restricciones

- **NUNCA look-ahead (causalidad — la guarda inquebrantable):** ninguna operación puede usar datos posteriores al instante de decisión; una operación con ventana sobre la barra N ve exclusivamente `1..N`, jamás `N+1`. Es lo que hace seguro abrir el catálogo.
- **NUNCA evaluación de strings en el hot path:** toda fórmula se compila una vez a AST tipado (ADR-0113); nunca se re-evalúa como texto por barra, sea tecleada o evolucionada.
- **NUNCA indeterminismo:** misma fórmula + mismos datos → mismo resultado bit a bit, backtest↔vivo.
- **Salida tipada por clase:** símbolo → OHLC; indicador → 1..N series nombradas; señal → veredicto; riesgo → parámetro. La clase del artefacto fija la forma de salida.

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| OPERATION_CATALOG | amplio | catálogo | Conjunto de operaciones matemáticas permitidas | [FIJO] |
| CAUSALITY_CHECK | on | on | Verificación de que ninguna operación mira el futuro | [FIJO] |
| MAX_FORMULA_DEPTH | 20 | 5 - 50 | Profundidad/complejidad máxima del árbol de la fórmula | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** parser de la fórmula a AST tipado; verificador de causalidad; evaluación determinista (vectorizada o secuencial) sobre las series de entrada.
- **Shell (Infraestructura):** compilación cacheada por fórmula; integración con `ast-compiler`; nodos de autoría en el canvas.
- **Frontera Pública:** interfaz que recibe una fórmula (texto/nodos) y las series de entrada, y produce el/los series de salida según la clase de artefacto.

---

## Ciclo de Vida de la Feature — Lienzo de Fórmulas

### Entrada
- Fórmula del usuario (texto o nodos de canvas) y la clase de artefacto (símbolo / indicador / señal / riesgo).
- Series de entrada (símbolos, timeframes, otras series).

### Proceso
- Parsea la fórmula a AST tipado; verifica causalidad (rechaza cualquier operación que mire el futuro).
- Compila una vez y evalúa de forma determinista sobre las series de entrada.

### Salida
- El/los series de salida con la forma que dicta la clase del artefacto (OHLC para símbolo; N series para indicador; veredicto para señal; parámetro para riesgo).

### Contextos de Uso
- **Ingesta / generación:** creación de símbolos sintéticos e indicadores custom.
- **Generación / validación:** señales y reglas de riesgo por fórmula, además de los indicadores clásicos.

---

## Tareas (TTRs)

### **TTR-001: Verificador de Causalidad del Lienzo Libre**
*   **¿Cuál es el problema?** Abrir el catálogo a transformadas/integrales/ventanas invita al look-ahead si una operación mira el futuro dentro de su ventana.
*   **¿Qué tiene que pasar?** Un verificador que, para toda operación con ventana, garantice que sobre la barra N solo se usan datos `1..N`; rechaza en compilación cualquier fórmula que lo viole.
*   **¿Cómo sé que está hecho?**
    - [ ] Una fórmula que intente usar una barra futura es rechazada en compilación con un mensaje claro.
    - [ ] Una transformada sobre ventana produce en la barra N el mismo valor en backtest y en vivo.

### **TTR-002: Compilación Única a AST y Salida Multi-Serie**
*   **¿Cuál es el problema?** Re-evaluar la fórmula como string por barra destruiría el rendimiento; y algunos artefactos (indicadores) producen varias series.
*   **¿Qué tiene que pasar?** Compilar la fórmula una vez a AST tipado (reutilizando `ast-compiler`) y soportar salida de 1..N series nombradas según la clase.
*   **¿Cómo sé que está hecho?**
    - [ ] Un indicador tipo MACD emite sus tres series nombradas desde una sola definición.
    - [ ] La fórmula no se re-parsea por barra durante un backtest.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `series_in` | `Bars` | Input | `1..N` | Series de entrada (símbolos, timeframes, otras series) sobre las que opera la fórmula |
| `bars_out` | `Bars` | Output | `0..N` | Salida con forma OHLC cuando el artefacto es un símbolo sintético |
| `signal_out` | `Signal` | Output | `0..N` | Salida de veredicto cuando el artefacto es una señal de entrada/salida |

> Motor compartido embebido por sus features cliente (`synthetic-symbols`, indicadores, señales, riesgo); la clase del artefacto determina el puerto de salida activo.

## Cáscara Visual (Thin Shell)

> **Pendiente de la Etapa 0.5** — la produce el skill `ui-designer`. No rellenar a mano.

### Contexto de superficie (ADR-0136)
**Canvas — Vista Interior** — nodos de autoría de fórmulas donde el usuario compone la expresión y ve el resultado.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil B (IA / R&D) — Grupo I completo + Linaje (`logic_hash` de la fórmula, `data_snapshot_id`) + Hardware (`process_id`, `node_id`).

## Dependencias y Bloqueantes
**Depende de:**
- [`ast-compiler`](./ast-compiler.md) — compilación a AST tipado y evaluación.

**Consumido por:**
- [`synthetic-symbols`](./synthetic-symbols.md) — símbolos por fórmula.
- Features futuras de indicadores custom, señales por fórmula y riesgo por fórmula (generate/validate).

**Contrato de Integración UI (ADR-0117):**
- **Superficie propia:** nodos de fórmula en el canvas. SVF: teclear una fórmula y crear un artefacto real vía `public_interface`; el usuario ve el resultado graficado; una fórmula con look-ahead es rechazada visiblemente.
