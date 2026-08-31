# Plan — Abrir el puerto de búsqueda, desbifurcar la narrativa y completar el mapeo 1:1

## Contexto

Tres frentes encadenados, descubiertos en este orden.

**Frente A — el puerto de búsqueda está encasillado en la doctrina.** El propietario preguntó si la documentación permite métodos de descubrimiento no evolutivos —fuerza bruta, grid search, y cualquier método relevante del mundo académico o de otras industrias— o si nos cataloga como "descubrimiento evolutivo". Verificado en el corpus: **ningún ADR lo prohíbe, pero la doctrina lo estrecha**, y hay un patrón activo de subordinación.

**Frente B — la narrativa simple se bifurcó por profesión.** `docs/PRODUCT-SIMPLE.md` §4 tenía cuatro versiones del pitch, una por profesión del interlocutor. Cada persona nueva que el propietario conozca sería una edición pendiente: mantenimiento sin techo.

**Frente C — el nivel bajo del mapeo 1:1 está vacío.** 0 de 166 features tienen su sección `## Narrativa de Producto`.

---

# Parte A — Abrir el puerto de búsqueda

## A.0 El diagnóstico, con evidencia

**Lo que se verificó, no lo que se supone:**

- **Ningún ADR prohíbe métodos no evolutivos.** El corpus ya documenta bayesiano (8 archivos), topológico (7), aprendizaje por refuerzo (4), regresión simbólica (4), fuerza bruta (3) y grid search (2).
- **La arquitectura ya los permite.** Por ADR-0137 cada feature es un hexágono con puertos tipados, y `nsga2-optimizer`, `bayesian-optimizer`, `parameter-optimization` e `hierarchical-parameter-optimization` **ya son features separadas y pares**. No hay refactorización estructural pendiente.
- **Cero impacto en código.** `crates/features/generation/` existe pero NSGA-II aún no está implementado. Corrección puramente documental.

**Las tres causas del encasillamiento, en orden de gravedad:**

1. **ADR-0113 justifica el alcance apelando a un competidor.** Su texto dice: *"la regla maestra del MVP ('copia descarada de SQX') lo confirma — SQX no hace regresión simbólica libre"*. Usar el catálogo de funciones de StrategyQuant X como criterio de qué NO construir le pone techo a una infraestructura de investigación. **Este es el defecto de fondo; el resto son síntomas.**

2. **La doctrina fusiona dos ejes que deben ser independientes.** ADR-0043/0108/0109/0110/0111 describen *qué* se busca (el AST, los dominios genómicos de señal, riesgo, régimen y portafolio) con lenguaje que asume *cómo* se busca: *"genes que **el motor evolutivo** combina y parametriza"*. Fusionados, todo método nuevo llega obligado a entrar como modo del genético.

3. **Existe un precedente activo de subordinación.** ADR-0113 absorbió la regresión simbólica como *"modo del motor NSGA-II"*, no como motor par. Para la regresión simbólica el argumento técnico es correcto —programación genética sobre árboles de expresión con frente de Pareto precisión/complejidad **es** una variante genética—, pero instaló el reflejo: reconocer cada método nuevo como modo del genético en lugar de como igual.

## A.1 La corrección: separar los dos ejes

| Eje | Qué define | Dónde vive |
|---|---|---|
| **Qué se busca** (representación) | El AST de reglas condición→acción y sus dominios genómicos: señal, riesgo, régimen, portafolio | ADR-0043, ADR-0108 a ADR-0111 — se conservan, se les quita el lenguaje que presupone el método |
| **Cómo se busca** (estrategia de búsqueda) | El algoritmo que explora ese espacio: evolutivo, bayesiano, exhaustivo, topológico, por recocido, por enjambre, y lo que venga | **Puerto abierto** — implementaciones pares, ninguna subordinada |

El eje "cómo" pasa a ser un **puerto con un contrato único**. Toda implementación entra como par. Ninguna es el motor por defecto del que las demás son modos.

## A.2 Sin ADR nuevo — se corrige el que está mal

**Decisión del propietario, y coincide con mi propio protocolo.** El Protocolo de Mantenimiento de ADRs de `.agents/skills/architect/SKILL.md` ya manda: *"NO se crean ADRs nuevos para decisiones que ya existen. Si la decisión ya está tomada en un ADR previo, actualiza ese ADR, no crees uno duplicado."* Proponer un ADR-0163 habría violado mi propia regla.

**ADR-0113 se reescribe para que lea como debió leer siempre**, no como un documento enmendado:

| Qué se conserva | Qué se corrige |
|---|---|
| La erradicación de PySR y de Python (es su decisión real y es correcta) | El título: deja de anunciar "Regresión Simbólica como Modo del Motor Genético" |
| El invariante de AST tipado — prohibido evaluar expresiones por texto en la ruta caliente | La justificación por paridad con StrategyQuant X: **se elimina**. El criterio pasa a ser valor de investigación y encaje en el puerto |
| El rechazo de `evalexpr`/`meval` en rutas de minería | La regresión simbólica pasa de "modo del genético" a **implementación par del puerto de búsqueda**, con la nota técnica de que comparte maquinaria con el evolutivo |
| La higiene de dependencias (crates de GA de catálogo no se adoptan) | Se añade el **contrato del puerto** y el **gate de admisión** de A.3 |

Los dos banners de enmienda que ADR-0113 ya carga (por ADR-0159 y ADR-0161) se conservan: son historia real de decisiones posteriores, no la deriva que se está corrigiendo.

## A.3 El gate de admisión

Un método entra al **catálogo** con tres condiciones; entra a la **implementación** con una cuarta.

**Para catalogarse (barato, y aquí la asimetría manda: ante duda genuina, incluir):**
1. Respaldo académico revisado por pares, **de cualquier disciplina** — no hace falta que se use en finanzas; venir de fuera es lo que lo hace valioso.
2. Caso de uso concreto declarado en Drasus: cuál de los cuatro problemas ataca (estructura de estrategia, parámetros, riesgo y capital, asignación entre estrategias).
3. Encaje en el contrato del puerto.

**Para implementarse:** además, un hueco real que los métodos ya construidos no cubran.

**Por qué la separación importa.** Drasus es infraestructura de investigación: el costo de catalogar un método y no construirlo es casi cero; el de no enterarnos de que existe es perder una oportunidad. El repositorio ya tiene la maquinaria exacta para esto — `docs/moonshots/` con 41 ideas documentadas y no comprometidas, y EPIC-9+ *"según ROI demostrado"*. El catálogo es ancho; la construcción es disciplinada. Mismo estándar con el que se decidió **no** admitir Kagi y Point&Figure al catálogo de barras: documentados, no construidos, reabribles.

## A.4 El catálogo — resultado del barrido

Barrido ejecutado (subagente Sonnet, solo lectura, referencias verificadas por búsqueda web). **35 métodos** de ingeniería de diseño, biología computacional, diseño de fármacos, logística, aeroespacial, AutoML, física computacional, métodos formales e investigación de operaciones. Cada uno con referencia canónica, clase de problema, costo relativo al genético y la razón honesta por la que podría **no** servir.

**Los cinco de mayor valor esperado**, que son los que justifican esta corrección de doctrina:

| # | Método | Por qué importa | Referencia |
|---|---|---|---|
| 23 | **Hyperband / multi-fidelidad** | Riesgo de implementación casi nulo: es un envoltorio de presupuesto compatible con cualquier motor. Drasus **ya tiene** la fidelidad natural (ventana corta vs. backtest completo) y ya tiene la cascada LIGHT/MEDIUM/HEAVY de ADR-0066 — Hyperband es su versión con fundamento estadístico | Li et al. 2018, JMLR 18 |
| 17 | **CMA-ES** | Estándar de oro en optimización continua de caja negra de baja-media dimensión. **Drasus hoy no tiene motor dedicado de parámetros continuos** — lo deja al genérico. Corrección más barata y de mayor impacto de la lista | Hansen & Ostermeier 2001 |
| 34 | **Valor de Shapley** | Ataca el problema #4 tal como está formulado: reparto de capital entre estrategias cuya contribución depende de con quién más estén. Computable por muestreo de permutaciones **reutilizando el Monte Carlo que ya existe**. Complementa a HRP, no compite | Shapley 1953 |
| 18/20 | **SMAC y TuRBO** | No son motores nuevos: son la generación siguiente del bayesiano que la doctrina ya acepta, y cubren sus dos debilidades conocidas — parámetros condicionales/categóricos (SMAC) y alta dimensión (TuRBO) | Hutter et al. 2011; Eriksson et al. 2019 |
| 27 | **Parada óptima** | Convierte "descubrir un stop" de búsqueda empírica a problema con teoría cerrada. Valor no solo de rendimiento: una regla de riesgo derivada analíticamente es **auditable y defendible**, cosa que una encontrada por caja negra no es | Bellman 1957; Chow, Robbins & Siegmund 1971 |

**El hallazgo meta, que cambia el diseño del puerto (#35, MDO — Sobieszczanski-Sobieski & Haftka 1997):** los cuatro problemas de Drasus no son independientes sino **subsistemas acoplados**. Una mala gestión de riesgo cambia qué estructura es óptima, y eso cambia cuánto capital merece la estrategia. La ingeniería aeroespacial resolvió esto hace décadas con arquitecturas de coordinación. **Consecuencia para ADR-0113:** el puerto debe declarar no solo *qué métodos entran*, sino *cómo se coordinan entre sí* — secuencial o iterativo-acoplado. Sin eso, abrir el puerto solo cambia el catálogo, no la capacidad.

**El hallazgo que cuestiona la doctrina actual (#14, NEAT — Stanley & Miikkulainen 2002):** coevoluciona estructura y parámetros a la vez, en lugar de tratarlos como dos fases. Drasus hoy los separa. Esa separación puede ser artificial, y merece quedar como pregunta abierta declarada en el ADR, no como invariante silencioso.

**Cinco apuestas asimétricas** quedan catalogadas con su prueba de verificación previa, no implementadas: recocido cuántico/QUBO (#16), aprendizaje de bibliotecas por síntesis de programas (#12), optimización topológica SIMP aplicada a capital (#31), autoencoders sobre gramática de ASTs (#13) y programación por conjuntos de respuesta como verificador de riesgo acotado (#11). Cada una con qué habría que medir **antes** de invertir en ella.

**Cinco descartados con razón escrita**, para que nadie los reproponga sin argumento nuevo: NAS diferenciable/DARTS (un AST booleano no tiene relleno continuo que relajar), ASP como buscador global (no escala), Nelder-Mead (superado por CMA-ES y RSM), control predictivo para stops (su valor decae con la fidelidad del modelo de precios, que es el eslabón más débil) y Branch-and-Price como motor principal (exponencial en el peor caso).

El catálogo completo de los 35 —incluidos los 30 no listados arriba— entra al ADR-0113 reescrito como tabla de catálogo, y los que ameriten construcción futura reciben su entrada en `docs/moonshots/`. **Ninguno se implementa en esta pasada.**

## A.5 Superficie documental a corregir

Verificado por grep. El eje "cómo se busca" aparece encasillado en:

- **`docs/adr/ADR-0113.md`** — la reescritura central de A.2
- **`docs/adr/ADR-0031.md`** — "Hybrid Genesis Engine" enumera tres motores como si fueran la lista; pasa a nombrarlos como las primeras implementaciones del puerto
- **`docs/modules/generate.md`** — se describe como "el núcleo del Hybrid Genesis Engine… orquesta los motores evolutivos, simbólicos y DRL"; pasa a orquestar el puerto de búsqueda
- **`docs/features/nsga2-optimizer.md`** — se presenta como "el motor de fine-tuning" subordinado al DRL; pasa a par
- **`docs/adr/ADR-0043.md`, `ADR-0108.md` a `ADR-0111.md`** — se les quita el lenguaje que presupone el método (*"el motor evolutivo combina"* → *"el motor de búsqueda combina"*). **Su contenido de dominio no se toca**: los cuatro dominios genómicos y su gramática condición→acción siguen íntegros
- **`docs/adr/ADR-0057.md`** — mención residual
- **`docs/sad/SAD-01.md`, `SAD-02.md`, `SAD-07.md`, `SAD-08.md`** — menciones al Hybrid Genesis
- **`docs/README.md` y `docs/ADR.md`** — el título de ADR-0113 en ambos índices

**Fuera de mi gate, se reporta y no se toca:** `CONTENT-STRATEGY.md` (raíz del repo) menciona el Hybrid Genesis y es dominio del skill `social-strategist`. Se le avisa al propietario para que decida quién lo corrige.

---

# Parte B — Desbifurcar la narrativa simple

## B.1 §4 pasa a guión único — `docs/PRODUCT-SIMPLE.md`

Se borran las cuatro versiones por profesión. En su lugar, **un solo guión hablado**, idéntico para cualquier persona, apoyado en las dos anclas que ya funcionan con todos: las mil monedas (§1.3) y Darwin (`CAP-03`). Aporta lo que §1 no tiene: las palabras exactas para decirlo en voz alta.

Texto aprobado por el propietario:

```
## 4. El guión (válido para cualquier persona)

¿Sabes que si mil personas lanzan una moneda diez veces, a alguna le salen diez
caras seguidas? Su moneda no es mágica. Hubo mil intentos.

En invertir con programas pasa exactamente eso: se prueban miles de estrategias
sobre el pasado, una se ve preciosa, le meten dinero y pierden. Nunca hubo nada
ahí. Fue la afortunada.

Yo construyo la máquina que detecta a esa afortunada y la reprueba. Cría miles de
estrategias por selección natural, como Darwin, y luego las mete a un examen
diseñado para tumbarlas. Solo las que sobreviven tocan dinero.

Y corre en tu computadora. No en la nube de nadie.
```

Con una nota corta: si el interlocutor profundiza, se sigue por la capacidad que le picó (`CAP-NN` en §2) — nunca por una versión distinta del guión. `## 5. Mantenimiento` conserva su número.

## B.2 §1.6 con lenguaje auto-identificable

**Regla nueva:** la etiqueta describe **la conducta de la persona**, no su gremio. Si un término del sector aporta, va entre paréntesis después de la descripción llana — nunca como título de la fila. Nadie se presenta como "trader retail".

`docs/PRODUCT-SIMPLE.md` §1.6:

| Antes | Después |
|---|---|
| Alguien que investiga estrategias por su cuenta | Alguien a quien le gusta probar ideas con datos y ver si de verdad funcionan |
| Alguien que ya opera con reglas fijas | Alguien que invierte su propio dinero, por su cuenta, siguiendo reglas en vez de corazonadas |
| Alguien que quiere pasar una prueba de "cuenta fondeada" | Alguien que quiere que una empresa le preste capital para operar y tiene que pasar una prueba para conseguirlo (lo llaman "cuenta fondeada") |
| Alguien que maneja varias estrategias o un fondo pequeño | Alguien que ya maneja dinero de otras personas, o varias cuentas a la vez |

`docs/PRODUCT.md` §1.6 conserva los términos de industria —su audiencia los conoce— y **gana una columna con la frase llana equivalente**, para que quien construya el sitio web tenga ambas a la mano y no propague jerga por descuido.

## B.3 `CAP-03` deja de llamarse "evolutivo"

Consecuencia directa de la Parte A. Pasa a **"Búsqueda automática en el espacio de estrategias"** — que es literalmente como lo planteó el propietario en el encargo original: *"un motor de búsqueda automático basado en múltiples algoritmos"*. La evolución queda como **un motor entre varios**, el mejor explicado, no el único.

En `docs/PRODUCT-SIMPLE.md`, Darwin **se conserva** como la analogía principal de `CAP-03`: es genuinamente la mejor puerta de entrada para alguien sin contexto. Se le añade un párrafo corto: existen otros métodos de búsqueda, y se nombran en llano (probar todas las combinaciones; ir aprendiendo dónde conviene buscar; mapear la forma del terreno).

## B.4 La regla anti-bifurcación

Borrar §4 sin escribir la regla garantiza que vuelva a crecer. Se canoniza en tres sitios:

- **`docs/PRODUCT-SIMPLE.md` §5** — prohibido añadir versiones por profesión, industria o perfil. Si una explicación solo funciona para un gremio, la analogía está mal elegida: se corrige la analogía, no se añade una variante.
- **`docs/PRODUCT.md` §5** — el mismo límite, más el invariante de las dos vías.
- **`.agents/skills/architect/SKILL.md` §📣** — dos reglas nuevas: *dos vías, nunca más* y *etiqueta por conducta, no por gremio*. Es lo que impide que un Architect futuro repita el error.

## B.5 Referencia desfasada

`CLAUDE.md` línea 38 describe el documento como *"guía de conversación por perfil de interlocutor"* — única referencia externa a §4 en todo el repositorio. Se corrige a *"guión único universal"*.

---

# Parte C — Las 166 narrativas de feature

Nivel bajo del mapeo 1:1, en la sección `## Narrativa de Producto` de cada `docs/features/*.md`. La plantilla ya está escrita en `docs/templates/FEATURE.md`. Hoy: 0 de 166.

## C.1 Profundidad graduada

No una lista fija, sino una pregunta aplicada feature por feature:

> **¿El usuario nota si esta feature no existe?**

- **La nota como una capacidad** → narrativa completa (~15 líneas): historia de usuario, problema, solución, versión llana. Estimado ~120 features.
- **Solo la nota como "la aplicación funciona"** (reloj interno, bitácora de auditoría, transporte binario, aislamiento de procesos) → entrada corta de 2-3 líneas. Estimado ~46.

La entrada corta **conserva su `CAP-NN`**: lo que se gradúa es la extensión, nunca la cobertura.

## C.2 Lotes

Uno por capacidad, en orden de pipeline, una sesión por lote con pausa entre lotes (`GOVERNANCE-6.1`).

| Lote | Capacidad | Features |
|---|---|---|
| 1 | `CAP-01` — datos confiables (`ingest`) | 22 |
| 2 | `CAP-02` — simulador honesto (`validate` núcleo) | 14 |
| 3 | `CAP-03` — búsqueda automática (`generate`) | 21 |
| 4 | `CAP-04` — guantelete de robustez | 29 |
| 5 | `CAP-05` + `CAP-07` — cuarentena y flota | 12 |
| 6 | `CAP-06` — ejecución y defensa | 22 |
| 7 | `CAP-08` — ciclo cerrado | 10 |
| 8 | `CAP-09` — interfaz y plomería transversal | 22 |
| 9 | `CAP-10` — soberanía y negocio | 14 |

**Las escriben subagentes Sonnet, uno por lote, con el skill `architect` cargado.** Aquí está el 90% del volumen de todo el plan —20.218 líneas que leer— y leer una feature para resumir qué problema resuelve no requiere juicio de Opus. Meterlo en el contexto del coordinador es gasto sin retorno.

**Briefing que recibe cada subagente** (sin él, la delegación produce deriva de voz):

1. Invocar el skill `architect` — dispara el gate de `governance.md` y las reglas de dominio de producto.
2. `docs/PRODUCT.md` y `docs/PRODUCT-SIMPLE.md` completos — la voz a replicar y el `CAP-NN` de su lote.
3. **El lote 1 ya escrito, como ejemplar canónico.** Es la referencia de tono, longitud y registro. Por eso el lote 1 lo escribe el coordinador, no un subagente.
4. La plantilla `## Narrativa de Producto` de `docs/templates/FEATURE.md`.
5. La lista exacta de features de su lote, y el criterio de profundidad graduada de C.1.
6. Las prohibiciones duras: cero promesas de rentabilidad, cero nombres de función o ruta, cero invención. **Si una feature no se puede narrar sin inventar, se reporta como subespecificada en vez de rellenarse** (C.3).

**Auditoría al volver:** el coordinador revisa cada lote contra los tres greps de verificación y contra el ejemplar. Un lote con deriva de voz se devuelve, no se parcha.

**Paralelismo:** los lotes 2 a 9 son independientes entre sí (features distintas, archivos distintos) y pueden correr en paralelo. El lote 1 es secuencial y bloqueante, porque es la referencia.

## C.3 Efecto secundario deseable

Escribir "qué problema resuelve esto para una persona" obliga a entender la feature. Las delgadas del corpus (mínimo 53 líneas, mediana 106) se van a resistir. Si una narrativa no se puede escribir sin inventar, **la feature está subespecificada**. Esos casos **no se rellenan con prosa de venta**: se reportan al propietario con nombre y qué falta. La pasada funciona como auditoría de producto gratuita.

---

# Modelo de ejecución — quién hace qué y por qué

El criterio no es "delegar todo" ni "hacerlo todo": es **dónde el volumen justifica el costo de briefing y dónde el acoplamiento lo prohíbe.**

| Parte | Quién | Volumen | Por qué |
|---|---|---|---|
| **A** — puerto de búsqueda | Coordinador | ~12 archivos, ediciones puntuales | Cirugía de doctrina sobre ADRs acoplados por referencias cruzadas. Un ADR mal editado crea contradicciones entre documentos vigentes — el fallo exacto que el Protocolo de Mantenimiento de ADRs existe para prevenir, y que ya costó 42 casos de deuda en este repositorio. El volumen es bajo; el riesgo de delegarlo, no |
| **B** — desbifurcar | Coordinador | 5 archivos | Texto ya aprobado por el propietario. Escribir el briefing cuesta más que ejecutar las ediciones |
| **C** — 166 narrativas | **Subagentes Sonnet**, uno por lote, con skill `architect` | 20.218 líneas a leer, 166 secciones a escribir | Aquí está el 90% del volumen del plan. Leer una feature para resumir qué problema resuelve no requiere juicio de Opus. El lote 1 lo escribe el coordinador como ejemplar canónico; los lotes 2 a 9 se delegan en paralelo y se auditan al volver |

**Orden obligatorio:** A → B → C. La Parte A renombra y reencuadra `CAP-03`, y la Parte B lo propaga a los dos documentos maestros. Los subagentes de la Parte C reciben esos documentos **ya corregidos** como su referencia de voz — arrancar C antes obligaría a reescribir el lote de `generate` entero.

---

# Verificación

**Parte A:**

```bash
# El techo por paridad con el competidor desapareció de la doctrina
grep -rn 'copia descarada de SQX\|SQX no hace' docs/adr/ docs/modules/
#   → sin resultados

# Ningún método aparece subordinado como "modo del motor genético"
grep -rniE 'modo del motor (genético|nsga)' docs/
#   → sin resultados

# El eje "qué se busca" ya no presupone el método
grep -rn 'el motor evolutivo combina' docs/adr/
#   → sin resultados

# El título de ADR-0113 quedó sincronizado en los dos índices
diff <(grep -o 'ADR-0113.*' docs/ADR.md) <(grep -o 'ADR-0113.*' docs/README.md)

# Cero referencias rotas tras la reescritura
grep -rn 'ADR-0113' --include=*.md docs/ | wc -l   # comparar contra el conteo previo
```

**Parte B:**

```bash
# Cero bifurcación por profesión
grep -niE 'fisioterap|abogado|piloto|mecánic|gerente de ventas|machine learning' docs/PRODUCT-SIMPLE.md
#   → sin resultados

# Cero jerga de gremio como etiqueta
grep -ni 'trader retail' docs/PRODUCT-SIMPLE.md
#   → sin resultados

# Las 10 capacidades siguen en paridad
diff <(grep -o 'CAP-[0-9][0-9]' docs/PRODUCT.md | sort -u) \
     <(grep -o 'CAP-[0-9][0-9]' docs/PRODUCT-SIMPLE.md | sort -u)

# El hook de formato acepta el skill tras las reglas nuevas
python3 .agents/hooks/validate-agent-docs.py --audit .agents/skills/architect/SKILL.md | grep -c BLOQUEA
#   → 0
```

**Prueba real, la que importa:** leer el guión de §4 en voz alta a alguien sin contexto técnico y confirmar que entiende qué problema resuelve Drasus sin una sola pregunta aclaratoria. Si hay que adaptarlo a quién es esa persona, el guión falló y se corrige el guión — no se añade una versión.

**Parte C, por lote:**

```bash
grep -l 'Narrativa de Producto' docs/features/*.md | wc -l   # cobertura acumulada
grep -h -o 'CAP-[0-9][0-9]' docs/features/*.md | sort -u     # solo CAP-01..CAP-10
grep -niE 'garantiza|rentabilidad asegurada|retorno esperado del' docs/features/*.md
#   → sin resultados
```

---

# Lo que NO se hace

- **No se crea un ADR nuevo** para el puerto de búsqueda. Se corrige ADR-0113, que es donde está el defecto.
- **No se toca el contenido de dominio de ADR-0043 y ADR-0108 a ADR-0111.** Los cuatro dominios genómicos y su gramática condición→acción quedan íntegros; solo se les quita el lenguaje que presupone el método de búsqueda.
- **No se implementa ningún método nuevo.** Esta pasada cataloga los 35 y abre el puerto. Construir es decisión posterior, con su propio gate — incluidos los cinco de mayor valor esperado.
- **No se delega la Parte A ni la B.** El volumen no lo justifica y el acoplamiento entre ADRs lo desaconseja.
- **No se añade una versión del pitch por profesión.** Es el defecto que este plan corrige.
- **No se borra §1.6.** No es narrativa bifurcada: es la tabla de segmentos de mercado. Cambia su redacción, no su existencia.
- **No se renumeran los `CAP-NN`.** Son de solo-añadir, misma disciplina que los ADR.
- **No se toca `CONTENT-STRATEGY.md`** — dominio del skill `social-strategist`. Se reporta.
- **No se commitea nada** sin que el propietario lo pida en el turno actual.
