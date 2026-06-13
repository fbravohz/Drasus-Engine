### **ADR-0001: Monolito Modular + FCIS**

*   **Decisión:** Adoptar un **Monolito Modular** con el patrón **Functional Core / Imperative Shell** (FCIS).
*   **Objetivo:** Lograr el orden de los microservicios con la simplicidad de un único binario (ideal para un solo desarrollador).
*   **Reglas:** Lógica de negocio (Core) pura y determinista, separada de la base de datos e infraestructura (Shell). Prohibido el acceso directo entre bases de datos de distintos módulos.
*   **Implementación:** Límites modulares estrictos mediante interfaces públicas y eventos internos.
*   **Ventajas:** Tests ultra-rápidos, código matemático limpio y facilidad para extraer módulos a microservicios en el futuro si fuera necesario.
*   **Costo:** Disciplina estricta y código adicional para mapeo de datos.
*   **Resultado:** Despliegue de un solo binario con el orden y aislamiento de microservicios.

**Flujos Alternativos (Overrides):**
* **Flujo Acelerado:** Omitir la fase de incubación (donde las estrategias se prueban antes de usar dinero real) y pasar directamente a la fase de validación, gestión y ejecución. Esto requiere que el usuario confirme manualmente. Se usa cuando reutilizamos estrategias probadas anteriormente.
* **Validación de Cartera Completa:** En lugar de validar cada estrategia por separado, validar todas las estrategias del cartera como un grupo antes de ejecutar, considerando cómo interactúan entre sí.
* **Ciclo de Retroalimentación Continua:** La retroalimentación (aprendizaje de resultados pasados) puede iniciar un nuevo ciclo de generación de estrategias, considerando limitaciones del ciclo anterior como cambios en el comportamiento del mercado o anomalías detectadas.

---

### **ADR-0002: Desacoplamiento de Persistencia**

*   **Decisión:** Separar completamente la representación de datos en la lógica de negocio (núcleo) de cómo se almacenan en la base de datos. En el núcleo usamos objetos simples con solo datos (sin comportamiento de base de datos). Los objetos de base de datos permanecen solo en la capa de infraestructura.
*   **Objetivo:** Garantizar que cuando la lógica de negocio funciona con datos, siempre produce resultados consistentes y predecibles, sin ser afectada por comportamientos sorpresa de la base de datos (como cargar datos perezosamente cuando no se espera).
*   **Reglas:** El núcleo lógico nunca interactúa directamente con la base de datos; la conversión de datos acontece únicamente en la capa de persistencia.
*   **Implementación:** Para procesar grandes cantidades de datos del mercado, usar formatos de datos que eviten copiar datos innecesariamente en memoria, manteniendo velocidad sin sacrificar memoria.
*   **Ventaja:** Es posible probar la lógica sin una base de datos; el comportamiento es predecible y puede optimizarse con herramientas de aceleración de cálculos numéricos; garantía de que los datos nunca se corrompen.
*   **Costo:** Requiere escribir código adicional para convertir datos entre el formato de la base de datos y el formato de la lógica de negocio.
*   **Resultado:** Garantía total de que si ejecutas la misma lógica con los mismos datos dos veces, obtienes exactamente los mismos resultados, sin sorpresas de base de datos.

---

### **ADR-0003: Organización de Módulos (FCIS) + Features Reutilizables**

*   **Decisión:** 
    * **8 Módulos independientes** = núcleo de la arquitectura: cada uno responsable de una etapa del pipeline de trading (obtener datos de mercado, generar ideas, validarlas, probarlas en papel, gestionar recursos, ejecutar operaciones, retirar fondos, aprender de resultados).
    * **Componentes reutilizables** = fragmentos de código comunes que múltiples módulos necesitan, ubicados en una sección compartida, accesibles desde cualquier lugar.
    * Separación física estricta entre lógica de negocio pura (que nunca tiene sorpresas) e interacción con el mundo exterior (entrada/salida, manejo de errores).

*   **Objetivo:** Evitar que un solo archivo se convierta en un enjambre de responsabilidades diferentes, forzar que cada parte del código sea independiente, permitir aceleración de cálculos intensivos, y facilitar que código se reutilice sin copiarlo.

*   **Estructura lógica:** El código está organizado en carpetas que representan propósitos diferentes:
    * Una carpeta principal que contiene el punto de entrada y orquesta los 8 módulos
    * Una sección de componentes compartidos (telemetría, tipos de datos, herramientas reutilizables)
    * Una sección con los 8 módulos (obtención de datos, generación, validación, incubación, gestión, ejecución, retiro, retroalimentación)
    * Una sección de infraestructura centralizada (configuración de base de datos, canal de eventos)
    * Una sección de migraciones de base de datos
    * Una sección de pruebas automatizadas
    * Un archivo de configuración de dependencias

*   **Estructura interna de cada Módulo (estructura fija que todos siguen):**
    * Un archivo que define la frontera pública (el único punto de entrada que otros módulos pueden usar)
    * Un archivo con la lógica pura de negocio (sin acceso a base de datos, sin entrada/salida, sin sorpresas)
    * Un archivo que orquesta: coordina la lógica pura, maneja errores, registra actividades
    * Un archivo que maneja persistencia en base de datos (busca y guarda datos del módulo)
    * Un archivo que define la estructura de las tablas en base de datos (propiedad exclusiva del módulo)
    * Un archivo que define qué datos se aceptan como entrada y se producen como salida

*   **Componentes reutilizables (igual estructura que los módulos):**
    * Configuración e infraestructura centralizada (cómo conectarse a la base de datos, canal para que módulos se comuniquen, manejo de conexiones)
    * Telemetría (registro de actividades, métricas de rendimiento, pruebas de velocidad)
    * Otros componentes reutilizables (tipos de datos comunes, herramientas, utilidades)
    * **Cada componente reutilizable TAMBIÉN sigue la misma estructura:** frontera pública, lógica pura, orquestación, persistencia (si aplica), tablas (si aplica), esquemas de entrada/salida.
    * Los módulos acceden a componentes reutilizables únicamente a través de sus puntos de entrada públicos, no directamente a archivos internos.

*   **Principio Universal:**
    * **TODOS los componentes** (los 8 módulos y los componentes reutilizables) siguen exactamente la misma estructura interna.
    * Lo que cambia es el contenido de cada archivo (un módulo podría no necesitar ciertas partes), no la estructura.
    * Consistencia: todos tienen frontera pública, lógica pura, orquestación, e interfaces de entrada/salida.

*   **Reglas de frontera (para mantener módulos independientes):**
    *   Si un archivo crece demasiado, dividirlo en piezas más pequeñas para mantener claridad.
    *   Los módulos NO pueden importar piezas internas uno del otro; solo usan la frontera pública del módulo que necesitan.
    *   Los módulos no pueden combinar datos directamente desde sus bases de datos; si necesita datos de otro módulo, pide al módulo a través de su API pública.
    *   Cada módulo controla completamente sus propias tablas de base de datos; otro módulo accede solo pidiendo al módulo.
    *   La lógica pura de negocio está aislada completamente: sin acceso a base de datos, sin entrada/salida del mundo exterior, sin operaciones que tengan "sorpresas" en diferentes ejecuciones.
    *   Componentes reutilizables son accesibles desde cualquier módulo (es decir, se pueden compartir).

*   **Ventaja:** Escalabilidad infinita, aislamiento perfecto, testeable sin DB, preparado para Rust, features reutilizables sin duplicación.
*   **Costo:** Boilerplate inicial (6 archivos por módulo), mapeo de datos entre boundaries.
*   **Resultado:** Monolito verdaderamente modular, reutilizable, sin acoplamiento circulatorio.

---

### **ADR-0004: Máquina de Estados (FSM)**

*   **Decisión:** Representar todos los estados y eventos de órdenes (órdenes de compra/venta) y posiciones (dinero puesto en mercado) usando números enteros, donde cada número representa un estado específico.
*   **Objetivo:** Garantizar que podamos rastrear cada orden y posición con precisión, que siempre sepa en qué estado está, y que el sistema responda rápidamente (sin demoras).
*   **Reglas:** Los estados posibles están predefinidos y limitados (no infinitos); cada cambio de estado debe registrarse para auditoría; las transiciones entre estados siguen reglas lógicas precisas.
*   **Implementación:** Usar números para representar estados en la lógica de negocio; la conversión de números a descripciones legibles ocurre cuando es necesario mostrar a humanos.
*   **Ventaja:** Acceso extremadamente rápido a la información de estado; compatibilidad con aceleración de cálculos vectorizados; historial completo y auditable.
*   **Costo:** Requiere traducir números a palabras para que los humanos entiendan qué significan.
*   **Resultado:** Cada orden y posición es completamente rastreable, auditable para reguladores, y sin sorpresas de comportamiento.

---

### **ADR-0005: Strategy-Portfolio Git-Like Versioning con DAG**

*   **Decisión:** Usar el mismo enfoque que Git (el sistema de control de versiones que usan programadores) para versionar estrategias y carteras de inversión. Esto significa: cada versión se guarda como un nodo en un árbol, cada nodo sabe de dónde vino, y puedes crear "ramas" (caminos alternativos) sin perder el historial.
*   **Objetivo:** Poder reproducir exactamente cualquier estrategia antigua en el futuro, hacer experimentos en paralelo (A/B testing con dinero real), y tener un registro irrefutable de todo lo que pasó.
*   **Implementación:** 
    * Cada versión se guarda con referencias a su versión anterior (`parent_hash`), nombre de rama, cuándo se creó, y todos los resultados de pruebas acumulados.
    * **Hash-Chain & Inmutabilidad:** Las estrategias son inmutables. Cambios = Nuevos hashes. El linaje se reconstruye via punteros criptográficos.
    * **Herencia de Resultados (Cumulative Results):** Si un cambio es solo de metatada o no afecta los parámetros evaluados por una prueba anterior (valorable via `logic_hash`), la versión hereda automáticamente el resultado (ej: WFA, Monte Carlo) ahorrando costo computacional.
    * Toda la información se guarda en un catálogo inmutable; el historial nunca se borra, garantizando auditoría forense total.
*   **Ventaja:** Puedes reproducir exactamente qué pasó hace 6 meses; puedes intentar ideas nuevas en paralelo sin arriesgar lo que funciona; tienes documentación irrefutable para reguladores.
*   **Costo:** Sistema más complejo; requiere más almacenamiento; cambio de mentalidad (pensar estrategias como "compromisos" como en Git).
*   **Resultado:** Historial completo, reproducible y auditable de cada versión de estrategia, con capacidad de experimentos en vivo sin perder trazabilidad.

---

### **ADR-0006: Migraciones Centralizadas con SQLx Migrator**

*   **Decisión:** Todas las migraciones de base de datos (cambios en estructura de tablas) se administran desde un único lugar centralizado mediante **SQLx Migrator**, compilando los archivos de migración directamente en el ejecutable final.
*   **Objetivo:** Garantizar que todos los ambientes locales y remotos tengan exactamente el mismo esquema de base de datos sin requerir intérpretes externos (como Python/Alembic) ni dependencias de runtime pesadas.
*   **Implementación:**
    *   Una carpeta `/migrations` en la raíz del proyecto contiene archivos `.sql` secuenciales (ej. `0001_init.sql`, `0002_jobs.sql`).
    *   La macro `sqlx::migrate!("./migrations")` embebe físicamente los scripts SQL dentro del binario de Rust durante la compilación.
    *   Durante el ciclo de inicialización del Core (antes de habilitar la comunicación vía FFI con Flutter), el orquestador ejecuta la migración programáticamente en la base de datos SQLite con `sqlx::migrate!().run(&pool).await`.
    *   Una única cadena de cambios lineal (sin bifurcaciones) controlada por timestamps/secuencias.
*   **Reglas Críticas:**
    *   Si un cambio ya se aplicó, su archivo de migración no se edita en caliente; se crea un nuevo archivo de migración secuencial para aplicar la modificación.
    *   Todos los esquemas de tablas definidos por los módulos se unifican bajo este directorio de migraciones centralizado.
    *   Cada migración debe ser determinista e idempotente.
*   **Ventaja:** Cero dependencias de Python (Alembic) en runtime; validación y ejecución de migraciones instantáneas en el arranque del cliente local; esquema inmutable dentro del binario de distribución.
*   **Costo:** Requiere recompilar el binario de Rust si se añade una nueva migración (comportamiento estándar en desarrollo nativo).
*   **Resultado:** Esquema de base de datos local SQLite siempre sincronizado automáticamente al iniciar la aplicación, sin fricción de instalación para el usuario final.

---

### **ADR-0007: Inyección Dinámica de Comportamiento (Feature Router)**

*   **Decisión:** Las variantes de comportamiento (ej: estrategias OLD-SCHOOL vs NEW-ERA, features opcionales) se inyectan dinámicamente mediante un **Feature Router** basado en un Registry centralizado.
*   **Objetivo:** Permitir coexistencia de múltiples variantes de comportamiento sin bifurcar el código; activar/desactivar features vía `config.yaml` sin recompilar.
*   **Mecanismo:** Un componente Registry mapea llaves de configuración a clases/instancias específicas. Módulos consumidores solicitan componentes por interfaz, no por implementación concreta.
*   **Reglas de Validación en Startup:**
    * El sistema carga `config/features.yaml` y busca las clases en el Registry.
    * **Validación de Dependencias:** El Router valida que todas las dependencias de una feature activa estén también activas (ej: `execute` requiere `order-fsm`).
    * Si una dependencia falta o hay un conflicto, el sistema falla rápido (fail-fast) con un error descriptivo.
*   **Ventaja:** Agregar una nueva variante de feature no requiere modificar módulos existentes; experimentación A/B sin bifurcar código.
*   **Costo:** Capa de indirección mínima en el arranque.
*   **Resultado:** Sistema extensible donde las variantes de comportamiento son ciudadanos de primera clase configurables.

---

### **ADR-0008: Configurabilidad Universal (TODO es Parámetro, Excepto Invariantes)**

*   **Decisión:** Cualquier número, umbral, regla o parámetro en la arquitectura (ejemplo: "considerar una estrategia buena si tiene una puntuación de calidad mayor que 2", "máxima pérdida permitida es -30%", "construir 8 módulos") puede cambiarse por el usuario, a menos que se marque explícitamente como FIJO. Solo las decisiones técnicas fundamentales (ejemplo: "guardar precios como números grandes para precisión", "la lógica pura nunca llama a funciones del mundo exterior") son inmutables.
*   **Objetivo:** Dar robustez infinita al sistema. Cada usuario/equipo encuentra su propia "configuración óptima" que nadie más tiene. Módulos intercambiables, reglas ajustables, flujos omitibles.
*   **Reglas:**
    *   Decisiones de negocio (cuando una estrategia es "buena", límites de pérdida, reglas de comercio) se guardan en archivos de configuración que el usuario puede editar
    *   La secuencia de pasos (qué módulos ejecutar, en qué orden, cuáles activar o desactivar) se puede cambiar sin modificar el código
    *   Cada parámetro tiene un valor por defecto razonable, pero el usuario siempre puede cambiar si lo necesita
    *   Lo único que no se puede cambiar: decisiones técnicas fundamentales (qué tipos de datos usar, cómo organizar carpetas, qué patrones de código seguir)
*   **Ventajas:**
    *   Flexibilidad extrema — no hay dos equipos con la misma configuración
    *   A/B testing de configuraciones sin re-deploy
    *   Iteración rápida: cambio un parámetro, pruebo, cambio otro
    *   Usuario no toca código — solo configuración
    *   Escalabilidad: mismo código, infinitas configuraciones
*   **Costo:** Sistema es más complejo (manejo de configuración, validación de parámetros). Necesita buena documentación de qué cada parámetro hace.
*   **Resultado:** Plataforma verdaderamente flexible donde cada estrategia/portfolio/usuario tiene su propia "receta óptima" sin tocarse en la base de código.

---

### **ADR-0009: Interfaz Unificada Strategy-Portfolio (ExecutableContainer)**

*   **Decisión:** Strategy y Portfolio implementan un contrato técnico idéntico. Ambos comparten los mismos campos de datos, permitiendo que módulos como validate, incubate y manage operen con lógica única, sin duplicación.
*   **Objetivo:** Eliminar duplicación de lógica en módulos que procesarían datos idénticos. Permitir que cualquier ExecutableContainer (Strategy o Portfolio) fluya a través del mismo pipeline sin bifurcaciones por tipo/clase.
*   **Campos Compartidos en Contrato:**
    * **config:** Objeto que contiene parámetros de configuración específicos del executable. Para Strategy: comportamiento de la regla de trading (umbrales, períodos, lógica de entrada/salida). Para Portfolio: estructura del portafolio (símbolos incluidos, pesos iniciales, correlaciones esperadas).
    * **rules:** Conjunto dinámico de restricciones y límites aplicables a cualquier executable. Ejemplos: máximo drawdown permitido, mínimo Sharpe ratio requerido, máximo de pérdidas consecutivas, límite de riesgo por trade. Aplica idénticamente a Strategy y Portfolio.
    * **test_results:** Historial acumulativo de todas las pruebas ejecutadas sobre el executable. Cada entrada registra: nombre de la métrica evaluada, valor numérico del resultado, marca de tiempo de ejecución. Se agrega a este historial, nunca se reemplaza (auditoría irreversible).
    * **test_analysis:** Análisis agregado post-validación. Contiene: veredicto final (APROBADO/RECHAZADO/REVISAR), Sharpe ratio, máximo drawdown observado, win rate (porcentaje de trades ganadores), puntuación de robustez (0-100). Este campo es único y se bloquea tras la validación inicial; no puede modificarse.
    * **extracted_constraints:** Restricciones recomendadas extraídas automáticamente durante validación. Ejemplos: "no operar en volatilidad < 10%", "máximo 3 trades por sesión", "desactivar en régimen de trending fuerte". Son sugerencias basadas en análisis, NO hard limits. El usuario puede ignorarlas.
    * **live_results:** Estado actual de ejecución en vivo (papel o real). Contiene: P&L acumulado, cantidad total de trades ejecutados, cantidad de trades ganadores, timestamp del último update, detalles de fills (lista de transacciones individuales con precio, cantidad, timestamp).
*   **Campo Adicional Permitido (Solo Portfolio):**
    * Portfolio incluye un campo extra: `strategy_versions` (mapeo de qué versión específica de cada estrategia está activa en este portfolio). Esto es información de Portfolio, irrelevante para Strategy.
    * Todos los demás campos tienen estructura y semántica idéntica.
*   **Implicación Arquitectónica:**
    * Módulo **validate** recibe un ExecutableContainer sin preguntar qué tipo es. Aplica lógica de validación UNA sola vez.
    * Módulo **incubate** recibe ExecutableContainer y ejecuta forward testing con lógica única.
    * Módulo **manage** recibe ExecutableContainer y puede gestionar tanto estrategias como portafolios con código compartido.
    * Si en futuro necesitamos nuevo procesamiento, se implementa UNA sola vez, no en paralelo para cada tipo.
*   **Ventaja:** Flexibilidad arquitectónica. Mañana, si requerimos validar "Portafolios de Portafolios" o "Portafolios Dinámicos", el patrón ya lo soporta sin refactorizar módulos existentes.
*   **Costo:** Disciplina estricta en el contrato. Los campos DEBEN mantenerse idénticos. Agregar un campo solo a Strategy o solo a Portfolio rompe el patrón y fuerza bifurcación en módulos.
*   **Resultado:** Pipeline unificado donde Strategy y Portfolio son intercambiables desde perspectiva de procesamiento. Cero duplicación de lógica entre módulos, máxima reutilización de código.

---

### **ADR-0010: Reglas Dinámicas (Hard Limits vs Soft Alerts)**

*   **Decisión:** El sistema distingue entre dos tipos de restricciones evaluables en tiempo de ejecución: Hard Limits (límites duros que se ejecutan automáticamente) y Soft Alerts (alertas que notifican al usuario).
*   **Objetivo:** Permitir autonomía operativa del sistema sin sacrificar control del usuario. El sistema puede actuar bajo condiciones críticas sin bloquearse esperando intervención humana, pero toda acción queda auditada y es reversible.
*   **Definiciones:**
    * **Hard Limit:** Una métrica (drawdown, pérdida, sharpe, volatilidad, etc.) cruza un umbral configurado → el sistema EJECUTA una acción automáticamente (cerrar posición, reducir peso, detener trading). Se registra inmediatamente en audit trail. Usuario puede revertir dentro de ventana configurable (ej: 5 minutos).
    * **Soft Alert:** Una métrica cruza un umbral configurado → el sistema NOTIFICA al usuario (dashboard, email, Slack). No ejecuta acción automática. Usuario decide manualmente qué hacer.
*   **Comportamiento en Tiempo Real:**
    * **Usuario online:** Sistema notifica de alerts y hard limits; usuario puede intervenir antes de que hard limit se ejecute.
    * **Usuario offline:** Sistema ejecuta hard limits automáticamente sin esperar. Soft alerts se quedan en cola hasta que usuario se conecte.
    * **Evento crítico (black swan):** Hard limits se ejecutan inmediatamente, priorizando supervivencia del capital sobre intervención del usuario.
*   **Aplicabilidad:**
    * Cada Strategy puede tener su propio conjunto de hard limits y soft alerts.
    * Cada Portfolio puede tener su propio conjunto, independiente de las estrategias individuales.
    * Portfolio rules tienen prioridad sobre strategy rules (si entra en conflicto, portfolio gana).
    * Cualquier métrica es candidata: Sharpe ratio, drawdown, P&L, volatilidad, win rate, correlación, cambio de régimen, etc.
*   **Auditoría y Reversión:**
    * Toda ejecución automática (hard limit) se registra en `live_results` con: timestamp, métrica que cruzó umbral, acción ejecutada, `revertible=true`.
    * Usuario puede ver decisiones automáticas en dashboard y marcarlas como "revertidas" dentro de ventana configurable.
    * Si se revierte, el sistema deshace la acción (ej: reabre posición) y registra la reversión.
*   **Ventaja:** Autonomía real. El sistema puede operar sin intervención humana constante, pero usuario mantiene poder de veto y auditoría completa.
*   **Costo:** Complejidad en evaluación continua de métricas. Requiere infraestructura de monitoring y auditoría en tiempo real. Alto overhead en logging si hay muchas métricas.
*   **Resultado:** Sistema verdaderamente autónomo con control humano como backstop, no bloqueador. Usuario es autoridad final pero no cuello de botella operativo.

---

### **ADR-0011: Operaciones Asincrónicas (Async Job Pattern)**

*   **Decisión:** Las operaciones computacionalmente costosas se ejecutan de forma asincrónica con patrón de tres fases (Disparo → Monitoreo → Recuperación), utilizando **Rust Tokio** y una tabla de persistencia en **SQLite**.
*   **Objetivo:** No bloquear la interfaz de usuario. Mantener la arquitectura "Local-First" evitando brokers pesados como Redis o Celery.
*   **Mecanismo de Persistencia y Recovery:**
    * **Durabilidad:** Los metadatos del job (`QUEUED`, `RUNNING`) y los resultados finales (`COMPLETED`, `FAILED`) se guardan en SQLite.
    * **Auto-Recovery:** Al arrancar el sistema tras un crash/reinicio, el orquestador escanea la tabla de jobs. Los jobs en estado `RUNNING` o `QUEUED` se re-insertan automáticamente en la cola de ejecución nativa de Tokio.
    * **Inmutabilidad:** Una vez un job alcanza un estado terminal, su resultado es inmutable (snapshot de auditoría).
*   **Ventaja:** Interfaz asincrónica robusta con concurrencia real y sin dependencias externas pesadas. Ideal para instalación en máquina local con alta confiabilidad ante fallos.
*   **Costo:** Gestión manual de la cola de jobs en SQLite (en lugar de delegar a brokers externos).
*   **Resultado:** Sistema responsive y no-bloqueante con auditoría y trazabilidad completa, optimizado para uso local con máxima eficiencia de memoria.

---

### **ADR-0012: Arquitectura Multi-Pipeline Paralela (Single Machine Architecture)**

*   **Decisión:** El sistema soporta N pipelines ejecutándose simultáneamente, restringidos a la **misma máquina física** mediante multi-processing y multi-threading gestionados por el orquestador del monolito.
*   **Objetivo:** Maximizar uso de recursos locales garantizando que operaciones de trading en vivo tengan prioridad absoluta.
*   **Mecanismo de Reserva de Recursos (SLA Interno):**
    * **Scenario A:** Los pipelines son instancias dentro del mismo binario (Jobs), no procesos externos.
    * **Capacidad Reservada:** El usuario define en `config/async.yaml` cuántos threads se reservan estrictamente para el pipeline de ejecución (`live_trading_reservation`).
    * **Capacidad de Exploración:** El resto de la capacidad se reparte entre pipelines de generación y validación.
*   **Nota Escalabilidad:** Si en el futuro se requiere escalar a clústeres, se requiere abandonar SQLite por un DB distribuido y añadir un broker de mensajes.
*   **Ventaja:** Simplicidad operativa máxima; rendimiento óptimo al evitar latencia de red. Seguridad de que el backtesting nunca interrumpirá una orden en vivo.
*   **Costo:** Límite físico de la máquina del usuario.
*   **Resultado:** Plataforma flexible para investigación en paralelo optimizada para despliegue local robusto.

---

### **ADR-0013: Selección de Stack Tecnológico (High-Performance Core)**

*   **Decisión:** Stack base del sistema: **Rust**, **Tokio** (Async Runtime), **Flutter** (Frontend nativo), SQLite 3 con WAL, Polars/DuckDB/Arrow (Motores OLAP nativos en Rust), y `SQLx` para migraciones y acceso a datos.
*   **Objetivo:** Seleccionar herramientas que maximicen performance extrema en operaciones críticas, baja latencia, y "local-first deployment" sin sacrificar determinismo en la memoria.
*   **Tecnologías Seleccionadas y Justificación:**
    * **Rust & Tokio:** Lenguaje compilado para el motor central. Elimina el GIL, la serialización pesada de intérpretes interpretados, y ofrece control total de la memoria sin pausas de "Garbage Collection".
    * **NautilusTrader (Integración):** Motor de trading institucional integrado a nivel de sistema para garantizar paridad absoluta entre simulación y operativa real. Se consume exclusivamente mediante los crates Rust nativos del núcleo v2 (sin intérprete Python); el mecanismo, la gobernanza de versiones y el cumplimiento de licencia se rigen por el ADR-0107.
    * **SQLite 3 con WAL (Write-Ahead Log):** Gestión de persistencia OLTP (configuraciones, eventos, ledger) local-first, con velocidad sub-milisegundo y crash recovery garantizado.
    * **DuckDB (Embebido) & Polars (Nativo en Rust):** Motores analíticos (OLAP) para ejecutar consultas y operaciones de dataframes hiperrápidas. Procesamiento Out-of-Core en Parquet y multihilo verdadero (cero sobrecarga de intérprete). Arrow permite intercambio binario Zero-Copy.
    * **Flutter & FFI:** La UI se renderiza con el motor Impeller de Flutter, comunicándose con Rust vía `flutter_rust_bridge` (memoria compartida C-ABI). Supera radicalmente el límite de dibujo en canvas web y serialización JSON.
*   **Alternativas Rechazadas:**
    * **Python / Rust CLI daemon / Numba / Node.js:** Excesiva latencia de serialización, limitantes por el GIL, e inestabilidad de memoria bajo carga masiva. Retirado de la arquitectura de forma permanente.
    * PostgreSQL / ClickHouse: Requieren servidores/contenedores externos. Violan el principio de **Soberanía del Dato** y simplicidad del "Client Zero".
    * Redis: Overkill para infraestructuras locales; Tokio y SQLite+WAL cubren FFI/gRPC/Estado.
*   **Ventaja:** Rendimiento nivel institucional real (Pro-State), eliminando cuellos de botella del stack tecnológico previo.
*   **Costo:** Tiempos de compilación de Rust y rigor estricto del Borrow Checker durante el desarrollo.
*   **Resultado:** Stack 100% nativo que opera en hardware doméstico explotando cada ciclo del CPU.

---

### **ADR-0014: Evolución Incremental de Contratos**

*   **Decisión:** Las nuevas funcionalidades o "features" descubiertas durante el ciclo de vida del proyecto no generan tareas paralelas (ej. TTR-001b). En su lugar, se actualiza el contrato del Puerto (`public_interface.rs`) y se refina el TTR original en la especificación correspondiente.
*   **Objetivo:** Mantener la densidad de información máxima y evitar el crecimiento descontrolado (bloat) de tareas redundantes. Garantizar que un TTR siempre represente el "estado del arte" de ese requisito.
*   **Reglas:**
    * Si una sección del Masterplan revela un detalle nuevo de una feature existente → Actualizar el TTR y el .md actual.
    * Si el cambio afecta la interfaz entre módulos → Actualizar el `public_interface.rs` y `schemas.rs` del módulo.
    * Solo se crean TTRs nuevos si la responsabilidad es funcionalmente independiente de las existentes.
*   **Ventaja:** Trazabilidad perfecta; el desarrollador siempre lee una única fuente de verdad por requisito.
*   **Costo:** Requiere disciplina para editar documentación existente en lugar de simplemente "agregar al final".
*   **Resultado:** Backlog y especificaciones compactas, de alta calidad y siempre actualizadas.

---

### **ADR-0015: Arquitectura de Causalidad y Aprendizaje Cerrado**

*   **Decisión:** Todo módulo del pipeline está obligado a generar un "Rastro de Evidencia" (Evidence Trail) estandarizado. El módulo de Feedback (MOD-07) se define como el **Consumidor Maestro** único de este rastro.
*   **Objetivo:** Permitir que el sistema realice "Autopsias de Rendimiento" automatizadas. Distinguir mediante datos entre varianza estadística (mala racha) y fallo estructural del modelo (Drift), cerrando el ciclo de Robert Pardo con retroalimentación causal hacia la Generación.
*   **Reglas:**
    * Los módulos deben emitir telemetría de éxito/fallo específica de su fase (ej: latencia de ejecución, calidad de datos de ingesta, intervalos de confianza de validación).
    * La evidencia debe ser inmutable y residir en el `audit-log` o tablas de veredicto.
    * El módulo de Feedback consume esta evidencia para producir los *Learning Constraints* que alimentan un nuevo ciclo en MOD-02.
*   **Ventaja:** Transforma el sistema de un pipeline lineal a una red de aprendizaje que se adapta sin intervención humana manual.
*   **Costo:** Mayor complejidad en el diseño de los Puertos de cada módulo; requerimiento de un esquema de auditoría común y estricto.
*   **Resultado:** Un "Cerebro Central" estadístico (Feedback) con visibilidad total de las causas de degradación en cualquier punto del pipeline.

---

### **ADR-0016: Local-First Processing & External Overlays**

*   **Decisión:** Drasus Engine es **Local-First**. El procesamiento de datos, ejecución de estrategias, base de datos y backend residen en la infraestructura local del usuario o su VPS personal.
*   **Excepciones (Overlays):** Se permite el uso de servicios externos (Cloud) exclusivamente para:
    * Autenticación y Gestión de Sesión (Sign-in).
    * Feature Flagging (ej: ConfigCat).
    * Logging/Tracing centralizado opcional.
    * Funcionalidades P2P como Copy Trading.
*   **Justificación:** Evitar costos masivos de infraestructura para el mantenedor del proyecto y garantizar la soberanía de los datos sensibles y el poder de cómputo para el usuario final.

---

### **ADR-0017: Simulación de Alta Fidelidad Institutional**

*   **Decisión:** El motor de simulación (Backtest/Incubate) debe implementar realismo institucional mediante modelos de fidelidad progresiva.
*   **Modos Soportados:**
    * **Real Ticks:** Fidelidad máxima sobre datos históricos reales.
    * **4-ticks OHLC:** Reconstrucción intra-vela de barras de 1M para validación de SL/TP.
    * **1 Minute OHLC:** Procesamiento secuencial de barras de 1 minuto para temporalidades mayores.
    * **Open Prices Only:** Optimización masiva de parámetros iniciales.
*   **Requisitos Obligatorios:**
    * **Bar-Open Alignment:** Ejecución estricta al abrirse una nueva vela (paridad 1:1 vs Real).
    * **Lógica de Settlement (Davey):** Diferenciación entre Settlement Price e histórico de último precio.
    * **Triple Swap:** Simulación del reloj de sesión con lógica de triple swap en Forex.
    * **Penetración de Ticks:** Exigir que el precio atraviese el límite por $X$ ticks para considerar una orden llenada (Pardo).
    * **Warm-up & Gap Handling:** Calentamiento automático de indicadores y políticas de `FillFlat`/`Ignore` para gaps.
    * **Slippage y Comisiones:** Modelado dinámico por activo.
*   **Justificación:** Eliminar el sesgo de "backtest perfecto" que no sobrevive a la realidad del mercado real, garantizando que el sistema sea apto para capital institucional.

---

### **ADR-0018: Taxonomía y Topología del Pipeline (Los 8 Módulos)**

*   **Decisión:** Definir formalmente la identidad, el propósito y la secuencia técnica de los 8 módulos que componen el pipeline de Drasus Engine, estableciendo a **Withdraw** como el estado final (archivo/graveyard) y a **Feedback** como el guardián de aprendizaje continuo.
*   **Objetivo:** Eliminar la ambigüedad sobre las responsabilidades de cada módulo y asegurar que el ciclo de vida de una estrategia refleje una evolución lógica desde los datos crudos hasta el retiro definitivo.

#### **La Secuencia Maestra (Topología):**
`Ingest → Generate → Validate → Incubate → Manage → Execute → Feedback → Withdraw`

#### **Definición de Módulos:**

1.  **MOD-01: Ingest**
    *   *Propósito:* Obtener, limpiar e historizar datos del mundo exterior.
    *   *Responsabilidad:* Transformar ticks/bars en estructuras deterministas y detectar el régimen de mercado inicial.

2.  **MOD-02: Generate**
    *   *Propósito:* Descubrir señales y combinar lógicas para crear candidatos de inversión.
    *   *Responsabilidad:* Evolución genética, regresión simbólica y ensamblado de "Alpha Blueprints".

3.  **MOD-03: Validate**
    *   *Propósito:* Evaluar la robustez de las estrategias candidatas ante datos no vistos.
    *   *Responsabilidad:* Backtesting institucional, análisis Walk-Forward, Monte Carlo y pureza de Alpha.

4.  **MOD-04: Incubate**
    *   *Propósito:* Validar la ejecución en tiempo real sin riesgo financiero.
    *   *Responsabilidad:* Paper trading avanzado y comparativa Pardo (Histórico vs Forward).

5.  **MOD-05: Manage**
    *   *Propósito:* Construir el portafolio y dictar las reglas de gestión de capital.
    *   *Responsabilidad:* Optimización de pesos, rebalanceo y establecimiento de Hard Limits.

6.  **MOD-06: Execute**
    *   *Propósito:* Interactuar con el mercado con precisión quirúrgica.
    *   *Responsabilidad:* Gestión de órdenes (FSM), conectividad con brokers y ejecución de checks pre-trade.

7.  **MOD-07: Feedback**
    *   *Propósito:* Analizar la delta entre lo esperado y lo real en tiempo real y batch.
    *   *Responsabilidad:* Reconciliación, detección de anomalías estructurales y **Veredicto de Continuidad**. Decide el fin de ciclo operativo.

8.  **MOD-08: Withdraw**
    *   *Propósito:* Gestionar el retiro emérito de estrategias y su preservación histórica offline.
    *   *Responsabilidad:* Ejecución de la transición de retiro estratégica, archivo definitivo y liberación de recursos.

#### **Invariantes de Topología:**
*   **Feedback Siempre Activo:** A diferencia de Ingest o Generate que pueden ser batch, Feedback actúa como un supervisor constante de Execute y Manage.
*   **Withdraw como Destino Final:** Una estrategia en Withdraw ha culminado su servicio operativo con honor. Solo puede ser reactivada mediante un proceso de "Refine" que la devuelva a Generate con nuevos aprendizajes.
*   **Separación de Poderes:** Feedback *detecta y decide*; Withdraw *ejecuta el retiro y archiva*.

#### **Ventajas:**
*   Trazabilidad absoluta: Sabemos exactamente por qué una estrategia culminó su ciclo operativo gracias al rastro dejado por MOD-07.
*   Orden lógico: Refleja el flujo natural de capital y conocimiento.

#### **Resultado:**
Un pipeline unificado donde el aprendizaje (Feedback) precede al olvido sistemático (Withdraw).

---

### **ADR-0019: Interoperabilidad Frontend-Backend (FFI/gRPC)**

*   **Decisión:** Diseñar la comunicación entre la lógica de negocio puramente en Rust (Backend) y la Interfaz de Usuario en Flutter (Frontend) utilizando estrictamente el modelo de Comandos FFI/gRPC (`flutter_rust_bridge`).
*   **Justificación:** Al desechar Rust CLI daemon y el paradigma de red HTTP, necesitamos una vía rápida, binaria y asíncrona. Flutter FFI emite y recibe datos directamente en la memoria sin latencia de enrutadores locales.
*   **Pilares de Interoperabilidad:**
    * **Apache Arrow / Parquet vía FFI/gRPC:** Uso del formato Arrow para pasar "arrays" masivos de datos (velas, métricas) directamente de Rust a Flutter de manera *Zero-Copy*.
    * **Structs Tipados a Dart:** Uso de `flutter_rust_bridge` para autogenerar bindings Dart directamente desde los `structs` de Rust. Un contrato roto no compila.
    * **Contratos Desacoplados:** Flutter no contiene ninguna lógica de procesamiento de trading, actúa exclusivamente como presentador. Cualquier evento que modifique el estado es un Comando de Rust.
*   **Ventaja:** Consistencia bit-a-bit de la capa base hasta el navegador y rendimiento inigualable para la visualización de grandes datos financieros.

---

### **ADR-0020 V2: Principio de Inundación de Fundaciones V2 (Foundation Inundation)**

*   **Decisión:** Inyectar "anclajes" técnicos y campos de base de datos de fases avanzadas (ej: Metadatos para Dashboard, Event-Sourcing para Nautilus) en las etapas iniciales de desarrollo de cada módulo.
*   **Justificación:** Evitar el retrabajo masivo y las migraciones de base de datos dolorosas cuando se implementen funcionalidades complejas en el futuro. Permite que el código inicial sea "consciente" de los requerimientos futuros.
*   **Contrato Global de Persistencia (El Set Maestro de 25 Campos):**
    *   **I. Identidad & Integridad:** `id` (UUID), `created_at` (Nanosegundos), `updated_at`, `audit_hash` (SHA-256), `audit_chain_hash` (Blockchain-lite link), `event_sequence_id` (Recovery sequence).
    *   **II. Soberanía & Propiedad:** `owner_id` (Dueño capital/IP), `institutional_tag` (Environment), `manifest_id` (Design Contract), `access_token_id` (Auth Tracking).
    *   **III. Linaje Alpha & Datos:** `version_node_id` (DAG Link), `parent_id` (Puntero Genético), `logic_hash` (Commit Código/Binario), `data_snapshot_id` (PIT Market Snapshot), `transformation_id` (ID del paso/tipo de transformación aplicado; auditable, p. ej. Raw vs Synthetic vs derivado).
    *   **IV. Infraestructura & Ops:** `process_id` (Job Anchor), `session_id` (Runtime Grouping), `node_id` (Hardware ID).
    *   **V. Forense & Ejecución:** `portfolio_container_id` (Governance), `compliance_status_id` (Veredicto Riesgo), `risk_audit_id` (Ticket detallado riesgo), `indicator_state_hash` (Technical Snapshot), `execution_latency_ms`, `source_signal_id` (Signal link), `signature_hash` (HMAC signals).


*   **Aplicación (Contrato Lógico, NO molde físico de 25 columnas):**
    * **Vocabulario lógico obligatorio:** Los 25 campos son un **contrato lógico** (vocabulario canónico de gobernanza), no 25 columnas calcadas en cada tabla. Definen *qué nombre y semántica* tiene cada anclaje cuando una entidad lo requiere.
    * **Grupo I universal:** El grupo **I. Identidad & Integridad** (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) es **universal**: aparece en toda tabla sin excepción.
    * **Resto por Filtro de Relevancia por Perfil (Tabla Canónica):** Los grupos II–V se inyectan de forma **selectiva según el Perfil Técnico** de la Feature. PROHIBIDO copy-paste masivo de los 25 campos en una tabla, módulo o documento. La tabla siguiente es la **fuente única de verdad** del filtro: `architect/SKILL.md` y `TEMPLATES.md` DEBEN referenciarla por nombre de perfil, NUNCA redefinirla con su propia lista (evita que ambos documentos diverjan entre sí).

        | Perfil Técnico | Cuándo aplica (qué tipo de Feature) | Grupos que se inyectan (además del Grupo I, siempre universal) |
        |---|---|---|
        | **A. Datos / Ingest** | Adquisición, limpieza, normalización o transformación de datos de mercado | III (Linaje Alpha & Datos, incl. `parent_id` cuando hay linaje jerárquico padre-hijo) + IV (Infraestructura & Ops) |
        | **B. IA / R&D** | Generación, optimización o detección basada en modelos / lógica evolutiva | II (Soberanía & Propiedad) + III, subset "Pesos/Arquitectura" (`logic_hash`, `data_snapshot_id`, `indicator_state_hash`, `version_node_id`, `parent_id` cuando hay linaje jerárquico DAG/herencia) + IV |
        | **C. Ops / Hot-Path** | Ruta crítica de ejecución, objetivo de latencia ≤1ms | II + IV + V, subset "Latencia" (`execution_latency_ms`, `source_signal_id`) + V, subset "Gobernanza" cuando aplica (`portfolio_container_id`, `compliance_status_id`) |
        | **D. Ops / Auditoría** | Registro forense, cumplimiento o reconciliación | II + IV + V, subset "Gobernanza/Cumplimiento" cuando aplica (`portfolio_container_id`, `compliance_status_id`, `risk_audit_id`, `signature_hash`) |

        Dentro de cada grupo asignado, la Feature toma solo los campos concretos que tienen sentido para lo que esa tabla representa — no el grupo completo (ver ejemplos ya aplicados en `features/adaptive-volume-indicators.md` Perfil B, `features/broker-connector.md` Perfil C, `features/audit-log.md` Perfil D). Si una Feature combina rasgos de más de un perfil, lo documenta explícitamente en su propia sección "Contrato de Persistencia" — no se crea un quinto perfil ad-hoc sin pasar por el Mecanismo de Mantenimiento (más abajo).
    * **Materialización en EPIC-0:** En Épica 0 el contrato se materializa como **tabla ancla de referencia** (`foundation_master_fields`, migración 0001), no como 25 columnas replicadas. Las features posteriores definen sus campos propios + los campos base que su perfil exige.
    * **Coherencia con el SAD:** Esto realiza el principio **"esquema Distribuidor y Basado en Requisitos"** del SAD: cada Feature define su propio contrato de persistencia, pero todas obedecen el **Contrato Global**.
*   **Mecanismo de Mantenimiento y Propagación:**
    1. **Detección:** Si se identifica un campo o requisito técnico repetido en 3+ features, o un requerimiento crítico en documentos de negocio (ej. Masterplan), se eleva a "Fundación Global".
    2. **Actualización de ADR-0020 V2:** Se añade el nuevo campo/hook a este ADR como estándar obligatorio.
    3. **Inundación Retroactiva:** El agente audita TODAS las features y módulos existentes para inyectar el nuevo anclaje en sus secciones de "Gobernanza" y "Contrato de Persistencia".
    4. **Sincronización de Plantillas:** Se actualiza `TEMPLATES.md` para que futuras creaciones nazcan con la nueva fundación.
*   **Registro de Mantenimiento (campos promovidos al filtro por perfil):**
    *   **2026-06-13 — TASK-004, Fase 3 (Architect):** Tres campos transversales detectados en ≥3 features cada uno se habilitan explícitamente en la **tabla canónica de perfiles** (ya existían en el Set Maestro de 25, pero no estaban expuestos en ningún subset de perfil; el conteo del catálogo se mantiene en **25 campos**, no se inventa nada nuevo):
        *   `parent_id` (Grupo III — Linaje Alpha & Datos): linaje jerárquico padre-hijo de DAG / herencia. Pedido por `strategy-versioning`, `incremental-test-engine`, `databank-lake`. Se expone en perfiles **A** y **B** (los que portan Grupo III), cuando la entidad modela una relación padre-hijo. Grupo III es el correcto: el linaje vive ahí junto a `version_node_id` (DAG link).
        *   `portfolio_container_id` (Grupo V — Forense & Ejecución, subset "Gobernanza"): agrupador de portafolio. Pedido por `fit-to-portfolio-search`, `cross-market-validation`, `federated-portfolio`. Se expone en perfiles **C** y **D** (los que portan Grupo V/Gobernanza); las features **B** que lo necesiten lo documentan como híbrido en su Contrato de Persistencia. Grupo V es el correcto: es gobernanza forense de a qué portafolio pertenece una entidad de ejecución/auditoría.
        *   `compliance_status_id` (Grupo V — Forense & Ejecución, subset "Cumplimiento"): estado/veredicto de cumplimiento normativo. Pedido por `toxicity-purifier`, `copy-trading-engine`, `prop-firm-grader`, `multiplatform-execution-bridge`. Se expone en perfiles **C** y **D**. Grupo V es el correcto: es un veredicto de riesgo/cumplimiento, semánticamente forense, no de soberanía (II).
*   **Costo:** Leve sobrecosto inicial de diseño, pero ahorro masivo en mantenimiento y evolución del sistema.

---

### **ADR-0021: Modelo de Decisión Dual (Autopilot con Veto)**

*   **Decisión:** Implementar un modelo donde el **Autopilot** ejecuta acciones automáticas basadas en Hard Rules, pero el usuario retiene soberanía total mediante un **Poder de Veto** y reversión auditada.
*   **Contexto:** Evitar que el sistema se bloquee esperando al usuario en mercados volátiles, pero garantizando que el usuario nunca pierda el control final sobre el capital.
*   **Reglas:**
    * El sistema actúa primero (ej: cierra posición si Drawdown > límite) y pregunta después.
    * Toda acción automática es reversible dentro de una ventana de tiempo predefinida.
    * No existe "escalada automática" de analista a científico; el usuario es el único supervisor.
*   **Ventaja:** Protección 24/7 sin latencia humana; auditoría forense de cada decisión automática.
*   **Costo:** Necesidad de un sistema de notificaciones en tiempo real y UI para reversión rápida.

---

### **ADR-0022: Pipeline No-Lineal (DAG Multiflujal)**

*   **Decisión:** Permitir que el usuario orqueste los módulos en cualquier orden lógico mediante un Grafo Dirigido Acíclico (DAG), en lugar de forzar un pipeline lineal estricto.
*   **Contexto:** Un investigador puede querer ir de `Generate` a `Execute` directamente para pruebas rápidas, o iterar entre `Validate` y `Manage` sin pasar por `Execute`.
*   **Reglas:**
    * El sistema recomienda la "Ruta de Máxima Confianza" (Ingest -> ... -> Feedback), pero no la impone.
    * La responsabilidad del riesgo al saltar fases (ej: omitir Validación) recae 100% en el usuario.
*   **Ventaja:** Flexibilidad total para diferentes perfiles de usuario (PhD Researcher vs Rapid Prototyper).
*   **Costo:** Complejidad en la visualización y validación de la topología del grafo.

---

### **ADR-0023: Dashboard Dinámico vs Arquitectura de Plugins**

*   **Decisión:** Utilizar una configuración dinámica de métricas (`selected_metrics`) en lugar de una arquitectura de plugins pesada para la visualización de resultados.
*   **Contexto:** La flexibilidad de ver diferentes KPIs (Sharpe, DD, Volatilidad) no debe comprometer la simplicidad del core.
*   **Mecanismo:** El backend retorna un diccionario dinámico basado en la selección del usuario. La UI renderiza componentes estándar para cada tipo de dato (float, array, matriz).
*   **Ventaja:** Cero complejidad de "Carga de Plugins"; facilidad para agregar nuevas métricas al sistema.
*   **Costo:** Requiere que la UI sea lo suficientemente genérica para manejar diferentes sets de datos.

---

### **ADR-0024: Reglas Dominantes (Extracted Constraints)**

*   **Decisión:** Las restricciones detectadas en la fase de validación (`test_analysis`) se convierten automáticamente en **Dominant Rules** para la ejecución en vivo si el usuario lo autoriza.
*   **Contexto:** Un backtest puede revelar que la estrategia falla los viernes a las 14:00. Esa anomalía debe poder "inyectarse" como una regla de protección viva sin programarla manualmente.
*   **Reglas:**
    * `portfolio_rules` siempre dominan sobre `strategy_rules` en caso de conflicto.
    * Las reglas extraídas son inmutables en el contexto de esa versión de la estrategia.
*   **Ventaja:** Transferencia inteligente de conocimiento desde la investigación a la producción.

---

### **ADR-0025: Pre-Trade Risk 10-Steps Gate**

*   **Decisión:** Cada orden DEBE pasar por una validación secuencial obligatoria en memoria (10 pasos) antes de que NautilusTrader dispare cualquier orden para evitar entrar si las condiciones de ejecución no son óptimas. Si cualquiera falla, la orden es abortada.
*   **Contexto:** Evitar errores fatales de ejecución (pérdida de liquidez, excesiva correlación, ráfagas de órdenes, violaciones de drawdown).
*   **Mecanismo:**
    1. **Liquidity & Spread Gap Check:** Mide volumen y spread. Bloquea si detecta caída de volumen >60% o spread excesivo.
    2. **Slippage Check:** Valida el precio de señal vs precio actual.
    3. **Position Size Check:** Valida si excede el lotaje máximo permitido para el símbolo o cuenta.
    4. **Portfolio Exposure Check:** Valida si excede el % de capital global asignado por símbolo o sector.
    5. **Correlation Check:** Valida si genera exposición correlacionada excesiva con posiciones abiertas.
    6. **Drawdown Breaker:** Valida si el DD actual > máximo permitido por el Design Manifest.
    7. **Daily Loss Limit Check:** Valida si la pérdida diaria > límite histórico o de la Prop Firm.
    8. **Order Frequency Check:** Limitador de ráfagas de órdenes (Anti-bug/Anti-HFT accidental).
    9. **Margin Check:** Valida si hay suficiente margen según las reglas del bróker.
    10. **Final Operational Approval:** Veredicto final del orquestador de ejecución.
*   **Ventaja:** Protección robusta multicapa; reducción drástica de "blow-up" técnico.
*   **Costo:** Latencia mínima en la ejecución (objetivo <1ms total).

---

### **ADR-0026: Shadow Watchdog & Heartbeat**

*   **Decisión:** Implementar un proceso supervisor independiente (Shadow Watchdog) que monitoree la salud de los daemons de ejecución.
*   **Contexto:** Si el proceso principal falla o se congela, las órdenes vivas en el mercado están en riesgo.
*   **Reglas:**
    * Si el latido (heartbeat) desaparece por > 5s → El Watchdog activa el Kill Switch global.
    * Realiza comparaciones de Pardo (Spread Real vs Histórico) para pausar si el mercado se degrada.
*   **Ventaja:** Detección y respuesta rápida ante fallos de hardware o software.
*   **Costo:** Requerimiento de multiprocesamiento para que el Watchdog sea verdaderamente independiente.

---

### **ADR-0027: Event Sourcing & Inventory Reconstruction**

*   **Decisión:** Persistir cada decisión, evento y cambio de estado como un flujo inmutable de eventos (`Event Store`) en SQLite WAL.
*   **Contexto:** En caso de crash completo, el sistema no reconstruye el estado desde una tabla estática, sino reproduciendo los eventos.
*   **Mecanismo:** El `Crash Recovery Protocol` lee el Event Store para reconstruir el inventario actual de órdenes/posiciones en <10s.
*   **Ventaja:** Resiliencia total ante reinicios; auditoría perfecta e irrefutable.
*   **Costo:** Mayor volumen de datos en la base de datos (mitigado por DuckDB/Parquet para histórico).

---

### **ADR-0028: ZUI Fractal Navigation (Orchestrator/Strategy Inspector)**

*   **Decisión:** Adoptar una interfaz Zoomable (ZUI) con 3 niveles de profundidad para la exploración de datos y pipelines.
*   **Contexto:** Los 100K+ candidatos y la complejidad de los DAGs requieren una navegación visual coherente que evite la sobrecarga cognitiva.
*   **Niveles:**
    * **Fleet Command:** Visión macro (Dashboard de orquestación de portafolios, suma vectorial de curvas de balance, y matriz de correlación Pearson vía DuckDB con parpadeo de alertas si Pearson > 0.85).
    * **Orchestrator:** Visión meso (Editor visual de nodos conectables mediante Flutter CustomPainter, layout automático Dagre, validación estricta de aciclicidad en backend Rust con `petgraph`, y bus de eventos Pub/Sub visual con pulsos de luz).
    * **Strategy Inspector:** Visión micro (Inspector de estrategia con gráficos interactivos nativos Flutter CustomPainter/Impeller, visualización de genoma AST y editor de código embebido nativo Flutter con inyección de código evaluado en Rust via Rhai).
*   **Restricciones:** Queda estrictamente prohibido el uso de lenguajes interpretados (Python u otros) para la evaluación de código dinámico o scripts en la interfaz; toda inyección de código se realiza y evalúa en entornos aislados de Rust (Rhai).
*   **Ventaja:** Navegación fluida y natural; permite ver el contexto macro y el detalle micro sin perder el hilo y previene sobredosificación de riesgo en vivo.
*   **Costo:** Complejidad significativa en el desarrollo frontend (Flutter CustomPainter/Canvas).
*   **Trazabilidad:** [`zui-navigation`](./features/zui-navigation.md), [`visual-dag-editor`](./features/visual-dag-editor.md)

---

### **ADR-0029: Patrón Todo en Uno (Rust + Flutter FFI)**

*   **Decisión:** Adoptar una arquitectura "Todo en Uno" acoplada mediante **FFI**, donde el Core (Backend) en Rust se enlaza estáticamente junto a la interfaz (Flutter). Se abandona definitivamente Tauri/WebView y cualquier puente HTTP local; la integración es exclusivamente `flutter_rust_bridge` (FFI).
*   **Objetivo:** Eliminar el uso de puertos locales y latencia de red, garantizando un binario de distribución comercial sumamente rápido, seguro y liviano para Windows (.exe), macOS (.app) y Linux (AppImage).
*   **Justificación:**
    * **Eficiencia Memoria Compartida Máxima:** En lugar de enviar payloads mediante serialización a un Webview DOM, Flutter lee y escribe objetos directamente en la memoria de Rust usando `flutter_rust_bridge` (punteros). Latencia sub-nanosegundo.
    * **Empaquetado Completo:** Rust y Dart se compilan en un archivo binario final nativo sin motores de navegador acoplados, reduciendo significativamente el tamaño del instalador final.
    * **Seguridad y Aislamiento:** Ejecución binaria cerrada en el OS.
    * **Interfaz Premium Nativa (Impeller Engine):** Flutter garantiza 120FPS renderizando millones de partículas al comunicarse directamente con Vulkan/Metal en la GPU, superando abismalmente a un Canvas de navegador embebido.
*   **Ventaja:** Cero latencia en llamadas Frontend-Backend, tamaño de ejecutable diminuto y sensación absoluta de App nativa.
*   **Costo:** Todo estado cruza la frontera Rust↔Dart mediante comandos FFI tipados; en modo headless, vía gRPC.
*   **Trazabilidad:** Integración FFI (rust-dart).
*   **Resultado:** Aplicación de escritorio ultra-rápida, determinista e inyectada con la potencia de Rust desde el primer byte.

---

### **ADR-0030: Persistencia Soberana "Zero-Docker"**

*   **Decisión:** Prohibir el uso de dependencias de red pesadas o bases de datos externas (Redis, ClickHouse, Docker) en la arquitectura base. Toda la persistencia y analítica se realiza mediante motores embebidos (SQLite, DuckDB) y archivos planos (Parquet).
*   **Objetivo:** Garantizar la **Soberanía Total de Datos** y simplificar el despliegue a una instalación de "un solo clic" (Client Zero Protocol).
*   **Estrategia de Datos (Despliegue Local):**
    * **OLTP (SQLite):** Todo el estado transaccional, libro de órdenes, estados de la app y un **Almacén de Eventos (Event Store)** inmutable para auditoría se maneja en SQLite (Modo WAL).
    * **OLAP (DuckDB + Parquet):** Los datos históricos masivos y resultados de investigación se almacenan en archivos Parquet (**Particionado Hive-Style**) y se consultan vía DuckDB (Out-of-Core).
    * **Excepción (SaaSCloudEngine):** El uso de contenedores (Docker/Podman) se permite estrictamente en la fase de Despliegue Headless para VPS masivo, pero NUNCA será un requisito para el usuario que instale el software localmente en su laptop.
*   **Ventaja:** Portabilidad absoluta de datos; el usuario es dueño físico de su historial sin cuotas de nube ni mantenimiento de servidores.
*   **Costo:** Mayor responsabilidad en los backups locales por parte del usuario.
*   **Trazabilidad:** [duckdb-sql-engine.md](./features/duckdb-sql-engine.md).
*   **Resultado:** Infraestructura privada de grado institucional comprimida en una arquitectura local persistente.

---

### **ADR-0031: Inteligencia Artificial Híbrida (Hybrid Genesis Engine)**

*   **Decisión:** Priorizar el uso de **IA Híbrida en un Hybrid Genesis Engine** que combina Regresión Simbólica nativa (como modo del motor genético NSGA-II sobre el AST, ADR-0113 — no PySR), Algoritmos Genéticos (NSGA-II) y, en fase de moonshot, Deep Reinforcement Learning (DRL).
*   **Objetivo:** Garantizar que las señales sean legibles (Ecuaciones) pero con la potencia de descubrimiento de regímenes del DRL.
*   **Reglas:**
    * **Sinergia Híbrida:** El DRL descubre la "Tesis" (Macro); el GA realiza el "Tuning" (Micro).
    * **No-Template Discovery:** No hay hipótesis humanas; el motor ensambla bloques funcionales o aprende de la serie temporal autónomamente.
    * **Compilador AST:** Los grafos de decisión se compilan en Árboles de Sintaxis Abstracta optimizados para matrices vectorizadas (Hardware Accelerated).
    * **Escalera de Cómputo Soberano (ADR-0112):** las tareas numéricas se resuelven primero en CPU con `ndarray`+Rayon (default); se asciende a `candle` (Rust puro, GPU dinámica opcional) solo si un benchmark lo justifica, y a `burn` solo en el moonshot DRL. **`tch-rs`/libtorch quedan erradicados** del árbol de dependencias.
*   **Ventaja:** Auditoría total de la lógica de decisión y resiliencia ante hardware variado, sin romper el binario único (ADR-0029).
*   **Costo:** Necesidad de mantener implementaciones CPU-first (`ndarray`/Rayon) optimizadas para componentes críticos.
*   **Resultado:** Laboratorio de IA potente pero transparente y adaptable.

---

### **ADR-0032: Estándares de Hardware Soberano (Single Machine Sovereignty)**

*   **Decisión:** Optimizar el sistema para hardware comercial de alta gama (Prosumador), rechazando la dependencia OBLIGATORIA de clústeres elásticos o HPC en la nube.
*   **Objetivo:** Permitir que un trader individual posea toda su infraestructura de cálculo en una estación de trabajo local (Sovereign Infrastructure).
*   **Especificaciones Objetivo:**
    * **CPU:** 16+ hilos (Ryzen 9 / Intel i9) para paralelización masiva de backtesting.
    * **RAM:** 32GB-64GB para manejo de datasets Out-of-Core y caché de DuckDB.
    * **GPU (opcional):** 8GB+ VRAM (NVIDIA RTX) como acelerador opcional vía `candle` para cargas de IA/reducción dimensional. El cómputo es CPU-first por defecto (`ndarray`/Rayon, ADR-0112); la GPU nunca es requisito.
*   **Trade-off:** El usuario debe invertir en hardware físico inicial, pero elimina costos operativos (OpEx) recurrentes y garantiza privacidad total del IP.
*   **Resultado:** Autonomía operativa total bajo el "Client Zero Protocol".

---

### **ADR-0033: Arquitectura de Despliegue Trimodal**

*   **Decisión:** Soportar de forma oficial tres modos de despliegue del binario Core (Rust), que modifican dinámicamente cómo se conecta con la UI (Flutter) y cómo administra sus recursos visuales.
*   **Modos Soportados:**
    1.  **LocalPowerUser (Default):** Flutter y Rust operan en el mismo proceso (OS) mediante FFI. Renderizado GPU completo (Impeller). Latencia cero.
    2.  **VpsMonolithic:** Ejecución local instalada dentro de un VPS externo (Windows Server, Linux Desktop). El sistema detecta la falta de GPU o sesión RDP, ajusta una variable global que *apaga shaders y animaciones* en Flutter, reduciendo la carga del CPU host en un 90%.
    3.  **SaaSCloudEngine (Headless CLI):** El motor Rust se compila como un Daemon independiente (sin UI) e incluye los 8 módulos (Ingest a Withdraw). Se ejecuta en un servidor remoto de alto rendimiento (Ej. Ubuntu Server CLI). La UI local de Flutter en la laptop del usuario se conecta al daemon remotamente vía gRPC/Websockets bidireccionales.
*   **Objetivo:** Adaptar el motor algorítmico a las limitaciones del hardware donde se ejecute, separando estrictamente la computación intensiva de la visualización, brindando la máxima comodidad al usuario final (laptop local) mientras exprime el cómputo remoto (VPS).
*   **Implicaciones:** La UI debe ser 100% "State-Driven" y capaz de desconectarse sin que el motor Rust detenga su ciclo operativo de Backtesting/Live Trading.

---

### **ADR-0034: Ingesta Híbrida Soberana (Bulk S3 + API Delta)**

*   **Decisión:** Implementar un pipeline de ingesta dual que prioriza la descarga masiva de archivos comprimidos desde buckets públicos (AWS S3, Binance Vision) para históricos y utiliza APIs REST exclusivamente para sincronizar el "Delta" (el hueco desde el último volcado hasta el minuto actual).
*   **Objetivo:** Saturar el ancho de banda del usuario y reducir el tiempo de carga inicial en un factor de 100x respecto a consultas API secuenciales.

*   **Mecanismo:**
    *   **Bulk Ingest:** Descarga concurrente con cliente HTTP nativo Rust (`reqwest`/`hyper`) de archivos `.zip` o `.csv.gz`.
    *   **API Delta:** Sincronización final vía adaptadores directos de NautilusTrader (Binance, IBKR, Oanda v20).
*   **Ventaja:** Evita bloqueos por *Rate Limits* y garantiza la disponibilidad instantánea de años de histórico.
*   **Costo:** Lógica de deduplicación y resolución de continuidad en la frontera de unión Bulk/Delta.
*   **Trazabilidad:** [sovereign-data-fetcher.md](./features/sovereign-data-fetcher.md).

---

### **ADR-0035: Persistencia en Particionado Hive-Style (Parquet)**

*   **Decisión:** Organizar los archivos Parquet en una estructura de directorios siguiendo el estándar **Hive-Style** (`key=value/`). Ejemplo: `exchange=binance/symbol=BTCUSDT/timeframe=1m/year=2025/month=01/data.parquet`.
*   **Objetivo:** Habilitar el **Partition Pruning** a nivel de sistema de archivos. DuckDB/Polars pueden omitir el escaneo de carpetas completas basándose en los metadatos del path, acelerando las consultas en un factor de 1000x para rangos temporales específicos. Esto es aprovechado directamente por la interfaz temporal (Time-Warp UI) al arrastrar el control de rango de fechas.
*   **Reglas:**
    *   No se permite *hardcoding* de rutas; todas deben derivar de una base configurable vía configuración tipada validada en Rust (Serde) (prefijo `DRASUS_`).
    *   Los nombres de las carpetas deben ser en minúsculas y usar el signo `=` como separador.
*   **Ventaja:** Rendimiento de base de datos distribuida en un sistema de archivos local y latencias inferiores a 200ms al cargar ventanas de transacciones e históricos en la UI.
*   **Costo:** Mayor profundidad de directorios y complejidad en la lógica de escritura inicial (escritores particionados).
*   **Trazabilidad:** [hive-partition-manager.md](./features/hive-partition-manager.md), [time-warp-debugger.md](./features/time-warp-debugger.md).

---

### **ADR-0036: Remuestreo Dinámico Multidimensional (DuckDB)**

*   **Decisión:** Delegar la síntesis de periodicidades arbitrarias (ej. 7m, 21m) a DuckDB mediante consultas SQL vectorizadas directamente sobre los archivos Parquet de 1m o Ticks en disco.
*   **Objetivo:** Eliminar la redundancia de datos. No se guardan archivos físicos para temporalidades > 1m si pueden ser calculadas al vuelo.
*   **Mecanismo:** Uso de `date_trunc` y agregaciones SQL (FIRST, MAX, MIN, LAST, SUM) para generar velas OHLCV dinámicas.
*   **Restricción (Regla de Múltiplos):** Solo se puede remuestrear hacia "arriba" (fuente <= target timeframe). El sistema rechaza intentos de crear granularidad inexistente (ej: 1m desde 1h).
*   **Ventaja:** Flexibilidad total para investigar temporalidades "ruidosas" sin ocupar espacio extra en disco y reducción del volumen de transferencia de datos FFI hacia el frontend al realizar downsampling de curvas directamente en base de datos.
*   **Costo:** Overhead de cómputo en cada consulta (mitigado por DuckDB JIT).
*   **Trazabilidad:** [duckdb-resampler.md](./features/duckdb-resampler.md), [time-warp-debugger.md](./features/time-warp-debugger.md).

---

### **ADR-0037: Protocolo de Calidad "The Sanitizer"**

*   **Decisión:** Implementar un pipeline de limpieza secuencial obligatorio antes de que los datos sean consumidos por NautilusTrader o el motor de backtest.
*   **Objetivo:** Garantizar la integridad institucional de los datos y eliminar el *look-ahead bias*.
*   **Pipeline:** `Raw Data → Delisted Filter → Corporate Events Adjuster → PIT Validator → Clean Data`.
*   **Reglas:**
    *   **Gap Auto-fill:** Interpolación lineal opcional para micro-gaps.
    *   **Alertas Spread:** Alerta automática si el spread > 3 sigma.
    *   **Integridad OHLC:** Validación innegociable de High >= Low y Open/Close dentro de rango.
*   **Ventaja:** Datos de nivel institucional ("Golden Source") listos para auditoría forense.
*   **Costo:** Latencia adicional en la fase de ingesta (compensada por procesamiento Polars).
*   **Trazabilidad:** [data-sanitizer-pipeline.md](./features/data-sanitizer-pipeline.md).

---

### **ADR-0038: Estándar de Nomenclatura Institucional (Sanitización Terminológica)**

*   **Decisión:** Adoptar de forma obligatoria y exclusiva la nomenclatura formal y técnica definida en la topología de módulos (`ingest`, `generate`, `validate`, `incubate`, `manage`, `execute`, `withdraw`, `feedback`).
*   **Objetivo:** Garantizar el rigor técnico e institucional del proyecto, eliminando términos subjetivos o "gamificados" que degradan la autoridad de la infraestructura.
*   **Reglas (Sustituciones Mandatorias):**
    * `Mining Rig` / `Mining` → `Ingest` / `Exploración de Alpha`.
    * `Torture Chamber` / `Sala de Torturas` → `Validate` / `Validación de Robustez`.
    * `Autopsia` → `Feedback` / `Análisis de Causa Raíz`.
    * `ADN` → `Strategy Inspector` / `Genoma`.
    * `Fábrica` → `Orchestrator` / `Pipeline Editor`.
    * `Autopilot` → `Execute` / `Vigilancia Automática`.
*   **Implementación:** Auditoría recursiva en toda la documentación (SAD, ADRs, Features, Módulos y TTRs) para eliminar alias prohibidos.
*   **Ventaja:** Facilidad de incorporación (onboarding) y consistencia absoluta entre código, base de datos y documentación.
*   **Resultado:** Infrastructura soberana con lenguaje técnico unificado y profesional.

---

### **ADR-0039: Infraestructura de Lógica Causal Híbrida (Legacy SQX + Sovereign QF)**

*   **Decisión:** El sistema de generación de estrategias debe soportar la combinación modular de nodos de lógica binaria/difusa (StrategyQuantX) con motores de confianza bayesianos continuos.
*   **Objetivo:** Facilitar la exploración de Alpha permitiendo que estrategias "Old-School" se potencien con métricas de confidencia predictiva modernas.
*   **Regla:** Toda salida de lógica causal debe normalizarse en un `Grado de Confianza de Señal (float 0.0-1.0)`.
*   **Ventaja:** Escalabilidad lógica ilimitada y compatibilidad total con métodos de investigación probados.

---

### **ADR-0040: Disparadores de Señal Metamórficos (Capital-Aware)**

*   **Decisión:** El umbral de disparo de las señales de ejecución es dinámico y condicional al estado del capital y la proximidad a límites de riesgo crítico.
*   **Objetivo:** Proteger el capital institucional aumentando la exigencia de certidumbre en escenarios de drawdowns profundos o límites de Prop Firm.
*   **Parámetros:** El umbral es configurable (ej. 95% en riesgo crítico, 75% con colchón de beneficios).
*   **Resultado:** Comportamiento defensivo automático sin intervención manual del usuario.

---

### **ADR-0041: Arquitectura de Hemisferios de Asimetría Estructural**

*   **Decisión:** Prohibir la simetría direccional forzada. El sistema debe evaluar y optimizar de forma independiente los hemisferios de compra (Largos) y venta (Cortos).
*   **Objetivo:** Capturar la realidad de que los pánicos de mercado y las distribuciones de precio poseen dinámicas de volatilidad y volumen radicalmente distintas.
*   **Implementación:** Los motores de asimetría operan como procesos desacoplados dentro del mismo `Alpha Blueprint`.

---

### **ADR-0042: Arquitectura de Fitness Metamórfico de Estado**
*   **Decisión:** La función de aptitud (Fitness) del motor NSGA-II debe ser dinámica y condicional al estado de la cuenta (*Account Status*).
*   **Objetivo:** Una estrategia optimizada para superar un *Challenge* (agresividad máxima) es letal para una cuenta fondeada (defensa de capital). El sistema debe mutar sus pesos automáticamente.
* **Restricciones:** No se permite el cambio de modo de fitness durante una generación evolutiva activa.
* **Efecto observable:** El sistema prioriza *Profit Factor* en Épica 1, y transmuta hacia estabilidad y defensa de *MAE* en Épica 2.

---

### **ADR-0043: Protocolo de Programación Evolutiva Parcial (WildCards)**
*   **Decisión:** El motor genético operará sobre nodos específicos denominados `wildcard_group` dentro de un AST parcial predefinido por el usuario.
*   **Objetivo:** Permite combinar la intuición técnica humana (fijando filtros de sesión) con la fuerza bruta computacional para descubrir el "Alpha" desconocido.
* **Restricciones:** Los nodos fijos definidos por el usuario son inmutables para el motor genético.
* **Generalización (ADR-0108):** Este protocolo es la instancia fundacional ("Dominio de Señal") del Registro de Dominios Genómicos formalizado en ADR-0108. La gramática Condición→Acción aquí descrita para `wildcard_group` se generaliza a los dominios de Riesgo y Gestión de Posición (ADR-0109), Régimen y Filtro de Entorno (ADR-0110) y Portafolio y Correlación (ADR-0111).
*   **Trazabilidad:** [ast-compiler.md](./features/ast-compiler.md).

---

### **ADR-0044: Framework de Dimensionamiento de Riesgo Multimodal**
*   **Decisión:** Los modelos de dimensionamiento de posición (*Fixed Ratio*, ATR, % Riesgo) serán implementados como una *Feature* transversal consumida por Backtest, Gestión y Ejecución Real.
*   **Objetivo:** Garantiza paridad absoluta (bit-a-bit) entre los tamaños de posición calculados en investigación y los ejecutados por el broker en vivo.
* **Restricciones:** El módulo de ejecución tiene prohibido implementar su propia lógica de cálculo de tamaño.
*   **Trazabilidad:** [precision-sizing-models.md](./features/precision-sizing-models.md).

---

### **ADR-0045: Prop-Firm Compliance Profile (Ley de Cero Hardcoding)**

*   **Decisión:** Todos los umbrales de evaluación de firmas de fondeo (Profit Factor mínimo, Drawdown Diario Máximo, Drawdown Total Máximo, Linealidad R² objetivo) son parámetros configurables gestionados exclusivamente vía configuración tipada validada en Rust (Serde). El sistema no posee valores de firma quemados en código.
*   **Objetivo:** Permitir la parametrización total por firma (FTMO, TopStep, Darwinex, Custom) sin modificar ni recompilar el código. Cumple con la **Ley #4: Absolute Parameterization** (ADR-0008).
*   **Mecanismo:**
    *   Un perfil de firma (`PropFirmComplianceConfig`) define el conjunto completo de umbrales y restricciones de esa entidad.
    *   El `PropFirmGrader` recibe el perfil como dependencia inyectada; no conoce la firma concreta.
    *   El perfil por defecto corresponde a la firma configurada en `settings.firm_type`. El usuario puede sobreescribir cualquier umbral individual sin cambiar el perfil base.
*   **Regla de Nulidad:** Si el `profit_factor` calculado cae por debajo de `profit_factor_threshold`, el veredicto de la estrategia es **RECHAZADA** de forma inmediata y categórica. No existen excepciones programáticas a esta regla; solo el usuario puede revisar el umbral.
*   **Ventaja:** Pivote entre firmas de fondeo en segundos; soporte multi-firma en investigación comparativa.
*   **Costo:** Leve complejidad en la gestión del archivo de configuración del perfil activo.
*   **Trazabilidad:** [prop-firm-grader.md](./features/prop-firm-grader.md).

---

### **ADR-0046: Vector-Time Pruning (Poda Temporal Autónoma)**

*   **Decisión:** El sistema debe ser capaz de detectar, aislar y eliminar automáticamente ventanas temporales recurrentemente tóxicas (ej. Viernes 14:00-15:00) del entorno de simulación y ejecución, sin intervención manual del usuario.
*   **Objetivo:** Convertir la observación estadística pasiva (detectar que los Viernes son malos) en una barrera de defensa activa que prevenga pérdidas sistémicas recurrentes.
*   **Mecanismo (Pipeline de 3 Pasos):**
    1.  **Detección:** El módulo `validate` ejecuta un análisis de Z-Score segmentado por ventana temporal (día de la semana × bloque de hora) sobre la serie de PnL. Cualquier ventana con Z-Score negativo crónico superior al umbral configurable (`time_pruning_z_threshold`) es candidata a poda.
    2.  **Confirmación Estadística:** La ventana candidata debe cumplir el umbral de frecuencia mínima (`time_pruning_min_occurrences`) para ser declarada poda definitiva, evitando falsos positivos por muestras pequeñas.
    3.  **Inyección:** La lista de ventanas prohibidas se serializa y se inyecta al `Environment Wrapper` de NautilusTrader como filtro pre-señal. Cualquier señal generada dentro de una ventana prohibida es descartada silenciosamente antes de llegar a la lógica de ejecución.
*   **Relación con ADR-0024:** ADR-0024 describe la transferencia de "Dominant Rules" desde validación a producción. Este ADR especifica el mecanismo de **generación automática** de esas reglas para la dimensión temporal, no la transferencia en sí.
*   **Parámetros (todos configurables):** `time_pruning_z_threshold`, `time_pruning_min_occurrences`, `time_pruning_granularity` (hora/día/semana).
*   **Ventaja:** Eliminación sistemática de pérdidas recurrentes sin disciplina manual del trader.
*   **Costo:** Requiere un dataset suficientemente amplio para que el Z-Score sea estadísticamente representativo; puede eliminar ventanas con poca historia.
*   **Trazabilidad:** [vector-time-pruning.md](./features/vector-time-pruning.md).

---

### **ADR-0047: Computación Asimétrica de Métricas (Hot-Path vs R&D)**

*   **Decisión:** El cálculo estadístico se bifurca por hardware/motor: `NautilusTrader` (Rust) calcula métricas transaccionales (PnL, MAE/MFE) en la ruta crítica caliente, mientras que el motor vectorizado Polars/Rust y Rust SIMD/Rayon calculan analíticas pesadas (R², Monte Carlo) en la ruta de investigación fría.
*   **Objetivo:** Proteger el loop de descubrimiento genético de asfixiarse por "sopa de métricas", cumpliendo con las Leyes #1 y #5.
*   **Restricciones:** Estrictamente prohibido calcular ratios avanzados (Sharpe, Z-Score, Ulcer) de forma iterativa dentro del evento de vela (`on_bar`) del simulador.
*   **Trazabilidad:** [institutional-metrics.md](./features/institutional-metrics.md).

---

### **ADR-0048: Neutralización Analítica de Beta (Alpha Decoupling)**

*   **Decisión:** El motor de validación aislará el rendimiento inherente de la estrategia (Alpha) del sesgo direccional general del mercado (Beta) mediante técnicas de neutralización en tiempo de simulación y evaluación.
*   **Objetivo:** Prevenir falsos positivos institucionales; garantizar que los resultados de *backtesting* no sean meramente producto de una tendencia general o empuje inercial (Bull/Bear Market).
*   **Mecanismo:** El módulo `validate` compara el rendimiento estadístico contra un índice base (Benchmark). El cálculo del Alpha puro excluye la ganancia esperada si simplemente se hubiera mantenido el Benchmark. 
*   **Restricciones:** El parámetro de activo *benchmark* es obligatorio en la configuración para que la métrica de Alpha Decoupling sea válida.
*   **Trazabilidad:** [alpha-decoupling.md](./features/alpha-decoupling.md).

---

### **ADR-0049: Validación Transversal de Robustez (Cross-Market Validation)**

*   **Decisión:** Exigir que toda estrategia apruebe simulaciones de robustez en una cesta de mercados correlacionados (ej. EURUSD → GBPUSD) antes de considerarse válida para la fase de incubación.
*   **Objetivo:** Demostrar que el Alpha descubierto explota una ineficiencia estructural verdadera, penalizando el sobreajuste (Curve Fitting) a un único activo.
*   **Justificación (Correlacionados vs Descorrelacionados):** Se usan mercados correlacionados porque comparten impulsores macro. Si el algoritmo descubrió Alpha real, debe sobrevivir en su "mercado hermano". Probar en descorrelacionados (ej. Forex vs Índices) fallaría por naturaleza física del activo, no por sobreajuste.
*   **Regla de Nulidad:** Si la estrategia colapsa catastróficamente al iterarse sobre un activo del mismo clúster de correlación (Drawdown excede umbral), se marca como sobreajustada y el veredicto es **RECHAZADA**.
*   **Trazabilidad:** [cross-market-validation.md](./features/cross-market-validation.md).

---

### **ADR-0050: Búsqueda Generativa Diversificada (Fit-to-Portfolio Search)**

*   **Decisión:** Incorporar el estado y conocimiento del portafolio vivo como penalización directa en la función de aptitud evolutiva (*Fitness Function*) para buscar proactivamente estrategias descorrelacionadas.
*   **Objetivo:** Acelerar drásticamente la construcción del portafolio integral. En lugar de filtrar ex post facto, el sistema desecha candidatos genéticos redundantes de inmediato, optimizando recursos computacionales.
*   **Mecanismo:** La presión evolutiva premia a los genomas cuyas curvas de equidad se comportan ortogonalmente a la curva de equidad agregada de la flota operativa.
*   **Justificación (Descorrelación):** A nivel macro de portafolio, se buscan estrategias descorrelacionadas para diluir el riesgo sistémico (Teoría de Markowitz), complementando la validación estricta individual de robustez (ADR-0049).
*   **Regla de Cero Hardcoding:** El umbral de correlación es un parámetro modificable (se recomienda `< 0.3`), cumpliendo con la Ley de Configurabilidad Universal.
*   **Trazabilidad:** [fit-to-portfolio-search.md](./features/fit-to-portfolio-search.md).

---

### **ADR-0051: Determinismo Asistido por LLM (Sovereign AI Wizard)**

*   **Decisión:** Los Modelos de Lenguaje (LLM locales o en la nube) se limitan exclusivamente al rol de traductores semánticos hacia y desde el `Strategy AST` (Abstract Syntax Tree). 
*   **Objetivo:** Mejorar masivamente la experiencia de usuario (UX/DX) sin comprometer el determinismo, la reproducibilidad matemática, ni ceder la lógica a cajas negras o alucinaciones.
*   **Reglas:**
    *   **Prohibición de Lógica Opaca:** Prohibido que un modelo envíe señales o lógicas directamente sin su representación correspondiente en el AST nativo del sistema.
    *   **Copilot Estructural:** La IA instancia, configura y enlaza nodos del sistema ("Fontanería"), permitiendo revisión total del usuario.
    *   **Trazabilidad:** [strategy-ast-copilot.md](./features/strategy-ast-copilot.md), [strategy-self-explanation.md](./features/strategy-self-explanation.md).

---

### **ADR-0052: QuantOps Daemonized Pipelines (Cron CI/CD Autónomo)**

*   **Decisión:** Adoptar un modelo de ejecución asíncrono basado en demonios (Daemons) para la orquestación continua de pipelines de investigación y trading (CI/CD QuantOps), independizando la minería de la UI.
*   **Objetivo:** Permitir flujos autónomos de generación, validación e incubación que operen 24/7 sin involucramiento humano, con encadenamiento automático entre proyectos.
*   **Restricciones:** Los daemons operan bajo el principio *Local-First* o VPS privado, nunca delegando la computación algorítmica a servicios SaaS externos opacos.
*   **Trazabilidad:** [quantops-daemon.md](./features/quantops-daemon.md).

---

### **ADR-0053: Envoltorio de Despliegue y Objetivos SMART**

*   **Decisión:** El AST (Árbol de Sintaxis Abstracta) de una estrategia matemática se encapsula dentro de un **Envoltorio de Despliegue** regido por el **Design Manifest**.
*   **Objetivo:** Separar las reglas de riesgo, metas SMART (Sharpe > 1.5, Max DD) y reglas de negocio del algoritmo matemático puro, garantizando que el cumplimiento del portafolio se evalúe sistemáticamente antes de la promoción.
*   **Regla:** Si la estrategia generada no cumple los objetivos SMART del manifiesto durante las pruebas, es rechazada (Filtro de Calidad) sin importar su rentabilidad neta.
*   **Trazabilidad:** [design-manifest.md](./features/design-manifest.md).

---

### **ADR-0054: Encadenamiento de Proyectos y Conectores Externos**

*   **Decisión:** Permitir el encadenamiento de tareas (`Encadenamiento de Proyectos`) entre diferentes espacios de trabajo (ej. Forex → Crypto) y soportar suspensiones temporales del pipeline para inyectar lógica externa (`Conectores de Scripts Externos`).
*   **Objetivo:** Brindar extensibilidad total al pipeline de Drasus Engine sin modificar el código fuente interno, permitiendo al investigador utilizar modelos de ML (ej. XGBoost) en lenguajes/entornos externos e inyectar el resultado.
*   **Restricciones:** Los scripts externos operan en *sandbox* con tiempo límite de ejecución (Timeout); su fallo detiene el pipeline específico sin afectar al Daemon central.
*   **Trazabilidad:** [quantops-daemon.md](./features/quantops-daemon.md).

---

### **ADR-0055: Separación Databank R&D vs Producción (Semillas vs AST)**

*   **Decisión:** El almacenamiento masivo de estrategias candidatas durante la fase evolutiva (R&D) se restringe a archivos Parquet columnares conteniendo únicamente métricas de rendimiento y una "Semilla de ADN" paramétrica. La base de datos SQLite y la representación JSON AST completa se reservan estrictamente para las estrategias que el usuario promueva manualmente a Producción.
*   **Objetivo:** Eliminar el cuello de botella de I/O y el colapso de RAM al procesar millones de backtests.
*   **Mecanismo:** 
    * El backend persiste millones de semillas paramétricas en particiones Parquet efímeras (`strategies.parquet`). 
    * El frontend visualiza estos datos conectándose vía DuckDB y empaquetado Arrow.
    * Al promover (Clic Mágico), el Orquestador Rust realiza la "Rehidratación": inyecta los valores de la semilla en el esquema base y reconstruye el Grafo Completo (AST) guardándolo en SQLite.
*   **Ventaja:** Permite iteraciones a escala industrial sin saturar los recursos del Host.
*   **Trazabilidad:** [databank-lake.md](./features/databank-lake.md).

---

### **ADR-0056: Portfolio Data Preparation (HMM & Matriz Pearson)**

*   **Decisión:** Institucionalizar la preparación de datos post-generación (Épica 1) delegando la normalización de curvas de equidad, el cálculo de la matriz de correlación de Pearson y la asignación del `regime_label` (vía HMM) como requisitos técnicos invariables antes de la optimización del portafolio (HRP).
*   **Objetivo:** Eliminar la redundancia computacional en fases avanzadas y habilitar una optimización de portafolio adaptativa basada en la segmentación matemática del régimen diario (Trending/Ranging/Crash).
*   **Mecanismo:** El `Data Sanitizer` inyecta el `regime_label` usando un HMM pre-entrenado. Tras el descubrimiento genético, se calcula una matriz de correlación temporal que reside en DuckDB, sirviendo como base inmutable para el módulo `manage`.
*   **Trazabilidad:** [portfolio-data-preparation.md](./features/portfolio-data-preparation.md).

---

### **ADR-0057: Glass-Box AI Translator (Semantic Explainer y AST)**

*   **Decisión:** Implementar un sistema de traducción determinista que convierta los pesos opacos de los modelos DRL (Deep Reinforcement Learning) en representaciones audibles mediante Regresión Simbólica **nativa** (modo del motor genético NSGA-II sobre el AST, ADR-0113 — no PySR), Árboles de Sintaxis Abstracta (AST Visual en Flutter CustomPainter) y un **reporte estructurado determinista** en lenguaje natural (ADR-0115; LLM local soberano opcional, nunca Ollama como requisito).
*   **Objetivo:** Destruir el sesgo de "caja negra" intrínseco a las redes neuronales, aumentando la confianza del analista humano y reduciendo el descarte de estrategias rentables por falta de interpretabilidad lógica.
*   **Mecanismo:** La salida de la regresión simbólica nativa alimenta el compilador AST. La topología resultante se decodifica de forma determinista por plantilla en una premisa humana auditable. El humano mantiene la capacidad de "anclar" nodos manuales sobre el output de la IA, creando un grafo de decisión híbrido.
*   **Trazabilidad:** [glass-box-ai-translator.md](./features/glass-box-ai-translator.md).

---

### **ADR-0058: Política de Scoring Ponderado de Robustez y Veredicto en Lenguaje Natural**

*   **Decisión:** Reemplazar el enfoque binario de "Muerte Súbita" (descartar estrategias por fallar un solo test) por un **Scoring Ponderado (0-100)** con 5 factores. Implementar un **Robustness Verdict Engine** basado en LLM local para emitir veredictos en lenguaje natural, identificación de puntos de ruptura y justificación semántica del score.

*   **Objetivo:** El enfoque de descartar estrategias por fallar un solo test genera parálisis por análisis y es estadísticamente ingenuo. Una estrategia puede tener WFA excelente pero Monte Carlo mediocre — matarla por el MC es perder una buena estrategia. El score ponderado permite decisiones granulares y el veredicto en lenguaje natural elimina la caja negra estadística para el trader retail.

*   **Reglas:**
  - Los 5 pesos (WFA 30%, MC Trades 25%, MC Tóxico 20%, CPCV/PBO 15%, Ockham 10%) son configurables por el usuario, pero la suma DEBE ser 100%.
  - El umbral de aprobación (default: 75) es configurable.
  - El Verdict Engine DEBE producir, por defecto y sin dependencia de LLM, un **reporte estructurado determinista** por plantilla (ADR-0115). Un LLM local soberano (vía `candle` embebido) es realce opcional; PROHIBIDO exigir Ollama como requisito y PROHIBIDO depender de APIs externas.
  - El score es inmutable una vez emitido para una versión específica de estrategia.

*   **Implementación:**
  - Tras ejecutar los 5 tests, el `robustness-score-aggregator` calcula el score ponderado final.
  - El `robustness-verdict-engine` toma los 5 resultados + el score final, genera un prompt y consulta al LLM local.
  - La salida incluye: veredicto textual, puntos de ruptura identificados, parámetro más sensible y recomendación.
  - El score determina el dimensionamiento de posición inicial en el módulo de ejecución.

*   **Costo:**
  - Complejidad adicional moderada: motor de scoring ponderado (cálculo simple) + generador de reporte estructurado determinista por plantilla.
  - Cero dependencia de runtimes externos para el Verdict Engine (ADR-0115); el LLM local soberano es opcional.
  - Nuevo contrato entre validate y execute para la transmisión del score → sizing.

*   **Trazabilidad:** [robustness-score-aggregator.md](./features/robustness-score-aggregator.md), [robustness-verdict-engine.md](./features/robustness-verdict-engine.md), [monte-carlo-simulator.md](./features/monte-carlo-simulator.md) (modo dual Trades/Tóxico), [validate.md](./modules/validate.md).

---

### **ADR-0059: Continuous Rolling Walk-Forward Matrix (Matriz Microrodante Nocturna)**

*   **Decisión:** Implementar un sistema de re-optimización diaria (23:59h) basado en una Matriz Walk-Forward de corto plazo (7-14 días), con transferencia inmediata de parámetros vía FFI/gRPC a los Bridges de ejecución.
*   **Objetivo:** Combatir la degradación del Alpha en tiempo real. Eliminar la dependencia de optimizaciones estáticas anuales o trimestrales, permitiendo que el sistema se calibre "vivo" antes de la apertura de los mercados de Londres y Nueva York.
*   **Mecanismo:**
    *   **Daemon de Optimización:** Un proceso persistente intercepta el cierre de mercado diario.
    *   **Ventana Micro-IS:** Escaneo de los pasados 7 o 14 días para identificar el conjunto de parámetros más estable bajo el régimen de volatilidad actual.
    *   **Filtro Cluster Contiguo:** Evaluación geométrica en la matriz; se rechazan configuraciones que no presenten un cluster de 3x3 celdas estables (verdes), mitigando el riesgo de "ruido afortunado".
    *   **Transferencia FFI/gRPC:** Los nuevos parámetros se inyectan en caliente a los Bridges de ejecución (Bridges preapertura) sin reiniciar el motor de trading.
*   **Regla de Seguridad:** Si la optimización nocturna no alcanza el `WFE_Threshold` o no encuentra un cluster 3x3 estable, el sistema mantiene los parámetros del día anterior y emite una alerta de "Pérdida de Sintonía con el Mercado".
*   **Ventaja:** Adaptación ultrarrápida a regímenes cambiantes; reducción del drawdown por obsolescencia paramétrica.
*   **Costo:** Alta complejidad en la orquestación (daemons + FFI/gRPC); riesgo de sobreajuste local si no se aplica el filtro de cluster.
*   **Trazabilidad:** [walk-forward-analyzer.md](./features/walk-forward-analyzer.md), [quantops-daemon.md](./features/quantops-daemon.md), [execute.md](./modules/execute.md).

---

### **ADR-0060: Tests Incrementales Versionados (Herencia + Delta)**

*   **Decisión:** Implementar un **Incremental Test Engine** como una feature transversal que permite la validación acumulativa y evita el recálculo redundante de pruebas en todo el sistema.
*   **Objetivo:** Maximizar la eficiencia computacional en todas las fases de validación de robustez. Permitir que cualquier test (WFA, Monte Carlo, Stress Tests) reutilice cálculos de versiones anteriores si los parámetros base o los datos de origen no han cambiado.
*   **Mecanismo:**
    *   **Identificación Determinista:** Uso de `params_hash` (SHA-256) generado a partir de la configuración completa del test.
    *   **Herencia Transversal:** Los motores de test consultan al `Incremental Test Engine` antes de ejecutar; si existe evidencia previa válida en el linaje de la estrategia, se inyecta el resultado.
    *   **Soberanía del Dato:** La herencia está condicionada a la inmutabilidad del `data_snapshot_id` y el `logic_hash`.
*   **Visualización (Query UI):** Marcado explícito de resultados `HEREDADOS` con referencia al ancestro original.
*   **Ventaja:** Optimización global del guantelete; ahorro masivo de CPU en procesos de refinamiento multi-test.
*   **Costo:** Requiere una orquestación centralizada de hashes y resultados.
*   **Trazabilidad:** [incremental-test-engine.md](./features/incremental-test-engine.md), [walk-forward-analyzer.md](./features/walk-forward-analyzer.md), [strategy-versioning.md](./features/strategy-versioning.md), [validate.md](./modules/validate.md).

---

### **ADR-0061: Motor HPC Monte Carlo Híbrido y Embudo Tóxico de Estrés**

*   **Decisión:** Implementar un motor de simulación de alto rendimiento **CPU-first** (`ndarray` + **Rust SIMD/Rayon**, Multihilo) para el procesamiento matricial masivo y la lógica de mutación dinámica. La permutación de Monte Carlo es barajado de matrices, no deep learning: no requiere GPU ni libtorch (ADR-0112). Una GPU opcional vía `candle` solo se considera si un benchmark demuestra que la CPU no alcanza. Introducir el **Embudo Tóxico de Estrés (Risk-Prop FirmMC)** con cortocircuito de ejecución intradiaria.
*   **Objetivo:** Alcanzar un throughput de 10,000 iteraciones en < 10 segundos para validaciones síncronas. Detectar fragilidad ante límites de cuentas de fondeo (FTMO/Darwinex) mediante la destrucción de cohortes que violen límites diarios absolutos (Drawdown > 4.5% Intradiario) durante la corrida de simulación.
*   **Mecanismo:**
    *   **Paralelización en CPU:** `ndarray` representa las matrices de trades de forma contigua y Rust SIMD/Rayon orquesta los hilos de CPU tanto para las permutaciones veloces como para aplicar las reglas de "Muerte Súbita" intradiaria.
    *   **Short-Circuit Evaluation:** Si una mutación toca el límite diario configurado, se detiene su procesamiento y se marca como `FAILED_COMPLIANCE`, salvando ciclos de cómputo.
    *   **Determinismo:** la ejecución CPU preserva siempre la semilla aleatoria para reproducibilidad bit-a-bit (ADR-0107); si en el futuro se adopta `candle` para una GPU opcional, la ausencia de GPU jamás impide la ejecución.
*   **Ventaja:** Validación institucional de supervivencia en tiempo real; eliminación de estrategias "rentables pero inoperables" en Prop Firms.
*   **Trazabilidad:** [monte-carlo-simulator.md](./features/monte-carlo-simulator.md), [prop-firm-grader.md](./features/prop-firm-grader.md), [SAD.md](./SAD.md).

---

### **ADR-0062: Motor de Robustez Decagonal y Física de Broker (Fricción Realista)**

*   **Decisión:** Estandarizar un motor de 10 perturbaciones obligatorias para la validación de robustez y una capa de simulación de "Broker Physics" para modelar fricciones aleatorias de mercado.
*   **Objetivo:** Descomponer la dependencia de la estrategia ante factores externos y errores de ejecución. Asegurar que el Alpha no provenga de un solo trade afortunado (Outlier Removal) o del orden temporal de los datos (Equity Reshuffling).
*   **Las 10 Perturbaciones:** Trade Reordering, Data Perturbation (métricas), Slippage/Spread Variation, Equity Reshuffling, Skip Random Bars, Break-Even Scenarios, Randomize Inputs, Shock Injection (3.5x ATR), Outlier Removal, y Dynamic MC Position Sizing.
*   **Física de Broker:** Aleatorización de `Min Distance` para órdenes pendientes, rangos variables de `Slippage` y `Spread` para simular iliquidez real.
*   **Mecanismo:** Cada perturbación se aplica como una transformación funcional sobre el stream de trades antes de la agregación de métricas.
*   **Trazabilidad:** [monte-carlo-simulator.md](./features/monte-carlo-simulator.md), [slippage-models.md](./features/slippage-models.md).

---

### **ADR-0063: Protocolo CPCV y Validación PBO (Lopez de Prado Standard)**

*   **Decisión:** Adoptar el protocolo **Combinatorial Purged Cross-Validation (CPCV)** como el motor de validación cruzada definitivo para series temporales, integrando el cálculo de **Probability of Backtest Overfitting (PBO)** como filtro de calidad obligatorio.
*   **Objetivo:** Eliminar el sesgo de selección y el sobreajuste estadístico inherente a la validación cruzada tradicional (K-Fold), garantizando que el Alpha sea estructural y no producto de la minería de datos masiva.
*   **Reglas Técnicas:**
    *   **Purging:** Limpieza obligatoria de $X$ barras antes y después de cada trade en el set de entrenamiento que se solape con el set de prueba.
    *   **Embargo:** Eliminación de $Y$ barras adicionales tras el set de prueba para neutralizar la correlación serial.
    *   **PBO Filter:** Una estrategia con PBO > `PBO_Threshold` (default 0.10) es rechazada automáticamente, independientemente de sus métricas de Sharpe o Profit Factor.
*   **Ventaja:** Rigor institucional que protege el capital contra falsos positivos estadísticos.
*   **Costo:** Elevado consumo de CPU (paralelización vía Rust SIMD/Rayon mandatoria).
*   **Trazabilidad:** [cpcv-analyzer.md](./features/cpcv-analyzer.md).

---

### **ADR-0064: Preservación de Memoria Estadística via Diferenciación Fraccional**

*   **Decisión:** Implementar la **Diferenciación Fraccional (Fractional Differencing)** como técnica de pre-procesamiento de señales para lograr estacionariedad preservando la máxima memoria estadística (correlación a largo plazo) de la serie.
*   **Objetivo:** Superar la limitación de la diferenciación entera ($d=1$) que destruye la capacidad predictiva de los indicadores para lograr que la serie sea apta para modelos estadísticos.
*   **Mecanismo:** Uso de una ventana fija de pesos (Fixed-Window Fractional Differencing) para calcular el valor diferenciado, permitiendo que el sistema aprenda sobre series "Semi-Estacionarias" con alto contenido de Alpha.
*   **Regla:** El parámetro $d$ (grado de diferenciación) es optimizable mediante búsqueda de umbral de ADF (Augmented Dickey-Fuller) para minimizar la pérdida de varianza.
*   **Trazabilidad:** [fractional-differencer.md](./features/fractional-differencer.md).

---

### **ADR-0065: Protocolo de Ablación de Reglas (Simplificación Estructural)**

*   **Decisión:** Implementar un motor de **Ablación de Reglas** (Rule Ablation) como fase obligatoria del guantelete de robustez para toda estrategia con >2 reglas lógicas.
*   **Objetivo:** Eliminar el ruido estadístico y la fragilidad estructural mediante la desactivación sistemática de componentes del AST. Garantizar que cada regla en la estrategia tenga una contribución positiva neta al Alpha.
*   **Mecanismo:** One-Rule-Out Testing con promoción de la variante más simple si el Sharpe Ratio se mantiene (Tolerancia 5%).
*   **Trazabilidad:** [rule-ablation.md](./features/rule-ablation.md).

---

### **ADR-0066: Orquestación en Cascada por Intensidad de Cómputo (Fail-Fast Scalability)**

*   **Decisión:** Institucionalizar una clasificación de **Intensidad de Cómputo** (Compute Intensity) como metadato mandatorio para toda feature de validación y análisis.
*   **Objetivo:** Lograr escalabilidad infinita en el guantelete de robustez sin necesidad de configurar manualmente el orden de ejecución para cada nueva feature.
*   **Categorías de Intensidad:**
    1.  **LIGHT (Épica 0):** Operaciones puramente analíticas sobre metadatos o resultados previos (ej. Ockham, Sharpe, WinRate). Costo CPU: Despreciable.
    2.  **MEDIUM (Épica 1):** Requiere ejecuciones de backtest limitadas o locales (ej. Sensitivity, Rule Ablation). Costo CPU: Moderado.
    3.  **HEAVY (Épica 2):** Requiere procesamiento masivo, paralelización extrema o uso de GPU (ej. Monte Carlo 10K, CPCV, Cross-Market). Costo CPU/GPU: Crítico.
*   **Mecanismo de Cascada (Fail-Fast):**
    *   El **Orquestador de Validación (MOD-03)** escanea dinámicamente las features activas y las ordena por su `ComputeIntensity`.
    *   **Ejecución Secuencial por Bloques:** Se ejecutan todas las LIGHT; si alguna falla el umbral de "Muerte Súbita", se aborta toda la cadena. Luego MEDIUM, luego HEAVY.
    *   **Inyección de Veredicto:** Las features HEAVY solo se disparan si la probabilidad de supervivencia (basada en LIGHT/MEDIUM) justifica el gasto energético.
*   **Ventaja:** Automatización total del ahorro de hardware; escalabilidad para cientos de features sin intervención manual.
*   **Costo:** Disciplina en la categorización de cada nueva feature añadida al ecosistema.

---

### **ADR-0067: Capa de Inferencia Estadística (EBTA)**

*   **Decisión:** Implementar un guantelete de validación estadística avanzada (EBTA) como filtro final en el módulo `validate`, incluyendo Deflated Sharpe Ratio (DSR), Romano-Wolf, Market Detrender y Logic Inversion.
*   **Objetivo:** Para neutralizar el sesgo de minería de datos (*Selection Bias*) y la ilusión de Alpha generada por tendencias alcistas de mercado (Beta). Garantiza que las estrategias aprobadas poseen una ventaja estadística real y no son producto de la suerte o el volumen de pruebas.
*   **Reglas:** 
    - El DSR requiere el registro exacto del número de intentos ($N$) de la sesión de origen.
    - Romano-Wolf debe ejecutarse sobre bootstrap acelerado (GPU/Rust SIMD-Rayon) para evitar bloqueos del pipeline.
    - Market Detrender es obligatorio para activos con sesgo direccional histórico conocido.
*   **Implementación:** 
    - Una estrategia con Sharpe de 2.0 podría ser deflactada a 1.2 por DSR si se realizaron 10,000 intentos.
    - El sistema rechaza automáticamente estrategias que pierden dinero en el escenario "Detrended".
    - El reporte final de robustez incluye el p-value ajustado por Romano-Wolf.
*   **Costo:** Incremento en el tiempo de validación "HEAVY" debido al uso de bootstrap masivo. Requiere infraestructura de rastreo de intentos ($N$) en el módulo de generación.
*   **Trazabilidad:** [`statistical-inference-ebta.md`](./features/statistical-inference-ebta.md), [`dsr-tracking-engine.md`](./features/dsr-tracking-engine.md).

---

### **ADR-0068: Certificación de Estabilización de Volatilidad (Target Vol)**

*   **Decisión:** Implementar una certificación obligatoria de **Target Vol** antes de la aprobación de cualquier estrategia. La estrategia debe demostrar estabilidad de riesgo bajo diferentes regímenes de volatilidad.
*   **Objetivo:** Las estrategias suelen colapsar cuando la volatilidad del mercado cambia bruscamente. El escalado dinámico por Target Vol normaliza el riesgo, permitiendo que la estrategia opere con la misma "presión" estadística sin importar el ruido del mercado.
*   **Reglas:** 
    - No se permite la aprobación de estrategias que presenten desviaciones de volatilidad realizada > 30% respecto al target en el set de prueba.
    - El cálculo de volatilidad debe ser determinista y coincidir bit-a-bit entre simulación y real.
*   **Implementación:** 
    - El sistema escala el tamaño de la posición inversamente a la volatilidad realizada para mantener un riesgo constante (ej. 10% anualizado).
    - Se añade un "Sello de Certificación Vol" en el reporte de robustez.
*   **Costo:** Requiere cálculos continuos de volatilidad (ATR/Desviación Estándar) en la ruta crítica, aumentando levemente la latencia de ejecución.
*   **Trazabilidad:** [`volatility-stabilization.md`](./features/volatility-stabilization.md).

---

### **ADR-0069: Modelado de Fricción Institucional (Adverse Selection)**

*   **Decisión:** Integrar un motor de modelado de **Adverse Selection** y **Probabilistic Fills** en el simulador. El sistema asume estadísticamente que un porcentaje de las órdenes Límite a favor NO se llenarán (Limit Order Drop-Out) y aplica una inversión de fricción (Friction Inversion) en escenarios de Mean-Reverting.
*   **Objetivo:** Los backtests retail son excesivamente optimistas al asumir llenado 100% si el precio toca el límite. En mercados reales, especialmente en alta frecuencia o reversión a la media, el mercado suele "tocar y rebotar" sin dar liquidez a tu orden (Adverse Selection).
*   **Reglas:** 
    - El "Fill Rate" nunca puede ser asumido como 100% para estrategias de microestructura.
    - El peor escenario estadístico de ejecución (60% fill rate en BBO) debe ser el estándar de estrés.
*   **Implementación:** 
    - El simulador descarta aleatoriamente un % de trades que en un backtest normal habrían sido ganadores.
    - Las métricas de rentabilidad se ajustan a la realidad de la cola de ejecución.
*   **Costo:** Mayor pesadez en el motor de simulación (procesamiento probabilístico trade-a-trade).
*   **Trazabilidad:** [`institutional-friction-modeling.md`](./features/institutional-friction-modeling.md).

---

### **ADR-0070: Monitoreo de Seguridad Operativa (Pardo Profile & SSL)**

*   **Decisión:** Implementar un sistema de vigilancia dual: **Pardo Profile Monitor** para detectar desviaciones de métricas en vivo vs histórico, y un **Strategy Stop-Loss (SSL)** mandatorio basado en el factor de seguridad del drawdown máximo histórico.
*   **Objetivo:** Para evitar el "blow-up" catastrófico. Una estrategia que funcionó en el pasado puede romperse por cambios estructurales. El sistema debe vetar la operativa si el comportamiento real (Win%, Avg Win/Loss) se desvía >50% del perfil histórico o si el drawdown vivo supera el límite estadístico.
*   **Reglas:** 
    - El SSL es un **Hard Limit** (ADR-0010); se ejecuta sin preguntar.
    - La desviación de métricas de Pardo genera una alerta inmediata y suspensión preventiva.
*   **Implementación:** 
    - Si `Live DD > HistMaxDD * 1.5`, el sistema cierra todas las posiciones y desactiva la estrategia.
    - Un panel de "Salud de Perfil" muestra el drift de Win% en tiempo real.
*   **Costo:** Necesidad de persistencia de perfiles de métricas por versión de estrategia y cálculo síncrono en cada trade.
*   **Trazabilidad:** [`operational-safety-monitor.md`](./features/operational-safety-monitor.md).

---

### **ADR-0071: Filtrado y Proyecciones Multidimensionales de Optimizaciones**

*   **Decisión:** Adoptar técnicas de proyección y filtrado dimensional (Parallel Coordinates, Cross-Filtering y UMAP/t-SNE) para analizar y evaluar optimizaciones paramétricas de más de 20 dimensiones en lugar de las tradicionales mallas tridimensionales rígidas.
*   **Objetivo:** Las optimizaciones masivas colapsan visualmente cuando se intenta modelar más de 3 dimensiones simultáneamente. El usuario necesita aislar las zonas paramétricas más robustas mediante brushing dinámico en coordenadas paralelas y condicionar las distribuciones en tiempo real usando vistas coordinadas.
*   **Reglas:** 
    - No se permite procesar el filtrado masivo de miles de backtests en el hilo principal de la UI.
    - Se debe utilizar un servicio de reducción de resolución (downsampling) y extracción rápida mediante DuckDB/Apache Arrow en el backend.
*   **Implementación:** 
    - El usuario "pinta" un rango del eje de una métrica y el visor oculta automáticamente las líneas perdedoras y resalta los clústeres ganadores estables.
    - Los histogramas de otros parámetros se re-calculan instantáneamente reflejando el subconjunto filtrado.
*   **Costo:** Mayor complejidad en la gestión del estado visual del frontend y mayor consumo de recursos de cómputo en la capa analítica DuckDB.
*   **Trazabilidad:** [`parallel-coordinates-visualizer.md`](./features/parallel-coordinates-visualizer.md), [`cross-filtering-visualizer.md`](./features/cross-filtering-visualizer.md), [`ai-dimensionality-suite.md`](./moonshots/ai-dimensionality-suite.md).

---

### **ADR-0072: PCA Toxicity Clustering**

*   **Decisión:** Implementar un módulo de reducción de dimensionalidad (PCA) y clústeres no supervisados (K-Means) en el guantelete de validación para aislar y purgar familias de estrategias tóxicas.
*   **Objetivo:** El filtrado por umbrales estáticos no detecta comportamientos tóxicos ocultos bajo combinaciones de métricas de riesgo. Agrupar las estrategias por sus características latentes permite purgar grupos espurios completos.
*   **Reglas:** 
    - Análisis de 10K estrategias debe resolverse en <15s usando el subproceso de IA en la CPU.
    - Se requiere inmutabilidad en las columnas del databank `toxicity_score` y `cluster_label`.
*   **Trazabilidad:** [`pca-toxicity-analyzer.md`](./features/pca-toxicity-analyzer.md).

---

### **ADR-0073: Adaptive Walk-Forward Analysis Windows**

*   **Decisión:** Implementar ventanas WFA dinámicas basadas en el régimen de mercado (HMM) en lugar de ventanas de tiempo fijas.
*   **Objetivo:** Las ventanas estables en entornos de baja volatilidad fallan catastróficamente al cambiar el régimen. Ajustar el tamaño IS/OOS en tiempo de simulación según el régimen de volatilidad previene el sobreajuste.
*   **Reglas:** 
    - Las ventanas WFA se configuran dinámicamente según la clasificación de estados del régimen `regime_label` (INT).
    - No se permite procesar ventanas adaptativas sin el linaje previo del dataset de regímenes (`market_data` con `regime_label`).
*   **Trazabilidad:** [`hmm-regime-detection.md`](./features/hmm-regime-detection.md).

---

### **ADR-0074: Autoencoder Outlier Detector**

*   **Decisión:** Integrar un detector de anomalías de transacciones mediante un modelo de Autoencoder entrenado en características específicas de las operaciones.
*   **Objetivo:** Prevenir que el proceso de selección y optimización favorezca estrategias cuyo rendimiento sea producto de un puñado de trades extremadamente afortunados (outliers).
*   **Reglas:**
    - El umbral de percentil para la detección de outliers es configurable.
    - Se debe calcular el impacto de los outliers sobre las métricas originales; si supera un umbral configurable, se penaliza el score de fitness de la estrategia.
*   **Trazabilidad:** [`autoencoder-outlier-detector.md`](./features/autoencoder-outlier-detector.md).

---

### **ADR-0075: Dynamic Portfolio Optimization & Walk-Forward Rebalancing**

*   **Decisión:** Implementar la optimización de pesos por HRP (Hierarchical Risk Parity), el backtesting a nivel portafolio y el rebalanceo de parámetros dinámico (Walk-Forward Optimization de parámetros de rebalanceo).
*   **Objetivo:** Mejorar la robustez del portafolio en comparación con Markowitz estático. La optimización Walk-Forward de rebalanceo descubre las frecuencias de rebalanceo, ventanas rodantes y umbrales óptimos tras deducir la fricción realista, previniendo el sobreajuste temporal y mejorando el Sharpe Ratio.
*   **Reglas:**
    - **Inmutabilidad de Políticas:** Toda política de rebalanceo óptima se guarda como un objeto inmutable en el Databank en formato JSON con su correspondiente hash.
    - **Backtesting en Paralelo:** El backtesting de portafolios corre en paralelo N estrategias sincronizadas a nivel de reloj común, aplicando fricción (spreads + comisiones agregadas) al portafolio y no de forma aislada.
    - **Delegación al Backtest Engine:** El backtesting a nivel de portafolio delega la simulación del reloj y el emparejamiento de órdenes al [`backtest-engine.md`](./features/backtest-engine.md), aprovechando el patrón [`ExecutableContainer`](./features/executable-container.md) (ADR-0009).
*   **Trazabilidad:** [`portfolio-optimizer.md`](./features/portfolio-optimizer.md), [`backtest-engine.md`](./features/backtest-engine.md).

---

### **ADR-0076: Direct Promotion & Visual Validation of Portfolios**

*   **Decisión:** Habilitar la promoción directa de estrategias o portafolios externos al módulo `manage` (MOD-05), y requerir validación visual mediante mapas de calor y dendrogramas de correlación.
*   **Objetivo:** Permitir flujos rápidos y bypasses seguros (ADR-0022), minimizando el boilerplate para promover estrategias validadas externamente. El uso de dendrogramas proporciona una justificación visual y matemática del descarte de candidatos redundantes en la asignación HRP.
*   **Reglas:**
    - **Validación Obligatoria pre-Promoción:** Cualquier estrategia promovida directamente debe poseer un `audit_hash` válido de su histórico de retornos antes de aceptarse en `manage`.
*   **Trazabilidad:** [`portfolio-data-preparation.md`](./features/portfolio-data-preparation.md), [`portfolio-rules.md`](./features/portfolio-rules.md).

---

### **ADR-0077: Portfolio Risk Metrics & Git-Like Portfolio Versioning with Clusters**

*   **Decisión:** Implementar el conjunto de métricas de riesgo de portafolio avanzado (Índice Herfindahl, CVaR, Descomposición Estacional), clustering K-Means para descorrelación de clústeres (`hrp_rank`), y el versionado inmutable Git-like de portafolios guardado en `portfolios.parquet`.
*   **Objetivo:** La gestión de riesgos a nivel portafolio requiere evaluar la concentración de activos y el riesgo de cola (CVaR) para evitar pérdidas sistémicas. El versionado tipo Git con un grafo dirigido acíclico (DAG) por ramas (`branches`) permite cambios experimentales de pesos y composiciones sin duplicar información ni corromper el linaje de auditoría.
*   **Reglas:**
    - **Persistencia en Parquet:** Toda composición e historial del portafolio se guarda en `portfolios.parquet` bajo el estándar Hive-Style.
    - **Inmutabilidad:** Toda versión del portafolio (`portfolio_version_hash`) es de solo lectura una vez creada.
*   **Trazabilidad:** [`portfolio-rules.md`](./features/portfolio-rules.md), [`portfolio-optimizer.md`](./features/portfolio-optimizer.md), [`strategy-versioning.md`](./features/strategy-versioning.md).

---

### **ADR-0078: Autopilot Execution & Multiplatform Infrastructure**

*   **Decisión:** Formalizar la infraestructura de ejecución de la Épica 3: "The Autopilot", que incluye un motor de ejecución directa NautilusTrader con paridad out-of-sample exacta, el `multiplatform-execution-bridge` para comunicación de comandos vía WebSockets/REST hacia terminales externas (MetaTrader, NinjaTrader, cTrader) sin exportación de lógica local ni stops, y el `multi-ticket-manager` para gestionar múltiples tickets individuales por estrategia identificados vía signal hash + timestamp.
*   **Objetivo:** Para garantizar que el capital real o las cuentas de fondeo se operen en un entorno blindado de forma multiplataforma. El desacoplamiento protege la lógica contra rastreo y permite romper la limitación de SQX de una única operación a la vez.
*   **Reglas:**
    - **Soberanía del VPS:** No se exportará código de estrategia ni indicadores hacia los receptores externos.
    - **Unicidad de Señales:** Se prohíbe abrir una segunda posición si el `signal_hash` es idéntico al de una posición ya activa en la misma barra.
*   **Trazabilidad:** [`broker-connector.md`](./features/broker-connector.md), [`pre-trade-validator.md`](./features/pre-trade-validator.md), [`volatility-stabilization.md`](./features/volatility-stabilization.md).

---

### **ADR-0079: Rules Wrappers for Portfolios & Universal Rules Injection (Challenge Mode)**

*   **Decisión:** Implementar una capa envolvente de reglas (**Rules Wrappers**) universal para portafolios de una o múltiples estrategias. Este wrapper intercepta señales de compra/venta y valida todas las restricciones configuradas (ej. Drawdown Diario absoluto, Trailing Max Drawdown, News Blackouts, FIFO/Netting) antes de permitir que la orden pase. El **Challenge Mode** opera como un perfil inyectado dentro de este wrapper, pero el sistema permite que las reglas se inyecten de forma genérica para cualquier necesidad operativa.
*   **Objetivo:** Para proporcionar máxima flexibilidad y gestión de capital global, independientemente de la naturaleza de las estrategias individuales. El uso de perfiles de reglas inyectables permite cumplir con las normas de cualquier firma de fondeo o requerimiento de riesgo sin alterar el motor central.
*   **Reglas:**
    - Las reglas globales tienen máxima soberanía sobre las estrategias individuales (ADR-0010).
    - Toda orden interceptada por la capa envolvente debe evaluarse en <10ms.
*   **Trazabilidad:** [`portfolio-rules.md`](./features/portfolio-rules.md), [`prop-firm-grader.md`](./features/prop-firm-grader.md).

---

### **ADR-0080: Order-Priority Queue (Anti-Throttling)**

*   **Decisión:** Implementar una cola inteligente de órdenes basada en prioridades concurrentes con reintento inmediato y backoff exponencial para mitigar el throttling del broker.
*   **Objetivo:** Los exchanges imponen límites estrictos de tasa. Durante congestión o alta volatilidad, las órdenes críticas (Stop Loss) deben transmitirse prioritariamente para evitar liquidaciones o pérdidas catastróficas.
*   **Reglas:**
    - Las órdenes P0 (Stop Loss) tienen prioridad absoluta y omiten cualquier límite de cola.
    - Se requiere el uso de un heap de prioridad concurrente sincronizado en memoria (<1ms).
*   **Trazabilidad:** [`order-priority-queue.md`](./features/order-priority-queue.md).

---

### **ADR-0081: Advanced Trade Management (ATM)**

*   **Decisión:** Integrar lógicas transaccionales base como Grid Trading, Hedging y Trailing Stop Mecánico en la gestión operativa multicapa.
*   **Objetivo:** Proveer lógicas convencionales de control para la micro-gestión básica de operaciones, permitiendo asimilar niveles piramidales y lógicas multi-escala.
*   **Reglas:**
    - El Trailing Stop mecánico debe recalcularse estrictamente barra-a-barra (o tick-by-tick en modo Real Ticks).
    - Los niveles de Grid deben estar precalculados y persistidos para garantizar paridad.
*   **Trazabilidad:** [`advanced-trade-management.md`](./features/advanced-trade-management.md).

---

### **ADR-0082: Micro-Gestión Cinética Institucional**

*   **Decisión:** Implementar un módulo defensivo hostil de micro-gestión que incluye: Micro-Scale Out Mandatorio, Z-Score Trailing Intervencionista y Tapering Logarítmico.
*   **Objetivo:** Para proteger las cuentas maestra o Prop Firms ante reversiones violentas. Evita el rastreo de stops rígidos por parte de brokers C-Book mediante cierres masivos market basados en anomalías estadísticas de PnL vivo.
*   **Reglas:**
    - El Z-Score Trailing requiere cálculo en vivo sobre el PnL de las últimas operaciones abiertas.
    - El Tapering Logarítmico reduce el volumen de operación inmediatamente tras rachas fallidas consecutivas.
*   **Trazabilidad:** [`kinetic-micro-management.md`](./features/kinetic-micro-management.md).

---

### **ADR-0083: Autopilot Dynamic Metrics Engine**
*   **Decisión:** Implementar la interfaz `ModuleMetricsProvider` en el módulo Autopilot (Execute) para exponer métricas en vivo en tiempo real directamente al Dashboard.
*   **Objetivo:** Permitir que la interfaz de usuario consuma métricas en vivo según la selección (`selected_metrics`), eliminando la necesidad de un proveedor centralizado.
*   **Reglas:**
    - El cálculo de volatilidad realizada usa exactamente las últimas 20 barras de la serie temporal activa.
    - Las métricas se exponen como un diccionario plano de tipos estándar.
*   **Trazabilidad:** [`autopilot-metrics-provider.md`](./features/autopilot-metrics-provider.md).

---

### **ADR-0084: Daemons Persistentes y Aislamiento de Núcleo (Core Pinning)**

*   **Decisión:** Implementar la ejecución en vivo mediante hilos de ejecución persistentes (*Daemons*) en Rust Tokio, con afinidad de CPU obligatoria (*Core Pinning*) para el **LiveNode**.
*   **Objetivo:** Garantizar latencia de microsegundos inalterable en la ejecución real, incluso bajo carga masiva de R&D (optimización genética). El aislamiento evita que el recolector de hilos de tareas pesadas interfiera con la ruta crítica de órdenes.
*   **Reglas:**
    - El núcleo reservado debe ser configurado explícitamente en el `SLA de reserva`.
    - Se prohíbe el uso de tareas efímeras (que nacen y mueren) para la gestión del LiveNode.
*   **Trazabilidad:** [`persistent-daemons.md`](./features/persistent-daemons.md).

---

### **ADR-0085: Bus de Datos Pub/Sub Zero-Copy (Multiplexación)**

*   **Decisión:** Utilizar un bus de mensajes en memoria basado en canales de difusión de Rust (`tokio::sync::broadcast`) para la distribución de datos de mercado en tiempo real hacia múltiples agentes.
*   **Objetivo:** Evitar bloqueos de IP en mercados externos al abrir múltiples conexiones para el mismo símbolo. Permite que +50 agentes consuman el mismo flujo de datos con latencia de nanosegundos y cero duplicación de memoria.
*   **Reglas:**
    - Un solo cliente de datos (Nautilus DataClient) por símbolo.
    - Los agentes leen por referencia inmutable, eliminando clonaciones costosas.
*   **Trazabilidad:** [`data-bus-pubsub.md`](./features/data-bus-pubsub.md).

---

### **ADR-0086: Minería Descentralizada de Estrategias (La Colmena)**

*   **Decisión:** Definir y formalizar la arquitectura de incubación para **La Colmena** (Red Descentralizada de Minería de Estrategias). El sistema opera bajo un modelo cliente-servidor descentralizado donde las máquinas de terceros ("Nodos Mineros") ejecutan búsquedas y backtests en segundo plano de forma silenciosa, reportando las mejores candidatas al servidor central de Drasus Engine.
*   **Objetivo:** La búsqueda generativa masiva en universos amplios requiere potencia de cálculo extrema. Crowdsourcing de capacidad de cómputo GPU/CPU de la comunidad permite acelerar el descubrimiento de alpha a gran escala sin necesidad de infraestructura de sites dedicados, ofreciendo incentivos y recompensas justas (regalías de fondos, cobro fijo por flujo o pago por estrategias aptas).
*   **Reglas:**
    - **Sandboxing Estricto:** El código de exploración en la máquina del minero debe correr en un entorno virtual aislado (Wasm / Rust Engine estático con APIs de red/sistema deshabilitadas).
    - **Protección de IP (Propiedad Intelectual):** No se exportará el motor comercial completo de NautilusTrader ni lógica propietaria sensible. El minero solo ejecuta evaluaciones parametrizadas y empaquetadas sin acceso al código fuente.
    - **Prueba de Trabajo Cuantitativa (Proof-of-Quant):** El servidor central de Drasus Engine debe re-verificar de forma aleatoria o determinista rápida los backtests reportados para evitar envíos fraudulentos de resultados falsos.
    - **Inmutabilidad y Trazabilidad:** Cada resultado aceptado en el databank debe incluir el linaje completo, firma de hash del minero, y los campos de inundación de fundaciones que su Perfil Técnico exija (Grupo I universal + Perfil IA/R&D; Filtro de Relevancia, ADR-0020 V2).
*   **Trazabilidad:** [`la-colmena.md`](./moonshots/la-colmena.md).

---

### **ADR-0087: El Guardián (Global Execution Router) & El Centinela (Rust Shadow Watchdog & Kill Switch)**

*   **Decisión:** 
    - **El Guardián:** Implementar un Gestor de Riesgo Centralizado (Global Execution Router) que intercepta todas las órdenes generadas por los agentes individuales de forma pre-trade. Cierra la brecha del estado global aplicando un pipeline secuencial estricto de validación de reglas globales (10 checks tácticos que incluyen Checks de Correlación, Margen, Drawdown, y Reglas de Cuentas de Fondeo).
    - **El Centinela (Shadow Watchdog):** Reemplazar cualquier propuesta de proceso en Python por un daemon o tarea asíncrona independiente escrita 100% en **Rust** (vía `tokio` o worker nativo), garantizando un monitoreo continuo de salud/latencia (<5s) del motor principal, un Shadow Mode de validación paralela de volumen cero, y un Kill Switch atómico de barrido total (`FlattenAll()`) operable remotamente mediante una Emergency PWA.
*   **Objetivo:** 
    - Garantizar que los agentes no puedan interactuar directamente con el exchange sin validar el estado global de riesgo y capital.
    - Cumplir de forma inquebrantable con la restricción de arquitectura exclusiva Rust/Flutter, eliminando la sobrecarga de latencia, dependencias pesadas y fallos de recolección de basura de Python.
    - Ofrecer un bypass de emergencia seguro y un monitoreo reactivo móvil sin exponer la propiedad intelectual de las estrategias.
*   **Reglas:**
    - **Soberanía Absoluta del Guardián:** Ningún agente individual posee claves de API o acceso directo de transmisión al cliente de ejecución de NautilusTrader.
    - **Independencia del Watchdog:** El Shadow Watchdog debe ejecutarse en un proceso o hilo completamente independiente del loop de trading para evitar bloqueos mutuos en caso de crash del motor de órdenes.
    - **Límite de Latencia:** La intercepción y validación del Guardián debe completarse en `<1ms` en el hot path.
*   **Trazabilidad:** [`pre-trade-validator.md`](./features/pre-trade-validator.md), [`system-watchdog.md`](./features/system-watchdog.md), [`execute.md`](./modules/execute.md).

---

### **ADR-0088: Protocolo de Incubación & Cono de Silencio (Sandbox de 7 Días, Proyección de Monte Carlo y Broken Strategy Flag)**

*   **Decisión:**
    Implementar un Protocolo de Incubación y Sandbox avanzado con perfiles de duración configurables y cinco pilares:
    1. **Incubación Prolongada (Legacy Paper Trading, 3-6 meses):** Validación tradicional en demo durante periodos de 3 a 6 meses para certificar resiliencia a largo plazo.
    2. **Sandbox Extendido (Extended Quarantine, 21 días):** Modo intermedio entre la cuarentena acelerada y la incubación prolongada. Aplica el mismo motor de Eutanasia Predictiva y Cono de Silencio que el Sandbox de 7 días, pero sobre una ventana de 21 días, para estrategias que requieren mayor confirmación estadística antes de promoción a capital real.
    3. **Sandbox de 7 Días (Live Quarantine):** Período de cuarentena virtual acelerado con Eutanasia Predictiva en caliente. El motor contrasta el desempeño actual contra una Matriz de Avance Progresivo (WFM) retroactiva. Se ejecuta una purga sintética (eliminando y desactivando el genoma de la estrategia) de inmediato si la equidad real se desvía del Umbral OOS (Fuera de Muestra), excediendo un límite configurable de riesgo flotante (Excursión Máxima Adversa extra de +15% o detención asimilada), protegiendo las cuentas maestras de fondeo antes de incurrir en drawdown colateral real.
    4. **Cono de Silencio (Auditoría Estadística):** Proyección dinámica hacia el futuro de bandas de confianza a 1, 2 y 3 sigmas basadas en simulaciones Monte Carlo previas. Calcula de forma diaria métricas de eficiencia de retorno (`Return Efficiency`) y degradación de drawdown (`Drawdown Efficiency`).
    5. **Broken Strategy Flag (Automated Kill Switch):** Mecanismo de seguridad operativo que pausa la estrategia de manera inmediata si la equidad real sale del Cono de Silencio por el borde inferior (-1 sigma), liquidando posiciones abiertas y reasignando el capital para proteger el balance general de la degradación estructural de Alpha.

*   **Objetivo:**
    Para mitigar drásticamente el costo de oportunidad del capital en cuentas de fondeo, acelerando la validación estadística de meses a solo 7 días con un filtro de autodescarte ultrarrápido y seguro. Proporcionar un control matemático riguroso contra la degradación estructural de estrategias en vivo.

*   **Reglas:**
    - **Inmutabilidad en Sandbox:** Queda prohibido modificar cualquier parámetro o lógica de la estrategia durante su periodo de cuarentena o incubación.
    - **Cero Conectividad Directa de API:** Las ejecuciones en cuarentena utilizan el feed de datos en tiempo real pero canalizan órdenes simuladas, evitando interacciones directas con claves de API del exchange real.
    - **Liquidación Inmediata por Deriva:** La activación del Broken Strategy Flag (-1 sigma) fuerza la cancelación de órdenes y cierre de posiciones con latencia menor a 1ms antes de pausar la estrategia en el DAG.

*   **Implementación:**
    - Selector visual en el Dashboard para configurar el tipo de incubación (Quarantine 7 días, Extended 21 días, o Legacy 3-6 meses).
    - Gráfico interactivo del Cono de Silencio con bandas sombreadas (1, 2 y 3 sigmas) sobre la equidad real en vivo.
    - Disparador automático de alerta y suspensión en la consola del operador ante purgas por rebasamiento MAE o violación de banda de confianza.

*   **Costo:**
    Mayor demanda computacional local en el hilo de telemetría para evaluar la MAE barra a barra en caliente y recalcular percentiles del cono de confianza diariamente.

*   **Trazabilidad:** [`incubation-manager.md`](./features/incubation-manager.md), [`paper-trader.md`](./features/paper-trader.md), [`monte-carlo-simulator.md`](./features/monte-carlo-simulator.md), [`incubate.md`](./modules/incubate.md).

---

### **ADR-0089: Motores de Optimización de Portfolio Clásicos & Ensamblador Singular D-Score con Hedging Cointegrativo, Router de Liquidez y Daemon de Rebalanceo**

*   **Decisión:**
    Implementar una arquitectura trimodal de gestión de capital y rebalanceo dinámico de portafolio que soporta:
    1. **Portfolio Optimization Engine Clásico:** Modelos académicos de distribución: Markowitz (Mean-Variance estándar), Black-Litterman (ajustes por opiniones/views), Equal Weighting, HRP (Hierarchical Risk Parity), Minimum Variance, Volatility Stabilization y Cluster Risk Convergence.
    2. **Ensamblador Singular D-Score (Risk Parity Dinámico & Alpha Decay):**
       - **Risk-Parity Normalizado:** Desacopla volatilidades exacerbadas por ATR castigando asimetrías de volatilidad macro para aplanar el ratio de retornos diarios y mitigar drawdowns.
       - **Hedging Tick-by-Tick (Cointegrative Voiding):** Monitor de nano-solapamientos cruzados de alta frecuencia. Si se detecta cointegración destructiva coincidente (+0.85) intra-segundo entre dos activos operados, se bloquean y desasisten márgenes recíprocamente, anulando volúmenes en caídas simultáneas con cero rezago.
       - **Router Viviente (Liquidez Radárica):** Monitor de predecibilidad. Detecta lateralidades sin alfa mayores a 72 horas y rota capital vía API hacia vectores eficientes exóticos (Materias primas o Criptoactivos).
    3. **Auto-Rebalancing Daemon:** Hilo daemon persistente (scheduler nativo Tokio) gobernado por triggers configurables:
       - Triggers: Calendario (semanal/mensual/trimestral), cambio de régimen dinámico HMM (confianza > umbral), threshold de desviación de pesos, o alertas por VaR/CVaR.
       - Mitigación de Riesgo: Circuit Breaker que restringe a máximo 1 rebalanceo por día para evitar sobrecarga operativa por comisiones (thrashing) y Portfolio Variance Check que prohíbe rebalanceos si la varianza del portafolio es mayor a 2σ (caos del mercado).

*   **Objetivo:**
    Para sustituir fronteras eficientes estáticas de rebalanceo anual por una asignación adaptativa en caliente de grado institucional. Optimizar el uso del capital reduciendo la exposición a solapamientos correlacionados en nano-segundos y rotando capital de forma oportunista.

*   **Reglas:**
    - **Prioridad del Portafolio:** Las reglas globales del portafolio (Hard Limits, VaR, limitación de drawdown) tienen prioridad jerárquica absoluta e invalidan cualquier orden o directiva de un agente individual.
    - **Circuit Breaker Inviolable:** Prohibido rebalancear más de una vez al día sin confirmación explícita del operador por seguridad.
    - **Restricción de Cointegración:** Las búsquedas de solapamientos destructivos operan a nivel de feed local en caliente en Rust Core con latencia menor a 2ms.

*   **Implementación:**
    - Panel interactivo en la UI con la matriz de pesos HRP, dendrograma interactivo y estado del rebalancer.
    - Configuración detallada en `portfolio_rules.yaml` de triggers y políticas.
    - Registro histórico inmutable de rebalanceos (`portfolio_rebalancing_history`) en SQLite para auditorías forenses.

*   **Costo:**
    Alta carga matemática de computación de matrices y cálculo de cointegraciones barra a barra/tick a tick en caliente, requiriendo procesamiento multi-hilo eficiente en Rust Core.

*   **Trazabilidad:** [`portfolio-optimizer.md`](./features/portfolio-optimizer.md), [`portfolio-rules.md`](./features/portfolio-rules.md), [`manage.md`](./modules/manage.md).

---

### **ADR-0090: Arquitectura de Portafolios Federados (Federated Portfolio Clusters)**

*   **Decisión:**
    Implementar una arquitectura de portafolios federados que permite coordinar múltiples contenedores de portafolios aislados y autónomos dentro de un meta-portafolio central sin interferencia de reglas cruzadas.
*   **Objetivo:**
    Para resolver la falta de sinergia y visibilidad unificada cuando un trader o institución opera múltiples subportafolios (ej. futuros y criptomonedas) en silos. Cada contenedor mantiene su gobernanza y reglas inmutables (frecuencia de rebalanceo, límites de correlación, objetivos de volatilidad) de manera independiente, compartiendo únicamente la infraestructura base (market feeds y adaptadores de broker) para optimizar recursos y mantener consistencia.
*   **Reglas:**
    - **Aislamiento Absoluto de Reglas:** Las reglas del portafolio A nunca interfieren con el portafolio B.
    - **Persistencia en Parquet y SQLite:** Las configuraciones se definen en esquemas JSON inmutables en SQLite y las métricas de rendimiento consolidadas se guardan en el lago de datos analítico.
    - **Kill Switch Global:** Existe un botón de emergencia global que detiene y liquida inmediatamente todos los subportafolios federados en paralelo ante riesgos sistémicos.
*   **Implementación:**
    - Un Dashboard unificado (Meta-Portfolio Panel) que muestra el rendimiento agregado (Sharpe/Sortino agregado, Drawdown del clúster, correlación inter-portafolios).
    - Vistas segregadas y aisladas para las métricas de cada contenedor individual.
    - Un panel de control del Kill Switch global con confirmación atómica en <5s.
*   **Costo:**
    Complejidad en la capa de ruteo de telemetría y ejecución (`NautilusTrader LiveNode`) para etiquetar y dirigir con precisión las órdenes y eventos de posición al contenedor correspondiente sin retrasos en el hot-path.
*   **Trazabilidad:** [`federated-portfolio.md`](./features/federated-portfolio.md), [`manage.md`](./modules/manage.md), [`execute.md`](./modules/execute.md).

---

### **ADR-0091: Simulación de Portafolio Real (Real Portfolio Backtesting)**

*   **Decisión:**
    Implementar un motor de simulación de portafolio avanzado que evalúa múltiples estrategias de forma concurrente compartiendo un pool de capital bajo restricciones realistas de margen, interés compuesto dinámico y sincronización exacta de horarios de mercado.
*   **Objetivo:**
    Los backtests agregados tradicionales cometen el error de asumir suma lineal de PnL independiente, lo que oculta colisiones de margen, llamadas de margen y fallos catastróficos cuando el capital total es insuficiente. Necesitamos paridad absoluta in-sample/out-of-sample simulando la contienda de recursos y el impacto de la capitalización sobre una sola cuenta unificada.
*   **Reglas:**
    - **Sincronización Determinista de Reloj:** Las estrategias se ejecutan sobre el mismo event-loop compartiendo un reloj determinista común sincronizado por el motor.
    - **Reglas de Compounding Inmutables por Período:** El interés compuesto dinámico se calcula de forma periódica inmutable y configurable, prohibiendo modificaciones intra-periodo para mantener la reproducibilidad bit-a-bit.
    - **Mapeo Riguroso de Sesiones:** El motor suspende y reanuda agentes de forma individual y determinista basándose en la configuración de sesión inmutable de cada exchange de origen.
*   **Implementación:**
    - Panel de configuración de backtest de portafolio que permite seleccionar el tipo de compounding (diario, semanal, mensual) y el pool de capital inicial.
    - Curva de equidad agregada real que refleja el impacto del consumo de margen cruzado y las comisiones unificadas.
    - Reportes analíticos que muestran alertas de margin calls simuladas y solapamientos de sesión.
*   **Costo:**
    Incremento notable en los tiempos de cómputo de optimizaciones masivas debido a la sincronización en memoria de múltiples agentes en el event-loop.
*   **Trazabilidad:** [`portfolio-backtest.md`](./features/portfolio-backtest.md), [`backtest-engine.md`](./features/backtest-engine.md), [`manage.md`](./modules/manage.md).

---

### **ADR-0092: Copy-Trading mediante Relé Ciego de Señales (E2E)**

*   **Decisión:**
    Implementar una topología de distribución de señales de trading de un solo sentido (Master a Copiers) utilizando un servidor relé ciego intermedio (Signal Relay) y encriptación simétrica AES-256-GCM de extremo a extremo (E2E).

*   **Objetivo:**
    Para eliminar la vulnerabilidad de seguridad y problemas de ancho de banda del Master inherentes a una arquitectura peer-to-peer (P2P) directa. Conectar múltiples clientes directamente expone la dirección IP del Master a ataques cibernéticos e inestabilidad de red. El relé ciego actúa como un intermediario sin acceso a la lógica ni datos descifrados del trade (Zero-Knowledge), optimizando la distribución de baja latencia a múltiples receptores.

*   **Reglas:**
    - **Cero Conocimiento del Relé:** El Signal Relay nunca almacena ni tiene acceso a las claves de encriptación de sesión o el contenido descifrado de las señales.
    - **Cifrado Obligatorio en Origen:** Toda señal debe ser serializada, comprimida, cifrada (AES-256-GCM) y firmada (HMAC-SHA256) antes de ser enviada al relé.
    - **Ejecución y Control de Riesgos Local:** Las órdenes se calculan y ejecutan localmente en la terminal de cada Copier según su capital y límites propios de riesgo, sin control centralizado de los brokers por parte del Master o del Relé.

*   **Implementación:**
    - El Master configura una clave única de sesión y ve el estado de conexión saliente hacia el servidor relé y el volumen de copiers activos en un dashboard de monitoreo.
    - El Copier introduce la clave de acceso criptográfica del Master y sus credenciales locales de broker, observando su latencia de señal (rechazo si >500ms) y su cuenta replicada.
    - El Signal Relay corre en segundo plano como un contenedor o daemon que únicamente valida tokens de sesión activos y retransmite bytes binarios asíncronamente.

*   **Costo:**
    Carga extra en el Copier para mantener feeds de volatilidad local (ATR) para el escalado de riesgo y latencia marginal añadida por el procesamiento de cifrado/descifrado y el salto intermedio del servidor (<50ms en total).

*   **Trazabilidad:** [`copy-trading-engine.md`](./features/copy-trading-engine.md), [`execute.md`](./modules/execute.md).

---

### **ADR-0093: Arquitectura de Seguridad Soberana (Sovereign Security Architecture)**

*   **Decisión:**
    Implementar un esquema de seguridad soberana Local-First e inmutable que exige: encriptación simétrica AES-256-GCM para credenciales de API de brokers administrada por variables de entorno de sistema (Vault-ready), registro inmutable en base de datos local SQLite de toda acción y respuesta operativa para auditoría forense, y operación en modo de privacidad absoluta con cero telemetría externa.

*   **Objetivo:**
    Para mitigar el riesgo de robo de claves de API y manipulación de registros de auditoría operativa, garantizando al mismo tiempo que la propiedad intelectual (estrategias y modelos) del operador retail se mantenga exclusivamente en la máquina del usuario (Soberanía de Datos).

*   **Reglas:**
    - **Cero Telemetría:** Prohibido el envío de cualquier dato operativo, telemetría o estadísticas fuera de la máquina local.
    - **Encriptación AES-256-GCM Obligatoria:** Todas las claves secretas en `broker_connections` deben encriptarse antes de persistir, utilizando una Master Key inyectada por el entorno.
    - **Inmutabilidad del Registro:** Las tablas `audit_log` y `events` son de solo inserción (Append-Only) y los registros deben validarse secuencialmente utilizando el hash criptográfico del registro anterior (`audit_chain_hash`).

*   **Implementación:**
    - Al iniciar, la aplicación valida la presencia de la variable de entorno de Master Key y descifra en memoria las claves del bróker.
    - El motor de trading escribe secuencialmente en las tablas relacionales locales de SQLite en modo WAL, calculando y enlazando hashes de fila consecutivamente.
    - Las conexiones salientes de telemetría o analíticas externas están bloqueadas por diseño.

*   **Costo:**
    Complejidad en la gestión manual de la clave maestra por el usuario y latencia mínima (<1ms) durante la encriptación/desencriptación en memoria y cálculo de hash de auditoría.

*   **Trazabilidad:** [`sovereign-security.md`](./features/sovereign-security.md), [`execute.md`](./modules/execute.md).

---

### **ADR-0094: Delegación Híbrida de Cómputo (Cooperative Hybrid Compute)**

*   **Decisión:**
    Soportar una modalidad de ejecución híbrida cooperativa (HybridComputeCooperative) que mantiene el backend Rust local acoplado al Frontend vía FFI compartiendo memoria local el 100% del tiempo, pero delega dinámicamente tareas de cómputo intensivo (búsqueda genética, optimización bayesiana, backtesting masivo) o daemons de ejecución persistentes 24/7 (Autopilot) a instancias remotas (VPS o clúster local/nube) sin desacoplar el core en la máquina local.

*   **Objetivo:**
    Para proporcionar una alternativa intermedia y complementaria a la migración completa a VPS (SaaSCloudEngine). Permite que la computadora local aproveche recursos gráficos y de CPU dedicados externos de manera fluida y temporal (a demanda) para análisis masivos, o de forma persistente para la operativa continua, sin perder la ultra-baja latencia FFI de la interfaz local para el uso cotidiano y la administración de datos soberanos.

*   **Reglas:**
    - **Inmutabilidad del Core Local:** El backend de la PC local sigue siendo la fuente de verdad (Source of Truth) de configuraciones, bases de datos de producción (SQLite) e historial.
    - **Protocolo de Mensajería Stateless:** La comunicación con los Workers de cómputo remoto debe ser estrictamente asíncrona mediante paso de mensajes serializados (gRPC/WebSockets) que no compartan estado de memoria del core local.
    - **Reconciliación Desconectada:** Ante una pérdida de conexión del cliente local, el daemon remoto del VPS debe operar bajo reglas estrictas de control de riesgo locales (autónomas) y sincronizar el ledger de transacciones al restablecerse la conectividad.

*   **Implementación:**
    - El usuario activa y configura las credenciales de conexión del pool de VPS / Workers remotos en el panel de hardware local.
    - Al lanzar un proceso pesado (ej. optimización de portafolios), el orquestador Rust local divide las tareas, las distribuye a los Workers remotos a través de gRPC, y recolecta las métricas consolidadas en memoria local para su visualización nativa.
    - Al activar el Autopilot remoto, el core local delega el estado del motor al VPS y permite cerrar la aplicación local de forma segura, mostrando el estado sincronizado en tiempo real al reabrir la app.

*   **Costo:**
    Complejidad en el orquestador Rust local para gestionar colas de tareas de red asíncronas, serialización/deserialización de metadatos de estrategias, y control de desincronización de estados (ledger) tras desconexiones prolongadas.

*   **Trazabilidad:** [`SAD.md`](./SAD.md), [`generate.md`](./modules/generate.md), [`execute.md`](./modules/execute.md).

---

### **ADR-0095: Veto Operativo por Degradación de Robustez de Slippage y Umbrales Monte Carlo**

*   **Decisión:**
    Establecer un veto operativo automático a nivel de pre-trade que bloquea la ejecución de cualquier orden perteneciente a una estrategia cuyo veredicto en las simulaciones Monte Carlo offline indique alta vulnerabilidad a la fricción de mercado (slippage y spread degradados), o que no posea un veredicto de robustez vigente.

*   **Objetivo:**
    Para mitigar las pérdidas catastróficas por fricción invisible en entornos reales. Si las simulaciones estocásticas de estrés demuestran que el Sharpe ratio de la estrategia se desintegra bajo un deslizamiento de $3\sigma$, es estadísticamente seguro que sus operaciones reales serán perdedoras. Este bloqueo automatiza la disciplina del operador y protege el capital.

*   **Reglas:**
    - **Fail-Safe por Defecto:** Si una estrategia no tiene registradas simulaciones de robustez en el databank local (SQLite/Parquet), el Pre-Trade Validator asume un estado inválido y bloquea las órdenes.
    - **Soberanía y Nivel de Veto:** El comportamiento del bloqueo (`HARD_VETO`, `WARNING_ALERT`, `DISABLED`) se configura exclusivamente a nivel de `Design Manifest` del portafolio/cuenta y no puede ser alterado dinámicamente por la estrategia.
    - **Latencia:** El check de veredicto debe realizarse mediante consulta en memoria compartida o cache indexing para mantener la validación pre-trade en menos de 1ms.

*   **Implementación:**
    - Al ingresar una señal de ejecución, el `Pre-Trade Validator` evalúa la versión de la estrategia y su `compliance_status_id`.
    - Si el veredicto es `PROP_FIRM_FRAGILE` o `TOXIC`, o carece de él, y la severidad es `HARD_VETO`, la orden se rechaza registrando el error `FAILED_ROBUSTNESS_VERDICT`.
    - Si la severidad es `WARNING_ALERT`, el sistema procesa la orden pero envía telemetría de alerta con prioridad máxima a la interfaz de usuario.

*   **Costo:**
    - Necesidad de asegurar que toda estrategia sea sometida al guantelete de robustez Monte Carlo antes de ir en vivo, aumentando el tiempo inicial de validación.
    - Overhead menor de memoria en el hot path para mantener cargados los veredictos de robustez vigentes por versión de estrategia activa.

*   **Trazabilidad:** [`SAD.md`](./SAD.md), [`pre-trade-validator.md`](./features/pre-trade-validator.md), [`monte-carlo-simulator.md`](./features/monte-carlo-simulator.md).

---

### **ADR-0096: Caché de Previews Locales de Nodo para Iteración Rápida**

*   **Decisión:**
    Almacenar localmente en la base de datos de producción (SQLite, modo WAL) un caché persistente con las curvas de equidad reducidas y métricas resumidas (`node_preview_cache` JSON blob) correspondientes a cada estrategia/nodo del editor de diagramas visuales (últimos 30 días de datos, periodicidad M15, sin simulaciones complejas de Monte Carlo). Si el usuario altera un parámetro micro de la estrategia en el inspector, el caché de pre-evaluación del nodo afectado y sus sucesores se invalida de forma inmediata y reactiva, requiriendo una regeneración manual asíncrona no bloqueante.

*   **Objetivo:**
    Para mitigar la latencia cognitiva de desarrollo. Ejecutar un backtest completo con todas las capas de robustez (Monte Carlo, WFA) en cada cambio de parámetro micro tarda minutos e interrumpe el flujo creativo del operador. La visualización instantánea (de 5 a 10 segundos) de curvas reducidas dentro del nodo del Strategy Inspector permite un descarte temprano y una sintonización fluida de parámetros de forma local-first.

*   **Reglas:**
    - **Aislamiento de Ejecución:** El cálculo de la vista previa no puede ejecutarse en el hot-path del live trading; se despacha a un hilo de baja prioridad del orquestador.
    - **Restricción de Parámetros de Simulación:** El motor de micro-backtests opera con condiciones fijas invariables (historial de 30 días, TF M15, sin comisiones/slippage dinámicos ni Monte Carlo) para garantizar tiempos de respuesta rápidos.
    - **Gobernanza de Invalidación:** Los resultados marcados como inválidos (caché nulo) muestran una advertencia en la UI e impiden la promoción de la estrategia a fases de incubación o producción hasta que se complete una validación exhaustiva exitosa.

*   **Implementación:**
    - Al seleccionar un nodo lógico en el editor visual de Flutter, el Inspector Contextual recupera y renderiza instantáneamente una curva de equidad reducida (50 puntos de datos) y métricas básicas (Sharpe, Profit).
    - Al editar un campo de entrada (ej. período de RSI), la equidad se desvanece visualmente a un estado inactivo (rojo/gris) con un botón para "Regenerar Preview".
    - El operador presiona el botón y un spinner indica el cálculo asíncrono en Rust en segundo plano sin congelar la interacción con el lienzo.

*   **Costo:**
    - Espacio adicional menor en la base de datos de producción para almacenar los blobs JSON serializados por cada versión/nodo lógico.
    - Overhead en la lógica del backend Rust para procesar las invalidaciones reactivas en cascada sobre el grafo del AST.

*   **Trazabilidad:** [`node-preview.md`](./features/node-preview.md), [`visual-dag-editor.md`](./features/visual-dag-editor.md).

---

### **ADR-0097: Renderizado Gráfico Multidimensional Nativo sin WebViews**

*   **Decisión:**
    Renderizar todos los gráficos de dispersión de alta dimensión (UMAP 2D/3D y proyecciones PCA) de manera nativa en Flutter mediante `CustomPainter` y aceleración de hardware (Impeller GPU), prohibiendo explícitamente la integración de WebViews, Plotly JS, Deck.gl o cualquier motor de dibujo web externo dentro de la aplicación.

*   **Objetivo:**
    Para cumplir con el ADR-0029 (Patrón Todo en Uno) y proteger la fluidez del hilo de UI en Dart. Los WebViews rompen la consistencia del renderizado en GPU de Flutter, aumentan drásticamente el consumo de memoria RAM (>200MB por instancia) e introducen latencias por serialización JSON de grandes arrays de puntos. El dibujo directo en lienzo nativo aprovecha la aceleración física de Vulkan/Metal y mantiene estables los 120 FPS.

*   **Reglas:**
    - **Cero Lógica Dimensional en Dart:** Toda reducción dimensional y preparación de datos (cálculo de UMAP/PCA en Polars) debe ocurrir en el backend Rust, enviando únicamente arrays vectoriales Arrow bidimensionales/tridimensionales al frontend.
    - **Límite de Densidad:** Envíos por encima de 100K puntos en el scatter plot de Flutter deben someterse a downsampling estocástico rápido en Rust antes de ser transmitidos para evitar saturación de la memoria compartida FFI.
    - **Interacción Determinista:** Las selecciones por lazo (lasso select) o caja deben procesar las intersecciones geométricas localmente en Dart en menos de 16ms.

*   **Implementación:**
    - El operador abre el visualizador de clústeres y el gráfico 3D gira con suavidad al interactuar con el mouse/gestos.
    - Al usar la herramienta de lazo, se dibuja un trazado cerrado libre sobre un grupo de puntos y de forma instantánea se resalta la tabla de estrategias correspondientes a las coordenadas interceptadas.

*   **Costo:**
    Complejidad en el desarrollo de la matemática de proyección 3D a 2D y el cálculo de colisiones geométricas de lazo (lasso intersection) dentro del hilo Dart en Flutter.

*   **Trazabilidad:** [`umap-scatter-visualizer.md`](./features/umap-scatter-visualizer.md), [`toxicity-purifier.md`](./features/toxicity-purifier.md).

---

### **ADR-0098: Gobernanza de Purgas y Snapshots de Databank**

*   **Decisión:**
    Gobernar las operaciones de eliminación masiva de clústeres de estrategias tóxicas bajo un protocolo estricto de confirmación multi-paso, marcado lógico (`soft-delete` mediante columna `is_purged=true`) y generación automatizada de snapshots de catálogo en SQLite antes de procesar el descarte definitivo. Se provee una vía rápida de reversión (rollback) que recupera la integridad anterior al vuelo.

*   **Objetivo:**
    Para mitigar fallos operativos críticos causados por el descarte accidental de estrategias. PCA clasifica agrupamientos de comportamiento sin conocer la lógica íntima de las estrategias; una purga errónea de clústeres podría eliminar meses de cómputo genético. El soft-delete y los snapshots garantizan que el operador posea un mecanismo rápido y de bajo costo para deshacer la purga si el impacto en los KPIs es destructivo.

*   **Reglas:**
    - **Paso Obligatorio de Snapshot:** Es imposible purgar un clúster sin generar previamente un identificador de snapshot de recuperación inmutable en SQLite.
    - **Exclusión Reactiva:** Las estrategias marcadas con `is_purged=true` se ocultan de inmediato de las consultas OLAP (DuckDB) de portafolios y visualizadores, liberando los slots calientes de simulación.
    - **Límite de Reversión (Rollback Window):** El deshacer de una purga debe ejecutarse de forma atómica en menos de 5 segundos restableciendo el estado lógico original.

*   **Implementación:**
    - El operador presiona "Purge Cluster 04". La UI exige confirmar escribiendo el código de confirmación tras mostrarle una previsualización de impacto (ej: "Eliminará 412 estrategias, Reducción de Sharpe promedio de cartera de +0.15").
    - Si el operador decide retroceder, el panel de historial de purgas muestra una lista de descartes con un botón "Rollback / Deshacer". Al presionarlo, las estrategias reaparecen en el visualizador sin requerir recálculo de backtests.

*   **Costo:**
    Leve overhead de base de datos para mantener el árbol de snapshots locales e historial de purgas, y lógica de descarte lógico en las consultas DuckDB habituales.

*   **Trazabilidad:** [`toxicity-purifier.md`](./features/toxicity-purifier.md), [`pca-toxicity-analyzer.md`](./features/pca-toxicity-analyzer.md).

---

### **ADR-0099: Marketplace de "Cajas Negras" con Zero-Knowledge Nodes**

*   **Decisión:**
    Implementar un esquema de cifrado asimétrico local-first para empaquetar subgrafos lógicos visuales (Strategy AST) en binarios cerrados (Cajas Negras) que se distribuyen en un marketplace local P2P. La lógica del subgrafo se descifra exclusivamente en la memoria RAM volátil del Hot-Path durante la ejecución y se evalúa de forma opaca (Zero-Knowledge) sin revelar la estructura ni las variables del AST al comprador del nodo.

*   **Objetivo:**
    Para habilitar la comercialización y uso cooperativo de estrategias entre usuarios de la plataforma sin forzar a los creadores a revelar su propiedad intelectual. La encriptación asimétrica y la carga directa en memoria evitan la ingeniería inversa del código sin requerir un servidor centralizado de ejecución en la nube.

*   **Reglas:**
    - **Cero Volcados a Disco:** Queda estrictamente prohibido guardar el AST descifrado en archivos temporales o logs de depuración locales.
    - **Aislamiento en Memoria:** Las funciones de evaluación del subgrafo cerrado no deben emitir trazas internas de nodos ejecutados al bus de eventos de telemetría de la UI.
    - **Firma por Hardware:** Las licencias de uso se firman digitalmente contra el Hardware ID único del comprador local.

*   **Implementación:**
    - Un creador hace clic derecho en un grupo de nodos de su editor visual, selecciona "Exportar como Caja Negra", ingresa sus metadatos y genera un archivo `.qfnode` cifrado.
    - El comprador arrastra el archivo `.qfnode` a su lienzo. Se muestra como un nodo único con entradas y salidas normales, pero no tiene opción de "Doble clic para expandir" o "Editar". Al simular, el backtest se ejecuta empleando la lógica encriptada de forma transparente.

*   **Costo:**
    Complejidad en la gestión de llaves y firmas digitales locales de licencias, y leve sobrecosto computacional durante la fase de descifrado en caliente en la inicialización de los workers.

*   **Trazabilidad:** [`marketplace-cajas-negras.md`](./moonshots/marketplace-cajas-negras.md).

---

### **ADR-0100: Relegación de Microestructura L3 a SaaS Institucional y Proxies Client Zero**

*   **Decisión:**
    Relegar el soporte de datos y simulaciones de Microestructura Nivel 3 (L3 Market-by-Order) fuera de la arquitectura base Client Zero local-first. Esta funcionalidad se clasifica con prioridad baja (P4) para la fase de incubación en moonshots y se reserva únicamente para una futura expansión de SaaS institucional. En la arquitectura local se aprueba el uso exclusivo de datos L1/L2 y métricas MAE/MFE como proxies eficientes de microestructura.

*   **Objetivo:**
    Para cumplir estrictamente con el Client Zero Protocol. Los feeds de datos L3 tienen costos de $5K-$20K/mes por instrumento y generan volúmenes de datos incompatibles con el almacenamiento de computadoras de consumo de uso personal (10-50 GB/día por símbolo, superando los 200TB anuales), requiriendo infraestructuras complejas de ClickHouse en red que anulan el diseño "Zero-Docker" y local-first.

*   **Reglas:**
    - **Prohibición de Feeds L3 Locales:** Ningún conector de broker local en el MVP del cliente local debe requerir feeds de Nivel 3.
    - **Paridad con NautilusTrader:** Mantener el uso del motor nativo `L3_MBO` de NautilusTrader inactivo en local, preparado para ser reactivado únicamente bajo perfiles de despliegue en la nube empresarial.

*   **Implementación:**
    - El catálogo de orígenes de datos locales no lista feeds de Nivel 3. El backtest se ejecuta de forma óptima consumiendo series temporales L1/L2.
    - El sistema simula fricciones y prioridad de colas mediante modelos analíticos de impacto de mercado en lugar de calcular colas orden por orden con feeds masivos.

*   **Costo:**
    Menor precisión microscópica de colas de ejecución en backtesting de alta frecuencia de milisegundos en la aplicación local, lo cual no es prioritario para traders retail y profesionales independientes.

*   **Trazabilidad:** [`microestructura-l3.md`](./moonshots/microestructura-l3.md).

---

### **ADR-0101: Transpilación Basada en Plantillas Tera para Modelos AST**

*   **Decisión:**
    Adoptar un motor de plantillas desacoplado (Tera) en el backend Rust para transpilar el Grafo de Lógica visual (Strategy AST) a múltiples lenguajes de programación y APIs de plataformas de trading externas (MQL4, MQL5, NinjaScript C#, EasyLanguage, Python), aislando la representación matemática del AST de la sintaxis del lenguaje destino.

*   **Objetivo:**
    Para posibilitar la compatibilidad y migración fluida de los usuarios de Drasus Engine hacia entornos e infraestructuras de corretaje heredadas (MT4/5, TradeStation) sin duplicar la lógica de parseo del AST de la estrategia. La separación por plantillas facilita agregar nuevos lenguajes destino simplemente escribiendo un nuevo template de código fuente.

*   **Reglas:**
    - **Validación Sintáctica Obligatoria:** Todo código transpilado debe ser validado localmente con expresiones regulares o compiladores locales antes de ser escrito en el disco duro.
    - **Preservación Semántica:** El código generado debe mantener paridad absoluta 1:1 en las condiciones lógicas de entrada, salida y stop-loss con el comportamiento del simulador en Rust.

*   **Implementación:**
    - El operador selecciona una estrategia aprobada en la UI, hace clic en "Exportar MQL5" y de forma instantánea se descarga un archivo `.mq5` limpio, estructurado con inputs configurables correspondientes a los parámetros del AST.

*   **Costo:**
    Mantenimiento y actualización periódica de las plantillas de transpilación ante cambios y actualizaciones de APIs en las plataformas propietarias de destino.

*   **Trazabilidad:** [`universal-strategy-transpiler.md`](./moonshots/universal-strategy-transpiler.md).

---

### **ADR-0102: Anonimización Criptográfica local-first en Collective Intelligence**

*   **Decisión:**
    Diseñar el intercambio de datos colectivos (Collective Intelligence) bajo un estricto protocolo local-first de anonimización criptográfica. Las métricas de rendimiento se alteran mediante ruido gaussiano controlado (Differential Privacy) y la topología de la estrategia se comprime a una firma hash unidireccional (SHA-256) antes de ser transmitida a la red, previniendo la ingeniería inversa de los parámetros exactos y fórmulas de la estrategia original.

*   **Objetivo:**
    Para permitir que la comunidad de usuarios aprenda de patrones de rendimiento globales e ineficiencias de mercado (sabiduría de la multitud) sin obligar a los traders a revelar su propiedad intelectual confidencial ni arriesgar el arbitraje de su ventaja competitiva.

*   **Reglas:**
    - **Consentimiento Explícito (Opt-In):** El intercambio de firmas está completamente desactivado por defecto en la instalación.
    - **Prohibición de Datos Crudos:** Queda estrictamente prohibido enviar ecuaciones del AST, nombres de variables reales, balances monetarios en dólares, llaves API, o IPs de servidores de trading en vivo al pool colectivo.

*   **Implementación:**
    - El panel de Collective Intelligence muestra un interruptor "Activar contribución anónima". Al activarlo, el sistema sube la firma hash `SHA256(RSI+MACD+EMA)` asociada a un Sharpe normalizado con ruido (+1.82 ± 0.05), recibiendo a cambio un feed de combinaciones recomendadas globalmente.

*   **Costo:**
    Pérdida de la precisión exacta de las métricas recopiladas debido al ruido inyectado, reduciendo levemente la fidelidad estadística del Meta-Learner a cambio de una privacidad inquebrantable.

*   **Trazabilidad:** [`collective-intelligence.md`](./moonshots/collective-intelligence.md).

---

### **ADR-0103: Filosofía Dual y Sandboxing en el Sistema de Plugins Institucionales**

*   **Decisión:**
    Gobernar la integración de plugins de terceros mediante una máquina virtual WebAssembly aislada (Wasmer Sandbox) y APIs de SDK locales expuestas vía gRPC restringido. El lanzamiento comercial del marketplace de plugins se supedita a la **Filosofía Dual**, priorizando el uso personal ("Client Zero") del fundador por un periodo de rentabilidad documentada (6-12 meses) antes de abrir la plataforma a la venta o despliegues B2B/B2C masivos.

*   **Objetivo:**
    Para garantizar que la base de código sea robusta, segura y rentable en primer lugar antes de asumir costes de soporte e integración corporativos, y proteger la máquina local del trader de accesos no autorizados a datos de brokers y archivos del sistema causados por plugins comunitarios maliciosos.

*   **Reglas:**
    - **Sandboxing por Defecto:** Ningún plugin de origen externo puede ejecutarse directamente en el host nativo; se cargan exclusivamente en entornos WebAssembly sin acceso al disco o red por defecto.
    - **Veto de gRPC en Live Trading:** Los plugins externos no pueden enviar órdenes al mercado real (`execute`) de forma directa; deben enrutarse por el Pre-Trade Validator y la cola de prioridad del monolito.

*   **Implementación:**
    - Un usuario descarga un plugin de visualización interactiva `.qfplugin`. El sistema inicializa Wasmer, carga el binario y renderiza los componentes gráficos en un canvas aislado. Si el plugin intenta leer archivos de claves locales, la VM bloquea la llamada reportando violación de acceso.

*   **Costo:**
    Complejidad en el mapeo de la API del SDK local hacia interfaces WebAssembly/gRPC y limitaciones de rendimiento gráfico para complementos complejos corriendo en entornos de sandbox virtualizados.

*   **Trazabilidad:** [`institutional-plugin-system.md`](./moonshots/institutional-plugin-system.md).

---

### **ADR-0104: Traducción de Características y Pila del Roadmap Acelerado a Rust/Flutter Core**

*   **Decisión:**
    Rechazar de forma absoluta e irreversible la adopción de una pila o monorepo basado en Python, FastAPI, herramientas de empaquetado de Python (`uv`, `pyproject.toml`) o servidores web basados en microservicios locales para el backend. En su lugar, todas las características avanzadas detalladas en el Roadmap Acelerado v2.0 (tales como la orquestación de nodos, minería genética NSGA-II, UMAP, autoencoders de anomalías, agrupamiento de toxicidad por PCA y rebalanceo dinámico de portafolios HRP/HMM) se diseñan e implementan de manera nativa en Rust para el Core/Backend y en Flutter (Dart/Impeller) para la interfaz gráfica, integrados mediante FFI local (`flutter_rust_bridge`) y flujos gRPC de respaldo para ejecución VPS Headless.

*   **Objetivo:**
    Para cumplir con el postulado de **Soberanía y Máximo Rendimiento ("Zero-Docker" / "Local-First")** del sistema. Introducir Python y FastAPI incrementa de manera inaceptable la latencia en el hot-path, añade problemas de concurrencia y recolección de basura, destruye el determinismo bit-a-bit en simulaciones concurrentes masivas, y multiplica la complejidad de distribución e instalación para el operador final (Client Zero).

*   **Reglas:**
    - **Prohibición de Librerías Python en Producción:** Queda terminantemente prohibida la inclusión de scripts Python, entornos virtuales o intérpretes en el binario comercial empaquetado. 
    - **Uso Nativo de Modelos IA (escalera ADR-0112):** El entrenamiento y la ejecución de redes neuronales (como Autoencoders) o algoritmos de reducción dimensional (como UMAP) usan, en este orden, crates de Rust puro de álgebra lineal optimizada (`ndarray` + Rayon, default) → `candle` (Rust puro, GPU dinámica opcional) si un benchmark lo justifica → `burn` solo en el moonshot DRL. **Prohibido `tch-rs`/libtorch** (rompe el binario único del ADR-0029).
    - **Interfaces FFI/gRPC:** La interfaz nodal visual e inspectores se gestionan con componentes Dart/Flutter reactivos a eventos FFI de Rust, eliminando el uso de frameworks web como React o React Flow en el frontend.

*   **Implementación:**
    - El usuario diseña estrategias visualmente en Flutter. Las peticiones de micro-backtests y la compilación del AST se delegan vía comandos FFI a workers en Rust que procesan en milisegundos sin sobrecarga de red o serialización JSON compleja.

*   **Costo:**
    Mayor tiempo de desarrollo y complejidad en el diseño de algoritmos de Machine Learning y procesamiento matricial en Rust en comparación con sus equivalentes listos para usar en el ecosistema Python.

*   **Trazabilidad:** [`SAD.md`](./SAD.md#18-plan-de-lanzamiento-rollout-strategy-v2.0), [`node-preview.md`](./features/node-preview.md).

---

### **ADR-0105: Estrategia de Datos (100% Polars Nativo en Rust)**

*   **Decisión:** Adoptar de forma exclusiva el ecosistema **Polars** (nativo en Rust) para todo el procesamiento de DataFrames, transformaciones analíticas (OLAP), e ingesta pesada, erradicando cualquier dependencia o herencia previa de Pandas o Python.
*   **Objetivo:** Aprovechar todo el poder del procesamiento multi-hilo, paralelización SIMD y **Lazy Evaluation** (Evaluación Perezosa) que Polars ofrece nativamente en su implementación de Rust, logrando el máximo performance sin sobrecargas de conversión.
*   **Reglas:**
    *   **DataFrames Exclusivos:** Toda manipulación, agregación de ventanas, cálculo de indicadores y agrupamiento se orquesta utilizando la API Nativa de Polars en Rust.
    *   **Prohibición de Alternativas Legadas:** Dado el paso a Rust, conceptos como Pandas o conversiones Zero-Copy FFI entre intérpretes carecen de sentido.
    *   **Integración matemática:** Los cálculos analíticos profundos (regresión, monte carlo) se elaboran escribiendo expresiones nativas Polars (`expr`) o delegando a librerías de álgebra lineal de Rust.
*   **Ventaja:** Supera radicalmente las limitaciones previas de memoria; procesa Gigabytes de mercado localmente en sub-segundos sin interbloqueos (GIL).
*   **Costo:** Curva de aprendizaje del diseño robusto y concurrente del API de Polars en Rust.
*   **Trazabilidad:** [hybrid-data-transformer.md](./features/hybrid-data-transformer.md).

---

### **ADR-0106: Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión**

*   **Decisión:**
    Adoptar una separación estricta entre la visualización interactiva y el motor matemático mediante renderizado nativo GPU acelerado (Impeller) en el Frontend (Flutter), comunicándose con el Backend (Rust) exclusivamente vía FFI (Foreign Function Interface) local-first de memoria compartida o canales de telemetría asíncronos en gRPC.

*   **Objetivo:**
    Para evitar el bloqueo o retardo visual en la interfaz al graficar cientos de miles de puntos de datos históricos, métricas de brokers y simulaciones en tiempo real. La lógica pesada de agregación se delega a Polars/DuckDB en Rust, el cual realiza downsampling estructurado antes de enviar los datos al frontend.

*   **Reglas:**
    - **Cero Cálculos Analíticos en Frontend:** Flutter es únicamente presentador; prohibido calcular coeficientes de correlación, drawdowns o retornos en el hilo Dart.
    - **Límite de Frecuencia de Actualización (Throttling):** Las señales de telemetría visuales deben limitarse a una frecuencia máxima de refresco de 100ms para evitar la saturación de los canales de la interfaz de usuario.
    - **Renderizado por GPU Impeller:** Todo elemento visual avanzado (lienzo nodal, mapas de calor dinámicos) se debe renderizar nativamente en hardware gráfico a una velocidad de refresco estable de 120 FPS / 60 FPS.

*   **Implementación:**
    - El usuario interactúa de forma fluida con mapas de calor de calidad de datos y matriz de años por meses sin experimentar congelamientos o caídas de FPS.
    - Las configuraciones de estrategias muestran diferencias visuales instantáneas (Strategy Config Diff) al contrastar los contratos lógicos almacenados localmente en SQLite.
    - Los reportes en PDF se generan server-side de forma headless y asíncrona, preservando los recursos del cliente.

*   **Costo:**
    Necesidad de estructurar y mantener contratos estrictos de schemas y comandos a través del puente de comunicación FFI y serializar eficientemente los objetos analíticos procesados por Rust.

*   **Trazabilidad:** [`efficiency-incubation-dashboard.md`](./features/efficiency-incubation-dashboard.md), [`throttling-metrics-dashboard.md`](./features/throttling-metrics-dashboard.md), [`monthly-performance-heatmap.md`](./features/monthly-performance-heatmap.md), [`trade-analysis-bi-suite.md`](./features/trade-analysis-bi-suite.md), [`strategy-config-diff.md`](./features/strategy-config-diff.md), [`pdf-charts-rendering.md`](./features/pdf-charts-rendering.md), [`visual-stockpicker-configurator.md`](./features/visual-stockpicker-configurator.md).

---

### **ADR-0107: Integración Nativa con NautilusTrader v2 (Crates Rust, Sin Python, Sin Fork)**

*   **Decisión:**
    Integrar NautilusTrader (NT) consumiendo directamente los **crates Rust nativos de su núcleo v2** publicados en crates.io (motor de backtesting de eventos, modelo de dominio, ejecución live y adaptadores de venue) como dependencias Cargo del Core, detrás de la capa anticorrupción ya contratada en [`nautilus-integration.md`](./features/nautilus-integration.md). Se **rechaza el fork del repositorio** y se **rechaza construir un motor de ejecución desde cero**; esta última opción queda archivada como contingencia en [`sovereign-execution-engine.md`](./moonshots/sovereign-execution-engine.md).

*   **Objetivo:**
    - **El conflicto "Python como interfaz" desapareció upstream.** La interfaz Python/Cython corresponde a NT v1 (legado). El núcleo v2 es Rust puro: permite escribir estrategias, ejecutar backtests de alta fidelidad y operar en vivo contra brokers reales íntegramente desde Rust, sin intérprete Python en el proceso. Python quedó relegado a capa opcional de conveniencia (bindings PyO3) que este sistema NO utiliza, en cumplimiento del ADR-0104.
    - **Lo que hace valioso a NT es exactamente nuestro pilar arquitectónico.** Su reputación institucional proviene de: (1) **paridad investigación-producción** — el mismo código de estrategia corre en backtest y en vivo bajo semántica de ejecución idéntica; (2) **arquitectura event-driven determinista** con bus de mensajes en memoria de cero copias; (3) **matching engine realista** con modelos de fill y latencia configurables; (4) **resolución temporal de nanosegundos** y soporte multi-venue/multi-activo (forex, acciones, futuros, opciones, cripto); (5) un **ecosistema de más de 25 adaptadores** de brokers y proveedores de datos mantenido por terceros. Replicar esto desde cero consumiría años-persona sin generar un solo punto de Alpha diferencial: el Alpha del sistema vive en Generate/Validate (minería y robustez), no en reimplementar un matching engine.
    - **Un fork no resuelve nada y destruye valor.** NT es LGPL-3.0: un fork privado con modificaciones sigue obligado por la licencia, pierde el flujo de mejoras y adaptadores del upstream, y convierte a este proyecto en mantenedor de un motor ajeno de cientos de miles de líneas.

*   **Reglas:**
    - **Versionado Congelado (FIJO):** La API Rust de NT v2 aún declara inestabilidad entre releases (serie 0.x). Las versiones de los crates se fijan exactas y su código fuente se incluye en modo vendoring local. Toda actualización del upstream es un evento deliberado: se ejecuta la suite de paridad bit-a-bit del puente antes de aceptar el cambio.
    - **Capa Anticorrupción (FIJO):** Ningún módulo de negocio importa tipos de NT. Solo la feature [`nautilus-integration`](./features/nautilus-integration.md) conoce los tipos del motor y los mapea a tipos propios del dominio. Esta capa es el cortafuegos contractual que preserva la opcionalidad de salida.
    - **Cumplimiento LGPL-3.0 (FIJO):** El binario comercial debe permitir el reenlazado de la porción LGPL (enlace dinámico de los crates de NT o entrega de objetos reenlazables). PROHIBIDO modificar el código de los crates vendorizados: cualquier cambio necesario se implementa fuera (composición contra traits públicos) o se contribuye upstream.
    - **Brechas de Adaptadores (CONFIG):** Los brokers cuyo adaptador solo es estable en NT v1 (ej. Interactive Brokers) se cubren con crates adaptadores propios escritos contra los traits públicos de cliente de datos/ejecución de NT v2, mantenidos como módulos independientes candidatos a contribución upstream. NUNCA parchando el núcleo.
    - **Cobertura de Clases de Activo (Mandato de Producto):** El puente debe tratar como ciudadanos de primera clase: acciones, forex, futuros, ETFs y CFDs. Las **opciones financieras se difieren a la última fase del roadmap** por la complejidad intrínseca del instrumento (griegas, cadenas de vencimientos, ejercicio/asignación) y el acceso problemático a datos históricos de calidad.

*   **Implementación:**
    - El backtesting masivo de Validate y el LiveNode de Execute corren dentro del mismo proceso Rust del Core (tareas Tokio), sin sidecars, sin puentes inter-proceso y sin intérpretes: la curva de equidad del backtest y la operación en vivo emergen del mismo loop de eventos determinista.
    - El operador cambia de broker o de modo (Backtest/Paper/Live) sin tocar la lógica de estrategia, porque el contrato de la capa anticorrupción es invariante al backend.

*   **Costo:**
    Vigilancia activa del upstream (cambios de firmas entre releases 0.x absorbidos por el puente), auditoría legal del empaquetado LGPL en cada release comercial, y el mantenimiento de adaptadores propios para los brokers no cubiertos por el núcleo v2.

*   **Plan de Contingencia (Opcionalidad de Salida):** Si el upstream se abandona o vira contra los intereses del proyecto, el orden de respuesta es: (1) congelar la última versión vendorizada estable (operativa indefinidamente por ser local-first); (2) fork de mantenimiento mínimo — solo seguridad y compatibilidad, publicado bajo LGPL; (3) activar el moonshot [`sovereign-execution-engine.md`](./moonshots/sovereign-execution-engine.md). La capa anticorrupción garantiza que ninguno de estos escenarios toca la lógica de negocio.

*   **Trazabilidad:** [`nautilus-integration.md`](./features/nautilus-integration.md), [`sovereign-execution-engine.md`](./moonshots/sovereign-execution-engine.md), [`SAD.md`](./SAD.md) §2.2, ADR-0013, ADR-0104.

---

### **ADR-0108: Arquitectura de Genomas Modulares por Dominio (Generalización del Patrón de Genes Condición→Acción)**

*   **Decisión:**
    Generalizar el mecanismo de Programación Evolutiva Parcial descrito en ADR-0043 (`wildcard_group` sobre un AST predefinido) de un mecanismo aplicado exclusivamente al dominio de Señal de Entrada/Salida, a un patrón arquitectónico transversal denominado **Gramática de Genes Condición→Acción**. Este patrón define dos categorías universales y reutilizables de nodos evolutivos:
    - **Genes de Condición:** predicados que el motor evolutivo combina y parametriza, evaluando el estado observable de un dominio determinado (mercado, posición abierta, portafolio, régimen estructural) y devolviendo un veredicto booleano o categórico en cada barra/evento.
    - **Genes de Acción:** primitivas paramétricas — ya existentes como comportamientos configurables en distintas Features — que el motor evolutivo activa, desactiva o reconfigura cuando su(s) Gen(es) de Condición asociado(s) se satisface(n).

    Se establece, además, un **Registro de Dominios Genómicos**: un catálogo arquitectónico cerrado de los dominios sobre los que esta gramática puede instanciarse. Cada entrada del registro es un Generador Genómico de Dominio independiente, con su propio espacio de Genes de Condición, su propio espacio de Genes de Acción, su propio segmento embebido dentro del AST (`wildcard_group` con espacio de nombres por dominio) y su propia batería de robustez heredada o extendida. El registro queda compuesto, a la fecha de este ADR, por:
    1. **Dominio de Señal** (línea base preexistente, formalizada originalmente en ADR-0043): genes de entrada/salida de operación.
    2. **Dominio de Riesgo y Gestión de Posición** (Fase A — ADR-0109).
    3. **Dominio de Régimen y Filtro de Entorno** (Fase B — ADR-0110).
    4. **Dominio de Portafolio y Correlación** (Fase C — ADR-0111).

    Un quinto dominio candidato — Ejecución y Enrutamiento de Órdenes — fue evaluado y **excluido** del registro activo por las restricciones descritas más abajo; queda archivado como exploración de largo plazo en `/moonshots/`.

*   **Objetivo:**
    La pregunta que originó este ADR fue si el sistema podía soportar estrategias cuya gestión de riesgo y posición muta drásticamente ante secuencias de eventos específicas (p. ej. "tras 3 pérdidas consecutivas, reducir el riesgo a 0.2% y redistribuir el Stop Loss en 3 fases y el Take Profit en 2"). La respuesta de diseño correcta no es modelar estas mutaciones como máquinas de estados finitos diseñadas a mano (un "perfil de riesgo" estático por estrategia), sino reconocer que el problema tiene **forma idéntica** al que ya resuelve WildCards para la lógica de entrada: dado un espacio de condiciones observables y un espacio de acciones paramétricas, evolucionar vía NSGA-II la combinación que maximiza el fitness multi-objetivo bajo el régimen de robustez correspondiente.

    Generalizar el patrón en lugar de construir motores nuevos independientes para cada idea evita triplicar la infraestructura de minería genética: el mismo motor de `wildcard_group`, el mismo orquestador NSGA-II ([nsga2-optimizer.md](./features/nsga2-optimizer.md)) y el mismo pipeline backtest → WFA → Monte Carlo se reutilizan, parametrizados por dominio. Formalizar un Registro de Dominios da además un lugar arquitectónico explícito para evaluar futuros dominios candidatos contra criterios de admisión fijos, en lugar de reabrir la arquitectura cada vez que surge una idea nueva de "qué más podríamos evolucionar".

*   **Reglas:**
    - **Wildcard Invertido Generalizado (FIJO):** `ACTIVE_GENOME_DOMAINS` puede contener cualquier subconjunto no vacío de los 4 dominios del Registro. Los genomas de los dominios **fuera** de `ACTIVE_GENOME_DOMAINS` permanecen congelados/bloqueados dentro del mismo Manifest — el "Wildcard Invertido" formalizado en ADR-0109. Cuando `ACTIVE_GENOME_DOMAINS` contiene más de un dominio, todos evolucionan **conjuntamente** como un único genoma compuesto (ver "Regla Genómica"). La co-evolución de cartera de Fase C (ADR-0111, población de Manifests) es un eje ortogonal que se combina libremente con cualquier `ACTIVE_GENOME_DOMAINS` por Manifest miembro.
    - **Regla Genómica (FIJO):** unidad de evolución de cada dominio (y del genoma compuesto cuando hay >1 dominio activo). Una Regla Genómica = 1..`MAX_CONDITIONS_PER_RULE` Genes de Condición (combinables con operadores AND/OR, de cualquier dominio en `ACTIVE_GENOME_DOMAINS`) → 1..`MAX_ACTIONS_PER_RULE` Genes de Acción simultáneos (también de cualquier dominio activo). Generaliza la estructura de reglas de entrada/salida multi-condición que el Dominio de Señal ya posee desde ADR-0043 a los 4 dominios del Registro. Cada gen conserva su etiqueta de dominio de origen para la atribución de fitness.
    - **Unicidad del Manifest (FIJO):** todos los genomas de dominio activos se serializan dentro del mismo Strategy Manifest y comparten un único `manifest_id`/`logic_hash`. Un Manifest nunca se fragmenta en documentos separados por dominio.
    - **Criterios de Admisión al Registro (CONFIG/Gobernanza):** para que un dominio candidato futuro entre al registro activo debe demostrar: (a) un espacio de Genes de Condición observable y reproducible determinísticamente sobre datos históricos almacenados; (b) un espacio de Genes de Acción cuya aplicación sea reproducible bit-a-bit entre backtest y operativa en vivo; y (c) una batería de robustez propia o heredada capaz de invalidar combinaciones sobreajustadas específicas del dominio. El Dominio de Ejecución y Enrutamiento fue evaluado contra estos criterios y reprobó el criterio (a): los datos de microestructura (profundidad de libro L2/L3, latencia real de enrutamiento) no están disponibles de forma consistente para el operador retail/solopreneur objetivo de este sistema — mismo principio que ADR-0100.
    - **Penalización de Complejidad Multi-Dominio (CONFIG):** la métrica de Ockham's Razor ([complexity-penalization.md](./features/complexity-penalization.md)) se extiende para sumar los grados de libertad de TODOS los genomas de dominio activos en el Manifest contra el mismo denominador de operaciones (`MIN_TRADES_PER_PARAM`), evitando que la complejidad evolutiva "migre" hacia el dominio menos penalizado para evadir el filtro.
    - **Trazabilidad de Atribución (CONFIG):** el reporte de fitness de NSGA-II debe poder atribuir, por separado, la contribución de cada genoma de dominio al score final, para que el operador entienda qué dominio impulsó una mejora o un deterioro.

*   **Implementación:**
    - Al inspeccionar un Strategy Manifest, el operador ve una sección de genoma por cada dominio activo (Señal; Riesgo y Gestión; Régimen y Filtro; Portafolio y Correlación si la estrategia participa en una cartera co-evolucionada), cada una con sus `wildcard_group` resueltos y su contribución de fitness diferenciada.
    - Al lanzar un ciclo de Generate, el operador elige qué dominio(s) evolucionar en esa corrida (uno, varios, o la co-evolución de cartera de Fase C) y cuáles permanecen congelados desde el Manifest base — incluyendo la opción de línea base actual de evolucionar solo el Dominio de Señal con todo lo demás fijo.
    - El reporte de robustez de Validate desglosa, por dominio, la sensibilidad de la estrategia a la perturbación de su genoma respectivo, permitiendo identificar si la fragilidad proviene de la Señal, del esquema de Riesgo, del Filtro de Régimen o de la interacción de Portafolio.

*   **Costo:**
    - Refactorización del motor de `wildcard_group` para soportar múltiples espacios de nombres de genoma dentro de un mismo AST en lugar de un único bloque homogéneo de wildcards.
    - Extensión del fitness multi-objetivo de NSGA-II para reportar atribución por dominio.
    - Mayor superficie de configuraciones válidas del pipeline Generate→Validate: cada combinación de dominios activos (incluida la co-evolución de cartera) es una configuración distinta que debe quedar cubierta por la batería de robustez correspondiente.

*   **Trazabilidad:** [`ast-compiler.md`](./features/ast-compiler.md), [`nsga2-optimizer.md`](./features/nsga2-optimizer.md), [`monte-carlo-simulator.md`](./features/monte-carlo-simulator.md), [`complexity-penalization.md`](./features/complexity-penalization.md), ADR-0043, ADR-0100, ADR-0109, ADR-0110, ADR-0111.

---

### **ADR-0109: Generador Genómico de Riesgo y Gestión de Posición (Fase A) — Wildcard Invertido y Réplica de Estado de Riesgo en Monte Carlo**

*   **Decisión:**
    Instanciar, como primer dominio nuevo del Registro de Dominios Genómicos (ADR-0108), el **Generador Genómico de Riesgo y Gestión de Posición**. Este generador evoluciona, vía NSGA-II, combinaciones de:
    - **Genes de Condición de Estado** (observan el estado de la cuenta, la posición abierta y la sesión en tiempo real): drawdown de equity, racha de pérdidas consecutivas, racha de ganancias consecutivas, duración del drawdown actual, ratio de volatilidad (ATR) actual frente a histórico, desequilibrio de volumen, tiempo hasta la apertura de sesión, duración de la operación en barras, múltiplo-R no realizado, y distancia porcentual al Stop Loss.
    - **Genes de Acción de Mutación de Tamaño** (alteran el dimensionamiento de la siguiente operación o de la posición abierta): multiplicador de factor sobre el tamaño base, riesgo porcentual recalculado sobre equity, dimensionamiento Kelly acotado a un riesgo máximo, y riesgo monetario fijo independiente del tamaño de cuenta.
    - **Genes de Acción de Morfología de Salida** (alteran la estructura de cierre de la posición): división de la posición en N fases de salida, disparador de salida parcial al alcanzar un múltiplo-R determinado con un porcentaje de volumen dado, movimiento del Stop Loss hacia un precio objetivo derivado de una fase de salida ya alcanzada, y salida por decaimiento temporal tras un número de barras con reducción gradual de la posición.

    Este genoma se evoluciona bajo el patrón **"Wildcard Invertido"**: a diferencia de ADR-0043 (donde el operador fija filtros/salidas y el motor descubre la entrada), aquí el **Genoma de Señal permanece congelado** (la lógica de entrada/salida ya validada de la estrategia) y es el **Genoma de Riesgo y Gestión** el que el motor evoluciona libremente sobre ese esqueleto fijo.

    Como precondición de robustez **bloqueante** para cualquier resultado de este generador, se extiende [monte-carlo-simulator.md](./features/monte-carlo-simulator.md) con el modo **"Réplica de Estado de Riesgo"**: en cada iteración de remuestreo del Modo 1 (reordenamiento de operaciones), la máquina de estados de los Genes de Condición de este genoma (rachas, drawdown, duración) se **re-simula desde cero** sobre la secuencia reordenada, re-evaluando en cada paso qué Genes de Acción se hubieran disparado bajo esa secuencia alternativa. El backtest histórico original deja de ser la única fuente de verdad sobre qué mutaciones de riesgo ocurrieron: cada reordenamiento genera su propia trayectoria de mutaciones.

*   **Objetivo:**
    El caso original que motivó esta iniciativa fue concreto: "si pierdo 3 veces seguidas, reduzco el riesgo a 0.2% y redistribuyo el Stop Loss en 3 fases y el Take Profit en 2". Modelar esto como un perfil de riesgo predefinido obliga a un diseñador humano a anticipar manualmente cada combinación relevante de condición→acción, exactamente el cuello de botella que WildCards ya eliminó para la lógica de entrada. Separar el espacio en Genes de Condición (qué observar) y Genes de Acción (qué hacer) permite que NSGA-II descubra combinaciones no intuitivas — por ejemplo, que la racha de pérdidas combinada con baja liquidez de sesión sea el disparador relevante, no la racha por sí sola — sin que el operador tenga que enumerarlas a priori.

    La Réplica de Estado de Riesgo es **el requisito técnico más crítico de este ADR**. Sin ella, el Monte Carlo actual —que reordena el PnL ya materializado— es estructuralmente ciego a genomas de riesgo dependientes del estado: el orden histórico de las operaciones determinó qué mutaciones de riesgo se dispararon y cuándo, por lo que el PnL observado ya incorpora esas decisiones bajo esa secuencia particular. Reordenar el PnL sin re-disparar la máquina de estados de riesgo produciría una curva de equidad que mezcla decisiones de riesgo tomadas bajo una secuencia con resultados de otra — un resultado internamente inconsistente que invalidaría cualquier conclusión de robustez sobre este genoma. Por eso esta capacidad se declara **bloqueante**: ningún genoma del Dominio de Riesgo y Gestión puede avanzar a la fase "En Incubación" del ciclo de vida (SAD §12) sin haber pasado por la Réplica de Estado de Riesgo.

*   **Reglas:**
    - **Wildcard Invertido (FIJO):** mientras el Genoma de Riesgo y Gestión está activo para evolución, el Genoma de Señal del mismo Manifest queda bloqueado/congelado, salvo que `ACTIVE_GENOME_DOMAINS` también incluya el Dominio de Señal — en cuyo caso ambos evolucionan conjuntamente como genoma compuesto bajo la regla general de ADR-0108.
    - **Réplica de Estado de Riesgo Obligatoria (FIJO):** todo Manifest cuyo Genoma de Riesgo y Gestión contenga al menos un Gen de Condición de Estado debe pasar por el modo "Réplica de Estado de Riesgo" del Monte Carlo antes de avanzar en el ciclo de vida de la estrategia (SAD §12). Resultados de backtest o WFA simple no son suficientes.
    - **Catálogo Cerrado de Primitivas de Acción (CONFIG, extensible solo vía nuevo ADR):** los Genes de Acción de Mutación de Tamaño y de Morfología de Salida deben mapear exclusivamente a comportamientos ya expuestos como parámetros configurables por features existentes ([precision-sizing-models.md](./features/precision-sizing-models.md), [kinetic-micro-management.md](./features/kinetic-micro-management.md), [advanced-trade-management.md](./features/advanced-trade-management.md), [multi-ticket-manager.md](./features/multi-ticket-manager.md)). El generador no inventa nuevos mecanismos de ejecución; combina y parametriza los existentes.
    - **Límites de Profundidad de Morfología (CONFIG):** el número máximo de fases de salida, el número máximo de movimientos de Stop Loss encadenados y la profundidad máxima de condiciones anidadas en el genoma de riesgo son límites configurables auditados por [complexity-penalization.md](./features/complexity-penalization.md), de la misma forma que los límites existentes de complejidad del Dominio de Señal.
    - **Reproducibilidad Determinista (FIJO):** la re-simulación de la máquina de estados de riesgo durante la Réplica de Estado debe ser bit-a-bit determinista dado el mismo `manifest_id` y la misma semilla de remuestreo, en cumplimiento de ADR-0107 (paridad investigación-producción).

*   **Implementación:**
    - El operador, al diseñar o revisar una estrategia con Genoma de Señal ya validado, lanza un ciclo de Generate específico para el Dominio de Riesgo y Gestión. El motor devuelve un conjunto Pareto de genomas de riesgo candidatos, cada uno mostrando sus Genes de Condición activos y los Genes de Acción que disparan.
    - El reporte de Validate de cada candidato incluye, además de las métricas estándar, una vista de trayectorias de mutación de riesgo bajo remuestreo: cuántas veces y bajo qué secuencias alternativas se habría activado cada Gen de Acción, y cómo varía el drawdown resultante.
    - En ejecución en vivo, el operador observa en tiempo real qué Genes de Condición están actualmente activos para la posición o cuenta corriente y qué Gen de Acción se disparó la última vez.

*   **Costo:**
    - Implementación del modo Réplica de Estado de Riesgo en [monte-carlo-simulator.md](./features/monte-carlo-simulator.md): requiere mantener y re-evaluar una máquina de estados completa (no solo PnL agregado) en cada iteración de remuestreo, incrementando significativamente el costo computacional del Modo 1 cuando el Genoma de Riesgo está presente.
    - Extensión de [precision-sizing-models.md](./features/precision-sizing-models.md), [kinetic-micro-management.md](./features/kinetic-micro-management.md), [advanced-trade-management.md](./features/advanced-trade-management.md) y [multi-ticket-manager.md](./features/multi-ticket-manager.md) para exponer sus comportamientos hoy fijos/binarios como Primitivas de Acción parametrizables y direccionables individualmente por el genoma evolutivo.
    - Extensión de [complexity-penalization.md](./features/complexity-penalization.md) con un nuevo eje de grados de libertad correspondiente al Genoma de Riesgo y Gestión.

*   **Trazabilidad:** [`ast-compiler.md`](./features/ast-compiler.md), [`nsga2-optimizer.md`](./features/nsga2-optimizer.md), [`monte-carlo-simulator.md`](./features/monte-carlo-simulator.md), [`complexity-penalization.md`](./features/complexity-penalization.md), [`precision-sizing-models.md`](./features/precision-sizing-models.md), [`kinetic-micro-management.md`](./features/kinetic-micro-management.md), [`advanced-trade-management.md`](./features/advanced-trade-management.md), [`multi-ticket-manager.md`](./features/multi-ticket-manager.md), ADR-0043, ADR-0107, ADR-0108.

---

### **ADR-0110: Generador Genómico de Régimen y Filtro de Entorno (Fase B) — Máscaras de Permiso/Prohibición por Estructura de Mercado**

*   **Decisión:**
    Instanciar el **Generador Genómico de Régimen y Filtro de Entorno** como segundo dominio nuevo del Registro (ADR-0108). Este generador evoluciona, vía NSGA-II, combinaciones de:
    - **Genes de Condición de Estructura de Mercado:** exponente de Hurst (persistencia o anti-persistencia de la serie de precios), entropía de Shannon del volumen (grado de aleatoriedad del flujo de órdenes), pendiente multinivel de medias móviles Hull (estructura de tendencia en múltiples horizontes simultáneos), y estado latente dominante de un modelo de Markov oculto (HMM).
    - **Genes de Acción de Permiso:** a diferencia de los Dominios de Señal y de Riesgo (que producen acciones paramétricas continuas o discretas), el espacio de acción de este dominio es deliberadamente el más simple de los tres dominios nuevos: cada combinación de Genes de Condición evoluciona hacia un veredicto binario de **Permitido / Prohibido** que se aplica como máscara de entrada al Genoma de Señal (congelado) del mismo Manifest.

    La robustez de este genoma se valida mediante una extensión segmentada por régimen del análisis Walk-Forward existente ([walk-forward-analyzer.md](./features/walk-forward-analyzer.md) / [cross-market-validation.md](./features/cross-market-validation.md)): además de las ventanas temporales estándar, se incorporan ventanas curadas correspondientes a regímenes históricos de referencia conocidos por su carácter extremo (capitulaciones de alta volatilidad, mercados de rango prolongado, tendencias de baja volatilidad sostenida), de forma que un genoma de régimen no pueda sobreajustarse a "permitir siempre" simplemente porque el dataset general de entrenamiento fue mayormente favorable.

*   **Objetivo:**
    Este dominio generaliza y formaliza, bajo la misma gramática Condición→Acción, tres mecanismos que ya existían de forma dispersa y con alcance fijo: [vector-time-pruning.md](./features/vector-time-pruning.md) (vetos por ventana temporal y eventos macro), [regime-guard.md](./features/regime-guard.md) (matriz de compatibilidad estrategia-régimen basada en HMM) y [hmm-regime-detection.md](./features/hmm-regime-detection.md) (clasificación de régimen). Estos tres mecanismos comparten la misma forma — observar una condición de entorno y emitir un veredicto de permiso — pero hoy son lógicas fijas, no genomas evolutivos. Convertir el espacio de condiciones de entorno (incorporando además genes de estructura de mercado más exóticos como Hurst y entropía de Shannon, no cubiertos por los mecanismos actuales) en un genoma evolutivo permite descubrir combinaciones de filtros de régimen no obvias, en lugar de depender únicamente de los umbrales y matrices que un operador definió a priori.

    Se eligió deliberadamente el espacio de acción binario (Permitido/Prohibido) — el más simple de los tres dominios nuevos — porque el riesgo de sobreajuste de este dominio no está en la complejidad de la acción sino en la complejidad y exotismo de las condiciones: Hurst, entropía de Shannon y estados HMM multinivel son medidas estadísticamente sutiles y fáciles de "memorizar" sobre un dataset finito. Concentrar la simplicidad en la acción y reforzar la validación de robustez (WFA segmentado por régimen) es la combinación de costo/beneficio correcta para este dominio.

*   **Reglas:**
    - **Espacio de Acción Cerrado a Permiso Binario (FIJO):** los Genes de Acción de este dominio solo pueden producir un veredicto Permitido/Prohibido aplicado como máscara sobre el Genoma de Señal. Cualquier acción paramétrica (ajuste de tamaño, modificación de salida) pertenece a los Dominios de Riesgo y Gestión (ADR-0109) o Portafolio y Correlación (ADR-0111), no a este.
    - **Wildcard Invertido (FIJO, heredado de ADR-0109):** el Genoma de Señal permanece congelado mientras este genoma evoluciona, salvo que `ACTIVE_GENOME_DOMAINS` también lo incluya. Si el Manifest también posee activo el Genoma de Riesgo y Gestión (ADR-0109) y/o de Portafolio y Correlación (ADR-0111), todos los dominios en `ACTIVE_GENOME_DOMAINS` evolucionan conjuntamente como genoma compuesto (Regla Genómica cruzada, ADR-0108), aplicando la atribución de complejidad correspondiente.
    - **WFA Segmentado por Régimen Obligatorio (FIJO):** todo Manifest con un Genoma de Régimen y Filtro activo debe pasar por la extensión segmentada por régimen de [walk-forward-analyzer.md](./features/walk-forward-analyzer.md)/[cross-market-validation.md](./features/cross-market-validation.md) antes de avanzar en el ciclo de vida (SAD §12). El WFA estándar no segmentado es insuficiente para este dominio.
    - **Catálogo Cerrado de Genes de Condición (CONFIG, extensible solo vía nuevo ADR):** Hurst, entropía de Shannon de volumen, pendiente multinivel de Hull MA y estado HMM son el conjunto inicial. Nuevos genes de condición de estructura de mercado requieren validar primero que son computables determinísticamente sobre los datos históricos disponibles (criterio de admisión de ADR-0108).
    - **No Reemplazo de Mecanismos Existentes (FIJO):** [vector-time-pruning.md](./features/vector-time-pruning.md) y [regime-guard.md](./features/regime-guard.md) no se eliminan; este dominio los generaliza como una capa evolutiva adicional. Las reglas fijas/aprendidas por esos mecanismos (vetos temporales, matriz HMM de compatibilidad) siguen aplicando como filtros de base; el Genoma de Régimen y Filtro opera como una capa de refinamiento evolutivo sobre ellas.

*   **Implementación:**
    - El operador, sobre una estrategia con Genoma de Señal ya validado, lanza un ciclo de Generate para el Dominio de Régimen y Filtro. El motor devuelve un conjunto Pareto de combinaciones de Genes de Condición de estructura de mercado, cada una con su matriz de Permiso/Prohibición resultante.
    - El reporte de Validate muestra, para cada candidato, su desempeño desglosado por cada ventana de régimen curada (capitulación, rango, tendencia de baja volatilidad), evidenciando si el filtro generaliza o si solo "memoriza" el régimen dominante del histórico de entrenamiento.
    - En ejecución en vivo, el operador ve en tiempo real el veredicto Permitido/Prohibido vigente y qué Gen de Condición lo está sosteniendo (por ejemplo, "Prohibido: Hurst por debajo del umbral — régimen de reversión a la media detectado").

*   **Costo:**
    - Implementación de los Genes de Condición de estructura de mercado exótica (Hurst, entropía de Shannon, pendientes Hull multinivel) como indicadores computables de forma determinista y eficiente sobre el histórico — capacidad nueva, no derivada de mecanismos existentes.
    - Extensión de [hmm-regime-detection.md](./features/hmm-regime-detection.md) para exponer el estado latente dominante como gen de condición consumible por el genoma evolutivo, además de su uso actual en [regime-guard.md](./features/regime-guard.md).
    - Extensión de [walk-forward-analyzer.md](./features/walk-forward-analyzer.md) y [cross-market-validation.md](./features/cross-market-validation.md) para soportar ventanas curadas de régimen como dimensión adicional de segmentación, y de [complexity-penalization.md](./features/complexity-penalization.md) con el eje de grados de libertad de este genoma.

*   **Trazabilidad:** [`ast-compiler.md`](./features/ast-compiler.md), [`nsga2-optimizer.md`](./features/nsga2-optimizer.md), [`walk-forward-analyzer.md`](./features/walk-forward-analyzer.md), [`cross-market-validation.md`](./features/cross-market-validation.md), [`vector-time-pruning.md`](./features/vector-time-pruning.md), [`regime-guard.md`](./features/regime-guard.md), [`hmm-regime-detection.md`](./features/hmm-regime-detection.md), [`complexity-penalization.md`](./features/complexity-penalization.md), ADR-0046, ADR-0108.

---

### **ADR-0111: Generador Genómico de Portafolio y Correlación (Fase C) — Co-evolución de Cartera y Monte Carlo de Desfase Temporal**

*   **Decisión:**
    Instanciar el **Generador Genómico de Portafolio y Correlación** como tercer dominio nuevo del Registro (ADR-0108) — el de mayor complejidad estructural de los tres, porque su unidad de evolución no es una estrategia individual sino un **conjunto co-evolucionado de estrategias** (una cartera). Este generador evoluciona, vía NSGA-II aplicado sobre la población de la cartera completa, combinaciones de:
    - **Genes de Condición Cruzada (entre estrategias):** correlación móvil entre las curvas de equidad de los miembros de la cartera, volatilidad agregada del portafolio, solapamiento de operaciones simultáneas en la misma dirección entre miembros, y drawdown agregado del portafolio.
    - **Genes de Acción de Cartera:** activación o desactivación de un miembro de la cartera condicionada a la racha o estado de otro miembro, rotación dinámica del peso de capital entre miembros, e inyección de cobertura sintética (posición compensatoria temporal) cuando las condiciones cruzadas lo ameritan.

    La robustez de este genoma requiere una capacidad de Monte Carlo que **no existe hoy**: el **"Monte Carlo de Desfase Temporal"**. A diferencia del Monte Carlo del Modo 1 (que remuestrea la secuencia de operaciones de una sola estrategia), este nuevo modo remuestrea, para cada miembro de la cartera, un desfase temporal independiente sobre su propia secuencia de operaciones, recombinando las curvas de equidad desfasadas para evaluar si la correlación y el drawdown agregado observados en el backtest conjunto son un artefacto de la alineación temporal específica del histórico, o si persisten bajo desalineaciones plausibles.

*   **Objetivo:**
    Este dominio generaliza y formaliza [fit-to-portfolio-search.md](./features/fit-to-portfolio-search.md) (presión de fitness por penalización de correlación durante la generación, hoy estática y de un solo eje), [portfolio-optimizer.md](./features/portfolio-optimizer.md) y [portfolio-rules.md](./features/portfolio-rules.md). La limitación común de estos mecanismos es que tratan la relación entre estrategias de una cartera como una restricción estática aplicada en el momento de selección o generación, no como un espacio evolutivo de comportamiento dinámico entre miembros. El caso de uso que motivó toda esta iniciativa (mutación drástica de comportamiento ante secuencias de eventos) aplica con la misma fuerza a nivel de cartera: "si la estrategia A entra en racha de pérdidas, rotar capital hacia B" es estructuralmente el mismo patrón Condición→Acción que "si pierdo 3 veces seguidas, reduzco mi riesgo" — solo que la condición y la acción operan sobre el estado de otros miembros de la cartera, no sobre el propio.

    Es, de los tres dominios nuevos, el de **mayor complejidad** porque introduce dos quiebres respecto a ADR-0109/ADR-0110: (1) la unidad de evolución es la cartera, no la estrategia individual, lo que requiere optimización sobre una población de poblaciones; y (2) la validación de robustez requiere una nueva capacidad de Monte Carlo (Desfase Temporal) que debe construirse desde cero, a diferencia de ADR-0109 (que extiende un modo existente) y ADR-0110 (que extiende WFA existente).

*   **Reglas:**
    - **Co-evolución a Nivel de Cartera (FIJO, eje ortogonal):** este es el único dominio del Registro (ADR-0108) donde la unidad evolutiva es un conjunto de Manifests, no un Manifest individual — eje independiente de `ACTIVE_GENOME_DOMAINS` (que define qué dominios evolucionan *dentro* de cada Manifest miembro). Por defecto cada Manifest miembro mantiene sus demás genomas (Señal, Riesgo y Gestión, Régimen y Filtro) congelados durante la evolución del Genoma de Portafolio y Correlación; si el `ACTIVE_GENOME_DOMAINS` de un miembro incluye además otros dominios, ese miembro evoluciona su genoma compuesto simultáneamente con la co-evolución de cartera.
    - **Monte Carlo de Desfase Temporal Obligatorio (FIJO):** todo conjunto de Manifests con un Genoma de Portafolio y Correlación activo debe pasar por el nuevo modo "Desfase Temporal" antes de avanzar en el ciclo de vida (SAD §12). Esta capacidad se declara, igual que la Réplica de Estado de Riesgo de ADR-0109, como **bloqueante**: sin ella, la correlación medida entre miembros es un artefacto del histórico específico, no una propiedad robusta de la cartera.
    - **Tamaño Mínimo de Cartera (CONFIG):** el número mínimo de miembros necesario para que los Genes de Condición Cruzada de este genoma sean estadísticamente significativos es un umbral configurable, análogo en espíritu a `MIN_TRADES_PER_PARAM` de [complexity-penalization.md](./features/complexity-penalization.md) pero a nivel de "miembros mínimos por cartera".
    - **Origen de Miembros desde Genomas Validados (FIJO):** los miembros candidatos a una cartera co-evolucionada deben provenir de Manifests cuyos Genomas de Señal (y, si aplica, de Riesgo y Gestión y de Régimen y Filtro) ya hayan superado independientemente su propia batería de robustez (ADR-0109/ADR-0110/línea base). Este dominio no co-evoluciona estrategias crudas sin validar — evita que la optimización de cartera "rescate" miembros individualmente frágiles mediante artefactos de correlación.
    - **Acción de Cobertura Sintética Acotada (CONFIG):** la inyección de cobertura sintética debe mapear a primitivas de ejecución ya existentes, mismo principio del catálogo cerrado de ADR-0109; no introduce nuevos tipos de instrumento ni nuevos mecanismos de ejecución.

*   **Implementación:**
    - El operador selecciona un conjunto de estrategias ya validadas individualmente como población base de una cartera, y lanza un ciclo de Generate para el Dominio de Portafolio y Correlación. El motor devuelve un conjunto Pareto de configuraciones de cartera, cada una con sus Genes de Condición Cruzada activos y sus Genes de Acción de rotación, activación o cobertura.
    - El reporte de Validate de cartera muestra, para cada configuración candidata, el resultado del Monte Carlo de Desfase Temporal: cómo varían la correlación agregada, el drawdown de cartera y el Sharpe conjunto bajo desalineaciones temporales plausibles entre los miembros.
    - En ejecución en vivo, el operador ve el estado actual de cada Gen de Condición Cruzada (por ejemplo, "Correlación Estrategia A↔B: 0.62 — por encima del umbral") y qué Gen de Acción de cartera está vigente (por ejemplo, "Rotación activa: 70% del capital hacia Estrategia B").

*   **Costo:**
    - Diseño e implementación desde cero del Monte Carlo de Desfase Temporal en [monte-carlo-simulator.md](./features/monte-carlo-simulator.md) — la capacidad de mayor costo de todo el conjunto de ADR-0109 a ADR-0111, al no existir un modo previo del que partir.
    - Extensión de [fit-to-portfolio-search.md](./features/fit-to-portfolio-search.md), [portfolio-optimizer.md](./features/portfolio-optimizer.md) y [portfolio-rules.md](./features/portfolio-rules.md) para exponer sus reglas hoy estáticas (tope de correlación, pesos) como Genes de Acción parametrizables por el genoma evolutivo de cartera.
    - Costo computacional del NSGA-II operando sobre una población de poblaciones: cada evaluación de fitness de un individuo de cartera requiere simular el conjunto completo de miembros bajo el genoma candidato.

*   **Trazabilidad:** [`ast-compiler.md`](./features/ast-compiler.md), [`nsga2-optimizer.md`](./features/nsga2-optimizer.md), [`monte-carlo-simulator.md`](./features/monte-carlo-simulator.md), [`fit-to-portfolio-search.md`](./features/fit-to-portfolio-search.md), [`portfolio-optimizer.md`](./features/portfolio-optimizer.md), [`portfolio-rules.md`](./features/portfolio-rules.md), [`complexity-penalization.md`](./features/complexity-penalization.md), ADR-0050, ADR-0089, ADR-0108.

---

### **ADR-0112: Veredicto SPIKE-002 — Erradicación de `tch-rs`/libtorch; Escalera de Cómputo Numérico Soberano (`ndarray`/Rayon → `candle` → `burn`)**

*   **Decisión:**
    Eliminar de forma total e irreversible la dependencia `tch-rs` (y con ella libtorch) de toda la arquitectura. El cómputo numérico pesado (Monte Carlo, autoencoders de outliers, reducción dimensional) se resuelve mediante una **escalera de adopción estrictamente ordenada**, sin saltar peldaños sin justificación medida:
    1. **`ndarray` + Rayon (primera línea, default):** álgebra lineal y permutación matricial en CPU multihilo. Cubre Monte Carlo y la mayoría de las cargas, porque ninguna de ellas es deep learning real.
    2. **`candle` (Rust puro, segunda línea):** solo si `ndarray`/Rayon se demuestran insuficientes para una carga concreta. Backends CUDA/Metal que se cargan dinámicamente **solo si hay GPU presente**; degrada a CPU sin GPU. Cero libtorch.
    3. **`burn` (tercera línea, reservado al moonshot DRL):** solo si se materializa el moonshot de Deep Reinforcement Learning *y* `candle` se demuestra insuficiente. Backend-agnóstico, lo que permite empezar en `ndarray`/wgpu y migrar.

*   **Objetivo:**
    `tch-rs` arrastra libtorch (~2GB de C++), lo que rompe dos invariantes FIJAS: el "binario único, instalador diminuto, 3 OS" del **ADR-0029** y la soberanía "sin runtimes pesados" del **ADR-0030**. Ninguna de las cargas que motivaron `tch-rs` lo necesita: Monte Carlo es barajado/permutación de matrices (no deep learning); autoencoders y UMAP son redes pequeñas resolubles en CPU pura. El único caso que históricamente justifica un backend tipo PyTorch es el DRL, que está deliberadamente diferido a moonshot de fase lejana. Cargar 2GB de dependencia nativa para un caso que aún no existe es exactamente la complejidad sin Alpha que el sistema rechaza.

*   **Reglas:**
    - **Erradicación Total (FIJO):** ningún crate del workspace declara `tch-rs` como dependencia, ni siquiera opcional, mientras no exista el moonshot DRL. La promesa de binario único del ADR-0029 es la invariante que esta decisión protege.
    - **Carga Condicional de GPU (FIJO):** cuando se llegue a `candle`/`burn`, el soporte CUDA/Metal se activa por detección en runtime; la ausencia de GPU jamás impide la ejecución (degradación a CPU), en cumplimiento del ADR-0032 (Single Machine Sovereignty).
    - **Ascenso por Evidencia (CONFIG/Gobernanza):** subir un peldaño de la escalera exige un benchmark documentado que demuestre que el peldaño inferior no alcanza. No se adopta `candle` "por si acaso".
    - **Determinismo (FIJO):** el fallback CPU debe preservar la semilla aleatoria y el determinismo bit-a-bit (ADR-0107), igual que ya exige el Monte Carlo.

*   **Implementación:**
    El binario de distribución pesa megabytes, no gigabytes, y se instala de un clic en los 3 OS. El Monte Carlo de 10K iteraciones corre en CPU con Rayon; si algún día un autoencoder lo exige, `candle` entra como dependencia Rust pura sin alterar el modelo de empaquetado.

*   **Costo:**
    Reescribir las rutas que asumían GPU/VRAM (Monte Carlo, suite de IA) para CPU-first. Es trabajo de migración acotado y elimina deuda de empaquetado.

*   **Trazabilidad:** [`monte-carlo-simulator.md`](./features/monte-carlo-simulator.md), [`autoencoder-outlier-detector.md`](./features/autoencoder-outlier-detector.md), [`statistical-inference-ebta.md`](./features/statistical-inference-ebta.md), ADR-0029, ADR-0030, ADR-0031, ADR-0032, ADR-0047, ADR-0061, ADR-0104.

---

### **ADR-0113: Veredicto SPIKE-003 — Erradicación de PySR; Regresión Simbólica como Modo del Motor Genético Nativo y Diferimiento de la Minería Simbólica Libre a Moonshot (`egg`)**

*   **Decisión:**
    Eliminar PySR de toda la arquitectura. La capacidad de "regresión simbólica" se resuelve en dos planos:
    1. **En el MVP:** la regresión simbólica acotada se reconoce como lo que es —**programación genética sobre árboles de expresión con un frente de Pareto precisión/complejidad**— y por tanto es un **modo del motor NSGA-II nativo ya especificado** ([`nsga2-optimizer.md`](./features/nsga2-optimizer.md)) sobre el contrato AST existente ([`ast-compiler.md`](./features/ast-compiler.md)), no una dependencia nueva.
    2. **La minería simbólica de forma libre** (descubrir ecuaciones matemáticas arbitrarias sin esqueleto humano) se **difiere a `/moonshots/`**, donde se designa la librería **`egg` (e-graphs, Rust puro)** como tecnología recomendada para la saturación de equivalencias y el control de *bloat* algebraico.
    Se **rechazan explícitamente** los evaluadores de expresiones por string en runtime (`evalexpr`, `meval`) para cualquier ruta de minería.

*   **Objetivo:**
    PySR es Python+Julia (SymbolicRegression.jl): no existe puerto Rust y viola frontalmente el **ADR-0104**. Pero además es innecesario: la regla maestra del MVP ("copia descarada de SQX") lo confirma — SQX **no hace** regresión simbólica libre; hace ensamblado genético sobre un catálogo cerrado de bloques, que es exactamente el patrón WildCards/Condición→Acción del ADR-0043/0108 ya diseñado. El núcleo de NSGA-II (non-dominated sorting + crowding distance) es código propietario de ~250 líneas; el valor vive en la representación del genoma y el fitness por dominio, no en un framework externo. `egg` es genuinamente excelente, pero su feature asesina (saturación algebraica anti-*bloat*) solo cobra valor sobre **álgebra libre**, no sobre la gramática de catálogo cerrado del MVP, donde el *bloat* se controla con `MAX_CONDITIONS_PER_RULE` y Ockham. Los evaluadores `evalexpr`/`meval` parsean y caminan un árbol genérico por evaluación, con asignaciones en heap: meterlos en el loop de minería destruiría el rendimiento (anti-patrón frente a ADR-0047).

*   **Reglas:**
    - **Cero PySR / Cero Python (FIJO):** heredado del ADR-0104. Ningún documento de arquitectura o feature nombra PySR como tecnología.
    - **AST Tipado, no Evaluador de Strings (FIJO):** el AST es un `enum` tipado de Rust evaluado por dos vías — compilación a expresiones columnar Polars (ruta vectorizada) o `match` plano (ruta secuencial). Prohibido introducir `evalexpr`/`meval` en rutas de minería; su único uso tolerable sería una fórmula tecleada por el usuario en una UI de configuración.
    - **`egg` Parqueado, no Importado (Gobernanza):** `egg` solo entra al workspace cuando se active el moonshot de minería simbólica libre, no antes.
    - **Higiene de Dependencias (FIJO, ADR-0020/Anti-Obsolescencia):** los nombres de crates de GA "de catálogo" propuestos en análisis exploratorios (p. ej. `oxen`, `rhea`, `oxigene`, `rs-genetic`) **no se adoptan**: varios no son crates de GA reales/mantenidos. El motor genético es nativo; Rayon es la única dependencia de paralelismo.

*   **Implementación:**
    El operador evoluciona estrategias con el mismo motor NSGA-II nativo, eligiendo dominios genómicos (ADR-0108). Si en el futuro se quiere descubrimiento de ecuaciones libres, el moonshot lo provee con `egg`, sin tocar el motor del MVP.

*   **Costo:**
    Purgar las menciones residuales a PySR en ADR-0031/0057 y en las features. Reconocer la regresión simbólica como modo del motor existente no añade infraestructura nueva.

*   **Trazabilidad:** [`nsga2-optimizer.md`](./features/nsga2-optimizer.md), [`ast-compiler.md`](./features/ast-compiler.md), [`glass-box-ai-translator.md`](./features/glass-box-ai-translator.md), [`moonshots/pysr-signal-discovery.md`](./moonshots/pysr-signal-discovery.md), ADR-0031, ADR-0043, ADR-0057, ADR-0104, ADR-0108.

---

### **ADR-0114: Veredicto SPIKE-004 — Motor de Backtest Dual con Ruta Express Híbrida (Vectorizada + Secuencial), Modo de Motor Elegido por el Usuario y Contrato de Consistencia Conservadora**

*   **Decisión:**
    Formalizar el simulador como **motor dual** con dos rutas independientes en código unidas por un contrato de consistencia:
    - **Ruta Express (exploración/minería):** enfoque **híbrido** en dos sub-fases. (1) Pre-cálculo **vectorizado** columnar (Polars/SIMD) de toda la lógica **sin estado** —indicadores, señales, genes de Condición de los Dominios de Señal y de Régimen/Filtro—, produciendo arrays de señales. (2) Un **mini-loop secuencial plano** en Rust que consume esas señales y resuelve la lógica **con estado** —sizing, stops dinámicos, curva de capital y el Dominio de Riesgo y Gestión (ADR-0109) completo—. Modo de datos: 1m OHLC con 4 fases.
    - **Ruta Event-Driven (validación de alta fidelidad):** crates nativos de NautilusTrader v2 (ADR-0107), con ticks reales (mercados centralizados) o reconstrucción de pseudo-ticks (OTC). Garantiza paridad simulación/vivo.

    **La elección de ruta es del usuario, no del sistema.** El modo de motor (`Express | EventDriven`) es un parámetro del contrato público del simulador, provisto por el llamador, con default por contexto pero **siempre anulable** en cualquier fase/módulo donde se invoque el simulador. El sistema nunca fuerza la promoción de una ruta a otra.

    Se **elimina el KPI absoluto** de "100K–500K bars/sec" de toda la documentación: era una métrica de vanidad. El criterio de rendimiento pasa a ser **relativo y competitivo**: ser medible y demostrablemente más rápido que MetaTrader 5, StrategyQuant X y QuantConnect en la misma máquina.

*   **Objetivo:**
    Forzar un único motor event-driven tick-a-tick a la minería masiva es un error arquitectónico (asfixia la exploración); forzar un único motor vectorizado puro impide modelar la gestión de riesgo dependiente de estado (ADR-0109) y rompe la paridad sim/vivo del ADR-0107. El enfoque híbrido disuelve la falsa dicotomía: **se vectoriza lo que no tiene memoria y se recorre en orden lo que sí la tiene**, y esa frontera coincide exactamente con la división de Dominios Genómicos del ADR-0108 (Señal/Régimen sin estado → vectorizado; Riesgo/Gestión con estado → secuencial). El mini-loop secuencial es además la estructura que hace viable la Réplica de Estado de Riesgo del Monte Carlo (ADR-0109). El diferencial frente a SQX (que paraleliza pero recalcula indicadores por candidata) es pre-calcular los indicadores **una sola vez** de forma vectorizada y reutilizarlos en toda la población.

*   **Reglas:**
    - **Contrato de Consistencia Conservadora (FIJO):** como el usuario puede promover a incubación un resultado solo-Express, la Ruta Express **nunca puede ser más optimista que la Event-Driven**. Se garantiza por dos reglas inflexibles: **(a) Bar-Open Alignment obligatorio** —toda señal calculada al cierre de la barra N se ejecuta al precio de apertura de la barra N+1, sin entradas a precios intermedios— y **(b) Regla Intrabar Pesimista** —si SL y TP se alcanzan dentro de la misma vela, se asume siempre que el Stop Loss se ejecutó primero—.
    - **Frontera Sin-Estado / Con-Estado (FIJO):** ningún cálculo dependiente del estado de cuenta/posición se ejecuta en la sub-fase vectorizada; ningún cálculo puramente analítico sobre datos de mercado obliga al mini-loop secuencial.
    - **Modo de Motor como Parámetro del Contrato (FIJO):** el simulador expone `Express | EventDriven` como entrada del llamador. El sistema no introduce gates automáticos de promoción.
    - **Compuertas de Robustez Independientes (FIJO):** la libertad de elegir Express no exime de las compuertas bloqueantes — la Réplica de Estado de Riesgo (ADR-0109) y el Desfase Temporal de cartera (ADR-0111) siguen siendo obligatorias antes de incubar los genomas correspondientes, sea cual sea la ruta elegida.
    - **Criterio de Rendimiento Relativo (CONFIG/Gobernanza):** prohibido reintroducir KPIs absolutos de throughput como criterio de salida. El benchmark de CI compara contra plataformas de referencia, no contra un número fijo.

*   **Implementación:**
    El operador lanza una corrida nocturna en Ruta Express y obtiene miles de candidatas evaluadas con sesgo pesimista; selecciona finalistas y las re-corre en Ruta Event-Driven para confirmar paridad antes de incubar. La curva de equidad del Motor B y la operación en vivo emergen del mismo loop de eventos determinista (ADR-0107).

*   **Costo:**
    Mantener dos rutas de código y una suite que verifique el contrato de consistencia (que Express jamás supere a Event-Driven en optimismo). Es el costo que compra simultáneamente velocidad de descubrimiento y confianza institucional.

*   **Trazabilidad:** [`backtest-engine.md`](./features/backtest-engine.md), [`nsga2-optimizer.md`](./features/nsga2-optimizer.md), [`monte-carlo-simulator.md`](./features/monte-carlo-simulator.md), [`slippage-models.md`](./features/slippage-models.md), [`institutional-metrics.md`](./features/institutional-metrics.md), ADR-0017, ADR-0047, ADR-0107, ADR-0108, ADR-0109.

---

### **ADR-0115: Veredicto SPIKE-005 — Verdict Engine Determinista sin LLM; Erradicación de Ollama como Requisito**

*   **Decisión:**
    El `robustness-verdict-engine` produce, **por defecto y sin ninguna dependencia de LLM**, un **reporte estructurado determinista** en lenguaje natural generado por plantilla a partir del score ponderado y de los puntos de quiebre ya calculados. Se **elimina Ollama como requisito**: el "DEBE operar con LLM local (Ollama)" del ADR-0058 queda derogado. Un LLM local soberano (vía `candle`, modelo cuantizado embebido, nunca un runtime externo) es un **realce estrictamente opcional** detrás de feature flag, jamás una dependencia del camino crítico.

*   **Objetivo:**
    El veredicto en lenguaje natural es confort/UX puro (clasificado Vanidad, EPIC-8 en el ROADMAP), no Alpha. Lo que mueve dinero —el sizing inicial— lo determina el **score determinista** del `robustness-score-aggregator`, que es matemática reproducible; el LLM solo narraría un número que ya existe. Exigir Ollama (un runtime externo que descarga modelos de varios GB y corre como proceso servidor aparte) contradice el binario único del ADR-0029 y la soberanía sin runtimes del ADR-0030. Esta decisión **repara una contradicción interna**, no solo cierra un gate.

*   **Reglas:**
    - **Determinismo del Reporte (FIJO):** el reporte base se deriva por plantilla determinista del score y los breakpoints; mismo input → mismo texto. No depende de muestreo estocástico de ningún modelo.
    - **LLM Opcional y Soberano (CONFIG):** si se habilita, la inferencia es local vía `candle` embebido; **prohibido** Ollama como requisito y **prohibidas** las APIs externas (heredado del ADR-0051).
    - **El Score Manda (FIJO):** el LLM, si está presente, actúa solo como traductor semántico; nunca modifica el score, los parámetros ni emite señales.

*   **Implementación:**
    Al cerrar la validación, el operador ve un veredicto textual claro ("sobrevive en el 98% de las mutaciones; parámetro más sensible: Trailing Stop; punto de quiebre: spread > 2.5 pips") generado de forma determinista, en una instalación de un clic sin runtimes externos. Quien quiera prosa más rica activa el LLM opcional.

*   **Costo:**
    Diseñar la plantilla determinista de reporte (esfuerzo bajo). Se elimina la dependencia operativa de Ollama y su superficie de fallo.

*   **Trazabilidad:** [`robustness-verdict-engine.md`](./features/robustness-verdict-engine.md), [`robustness-score-aggregator.md`](./features/robustness-score-aggregator.md), [`glass-box-ai-translator.md`](./features/glass-box-ai-translator.md), ADR-0029, ADR-0030, ADR-0051, ADR-0058.

---

### **ADR-0116: Veredicto SPIKE-006 — Downsampling Obligatorio en Backend como Condición de la Frontera FFI; `ZeroCopyBuffer` solo para Cargas Masivas**

*   **Decisión:**
    Confirmar `flutter_rust_bridge` (ADR-0029/0019) como viable a escala, bajo una **regla de oro fija**: **nunca se cruza la frontera FFI con más resolución de datos de la que la pantalla puede dibujar**. Toda reducción a resolución de viewport (downsampling) ocurre en el backend Rust (Polars/DuckDB) antes del cruce. Para los casos de transferencia masiva legítima y poco frecuente (carga de un dataset a un visualizador), se usa `ZeroCopyBuffer` transportando buffers Arrow IPC, con validación de lifetime. Los streams de alta frecuencia aplican throttling (p. ej. 100ms) y backpressure **en Rust antes de cruzar**. El modo headless usa gRPC como fallback (ADR-0033).

*   **Objetivo:**
    El supuesto riesgo de "zero-copy de 1M+ puntos" es en gran parte un falso problema: una curva de 1M de puntos se renderiza en ~2000px; transferir el millón es desperdicio. Reducir en backend a lo que el humano puede distinguir elimina la presión sobre la frontera FFI y respeta el ADR-0098 ("Cero Lógica Dimensional en Dart"). Lo que queda es un spike de medición, no un riesgo existencial.

*   **Reglas:**
    - **Downsampling en Backend (FIJO):** prohibido transmitir a Flutter arrays por encima de la resolución útil del viewport; la reducción es responsabilidad de Rust (refuerza ADR-0098).
    - **`ZeroCopyBuffer` Acotado (CONFIG):** reservado a cargas masivas reales; no es el camino por defecto de los streams.
    - **Throttling en Origen (FIJO):** la limitación de frecuencia y el backpressure se aplican en Rust antes del FFI, no en Dart.
    - **Fallback gRPC (FIJO):** el modo headless/VPS degrada a gRPC sin cambiar la lógica de negocio (ADR-0033).

*   **Implementación:**
    El usuario ve curvas y scatter plots fluidos a 120 FPS porque recibe solo los puntos dibujables; al cargar un dataset completo para inspección, el backend entrega un `ZeroCopyBuffer` Arrow puntual.

*   **Costo:**
    Implementar las rutinas de downsampling por viewport y validar la semántica de lifetime del `ZeroCopyBuffer`. El spike EPIC-0 ya planificado mide latencia de stream con throttle y tiempo de transferencia masiva.

*   **Trazabilidad:** [`binary-arrow-transport.md`](./features/binary-arrow-transport.md), ADR-0019, ADR-0029, ADR-0033, ADR-0098.
