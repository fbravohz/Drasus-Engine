# Drasus Engine — Definición de Producto (Nivel Técnico)

Fuente única de verdad para explicar **qué es Drasus Engine, para quién y por qué**, a una audiencia que tolera vocabulario técnico. De aquí salen las infografías, presentaciones, el sitio web, el roadmap público y el elevator pitch — ningún material de difusión inventa contenido por su cuenta: lo deriva de aquí.

**Gemelo no técnico:** [`PRODUCT-SIMPLE.md`](./PRODUCT-SIMPLE.md). Misma estructura, mismos identificadores `CAP-NN`, cero jerga. Los dos documentos son **espejos**: si una capacidad cambia aquí, cambia allá en la misma edición.

---

## 0. Cómo usar este documento

### 0.1 Los tres niveles de profundidad

| Nivel | Qué responde | Dónde vive | Tiempo de lectura |
|---|---|---|---|
| **General** | ¿Qué es esto y por qué debería importarme? | §1 de este archivo (y su gemelo simple) | 2 minutos |
| **Medio** | ¿Qué hace exactamente, capacidad por capacidad? | §2 — las diez capacidades `CAP-01`…`CAP-10` | 20 minutos |
| **Bajo** | ¿Qué problema concreto resuelve cada pieza y cómo? | Sección `## Narrativa de Producto` dentro de cada [`docs/features/*.md`](./features/) | bajo demanda, una feature a la vez |

El nivel bajo **no se duplica aquí**. Vive dentro del documento de cada feature, en el mismo archivo que su especificación técnica. Es una decisión deliberada: dos archivos separados que describen las mismas 166 features se desincronizan en semanas; una sección dentro del archivo que ya existe no puede desincronizarse, porque el ingeniero que cambia la especificación tiene la narrativa a la vista.

### 0.2 El contrato de sincronía 1:1

Tres reglas hacen que el mapeo se sostenga solo:

- **Los identificadores `CAP-NN` son estables y de solo-añadir.** Un `CAP` publicado nunca se renumera ni se reutiliza. Capacidad nueva = siguiente número libre. Es la misma disciplina de los ADR.
- **Toda feature pertenece a exactamente una capacidad.** El índice de §4 es la fuente de verdad de esa asignación. Una feature sin `CAP` asignado es un hueco de producto, no un descuido de formato.
- **Este archivo y `PRODUCT-SIMPLE.md` se editan juntos.** Un `CAP` que existe aquí y no allá (o al revés) es una violación, no una versión en progreso.

### 0.3 Qué material se genera desde qué nivel

| Material | Nivel de origen |
|---|---|
| Elevator pitch, portada del sitio, post de redes | §1 (general) |
| Presentación a inversionista, deck de ventas, página de producto | §2 (medio) |
| Documentación de usuario, ficha de feature, comparativa contra competencia | §3 nivel bajo (dentro de cada feature) |
| Roadmap público | §3 |

---

## 1. Nivel General — Qué es Drasus Engine

### 1.1 En una frase

**Una fábrica de estrategias de trading que corre entera en tu propia máquina: las descubre por búsqueda automática, las somete a un guantelete de pruebas estadísticas diseñado para reprobarlas, y solo deja operar dinero real a las que sobreviven.**

### 1.2 En un párrafo

Drasus Engine es un motor de investigación cuantitativa de escritorio. Automatiza el ciclo completo de vida de una estrategia de trading algorítmico: ingiere y sanea datos de mercado del propio usuario, genera miles de estrategias candidatas mediante algoritmos evolutivos multiobjetivo, las somete a validación estadística de grado institucional (validación cruzada combinatoria, simulaciones Monte Carlo, análisis walk-forward, corrección por pruebas múltiples), las incuba en simulación en tiempo real antes de arriesgar un peso, las ejecuta en cuentas reales con capas de defensa de capital, y cierra el bucle retroalimentando lo aprendido a la fase de generación. Todo el cómputo ocurre en el hardware del usuario. El proveedor nunca ejecuta nada.

### 1.3 El problema

El trading algorítmico retail tiene un problema estructural, y no es la falta de ideas:

- **El sobreajuste es la norma, no la excepción.** Cualquiera puede encontrar una curva de capital hermosa buscando lo suficiente en datos históricos. La abrumadora mayoría de esas curvas son ruido con forma de señal, y se derrumban en el primer mes de dinero real.
- **Las herramientas existentes optimizan la búsqueda, no el filtro.** Generar diez mil estrategias es fácil y ya está resuelto. Saber cuáles de esas diez mil merecen capital es el problema difícil, y es donde casi ninguna herramienta invierte.
- **Nadie cuenta cuántas veces buscó.** Si pruebas diez mil configuraciones, la mejor de ellas se ve genial por pura probabilidad. Sin descontar ese número de intentos, la estadística que reportas está inflada y no lo sabes.
- **El dato del usuario no es del usuario.** Las plataformas en la nube ven las estrategias, los parámetros y los resultados de quien las usa. El activo más valioso de un investigador cuantitativo vive en el servidor de alguien más.
- **La simulación miente y nadie audita la mentira.** Costos de transacción idealizados, ejecución instantánea, ausencia de deslizamiento y de selección adversa producen backtests que no se parecen a la operación real.

### 1.4 La tesis

**El valor no está en generar estrategias. Está en destruirlas correctamente.**

Drasus invierte la proporción habitual de esfuerzo: la generación es una fase, la validación es el producto. El sistema está construido alrededor de un principio hostil — *toda estrategia es culpable hasta que sobrevive la prueba* — y de un compromiso de soberanía: el cómputo, los datos y las estrategias viven en la máquina del usuario.

### 1.5 Lo que Drasus Engine **no** es

Decirlo explícitamente ahorra la mitad de las conversaciones:

- **No es un proveedor de datos de mercado.** Drasus aporta los conectores que traen los datos del usuario y el motor que los refina; el dato es del usuario y de su fuente (ADR-0154).
- **No es un servicio de señales ni un gestor de capital de terceros.** El usuario opera su propia cuenta. Drasus es software, no un fondo.
- **No es una plataforma en la nube.** El motor corre en el hardware del usuario — su PC o su servidor privado — nunca en servidores del proveedor.
- **No promete rentabilidad.** Promete un proceso de validación honesto y auditable. Lo que ese proceso encuentre depende del mercado y del usuario.
- **No es una caja negra.** Toda estrategia generada es un árbol de reglas legible y traducible a lenguaje natural, no una red neuronal opaca.

### 1.6 Para quién

La columna llana es la que se usa en material dirigido al público: **nadie se auto-asigna una etiqueta de gremio.** Un abogado que invierte los sábados no lee "trader retail" y piensa "ese soy yo"; sí se reconoce en "invierto mi propio dinero siguiendo reglas".

| Cómo se describe en llano | Perfil (término de industria) | Qué obtiene |
|---|---|---|
| Alguien a quien le gusta probar ideas con datos y ver si de verdad funcionan | Investigador cuantitativo independiente | Un laboratorio completo de descubrimiento y validación sin infraestructura ni licencias institucionales. |
| Alguien que invierte su propio dinero, por su cuenta, siguiendo reglas en vez de corazonadas | Trader retail sistemático | Estrategias con evidencia estadística detrás, y capas de defensa que protegen la cuenta cuando algo se rompe. |
| Alguien que quiere que una empresa le preste capital para operar y tiene que pasar una prueba para conseguirlo | Candidato a cuenta de fondeo | Un evaluador que aplica las reglas de la firma de fondeo *antes* de arriesgar la evaluación, no después de reprobarla. |
| Alguien que ya maneja dinero de otras personas, o varias cuentas a la vez | Operador profesional / gestor de fondo pequeño | Gestión de flota multi-estrategia, jerarquía de cuentas, roles de operador y reportes trazables. |

---

## 2. Nivel Medio — Las diez capacidades

Cada capacidad se presenta como **historia de usuario → problema → solución → por qué así**, con las features que la materializan y la fase del roadmap donde se construye.

---

### CAP-01 — Datos en los que se puede confiar

> **Como** investigador de estrategias, **quiero** que mis datos de mercado lleguen limpios, auditados y sin información del futuro filtrada, **para** que ningún resultado que vea después sea ficción.

**El problema.** Un backtest sobre datos sucios no es un backtest, es una alucinación con gráfica. Los defectos son sistemáticos: velas faltantes, precios imposibles, husos horarios inconsistentes, y sobre todo **fuga de información del futuro** (*look-ahead bias*) — usar en la simulación un dato que en ese instante real todavía no existía. Un dato fundamental que se revisó tres meses después de publicarse, usado con su valor revisado, convierte una estrategia mediocre en una máquina de dinero imaginaria.

**La solución.** Una capa de ingesta soberana con tres vías de entrada (interfaz de programación autenticada, credenciales del bróker, exportación manual desde el terminal del usuario), un protocolo de saneamiento de seis capas, y dos validadores independientes: uno estructural (precios negativos, saltos imposibles, huecos) y uno de **punto-en-el-tiempo**, que verifica que cada dato usado en el instante *t* realmente estaba disponible en *t*. Los eventos fundamentales se versionan por *vintage*: se guarda la primera publicación y cada revisión posterior por separado, y la simulación solo puede ver lo que existía entonces.

**Por qué así.**
- **Point-in-time (PIT)** es la única defensa real contra el sesgo de anticipación. No es una comprobación opcional al final: es una propiedad del almacenamiento.
- **Drasus no vende datos** (ADR-0154). Vende los conectores y el refinado. Esto evita el costo recurrente de licencias de datos y deja al usuario dueño de su fuente, pero obliga al sistema a **declarar sus capacidades** por bróker: si una combinación de activo y granularidad no está soportada, se inhabilita *antes* del intento en vez de fallar con un error opaco.
- **Barras algorítmicas** (por número de operaciones, por volumen, Renko, de rango, por valor negociado) además de las barras por tiempo, porque el reloj es una forma arbitraria de cortar el mercado y no siempre la mejor.
- **Diferenciación fraccional** para volver estacionaria una serie de precios conservando el máximo de memoria estadística — el método clásico (restar el precio anterior) la vuelve tratable pero le borra la memoria que contenía la señal.

**Features:** [`sovereign-data-fetcher`](./features/sovereign-data-fetcher.md) · [`data-sanitizer-pipeline`](./features/data-sanitizer-pipeline.md) · [`data-validator`](./features/data-validator.md) · [`pit-data-validator`](./features/pit-data-validator.md) · [`data-normalization-layer`](./features/data-normalization-layer.md) · [`data-import-wizard`](./features/data-import-wizard.md) · [`hive-partition-manager`](./features/hive-partition-manager.md) · [`databank-lake`](./features/databank-lake.md) · [`databank-manager`](./features/databank-manager.md) · [`duckdb-sql-engine`](./features/duckdb-sql-engine.md) · [`duckdb-resampler`](./features/duckdb-resampler.md) · [`temporal-aggregator`](./features/temporal-aggregator.md) · [`algorithmic-bars`](./features/algorithmic-bars.md) · [`fractional-differencer`](./features/fractional-differencer.md) · [`hybrid-data-transformer`](./features/hybrid-data-transformer.md) · [`order-flow-microstructure`](./features/order-flow-microstructure.md) · [`fundamental-event-store`](./features/fundamental-event-store.md) · [`quality-heatmap-generator`](./features/quality-heatmap-generator.md) · [`background-download-manager`](./features/background-download-manager.md) · [`synthetic-symbols`](./features/synthetic-symbols.md) · [`instrument-catalog`](./features/instrument-catalog.md) · [`session-catalog`](./features/session-catalog.md)

**Módulo:** `ingest` · **Fase:** EPIC-1 (en curso)

---

### CAP-02 — Un simulador que no miente

> **Como** investigador de estrategias, **quiero** un motor de backtest determinista que modele la fricción real del mercado, **para** que la diferencia entre lo simulado y lo real sea una sorpresa pequeña y no un desastre.

**El problema.** El backtest promedio es optimista por construcción. Asume que la orden se llena al precio deseado, ignora el deslizamiento, ignora que las órdenes límite solo se llenan cuando el mercado va en tu contra (**selección adversa**), y con frecuencia comete el error más caro de todos: calcular la señal con el precio de cierre de la vela y ejecutar en esa misma vela — información que en la realidad todavía no tenías.

**La solución.** Un motor dual: una ruta **express** vectorizada para descartar rápido a escala masiva, y una ruta **secuencial** dirigida por eventos para la simulación de alta fidelidad, con un contrato de consistencia conservadora entre ambas (la ruta rápida nunca reporta mejor que la ruta fiel). Sobre eso se apila el modelado de fricción institucional: deslizamiento, comisiones, selección adversa en órdenes límite, y el invariante de **alineación a la apertura** — la señal calculada al cierre de la vela *N* se ejecuta en la apertura de la vela *N+1*, siempre, sin excepción configurable.

**Por qué así.**
- **Determinismo bit a bit:** dos corridas idénticas producen el mismo hash. Sin esa propiedad no hay auditoría posible ni reproducción de un resultado que alguien discute.
- **La ruta dual existe porque el volumen es Alpha.** Descartar diez mil candidatas malas rápido y simular las cien buenas con fidelidad total es estrictamente mejor que hacer todo con fidelidad media.
- **El registro de procedencia (`expedition-ledger`)** anota cada corrida de forma inmutable y la ata a los artefactos que tocó. Esto no es contabilidad burocrática: es el insumo que la corrección estadística de `CAP-04` necesita para saber cuántas veces se buscó realmente.

**Features:** [`backtest-engine`](./features/backtest-engine.md) · [`nautilus-integration`](./features/nautilus-integration.md) · [`slippage-models`](./features/slippage-models.md) · [`institutional-friction-modeling`](./features/institutional-friction-modeling.md) · [`equity-curve-tracker`](./features/equity-curve-tracker.md) · [`institutional-metrics`](./features/institutional-metrics.md) · [`precision-sizing-models`](./features/precision-sizing-models.md) · [`executable-container`](./features/executable-container.md) · [`strategy-versioning`](./features/strategy-versioning.md) · [`expedition-ledger`](./features/expedition-ledger.md) · [`pipeline-registry`](./features/pipeline-registry.md) · [`incremental-test-engine`](./features/incremental-test-engine.md) · [`universal-basket-backtester`](./features/universal-basket-backtester.md) · [`portfolio-backtest`](./features/portfolio-backtest.md)

**Módulo:** `validate` (núcleo) · **Fase:** EPIC-2

---

### CAP-03 — Búsqueda automática en el espacio de estrategias

> **Como** investigador de estrategias, **quiero** un motor de búsqueda automático basado en múltiples algoritmos para explorar Alpha en el espacio de variables del mercado, **para** no depender de mi intuición ni de mi capacidad de escribir código a mano.

**El problema.** El espacio de estrategias posibles es astronómico: qué indicadores, con qué parámetros, con qué condiciones de entrada y salida, en qué régimen de mercado, con qué gestión de riesgo. Un humano explora decenas de combinaciones. La probabilidad de que las mejores estén entre las que a él se le ocurrieron es despreciable. Y el enfoque ingenuo —probar todo por fuerza bruta y quedarse con lo mejor— produce sobreajuste industrializado.

**La solución.** Un **puerto abierto de estrategias de búsqueda**: la representación (un árbol de reglas condición→acción) es común y fija, y el método que la explora es intercambiable. Ninguna estrategia de búsqueda es el motor por defecto del cual las demás son modos — todas entran como pares.

La primera implementación, y la mejor entendida, es el motor genético **NSGA-II**.

**Qué es NSGA-II, en concreto.** Es un algoritmo evolutivo: parte de una población de estrategias aleatorias, las evalúa, cruza y muta a las mejores, y repite por generaciones. Su diferencia con un algoritmo genético común es que **no colapsa la calidad a un solo número**. En vez de premiar "el mejor Sharpe", mantiene el **frente de Pareto**: el conjunto de estrategias donde ninguna es mejor que otra en *todos* los objetivos a la vez. Una estrategia con rentabilidad media y caída máxima pequeña, y otra con rentabilidad alta y caída grande, ambas sobreviven — son incomparables, no rivales. El resultado no es una estrategia ganadora, es un abanico de compromisos entre los que el usuario elige según su tolerancia real al riesgo.

**Por qué NSGA-II y no una función de puntaje ponderada.** Una fórmula que suma objetivos con pesos fijos (el enfoque de StrategyQuant X) obliga a decidir *antes de buscar* cuánto vale un punto de rentabilidad frente a un punto de caída máxima. Esa decisión es arbitraria, esconde la frontera real de compromisos, y sesga la búsqueda hacia una región del espacio que el usuario eligió a ciegas. El frente de Pareto no impone esa decisión: la difiere hasta que hay evidencia para tomarla.

**Las otras estrategias de búsqueda, ya especificadas:**
- **Optimización bayesiana** (Procesos Gaussianos): construye un modelo probabilístico de dónde vive el óptimo y decide dónde probar a continuación. Sirve cuando cada evaluación es cara y no puedes permitirte miles.
- **Búsqueda exhaustiva:** recorre el espacio completo sin heurística. Lenta pero sin puntos ciegos, y es la referencia contra la cual se mide si una heurística realmente aporta.
- **Optimización jerárquica:** mapea las variables por bloques secuenciales en vez de todas a la vez, evitando la explosión combinatoria de la fuerza bruta.
- **Regresión simbólica:** descubre la *forma* de la fórmula, no solo sus constantes.
- **Búsqueda topológica:** en vez del punto de máximo rendimiento, busca la meseta ancha. Un pico estrecho es un artefacto de la búsqueda; una meseta es señal.

**Y la representación sobre la que todas operan, común e independiente del método:**
- **Genomas modulares por dominio:** el mismo patrón `condición → acción` aplicado a riesgo, a régimen de mercado y a portafolio — no solo a la señal de entrada.
- **Comodines (WildCards):** huecos deliberados en el árbol de reglas que el motor de búsqueda rellena, en vez de fijar la estructura completa de antemano.

**Por qué el puerto está abierto.** Drasus es infraestructura de investigación, y descartar un método que funciona porque no es el que se usó primero sería tirar dinero. La optimización de caja negra es un campo activo en ingeniería de diseño, biología computacional, logística, aeroespacial y diseño de fármacos — disciplinas que llevan décadas resolviendo el mismo problema con presupuestos mayores. **El catálogo vigente de métodos admitidos, con su respaldo académico, vive en ADR-0113**, junto con el gate que decide qué se cataloga y qué se construye.

**El freno anti-sobreajuste, y la condición de entrada al puerto:** desde el primer intento y **sin importar el método**, el sistema cuenta cuántas configuraciones se probaron. Ese número es el insumo de la corrección estadística de `CAP-04`. Contarlo después es imposible, así que se instrumenta desde la primera corrida — y es el filtro de admisión que ninguna estrategia de búsqueda puede saltarse: **un método que no sepa reportar su presupuesto consumido no entra**, porque invalidaría la corrección río abajo.

**Features:** [`nsga2-optimizer`](./features/nsga2-optimizer.md) · [`bayesian-optimizer`](./features/bayesian-optimizer.md) · [`parameter-optimization`](./features/parameter-optimization.md) · [`hierarchical-parameter-optimization`](./features/hierarchical-parameter-optimization.md) · [`ast-compiler`](./features/ast-compiler.md) · [`dsr-tracking-engine`](./features/dsr-tracking-engine.md) · [`hmm-regime-detection`](./features/hmm-regime-detection.md) · [`manual-regime-tagger`](./features/manual-regime-tagger.md) · [`zero-crossing-filter`](./features/zero-crossing-filter.md) · [`strategy-ensemble`](./features/strategy-ensemble.md) · [`feature-router`](./features/feature-router.md) · [`node-preview`](./features/node-preview.md) · [`signal-topology`](./features/signal-topology.md) · [`formula-canvas`](./features/formula-canvas.md) · [`adaptive-volume-indicators`](./features/adaptive-volume-indicators.md) · [`event-impact-scorer`](./features/event-impact-scorer.md) · [`asset-exposure-map`](./features/asset-exposure-map.md) · [`fundamental-indicator-projector`](./features/fundamental-indicator-projector.md) · [`design-manifest`](./features/design-manifest.md) · [`visual-stockpicker-configurator`](./features/visual-stockpicker-configurator.md) · [`alpha-harvesting-gateway`](./features/alpha-harvesting-gateway.md)

**Módulo:** `generate` · **Fase:** EPIC-3

---

### CAP-04 — El guantelete de robustez

> **Como** investigador de estrategias, **quiero** un conjunto de pruebas diseñado explícitamente para reprobar mis estrategias, **para** que las que sobrevivan lo hagan por mérito y no porque nadie las cuestionó.

**El problema.** Este es el problema central del trading cuantitativo, y merece nombrarse sin eufemismo: **si buscas lo suficiente, encontrarás algo que se ve bien y no vale nada**. Diez mil intentos garantizan que el mejor de ellos tenga estadísticas brillantes por azar puro. La curva de capital ascendente no es evidencia; es el resultado esperado de haber buscado mucho. Ninguna cantidad de belleza gráfica sustituye una prueba de significancia.

**La solución.** Una cascada de pruebas ordenada por costo computacional (ligera → media → pesada) con abandono temprano: si una estrategia reprueba una prueba barata, nunca llega a las caras. Cada prueba ataca un modo de falla distinto:

- **Validación cruzada combinatoria purgada (CPCV)** con **probabilidad de sobreajuste del backtest (PBO)**. Divide el historial en muchos bloques, prueba todas las combinaciones de bloques de entrenamiento y prueba, y estima la probabilidad de que la estrategia que se ve mejor en entrenamiento sea, en realidad, mediocre fuera de muestra. El "purgado" elimina los datos adyacentes al corte para que la información no se filtre a través de la frontera.
- **Análisis walk-forward.** Optimiza en una ventana, opera en la siguiente, avanza, y repite. Mide si la estrategia sigue funcionando cuando solo puede usar información pasada — que es la única condición real de operación.
- **Monte Carlo.** Reordena, remuestrea y perturba las operaciones miles de veces para dibujar la distribución de resultados posibles. La pregunta que responde: *el resultado que obtuve, ¿está en el centro de esa distribución o en la cola afortunada?*
- **Corrección por pruebas múltiples (Sharpe deflactado).** Descuenta del Sharpe reportado el número de configuraciones que se probaron. Sin esta corrección, cualquier búsqueda masiva reporta cifras infladas y el investigador se engaña con rigor aparente.
- **Validación cruzada de mercados.** Corre la estrategia en mercados hermanos correlacionados **sin reoptimizar parámetros**. Una lógica genuina degrada con gracia; una lógica sobreajustada se desploma.
- **Ablación de reglas.** Desactiva componentes de la estrategia uno por uno. Si quitar una regla no cambia el resultado, esa regla es decoración sobreajustada. Es la Navaja de Ockham aplicada mecánicamente.
- **Aislamiento de componentes.** Verifica que la estrategia le gane a una versión aleatoria de sí misma. Sorprende cuántas no lo hacen.
- **Agente de ruido adversario.** Perturba deliberadamente los datos de entrada buscando romper la estrategia. Es un equipo rojo automatizado.
- **Auditoría de gradiente de fragilidad y búsqueda topológica de mesetas.** Una estrategia cuyo rendimiento se derrumba al mover un parámetro un 1% está sobre un pico, no sobre una meseta. Los picos son artefactos de la búsqueda; las mesetas son señal.
- **Agrupamiento de toxicidad por componentes principales (PCA).** Detecta familias enteras de estrategias que comparten el mismo defecto estructural, para purgarlas en bloque en vez de una por una.
- **Poda temporal vectorial.** Detecta ventanas horarias o de día de semana donde la estrategia pierde sistemáticamente, y las poda.
- **Evaluador de firma de fondeo.** Aplica las reglas exactas de la firma (pérdida diaria máxima, pérdida total, objetivo de beneficio) antes de arriesgar la evaluación pagada.
- **Capa de inferencia estadística (EBTA).** Cuantifica la significancia del resultado en vez de aceptarlo por inspección visual.

Todo eso converge en un **agregador de score de robustez** que emite un número y un **veredicto en lenguaje natural** explicando en qué prueba falló y qué significa — determinista, sin modelo de lenguaje en la ruta crítica.

**Por qué así.** Porque el filtro *es* el producto. Es la diferencia entre "encontré algo que se ve bien" y "encontré algo que sobrevivió doce intentos independientes de destruirlo". La cascada por costo existe porque el guantelete completo sobre diez mil candidatas es computacionalmente inviable; la cascada lo vuelve viable sin bajar el estándar de las que llegan al final.

**Features:** [`cpcv-analyzer`](./features/cpcv-analyzer.md) · [`walk-forward-analyzer`](./features/walk-forward-analyzer.md) · [`monte-carlo-simulator`](./features/monte-carlo-simulator.md) · [`cross-market-validation`](./features/cross-market-validation.md) · [`rule-ablation`](./features/rule-ablation.md) · [`component-isolation`](./features/component-isolation.md) · [`adversarial-noise-agent`](./features/adversarial-noise-agent.md) · [`fragility-gradient-auditor`](./features/fragility-gradient-auditor.md) · [`topological-plateau-finder`](./features/topological-plateau-finder.md) · [`plateau-copilot`](./features/plateau-copilot.md) · [`pca-toxicity-analyzer`](./features/pca-toxicity-analyzer.md) · [`toxicity-purifier`](./features/toxicity-purifier.md) · [`vector-time-pruning`](./features/vector-time-pruning.md) · [`prop-firm-grader`](./features/prop-firm-grader.md) · [`statistical-inference-ebta`](./features/statistical-inference-ebta.md) · [`robustness-score-aggregator`](./features/robustness-score-aggregator.md) · [`robustness-verdict-engine`](./features/robustness-verdict-engine.md) · [`complexity-penalization`](./features/complexity-penalization.md) · [`alpha-purity-analyzer`](./features/alpha-purity-analyzer.md) · [`alpha-decoupling`](./features/alpha-decoupling.md) · [`factor-decomposition`](./features/factor-decomposition.md) · [`perfect-profit-benchmark`](./features/perfect-profit-benchmark.md) · [`regime-guard`](./features/regime-guard.md) · [`contextual-fitness-scorer`](./features/contextual-fitness-scorer.md) · [`dtw-adaptive-window`](./features/dtw-adaptive-window.md) · [`autoencoder-outlier-detector`](./features/autoencoder-outlier-detector.md) · [`volatility-stabilization`](./features/volatility-stabilization.md) · [`adaptive-logic-er`](./features/adaptive-logic-er.md) · [`event-driven-pipeline-triggers`](./features/event-driven-pipeline-triggers.md)

**Módulo:** `validate` (guantelete) · **Fase:** EPIC-4

---

### CAP-05 — Cuarentena antes del dinero real

> **Como** trader, **quiero** que toda estrategia aprobada opere en simulación en tiempo real durante un período de prueba antes de tocar mi cuenta, **para** detectar la brecha entre el papel y la realidad sin pagarla.

**El problema.** Una estrategia puede pasar todas las pruebas históricas y aun así fallar en vivo, porque el historial no contiene el presente: la liquidez de hoy, la latencia de hoy, el comportamiento del bróker de hoy. La brecha entre simulación y realidad no se descubre revisando el pasado con más cuidado; se descubre operando.

**La solución.** Un período de incubación configurable (por defecto siete días) en el que la estrategia opera contra el flujo de mercado **en vivo**, sin capital, mientras el sistema compara continuamente su comportamiento real contra la proyección estadística que el backtest predijo. La comparación usa la **prueba de Pardo**: contrasta dos series de resultados y determina si son estadísticamente compatibles o si la estrategia ya se desvió. Si la degradación cruza el umbral antes de terminar la cuarentena, la estrategia se retira sin llegar a operar — el sistema no espera a que el período termine para admitir lo que ya sabe.

**Por qué así.** Porque el costo de descubrir la brecha en simulación es cero y el de descubrirla con dinero real es el dinero real. Y porque el **cono de silencio** (aislar la estrategia en cuarentena de cualquier decisión de promoción prematura) evita el impulso, muy humano, de promover algo porque su tercer día se ve bien.

**Features:** [`paper-trader`](./features/paper-trader.md) · [`incubation-manager`](./features/incubation-manager.md) · [`pardo-comparison`](./features/pardo-comparison.md) · [`efficiency-incubation-dashboard`](./features/efficiency-incubation-dashboard.md) · [`live-strategy-view`](./features/live-strategy-view.md)

**Módulo:** `incubate` · **Fase:** EPIC-5

---

### CAP-06 — Ejecución real y defensa del capital

> **Como** trader, **quiero** que cada orden pase por filtros de seguridad antes de salir al mercado y que exista un interruptor de emergencia independiente, **para** que un error de software no me cueste la cuenta.

**El problema.** El código de trading falla de formas que el código normal no: se desconecta a media posición, envía una orden duplicada, calcula un tamaño de posición con la unidad equivocada, o sigue operando mientras el mercado se rompe. Y falla a la velocidad de la máquina, no a la del humano que lo supervisa.

**La solución.** Defensa en capas, cada una independiente de las demás:

- **Validador pre-operación:** once filtros de seguridad que cada orden atraviesa antes de salir, en menos de un milisegundo. Tamaño, exposición, margen, coherencia con el estado del portafolio, veto por veredicto de robustez degradado.
- **Máquina de estados de orden:** seis estados y transiciones válidas explícitas, con soporte de múltiples posiciones concurrentes independientes por estrategia y de-duplicación de señal por vela.
- **Vigilante en la sombra e interruptor de emergencia:** un proceso separado que vigila latido, latencia y conexión, y corta la operación completa cuando la salud técnica se degrada. Separado a propósito: un vigilante que vive dentro del proceso que vigila no sirve de nada cuando ese proceso se cuelga.
- **Monitor de seguridad operativa:** detecta deriva estadística en vivo — la estrategia sigue funcionando, pero ya no como funcionaba.
- **Recuperación ante caídas:** reconstruye el estado exacto tras un corte de luz o un reinicio, reconciliando contra el bróker en vez de asumir.
- **Seguridad soberana:** las credenciales del bróker se cifran localmente y **nunca salen de la máquina del usuario, en ningún nivel de suscripción**.
- **Cola de órdenes por prioridad y ruteo por perfil de volumen:** protege contra los límites de tasa del exchange en episodios de alta volatilidad, y suspende órdenes ante caídas de liquidez.
- **Reglas de portafolio como capa dominante:** límites globales que ninguna estrategia individual puede violar, sin importar lo que su lógica diga.

**Por qué así.** Porque estas capas son **Alpha defensivo**: no generan rentabilidad, evitan la ruina. Y porque el orden importa — se construyen en la misma fase en que el sistema pone dinero en riesgo, nunca después.

**Features:** [`pre-trade-validator`](./features/pre-trade-validator.md) · [`order-fsm`](./features/order-fsm.md) · [`system-watchdog`](./features/system-watchdog.md) · [`operational-safety-monitor`](./features/operational-safety-monitor.md) · [`crash-recovery`](./features/crash-recovery.md) · [`sovereign-security`](./features/sovereign-security.md) · [`broker-connector`](./features/broker-connector.md) · [`multiplatform-execution-bridge`](./features/multiplatform-execution-bridge.md) · [`order-priority-queue`](./features/order-priority-queue.md) · [`volume-profile-router`](./features/volume-profile-router.md) · [`portfolio-rules`](./features/portfolio-rules.md) · [`advanced-trade-management`](./features/advanced-trade-management.md) · [`kinetic-micro-management`](./features/kinetic-micro-management.md) · [`multi-ticket-manager`](./features/multi-ticket-manager.md) · [`audit-event-store`](./features/audit-event-store.md) · [`persistent-daemons`](./features/persistent-daemons.md) · [`data-bus-pubsub`](./features/data-bus-pubsub.md) · [`auto-auditoria-portafolios-vivos`](./features/auto-auditoria-portafolios-vivos.md) · [`throttling-metrics-dashboard`](./features/throttling-metrics-dashboard.md) · [`autopilot-metrics-provider`](./features/autopilot-metrics-provider.md) · [`copy-trading-engine`](./features/copy-trading-engine.md) · [`remote-portfolio-access-protocol`](./features/remote-portfolio-access-protocol.md)

**Módulo:** `execute` · **Fase:** EPIC-5 (bridge y conectores prioritarios) y EPIC-6 (stack nativo profundo)

---

### CAP-07 — Gestión de flota y capital

> **Como** operador con varias estrategias vivas, **quiero** que el capital se asigne entre ellas según su correlación real y no según mi corazonada, **para** que la cartera sea más robusta que la suma de sus partes.

**El problema.** Diez estrategias rentables pueden ser peor que tres si todas ganan y pierden al mismo tiempo. La correlación oculta es la forma más común de destruir una cuenta creyendo que está diversificada: diez apuestas al mismo factor son una apuesta grande disfrazada de diez pequeñas.

**La solución.** Un optimizador de portafolio con **paridad de riesgo jerárquica (HRP)** — agrupa las estrategias por similitud de comportamiento, construye una jerarquía y asigna capital por niveles. A diferencia del método clásico de media-varianza, HRP no necesita invertir la matriz de covarianzas, que es numéricamente inestable justo cuando más importa (muchas estrategias, poco historial). Encima se apilan: simulación de portafolio real con margen compartido, rebalanceo automático con cortacircuitos, análisis de correlación entre señales, portafolios federados que coexisten sin interferirse, y una **búsqueda de ajuste al portafolio** que inyecta el estado de la cartera viva como presión restrictiva dentro del motor evolutivo — de modo que `CAP-03` genera candidatas que *complementan* lo que ya opera, en vez de cientos de clones de lo mismo.

**Por qué así.** Porque la asignación de capital es donde el trabajo de descubrimiento se convierte, o no, en resultado. Y porque cerrar el bucle entre la cartera viva y la generación es una ventaja estructural: la mayoría de las herramientas generan a ciegas y dejan al usuario el problema de la diversificación.

**Features:** [`portfolio-optimizer`](./features/portfolio-optimizer.md) · [`portfolio-data-preparation`](./features/portfolio-data-preparation.md) · [`federated-portfolio`](./features/federated-portfolio.md) · [`fit-to-portfolio-search`](./features/fit-to-portfolio-search.md) · [`signal-correlation-analyzer`](./features/signal-correlation-analyzer.md) · [`strategy-config-diff`](./features/strategy-config-diff.md) · [`production-optimization`](./features/production-optimization.md)

**Módulo:** `manage` · **Fase:** EPIC-6

---

### CAP-08 — El ciclo que se cierra solo

> **Como** operador, **quiero** que el sistema compare lo que predijo contra lo que pasó, retire lo que se degradó y devuelva lo aprendido a la fase de generación, **para** tener una fábrica que mejora mientras yo duermo.

**El problema.** Una estrategia rentable no lo es para siempre. Los regímenes de mercado cambian y el borde se erosiona. La pregunta operativa no es *"¿funciona?"* sino *"¿sigue funcionando, y cómo lo sabré antes de que me cueste caro?"*. Y la pregunta de investigación es aún más valiosa: *"¿qué me enseña este fracaso sobre lo que debo dejar de generar?"*.

**La solución.** Un módulo de retroalimentación que reconcilia diariamente la realidad cruda del bróker contra lo que el sistema creía, detecta anomalías, emite veredicto de continuidad por la prueba de Pardo, y —la parte que la mayoría de las herramientas no tiene— **convierte el fracaso en restricciones de aprendizaje que viajan de vuelta al generador**. Junto a él, un módulo de retiro que archiva la estrategia degradada con su historial completo, en pausa reversible, no en borrado. Y sobre ambos, daemons de automatización continua y una matriz walk-forward que corre de madrugada revalidando la flota.

**Por qué así.** Porque una herramienta que descubre pero no aprende repite sus errores a escala industrial. El bucle cerrado (`generate → validate → incubate → execute → feedback → generate`) es lo que convierte a Drasus de herramienta en fábrica. El objetivo declarado es treinta días de operación desatendida con auditoría completa y cero intervenciones de emergencia.

**Features:** [`trade-reconciler`](./features/trade-reconciler.md) · [`anomaly-detector`](./features/anomaly-detector.md) · [`performance-monitor`](./features/performance-monitor.md) · [`quantops-daemon`](./features/quantops-daemon.md) · [`trade-analysis-bi-suite`](./features/trade-analysis-bi-suite.md) · [`robust-reporting`](./features/robust-reporting.md) · [`monthly-performance-heatmap`](./features/monthly-performance-heatmap.md) · [`pdf-charts-rendering`](./features/pdf-charts-rendering.md) · [`run-replay`](./features/run-replay.md) · [`interactive-stress-lab`](./features/interactive-stress-lab.md)

**Módulos:** `feedback` + `withdraw` · **Fase:** EPIC-7

---

### CAP-09 — Una interfaz donde se ve el razonamiento

> **Como** usuario, **quiero** construir y auditar estrategias en un lienzo visual donde cada decisión del sistema sea inspeccionable, **para** entender por qué el motor concluyó lo que concluyó en vez de creerle.

**El problema.** Las herramientas cuantitativas obligan a elegir entre dos malas opciones: una interfaz que oculta todo detrás de botones (y no puedes auditar nada) o un cuaderno de código donde todo es posible y nada es navegable. Y el problema más grave: cuando el motor genético produce un árbol de reglas de cuarenta nodos, nadie lo entiende — ni quien lo generó.

**La solución.** Un lienzo unificado con nodos-tarjeta de puertos tipados, donde cada feature es una pieza conectable y cada módulo es solo una plantilla de cableado recomendada — no una jaula. Sobre él: paneles inspectores por feature, visualizadores de alta densidad (coordenadas paralelas para optimizaciones de más de veinte dimensiones, proyección UMAP del espacio de robustez, filtrado cruzado coordinado), y dos capacidades de auditoría temporal: **vista en vivo** (observar una estrategia operando mientras se simula, con sus indicadores, señales y stops) y **repetición de corrida** (rebobinar una corrida terminada, plantar el cabezal en cualquier instante y ver el estado exacto reconstruido).

Y la pieza que cierra el problema de la caja negra: el **traductor de caja de cristal** y la **auto-explicación de estrategia**, que convierten el árbol de reglas críptico en un párrafo en español legible. Más un copiloto que traduce lenguaje natural a la estructura determinista de reglas — el modelo de lenguaje propone, el motor determinista dispone; nunca al revés.

**Por qué así.** Porque la auditabilidad no es una función de la interfaz, es la razón de ser del producto. Un sistema que te dice "confía en mí" no se distingue de la superstición. Todo renderizado es nativo, sin componentes web incrustados, por rendimiento y por coherencia visual.

**Features:** [`visual-dag-editor`](./features/visual-dag-editor.md) · [`canvas-navigation`](./features/canvas-navigation.md) · [`glass-box-ai-translator`](./features/glass-box-ai-translator.md) · [`strategy-self-explanation`](./features/strategy-self-explanation.md) · [`strategy-ast-copilot`](./features/strategy-ast-copilot.md) · [`parallel-coordinates-visualizer`](./features/parallel-coordinates-visualizer.md) · [`umap-scatter-visualizer`](./features/umap-scatter-visualizer.md) · [`cross-filtering-visualizer`](./features/cross-filtering-visualizer.md) · [`visual-downsampling-service`](./features/visual-downsampling-service.md) · [`binary-arrow-transport`](./features/binary-arrow-transport.md) · [`localization`](./features/localization.md) · [`notification`](./features/notification.md) · [`verification-bench`](./features/verification-bench.md) · [`secure-updater`](./features/secure-updater.md) · [`flutter-packaging-manager`](./features/flutter-packaging-manager.md) · [`infrastructure-setup`](./features/infrastructure-setup.md) · [`clock`](./features/clock.md) · [`audit-log`](./features/audit-log.md) · [`telemetry`](./features/telemetry.md) · [`async-job-executor`](./features/async-job-executor.md) · [`worker-isolation-orchestrator`](./features/worker-isolation-orchestrator.md) · [`agentic-mcp-gateway`](./features/agentic-mcp-gateway.md)

**Fase:** las cáscaras se entregan feature por feature desde EPIC-0; la unificación es EPIC-8

---

### CAP-10 — Soberanía, identidad y modelo de negocio

> **Como** usuario, **quiero** saber exactamente qué sale de mi máquina y qué no, y **como** proveedor, **quiero** poder cobrar sin convertirme en custodio del trabajo de mis clientes.

**El problema.** El modelo dominante en la industria es una falsa dicotomía: o corres todo local sin poder monetizar ni sincronizar nada, o subes todo a la nube y el proveedor ve tus estrategias, tus resultados y tus posiciones. Y del lado del proveedor, el problema simétrico: instrumentar el negocio después de tener usuarios reales cuesta diez veces más que cimentarlo antes.

**La solución.** Un **modelo de tres planos** con soberanía condicionada por nivel de suscripción:

| Plano | Qué hace | Dónde corre |
|---|---|---|
| **Interfaz** | La aplicación que el usuario ve | Su máquina |
| **Ejecución** | Todo el cómputo: backtests, generación, órdenes | Su hardware (PC o servidor privado) |
| **Cabina de Mando Central** | Autenticación, licencias, facturación, agregación de telemetría. **Nunca computa nada.** | Servidor del proveedor |

La regla de soberanía es explícita y no está escondida en letra chica:

- **Nivel gratuito:** el trabajo del usuario (estrategias, backtests, portafolios, resultados) fluye al proveedor y es suyo por términos de servicio. El usuario paga con datos.
- **Nivel de pago al corriente:** la telemetría de trabajo se **suprime en origen**. Privacidad real, no una promesa de no mirar.
- **Nivel de pago vencido:** la emisión se reactiva. El entorno del usuario nunca se borra.
- **En todos los niveles, sin excepción:** las credenciales del bróker y las direcciones de las máquinas en producción **jamás salen de la máquina del usuario**.

Sobre esa base se cimientan catorce piezas de infraestructura de negocio —identidad central, licencias, catálogo de planes y cuotas, medición de uso, registro de consentimiento versionado, eventos de dominio enriquecidos, motor de reportes institucionales, pasarela pública para terceros, agregación anonimizada de datos, registro de cuentas verificadas con atestación criptográfica, continuidad de instancia entre máquinas, jerarquía de cuentas maestras para fondos, portabilidad de datos conforme a GDPR y roles de operador granulares— construidas **antes** de tener usuarios, como fontanería. Los productos concretos son adaptadores posteriores sobre esos puertos.

**Por qué así.** Es Inundación de Fundaciones aplicada al negocio: *construye la fontanería una vez, vende el agua de mil formas*. Reinstrumentar el consentimiento o la portabilidad de datos bajo un plazo legal de treinta días, con usuarios reales encima, es el escenario que este diseño existe para no vivir.

**Features:** [`central-identity`](./features/central-identity.md) · [`licensing-system`](./features/licensing-system.md) · [`plan-tier-quota`](./features/plan-tier-quota.md) · [`usage-metering`](./features/usage-metering.md) · [`consent-registry`](./features/consent-registry.md) · [`enriched-domain-events`](./features/enriched-domain-events.md) · [`institutional-report-engine`](./features/institutional-report-engine.md) · [`third-party-api-gateway`](./features/third-party-api-gateway.md) · [`data-aggregation`](./features/data-aggregation.md) · [`verified-account-registry`](./features/verified-account-registry.md) · [`instance-continuity`](./features/instance-continuity.md) · [`master-account-hierarchy`](./features/master-account-hierarchy.md) · [`data-portability`](./features/data-portability.md) · [`operator-roles`](./features/operator-roles.md)

**Fase:** Cimientos de Monetización (transversal, antes de EPIC-1 en prioridad) · Cabina de Mando: disparador = primer cobro real, tras EPIC-5

---

## 3. Roadmap público

El orden lo decide un principio único — **Alpha-First**: acortar la distancia entre el código y el dinero real lo antes posible. La unidad de entrega es un módulo completo, no una selección de piezas.

| Fase | Entrega | Qué desbloquea | Capacidades | Estado |
|---|---|---|---|---|
| **EPIC-0** | Fundación | Esqueleto compilable, riesgos técnicos resueltos, primera interfaz | base de `CAP-09` | ✅ cerrada |
| **Cimientos** | Substrato de negocio | Cualquier modelo de monetización sin rehacer el núcleo | `CAP-10` | 🟡 backends cerrados, conexión a interfaz en curso |
| **EPIC-1** | Soberanía de datos | Un lago de datos limpio y auditado | `CAP-01` | 🟡 en curso |
| **EPIC-2** | Motor de backtest | Simulación determinista y confiable | `CAP-02` | pendiente |
| **EPIC-3** | Generación | Miles de candidatas por noche | `CAP-03` | pendiente |
| **EPIC-4** | Guantelete de robustez | Estrategias operables, no solo bonitas | `CAP-04` | pendiente |
| **EPIC-5** | **Primer dinero real** | Estrategia viva en cuenta real | `CAP-05`, `CAP-06` | pendiente |
| **EPIC-6** | Portafolio y ejecución nativa | Flota multi-estrategia con gestión de capital | `CAP-07`, `CAP-06` | pendiente |
| **EPIC-7** | Ciclo cerrado 24/7 | Fábrica autónoma que se mejora sola | `CAP-08` | pendiente |
| **EPIC-8** | Lienzo unificado y pulido | Interfaz completa e instaladores | `CAP-09` | pendiente |
| **EPIC-9+** | Frontera de investigación | Según retorno demostrado | — | pendiente |

**Dependencias que no se negocian:**

1. Sin validación punto-en-el-tiempo (EPIC-1), todo backtest miente.
2. Sin motor confiable (EPIC-2), generar es fabricar sobreajuste a escala.
3. La corrección por pruebas múltiples (EPIC-4) exige contar los intentos desde EPIC-3. Orden inverso = estadística inválida.
4. El dimensionamiento en vivo (EPIC-5) consume el score de robustez de EPIC-4.

### 3.1 Frontera de investigación (no comprometida)

Existen unas cuarenta líneas de investigación documentadas en [`docs/moonshots/`](./moonshots/) — aprendizaje profundo, minería distribuida, mercado de estrategias cifradas, opciones financieras, computación cuántica aplicada a optimización de cartera, entre otras. **No son roadmap.** Son hipótesis con documento de diseño, condicionadas a retorno demostrado. Se comunican como exploración, nunca como promesa.

---

## 4. Nivel Bajo — Índice de mapeo 1:1

Esta tabla es la **fuente de verdad de la asignación feature → capacidad**. La narrativa detallada de cada feature (problema concreto → solución concreta) vive en la sección `## Narrativa de Producto` de su propio documento.

| Capacidad | Módulo(s) | Features | Fase |
|---|---|---|---|
| `CAP-01` — Datos confiables | `ingest` | 22 | EPIC-1 |
| `CAP-02` — Simulador honesto | `validate` (núcleo) | 14 | EPIC-2 |
| `CAP-03` — Búsqueda automática en el espacio de estrategias | `generate` | 21 | EPIC-3 |
| `CAP-04` — Guantelete de robustez | `validate` (guantelete) | 29 | EPIC-4 |
| `CAP-05` — Cuarentena | `incubate` | 5 | EPIC-5 |
| `CAP-06` — Ejecución y defensa | `execute` | 22 | EPIC-5 / EPIC-6 |
| `CAP-07` — Flota y capital | `manage` | 7 | EPIC-6 |
| `CAP-08` — Ciclo cerrado | `feedback` + `withdraw` | 10 | EPIC-7 |
| `CAP-09` — Interfaz auditable | transversal | 22 | EPIC-0 → EPIC-8 |
| `CAP-10` — Soberanía y negocio | transversal | 14 | Cimientos |

La lista nominal de features por capacidad está en el bloque **Features** de cada `CAP-NN` en §2.

---

## 5. Mantenimiento

**Quién es dueño:** el Architect. Este documento y su gemelo simple se editan desde el skill `architect`, nunca desde un skill de implementación.

**Cuándo se toca — disparadores obligatorios:**

- Se crea una feature nueva → recibe su `CAP-NN` en §2 y su sección de narrativa en su propio documento, en la misma edición.
- Una feature cambia de módulo o de fase → se corrige su fila y su capacidad.
- Una decisión arquitectónica altera lo que el producto promete → se corrige la capacidad afectada aquí **y** en `PRODUCT-SIMPLE.md`.
- Una capacidad nueva → siguiente `CAP-NN` libre, en ambos documentos.

**Qué nunca se hace aquí:**

- **Añadir una tercera vía.** Existen exactamente dos: este documento y el llano. **Prohibido crear versiones por profesión, industria o perfil de interlocutor.** Si una explicación solo funciona para un gremio, la analogía está mal elegida: se corrige la analogía, no se añade una variante. La prueba: si conocer a alguien de un oficio nuevo obligara a escribir una sección nueva, el documento ya falló.
- **Usar una etiqueta de gremio como título** al describir a quién sirve algo. Se nombra la conducta de la persona; el término de industria va después, como referencia secundaria.
- Prometer rentabilidad, cifras de rendimiento o resultados esperados.
- Incluir nombres de funciones, tipos, rutas de archivo o fragmentos de código — este documento describe comportamiento observable, no implementación.
- Presentar un moonshot como roadmap comprometido.
- Editar un documento sin su gemelo.
