# CONTENT-STRATEGY.md — Drasus Engine

> **Marco de referencia:** `content-creator.md` (brief original), `.claude/skills/social-strategist/SKILL.md` (motor de ejecución y orquestación de esta estrategia) y `docs/SAD.md` / `docs/ROADMAP.md` (arquitectura y plan de épicas).
> **Objetivo:** Construir autoridad técnica real en el nicho de trading cuantitativo/algorítmico — **en español e inglés** —, con producción visual de nivel QuantumFracture / LeMMiNo / HistoryMarche, **sin depender de mostrar la cara**, sin "vender el sueño" (robots, señales, cursos) — **y arrancando HOY, con 14 commits y un proyecto en EPIC-0 (la fase de cimientos).**

---

## 🧭 Cómo Leer Este Documento (Empieza Aquí)

**En una frase:** cuando termines un trabajo (con código o sin él), corre `/social-strategist`. Te dice qué detectó, te da 2-4 opciones de qué publicar o producir, y tú elige un número. Si lo que hiciste todavía no alcanza para un video, sale algo corto de todos modos — nunca te quedas sin nada que mostrar.

### Glosario rápido

| Término | En español llano |
|---|---|
| **Épica (EPIC-n)** | Un gran bloque del plan. Ej: "Épica 0" = construir los cimientos. |
| **Historia (STORY-###)** | Un trabajo que produjo código (ej. "el reloj del motor"). |
| **Riesgo investigado (SPIKE-###)** | Una decisión técnica de alto riesgo, ya resuelta y documentada. |
| **Tarea (TASK-###)** | Trabajo sin código (ej. una auditoría). |
| **Decisión documentada (ADR-XXXX)** | El "por qué" de una decisión técnica, ya por escrito. |
| **Pulso (Tier 1)** | Publicación corta y automática (Twitter/Discord/IG) cada vez que cierras algo. |
| **Episodio (Tier 2)** | Video de tamaño medio. No necesita datos reales del motor. Disponible ya. |
| **Gran Estreno (Tier 3)** | Video grande con datos reales del motor (gráficos, curvas). Se activa por fases. |
| **Pilar** | Un tema recurrente del canal (ej. Pilar G = "cómo construimos esto desde cero"). |
| **Capa** | Una capa visual del video — voz, pantalla real, animación, etc. (§4). |
| **Masa narrativa** | Cuando varias publicaciones de Pulso ya cuentan, juntas, una historia completa — momento de subir a Episodio. |

> El resto del documento usa estos términos sin repetir la traducción. Si una frase no se entiende en una sola lectura, vuelve a esta tabla.

---

## 0. Tesis Central (TL;DR)

1. **El producto ES el contenido.** Drasus Engine documenta, en cada ADR/STORY, los problemas que el 95% del contenido hispano de trading algorítmico ni siquiera nombra. Cada decisión es un guion en bruto.
2. **La cara es opcional, la voz no.** Autoridad = voz humana consistente + visuales de alto nivel. La cara entra de forma *progresiva* (§4.1), nunca como requisito día 1.
3. **El B-roll real gana siempre.** IDE con Rust real, `cargo test` en verde, el primer dashboard — eso es mejor que cualquier render genérico. La IA generativa (Manim, Remotion, Runway, Midjourney) es **acelerante**, no sustituto.
4. **Toda pieza debe defenderse en una sala de Quants.** Si se simplifica, se declara. Esa honestidad ES el diferenciador.
5. **Pipeline code-first** (Manim + Remotion + ffmpeg + OBS + DaVinci Resolve) da el 80% del valor visual con herramientas que Claude puede generar/automatizar. Adobe/Canva = pulido manual opcional.
6. **🆕 "Pocos commits" no es un problema, es el Episodio 0.** Drasus Engine está en `EPIC-0 (Fundación y Spikes)`, con 14 commits y 5 STORYs cerradas. **La narrativa de "cómo se construye algo riguroso desde absolutamente cero, en público, antes de tocar una sola estrategia" es en sí misma el primer arco de contenido** — y nadie en el nicho hispano lo está haciendo. Ver §1.

---

## 1. Realidad de Partida: EPIC-0 y la Oportunidad de "Building in Public"

### 1.1 Dónde estamos hoy (verificado en `git log` + `docs/execution/` + `docs/ROADMAP.md`)

| ID | Qué hizo | Estado | Gancho narrativo |
|---|---|---|---|
| **STORY-001** | Workspace Cargo con los 8 módulos + `shared`, cajas vacías que compilan | ✅ | "Antes de escribir lógica, diseñamos el esqueleto completo de un motor que va a manejar dinero real." |
| **STORY-002** | Primera migración SQLite (25 campos maestros) + pool WAL | ✅ | "La primera decisión de nuestro motor de trading fue... sobre una base de datos. Así pensamos la soberanía de datos desde el día 1." |
| **STORY-003** | Reloj real/determinista — mismo input → mismo resultado, siempre | ✅ | "Le construimos un reloj propio a nuestro motor. Sin esto, ningún backtest es confiable." |
| **STORY-004** | Audit-log inmutable, encadenado por hash (append-only) | ✅ | "Cada evento queda encadenado por hash — alterar el pasado se detecta al instante. Sí, como un blockchain, pero para auditar tus propias decisiones." |
| **STORY-005** | Cola de trabajos async con recuperación tras `kill -9` | ✅ | "¿Qué pasa si el motor se cae a mitad de un cálculo costoso? Construimos esto para que la respuesta sea: nada." |
| **TASK-006** | Auditoría asistida por IA de 137 features del diseño (cerrada) | ✅ | "Antes de construir, auditamos 137 piezas de nuestro propio plano — con IA, y con un proceso que cualquier auditor podría revisar." |
| **STORY-007** | `telemetry` — buffer de alta velocidad + heartbeat | ver `docs/ROADMAP.md` | (estado vivo, no copiar aquí — consultar antes de publicar) |

**Estado de EPIC-0:** Los 6 SPIKEs de viabilidad bloqueantes (SPIKE-001 a SPIKE-006) **ya tienen veredicto documentado como ADR** (ADR-0107, ADR-0112 a ADR-0116). Resta validación residual (smoke tests), pero **las decisiones — y sus razones — ya existen y son publicables hoy**:

| Spike | Decisión ya tomada | Por qué es contenido HOY |
|---|---|---|
| SPIKE-001 (ADR-0107) | NautilusTrader se integra como crates Rust nativos, sin Python | "Integramos un motor institucional... sin un intérprete de Python en el medio." |
| SPIKE-002 (ADR-0112) | **Erradicamos `tch-rs`/libtorch** del árbol de dependencias | "Eliminamos PyTorch de nuestro motor de IA — antes de escribir el motor de IA." |
| SPIKE-003 (ADR-0113) | **Erradicamos PySR**; regresión simbólica como modo de NSGA-II nativo | "Dijimos no a una herramienta que casi todo el mundo en 'IA cuantitativa' usa." |
| SPIKE-004 (ADR-0114) | Motor de backtest dual (Express híbrido + Event-Driven) | "Por qué un solo modo de simulación nunca es suficiente." |
| SPIKE-005 (ADR-0115) | Ollama derogado; LLM local soberano opcional | "Nuestro 'copiloto' nunca depende de un servidor externo." |
| SPIKE-006 (ADR-0116) | `flutter_rust_bridge` con downsampling obligatorio | "Cómo evitar que un gráfico de 1M de puntos congele tu interfaz." |

### 1.2 El giro: la fundación ES la historia

El error sería esperar a EPIC-3 (Generación) o EPIC-4 (Guantelete) para "tener algo que mostrar". **Eso es pensar como un producto, no como una narrativa.** El arco "Episodio 0: seis apuestas que podían hundir el proyecto, decididas con evidencia antes de escribir una línea de trading" es:

- **Demostrable hoy** (los ADRs ya están escritos y commiteados).
- **Diferenciador absoluto**: nadie en el nicho hispano muestra *el proceso de decisión* de un motor cuantitativo.
- **Coherente con el manifiesto** (§2): rigor antes que velocidad, transparencia sobre hype.
- **Genera anticipación**: cada SPIKE resuelto hoy se "paga" más adelante cuando EPIC-2/3 entreguen los resultados que esas decisiones hicieron posibles ("¿recuerdas cuando dijimos que el motor dual nos daría X? Aquí está, con datos reales").

### 1.3 Sistema de 3 Tiers de Contenido

Resuelve la tensión entre "producción altísima" y "ritmo de desarrollo aún lento/temprano":

| Tier | Nombre | Disparador | Producción | Pilares |
|---|---|---|---|---|
| **Tier 1 — Pulso** | Cada STORY/SPIKE/TASK cerrado | Automático, vía `/social-strategist` (skill ya reducido a 114 líneas) | Mínima (texto + screenshot + prompt de imagen IA) | Alimenta Discord/Twitter constantemente, mantiene la "bitácora pública" viva |
| **Tier 2 — Episodio** | 2-4 STORYs relacionadas o un SPIKE/ADR con peso narrativo propio | Media (Capas 0,1,3,4 — voz + pantalla real + animación conceptual, sin datos reales del motor) | Pilares E, G | **Disponible desde HOY** (EPIC-0) |
| **Tier 3 — Gran Estreno** | Hito de EPIC con **datos reales** del motor (curvas de equity, Pareto fronts) | Completa (sistema de capas íntegro, Remotion data-driven) | Pilares A, B, C | Se desbloquea progresivamente — ver mapa §1.4 |

> Tier 1 NO compite con Tier 2/3: es el "ruido de fondo" constante que construye expectativa. El skill `social-strategist` ya está optimizado para esto — ver §10.

### 1.4 Mapa de Desbloqueo: EPIC → Pilar

| Épica (`docs/ROADMAP.md`) | Estado | Pilares que desbloquea | Qué se vuelve filmable |
|---|---|---|---|
| **EPIC-0 — Fundación y Spikes** | 🟢 En curso (STORY-001-005 ✅, STORY-006 próxima) | **G** (Building in Public), **E** (diario de decisiones) | Las 6 apuestas (ADR-0107/0112-0116), esqueleto del workspace, reloj determinista, audit-log con hash chain |
| **EPIC-1 — Soberanía de Datos** (`ingest`) | 🔵 Próxima | **D** completo (Zero-Docker/soberanía), parte de A | Pipeline "The Sanitizer" (datos sucios → limpios, visualmente potente), ingesta multi-fuente |
| **EPIC-2 — Motor de Backtest** (`validate` núcleo) | ⚪ Futura | **A** parcial (con datos reales) | Primera curva de equity real, motor dual Express vs Event-Driven en acción (paga la promesa de ADR-0114) |
| **EPIC-3 — Generación** (`generate`) | ⚪ Futura | **B**, **A** completo | NSGA-II descubriendo lógica, Pareto fronts reales — **aquí vive el Caso de Estudio del Fitness Compuesto (§8)** |
| **EPIC-4 — Guantelete de Robustez** (`validate` completo) | ⚪ Futura | **C** (La Sala de Tortura) | Robustez Decagonal, Monte Carlo tóxico con datos reales |
| **EPIC-5+ — Primer Dinero Real y más allá** | ⚪ Futura | **F** maduro, contenido "live" | Track record real, comunidad de fondeo |

### 1.5 Estrategia de Idioma: Español + Inglés

El público objetivo es cualquiera que hable español O inglés — no solo hispanohablantes. El creador puede grabar voz en ambos idiomas: no se depende de doblaje por IA como primera opción.

| Tier | Idioma | Cómo |
|---|---|---|
| **Pulso (Tier 1)** | **ES + EN desde el día 1** | El skill genera siempre el par `-es.md` / `-en.md` (§10). Costo marginal bajo: son posts cortos. |
| **Episodio (Tier 2)** | **Un idioma primero** — el que el creador sienta más natural para ese tema | No se duplica producción por defecto: protege el ritmo de publicación. |
| **Gran Estreno (Tier 3)** | **Ambos**, pero el segundo idioma solo si la pieza alcanzó **masa narrativa** (buen desempeño) | Se re-grava el voiceover en el segundo idioma (mismo creador). ElevenLabs queda como acelerador puntual, nunca como sustituto (§6.2, líneas rojas §14). |

**Regla simple:** lo corto siempre es bilingüe. Lo largo se vuelve bilingüe solo cuando ya demostró que vale la pena duplicar el esfuerzo.

---

## 2. Manifiesto de Marca: "Construimos, No Vendemos Humo"

**Posicionamiento de una frase:**
> *"Mientras otros te venden la señal, nosotros te mostramos el motor que la descubrió, la auditó y decidió si merece tu capital — y te explicamos exactamente cómo funciona, desde el primer commit."*

**Lo que SÍ somos:**
- Un laboratorio de ingeniería cuantitativa que documenta su propio proceso de construcción — **desde el día 1, no desde el día del lanzamiento**.
- La fuente hispana que explica *el cómo* (matemática, arquitectura, validación), no solo *el qué* (resultados, screenshots de ganancias).
- Honestos sobre incertidumbre, fallos de diseño y decisiones revertidas. Los ADRs de "erradicación" (ADR-0112, ADR-0113, ADR-0115) son oro narrativo: *"consideramos X, lo descartamos, así fue por qué"*.
- Constructores que usan IA para acelerar el desarrollo **sin sacrificar rigor** — y lo demuestran mostrando el proceso de auditoría (ADRs, Órdenes de Trabajo con criterios de aceptación, revisión Tech-Lead/QA — ver TASK-006).

**Lo que NO somos (líneas rojas):**
- No vendemos estrategias, señales, "robots" ni resultados de cuentas en vivo como gancho.
- No usamos avatares de IA ni voces sintéticas que pretendan ser una persona que no existe.
- No prometemos rentabilidad. Prometemos **rigor** y **transparencia metodológica**.
- No generamos B-roll de IA que simule funcionalidad de Drasus que no existe aún. Todo screenshot de producto es producto real — y si algo aún no existe, se dice explícitamente ("esto es el roadmap, no el producto hoy").

**El "villano" recurrente:** una *métrica engañosa* o un *sesgo cognitivo* (Profit Factor inflado, overfitting, curve-fitting). Fórmula repetible: **"La métrica/práctica que parece buena pero te va a quebrar — y cómo Drasus la neutraliza (o neutralizará) por diseño."**

---

## 3. El Problema De Fondo Y La Solución Visual

**Por qué "pizarra + Excel" no funciona:** una fórmula estática no tiene movimiento, comparación ni consecuencia visible. El cerebro engancha con (a) algo que cambia en tiempo real, (b) un "antes/después" o "ganador/perdedor", (c) un humano que reacciona/narra ese cambio.

**La fórmula: Capas, no Cara.** En lugar de "¿cara o animación?", la respuesta es **ambas, en capas independientes**, donde la cara es la capa más pequeña y más opcional — patrón QuantumFracture/LeMMiNo: 90% pantalla narrada + cápsulas cortas de presencia humana. Ver sistema completo en §4.

---

## 4. Identidad Visual — Sistema de Capas (The Layer Stack)

| Capa | Contenido | Obligatoria | Disponible desde |
|---|---|---|---|
| **0. Voz** | Narración propia (micrófono de condensador USB) | ✅ Sí | Ya |
| **1. Pantalla Real** | IDE con Rust real, `cargo test`/`cargo bench`, dashboard Flutter cuando exista | ✅ Sí | Ya (código de STORY-001 a 005) |
| **2. Animación Matemática/Algorítmica** | Fórmulas, conceptos (hash chains, determinismo), y más adelante Pareto fronts/distribuciones reales | ✅ Sí (temas "duros") | Ya, en modo conceptual (Manim) |
| **3. Motion Graphics de Marca** | Intros, lower-thirds, transiciones | ✅ Sí (consistencia) | Se hace una vez, ya |
| **4. Diagramas Estáticos** | Infografías de arquitectura (C4, pipeline de 8 módulos, AST) → SVG → animadas | ✅ Sí | Ya (el C4 de `SAD.md` es la fuente) |
| **5. Cara (PiP)** | Webcam en recuadro pequeño (15-20%) | ⚠️ Opcional, progresiva | Ver §4.1 |
| **6. B-roll Generativo IA** | Texturas abstractas, fondos de thumbnail | ⚠️ Opcional, quirúrgico | Ya, uso puntual |

### 4.1 Escalera de Exposición de Cara (sin presión, progresiva)

- **Nivel 0 — Faceless total:** Voz + Capas 1-4. **Recomendado para el video de lanzamiento (Caso de Estudio #0, §7).**
- **Nivel 1 — Intro/Outro grabado una vez:** Una sola grabación de marca, reusada siempre.
- **Nivel 2 — PiP en "momentos de postura":** Recuadro pequeño en segmentos de opinión/crítica (10-20s). El sweet spot que pedía `content-creator.md` (opción 2).
- **Nivel 3 — Futuro (Discord/comunidad):** Vlogs más personales, reservados a audiencia ya leal.

**Recomendación:** arrancar en Nivel 0-1; introducir Nivel 2 a partir del video 4-5.

---

## 5. Pilares de Contenido

Cada pilar mapea a material que **ya existe** en `docs/SAD.md` / `docs/ADR.md` / `docs/execution/`.

### Pilar A — "La Métrica Mentirosa" (Anti-Hype Educativo)
- **Tesis:** Una métrica popular (Profit Factor, Win Rate) parece indicar calidad pero esconde fragilidad.
- **Desbloquea en:** EPIC-2/3 (necesita datos reales de backtest).
- **Ejemplo:** *"Por qué optimizar el Profit Factor es la forma más rápida de quebrar tu cuenta (con matemáticas)"* — Caso de Estudio Futuro, §8.

### Pilar B — "Cómo Piensa una Máquina que Descubre Alpha"
- **Tesis:** El Hybrid Genesis Engine (NSGA-II + regresión simbólica sobre AST) descubre lógica sin hipótesis humana.
- **Desbloquea en:** EPIC-3.
- **Ejemplo:** *"Hicimos que un algoritmo genético escriba código de trading sin que nosotros le digamos cómo"*

### Pilar C — "La Sala de Tortura" (Robustez y Validación)
- **Tesis:** Antes de arriesgar un euro, una estrategia pasa por un "guantelete": WFA, Monte Carlo tóxico, CPCV, Robustez Decagonal.
- **Desbloquea en:** EPIC-4.
- **Ejemplo:** *"Sometimos una estrategia rentable a 10 escenarios de crisis. Esto es lo que pasó."*

### Pilar D — "Infraestructura Soberana" (Zero-Docker / Local-First)
- **Tesis:** Por qué NO depender de la nube, Docker, ni APIs de terceros para tradear tu propio capital.
- **Desbloquea en:** **EPIC-0 (parcial, ya hay SQLite WAL real)**, completo en EPIC-1.
- **Ejemplo:** *"Por qué nuestro motor de trading no necesita internet para decidir"*

### Pilar E — "Devlog: Decisiones que Cambiamos" (Behind the Build)
- **Tesis:** Documentar errores y pivotes de arquitectura como aprendizaje público.
- **Desbloquea en:** **EPIC-0, disponible HOY** (ADR-0112, ADR-0113, ADR-0115).
- **Ejemplo:** *"Eliminamos PyTorch de nuestro motor de IA. Así fue (y por qué no es lo que piensas)"*

### Pilar F — "Mitos del Trading Algorítmico Hispano" (Confrontación directa, alto potencial viral)
- **Tesis:** Atacar de frente las afirmaciones de vendedores de cursos/robots con argumentos técnicos verificables.
- **Desbloquea en:** Desde HOY (no depende del estado de Drasus — es contenido de "postura").
- **Formato:** Shorts/Reels/TikTok, 30-60s.
- **Ejemplo:** *"'Mi backtest tiene 95% de winrate' — esto es lo que NO te dicen"*

### 🆕 Pilar G — "Building in Public: La Bitácora de la Fundación"
- **Tesis:** El proceso mismo de construir un motor cuantitativo institucional, **desde absolutamente cero, con rigor documentado y asistido por IA**, es una historia con arco propio — y es el pilar **insignia mientras EPIC-0/1 están en curso**.
- **Desbloquea en:** **HOY.**
- **Estructura tipo "temporada":** cada EPIC es un arco; cada SPIKE/ADR/STORY resuelto es un episodio. El espectador que entra en el Episodio 0 ve "pagarse" las decisiones tempranas cuando EPIC-2/3 entreguen resultados.
- **Ejemplos:**
  - *"Construimos un motor de trading. Cero líneas de estrategia. Esto es lo que SÍ hicimos (y por qué)"* — **Caso de Estudio #0, §7**.
  - *"Le dimos a nuestro motor de trading un libro de contabilidad que ni nosotros podemos alterar"* (STORY-004).
  - *"Auditamos 137 piezas de nuestro propio diseño con IA, antes de escribir código. Así fue el proceso"* (TASK-006).

### 🆕 Pilar H — "La Ciencia Detrás del Motor" (Educación Conceptual)
- **Tesis:** Cada feature de Drasus implementa un concepto con historia, creador y propósito. Enseñar el contexto —no la fórmula completa— es la forma más accesible de construir autoridad técnica real.
- **Desbloquea en:** **SIEMPRE.** Contexto histórico y conceptual, sin dependencia de datos reales ni de ninguna épica concreta.
- **Fuentes:** `docs/features/*.md`, `docs/adr/*.md`, TTRs — cualquier entidad que implemente un algoritmo, método estadístico o patrón de ingeniería.
- **Formato de Cápsula (4 bloques fijos, en este orden):**
  1. **Contexto:** "En [año], [creador/institución] enfrentaba el problema de [X]..." — el origen, el momento, la pregunta que nadie había resuelto bien.
  2. **La idea:** El concepto central explicado sin cálculos completos — suficiente para entender *qué* resuelve y *por qué* es elegante.
  3. **Por qué importa en quant/trading:** Una aplicación concreta al mundo de estrategias, backtesting o ejecución — algo que el espectador reconoce o padece.
  4. **En Drasus:** "Lo implementamos en [feature/módulo] para [propósito específico]." — el puente entre el concepto universal y la decisión de diseño real.
- **Taxonomía de candidatos (para detectar automáticamente en `docs/features/`):**
  - *Algoritmos:* NSGA-II, Monte Carlo, algoritmos genéticos, programación simbólica, CPCV, WFA, Simulated Annealing...
  - *Métodos estadísticos:* Sharpe/Sortino/Calmar, drawdown, Pareto fronts, bootstrapping, regresión simbólica, distribuciones de rendimiento...
  - *Patrones de ingeniería:* append-only, determinismo, WAL (Write-Ahead Log), hash chains, colas idempotentes, reloj lógico...
- **Granularidad adaptable:** un concepto simple (ej. WAL) → thread de Twitter de 4 tweets. Un algoritmo rico (ej. NSGA-II, Monte Carlo) → Short/Reel 60s con animación Manim + thread de soporte.
- **Sinergias clave:**
  - Alimenta **Pilar F:** un mito se desmonta con la cápsula del concepto correcto (la educación ES el argumento).
  - Alimenta **Tier 2:** cuando 3+ cápsulas relacionadas acumulan tracción, el Episodio profundiza con código y datos reales.
  - Complementa **Pilar G:** el "building in public" gana densidad cuando se explica *por qué* se eligió un concepto, no solo que se eligió.
- **Ejemplos:**
  - *"NSGA-II: el algoritmo que no busca 'la mejor estrategia' — busca el frente que no puedes mejorar sin sacrificar algo"* — algoritmo, creadores (Deb et al., 2002), historia del Problema Multi-Objetivo, uso en Drasus para descubrir estrategias sin sesgar el fitness.
  - *"Monte Carlo no es suerte: es tortura estadística sistemática"* — método, origen en Los Álamos (1940s), por qué un backtest sin él miente, cómo Drasus lo usa en el Guantelete de Robustez.
  - *"Append-only: la decisión de diseño que hace que Drasus no pueda mentir sobre su historial"* — patrón de ingeniería, STORY-004, por qué los sistemas que permiten borrar no son auditables.
  - *"WAL: cómo SQLite sobrevive a un apagón inesperado sin perder datos"* — decisión de infraestructura de STORY-002, aplicación en integridad de datos del motor.

> **Distribución de esfuerzo (fase EPIC-0/1):** Pilar **G** = contenido insignia actual (Tier 2, cadencia ~cada 2-4 semanas según ritmo de STORYs). Pilar **H** = combustible constante, disponible ya — una cápsula por feature/ADR relevante, independiente del calendario de épicas. Pilar **E** = puente (se funde con G en esta fase). Pilar **D** = arranca en paralelo apenas haya material de EPIC-1. Pilar **F** = confrontación directa para Shorts; se potencia con las cápsulas de Pilar H como argumento técnico. Pilares **A/B/C** = en backlog, se activan según §1.4.

---

## 6. Stack de Herramientas — Pipeline de Producción

### 6.1 Principio rector
Priorizamos herramientas **code-first** que Claude Code puede generar/automatizar vía Bash. Herramientas de escritorio (Adobe, Canva) = pulido manual opcional, no bloqueante.

### 6.2 Tabla de Herramientas

| Función | Herramienta | Estado | Cómo se opera |
|---|---|---|---|
| Guion y research | Claude (este entorno) | ✅ Confirmado | Traduce ADRs/STORYs a guion narrativo |
| Animación matemática/conceptual | **Manim** (Python) | ✅ Confirmado, open source | `manim render scene.py`. Ideal para: fórmulas, hash chains, Pareto fronts (cuando haya datos) |
| Animación de diagramas/explainers | **Motion Canvas** (TypeScript) | ✅ Confirmado, open source | Alternativa para diagramas de arquitectura/AST |
| Ensamblaje programático data-driven | **Remotion** (React/TS) | ✅ Confirmado, open source | Lee JSON/CSV exportado de Drasus y renderiza video con datos reales — **clave para Tier 3** |
| Captura de pantalla (IDE + Drasus UI) | **OBS Studio** | ✅ Confirmado, gratis | Grabación de Rust IDE y `cargo test` |
| Procesamiento/concatenación | **ffmpeg** | ✅ Confirmado | Comandos vía Bash |
| Transcripción/subtítulos | **Whisper** (local) | ✅ Confirmado, open source | Subtítulos automáticos |
| Edición final | **DaVinci Resolve** (free) | ✅ Confirmado, gratis | Ensambla todas las capas |
| Snippets de código bonitos | **Silicon** / **Carbon** | ✅ Confirmado | Código Rust real para redes estáticas |
| Diagramas vectoriales | **Illustrator** o **Figma** | ✅ Existen | Figma tiene MCP oficial (Dev Mode MCP Server) |
| Branding/intros reutilizables | **After Effects** | ✅ Existe (pago) | Manual; se hace una vez. Sin MCP oficial confirmado |
| Thumbnails/carruseles IG | **Canva** | ✅ Existe (freemium) | Manual; sin MCP estable confirmado |
| Voz — segundo idioma (respaldo) | **ElevenLabs** | ✅ Existe (pago) | El creador habla ES y EN — re-grabación nativa es la opción por defecto (§1.5). ElevenLabs solo acelera piezas puntuales, nunca sustituye la voz real. |
| B-roll abstracto | **Midjourney / SD / DALL·E / Runway** | ✅ Existen | Solo texturas — nunca simular producto |
| 3D avanzado | **Blender** + MCP comunitario | ⚠️ Verificar mantenimiento | No crítico para MVP |

### 6.3 Sobre los MCPs — Qué confío y qué no
- **Confío:** Figma Dev Mode MCP (oficial); y que Claude Code opera Manim/Remotion/ffmpeg/Whisper vía Bash sin MCP — son CLIs estándar.
- **Verificar antes de depender:** MCP de Blender (comunidad), cualquier "MCP de Canva/After Effects/Premiere" — no confirmado y mantenido a la fecha. No construir el camino crítico sobre estos.

---

## 7. 🆕 Caso de Estudio #0: El Video de Lanzamiento (Tier 2, disponible HOY)

> **Este es el primer video del canal.** Pilar G, Nivel 0 de exposición (sin cara), producción media — sin necesidad de datos reales del motor.

**Título de trabajo:** *"Construimos un motor de trading institucional. Cero líneas de estrategia todavía. Esto es lo que SÍ hicimos (y por qué)"*

### 7.1 Guion estructurado (≈9-10 min)

| Tiempo | Capa(s) | Contenido |
|---|---|---|
| 0:00–0:20 | Voz + Animación (gancho) | *"Antes de escribir una sola estrategia, tomamos 6 decisiones que podían hundir este proyecto. Las documentamos todas. Esto es 'construir en público', versión quant."* |
| 0:20–2:00 | Voz + Diagrama animado (Capa 4) | El manifiesto: rigor antes que velocidad. Mostrar el pipeline de 8 módulos (`ingest → ... → withdraw`, C4 de `SAD.md`) animándose pieza por pieza. |
| 2:00–5:30 | Voz + Manim (conceptual) | 3 de las 6 apuestas (SPIKEs) más narrables: **"dijimos no a PyTorch"** (ADR-0112), **"dijimos no a Ollama"** (ADR-0115), **"dijimos no a Docker/microservicios"** (Zero-Docker). Cada una: problema → decisión → por qué, con animación simple del "árbol de opciones descartadas". |
| 5:30–8:00 | Pantalla real + Voz | `cargo test` en verde sobre el workspace de 8 crates (STORY-001). Zoom a dos piezas concretas: el **reloj determinista** (STORY-003) y el **audit-log encadenado por hash** (STORY-004) — animación tipo "cadena de bloques" (Manim) explicando por qué un motor de trading necesita su propio libro contable inmutable. |
| 8:00–9:00 | Voz + Diagrama | Cierre: "Esto es el Episodio 0. Cada decisión que el motor tome de aquí en adelante será tan auditable como las que acabamos de tomar para construirlo." |
| 9:00–9:30 | PiP cara (opcional, Nivel 2) + Voz | CTA a Discord: ahí se publica el "detrás de cámaras" de TASK-006 (auditoría de 137 features con IA). |

### 7.2 Assets a generar

1. **Manim — "árbol de decisiones descartadas"** para cada SPIKE (esqueleto simple: nodo central, ramas que se desvanecen en gris excepto la elegida que se ilumina).
2. **Manim — animación de hash chain** (bloques A→B→C, cada uno con hash visible, intento de alterar B se detecta porque rompe la cadena).
3. **Diagrama C4 animado** (Capa 4) a partir del ASCII de `SAD.md` §4 — redibujado en Figma/Illustrator, exportado SVG, animado en Motion Canvas/Remotion.
4. **Pantalla real:** grabación OBS de `cargo test --workspace` en verde + recorrido breve por la estructura de `crates/`.
5. **PiP cara (opcional):** 2 tomas cortas (gancho + CTA), mismas que el intro de marca (Nivel 1).

### 7.3 Reutilización cross-tier
- Cada uno de los 6 "árboles de decisión" → recortado como **Short independiente** (Pilar G/F): *"Eliminamos PyTorch de nuestro motor de IA — antes de tener un motor de IA"*.
- La animación del hash chain → carrusel de Instagram (Pilar G).
- El guion completo → post de Discord (Tier 1, vía `/social-strategist`) con los 6 ADRs linkeados.
- `git log` de las 5 STORYs → hilo de Twitter "5 commits, 5 decisiones" (Tier 1).

---

## 8. Caso de Estudio Futuro: "Por qué optimizar el Profit Factor te va a quebrar" (Tier 3 — EPIC-3)

> Este caso de estudio (Pilar A, el que originó esta estrategia) **requiere datos reales de NSGA-II/backtest, que no existen hasta EPIC-3**. Se mantiene documentado aquí como objetivo de producción Tier 3.

### 8.1 Guion estructurado (≈11 min, formato long-form)

| Tiempo | Capa(s) | Contenido |
|---|---|---|
| 0:00–0:15 | Voz + Animación (gancho) | *"Si tu robot de trading busca el Profit Factor más alto, te voy a mostrar exactamente el momento en que eso te va a quebrar."* |
| 0:15–1:30 | Voz + Manim | Qué es el Profit Factor y por qué PF=8 con 5-8 trades es indistinguible del azar. |
| 1:30–3:30 | Pantalla real (Drasus UI) + Voz | El motor genético explorando variantes; si fitness = solo PF, converge hacia "francotiradores". |
| 3:30–6:00 | Manim (fórmula) | Construcción del **fitness compuesto ponderado**: `Fitness = w1·Sharpe + w2·R²(Davey) + w3·f(N_trades)`. Radar chart comparando "PF=8, 5 trades" vs "PF=1.8, 340 trades, Sharpe=1.4, R²=0.85". |
| 6:00–9:00 | Remotion (datos reales) | Pareto front real exportado de un backtest de Drasus (Sharpe vs Drawdown, coloreado por R²). |
| 9:00–10:30 | Pantalla real (código Rust) + Voz | Snippet real de Rust con la función de fitness compuesto en NSGA-II. |
| 10:30–11:30 | PiP cara (opcional) + Voz | Cierre de postura + CTA a Discord. |

### 8.2 Asset clave (Remotion, data-driven)
```tsx
// ParetoFront.tsx
import {useCurrentFrame, interpolate} from 'remotion';
import data from './backtest_export.json'; // exportado real desde Drasus (Parquet -> JSON)

export const ParetoFront: React.FC = () => {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 30], [0, 1]);
  return <ScatterPlot points={data.candidates} highlight={['naive_pf', 'robust_composite']} opacity={opacity} />;
};
```

**Pre-requisito técnico:** un script de exportación Parquet→JSON desde el catálogo de Drasus (ver §15, punto 3).

---

## 9. Formato y Adaptación por Red Social

| Red | Formato (Tier 2/3) | Formato (Tier 1, vía social-strategist) | Frecuencia | Idioma |
|---|---|---|---|---|
| **YouTube (long-form)** | Pieza completa (9-15 min) | — | Según disponibilidad de Tier 2/3 (no calendario fijo en EPIC-0) | ES o EN según §1.5; el otro si hay masa narrativa |
| **YouTube Shorts/TikTok/Reels** | Recortes de gancho (Pilar G/F) | — | 3-4/semana | Idioma del Episodio de origen |
| **Twitter/X** | Hilo con capturas + snippets | 3 variantes por STORY/SPIKE cerrado | 3/semana (Tier 1 cubre las semanas sin Tier 2/3) | ES + EN (hilos duplicados) |
| **Instagram** | Carrusel del diagrama/comparación | Visual + caption corta | 1-2/semana | ES + EN |
| **Discord** | Versión "uncut" + código + razonamiento | Premium, 800-1200 palabras, 12-24h antes que redes | 1/semana — **siempre activo desde EPIC-0** | ES + EN (canales o hilos separados) |
| **LinkedIn (futuro)** | Pilar D en tono profesional | — | Se activa en EPIC-1+ | EN primero (audiencia institucional) |

---

## 10. Integración con el Skill `social-strategist` (Motor de Ejecución)

El skill (`.claude/skills/social-strategist/SKILL.md`) es el **punto de entrada único** para aplicar esta estrategia. No hace falta decidir manualmente "¿toca Pulso o Episodio?" — el skill:

1. **Escanea el estado:** `git log`, `docs/execution/` (Historias/Riesgos/Tareas cerradas) y su propio archivo de seguimiento `.agents/state/social-strategist/PROGRESS.md`.
2. **Calcula masa narrativa** con el mapa Épica → Pilar (§1.4): si 2+ entradas de Pulso sin "graduar" caen en el mismo Pilar ya desbloqueado, propone subir a Episodio.
3. **Presenta un menú de 2-4 opciones** (generar Pulso pendiente, producir Episodio, configurar herramientas de producción, ver estado) — eliges un número y el skill ejecuta el pipeline completo, sin más preguntas salvo acciones irreversibles (ej. instalar paquetes del sistema).
4. **Genera bilingüe (ES+EN) por defecto** todo el contenido de Pulso (§1.5). Para Episodio/Gran Estreno sigue la regla de §1.5: un idioma primero, el segundo si hay masa narrativa.
5. **Regla de no duplicación:** un STORY/SPIKE que ya recibió tratamiento de Episodio/Gran Estreno (ej. ADR-0112/0113/0115 en el Caso de Estudio #0) se marca como "cubierto" en `PROGRESS.md` — el Pulso correspondiente solo referencia el video, no repite el ángulo.

> El detalle operativo (menú exacto, pipelines, plantillas de archivos, verificación de herramientas) vive en `SKILL.md`. Esta sección es el contrato: la estrategia define **qué y cuándo**, el skill define **cómo**.

---

## 11. Flujo de Trabajo End-to-End

```
0. ESCANEO (nuevo)
   └─ Tras cada cierre de STORY/SPIKE/TASK: /social-strategist genera Tier 1 automáticamente
   └─ Claude evalúa: ¿esta entrada + el backlog reciente alcanzan masa narrativa para Tier 2/3?

1. SELECCIÓN DE TEMA
   └─ Si hay masa narrativa → Claude propone ángulo (Pilar A-G) según mapa de desbloqueo (§1.4)

2. GUION
   └─ Claude redacta guion minuto a minuto (formato §7.1 / §8.1)
   └─ Usuario ajusta tono/anécdotas personales

3. GENERACIÓN DE ASSETS (paralelo)
   ├─ Manim → animaciones conceptuales o data-driven
   ├─ Remotion → render data-driven (solo Tier 3, requiere export Parquet→JSON)
   ├─ OBS → grabación de pantalla (IDE + tests + UI cuando exista)
   ├─ Illustrator/Figma → diagramas estáticos
   └─ Grabación de voz (+ PiP cara si Nivel 2)

4. ENSAMBLAJE
   └─ DaVinci Resolve: une capas + subtítulos (Whisper) + música

5. DERIVADOS
   ├─ Recortes para Shorts/Reels (ffmpeg)
   ├─ Snippets de código (silicon/carbon)
   ├─ Frames estáticos para IG/Twitter
   └─ Posts adaptados por red (social-strategist, marcando como "ya cubierto en video")

6. PUBLICACIÓN ESCALONADA
   └─ Discord (T-24h) → YouTube → Twitter/IG → Shorts/TikTok

7. TRACKING
   └─ .agents/documents/social-strategist/ (Tier 1) + nota de qué Tier 2/3 cubrió qué STORY/ADR
```

---

## 12. Roadmap de Implementación (re-pacing por EPIC, no por semanas fijas)

### Fase 0 — Ahora (durante EPIC-0)
- Activar Tier 1 inmediatamente sobre STORY-001 a 006 (el skill ya está listo).
- Setup mínimo: Manim, OBS, DaVinci Resolve, plantilla de marca (Capa 3, una vez).
- Producir **Caso de Estudio #0 (§7)** — Nivel 0, sin cara, sin datos reales requeridos.

### Fase 1 — Durante EPIC-1 (Soberanía de Datos)
- Tier 2 sobre el pipeline "The Sanitizer" (datos sucios → limpios es muy visual incluso sin Manim avanzado).
- Introducir Nivel 1 (intro grabado).
- Medir retención de Caso de Estudio #0 antes de comprometer más producción Tier 2.

### Fase 2 — Durante EPIC-2 (Motor de Backtest)
- Montar el script de exportación Parquet→JSON (pre-requisito de Tier 3, §8.2).
- Primer contenido Tier 3 "lite": primera curva de equity real, motor dual Express vs Event-Driven (paga ADR-0114).

### Fase 3 — Durante EPIC-3 (Generación)
- **Caso de Estudio del Fitness Compuesto (§8)** se vuelve producible con datos reales de NSGA-II.
- Introducir Nivel 2 (PiP) si el ritmo lo permite.
- Evaluar doblaje EN (ElevenLabs) para el contenido tentpole acumulado.

### Fase 4 — EPIC-4+
- Pilar C (Sala de Tortura) y Pilar F maduran con casos reales.
- Evaluar LinkedIn (Pilar D institucional) según tracción.

---

## 13. Métricas de Éxito y Señales de Pivote

| Señal | Significado | Acción |
|---|---|---|
| Caso de Estudio #0 retiene bien pero no convierte a Discord | El formato "building in public" funciona, falta CTA más claro | Reforzar CTA: "el roadmap completo de los próximos episodios está en Discord" |
| Shorts de Pilar F/G con alta retención pero bajo CTR a long-form | El gancho funciona, audiencia no busca profundidad aún | Mantener shorts como embudo, no forzar conversión |
| Discord crece más rápido que YouTube | La comunidad valora la profundidad "uncut" del Tier 1/2 | Priorizar Pilar G/E para Discord |
| Comentarios pidiendo "dónde compro la estrategia" | El mensaje anti-hype no calando | Reforzar framing: "el motor, no la señal" |
| Engagement alto con PiP (Nivel 2) vs sin él | Audiencia valora presencia humana | Subir gradualmente % de Nivel 2 |

---

## 14. Riesgos y Líneas Rojas

- **No** simplificar al punto de decir algo técnicamente falso "por claridad" — si se simplifica, se declara.
- **No** usar B-roll generativo para simular funcionalidad de Drasus que no existe aún.
- **No** prometer timelines de producto en redes que no estén confirmados internamente (ver `docs/ROADMAP.md` / memoria de auditoría TASK-006).
- **No** depender de herramientas/MCPs no verificados para el camino crítico de producción (§6.3).
- **🆕 No** presentar contenido de Pilares A/B/C (datos/resultados) antes de que el EPIC correspondiente los haya producido realmente (§1.4) — el "Episodio 0" promete auditabilidad; incumplirla rompe la marca.

---

## 15. Próximos Pasos Inmediatos

1. Activar Tier 1: correr `/social-strategist` sobre las 5 STORYs cerradas + TASK-006 para empezar la bitácora pública de inmediato (bajo costo, alto valor de "track record").
2. Decidir nombre/identidad mínima del canal.
3. Producir **Caso de Estudio #0 (§7)** como video de lanzamiento — Nivel 0, sin cara, sin pre-requisitos técnicos de exportación de datos.
4. En paralelo (no bloqueante), diseñar el script de exportación Parquet→JSON para cuando EPIC-2/3 entreguen datos reales (Tier 3) — a decidir con el Arquitecto si vive en `crates/shared` o en un repo de contenido separado.
5. Grabar el piloto, medir, iterar.
