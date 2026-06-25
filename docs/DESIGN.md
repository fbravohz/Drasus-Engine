# Drasus Engine — Style Reference
> biolaboratorio cyberpunk — una sala de control nocturna donde las estrategias viven como células en una galaxia oscura, y el color nunca decora: respira.

**Theme:** dark

**Render obligatorio:** 100% Flutter/Dart sobre Impeller (`CustomPainter`/`Canvas`/`Shader`). Prohibido HTML, CSS, DOM, WebView y SVG (ADR-0029, ADR-0097, ADR-0106). Los `hex`, snippets `dart` y tablas de este documento son **especificación de diseño (design tokens)**, no código literal a copiar.

Drasus Engine es una plataforma de escritorio para trading cuantitativo institucional donde el humano **no opera** — los algoritmos lo hacen — así que la interfaz **destrona al precio**: nada de velas japonesas ni libros de órdenes, sino la "digestión" matemática del mercado (regímenes, conos de Monte Carlo, signos vitales). El lienzo es un casi-negro azul-violeta (Deep Space `#080A18`), nunca negro plano: sobre él flota una **galaxia** tenue de estrellas y refracciones. El *chrome* (navegación, menús, botones, inputs) es **vidrio Apple** (translúcido pero opaco, con desenfoque del fondo y luz de borde), mientras que las rejillas de datos densos son superficies **sólidas y limpias** para que el número mande. El color es un lenguaje semántico estricto — un **espectro de vitalidad** que va del cian óptimo al carmesí letal — encendido como un reactor: **glow de color y gradientes** recorren casi todos los componentes (botones, nodos, líneas, chips, gauges, focos), siempre atados a un estado y usados con disciplina. Cada interacción **se anima** (clic con propagación de luz, hover que enciende, foco con glow). La navegación usa dos superficies (ADR-0136): un **Dashboard** como centro de monitoreo (widgets read-only en bento grid) y un **Canvas unificado** [Forge/Reactor] — lienzo infinito con card-nodes y zoom de dos estados (Vista Relacional ↔ Vista Interior), sin pestañas ni pantallas fijas. El sistema premia la disciplina: una pantalla nueva debe sentirse como mirar un reactor por un microscopio, no como un panel de Material.

> **Fuente única de verdad.** Este archivo absorbe la galería de componentes (antes `DESIGN-COMPONENTS-GALLERY.md`). Para modificar un token, un componente o una regla visual, se edita aquí primero — el código sigue al documento. El catálogo de IDs de componentes vive en §"Catálogo de Componentes". Los `hex`, snippets `dart` y tablas son especificación de diseño, no código literal a copiar.

## Tokens — Colors

| Name | Value | Token | Role |
|------|-------|-------|------|
| Deep Space | `#080A18` | `deepSpace` | Lienzo base del body y del ZUI. Casi-negro de fondo azul-violeta — nunca negro plano. La galaxia se dibuja sobre él |
| Nav Rail | `#0B1022` | `navRail` | Riel de navegación lateral, barras de herramientas verticales |
| Panel Sólido | `#0E1426` | `panelSolid` | Fondo de los Paneles de Datos: tablas, rejillas, contenedores con números. La superficie de trabajo densa |
| Tarjeta Interna | `#11182E` | `cardInner` | Tercer nivel de profundidad: tarjetas dentro de paneles, relleno de nodos del DAG |
| Superficie Elevada | `#161E38` | `surfaceRaised` | Hover de fila, estados elevados, celda activa de tabla |
| Vidrio (chrome) | `0x40F0F2FF` @ 25% | `glassFill` | Vidrio **Apple**: tinte claro sobre fondo oscuro. `BackdropFilter` blur 36 + tinte superior `0x14AAAAFF` + rim-light `0x20A096FF @ 28%`. NUNCA se multiplica la opacidad con `.withOpacity()` — el alpha ya es el correcto. Para nav, menús, botones, inputs, dropdowns, modales, tooltips. El fill es un blanco frío translúcido (~25%), NO un color oscuro — sobre fondo oscuro el tinte claro crea el contraste de vidrio real |
| Borde Panel | `#1B2440` | `borderPanel` | Hairline tintado del Panel de Datos sólido — la "rayita fina" limpia. Jamás gris neutro |
| Separador | `#141C32` | `divider` | Separador interno sutil: cabeceras, filas de tabla, key-value rows |
| Texto Principal | `#E6ECF8` | `textPrimary` | Títulos, cuerpo de alto contraste, valores destacados — blanco azulado, nunca blanco puro |
| Texto Secundario | `#AEBBD6` | `textSecondary` | Descripciones, cabeceras de panel, texto legible de apoyo |
| Etiqueta | `#8492B0` | `textLabel` | Labels de métricas y columnas, lado izquierdo de las key-value rows |
| Inactivo | `#5C6B8C` | `textMuted` | Texto inactivo, placeholders, metadatos de baja importancia |
| Blanco Puro | `#FFFFFF` | `pureWhite` | Solo máximo énfasis sobre un relleno neón (texto de botón de acción) |
| Cian Óptimo | `#54E8D0` | `optimaCyan` | 🟢 Estado óptimo / tendencia / salud perfecta — neón principal de vitalidad |
| Teal | `#2DD4BF` | `optimaTeal` | 🟢 Variante de óptimo para trazos de barra y relleno de gauge |
| Verde Reactor | `#7CF06A` | `reactorGreen` | 🟢 Acción viva: "encendido", confirmar, ejecutar. El interruptor de poder |
| Índigo Transición | `#9A8CFF` | `transitionIndigo` | 🔵 Calma / incubación / simulación / modo seguro — neón principal de transición |
| Azul | `#56A8FF` | `transitionBlue` | 🔵 Variante de transición, enlaces de estado, foco azul |
| Púrpura | `#8B83E8` | `transitionPurple` | 🔵 Variante suave de transición para decoración de baja frecuencia |
| Ámbar Alerta | `#FFC94D` | `alertAmber` | 🟠 Alerta / volátil / pausa / deriva del modelo — neón principal de advertencia |
| Naranja | `#F59423` | `alertOrange` | 🟠 Variante intensa de alerta para deriva avanzada |
| Rojo Crítico | `#FF8A8A` | `criticalRed` | 🔴 Riesgo crítico / retiro — variante suave del fallo |
| Carmesí | `#F0413F` | `criticalCrimson` | 🔴 Fallo sistémico / muerte / slippage letal — el estado terminal |
| Glow | `glow(color)` · `glowStrong(color)` | helper | Halo de color (`BoxShadow` desenfocado) para botones, nodos, líneas, chips, focos. Es la señal de "encendido" Reflect, presente en CASI TODO componente. `glowStrong` es doble capa: núcleo intenso (blur 10) + halo amplio (blur 30) |
| Text Glow | `textGlow(color)` | helper | Sombra de color sobre las letras del neón semántico — el dato/estado encendido brilla |
| Gradiente Óptimo | `#54E8D0 → #2DD4BF` | `gradOptima` | Degradado dentro de la familia óptima: rellenos de gauge, números KPI |
| Gradiente Reactor | `#7CF06A → #54E8D0` | `gradReactor` | Botón de acción viva |
| Gradiente Transición | `#9A8CFF → #56A8FF` | `gradTransition` | Familia incubación/calma: sliders, progreso |
| Gradiente Aurora | `#8B83E8 → #9A8CFF → #56A8FF` | `gradAurora` | Acentos violeta (barra de sección, supernova) |
| Gradiente Alerta | `#FFC94D → #F59423` | `gradAlert` | Familia de alerta |
| Gradiente Crítico | `#FF8A8A → #F0413F` | `gradCritical` | Familia de fallo |
| Orbe de Cristal | `RadialGradient(cian → índigo → púrpura)` + `glowStrong` | crystal orb | Cristal LIMPIO que **sustituye** la aberración cromática RGB (los tres filos desfasados rojo/verde/azul quedaban mal). Gradiente radial + glow potente. Vive en focos, vista MICRO y acentos |
| Gradiente Cósmico | `#E59CFF → #B79CFF → #56A8FF` | `gradCosmic` | Gradiente decorativo para texto ceremonial (splash, onboarding, portada de autopsia, número-héroe KPI vía `ShaderMask`). Nunca como relleno de superficie/botón |
| Star-field | `#E6ECF8` @ 2–5% | `starField` | Puntos de 1–2px del campo de estrellas del telón cósmico |

## Tokens — Typography

Tres voces, servidas por **`google_fonts` en runtime** (cachea en disco; plan de migración a `assets/fonts` cuando se estabilice el set final). La familia exacta la fija Naming/Flutter-Engineer; aquí se fija el rol y el carácter (ADR-0121: identificadores en inglés, prosa en español).

> **Instalado (2026-06-22):** `google_fonts ^8.1.0` (Flutter 3.44.2). Familias asignadas: `displayGrotesque` → **Space Grotesk** · `uiSans` → **Inter** · `dataMono` → **JetBrains Mono**. Implementadas como helpers estáticos en `gallery_tokens.dart` (clase `Gx`): `Gx.displayGrotesque(...)`, `Gx.uiSans(...)`, `Gx.dataMono(...)`.
>
> **Embebido (2026-06-22):** Los `.ttf` (w400/w500 de las tres familias) están en `ui/assets/fonts/` y declarados en `pubspec.yaml`. Los helpers `Gx` usan `TextStyle(fontFamily: ...)` — **la galería es 100% offline**, sin descarga de google_fonts en runtime. `google_fonts` permanece en el pubspec para el resto del proyecto. **Pendiente:** `JetBrainsMono-Medium.ttf` es la versión NerdFont (2.4MB); reemplazar con el .ttf limpio de [fonts.google.com/specimen/JetBrains+Mono](https://fonts.google.com/specimen/JetBrains+Mono) (pesos w400/w500, ~110KB c/u).

### Display — Grotesco Moderno · `displayGrotesque` → **Space Grotesk**
Voz de los titulares. Peso medio (500, **no** bold): "susurra, no grita". Tracking ligeramente negativo en tamaños grandes hace que el título se sienta tallado, no inflado. Tecnológico, limpio, recto.
- **Familia instalada:** Space Grotesk (vía `google_fonts`)
- **Substitute:** Aeonik, Goga, General Sans, Inter Display
- **Weights:** 500
- **Sizes:** 14, 20, 24, 28, 32, 44, 56, 72
- **Line height:** 1.05–1.15
- **Letter spacing:** -0.02em en ≥24px; -0.01em en 14–20px
- **Role:** Títulos de nivel ZUI, cabeceras de sección, título de panel, número-héroe de la vista MICRO y pantallas ceremoniales (splash, autopsia).

### UI — Sans · `uiSans` → **Inter**
La fuerza de trabajo que desaparece en la interfaz. Pesos 400 para texto corrido, 500 para énfasis interactivo y etiquetas de botón.
- **Familia instalada:** Inter (vía `google_fonts`)
- **Substitute:** Inter, Geist, Söhne
- **Weights:** 400, 500
- **Sizes:** 12, 13, 14, 16
- **Line height:** 1.4–1.5
- **Letter spacing:** normal
- **Role:** Cuerpo, descripciones, nombres de estrategias, etiquetas de ejes, labels, texto de botón.

### Datos — Monospace · `dataMono` (estilo reutilizable `numStyle`) → **JetBrains Mono**
La voz de los datos: señala "esto es un número real", no metáfora. Toda cifra y todo identificador van aquí.
- **Familia instalada:** JetBrains Mono (vía `google_fonts`)
- **Substitute:** JetBrains Mono, Berkeley Mono, CommitMono
- **Weights:** 400, 500
- **Sizes:** 11, 13, 16, 28
- **Line height:** 1.3–1.5
- **Letter spacing:** normal
- **Role:** **Cualquier número**, fechas, IDs de nodo (`node-07`), símbolos (`SPX`, `G10`), porcentajes, valores de tabla, número-héroe.

### Type Scale

| Role | Size | Line Height | Letter Spacing | Token |
|------|------|-------------|----------------|-------|
| micro-label | 11px | 1.3 | — | `textMicro` (mono/sans) |
| label | 12px | 1.4 | — | `textLabel` (sans) |
| table-data | 13px | 1.4 | — | `textData` (mono) |
| body | 14px | 1.5 | — | `textBody` (sans) |
| subheading | 16px | 1.5 | — | `textSubheading` (sans/mono) |
| panel-title | 14px | 1.3 | -0.01em | `textPanelTitle` (display 500) |
| section-heading | 20–24px | 1.15 | -0.02em | `textSection` (display 500) |
| micro-hero | 28px | 1.1 | — | `textMicroHero` (mono) |
| zui-title | 32–44px | 1.1 | -0.02em | `textZuiTitle` (display 500) |
| ceremonial | 56–72px | 1.05 | -0.02em | `textCeremonial` (display 500) |

## Tokens — Spacing & Shapes

**Base unit:** 4px

**Density:** dense (cockpit) en MACRO/MESO · comfortable (aire) en MICRO y pantallas ceremoniales — la densidad se **estratifica por nivel ZUI**.

### Spacing Scale

| Name | Value | Token |
|------|-------|-------|
| 4 | 4px | `space4` |
| 8 | 8px | `space8` |
| 9 | 9px | `space9` |
| 12 | 12px | `space12` |
| 16 | 16px | `space16` |
| 24 | 24px | `space24` |
| 32 | 32px | `space32` |
| 48 | 48px | `space48` |
| 64 | 64px | `space64` |

### Border Radius

| Element | Value |
|---------|-------|
| panel sólido | 11px |
| chrome / vidrio | 14–16px |
| botones | 10px |
| inputs | 10px |
| chips / badges | 8px (o 999px para estado vivo) |
| tooltips / popovers | 12px |

### Elevation y Glow (rim-light + halo — NUNCA drop-shadow gris tipo Material)

La profundidad y el "poder" Reflect vienen del **glow de color**, no de sombras grises. El glow es protagonista y se aplica a lo largo de casi todos los componentes (botones, nodos, líneas, chips, gauges, iconos, focos), siempre en el color del estado.

| Name | Value | Token |
|------|-------|-------|
| glass-rim | `Color(0x0DA096FF)` blur 24, interno | `glassRim` |
| glass-edge | filo superior de luz `inset 0 1px 0 rgba(180,170,255,0.12)` | `glassEdge` |
| glow | `BoxShadow(color @ ~45%, blurRadius 16)` | `glow(c)` |
| glow-strong | doble capa: núcleo (`@55%`, blur 10) + halo (`@28%`, blur 30, spread 2) | `glowStrong(c)` |

### Layout

- **Section gap:** 8–9px (denso MACRO/MESO) · 24–32px (ceremonial MICRO/splash)
- **Panel padding:** 10px
- **Grid de métricas:** 5 columnas (`GridView.count(crossAxisCount: 5)` o `Row` con 5 `Expanded`)
- **Alineación numérica:** siempre a la derecha

## Components

### Panel de Datos (Sólido)
**Role:** Contenedor de zonas densas — tablas, rejillas, métricas con números

Relleno sólido `#0E1426`, borde de 1px tintado `#1B2440` (sin gloss, sin sombra), radio 11px, padding 10px. Separación entre paneles 8–9px. La profundidad nace del hairline más la oscuridad de la pila de superficies, nunca de box-shadow. Cabecera: icono Tabler 12–14px + título en `#AEBBD6` a 14px peso 500 (display).

### Panel / Control de Cristal (Vidrio Apple)
**Role:** Todo el chrome — nav ZUI, menús, dropdowns, modales, tooltips, botón secundario

Vidrio **Apple**: translúcido pero opaco. `BackdropFilter` blur 36 + relleno `glassFill` `0x40F0F2FF` + tinte superior `0x14AAAAFF` (milk glass) + rim-light `0x20A096FF @ 28%`. Radio 14–16px. El fill es blanco frío al ~25%: sobre fondo oscuro crea el contraste que define el vidrio. NUNCA aplicar `.withOpacity()` sobre `glassFill` — el alpha ya es el correcto.

### Chip / Badge de Estado
**Role:** Píldora que indica régimen o estado del pipeline

Borde de 1px, fondo oscuro teñido del color de estado, texto del neón. `font-size: 12px; padding: 4px 10px; border-radius: 8px` (o 999px para estado vivo/parpadeante). Nunca fondo sólido brillante. Ejemplo Óptimo (cian): texto `#54E8D0`, fondo `#08251F`, borde `#1E5E4F`. Ejemplo Crítico: texto `#F0413F`, fondo `#2A0C0C`, borde `#7A2A28`.

### Fila Key-Value
**Role:** Métrica tabular — las cifras no van en párrafos

Flex `space-between`: etiqueta izquierda en sans `#8492B0`, valor derecho en mono con el color de estado, separador inferior `#141C32`, 13px, `padding: 6px 0`.

### Tabla Densa
**Role:** Rejilla de datos del Dashboard y listados

Cabecera 11–12px `#8492B0`; filas en mono 13px; números alineados a la derecha; separador de fila `#141C32`; hover de fila `#161E38`. Sin zebra agresivo — la densidad se lee por alineación, no por rayado.

### Input (texto / búsqueda / dropdown)
**Role:** Entrada de datos y filtros

Superficie de vidrio Apple, radio 10px, texto 14px, placeholder `#5C6B8C`. **Foco (real, vía `FocusNode`):** el borde sube a 1.5px en el color del contexto, el relleno se vuelve más opaco y aparece un **glow limpio** alrededor (`glow(color, blur 18)`). **PROHIBIDA la aberración cromática RGB en el foco** — quedaba mal; el foco se señala con glow, no con filos desfasados. El dropdown abre/cierra con animación (`AnimatedSize` + rotación del chevron) y glow al abrir.

### Botón de Acción Viva (Primario)
**Role:** La acción que importa — ejecutar, confirmar, encender

Relleno con **gradiente** del estado (`gradReactor` / `gradOptima`), texto `#080A18`, radio 10px, **glow potente** (`glowStrong`) siempre. Es el "interruptor encendido": escaso, uno por contexto. **Interacción:** al hover el glow se intensifica; al clic se hunde levemente (escala 0.96) y dispara una **propagación de luz** — un pulso de glow que estalla del centro hacia afuera (inspiración Reflect). El botón secundario es vidrio Apple + glow tenue; el terciario es fantasma (texto que se enciende al hover).

### Botón de Cristal (Secundario) y Fantasma (Terciario)
**Role:** Acciones de menor jerarquía

Cristal: vidrio + rim-light, texto `#E6ECF8`. Fantasma: sin fondo, texto `#8492B0`, hover a `#E6ECF8`.

### Barra de Vitalidad (Micro-Gauge)
**Role:** Signo vital de una célula (drawdown, sharpe, etc.)

Etiqueta izquierda, barra central (`height: 6px; border-radius: 3px; background: #16203A`) con relleno del color de estado y leve glow, valor derecho en mono. Prohibido donas o gauges automotrices.

### Grafo / DAG (CustomPainter) — Card-Node Style (N8N / React Flow)
**Role:** Orquestador visual del conducto de 8 pasos y del editor micro de señales

El DAG **no es** un grafo de círculos conectados por palitos. Es un lienzo nodal al estilo N8N / React Flow: nodos-tarjeta rectangulares, conexiones bezier con punta de flecha, fondo con dot-grid. Toda implementación debe seguir esta especificación.

#### Lienzo (`CustomPainter` + `InteractiveViewer`)
- Fondo: `deepSpace #080A18`.
- **Dot-grid:** puntos de 1.5px en `borderPanel #1B2440` separados 20px (inspiración N8N / Figma). Sin líneas de cuadrícula — solo puntos para no contaminar los datos.
- Pan y zoom nativos (`InteractiveViewer`). Snap a grid al soltar un nodo.

#### Anatomía del Nodo-Tarjeta
El nodo es un `Container`/`Stack` de Flutter — **NO** `canvas.drawCircle`:
- **Cuerpo:** `cardInner #11182E`, radio 10px, `borderPanel #1B2440` hairline 1px, padding 10–12px. Ancho estándar 240–280px; alto variable.
- **Header (32–36px):** borde izquierdo de 3px en el color de estado semántico + icono 14px + nombre del nodo en `displayGrotesque 13px 500` + badge de tipo (`chip`).
- **Body:** campos de configuración en key-value (`textLabel` izq, mono 12px der). Máximo 4–5 pares antes de colapsar.
- **Puertos de conexión (handles):** círculos de 10px pegados al borde lateral (`cardInner` + anillo 1.5px del color del tipo de dato). Izquierda = entrada, derecha = salida. Múltiples puertos por nodo permitidos.
- **Estado del nodo:**
  - Reposo: borde 1px `borderPanel`, glow tenue del color de estado.
  - Seleccionado/activo: borde 2px en el color semántico + `glow(color)`.
  - Procesando: `scanRing(estadoColor)` en el puerto de salida activo.
  - Recibe datos: `sonarPulse(estadoColor)` en el puerto de entrada.
  - Inválido / error en cascada: borde `criticalCrimson #F0413F` parpadeante + `glowStrong(criticalCrimson)`.

#### Conexiones (bezier S-curve)
- Curva bezier cúbica (estilo N8N): los puntos de control se separan **horizontalmente** desde cada puerto, creando una S natural entre nodos vecinos.
- Núcleo: `strokeWidth = 2`, color del tipo de dato o `transitionIndigo #9A8CFF` por defecto.
- Halo de glow: misma curva, `strokeWidth = 5–6`, `color.withAlpha(60)`, `blur 4` (`maskFilter`).
- **Punta de flecha** (arrowhead): 8px, rellena con el color de la línea, en el extremo del nodo destino.
- Seleccionada: `strokeWidth = 3`, glow más potente.
- Línea con datos inválidos: `criticalCrimson` parpadeante.
- Hover sobre línea: `strokeWidth = 3`, glow intensificado, tooltip vidrio Apple con el tipo de dato.

#### Interacción de lienzo
| Acción | Respuesta visual |
|---|---|
| Hover nodo | `glowStrong(estadoColor)` + escala 1.02 · 160ms |
| Hover puerto | anillo del puerto → 14px + glow del tipo + cursor crosshair |
| Arrastrar nodo | sombra semántica (glow potente del estado) + snap al soltar |
| Seleccionar nodo | filo neón 2px + glow del estado |
| Clic en conexión | línea se resalta + tooltip tipo de dato |

### Loader / Progreso
**Role:** Trabajo en curso, incubación, escaneo

Anillo o barra con glow pulsante del color de estado (incubación = índigo). Nunca el spinner Material genérico.

### Calendario
**Role:** Eventos, ventanas de mercado, hitos

Rejilla de vidrio; día actual con anillo neón; eventos como puntos del color de estado.

### Breadcrumb del Canvas (Pill de Cristal)
**Role:** Barra flotante de navegación jerárquica (ADR-0136)

Pill de cristal flotante de radio grande con rim-light; muestra la ruta de navegación (`Cluster A › Portfolio B › Strategy 3`). Clic en cualquier segmento → zoom out in-place hasta ese nivel con animación. Sin cambio de pestaña.

### Tooltip / Popover
**Role:** Contexto efímero sobre un dato

Vidrio + rim-light, radio 12px, texto 13px.

---

## Catálogo de Componentes

> Todos los IDs son `kebab-case` estables. `[CORE]` = usado seguro en Drasus. `[STD]` = repertorio estándar (Material, shadcn, Bootstrap) incluido para paleta completa.

### Convención de Spec y Matriz de Estados

Columnas de cada tabla: **id** (nombre kebab-case del widget) · **Role** (qué representa en una frase) · **Variantes / Estados** (versiones a renderizar) · **Tokens** (qué tokens de §"Tokens" consume).

**Matriz global de estados** (todo componente interactivo se renderiza en estos estados):

| Estado | Tratamiento visual |
|---|---|
| default | Reposo — glow tenue del color de estado |
| hover | `glowStrong` intensificado + leve escala · superficie → `surfaceRaised` |
| focus | Borde 1.5px + glow limpio (`glow(color, blur 18)`) · **NUNCA** aberración cromática RGB |
| active / selected | Filo neón 2px + glow del color de estado |
| disabled | Texto `textMuted` · opacidad reducida · sin glow |
| loading | Glow pulsante del color de estado |
| error / crítico | Borde + texto + glow en `criticalCrimson` |

**Ejes semánticos de color** (irrompible — ver §"Tokens — Colors"): `óptimo` → `optimaCyan` · `transición` → `transitionIndigo` · `alerta` → `alertAmber` · `crítico` → `criticalCrimson`. Cada relleno usa el gradiente de su familia (`gradOptima`, `gradTransition`, `gradAlert`, `gradCritical`).

**Animación de interacción** (ver §"Motion Philosophy"): clic de botón = propagación de luz · hover = glow · foco = glow · dropdown = `AnimatedSize` + chevron · calendario = anillo de glow · switch = knob deslizante · slider = arrastre con manija glow.

---

### §4 Layout y estructura

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| panel-solid `[CORE]` | Contenedor de datos denso | default, con cabecera | panelSolid, borderPanel, radio 11 |
| panel-glass `[CORE]` | Contenedor de chrome | default | glassFill, glassRim, radio 14–16 |
| card `[CORE]` | Tarjeta de contenido | default, hover, seleccionada | cardInner, borderPanel |
| stat-card / kpi `[CORE]` | Métrica destacada | óptimo/alerta/crítico | panelSolid, dataMono 28, neón |
| container / box `[STD]` | Caja genérica | — | panelSolid |
| grid / row / column `[CORE]` | Rejilla de layout (5 col métricas) | 2/3/4/5 col | space8/9 |
| stack / wrap `[STD]` | Apilado / envoltura | — | — |
| divider / separator `[CORE]` | Línea separadora | horizontal, vertical | divider |
| spacer `[STD]` | Espacio flexible | — | space scale |
| aspect-ratio-box `[STD]` | Caja de proporción fija | — | — |
| scroll-area `[STD]` | Área desplazable con barra | — | borderPanel |
| splitter / resizable-panes `[CORE]` | Paneles redimensionables (ZUI) | horizontal, vertical | borderPanel |
| sidebar / nav-rail `[CORE]` | Riel lateral | colapsado, expandido | navRail |
| drawer / sheet `[STD]` | Cajón lateral | izq, der, abajo | glassFill |
| appbar / toolbar `[CORE]` | Barra superior | default | navRail, divider |
| status-bar / footer `[CORE]` | Barra de estado inferior | default | navRail, dataMono |
| accordion / collapse `[STD]` | Secciones plegables | abierto, cerrado | panelSolid, divider |
| expansion-panel `[STD]` | Panel expandible | — | panelSolid |
| tabs / tab-bar `[CORE]` | Pestañas | activo (filo neón 2px), inactivo | transitionIndigo, divider |
| segmented-control `[CORE]` | Conmutador de opciones | seleccionado | glassFill, neón |
| stepper / wizard `[CORE]` | Pasos secuenciales | completado, actual, pendiente | optimaCyan, textMuted |
| pipeline-8-steps `[CORE]` | Conducto Ingestión→…→Retiro | etapa activa teñida | espectro de vitalidad |

---

### §5 Navegación

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| navbar / top-nav `[CORE]` | Navegación superior | default | navRail, divider |
| canvas-breadcrumb `[CORE]` | Breadcrumb flotante del canvas (ADR-0136) | segmento activo `textPrimary` | glassFill, glassRim, textSecondary |
| breadcrumbs `[STD]` | Ruta jerárquica | — | textSecondary, textMuted |
| pagination `[STD]` | Paginado | página activa | transitionIndigo |
| menu / dropdown-menu `[CORE]` | Menú desplegable | default, item hover | glassFill, surfaceRaised |
| context-menu `[CORE]` | Menú contextual (click derecho) | — | glassFill |
| command-palette `[CORE]` | Paleta de comandos (Cmd+K) | input + lista | glassFill, dataMono |
| tree-view `[CORE]` | Árbol de navegación | nodo expandido/colapsado/selecto | panelSolid, transitionIndigo |
| bottom-navigation `[STD]` | Navegación inferior | item activo | navRail, neón |
| anchor / scrollspy `[STD]` | Índice de anclas | sección activa | transitionIndigo |
| back-to-top `[STD]` | Botón volver arriba | — | glassFill |

---

### §6 Inputs y formularios

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| text-field / input `[CORE]` | Entrada de texto | default, focus (glow), error, disabled | glassFill, criticalCrimson |
| textarea `[CORE]` | Texto multilínea | default, focus | glassFill |
| number-input / stepper `[CORE]` | Numérico con +/− | default, focus | glassFill, dataMono |
| password-input `[STD]` | Contraseña con toggle | oculto, visible | glassFill |
| search-input `[CORE]` | Búsqueda con icono | default, con valor | glassFill |
| select / dropdown `[CORE]` | Selección única | cerrado, abierto, selecto | glassFill, surfaceRaised |
| combobox / autocomplete `[CORE]` | Selección con filtro | escribiendo, sugerencias | glassFill |
| multiselect `[CORE]` | Selección múltiple con chips | — | glassFill, chips |
| cascader `[STD]` | Selección jerárquica en cascada | — | glassFill |
| transfer / dual-list `[STD]` | Listas de transferencia | — | panelSolid |
| checkbox `[CORE]` | Casilla | off, on, indeterminado, disabled | optimaCyan |
| radio / radio-group `[CORE]` | Opción exclusiva | off, on, disabled | optimaCyan |
| switch / toggle `[CORE]` | Interruptor | off, on, disabled | reactorGreen, textMuted |
| slider `[CORE]` | Deslizador | default, con valor | transitionIndigo, dataMono |
| range-slider `[CORE]` | Rango (dos manijas) | — | transitionIndigo |
| date-picker `[CORE]` | Selector de fecha | cerrado, abierto (calendario) | glassFill |
| time-picker `[STD]` | Selector de hora | — | glassFill, dataMono |
| date-range `[CORE]` | Rango de fechas | — | glassFill |
| color-picker `[STD]` | Selector de color | — | glassFill |
| file-upload / dropzone `[STD]` | Carga de archivos | reposo, arrastrando, cargando | glassFill, transitionIndigo |
| rating `[STD]` | Valoración por estrellas | — | alertAmber |
| tags-input / chips-input `[CORE]` | Entrada de etiquetas | con chips | chips |
| otp / pin-input `[STD]` | Código de un solo uso | — | glassFill, dataMono |
| mention-input `[STD]` | Entrada con @menciones | — | transitionIndigo |
| rich-text-editor `[STD]` | Editor enriquecido (placeholder) | barra + área | panelSolid |
| form-field `[CORE]` | Campo (label + input + helper + error) | normal, error | textLabel, criticalCrimson |
| form-group `[CORE]` | Agrupación de campos | — | divider |

---

### §7 Botones y acciones

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| button-primary `[CORE]` | Acción viva | default, hover, pressed, disabled, loading | reactorGreen/optimaCyan, deepSpace |
| button-glass `[CORE]` | Secundario | default, hover | glassFill, glassRim |
| button-ghost `[CORE]` | Terciario | default, hover | textLabel→textPrimary |
| button-danger `[CORE]` | Acción destructiva | default, hover | criticalCrimson |
| button-link `[STD]` | Botón tipo enlace | default, hover | transitionBlue |
| icon-button `[CORE]` | Botón solo icono | default, hover, active | textSecondary |
| button-group `[STD]` | Grupo de botones | — | borderPanel |
| split-button `[STD]` | Botón con dropdown | — | glassFill |
| toggle-button `[CORE]` | Botón conmutable | off, on | transitionIndigo |
| fab `[STD]` | Botón flotante de acción | — | reactorGreen |
| loading-button `[CORE]` | Botón en carga | spinner/glow | glow pulsante |

---

### §8 Data display

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| table / data-grid `[CORE]` | Tabla densa | default, fila hover, fila selecta, columna ordenada | panelSolid, divider, dataMono, surfaceRaised |
| tree-table `[CORE]` | Tabla jerárquica | nodo expandido | panelSolid |
| list / list-item `[CORE]` | Lista | default, hover, selecto | panelSolid, surfaceRaised |
| key-value-row `[CORE]` | Métrica tabular | normal, con color de estado | textLabel, dataMono, divider |
| description-list `[STD]` | Lista término-definición | — | textLabel, textPrimary |
| badge / tag / chip / pill `[CORE]` | Estado o régimen | óptimo/transición/alerta/crítico, 999px | chips por estado |
| regime-chip `[CORE]` | Chip de régimen de mercado | parpadeante | espectro de vitalidad |
| avatar / avatar-group `[STD]` | Identidad | único, grupo | cardInner |
| tooltip `[CORE]` | Contexto efímero | — | glassFill, glassRim, radio 12 |
| popover `[CORE]` | Contenido flotante | — | glassFill |
| timeline `[STD]` | Línea de eventos | — | transitionIndigo, divider |
| carousel `[STD]` | Carrusel | — | — |
| calendar / scheduler `[CORE]` | Calendario | día actual (anillo neón), eventos (puntos de estado) | glassFill, neón |
| code-block `[CORE]` | Bloque de código/IDs | — | cardInner, dataMono |
| kbd `[STD]` | Tecla de teclado | — | cardInner, dataMono |
| stat / metric `[CORE]` | Cifra grande | óptimo/alerta/crítico | dataMono 28, neón |
| gauge `[CORE]` | Indicador radial/lineal | por estado | espectro de vitalidad |
| micro-gauge / vitality-bar `[CORE]` | Signo vital (drawdown, sharpe) | por estado | barra 6px, neón, dataMono |
| progress-bar `[CORE]` | Progreso lineal | determinado, indeterminado | transitionIndigo |
| progress-circular `[CORE]` | Progreso circular | — | transitionIndigo |
| skeleton `[CORE]` | Esqueleto de carga | — | surfaceRaised, shimmer |
| empty-state `[CORE]` | Estado vacío | — | textMuted, cristal latente |
| image / thumbnail `[STD]` | Imagen | — | borderPanel |

---

### §9 Feedback y overlays

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| alert / banner / callout `[CORE]` | Aviso en línea | info/óptimo/alerta/crítico | espectro de vitalidad, glassFill |
| toast / snackbar `[CORE]` | Notificación efímera | éxito/alerta/error | glassFill, neón |
| notification-card `[CORE]` | Notificación persistente | leída, no leída | panelSolid, transitionIndigo |
| modal / dialog `[CORE]` | Diálogo modal | default, con backdrop | glassFill, glassRim |
| drawer / sheet `[STD]` | Panel deslizante | — | glassFill |
| popconfirm `[STD]` | Confirmación inline | — | glassFill |
| spinner / loader `[CORE]` | Cargando | — | glow pulsante de estado |
| backdrop / scrim `[CORE]` | Velo de fondo | — | deepSpace @ opacidad |
| result / status-page `[STD]` | Página de resultado | éxito, error, vacío | espectro de vitalidad |
| progress-steps `[STD]` | Progreso por pasos | — | optimaCyan, textMuted |

---

### §10 Data-viz (dominio Drasus) `[CORE]`

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| dag-node-graph | Grafo nodal estilo N8N/React Flow — card-nodes rectangulares con header, body key-value y puertos laterales; conexiones bezier S-curve con arrowhead | nodo reposo / hover (glowStrong + escala 1.02) / seleccionado (filo neón 2px) / inválido (criticalCrimson parpadeante) / procesando (scanRing puerto salida) / recibe dato (sonarPulse puerto entrada); línea reposo / hover (gruesa + glow + tooltip) / inválida (criticalCrimson) | cardInner #11182E body; header borde 3px color estado; handles anillo 1.5px tipo dato; bezier strokeWidth 2 + glow blur 4; dot-grid borderPanel 20px; glowStrong hover; sonarPulse + scanRing |
| monte-carlo-cone | Sobre de Expectativa (cono de probabilidad) | dentro/fuera del cono | polígono `gradTransition` alpha 0.05→0.22, estado |
| scatter-umap-pca | Dispersión de clústeres 2D/3D | puntos por estado, lazo | neón, deepSpace |
| sparkline | Mini-serie inline | óptimo/alerta | neón fino |
| drawdown-curve / time-warp | Curva de drawdown | por severidad | espectro de vitalidad |
| heatmap | Mapa de calor | gradiente de estado | espectro |
| regime-map | Mapa de régimen (no línea de tiempo) | Tendencia/Rango/Volátil/Calmo | espectro de vitalidad |
| parallel-coordinates | Coordenadas paralelas | — | transitionIndigo |
| correlation-matrix | Matriz de correlación | — | gradiente |
| equity-curve `[CORE]` | Curva de equity acumulada de una estrategia (P&L normalizado, sin velas) | activo/pausado/muerto; variante log-escala | línea `gradOptima` con `glow(optimaTeal)`; zona de drawdown rellena `gradCritical` alpha 0.08; área positiva `optimaCyan` alpha 0.04 |
| multi-equity-overlay `[CORE]` | Superposición de curvas de equity de múltiples estrategias (vista de portafolio) | por estado de estrategia; hover resalta la curva más cercana al cursor Y | **Áreas apiladas (regla):** en cada segmento X se ordenan las curvas por Y; el área entre la curva j (superior) y j+1 (inferior) se rellena con el color de la curva j alpha 28; la curva inferior rellena hasta el eje. Las bandas cambian de color cuando las curvas se cruzan. Hover: curva activa strokeWidth 2.5 + glowStrong; curvas no activas textMuted |
| wfa-chart `[CORE]` | Walk Forward Analysis: segmentos IS/OOS con resultado por ventana | IS sombreado, OOS con glow; robusto=cian / frágil=carmesí | banda IS `panelSolid` alpha 0.6; banda OOS `gradTransition` alpha 0.12; resultado por segmento como chip del espectro |
| trade-timeline `[CORE]` | Línea de tiempo de trades: marcas de entrada/salida sobre eje temporal | long=cian, short=índigo, SL=carmesí, TP=reactor | marcas verticales `strokeWidth 1.5` con `glow(color)`, base `borderPanel`; hover muestra tooltip vidrio |
| returns-calendar `[CORE]` | Calendario de rentabilidad mensual/anual (estilo heatmap de contribuciones) | por rendimiento: espectro de vitalidad | celdas `cardInner` 11px radio; fill del color de estado alpha 0.25→1 por magnitud; hover `glowStrong` |
| fitness-evolution `[CORE]` | Curva de fitness por generación (optimización genética AG) | convergiendo=transitionIndigo→optimaCyan, estancado=alertAmber, renovado=glow burst | línea `gradTransition` → `gradOptima` al converger; anotación de renovación como `sonarPulse` |
| rolling-metric `[CORE]` | Métricas rolling en el tiempo (Sharpe, vol, max-DD) — múltiples líneas | por umbral de alerta: espectro | líneas multicolor del espectro; banda de umbral crítico rellena `criticalCrimson` alpha 0.06 |
| underwater-plot `[CORE]` | Drawdown como área rellena bajo el eje cero ("under the water") | por profundidad: espectro según magnitud | área `gradCritical` alpha 0.05→0.35 proporcional a profundidad; eje zero `borderPanel`; línea relleno `criticalRed` |
| risk-return-scatter `[CORE]` | Frontera de eficiencia: dispersión riesgo (X) vs retorno (Y) por estrategia | por estado de estrategia | puntos neón del estado `glow(color)`; frontera de Pareto `gradOptima` strokeWidth 1.5 |
| trade-distribution `[CORE]` | Histograma de distribución de P&L por trade (ganadoras vs perdedoras) | ganadoras=optimaCyan, perdedoras=criticalCrimson | barras bicolor lado a lado; línea de media `textPrimary` con `glow(optimaTeal)`; línea zero `borderPanel` |
| parameter-sensitivity `[CORE]` | Barras de robustez por parámetro (degradación desde el óptimo) | robusto=cian, marginal=alertAmber, frágil=criticalCrimson | barras coloreadas por espectro; marcador de referencia óptima línea `optimaCyan`; `glow(color)` proporcional a robustez |
| regime-timeline `[CORE]` | Línea de tiempo de régimen de mercado: bandas horizontales coloreadas por estado | Tendencia/Rango/Volátil/Calmo | bandas alpha 0.12 del espectro; labels `dataMono` 11px; `regime-chip` encima para el período activo |
| optimization-contour `[CORE]` | Mapa de contorno 2D del espacio de parámetros (fitness landscape sobre 2 params) | por fitness: espectro de vitalidad | gradiente espectro via `CustomPainter`; isolíneas `strokeWidth 0.5` + `textMuted`; óptimo marcado con `glowStrong(optimaCyan)` |

---

### §11 Núcleo Drasus (piezas únicas) `[CORE]`

| id | Role | Variantes / Estados | Tokens |
|---|---|---|---|
| organism-cell-card | Tarjeta de una célula/estrategia | salud óptima/transición/alerta/muerta | cardInner, neón teñido por salud; `sonarPulse` al activar; `scanRing` en estado live |
| vitality-spectrum-legend | Leyenda del espectro de vitalidad | los 4 estados | espectro completo |
| crystal-orb | Orbe de cristal (gradiente radial + glowStrong) | teñido por estado | RadialGradient(cian→índigo→púrpura) + glowStrong; `scanRing(optimaCyan)` en monitoreo activo; `sonarPulse` al cambiar de estado |
| galaxy-background | Fondo de galaxia (star-field) | tenue | starField |
| autopsy-header | Portada de autopsia/reporte funerario | muerte (carmesí) | gradCosmic, criticalCrimson, dataMono |
| canvas-zoom-frame | Marco de transición de zoom canvas (relacional ↔ interior) | los 2 estados + breadcrumb | glassFill, galaxia |
| dashboard-panel | Panel de monitoreo del Dashboard (widgets + métricas) | tabla + chips + métricas; `scanRing(optimaCyan)` ambiental mientras el portafolio está activo | panelSolid, chips, dataMono |
| expectation-envelope-badge | Indicador "dentro/fuera del sobre" | dentro=óptimo, fuera=crítico | optimaCyan, criticalCrimson |

---

## Do's and Don'ts

### Do
- Pon cada zona de datos sobre la pila sólida (`#080A18` → `#0E1426` → `#11182E`) y reserva el vidrio para el chrome (nav, menús, botones, inputs).
- Ata SIEMPRE el color a un estado (óptimo/transición/alerta/crítico). El neón es semántico y escaso: pocos puntos encendidos sobre la oscuridad.
- Usa el grotesco display a peso 500 para titulares; sube el piso tipográfico (cuerpo 14px, datos 13px) — se acabaron los 9–10px.
- Renderiza todo (DAGs, conos, vidrio, galaxia) con `CustomPainter`/`Canvas`/`Shader` nativo sobre Impeller.
- Da profundidad con **glow de color**, rim-light interno y el telón cósmico — nunca con sombra gris tipo Material.
- Usa **glow y gradientes ampliamente**, a lo largo de casi todos los componentes (botones, nodos, líneas, chips, gauges, iconos, KPIs, focos), siempre en el color del estado.
- **Estandariza el vidrio:** todo componente que requiera superficie translúcida usa `frosted()` o `GlassSurface` — NUNCA `Gx.glassFill` suelto en `BoxDecoration` sin `BackdropFilter`. El `glassFill` es un color, no un componente. La receta completa (blur 36 + fill 0x40F0F2FF + rim 0x20A096FF @ 28%) vive en `frosted()` y `GlassSurface`, que deben producir resultados idénticos.
- Alinea todos los números a la derecha y ponlos en mono (`numStyle`).
- El cristal limpio es un **gradiente radial + `glowStrong`** (orbe), no filos RGB desfasados.
- Anima la **interacción**, no la decoración: clic (propagación de luz), hover (glow), foco (glow), abrir dropdown, tocar día, switch, slider.
- Estratifica la densidad: MACRO/MESO densos y operativos; MICRO y splash con aire y escala ceremonial.

### Don't
- No uses gráficos de velas japonesas ni libros de órdenes como elemento central — Drasus destrona al precio (prioridad mínima, salvo que se especifique).
- No uses el color como decoración: nada de neón "porque se ve bonito" sin un estado detrás.
- No uses negro plano `#000000` de lienzo ni gris neutro de oficina en bordes — todo lleva el tinte azul-violeta.
- No apliques sombra **gris/negra** ni elevación tipo Material a las tarjetas — la única sombra es el glow de color.
- No uses **aberración cromática RGB** (los tres filos rojo/verde/azul desfasados) en inputs ni superficies — quedó mal; el cristal limpio es gradiente radial + glow.
- No uses el gradiente cósmico como relleno de superficie o botón — solo texto ceremonial y números-héroe vía `ShaderMask`.
- No bajes el cuerpo de 12px ni uses pesos bold (700) en los titulares — el grotesco vive en el peso medio 500.
- No metas HTML, CSS, DOM, WebView ni SVG (ADR-0097): si un fragmento `css` aparece, es solo especificación de token.
- No satures: el poder del neón nace de la escasez. Si todo brilla, nada significa.

## Surfaces

| Level | Name | Value | Purpose |
|-------|------|-------|---------|
| 0 | Deep Space | `#080A18` | Lienzo base del body y del ZUI. La galaxia se dibuja encima. Casi-negro azul-violeta |
| 1 | Nav Rail | `#0B1022` | Rieles de navegación lateral y barras verticales |
| 2 | Panel Sólido | `#0E1426` | Paneles de Datos: tablas, rejillas, contenedores con números |
| 3 | Tarjeta Interna | `#11182E` | Tarjetas dentro de paneles, relleno de nodos del DAG |
| 4 | Superficie Elevada | `#161E38` | Hover de fila, celda activa, estado elevado |
| — | Vidrio (chrome) | `0x40F0F2FF` @ 25% + blur 36 + rim `0x20A096FF` @ 28% | Capa translúcida del chrome, fuera de la pila sólida — se separa por rim-light, no por nivel. Tinte claro sobre fondo oscuro: física inversa a lo intuitivo |

## Elevation

El sistema evita las sombras **grises** tipo Material. La profundidad y la energía nacen de cuatro fuentes: (1) la **pila de superficies** sólidas que se oscurecen hacia el fondo, (2) el **rim-light** interno del vidrio (glow `glassRim` + filo de luz `glassEdge`), (3) el **glow de color** del estado, que es el protagonista (`glow` / `glowStrong` en botones, nodos, líneas, chips, gauges, iconos, focos), y (4) el **telón cósmico** (supernova violeta + estrellas) que da la sala profunda. La única "sombra" permitida es un `BoxShadow` **de color** (glow); nunca una sombra negra/gris de elevación Material.

## Imagery

La imaginería es generativa y nativa, nunca fotográfica ni de stock: no hay personas, lugares ni productos, y **no hay velas japonesas**. El telón es un **cosmos estático** sutil — supernova violeta (núcleo radial), tenue disco de acreción y campo de estrellas de 1–2px a 2–5% — pero la fuerza visual vive **dentro de los componentes**: **glow de color** y **gradientes** por todas partes (botones, nodos, líneas, chips, gauges, KPIs, focos). El **cristal** es un **orbe limpio** (gradiente radial cian→índigo→púrpura + glow potente), NO aberración cromática RGB. Los datos se dibujan como **grafos matemáticos** (DAGs del conducto de 8 pasos), **conos de Monte Carlo** (el "Sobre de Expectativa") y **barras de signos vitales**. Las estrategias son **organismos** vivos — células con ADN (genoma de hiperparámetros) y signos vitales (drawdown, sharpe) — no scripts. El tratamiento es oscuro y de reactor: luz solo donde hay dato o estado.

## Layout

La interfaz tiene dos superficies (ADR-0136 — supersede ADR-0028):

- **Dashboard:** centro de monitoreo y navegación. Widgets de features arrastrables (bento grid, read-only). Clic en cualquier entidad → abre el canvas en ese contexto.
- **Canvas [Forge/Reactor — TBD]:** lienzo infinito único con card-nodes rectangulares y conexiones bezier tipadas. Dos estados de zoom:
  - **Vista relacional** (alejado): topología visible, nodos en miniatura. Aquí se conecta, reordena, crea pipelines.
  - **Vista interior** (doble clic en un nodo): el nodo se expande in-place; el contexto padre se desplaza al borde. Aquí se edita la lógica interna.
  - Zoom continuo (estilo Figma/Google Maps). Breadcrumb flotante de vidrio Apple: `Cluster A › Portfolio B › Strategy 3`.
- **Inspector panel lateral:** se abre al hacer clic en un nodo hoja (feature, logic block). La UI específica de la feature vive aquí — nunca en una pantalla full separada.
- **Ceremonial (Splash / Onboarding):** pantalla a pantalla completa con display 56–72px, cristal y galaxia como protagonistas. Sin cambios.

**Jerarquía de entidades en el canvas (via anidación):**
`Cluster → Portfolio → Strategy → Logic Blocks`
**Jerarquía de proceso en el canvas:**
`Pipeline → Módulo (compound node) → Feature (atomic node)`
Ambas viven en el mismo canvas con focus mode de visibilidad independiente.

## Agent Prompt Guide

**Quick Color Reference**
- text (principal): `#E6ECF8`
- text (secundario): `#AEBBD6`
- background: `#080A18`
- panel sólido: `#0E1426`
- borde: `#1B2440`
- vidrio (chrome): `0x40F0F2FF` @ 25% + blur 36 + rim `0x20A096FF @ 28%`
- óptimo: `#54E8D0` · transición: `#9A8CFF` · alerta: `#FFC94D` · crítico: `#F0413F`
- acción viva (botón primario): `#7CF06A` (relleno) con texto `#080A18`

**Example Component Prompts**

1. Crea un panel de monitoreo del Dashboard: fondo `#0E1426`, borde 1px `#1B2440`, radio 11px, padding 10px. Dentro, una tabla densa con cabecera 12px `#8492B0` y filas en mono 13px alineadas a la derecha, separador `#141C32`, hover de fila `#161E38`. A la derecha de cada fila, un chip de régimen (texto `#54E8D0`, fondo `#08251F`, borde `#1E5E4F`, radio 8px).

2. Crea el breadcrumb flotante del canvas: pill de cristal flotante, `BackdropFilter` blur 24 + relleno `Color(0x73141C36)`, radio 16px, rim-light interno (sin borde duro). Segmento activo con texto `#E6ECF8`.

3. Crea un nodo de DAG para el Canvas (Vista Relacional — estilo N8N/React Flow): el nodo es un `Container` Flutter (NO `canvas.drawCircle`). Cuerpo `cardInner #11182E`, radio 10px, borde hairline `#1B2440`. Header: franja izq 3px en el color de estado + nombre `displayGrotesque 13px`. Body: key-values mono 12px. Puertos laterales: círculos 10px con anillo 1.5px del color del tipo. Las conexiones bezier se dibujan con `CustomPainter`: S-curve (`strokeWidth 2` + halo `blur 4`) + punta de flecha en el target 8px. Lienzo: dot-grid `#1B2440` @ 1.5px / 20px sobre `#080A18`. Al hover, el nodo escala 1.02 + `glowStrong(estadoColor)`; al recibir datos, `sonarPulse(estadoColor)` en el puerto de entrada; mientras procesa, `scanRing(estadoColor)` en el puerto de salida.

4. Crea un input de filtro: superficie de vidrio Apple, radio 10px, texto 14px en `#E6ECF8`, placeholder `#5C6B8C`. En foco (real, `FocusNode`), el borde sube a 1.5px en `#9A8CFF`, el relleno se vuelve más opaco y aparece un glow limpio alrededor (`glow(#9A8CFF, blur 18)`). Sin aberración cromática.

5. Crea la portada de una autopsia (Inspector Panel): fondo `#080A18` con telón cósmico tenue, un número-héroe en mono 28px, un título ceremonial en display 500 a 56px con el gradiente cósmico (`#E59CFF → #B79CFF → #56A8FF`) vía `ShaderMask`, borde y glow en carmesí `#F0413F` indicando muerte de la estrategia.

## Type Pairing Logic

El sistema tiene tres voces sin solapamiento de rol: **display grotesco** (titulares, peso 500 que susurra), **sans** (la UI que desaparece) y **mono** (los datos, que gritan "soy un número real"). La jerarquía no se construye con bold, sino con **saltos de escala** y con el contraste de familia: un dato en mono 13px convive en el mismo panel que un título display de 44px, y la cifra mono siempre se distingue del texto sans que la rodea. El grotesco nunca baja a etiquetas de UI; el mono nunca sube a prosa; la sans nunca toca un número.

## Motion Philosophy

La animación es funcional, nunca decorativa, pero **rica**. El **zoom del canvas** entre Vista Relacional y Vista Interior (ADR-0136) es la transición rectora — la cámara se acerca al nodo in-place, no cambia de pantalla. Cada interacción tiene su micro-animación, todas sobre Impeller/GPU:

- **Clic de botón:** se hunde (escala 0.96) y dispara una **propagación de luz** — un pulso de glow que estalla del centro hacia afuera (inspiración Reflect, ~460ms).
- **Hover** en nodos, líneas, tarjetas, iconos: el glow se intensifica y el elemento escala levemente (~160–220ms).
- **Foco** de input: borde + glow limpios entran animados (~200ms).
- **Dropdown:** abre/cierra con `AnimatedSize` y el chevron rota 180°.
- **Calendario:** al tocar un día aparece su anillo de glow (~180ms).
- **Switch:** el knob se desliza con `AnimatedAlign` y enciende su glow.
- **Slider:** se arrastra; la manija lleva `glowStrong` y el relleno es gradiente.
- **Texto con propagación de luz:** la iluminación se expande del centro hacia afuera (como el hero de Reflect).
- **Glow pulsante:** señala trabajo vivo (incubación, escaneo) latiendo en el color de estado.
- **Sonar Pulse `sonarPulse(c)`:** un único anillo circular que se expande desde el centro del componente y se desvanece (eco de sonar, ~600–900ms). Indica un evento discreto: "acabo de activar", "acabo de recibir señal", "acabo de detectar". Un solo anillo por evento — no anillos múltiples simultáneos. Aplicable a: `organism-cell-card` al activar una estrategia, nodo DAG al recibir datos, `crystal-orb` al cambiar de estado.
- **Ring-Breath `scanRing(c)`:** dos o tres anillos concéntricos que pulsan en secuencia con retardo de fase (staggered: +0ms / +300ms / +600ms), repitiendo indefinidamente mientras dure el estado. Indica monitoreo sostenido — "estoy vivo y vigilando". La velocidad del ciclo codifica urgencia: lento 4–6s + `transitionIndigo` = incubando · medio 2–3s + `optimaCyan` = activo en calma · rápido 1–1.5s + `alertAmber` = alerta activa. Para `criticalCrimson` usar parpadeo de glow (ya existente), no anillos. Aplicable a: `crystal-orb` en monitoreo, `organism-cell-card` en live, `dashboard-panel` mientras el portafolio está activo.

- **Hover en data-viz:** al entrar el cursor en cualquier gráfico, la línea se engrosa (1.5 → 2.5px) y el glow sube; aparece un cursor vertical tenue + círculo en el punto de datos más cercano. Para barras: la barra hovereada crece 2px y su glow se intensifica; las demás se atenúan levemente. Para celdas (heatmap, returns-calendar): la celda resalta con borde semántico de 1.5px. Para scatter: el punto más cercano al cursor se agranda con `glowStrong`. Implementado via `HoverableChart` — `MouseRegion` que pasa `Offset?` al `CustomPainter`.
- **Área de relleno en líneas:** toda línea de gráfico de renta, equity, fitness o métrica rolling lleva un gradiente de relleno bajo la curva del color de su familia semántica (alpha ≈ 12–30 en la parte alta, transparente en la base). En el hover el alpha del área sube ~50%. El relleno es siempre del color que corresponde al estado de la línea — nunca decorativo.

Ninguna animación existe "porque se ve bien" — cada una comunica un cambio de dato, foco o estado.

## Performance Guidelines

Reglas duras de rendering para mantener 60fps en CustomPainter sobre Impeller:

### GPU (Raster)

| Operación | Costo típico | Regla |
|---|---|---|
| `saveLayer` + `ImageFilter.blur(sigma > 4)` sobre lienzo >200px² | ~90ms | **Prohibido en animación.** Sustituir con capas de círculos/líneas a distintas opacidades (halo sin blur) |
| `MaskFilter.blur` en loop de >100 iteraciones | 15–90ms | **Prohibido.** Reemplazar con `strokeWidth` más grueso a baja opacidad |
| `drawPicture()` con Picture pre-grabado | <1ms | **Forma canónica para fondos estáticos.** Grabar una vez, dibujar siempre |
| `drawCircle()` × 5000 | ~5ms | Aceptable si se evita blur. Reducir a 1 sola pasada (no nebula+sharp) durante rotación |

### CPU (UI Thread)

| Operación | Costo típico | Regla |
|---|---|---|
| Proyección 3D + sort en `paint()` | 10–15ms | Mover al State con caché por ángulo. Recalcular solo cuando el ángulo cambia |
| `exp()` en loop de >1000 iteraciones | 5–10ms | Pre-calcular array de intensidades una vez por frame, no por segmento |
| Sort O(n log n) de >1000 elementos por frame | 5ms | Mover al State. Cachear resultado ordenado; invalidar solo con cambio de datos |
| `compute()` (Isolate) para generación de datos | asíncrono | Usar para generar datos sintéticos. No bloquea el UI thread |

### Medicion

```bash
flutter run --profile -t lib/gallery/gallery_preview_main.dart
# Abrir URL de DevTools en Chrome → Performance → Record
# Barra superior = GPU (raster thread), inferior = UI (Dart thread)
# Target: ambas <16ms para 60fps
```

## Similar Aesthetics

- **Reflect** — Lienzo violeta-negro con galaxia de estrellas, rim-light en vez de sombras y un acento que es puntuación, no pintura. La esencia "mapa de constelaciones".
- **Modal** — Terminal de fósforo: negro absoluto, verde reactor como acción viva, neutros tintados (nunca gris clínico) y un objeto de cristal volumétrico como única pieza de peso.
- **Resend** — Obsidiana premium: negro mate, elevación por hairline tintado, color como dato escaso y disciplinado, voz mono para el código.
- **Vivid+Co** — Prismas de cristal 3D con aberración cromática roja/azul/verde como firma visual; restraint editorial.
- **Sala de control de reactor / monitoreo de biolaboratorio** — Oscuridad absoluta, luz solo para transmitir estado, densidad de instrumentos.

## Quick Start

### Tokens Flutter (Dart)

> Drasus es Flutter/Impeller: estos tokens se materializan como constantes `Color`/`TextStyle` reutilizables en un `ThemeData`/design-system, no como CSS (ADR-0097 prohíbe DOM/CSS).

```dart
// Superficies (pila sólida + vidrio)
const deepSpace      = Color(0xFF080A18); // lienzo base / ZUI
const navRail        = Color(0xFF0B1022); // riel de navegación
const panelSolid     = Color(0xFF0E1426); // panel de datos
const cardInner      = Color(0xFF11182E); // tarjeta interna
const surfaceRaised  = Color(0xFF161E38); // hover de fila
const glassFill      = Color(0x73141C36); // vidrio del chrome (≈45%) + BackdropFilter blur 24

// Estructura
const borderPanel    = Color(0xFF1B2440); // hairline tintado del panel
const divider        = Color(0xFF141C32); // separador interno

// Texto
const textPrimary    = Color(0xFFE6ECF8);
const textSecondary  = Color(0xFFAEBBD6);
const textLabel      = Color(0xFF8492B0);
const textMuted      = Color(0xFF5C6B8C);

// Espectro de Vitalidad (semántico — neón vivo, escaso)
const optimaCyan       = Color(0xFF54E8D0); // 🟢 óptimo
const optimaTeal       = Color(0xFF2DD4BF);
const reactorGreen     = Color(0xFF7CF06A); // 🟢 acción viva
const transitionIndigo = Color(0xFF9A8CFF); // 🔵 incubación / calma
const transitionBlue   = Color(0xFF56A8FF);
const transitionPurple = Color(0xFF8B83E8);
const alertAmber       = Color(0xFFFFC94D); // 🟠 alerta / volátil
const alertOrange      = Color(0xFFF59423);
const criticalRed      = Color(0xFFFF8A8A); // 🔴 crítico
const criticalCrimson  = Color(0xFFF0413F); // 🔴 fallo / muerte

// Gradientes (compatibles dentro de cada familia semántica)
const gradOptima     = [optimaCyan, optimaTeal];
const gradReactor    = [reactorGreen, optimaCyan];
const gradTransition = [transitionIndigo, transitionBlue];
const gradAurora     = [transitionPurple, transitionIndigo, transitionBlue];
const gradAlert      = [alertAmber, alertOrange];
const gradCritical   = [criticalRed, criticalCrimson];
const gradCosmic     = [Color(0xFFE59CFF), Color(0xFFB79CFF), Color(0xFF56A8FF)]; // texto ceremonial / KPI

// Glow (el "poder" Reflect — protagonista, a lo largo de casi todo componente)
List<BoxShadow> glow(Color c, {double blur = 16, double opacity = 0.45}) =>
    [BoxShadow(color: c.withOpacity(opacity), blurRadius: blur)];
List<BoxShadow> glowStrong(Color c) => [
  BoxShadow(color: c.withOpacity(0.55), blurRadius: 10),
  BoxShadow(color: c.withOpacity(0.28), blurRadius: 30, spreadRadius: 2),
];
List<Shadow> textGlow(Color c) => [Shadow(color: c.withOpacity(0.7), blurRadius: 8)];

// Orbe de cristal (SUSTITUYE la aberración RGB, que quedaba mal):
// RadialGradient(colors: [optimaCyan, transitionIndigo, transitionPurple]) + glowStrong
// Telón cósmico estático: supernova violeta + estrellas. starField: textPrimary @ 2–5%

// Profundidad: glow de color + rim-light, NUNCA sombra gris tipo Material
const glassRim = BoxShadow(color: Color(0x0DA096FF), blurRadius: 24); // interno
// glassEdge: filo superior de luz inset 0 1px 0 rgba(180,170,255,0.12)
```

### Familias y escala

```dart
// Tres voces, empaquetadas en assets/fonts (familia exacta vía Naming/Flutter-Engineer)
// displayGrotesque — títulos, peso 500, tracking -0.02em en ≥24px
// uiSans          — cuerpo y UI, pesos 400/500
// dataMono        — todo número / ID / símbolo (estilo reutilizable numStyle)

// Escala: 11 / 12 / 13 / 14 / 16 / 20–24 / 28 / 32–44 / 56–72
// Radios: panel 11 · chrome 14–16 · botón/input 10 · chip 8 (o 999) · tooltip 12
// Spacing: 4 · 8 · 9 · 12 · 16 · 24 · 32 · 48 · 64  (gaps densos 8–9, ceremonial 24–32)
```

### Galería de Componentes — Implementación

**Estado:** implementado (2026-06-22). `flutter build linux --debug` verde + `flutter test test/gallery_smoke_test.dart` verde (Flutter 3.44.2).

**Punto de entrada aislado:** `flutter run -d linux -t lib/gallery/gallery_preview_main.dart` — sin Rust/Bridge.

**Estructura `ui/lib/gallery/`:**
```
ui/lib/gallery/
  gallery_tokens.dart   # ÚNICO sitio con hex → Color/TextStyle + gradientes + glow. Clase Gx.
  gallery_painters.dart # CustomPainter: telón cósmico, líneas de DAG, cono MC, heatmap, scatter
  gallery_fx.dart       # interacción: vidrio Apple, GlowButton, GlowSwitch, GlowSlider,
                        #   GlowInput, GlowDropdown, GlowCalendar, LightBurstText, InteractiveDag
  gallery_tab.dart      # GalleryTab: arma la vitrina por secciones
  sections/             # archivos por sección del catálogo
```

**Pestaña Components** en `ui/lib/panel_operativo.dart` al nivel de Reloj/Trabajos/Auditoría. `DefaultTabController(length: 4)`.

**Iconos:** `iconsax_plus ^1.0.0` — 896 iconos `const IconData`, estilo Linear. Capa semántica `Gx.icon*` en `gallery_tokens.dart`. Descartado `phosphor_flutter 2.1.0` (incompatible Flutter 3.44).

**Fuentes embebidas (offline):** Space Grotesk w500 · Inter w400/w500 · JetBrains Mono w400/w500 en `ui/assets/fonts/`. Helpers `Gx.displayGrotesque/uiSans/dataMono` via `TextStyle(fontFamily: ...)`. Zero dependencia de `google_fonts` en runtime.
> `JetBrainsMono-Medium.ttf` es versión NerdFont (2.4MB) — reemplazar con el .ttf limpio de fonts.google.com (~110KB).

**Tests:** smoke `flutter test test/gallery_smoke_test.dart` · goldens `flutter test --update-goldens test/gallery_golden_test.dart` (en `ui/test/goldens/`: `gallery_full_scroll.png` 1440×5000, `gallery_fundamentos.png` 1200×900).

**Cobertura:** §4–§11 [CORE] + [STD] completos. Pendiente mínimo: `anchor/scrollspy` (decorativa, baja prioridad).
