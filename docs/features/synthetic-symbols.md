# Símbolos Sintéticos

**Carpeta:** `./features/synthetic-symbols/`
**Estado:** En Diseño
**Última actualización:** 2026-07-21
**Decisión Arquitectónica Asociada:** ADR-0159 (Símbolos Sintéticos por Fórmula), ADR-0161 (Lienzo de Fórmulas / motor de expresiones), ADR-0113 (AST tipado)

---

## ¿Qué es esta feature?

Un **símbolo sintético** es un instrumento derivado **determinista** definido por una fórmula matemática sobre N símbolos reales × N timeframes. Se comporta como un símbolo más: las estrategias buscan señales en él, y se grafica y opera como cualquier mercado (salida con forma OHLC).

**Problema que resuelve:** combinar cualquier conjunto de símbolos y temporalidades con notación matemática libre (ej. `(√BTCUSD · EURUSD² / Ln(SP500))`) para descubrir señales que no existen en ningún instrumento individual — índices propios que solo el usuario ve (foso defensivo). Es distinto de la generación estocástica con GAN (`gans-universos-sinteticos.md`): un sintético es una transformación **determinista** de series reales.

**Dos modos de creación:**
- **Declarado:** el usuario teclea la fórmula sobre el lienzo matemático libre (ADR-0161), desde nodos del canvas. Él es dueño del sentido — una fórmula mala produce un instrumento basura que ve en la gráfica.
- **Evolucionado:** el motor de búsqueda descubre combinaciones de símbolos por sí mismo (programación genética sobre árboles de expresión, promovida de moonshot a motor activo). Exige guardas anti-sobreajuste que el declarado no.

---

## Narrativa de Producto

**Capacidad:** `CAP-01` — Datos en los que se puede confiar

**Historia de usuario:** Como investigador de estrategias, quiero construir instrumentos propios combinando por fórmula los que ya existen, para buscar señales en relaciones que ningún mercado cotiza por separado.

**El problema (técnico):** el catálogo de instrumentos negociables está dado, y todo el mundo mira los mismos. Pero muchas relaciones económicas reales no cotizan como instrumento: el precio de un activo relativo a otro, la diferencia entre dos vencimientos, una cesta ponderada a medida. Quien solo puede analizar lo que ya está listado compite en el terreno más concurrido que existe, donde cualquier ventaja se agota rápido porque todos la ven.

**La solución (técnica):** instrumentos derivados **deterministas**, definidos por fórmula sobre varios símbolos y temporalidades reales. El resultado se comporta como un instrumento más: se grafica, se analiza y se opera igual. Dos modos con responsabilidad distinta. En el **declarado**, el usuario escribe la fórmula y es dueño de su sentido: una fórmula mala produce un instrumento basura que él verá en la gráfica. En el **evolucionado**, el motor de búsqueda descubre combinaciones por sí mismo, y por eso exige guardas anti-sobreajuste que el declarado no necesita — buscar entre millones de combinaciones posibles es, otra vez, el problema de las pruebas múltiples.

Es distinto de generar datos artificiales: aquí no se inventa nada. Es una transformación determinista de series reales.

**Explicado sin tecnicismos:** solo puedes comprar lo que está en la lista. Pero muchas de las relaciones interesantes no están en ninguna lista.

Un ejemplo cotidiano: el precio del café te dice poco por sí solo. El precio del café **dividido entre** el precio del azúcar te dice algo distinto, sobre cuál conviene más ahora mismo. Esa segunda cosa no cotiza en ningún lado — pero se puede calcular.

Drasus te deja construir esos instrumentos propios con una fórmula, y a partir de ahí se comportan como cualquier otro: los ves en la gráfica y buscas señales en ellos. Es un foso defensivo: si tu instrumento solo existe en tu máquina, la ventaja que encuentres ahí no la está mirando nadie más.

Con una advertencia honesta: si dejas que la máquina invente las combinaciones por sí sola, vuelve el problema de las mil monedas — probando millones de fórmulas, alguna se verá espectacular por pura casualidad. Por eso ese modo lleva frenos que el modo manual no necesita.

---

## Comportamientos Observables

- [ ] El usuario teclea una fórmula sobre varios símbolos/timeframes (desde nodos de canvas), declara cómo los compatibiliza, y obtiene un símbolo sintético disponible como fuente de señal.
- [ ] El sintético solo produce valor en los instantes donde **todos** sus constituyentes ya cerraron barra; en los huecos marca "sin dato", no inventa.
- [ ] La temporalidad mínima del sintético es la del constituyente **más grueso** (1m + 30m + 1h → mínimo 1h).
- [ ] En modo evolucionado, el motor genera y evalúa combinaciones, controlando la explosión algebraica; cada sintético evolucionado cuenta como un ensayo en la corrección por pruebas múltiples.
- [ ] El linaje del sintético registra **todos** sus constituyentes, no uno solo.

---

## Restricciones

- **NUNCA look-ahead:** ninguna operación de la fórmula usa datos posteriores al instante de decisión (hereda la causalidad de ADR-0161); un parámetro auto-ajustado solo usa datos hasta `t`.
- **NUNCA evaluación en caliente:** la fórmula se compila una vez a AST tipado; nunca se re-evalúa como string en el hot path (ADR-0113), sea tecleada o evolucionada.
- **NUNCA conocibilidad optimista:** el instante conocible del sintético es el **mínimo** de los últimos cierres conocibles de cada constituyente (eslabón más débil), nunca el máximo ni un promedio.
- **NUNCA combinar niveles crudos en silencio:** en modo declarado la fórmula es la intención; en modo evolucionado se aplica una guía de compatibilización de escala para no gastar generaciones en combinaciones sin sentido.
- **NUNCA constituyentes ilimitados:** acotados por `MAX_SYNTHETIC_CONSTITUENTS`; en modo evolucionado, además `MAX_SYNTHETIC_COMPLEXITY`.
- **Prerrequisito de lanzamiento del modo evolucionado:** DEBT-020 (`N_eff` por clustering) debe estar pagada antes de encender la evolución de símbolos.

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MAX_SYNTHETIC_CONSTITUENTS | 3 | 1 - 10 | Máximo de símbolos reales en una fórmula | CONFIG |
| MAX_SYNTHETIC_COMPLEXITY | 10 | 3 - 20 | Máximo de operaciones/profundidad del árbol (modo evolucionado) | CONFIG |
| COMPATIBILIZATION | (declarada) | ratio / log-retorno / z-score / nivel-crudo | Transformación de compatibilización de escala al combinar símbolos | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** evaluación determinista de la fórmula (AST tipado) sobre las series constituyentes alineadas; resolución de conocibilidad por eslabón más débil; salida OHLC.
- **Shell (Infraestructura):** carga de las series constituyentes; en modo evolucionado, el motor genético con `egg` para control de *bloat* y la presión de parsimonia en la selección.
- **Frontera Pública:** interfaz que recibe la definición del sintético (constituyentes, fórmula, compatibilización) y produce la serie sintética como fuente.

---

## Ciclo de Vida de la Feature — Símbolos Sintéticos

### Entrada
- Definición: constituyentes (símbolo × timeframe), fórmula (AST) y transformación de compatibilización.
- Series históricas de los constituyentes.

### Proceso
- Alinea los constituyentes por reloj; en cada instante conocible (eslabón más débil) evalúa la fórmula.
- Produce una serie sintética con forma OHLC, respetando causalidad.

### Salida
- Símbolo sintético (instrumento OHLC) disponible como Fuente de Señal en la topología (ADR-0158).
- Linaje N-ario con todos los constituyentes.

### Contextos de Uso
- **Ingesta / generación (ingest / generate):** el sintético se crea aquí; el modo evolucionado corre en la minería.
- **Fuente en la topología (validate / execute):** las estrategias lo consumen como cualquier símbolo.

---

## Tareas (TTRs)

### **TTR-001: Evaluación Determinista con Conocibilidad por Eslabón Más Débil**
*   **¿Cuál es el problema?** El sintético no puede ver un constituyente que aún no cerró barra, ni rellenar huecos.
*   **¿Qué tiene que pasar?** Alinear constituyentes por reloj; evaluar la fórmula solo en el instante donde todos cerraron (mínimo de cierres); marcar "sin dato" donde no.
*   **¿Cómo sé que está hecho?**
    - [ ] Un sintético de 1m+1h solo produce barras al ritmo de 1h; nunca antes.
    - [ ] Si un constituyente falta en `t`, el sintético marca "sin dato" y no forward-fill.

### **TTR-002: Modo Evolucionado con Control de Bloat y Parsimonia**
*   **¿Cuál es el problema?** La evolución libre de fórmulas explota en un espacio no numerable y produce casi-duplicados que engañan la corrección de sesgo.
*   **¿Qué tiene que pasar?** Motor genético sobre árboles de expresión con `egg` (saturación algebraica), tope `MAX_SYNTHETIC_COMPLEXITY`, presión de parsimonia en la selección, y conteo de cada sintético como ensayo (linaje) — con DEBT-020 pagada antes de encender.
*   **¿Cómo sé que está hecho?**
    - [ ] El árbol de la fórmula evolucionada no crece sin control entre generaciones.
    - [ ] El N de una estrategia que consume un sintético se reconstruye como la unión de su pool y el de la Expedition del sintético.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `constituents_in` | `Bars` | Input | `1..N` | Series de los símbolos reales constituyentes |
| `synthetic_out` | `Bars` | Output | `1` | El símbolo sintético como instrumento OHLC, consumible como fuente |

## Cáscara Visual (Thin Shell)

> **Pendiente de la Etapa 0.5** — la produce el skill `ui-designer`. No rellenar a mano.

### Contexto de superficie (ADR-0136)
**Canvas — Vista Interior** — nodos de combinación de símbolos donde el usuario teclea la fórmula y ve el sintético graficado.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil A (Datos / Ingest) — Grupo I completo + Linaje (`data_snapshot_id`, `transformation_id` = "synthetic", `logic_hash`) + Hardware (`node_id`, `process_id`). **Linaje N-ario:** tabla de unión constituyente↔sintético (ADR-0159 extiende ADR-0020; `parent_id` singular no basta).

## Dependencias y Bloqueantes
**Depende de:**
- [`formula-canvas`](./formula-canvas.md) — el motor de expresiones / lienzo matemático libre (ADR-0161).
- [`ast-compiler`](./ast-compiler.md) — compilación de la fórmula a AST tipado.
- **Prerrequisito del modo evolucionado:** DEBT-020 pagada.

**Consumido por:** [`signal-topology`](./signal-topology.md) — el sintético es una Fuente de Señal más; ingest, generate.

**Contrato de Integración UI (ADR-0117):**
- **Superficie propia:** nodos de combinación de símbolos en el canvas. SVF: teclear una fórmula y crear un sintético real vía `public_interface`; el usuario ve el instrumento graficado; persiste como fuente disponible tras recargar.
