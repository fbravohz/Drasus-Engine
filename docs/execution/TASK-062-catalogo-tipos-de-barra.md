# TASK-062 · Cerrar el catálogo de tipos de barra y gráfico

| Campo | Valor |
|---|---|
| **ID** | TASK-062 |
| **Tipo** | Task (arquitectura / especificación) |
| **Responsable** | Architect |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-19 |
| **Origen** | Pregunta del propietario sobre el alcance de la agregación (2026-07-19); inconsistencia hallada por el Tech-Lead al contrastar los documentos |

## Contexto

El propietario preguntó si la agregación de ticks a barras y los distintos tipos de gráfico —al estilo de NinjaTrader 8— están planeados. **Sí lo están**, pero el catálogo está incompleto y los documentos no coinciden entre sí.

## Lo que YA está especificado

`docs/features/algorithmic-bars.md` declara cuatro tipos, con sus parámetros configurables, consumiendo un flujo de ticks:

- **Renko** (tamaño de ladrillo)
- **Rango** (rango máximo-mínimo fijo)
- **Volumen** (cierra al alcanzar un umbral de volumen)
- **Tick** (cierra cada N transacciones)

Es decir: la agregación de **ticks → barras de cualquier tipo** ya es parte del diseño, no una idea nueva.

## Las tres inconsistencias a resolver

### 1. El catálogo de tipos no coincide entre documentos

`ADR-0155` fija en el esquema `bar_type TEXT CHECK (col IN ('TIME','TICK_COUNT','VOLUME','DOLLAR','RENKO'))`.

Contraste:

| Tipo | En ADR-0155 | En `algorithmic-bars.md` |
|---|---|---|
| Tiempo | ✅ | (implícito, es el caso base) |
| Tick | ✅ `TICK_COUNT` | ✅ |
| Volumen | ✅ | ✅ |
| Renko | ✅ | ✅ |
| **Rango** | ❌ **ausente** | ✅ especificado |
| **Dólar** | ✅ | ❌ **ausente** |

Dos huecos cruzados: la base de datos **rechazaría** una barra de Rango que la feature dice que vamos a producir, y admite un tipo "Dólar" que ninguna feature especifica.

### 2. Falta Heikin Ashi, y no aparece en ningún documento del repositorio

Búsqueda en todo `docs/`: **cero coincidencias**. Es uno de los tipos de gráfico más usados del sector y el propietario lo nombró explícitamente.

**Ojo, porque no es del mismo tipo que los demás y esa es la parte interesante:** Renko, Rango, Volumen y Tick son **reglas de agregación** — deciden *cuándo cierra* una barra. Heikin Ashi **no**: es una **transformación de los valores** de barras ya construidas, donde cada barra se calcula a partir de la anterior. Es decir, por la doctrina de materia prima frente a indicadores, **Heikin Ashi es un indicador, no un tipo de barra primitivo** — el mismo razonamiento que sacó el precio medio ponderado de la ingesta (DEBT-028).

El Architect debe pronunciarse sobre eso explícitamente, porque determina en qué fase del pipeline vive.

### 3. Falta decidir el resto del "largo etcétera"

El propietario mencionó que NinjaTrader ofrece muchos más. Hay que decidir cuáles entran, cuáles no, y **con qué criterio** — no ir añadiéndolos de uno en uno según se acuerde alguien. Candidatos habituales del sector: Kagi, Punto y Figura, Line Break, Barras de Rango Momentum, barras de desequilibrio y de run (las de López de Prado).

## Qué debe entregar el Architect

1. **El catálogo cerrado y único**, coherente entre ADR-0155, `algorithmic-bars.md` y el `CHECK` del esquema. Un solo sitio canónico; los demás lo citan.
2. **La clasificación de cada tipo** en regla de agregación (ingesta) o transformación derivada (indicador, motor). Con el criterio explícito que permita clasificar los futuros sin volver a discutir.
3. **El criterio de admisión**: qué hace que un tipo de barra merezca entrar en el catálogo.
4. **En qué fase se construye cada uno**, enganchado al ROADMAP.

## Nota de método

Esta Task es exactamente el tipo de hallazgo que la nueva **Etapa 0.4 (Barrido de Coherencia Pre-Orden)** existe para producir *antes* de escribir una Orden, en vez de descubrirlo a mitad de implementación. Aquí salió tarde, por una pregunta del propietario. La siguiente vez debe salir del barrido.

---

## Análisis del Quant-Engineer (2026-07-19, primer despacho — insumo para el Architect, no reemplaza su decisión)

### 1. Veredicto sobre la tesis del TL (Heikin Ashi = indicador)

**Confirmada, con fundamento de microestructura.** Heikin Ashi es una fórmula recursiva sobre valores YA persistidos de barras ya cerradas por alguna otra regla (tiempo, tick, volumen, dólar): `HA_close = (O+H+L+C)/4`; `HA_open = (HA_open_prev + HA_close_prev)/2`; `HA_high = max(H, HA_open, HA_close)`; `HA_low = min(L, HA_open, HA_close)`. No decide *cuándo* cierra una barra — repinta los valores O/H/L/C de una barra que ya cerró bajo otra regla. Encaja exactamente en la doctrina materia-prima/indicador (`docs/debt/DEBT-028.md`, citada también por el propio TL): misma categoría que el VWAP, el ATR o el RSI — una fórmula aplicada sobre primitivos ya persistidos, con el mismo propósito de ayudar a decidir. Pertenece al motor (EPIC-2/EPIC-3), no a la ingesta.

**Matiz de riesgo a documentar cuando se construya (no aquí):** Heikin Ashi es recursivo (necesita semilla/warm-up, igual que una EMA) y su `close` es un promedio — **nunca un precio realmente negociado**. Cualquier simulador que use precios Heikin Ashi como si fueran fills reales viola el mismo principio de realismo que gobierna el Contrato de Consistencia Conservadora (ADR-0114). Advertencia para la futura feature: Heikin Ashi es capa de señal/visualización sobre barras reales, jamás precio de ejecución.

### 2. Catálogo canónico propuesto

**Grupo A — reglas de cierre (ingesta, `bar_type` primitivo, esquema `Bars` estándar sin cambio de forma):**

| Tipo | Estado | Veredicto |
|---|---|---|
| `TIME` | Construido | Caso base, se mantiene |
| `TICK_COUNT` | En ambos docs | Se mantiene (nombre ya coincide) |
| `VOLUME` | En ambos docs | Se mantiene |
| `RENKO` | En ambos docs | Se mantiene |
| `RANGE` | Falta en el `CHECK` de ADR-0155, presente en `algorithmic-bars.md` como "Rango" | **Añadir al `CHECK`** — mismo concepto, unificar nombre a `RANGE` (identificadores en inglés, ADR-0121) |
| `DOLLAR` | En el `CHECK` de ADR-0155, ausente en `algorithmic-bars.md` | **Sobrevive — añadir a `algorithmic-bars.md`.** Ver justificación abajo |

**`DOLLAR` debe sobrevivir.** Es la barra de dólar de López de Prado (*Advances in Financial Machine Learning*, 2018, cap. 2): cierra al cruzar un umbral de valor nocional acumulado (precio × volumen), no un umbral de volumen o de ticks. Es más robusta que `VOLUME`/`TICK_COUNT` frente a cambios estructurales de participación de mercado a lo largo del tiempo (la razón por la que un umbral de volumen fijo queda descalibrado en 5 años, uno de valor nocional no). Es una regla de cierre legítima, causal, computable en ingesta — misma categoría que `VOLUME`/`TICK_COUNT`/`RENKO`. La inconsistencia con `algorithmic-bars.md` es un vacío de sincronía documental (el insumo original del Quant, `.agents/plans/esquema-canonico-datos-mercado.md:83`, ya la incluía sin cruzarla contra `algorithmic-bars.md` — error de origen propio, se reconoce). Nota: `DOLLAR` es exactamente el tipo de barra que más sufre la asimetría de escala precio/volumen analizada en TASK-061 §5 (acumula precio×volumen toda la sesión) — su implementación debe esperar a que esa Task se resuelva.

**Candidatos nuevos con valor cuantitativo real, ausentes hoy — recomendados para el ROADMAP, no para esta Story:**
- **Barras de desequilibrio y de "run"** (tick/volumen/dólar imbalance y run bars — López de Prado, misma referencia, §2.3.2–2.4). Mismo esquema `Bars` (sin cambio de forma), pero con umbral **adaptativo** derivado de un EWMA de barras previas — mejores propiedades estadísticas (retornos más cercanos a i.i.d./Gaussianos) que `TICK_COUNT`/`VOLUME`/`DOLLAR` simples. Riesgo real a vigilar en su implementación: el umbral de la barra N debe derivarse **solo** de las barras `1..N-1`, nunca de la propia barra en curso — de lo contrario es look-ahead bias.

**Grupo B — no encajan en el esquema OHLC estándar de `Bars`, requieren decisión de forma de salida antes de admitirse (no es solo agregar un valor al `CHECK`):**
- **Kagi, Point & Figure, Line Break.** Confirmado por búsqueda externa (NinjaTrader, soporte oficial): **el propietario tiene razón, NT8 sí los ofrece**, junto con Heikin Ashi y "Order Flow Volumetric/Delta Bars". Los tres primeros son construcciones basadas en reversión de precio, con salida nativa de **líneas/cajas sin eje de tiempo uniforme** — no son naturalmente `open/high/low/close`. Admitirlos exige que el Architect decida primero su tipo de salida (¿aproximar a OHLC perdiendo la esencia del método, o modelar un tipo de dato nuevo, distinto de `Bars`?) — pregunta abierta, no la cierro yo.
- **Order Flow Volumetric/Delta Bars (NT8).** No son un tipo de barra nuevo — son una vista de volumen por nivel de precio sobre datos que **ya están canonizados**: `DepthLevel` (Nivel 2) y `OrderEvent` (Nivel 3) de ADR-0155. No requieren nueva entrada en `bar_type`.
- **"Barras de Rango Momentum"** (mencionada en el contexto de esta Task): no es un término estándar de la literatura de López de Prado ni aparece en la documentación oficial de NinjaTrader 8 verificada. Búsqueda: NinjaTrader 8 bar types (soporte oficial) y López de Prado cap. 2 — cero coincidencias con ese nombre exacto. Vacío real — posible variante propietaria de Range Bar con filtro de momentum, o confusión de nombre; no se puede admitir al catálogo sin una fuente concreta.

### 3. Criterio de admisión propuesto

Un tipo de barra entra al catálogo `bar_type` si cumple las tres: (a) es una regla de cierre computable causalmente desde el flujo de ticks, sin usar información posterior al cierre de la propia barra; (b) su salida encaja en el esquema `Bars` (`open/high/low/close` + metadatos) sin inventar una forma nueva — si no encaja, es Grupo B y necesita su propio tipo de dato antes de admitirse; (c) tiene respaldo verificable — literatura cuantitativa (López de Prado) o adopción de plataforma mayor confirmada por fuente (NinjaTrader, TradingView). Cualquier transformación de valores de barras **ya cerradas** (Heikin Ashi y análogos) no es `bar_type`: es indicador, va al motor.

### 4. Consecuencias de sesgo por tipo de barra — advertencia para quien haga backtest

- **`TIME`** (el único construido hoy): sobremuestrea períodos de baja actividad y submuestrea ráfagas de alta actividad/información — produce retornos heterocedásticos, no gaussianos y con autocorrelación serial inducida (López de Prado, cap. 2, es la crítica central que motiva el resto del catálogo). Es, de las opciones estadísticamente estudiadas, la de peores propiedades — y hoy es la única fuente de datos para el guantelete (DSR/WFA/CPCV/Monte Carlo), cuyos supuestos de baja autocorrelación descansan sobre el tipo de barra con más autocorrelación inducida del catálogo.
- **`TICK_COUNT`**: mejor que `TIME`, pero no estacionario en horizontes largos — el número de ticks por unidad de tiempo real cambia estructuralmente con la evolución del trading algorítmico; un backtest de varios años sobre `TICK_COUNT` mezcla regímenes de actividad distintos bajo el mismo umbral.
- **`VOLUME`**: mejor contenido informativo que `TICK_COUNT` (pondera por tamaño, no solo cuenta), pero mismo problema de estacionariedad ante crecimiento estructural de participación.
- **`DOLLAR`**: la más robusta a cambio estructural de las cuatro simples (razón de ser en López de Prado); mejor default para backtests de horizonte largo.
- **`RENKO`/`RANGE`**: nacen para filtrar ruido **visual** (uso discrecional/técnico), no para robustez estadística — descartan movimientos pequeños deliberadamente, lo que induce autocorrelación artificial en los movimientos que sí registran. Advertencia explícita: si el guantelete corre alguna vez sobre series Renko/Range, los supuestos de independencia que CPCV/WFA/DSR asumen son más propensos a violarse; debe señalarse en la UI/documentación de cualquier feature futura que las consuma para validación estadística, no solo para visualización.
- **Imbalance/Run bars** (si se construyen): mejor propiedad estadística del grupo, pero requieren implementación cuidadosa del umbral adaptativo — el riesgo de look-ahead descrito en §2 es real si se implementa ingenuamente.

### 5. Confirmación sobre NinjaTrader 8

**El propietario tiene razón.** Verificado contra la documentación oficial de soporte de NinjaTrader (`ninjatrader.com/support/helpGuides/nt8/bar_types.htm` y `chart_styles.htm`): el catálogo de NT8 incluye Renko, Range, Kagi, Point & Figure, Line Break, Heikin Ashi y Order Flow Volumetric/Delta Bars — más amplio que el catálogo actual de `algorithmic-bars.md` (4 tipos). De ese conjunto, con valor cuantitativo real y no cubierto hoy: `DOLLAR` (ya en ADR-0155, falta sincronizar), Kagi/P&F/Line Break (Grupo B, pendientes de decisión de forma), y — fuera del catálogo de NT8 pero con más rigor estadístico que todo lo de NT8 — las barras de desequilibrio/run de López de Prado, coherentes con que este proyecto ya cita esa misma literatura para DSR/CPCV.

### Preguntas abiertas para el Architect

1. ¿Se acepta `DOLLAR` como quinto `bar_type` de ingesta (Grupo A), con implementación diferida hasta que TASK-061 fije la escala precio/volumen?
2. ¿Se admite Kagi/Point & Figure/Line Break al alcance del proyecto? Si sí, ¿qué tipo de dato de salida usan (no `Bars`) y en qué fase/EPIC del ROADMAP se construyen?
3. ¿Se añaden las barras de desequilibrio/run de López de Prado al ROADMAP como candidatos de mayor rigor estadístico que el catálogo de NinjaTrader, aunque el propietario no las haya nombrado?
4. ¿Quién redacta la advertencia de sesgo estadístico por tipo de barra (§4) que debe llegar a la UI/documentación de cualquier feature de validación (WFA/CPCV/Monte Carlo/DSR) que permita elegir la fuente de barras?

---

## Resolución del Architect (2026-07-19)

**Catálogo cerrado:** `TIME`, `TICK_COUNT`, `VOLUME`, `RENKO`, `RANGE`, `DOLLAR` — seis valores, un solo sitio canónico en `docs/features/algorithmic-bars.md`, `ADR-0155` cita ese documento y su `CHECK` ya se corrigió para coincidir (`RANGE` añadido, faltaba pese a estar ya especificado). Heikin Ashi confirmado como indicador, no `bar_type` — no se añade al catálogo, se documenta su exclusión y el porqué en `algorithmic-bars.md`. Kagi/Point & Figure/Line Break **no admitidos** — Grupo B, sin tipo de dato de salida decidido y sin caso de uso concreto que lo demande hoy; disparador de reapertura documentado. Imbalance/Run bars anotadas como candidatas de ROADMAP, sin EPIC asignado. Criterio de admisión canonizado (tres condiciones) para no volver a discutir esto por cada tipo nuevo. Advertencia de sesgo estadístico (§4) trasladada íntegra a `algorithmic-bars.md` para que no se pierda antes de que exista la feature de validación que la necesite. Detalle completo: `.agents/plans/architect-2026-07-19-escala-barras-ingesta.md`.
