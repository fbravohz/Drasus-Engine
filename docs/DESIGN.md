# DESIGN.md — Sistema de Diseño

Fuente única de verdad visual de Drasus Engine: tokens, gramática de layout, comportamiento nativo, motion y catálogo de componentes. Autoridad de estructura: `ADR-0138` (tokens en `ThemeExtension`, componentes funcionales neutrales), `ADR-0139` (regla de reutilización, ≥2 consumidores), `ADR-0106`/`ADR-0097` (render Impeller, cero WebView), `ADR-0136` (Dashboard + Canvas).

---

## 0. Principios Rectores

1. **Sobriedad por defecto.** Un componente nuevo nace sin glow, sin gradiente, sin decoración — el efecto se añade solo si comunica un estado real (dato, foco, o cambio). Glow/vidrio son acento, no textura de fondo del sistema entero.
2. **El token es la única fuente de valor visual.** Ningún color, tamaño, radio o duración vive como literal fuera de este documento y su implementación en `ThemeExtension`. Si hace falta un valor nuevo, se añade aquí primero.
3. **Componente = contrato funcional, no estética horneada.** Un componente nunca ramifica por modo de tema ni por color de acento — consume el token dinámico. Nombres neutrales, sin prefijo de marca ni de estilo (`Button`, no `GlowButton` ni `DrasusButton`).
4. **Construcción bajo demanda, no especulativa.** Un componente se construye cuando tiene un consumidor real (`ADR-0139`); la librería no se llena de piezas "por si acaso". El índice vivo de lo que existe y quién lo consume es `ui/COMPONENTS.md` — este documento define el vocabulario y la regla, no la lista viva.
5. **Terminado se demuestra, no se declara.** Ningún componente ni pantalla se reporta como completo sin evidencia verificable (§7, Piso de Verificación). El código que implementa un estado no es prueba de que ese estado se ve o se siente terminado.
6. **Nativo antes que pintado.** Antes de resolver algo con un `CustomPainter` o un efecto propio, verifica si Flutter ya ofrece el comportamiento nativo equivalente (selección de texto, scroll, foco). Pintar todo a mano es la causa más común de que una app Flutter se sienta "sobrepuesta" — §5 fija el piso obligatorio.

---

## 1. Tokens — Superficie y Color

Implementados como `ThemeExtension`s (nunca constantes sueltas). Acceso desde widgets vía extensiones de contexto: `context.colors`, `context.surfaces` — nunca un facade estático global ni un literal `Color(0x...)`.

### 1.1 Pila de superficie (modo oscuro, ancla del sistema)

| Token | Hex | Uso |
|---|---|---|
| `canvas` | `#0B0D12` | Fondo raíz de la ventana. Nunca negro puro. |
| `surface` | `#12141B` | Paneles de contenido: tablas, formularios, tarjetas. |
| `surfaceRaised` | `#181B24` | Hover de fila, elemento elevado, celda activa. |
| `border` | `#242832` | Borde hairline por defecto — 1px, nunca gris puro sin tinte. |
| `borderStrong` | `#343A47` | Borde de énfasis: foco, selección, división estructural. |
| `overlay` | `#12141BE6` (90% alpha) + blur 20 | Chrome flotante: menú, modal, tooltip, popover. Único lugar donde el "vidrio" es el default — nunca en superficies de contenido denso. |

### 1.2 Texto

| Token | Hex | Uso |
|---|---|---|
| `textPrimary` | `#EDEFF3` | Títulos, valor destacado. |
| `textSecondary` | `#A6ACBA` | Cuerpo, descripciones, cabeceras de panel. |
| `textMuted` | `#6B7180` | Labels, metadatos, placeholder. |
| `textOnAccent` | `#0B0D12` | Texto sobre un fondo de acento saturado (botón primario). |

### 1.3 Acento dinámico

Un único color de acento, elegido por el usuario (selector híbrido: swatches curados + rueda HSV — un solo patrón de entrada de color en todo el sistema). El acento tiñe chrome interactivo (foco de input, tab activo, scrollbar, borde de selección) — **nunca** los colores semánticos de vitalidad (1.4). Default de fábrica: `#4C8DFF` (azul contenido, no saturado al máximo).

### 1.4 Espectro semántico de vitalidad

Cuatro estados, un color cada uno — sin variantes redundantes. Es información de negocio, nunca decoración; todo uso lleva un estado real detrás.

| Estado | Token | Hex | Significado de negocio |
|---|---|---|---|
| Óptimo | `statusOk` | `#3DD68C` | Salud correcta, completado, tendencia confirmada. |
| Transición | `statusPending` | `#7C8CFF` | En curso, incubando, calculando. |
| Alerta | `statusWarn` | `#E8A93D` | Atención requerida, deriva, pausa. |
| Crítico | `statusError` | `#E8543D` | Fallo, riesgo, estado terminal. |

**Regla de chip único para cualquier estado:** fondo = color @ 12% sobre `surface`; borde 1px color @ 35%; texto = color puro; radio 6px (999px solo si el estado es "vivo" con animación de pulso). Sin excepciones por componente.

### 1.5 Modos de superficie — registro abierto, semilla mínima

La arquitectura es N-extensible (`ADR-0138` enmienda 2026-06-25): un modo nuevo es una entrada en el registro + su receta, cero cambios en componentes. La semilla de fábrica son **dos modos**, no ocho paletas — se añaden más solo cuando un caso de uso real lo pida:
- `solid` (default): la pila de §1.1 tal cual, sin blur. Úsalo en el 90% de la app — dashboards, tablas, formularios, paneles densos.
- `glass`: aplica `overlay` (blur + transparencia) — reservado a chrome flotante (§1.1). Nunca es el modo por defecto de una pantalla completa.

---

## 2. Tokens — Tipografía

Tres roles, no nueve. Cada número va en `mono`, alineado a la derecha; ningún otro rol toca un número.

| Rol | Familia | Tamaño | Peso | Uso |
|---|---|---|---|---|
| `mono` | monoespaciada (JetBrains Mono o equivalente del sistema) | 13px | 400/500 | Todo número, fecha, ID, símbolo, porcentaje. |
| `body` | sans (Inter o equivalente del sistema) | 14px | 400/500 | Texto de UI: labels, cuerpo, botones, navegación. |
| `heading` | sans, mismo family que `body` | 18–24px | 600 | Títulos de sección y de panel — **una sola familia tipográfica en toda la app**, diferenciada por peso y tamaño, no por una segunda fuente "de exhibición". |

**Regla dura:** una jerarquía de texto se construye con tamaño y peso, nunca añadiendo una tercera familia tipográfica. Elimina la tentación de una tipografía "ceremonial" separada — es la fuente más común de inconsistencia visual entre pantallas.

---

## 3. Tokens — Espaciado y Radios

Escala base 4px, sin excepciones: `4 · 8 · 12 · 16 · 24 · 32 · 48 · 64`.

| Elemento | Radio |
|---|---|
| Panel / tarjeta | 10px |
| Chrome flotante (menú, modal, tooltip) | 12px |
| Botón / input | 8px |
| Chip / badge | 6px (999px si es estado "vivo") |

---

## 4. Gramática de Layout (piso obligatorio)

El motivo de esta sección: Flutter no impone una gramática de flujo como HTML/CSS — cada pantalla se posiciona a mano. Sin una regla explícita, cada agente resuelve el layout distinto y el resultado se siente "sin orden". Esta gramática es el reemplazo deliberado de esa disciplina implícita que Flutter no trae de fábrica.

- **`Row`/`Column`/`Expanded`/`Flexible` son el default.** Resuelven el 95% de cualquier pantalla: formularios, listas, paneles, barras.
- **`Stack`/`Positioned` solo en tres casos:** (a) overlays (tooltip, menú, badge sobrepuesto), (b) `CustomPainter` de datos (gráficas, canvas DAG), (c) efectos decorativos explícitamente aprobados (§8). Fuera de esos tres casos, `Positioned` en una revisión de código es defecto, no estilo.
- **Ancho de contenido con tope.** Ninguna pantalla de formulario o lectura estira su contenido al 100% de una ventana ultra-wide sin límite: usa `ConstrainedBox(maxWidth: ...)` centrado. Paneles de datos densos (tablas, dashboard) sí pueden ocupar el ancho completo — es la excepción declarada, no el default.
- **Alineación consistente por eje.** Dentro de un mismo panel, todos los elementos de un eje comparten el mismo punto de anclaje (`CrossAxisAlignment` único por `Column`/`Row`) — mezclar anclajes dentro del mismo contenedor es el patrón más visible de "elementos flotando sin orden".
- **Espaciado explícito, nunca estimado.** El gap entre elementos hermanos es siempre un token de §3 vía `SizedBox`/`gap` — nunca un padding "que se ve bien" a ojo.

---

## 5. Paridad de Comportamiento Nativo (piso obligatorio)

Flutter dibuja su propio árbol — no hereda gratis lo que un navegador o un toolkit nativo dan por defecto. Sin esta lista, la app se siente "pintada encima" en vez de "una interfaz real". Ninguno de estos puntos es opcional para una pantalla que se declare terminada.

- **Selección de texto.** La app envuelve su árbol en `SelectionArea` desde la raíz. Todo texto de dato (valores, IDs, contenido de celdas, mensajes de error) debe ser seleccionable y copiable — un `Text` plano sin selección en una pantalla de datos es defecto, no default aceptable.
- **Orden de foco y navegación por teclado.** Todo formulario y panel interactivo define un orden de `Tab` explícito y coherente (`FocusTraversalGroup`/`FocusTraversalOrder`) — no el orden accidental de construcción del árbol de widgets. Atajos de teclado estándar (`Enter` para confirmar, `Esc` para cerrar overlay) funcionan en todo overlay/modal.
- **Scrollbars visibles, no solo al arrastrar.** Todo contenedor con overflow usa `Scrollbar` con `thumbVisibility: true` en desktop — el scroll invisible hasta que el usuario arrastra es un patrón de app móvil, no de escritorio.
- **Menú contextual donde un usuario de escritorio lo espera.** Clic derecho sobre una celda de tabla, un nodo o un campo de texto ofrece las acciones aplicables (copiar, ver detalle) — no queda en blanco ni cae al menú del sistema operativo.
- **Cursor correcto por elemento.** `MouseRegion` con el cursor apropiado (`click` sobre interactivo, `text` sobre texto seleccionable, `resize` sobre bordes redimensionables) — el cursor de flecha genérico sobre todo es la señal más rápida de que nada es interactivo.

---

## 6. Motion Philosophy

La animación es funcional, nunca decorativa — cada una comunica cambio de dato, foco o estado. Antes de animar algo nuevo, resuelve en orden: (1) **¿debe animar?** — nunca en acciones repetitivas de alta frecuencia (atajo de teclado, toggle usado cientos de veces al día: ahí se siente como lentitud, no pulido); (2) **¿qué propósito?** — continuidad espacial, indicación de estado, feedback de acción, o prevenir un salto brusco; (3) **¿qué curva?** — tabla abajo; (4) **¿qué duración?** — tabla abajo.

**Curvas por rol** (nunca `Curves.easeIn` puro en una interacción de UI — arranca lento y se siente lento; resérvalo para salidas que ceden protagonismo):

| Rol | Curva Flutter | Duración |
|---|---|---|
| Entrada/salida de overlay (dropdown, tooltip, popover) | `Curves.easeOut` | ~200ms |
| Movimiento en pantalla (drag, reorder, zoom de canvas) | `Curves.easeInOut` | ~300–460ms |
| Velocidad constante (marquee, progreso indeterminado) | `Curves.linear` | continuo |
| Clic de botón (hundimiento + feedback) | `Curves.easeOut` | ~160ms |

**Reglas duras:**
- Nunca animar `Transform.scale`/tamaño desde `0` — nada real aparece de la nada. Arranca desde `0.92–0.96` + opacity.
- Popovers y menús escalan desde el `Alignment` de su trigger, nunca desde `Alignment.center` — excepción: los modales sí centran.
- Listas con más de 3 ítems entrando juntos: stagger de 30–80ms entre ítems, nunca todos a la vez.
- Respeta `MediaQuery.of(context).disableAnimations`: reduce el movimiento, nunca lo elimines por completo — conserva transiciones de color/opacidad del espectro de vitalidad, retira desplazamiento/escala.
- Anima solo propiedades baratas para el raster thread (`Transform`, `Opacity`, color) — nunca reconstruyas layout (padding, tamaño) en cada frame de una animación en curso.

---

## 7. Piso de Verificación (Craft Floor)

Checklist obligatoria antes de dar por **terminada** cualquier pantalla, panel o componente — para el UI Designer (al especificar) y el Flutter Engineer (al implementar y reportar). Un sistema de tokens correcto en el papel no garantiza una ejecución que se sienta terminada: la brecha casi siempre está en uno de estos puntos, no en los hex.

- **Ciclo completo de estados.** Todo componente interactivo declara y muestra: default, hover, focus, pressed, disabled, loading, error y empty — cuando aplican. Si un estado genuinamente no aplica (ej. un chip read-only no tiene "disabled"), se declara explícitamente por qué; nunca se omite en silencio.
- **Loading nunca es un spinner genérico por defecto.** Usa un esqueleto con la forma del layout final salvo que la operación sea instantánea o el spinner de estado ya exista en el vocabulario.
- **Vacío es una composición, no un hueco.** Un estado empty (tabla sin filas, portafolio sin estrategias) indica cómo poblarlo — nunca un panel en blanco sin mensaje ni acción.
- **Piso de accesibilidad táctil:** área de toque mínima **48×48** con **8px** mínimos de separación entre controles interactivos adyacentes. En desktop con mouse el hitbox visual puede ser menor al ícono, pero el hitbox de interacción (`GestureDetector`/`InkWell`) mantiene el piso con padding invisible.
- **Piso de contraste:** todo par texto/fondo, y todo control/icono interactivo, cumple **≥4.5:1** (texto normal) o **≥3:1** (texto grande ≥18px, iconos, bordes de control) medido contra el color de fondo real de esa superficie.
- **Squint test:** antes de cerrar una pantalla, obsérvala desenfocada o reducida a miniatura. La jerarquía primaria tiene que ser identificable sin leer una sola palabra. Si todo compite por igual, no hay jerarquía.
- **Prueba en estados extremos:** nombre muy largo, tabla vacía, cifra de muchos dígitos, ventana redimensionada al mínimo soportado. El layout no debe romperse en ninguno de estos casos.
- **Prueba, no afirmes.** Un reporte de componente o pantalla "completada" se verifica contra una captura real de la app corriendo (`flutter run`/screenshot), no contra la lectura del código ni la intención de diseño. El código implica intención; la captura implica resultado — solo el segundo cuenta como evidencia.

---

## 8. Vidrio y Glow — uso restringido, no textura por defecto

El vidrio (`overlay`, §1.1) y el glow son acento para chrome flotante y para se ñalar un estado "vivo" — no la textura por defecto de cada superficie. Antes de aplicar cualquiera de los dos, responde: ¿esto comunica algo (foco, estado activo, jerarquía flotante) o solo decora? Si es lo segundo, no se aplica.

- **Vidrio (`overlay`):** solo en chrome flotante (menú, modal, tooltip, popover) y en la barra de navegación. Nunca en un panel de contenido denso (tabla, formulario, tarjeta) — ahí siempre `surface` sólido.
- **Glow:** reservado a dos casos — (a) foco de un control interactivo (`glow(accent)`, blur 8, opacity 0.35), (b) indicar un estado "vivo" del espectro de vitalidad (`glow(statusPending)` pulsante mientras algo corre). Un botón primario en reposo, una tarjeta est ática, o un ícono decorativo **no** llevan glow.
- **Ningún widget decorativo sin función.** Un elemento visual (orbe, telón de fondo animado, textura de partículas) se añade solo si comunica un estado del sistema — nunca como ambientación. Si un ui-designer propone uno, debe nombrar qué dato o estado representa antes de que se apruebe.

---

## 9. Catálogo de Componentes

El vocabulario vive en código (`ui/lib/components/`, un archivo por componente, nombres neutrales sin prefijo de marca) y su índice vivo es `ui/COMPONENTS.md` (componente · archivo · props/callbacks · estados · consumidores reales) — este documento no duplica esa lista, la gobierna.

**Regla de elevación (`ADR-0139`):** un componente entra a la librería compartida cuando tiene ≥2 consumidores reales. Con un solo consumidor, el widget vive inline en la feature — elevarlo antes es sobre-ingeniería especulativa, exactamente el patrón que dejó la librería anterior con piezas sin usar.

**Categorías del vocabulario** (no una lista cerrada — se extiende bajo demanda):
- **Layout:** panel, tarjeta, tabs, stepper.
- **Navegación:** breadcrumb, paleta de comandos, árbol de navegación.
- **Inputs:** campo de texto, selector, slider, date picker, campo de formulario completo.
- **Acciones:** botón primario/secundario/terciario/destructivo.
- **Datos:** tabla, key-value, chip/badge, medidor, barra/círculo de progreso.
- **Feedback:** alerta/banner, toast, modal, loader/skeleton.
- **Data-viz de dominio:** cada visualización financiera (curva de equity, drawdown, heatmap, dispersión, etc.) se especifica en la Cáscara Visual de la feature que la origina — no se pre-declara aquí sin consumidor real.

**Contrato de todo componente nuevo:**
- Nombre neutral (`Button`, no `GlowButton`/`AppButton`), consumido con import namespaced (`import '...' as ui;` → `ui.Button`).
- Sin lógica de negocio ni cálculo — recibe datos ya resueltos.
- Sin estilo horneado — consume los tokens de §1–§3 vía `context.*`, nunca un color/tamaño literal.
- Cumple el Piso de Verificación (§7) antes de entrar a la librería.

---

## 10. Gate de Gusto — antes de aprobar una Cáscara Visual

Ninguna sección `## Cáscara Visual` se da por aprobada solo porque usa tokens válidos y componentes existentes. Antes de cerrarla, el UI Designer verifica:
- **Restraint:** ¿cada efecto (glow, vidrio, animación) tiene una función declarada, o es decoración? Si no puede nombrarla, se retira (§8).
- **Jerarquía real:** aplica el squint test (§7) sobre el layout propuesto, no solo sobre un componente aislado.
- **Comparación con una referencia real.** La spec nombra a qué interfaz profesional se parece el resultado esperado (no una app genérica de IA) — un dashboard de datos serio, sobrio, legible.
- **Consistencia entre pantallas.** La densidad, alineación y vocabulario de esta Cáscara Visual coinciden con los de features ya entregadas — una pantalla que "se ve distinta" al resto del sistema sin razón declarada no pasa el gate.

---

## 11. Gobernanza

Trazabilidad: `ADR-0106` (render Impeller, cero cálculo en Dart), `ADR-0097` (cero WebView), `ADR-0117` (Cáscara Delgada), `ADR-0136` (Dashboard + Canvas, chrome consume `overlay`), `ADR-0137` (hexágono — UI consume primitivos), `ADR-0138` (arquitectura de tokens y componentes que este documento reinstancia), `ADR-0139` (regla de reutilización general).

Autoridad de edición: el UI Designer edita este documento bajo las reglas de su skill (`.agents/skills/ui-designer/SKILL.md`). Cambios a §1–§3 (valores de token fundamentales) requieren visto bueno del Architect.
