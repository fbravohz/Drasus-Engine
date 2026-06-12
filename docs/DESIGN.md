# 🌌 DRASUS ENGINE: MANIFIESTO FILOSÓFICO Y ARQUITECTURA "BIG PICTURE"

**Contexto del Proyecto para el Agente:**
Estás construyendo "Drasus Engine", una plataforma de gestión para trading cuantitativo y algorítmico institucional. Tu objetivo es diseñar la interfaz **nativa** de una aplicación de escritorio **Flutter** (Windows/macOS/Linux), renderizada por GPU vía **Impeller/CustomPainter** — sin HTML, CSS, DOM ni WebViews (ADR-0029, ADR-0097, ADR-0106). **REGLA CERO: Olvida todo lo que sabes sobre plataformas de trading retail.** No estás diseñando MetaTrader, ni TradingView, ni Bookmap.

## 1. El Concepto Central: "Destronar al Precio"

Las plataformas tradicionales glorifican el gráfico de velas (candlesticks), el libro de órdenes y los indicadores en pantalla porque el humano necesita ver el mercado para operar. En Drasus Engine, el humano **no opera**, los algoritmos lo hacen.

* **Prohibición de Gráficos Tradicionales:** A menos que se especifique lo contrario, está prohibido usar gráficos de velas japonesas. El usuario no necesita ver la "forma" del mercado, necesita ver la "digestión" matemática del mercado.
* **El Mercado como un "Lugar", no un "Tiempo":** No se grafica una línea de tiempo con precios; se grafica en qué "Régimen" está el mercado (ej. Tendencia, Rango, Volátil, Calmo).
* **Salud Sistémica vs. Puntos de Precio:** En lugar de mostrar un "Stop Loss" físico en un gráfico, debes mostrar un **"Sobre de Expectativa"** (un cono de Monte Carlo). Si la estrategia se sale del cono, hay un fallo matemático, no un simple toque de precio.

## 2. El Paradigma Estético: Biolaboratorio Cyberpunk

La estética visual no es la de un banco, ni la de un broker. Es la de una sala de control nocturna, o la pantalla de monitoreo de un reactor nuclear o un laboratorio biológico.

* **Oscuridad Absoluta y Contraste Funcional:** El fondo es el vacío espacial (negros azulados). La luz solo se usa para transmitir datos y estados de alerta.
* **Organismos, no Scripts:** Trata a las estrategias de trading como "células" o "nodos biológicos" vivos. Estas células tienen ADN (genoma de hiperparámetros) y signos vitales (drawdown, sharpe ratio).
* **El Espectro de la Vitalidad:** El color nunca es decorativo. El color dicta la salud de la célula. Debes crear una transición visual entre:
* **Estabilidad Perfeccionada:** Colores cian y teal neón (funcionamiento óptimo).
* **Calma / Transición:** Azules profundos e índigos (modo seguro, incubación, simulación).
* **Alerta y Deriva:** Ámbar y naranja (el algoritmo se está desviando de su modelo, advertencias de riesgo).
* **Fallo Sistémico / Muerte:** Rojos y carmesí (operación letal, slippage severo, "retiro" o muerte de la estrategia).



## 3. La Arquitectura de Navegación: El "ZUI" (Zoomable User Interface)

La interfaz no debe sentirse como una página web con pestañas desconectadas, sino como un mapa fractal infinito donde el usuario hace "zoom in" o "zoom out". El ecosistema tiene 3 niveles de profundidad espacial:

* **Nivel 1 (MACRO) - El Comando de Flota:** La vista de pájaro. Aquí el usuario ve el estado general del cluster. Hay densas tablas de datos y pequeños "chips" (píldoras de colores) parpadeando que indican si hay algoritmos pausados, alertas críticas en la infraestructura SaaS, o cambios drásticos en los regímenes de los activos (ej. "SPX acaba de pasar a Volátil").
* **Nivel 2 (MESO) - El Orchestrator (Editor Visual de DAGs):** El acercamiento a la lógica. Es un lienzo de nodos conectables (DAG) renderizado nativamente con `CustomPainter`, layout automático Dagre y validación de aciclicidad en Rust (`petgraph`) — ADR-0028. En este nivel el usuario también ve gráficamente por qué etapa del conducto de 8 pasos (`Ingestión → Generación → Validación → Incubación → Ejecución → Gestión → Feedback → Retiro`) está pasando el capital o un nuevo algoritmo.
* **Nivel 3 (MICRO) - El Microscopio y la Autopsia:** El zoom máximo sobre un organismo individual. Aquí se despliegan grafos matemáticos intrincados (DAGs) mostrando las conexiones lógicas del algoritmo, barras de signos vitales hiper-específicas, y si la estrategia falló, los "reportes funerarios" o autopsias de por qué murió el algoritmo.

---

### 1. TOKENS DE DISEÑO (HEX CODES)

**Atención Agente Frontend:** Las siguientes reglas son absolutas y no están sujetas a interpretación creativa. Debes aplicar estos tokens y estructuras exactamente como se describen para mantener la coherencia del ecosistema Drasus Engine.

**Nota de Implementación:** Los fragmentos en sintaxis `css` de este documento son una **especificación de diseño (design tokens)**, no código literal a copiar. Toda implementación es 100% Flutter/Dart sobre Impeller (ADR-0029, ADR-0106): traduce `.clase { ... }` → `ThemeData`/`BoxDecoration`/`TextStyle` reutilizables, y SVG/`<i>` → `CustomPainter`/`Canvas`/`Icon` nativos (ADR-0097 prohíbe DOM, CSS y SVG).

El color es semántico. Nunca uses un color neón por razones puramente estéticas; debe estar atado a un estado de los datos.

**A. El Vacío (Fondos y Superficies)**

* `#070B16` - Fondo principal del Body / Lienzo base (Deep Space).
* `#0E1426` - Fondo de Paneles (`.pn`), Tarjetas y Contenedores secundarios.
* `#0A0F1E` - Fondo de Barras laterales o rieles de navegación (`.rail`).
* `#0C1322` / `#0B1120` - Fondos de tarjetas internas (tercer nivel de profundidad).

**B. La Estructura (Bordes y Separadores)**

* `#1A2336` - Borde exterior principal (App shell).
* `#18223A` - Borde de paneles (`.pn`) y tarjetas.
* `#141C30` - Separadores internos sutiles (`border-bottom` de cabeceras).

**C. Texto (Jerarquía de Lectura)**

* `#C9D4E6` - Texto principal (blanco azulado).
* `#AFC0DC` - Texto secundario / Descripciones legibles.
* `#9FB0CC` - Cabeceras de paneles y etiquetas destacadas.
* `#7E8CA6` - Etiquetas (Labels) de métricas o columnas.
* `#8493AE` - Textos de listas o reglas.
* `#5E6E8C` - Texto inactivo, notas al pie, o metadatos de muy baja importancia.

**D. Neones Semánticos (Espectro de Vitalidad)**

* 🟢 **Óptimo / Tendencia:** `#5DE0C8` (Cian principal), `#2DD4BF` (Teal), `#97C459` (Verde).
* 🔵 **Transición / Calmo:** `#9D95F0` (Índigo), `#4D9BE6` (Azul claro), `#8B83E8` (Púrpura suave).
* 🟠 **Alerta / Volátil / Pausa:** `#FAD08A` (Ámbar), `#EF9F27` (Naranja oscuro).
* 🔴 **Crítico / Retiro / Fallo:** `#F39A9A` (Rojo suave/Rosa), `#E24B4A` (Carmesí).

---

### 2. REGLAS TIPOGRÁFICAS

Solo existen dos familias tipográficas. Su uso está estrictamente delimitado:

1. **Texto UI (Lectura y Etiquetas):**
* `TextStyle(fontFamily: <sans empaquetada en assets/fonts>)`. Flutter Desktop no garantiza fuentes de sistema (`system-ui`) en Windows/Linux, por lo que la familia sans-serif debe empaquetarse con la app (a definir vía Naming/Flutter-Engineer).
* *Uso:* Títulos de paneles, descripciones, nombres de estrategias, etiquetas de ejes.
* *Tamaños:* Extremadamente pequeños. Etiquetas en `9.5px`, notas en `9px` o `10px`, texto normal en `11px`.


2. **Datos (Monospace Estricto):**
* `TextStyle(fontFamily: <monospace empaquetada en assets/fonts>)` (estilo reutilizable recomendado: `numStyle`). Por la misma razón que arriba, no depender de `Menlo`/`SFMono` (solo macOS) ni de fuentes de sistema.
* *Uso:* **Cualquier número**, fechas, IDs de nodos (ej. `node-07`), símbolos de trading (`SPX`, `G10`), y porcentajes.
* *Tamaños:* Valores principales numéricos en `15px` o `26px`. Datos en tablas `10px` o `11px`.



---

### 3. COMPONENTES CORE: CONSTRUCCIÓN Y LAYOUT

**A. El Panel Estándar (`.pn`)**
Todo el contenido vive dentro de paneles aislados.

```dart
// Widget reutilizable "Panel" (.pn) — BoxDecoration, no clase CSS
BoxDecoration(
  color: Color(0xFF0E1426),
  border: Border.all(color: Color(0xFF18223A), width: 1),
  borderRadius: BorderRadius.circular(9),
)
// padding: EdgeInsets.all(10), margen inferior: 9 (u 8 según densidad)
```

*Regla de Cabecera de Panel:* Debe incluir un icono (Tabler Icons `ti`) y texto descriptivo en color `#9FB0CC` a `11px`.

**B. Chips y Badges (Estados)**
Los chips son píldoras que indican regímenes o estados del pipeline. Nunca usan fondo sólido brillante.

* **Construcción:** Borde de `1px solid`, fondo muy oscuro teñido del color principal, texto del color principal.
* *Ejemplo para Estado Activo (Cian):* `color: #5DE0C8; background: #0C2B24; border-color: #2A6E5A;`
* *Ejemplo para Estado Inactivo/Neutral:* `color: #6E7C98; background: #0C1322; border-color: #222B42;`
* *CSS Base:* `font-size: 11px; padding: 3px 8px; border-radius: 6px; display: inline-flex; align-items: center; gap: 5px;`

**C. Filas de Datos (Key-Value Rows)**
Las métricas no se muestran en párrafos, se muestran en listas tabulares (clase `.rl` o `.rb`).

```css
.rl {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #141C30;
  font-size: 10px;
}
/* El texto izquierdo usa #8493AE (sans-serif), el valor derecho usa la clase .num (monospace) y color de estado. */

```

**D. Barras de Vitalidad (Micro-Gauges)**
Prohibido usar componentes pesados tipo "dona" o gauges automotrices.

* *Estructura:* Un contenedor flex, un texto a la izquierda, una línea en el centro que actúa de barra, un valor a la derecha.
* *CSS de la barra:* `height: 6px; border-radius: 3px; background: #16203A; overflow: hidden;` con un `<i>` interno representando el porcentaje relleno (ej. `width: 42%; background: #2DD4BF; display: block; height: 100%;`).

**E. Grafos y Renderizado Nativo (Data-Viz)**

* Las líneas de los gráficos (DAGs, Monte Carlo) se renderizan con `CustomPainter`/`Canvas` nativo sobre GPU (Impeller) — **prohibido SVG/DOM** (ADR-0028, ADR-0097, ADR-0106).
* *Grosor de línea:* `Paint()..strokeWidth = 1.5` (o `1.0`).
* *Nodos/Puntos:* Círculos con relleno oscuro (ej. `#1A0E0E`) y borde del color de estado (ej. `#A33D3D`), dibujados con `canvas.drawCircle`.
* *Conos de probabilidad:* Polígonos (`canvas.drawPath`) con color de opacidad reducida (alpha ≈ `0.22`).

---

### 4. REGLAS DE ESTRUCTURA GRID Y DENSIDAD

Drasus Engine abraza la **densidad de información**.

* **Grid:** Usa `Wrap`/`GridView`/`Row` con `spacing`/`runSpacing: 8` o `9`. Las métricas superiores a menudo van en un layout de 5 columnas (`GridView.count(crossAxisCount: 5)` o `Row` con 5 `Expanded`).
* **Alineación:** Los textos numéricos en listas siempre deben estar alineados a la derecha (`Alignment.centerRight`/`TextAlign.right`) para facilitar el escaneo vertical.
* **Iconografía:** Usa exclusivamente el set **Tabler Icons** vía paquete Flutter (`IconData`/`Icon` nativos, no fuentes web `<i class="ti">`). Los iconos deben tener un tamaño proporcional al texto (ej. `11px` o `12px` en cabeceras, `18px` en barras de navegación laterales).

**[FIN DEL DOCUMENTO DE INSTRUCCIONES]**