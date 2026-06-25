# Plan: Feedback Visual Sesión 2026-06-24 — 7 Puntos

## Contexto

El usuario analizó el competidor Algory OS y proveyó feedback visual detallado sobre la galería actual. El objetivo es llevar Drasus Engine a un nivel de impacto visual cinematográfico comparable, con identidad propia. Los 7 puntos abarcan: sistema de acento dinámico, shells de Dashboard y DAG, efecto eléctrico en Monte Carlo, tipografía más agresiva, componente de cinta de órdenes, y mejora radical de la galaxia 3D.

Estado actual confirmado por exploración:
- `gallery_tokens.dart`: pila bunker (#04050E) y accentPrimaryA/B ya aplicados
- `section_dataviz_new.dart`: Monte Carlo (200px alto, sin efecto eléctrico) + Cluster 3D (300 pts, 640×480)
- `panel_operativo.dart`: 4 pestañas (Reloj, Trabajos, Auditoría, Components) — sin Dashboard ni DAG
- ROADMAP: Dashboard + DAG están en EPIC-8 (diferidos); usuario los quiere como shells YA

---

## 1. Sistema de Acento Dinámico (Punto 1)

### Decisión de diseño: qué elementos usa el acento

El acento NO toca elementos semánticos (los 4 estados de vitalidad son intocables). Aplica a **chrome interactivo neutro**:

| Elemento | Dónde |
|---|---|
| Input/field — borde de foco | Todos los `GlowInput`, `GlowDropdown`, `GlowCombobox` |
| Tab activo — underline/indicador | `GlowTabBar`, segmented controls |
| Navegación activa — punto/línea | Nav rail, `ZuiNavPill` |
| Scrollbar — thumb | Todo área scrollable |
| Slider neutro — handle y relleno | Sliders sin estado semántico |
| Panel header — borde izquierdo de 3px | Los paneles que tienen acento decorativo |
| Checkbox/Radio — anillo "on" neutro | Cuando el contexto no tiene estado semántico |
| Botón secundario — borde y glow | `button-glass` en contextos neutros |
| Engranaje de settings — glow en hover | El ícono de settings |
| Sort indicator activo en tablas | Columna ordenada activa |
| Breadcrumb — segmento activo | `canvas-breadcrumb` |

### Settings Panel (engranaje en esquina)

Vive en la esquina superior derecha del `AppBar` de `panel_operativo.dart` como `IconButton` con `Iconsax.setting_2` + glow en hover. Al clickear: panel lateral deslizante (drawer de vidrio Apple, 320px ancho, desde la derecha).

**Contenido del settings drawer:**

```
┌── CUENTA ──────────────────────────────────┐
│  [Avatar initials]  Felipe Bravo            │
│  fbravo.hz@gmail.com                        │
│  Drasus Engine v0.1.0-alpha                 │
└─────────────────────────────────────────────┘

┌── APARIENCIA ───────────────────────────────┐
│  Color de Énfasis                           │
│  [● ● ● ● ● ● ● ● ● ● ● ●]  12 swatches   │
│  [Hex #___________] [Live preview strip]    │
│                                             │
│  Swatches preset (compatibles con #04050E): │
│  Cian #54E8D0 · Verde #7CF06A · Índigo #9A8CFF │
│  Azul #56A8FF · Ámbar #FFC94D · Rojo #CC2B2B │
│  Neutro #B4BFCE · Coral #FF6B6B · Lima #AAFF00 │
│  Magenta #FF4FD8 · Dorado #FFD700 · Cian puro #00FFFF │
└─────────────────────────────────────────────┘

┌── SISTEMA ──────────────────────────────────┐
│  Build: 2026-06-24                          │
│  Rust core: conectado / desconectado        │
└─────────────────────────────────────────────┘
```

### Implementación técnica (ThemeState)

- `DrasusTheme` (`InheritedWidget` o `ValueNotifier<Color>`) envuelve la app desde `main.dart`
- `accentColor` es la única variable dinámica
- Los widgets consumen `DrasusTheme.of(context).accentColor`
- Persiste en `SharedPreferences` (key: `accent_color_hex`)
- Los `Gx.*` helpers que afectan el acento reciben el color como parámetro en lugar de constante

### Impacto en documentos

- `DESIGN.md §Tokens—Colors`: reemplazar `accentPrimary A/B` por `accentPrimary` como token dinámico; documentar los 12 swatches preset; documentar qué elementos lo consumen
- `DESIGN.md §Motion Philosophy`: nota de que el settings drawer entra con `AnimatedSlide` desde la derecha
- `DESIGN.md §Components`: añadir spec de `settings-drawer` y `settings-gear-button`
- `docs/ROADMAP.md`: añadir STORY-016 "DrasusTheme — Sistema de Acento Dinámico + Panel de Settings"

---

## 2. Dashboard Shell (Punto 2)

### Alcance del shell

El Dashboard existe HOY como shell vacío — sin funcionalidad real, pero estructuralmente completo para ir poblándose. Es **modo lectura** hasta que las features de Rust existan.

### Estructura Flutter

- Nueva pestaña en `panel_operativo.dart`: "Dashboard" (5ta pestaña, ícono `Iconsax.home_1`)
- `ui/lib/tabs/dashboard_tab.dart`: nuevo archivo
- **Bento grid vacío**: `Wrap` o `GridView` de celdas de 240×160px con estado empty (línea punteada + ícono + "Sin widgets")
- **Botón "+" flotante**: abre un `BottomSheet` o `SidePanel` con catálogo de widgets disponibles
- **Catálogo de widgets**: lista de tarjetas con nombre del widget, descripción, ícono, y estado (disponible/no disponible según si el feature Rust existe). Drag to grid.
- **Persistencia de layout**: `SharedPreferences` con JSON del grid (lista de widget IDs + posiciones)

### Widget registry

Un `Map<String, DashboardWidgetMeta>` en un archivo `dashboard_registry.dart` define todos los widgets posibles. Por ahora todos están en estado "próximamente" salvo los que tienen implementación real (ninguno inicialmente).

### Impacto en documentos

- `DESIGN.md §Layout`: actualizar descripción del Dashboard para reflejar bento grid real con drag-and-drop
- `docs/ROADMAP.md`: añadir STORY-017 "Dashboard Shell — Bento Grid con Widget Registry" (EPIC-0)
- `docs/sad/SAD-06.md`: actualizar §UI/UX flows para reflejar que el Dashboard existe como shell real desde EPIC-0

---

## 3. DAG Canvas Shell (Punto 3)

### Alcance del shell

El Canvas DAG existe HOY como shell navegable con la pantalla del `InteractiveDag` (del gallery_fx.dart que ya existe) escalado a pantalla completa, más un panel lateral con features disponibles para agregar.

### Estructura Flutter

- Nueva pestaña en `panel_operativo.dart`: "Canvas" (6ta pestaña, ícono `Iconsax.bezier`)
- `ui/lib/tabs/canvas_tab.dart`: nuevo archivo
- **Canvas principal**: `InteractiveViewer` con `InteractiveDagPainter` (ya existe en `gallery_fx.dart`) — reutilizar directamente
- **Panel lateral izquierdo** (200px): lista de features disponibles para arrastrar al canvas
  - Por ahora: features de EPIC-0 completadas (Clock, Jobs, Audit-log, etc.)
  - Cada feature es una tarjeta draggable que al soltarla en el canvas crea un nodo vacío
- **Toolbar superior**: zoom in/out, fit-to-screen, clear canvas
- **Breadcrumb flotante**: `canvas-breadcrumb` (ya especificado en DESIGN.md)

### Feature list inicial (side panel)

```
[Reloj Determinista]    — EPIC-0 ✓
[Async Job Executor]    — EPIC-0 ✓
[Audit Log]             — EPIC-0 ✓
[Telemetría]            — EPIC-0 parcial
[MCP Gateway]           — EPIC-0 parcial
[... próximas features] — locked
```

### Impacto en documentos

- `docs/ROADMAP.md`: añadir STORY-018 "Canvas Shell — DAG interactivo con Feature List" (EPIC-0)
- `docs/sad/SAD-06.md`: actualizar con Canvas como artefacto existente desde EPIC-0

---

## A. Efecto Eléctrico Universal — Todas las Gráficas de Líneas (Adición A)

### Alcance

El efecto de scan + ignición eléctrica NO es solo de Monte Carlo — aplica a **toda gráfica de líneas** compatible:

| Componente | Compatible |
|---|---|
| `equity-curve` | ✅ |
| `multi-equity-overlay` | ✅ |
| `rolling-metric` | ✅ |
| `drawdown-curve` / `underwater-plot` | ✅ |
| `fitness-evolution` | ✅ |
| `wfa-chart` | ✅ |
| `monte-carlo-lines` | ✅ |
| `sparkline` | ✅ (simplificado, sin comet tail) |
| Barras / gauges / scatter | ❌ no aplica |

### Implementación como primitivo reutilizable

Se extrae un mixin/helper `ElectricScanMixin` que cualquier `CustomPainter` de líneas puede incorporar:
- Recibe `scanProgress` (0.0–1.0) y calcula la intensidad de glow por segmento
- Fórmula de ignición: `intensity = max(0, exp(-(timeSinceScan * 8)))` → peak al paso del scan, decay suave
- Comet tail: `LinearGradient` de 120px a la izquierda del scan, `Colors.transparent → accentColor.withOpacity(0.25)`
- El scan completo toma 600–900ms (ajustado por longitud del chart)

### Impacto en documentos

- `DESIGN.md §Motion Philosophy`: actualizar descripción de `scanInitLine` para ser universal en toda gráfica de líneas, no solo Monte Carlo

---

## B. Efecto Vidrio Mate — Mucho Más Pronunciado (Adición B)

### Problema actual

`glassFill = Color(0x730E1530)` con blur 24 es casi indistinguible del fondo `#04050E`. El vidrio Apple requiere contraste real entre el contenido del fondo y la superficie translúcida.

### Corrección

| Parámetro | Actual | Target |
|---|---|---|
| Alpha del fill | 0x73 (45%) | 0x9A (60%) |
| BackdropFilter blur | sigma 24 | sigma 36 |
| Tint interior | sin tinte | añadir capa blanca/azul 8% |
| glassRim blur | 24 | 30 |
| glassEdge opacity | rgba(180,170,255,0.12) | rgba(180,170,255,0.22) |

El fill queda: `Color(0x9A0E1530)` + `BackdropFilter.blur(36)` + `Color(0x14AAAAFF)` tinte interior (capa blanca-azulada 8%) + rim-light más pronunciado.

### Impacto en documentos

- `DESIGN.md §Tokens—Colors`: actualizar `glassFill` a `0x9A0E1530`, blur 36, añadir nota del tinte interior
- `DESIGN.md §Elevation`: actualizar `glass-rim` a blur 30, `glassEdge` a 0.22 opacity
- `gallery_tokens.dart`: actualizar constantes y helpers de vidrio

---

## C. Sistema de Paleta de Fondo (Adición C)

### Concepto

El `DrasusTheme` controla TANTO el `accentColor` como el `backgroundPalette`. Existen 8 paletas preset que el usuario elige en el settings drawer. Cada paleta define los colores de la pila sólida (Deep Space, Nav Rail, Panel Sólido, etc.).

### Paletas propuestas

| ID | Nombre | Deep Space | Carácter |
|---|---|---|---|
| `bunker` | Bunker Puro (default) | `#04050E` | Neutral oscuro |
| `ash` | Ceniza | `#060606` | Gris neutro extremo |
| `crimson` | Bunker Carmesí | `#0E0406` | Tinte rojo/peligro |
| `forest` | Bunker Forestal | `#040E06` | Tinte verde/night-vision |
| `navy` | Bunker Naval | `#04090E` | Tinte azul/inteligencia |
| `void` | Vacío Púrpura | `#07040E` | Tinte púrpura |
| `slate` | Pizarra Clara | `#D8DCE8` | Gris claro, modo diurno |
| `paper` | Papel Blanco | `#F0F2F8` | Blanco frío, modo diurno |

Para las paletas oscuras: la pila sólida se escala proporcionalmente desde el Deep Space (los 5 niveles mantienen el mismo delta relativo de luminosidad).

Para las paletas claras (slate/paper): TODOS los tokens de texto se invierten, las superficies se vuelven claras, el glow se ajusta a versiones más contenidas. Es un cambio de diseño completo — se implementa como variante. Se incluye como "beta" en el settings.

### Impacto en documentos

- `DESIGN.md §Surfaces`: añadir tabla de paletas con los 8 Deep Space base y nota de escala proporcional
- `DESIGN.md §Tokens—Colors`: nota de que `deepSpace` es el punto de ancla de la paleta, no un valor fijo
- `docs/ROADMAP.md`: STORY-016 se amplía para incluir paleta de fondo además de acento

---

## D. Nodos y Conexiones DAG en la Galería (Adición D)

### Brechas actuales

El `InteractiveDag` existe en `gallery_fx.dart` pero NO hay una sección dedicada en la galería que muestre:
- Nodo individual con todos sus estados (reposo, hover, seleccionado, procesando, inválido)
- Conexiones bezier con sus variantes (normal, hover, seleccionada, inválida)
- Los diferentes tipos de puerto (entrada, salida, múltiples)
- El dot-grid del lienzo

### Nueva sección en galería: `section_dag_nodes.dart`

Contenido de la sección:

1. **Anatomía del Nodo** — Un nodo estático (sin movimiento) mostrando todas sus partes: header con borde de color, body con key-values, puertos circulares, badge de tipo.

2. **Matriz de estados del nodo** — 6 nodos en grid, uno por estado:
   - Reposo: borde 1px borderPanel, glow tenue
   - Hover: `glowStrong` + scale 1.02
   - Seleccionado: borde 2px neón + glow del estado
   - Procesando: `scanRing` en puerto de salida
   - Recibe datos: `sonarPulse` en puerto de entrada
   - Inválido/error: borde criticalCrimson parpadeante + `glowStrong`

3. **Tipos de conexión** — Pares de nodos con bezier S-curve mostrando:
   - Conexión normal (índigo)
   - Conexión hover (engrosada + tooltip)
   - Conexión inválida (crimson parpadeante)
   - Múltiples conexiones desde un nodo

4. **Canvas completo** — El `InteractiveDag` existente de `gallery_fx.dart` mostrado en un `SizedBox` de 700×400px con los controles de interacción.

### Impacto en documentos

- `DESIGN.md §10 Data-viz` / `§Components`: la spec de `dag-node-graph` ya existe — no hay cambios de spec, solo se añade la nota de que la sección de galería existe

---

## 4. Monte Carlo — Efecto Eléctrico (Punto 4)

### Brechas identificadas vs Algory

| Aspecto | Algory | Nuestro actual | Target |
|---|---|---|---|
| Altura lienzo | ~400-500px | 200px | 420px |
| Scan line | Neon rojo intenso | Cian simple | Cian brillante |
| Comet tail | Gradiente izquierdo | Sin tail | Gradiente 120px left |
| Electric reveal | Líneas se iluminan al paso | Sin efecto | Ignition + decay |
| Colores | Pinks/magentas/rojos | Verde/rojo | Espectro completo saturado |

### Implementación del efecto eléctrico

El scan line ya no es un painter separado — se integra en el painter principal para que pueda afectar el brillo de cada segmento:

1. **Comet tail**: Desde `scanX - 120` hasta `scanX`, un `LinearGradient` de `Colors.transparent` a `optimaCyan.withOpacity(0.25)`, dibujado como rect antes de la línea nítida.

2. **Electric ignition por segmento**: Cada trayectoria pasa por un "tiempo desde que fue escaneada". La intensidad de glow en cada segmento es:
   ```
   timeSinceScan = (scanX - segmentX) / width  
   intensity = max(0, exp(-timeSinceScan * 8))  // decay exponencial
   lineOpacity = baseOpacity + intensity * (1 - baseOpacity)  
   glowRadius = baseGlow + intensity * 16px
   ```
   Esto hace que cada segmento "encienda" al paso del scan y luego decaiga a su color base.

3. **Colores más saturados**: Las trayectorias ganadoras usan todo el rango `gradOptima` (no solo alpha 0.30 plano) — el color varía por posición en la curva. Las trayectorias en el rango medio también usan `transitionBlue`, `transitionPurple`, `alertAmber` para variedad.

4. **Altura**: `SizedBox` de 420px mínimo, expandible.

### Archivos a modificar

- `ui/lib/gallery/sections/section_dataviz_new.dart`: refactor de `_MonteCarloBackgroundPainter` + `ScanInitLinePainter` para integrar el efecto eléctrico. La animación de scan y la de líneas se conectan via `Animation<double>` compartido.

### Impacto en documentos

- `DESIGN.md §10 Data-viz`: actualizar spec de `monte-carlo-lines` con descripción del efecto eléctrico y los parámetros del comet tail.

---

## 5. Tipografía (Punto 5)

### Problema

Space Grotesk es demasiado redondeada y amigable. Las secciones en galería usan 14px cuando deberían usar 20-24px.

### Recomendación de fuente: **Rajdhani** w600 para display

| Criterio | Space Grotesk | Rajdhani w600 |
|---|---|---|
| Forma de letras | Redondeada, amigable | Angular, geométrica |
| Carácter | Startup tech | Militar/aeroespacial |
| Legibilidad | Excelente | Excelente |
| Pesos disponibles | 300-700 | 300-700 |
| Google Fonts | Sí | Sí (embeddable) |
| Uso real | Productos de diseño | Interfaces de defensa/aerospace |

Rajdhani w600 proyecta exactamente "tecnología nuclear sin perder elegancia". Sus letras tienen cortes angulares, tracking natural más estrecho, y no pierde legibilidad en tamaños pequeños.

**Inter y JetBrains Mono se mantienen** — Inter para sans (perfecto neutral) y JetBrains Mono para datos (estándar de la industria).

### Corrección de tamaños

Los títulos de sección en galería usan `displayGrotesque()` de 14px cuando deben usar `textSection` (20-24px, w600). Cada `_sectionTitle()` helper debe llamar al escala correcta.

### Archivos a modificar

- `ui/assets/fonts/`: añadir `Rajdhani-SemiBold.ttf` (w600)
- `ui/pubspec.yaml`: declarar la fuente
- `gallery_tokens.dart`: cambiar `fontDisplay = 'Rajdhani'`, actualizar `displayGrotesque()` para que la escala base sea 20px (no 14px). Crear helper `sectionTitle()` en 22px w600.
- `gallery_tab.dart` + secciones: donde se usen `_sectionTitle()` hardcodeados, usar el nuevo helper.

### Impacto en documentos

- `DESIGN.md §Tokens—Typography`: cambiar `displayGrotesque` a Rajdhani, documentar el cambio y la razón (militech > amigable). Actualizar la entrada de instalación.

---

## 6. Cinta de Órdenes — `trade-tape` (Punto 6)

### Descripción del componente

Un panel de scroll vertical continuo con entradas de trades que aparecen desde abajo y suben. Similar a un ticker tape de bolsa, pero vertical y con datos cuantitativos de las órdenes del algoritmo.

### Spec visual

```
┌── LIVE TRADE TAPE ─── ADM 13 · 5.737 trades ──┐
│                                                  │
│  [fade gradient out at top]                      │
│                                                  │
│  EURUSD   BUY   1.08432   0.5L   ▲ +$124.50     │  ← optimaCyan
│  GBPUSD   SELL  1.26891   1.0L   ▼ −$78.20      │  ← criticalRed
│  XAUUSD   BUY   2341.20   0.2L   ▲ +$441.00     │  ← optimaCyan
│  USDJPY   SELL  149.321   0.5L   ▼ −$22.10      │  ← criticalRed
│  EURUSD   BUY   1.08445   0.3L   ▲ +$87.30      │  ← optimaCyan
│  ...                                             │
│  [fade gradient in at bottom — nuevas entradas]  │
│                                                  │
└──────────────────────────────────────────────────┘
```

Header: `dataMono 11px` en `textSecondary` + chip "LIVE" en `reactorGreen` parpadeante.
Cada entrada: mono 12px, dirección comprada=`optimaCyan`, vendida=`criticalRed`.
Fade: `ShaderMask` con `LinearGradient` transparent→opaque en top 32px y bottom 32px.
Scroll: `AnimatedList` con inserción desde abajo + auto-scroll. En galería: datos sintéticos con `Timer.periodic`.

También: **`trade-ticker-bar`** — variante horizontal (una sola línea de texto que scrollea de derecha a izquierda), para usar como barra de estado inferior o footer.

### En la galería

Nueva sección en `section_dataviz_new.dart` o archivo propio `section_trade_tape.dart`, integrado en `gallery_tab.dart` después de la sección de Monte Carlo.

### Impacto en documentos

- `DESIGN.md §10 Data-viz`: añadir `trade-tape [CORE]` y `trade-ticker-bar [CORE]`

---

## 7. Galaxia 3D — Upgrade (Punto 7)

### Brechas identificadas vs Algory

| Aspecto | Algory | Nuestro actual | Target |
|---|---|---|---|
| Puntos | ~5.000-10.000 | 300 | 5.000 |
| Tamaño de punto | 1-1.5px (micro-dots) | 3px | 1.5px |
| Efecto nebulosa | Bloom layer sobre los dots | Sin bloom | `saveLayer` con blur |
| Densidad visual | Clusters parecen nubes | Scatter plot claro | Nubes de polvo estelar |
| Color variación | Multi-hue dentro del cluster | Color sólido por cluster | Gradiente radial por cluster |
| Tamaño en galería | Pantalla casi completa | 640×480 | 840×600 |
| Lista de clusters | Panel izquierdo con conteos | Solo etiquetas en canvas | Panel lateral con leyenda |

### Implementación del efecto nebulosa

En `_Cluster3dPainter.paint()`:
1. `canvas.saveLayer(bounds, Paint()..imageFilter = ImageFilter.blur(sigmaX: 8, sigmaY: 8))` → dibuja todos los puntos de cada cluster en este layer → `canvas.restore()` → resultado: cloud borrosa/nebulosa
2. Encima, `canvas.drawCircle()` normal de los mismos puntos a tamaño 1.5px sin blur → puntos nítidos sobre la nube

Para los 5.000 puntos y performance:
- Pre-calcular toda la distribución en `initState()` con `compute()` (Isolate)
- Solo recalcular proyección 2D en cada frame de animación (rápido: solo matriz de rotación)
- `RepaintBoundary` para aislar el painter

### Panel lateral de leyenda

Un `Column` a la izquierda del canvas (180px) con:
- Por cada cluster: chip de color + nombre + conteo + porcentaje
- Hover en leyenda resalta ese cluster en el canvas

### Archivos a modificar

- `ui/lib/gallery/sections/section_dataviz_new.dart`: refactor de `StrategyCluster3dWidget` y `_Cluster3dPainter`

### Impacto en documentos

- `DESIGN.md §10 Data-viz`: actualizar spec de `strategy-cluster-3d` con bloom layer, 5.000 puntos, 1.5px, panel lateral de leyenda, 840×600

---

## Cross-cutting: Retroalimentar secciones pre-existentes

Los componentes en secciones antiguas que tienen números, gauges, o gráficos lineales no tienen las animaciones universales (odómetro, arco, path drawing) porque esas reglas se definieron hoy. Se deben actualizar:

- `section_dataviz_quant.dart`: cualquier gauge o metric number → odómetro + arco
- `section_drasus_core_extended.dart`: fitness-evolution curve → path drawing
- `section_std_missing.dart`: si hay progress-circulars → arco animado

---

## Documentos a impactar

| Documento | Cambios |
|---|---|
| `docs/DESIGN.md` | §Colors: acentPrimary dinámico + 12 swatches; §Components: settings-drawer + settings-gear-button; §10 Data-viz: trade-tape, trade-ticker-bar, actualizar monte-carlo-lines y strategy-cluster-3d; §Typography: Rajdhani como displayGrotesque; §Do: regla acento dinámico |
| `docs/ROADMAP.md` | STORY-016 (DrasusTheme), STORY-017 (Dashboard Shell), STORY-018 (Canvas Shell) en EPIC-0 |
| `docs/sad/SAD-06.md` | Actualizar §UI/UX flows: Dashboard y Canvas existen como shells desde EPIC-0 |
| `.claude/skills/ui-designer/SKILL.md` | No requiere cambios — DESIGN.md es fuente de verdad |
| ADRs | No se requieren ADRs nuevos — son decisiones de UI preference/shell, no arquitectónicas |

---

## Resumen de todos los cambios (puntos originales + adiciones A/B/C/D)

| # | Cambio | Archivos Flutter | Documentos |
|---|---|---|---|
| 1 | Acento dinámico + settings drawer | `gallery_tokens.dart`, `main.dart`, todos los GlowInput/Tab/Nav | DESIGN.md, ROADMAP |
| 2 | Dashboard shell | `panel_operativo.dart`, `dashboard_tab.dart` | ROADMAP, SAD-06 |
| 3 | Canvas DAG shell | `panel_operativo.dart`, `canvas_tab.dart` | ROADMAP, SAD-06 |
| 4 | Monte Carlo eléctrico | `section_dataviz_new.dart` | DESIGN.md |
| 5 | Rajdhani display font | `gallery_tokens.dart`, pubspec, assets | DESIGN.md |
| 6 | Trade tape | `section_trade_tape.dart` (nuevo) | DESIGN.md |
| 7 | Galaxy 3D upgrade | `section_dataviz_new.dart` | DESIGN.md |
| A | Efecto eléctrico universal | `section_dataviz_quant.dart`, `gallery_fx.dart` | DESIGN.md |
| B | Vidrio mate más pronunciado | `gallery_tokens.dart`, todos los vidrio | DESIGN.md |
| C | Paleta de fondo dinámica | `gallery_tokens.dart`, `DrasusTheme` | DESIGN.md |
| D | Nodos DAG en galería | `section_dag_nodes.dart` (nuevo) | — |

---

## Orden de ejecución (subagentes)

1. **Subagente UI Designer** → Actualiza `DESIGN.md` con todos los cambios de spec (acento dinámico, Rajdhani, trade-tape, galaxy update, settings drawer). Luego entrega instrucciones al Flutter Engineer.
2. **Subagente Architect** → Actualiza `ROADMAP.md` (STORY-016/017/018) + `docs/sad/SAD-06.md`.
3. **Subagente Flutter Engineer** → Implementa en `ui/`:
   - DrasusTheme + settings drawer + acento aplicado
   - Dashboard tab (shell)
   - Canvas tab (shell)
   - Monte Carlo eléctrico (height 420px, comet tail, ignition decay)
   - Rajdhani en gallery_tokens.dart
   - trade-tape component
   - Galaxy 3D upgrade (5K pts, bloom, panel leyenda)
   - Retroalimentar secciones antiguas con animaciones universales

---

## Verificación adicional (A/B/C/D)

- [ ] Efecto eléctrico visible en equity-curve, rolling-metric, fitness-evolution (no solo MC)
- [ ] Vidrio Apple claramente visible como superficie diferenciada del fondo
- [ ] Settings drawer tiene selector de paleta de fondo (8 opciones) + live preview
- [ ] Cambiar paleta → toda la pila de superficies cambia en tiempo real
- [ ] Sección "Nodos y Conexiones DAG" en galería con 6 estados del nodo + 3 tipos de conexión

---

## Verificación

```bash
# Compilar sin errores
cd ui && flutter build linux --debug

# Smoke test
cd ui && flutter test test/gallery_smoke_test.dart

# Actualizar goldens (la nueva paleta y componentes los invalidan)
cd ui && flutter test --update-goldens test/gallery_golden_test.dart

# Ejecutar galería visual
cd ui && flutter run -d linux -t lib/gallery/gallery_preview_main.dart

# Ejecutar app principal (para ver Dashboard + Canvas tabs)
cd ui && flutter run -d linux
```

Checklist visual en galería:
- [ ] Monte Carlo: altura 420px, comet tail visible, líneas se iluminan al paso del scanner
- [ ] Galaxy 3D: >5K puntos, efecto nebulosa/bloom, panel de leyenda lateral, 840×600
- [ ] Acento dinámico: cambiar color en settings → todos los elementos afectados cambian live
- [ ] Settings gear visible en esquina superior derecha, drawer se abre con slide
- [ ] Dashboard tab: bento grid vacío con botón "+"
- [ ] Canvas tab: DAG canvas con panel lateral de features
- [ ] Rajdhani visible en todos los títulos de sección de galería (angular, no redondeado)
- [ ] trade-tape: scroll continuo, fade en bordes, colores buy/sell
- [ ] Secciones antiguas: gauges y números con animación de entrada
