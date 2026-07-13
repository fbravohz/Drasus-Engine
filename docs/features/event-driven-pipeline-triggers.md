# Event-Driven Pipeline Triggers

**Carpeta:** `./features/event-driven-pipeline-triggers/`
**Estado:** En Diseño
**Última actualización:** 2026-07-12
**Decisión Arquitectónica Asociada:** ADR-0011 (Operaciones Asincrónicas), ADR-0012 (Arquitectura Multi-Pipeline Paralela), ADR-0150 (Expedition — cada iteración recurrente es una Expedition nueva)

> 🔶 **Resolución de `DEBT-024` (2026-07-12, sesión dedicada con el propietario):** esta feature cubría solo disparo **reactivo** (por condición de mercado/portafolio). Se generaliza a **dos modos de disparo** — ver "Dos Modos de Disparo" abajo. El nombre de la feature no cambia (el disparo reactivo sigue siendo el caso fundacional); el modo recurrente se añade como TTR-003.

---

## ¿Qué es esta feature?

El sistema de disparadores de pipelines automatiza la ejecución de flujos de descubrimiento y validación de estrategias (pipelines de QUANTOPS) sin intervención manual repetida. Cubre **dos modos de disparo** complementarios:

1. **Reactivo** (TTR-001/002, caso fundacional): ante eventos como el incremento de la volatilidad (ej. VIX cruzando un umbral) o cambios en el régimen de mercado, se disparan de forma autónoma secuencias de ingestión, búsqueda genética y validación.
2. **Recurrente** (TTR-003, nuevo): el Pipeline corre en bucle continuo o por intervalo programado — "genera y valida candidatas cada noche" — hasta que el usuario lo detiene explícitamente, no atado a ninguna condición de mercado.

**Problema:** Tradicionalmente, la ejecución de la exploración, optimización o rebalanceo se realiza de manera manual mediante la interfaz de usuario, una corrida a la vez. Con esta feature, el sistema se vuelve proactivo (modo reactivo) y persistente (modo recurrente): notifica al operador únicamente para la aprobación del despliegue final, nunca autodespliega solo.

---

## Comportamientos Observables

- [ ] **Daemon de Escucha de Eventos:** Un proceso persistente en segundo plano monitorea las métricas del mercado (volatilidad, spreads) y el estado del portafolio (drawdown diario, desviación de pesos).
- [ ] **Definición de Reglas de Disparo:** El usuario define reglas lógicas de condición y acción (ej: "SI la volatilidad excede cierto umbral, ENTONCES ejecutar el pipeline de generación de reversión a la media").
- [ ] **Máquina de Estados de Ejecución:** El sistema registra y hace seguimiento a los estados de los pipelines disparados (pendiente, en ejecución, completado, fallido).
- [ ] **Flujo de Aprobación Manual:** Al finalizar un pipeline disparado por eventos, si los candidatos generados superan los criterios de calidad mínimos, el sistema notifica al usuario con un resumen y solicita aprobación explícita para la promoción, en lugar de realizar un autodespliegue automático.
- [ ] **Registro de Auditoría e Historial:** Cada trigger evaluado y ejecutado se escribe de manera persistente con sus resultados y latencia de respuesta en el registro inmutable.
- [ ] **Pipeline Recurrente (TTR-003, ADR-0150):** El usuario activa una política de recurrencia sobre un Pipeline (continuo o por intervalo). El daemon lanza una **Expedition nueva** en cada iteración — nunca una Expedition de larga duración que se reutiliza. El usuario puede pausar el bucle (`RUNNING → PAUSED`, ninguna Expedition nueva se lanza, el histórico de Expeditions pasadas permanece intacto), modificar la topología del Pipeline (crea una nueva `pipeline_version_hash` vía `pipeline-registry`, ADR-0005/ADR-0150 — el bucle no corre mientras está `PAUSED`), y reanudar (`PAUSED → RUNNING`, la siguiente iteración usa la versión vigente, sea la misma o la nueva).

---

## Restricciones

- **NUNCA** permitir el despliegue automático directo en cuentas vivas sin la confirmación manual explícita del operador a través del flujo de aprobación.
- **NUNCA** bloquear el hilo principal de procesamiento de órdenes o recepción de cotizaciones del bróker durante la ejecución de los pipelines disparados.
- **FIJO:** Los disparadores múltiples que coincidan en la misma ventana temporal se evalúan de forma secuencial o con concurrencia controlada para evitar la saturación de los recursos de hardware de la máquina local.
- **NUNCA una iteración de un Pipeline recurrente reutiliza o extiende una Expedition anterior.** Cada iteración es una Expedition nueva e inmutable (config snapshot propio) — preserva la reproducibilidad (ADR-0002) y mantiene `trials_count`/σ² correctos para el DSR (ADR-0150/ADR-0151); una "Expedition eterna" con config mutable rompería ambas garantías.
- **NUNCA el run-state de recurrencia vive en `pipeline-registry`.** `pipeline-registry` es solo definición/versión (nunca ejecuta nada, por su propio contrato); el estado `RUNNING`/`PAUSED`/`STOPPED` del bucle vive en esta feature (el daemon), referenciando el `pipeline_version_hash` vigente.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| EVALUATION_INTERVAL_SECS | 60 | 5 - 3600 | Intervalo de tiempo para evaluar las condiciones de disparo | CONFIG |
| MAX_PARALLEL_PIPELINES | 2 | 1 - 8 | Límite máximo de pipelines automatizados que se pueden ejecutar en paralelo | CONFIG |
| APPROVAL_TIMEOUT_HOURS | 24 | 1 - 168 | Tiempo que permanece activa la notificación de aprobación antes de descartar el resultado | CONFIG |
| RECURRENCE_MODE | CONTINUOUS | CONTINUOUS, INTERVAL | `CONTINUOUS`: lanza la siguiente Expedition en cuanto la anterior termina. `INTERVAL`: lanza según `RECURRENCE_INTERVAL_SECS` | CONFIG |
| RECURRENCE_INTERVAL_SECS | 86400 | 60 - 2592000 | Intervalo entre iteraciones en modo `INTERVAL` (default: 1 día) | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Motor de evaluación de reglas lógicas. Determina si el estado actual del mercado o portafolio coincide con el criterio de disparo.
- **Shell (Infraestructura):** Daemon persistente conectado al bus de eventos y base de datos local SQLite. Lanza los jobs asíncronos de los pipelines.
- **Frontera Pública:** Interfaz para el registro de disparadores, consulta de estado de pipelines y envío de señales de aprobación/rechazo del operador.

---

## Ciclo de Vida de la Feature — Event-Driven Pipeline Triggers

### Entrada
- Flujo de eventos de mercado (precios, volatilidad, régimen).
- Estado del portafolio en vivo (ledger de balance, posiciones y drawdown).
- JSON de definición de triggers configurado por el usuario.

### Proceso
- El Daemon evalúa periódicamente las condiciones de los triggers registrados contra el estado de las variables.
- Si una condición se cumple, se genera un comando para disparar el pipeline asociado en segundo plano.
- Al terminar la validación del pipeline, se evalúan los candidatos resultantes contra el filtro de calidad.

### Salida
- Notificación al operador en la UI con los resultados y botón de aprobación para promover los candidatos a la incubadora.
- Registro en base de datos del historial de ejecuciones y triggers disparados.

---

## Tareas (TTRs)

### **TTR-001: Daemon de Evaluación de Reglas de Disparo**
*   **¿Cuál es el problema?** Se necesita un componente que monitoree constantemente el mercado y el portafolio para disparar pipelines sin consumir excesiva CPU ni bloquear la operativa.
*   **¿Qué tiene que pasar?** Implementar un daemon en segundo plano que escuche eventos específicos en el bus local y evalúe las expresiones lógicas definidas por el usuario a intervalos regulares.
*   **¿Cómo sé que está hecho?**
    - [ ] El daemon detecta un evento simulado (ej: volatilidad > 30) y cambia el estado del disparador a "Ejecutando pipeline" en < 100ms.
*   **¿Qué no puede pasar?**
    - El daemon no debe realizar llamadas a red bloqueantes ni consumir más del 2% de CPU en su fase de espera pasiva.

### **TTR-002: Orquestador de Aprobación y Expiración**
*   **¿Cuál es el problema?** Si un pipeline termina y genera estrategias viables, estas no deben quedarse flotando indefinidamente ni desplegarse solas.
*   **¿Qué tiene que pasar?** Crear el flujo de trabajo de aprobación que retenga las estrategias generadas en un almacén temporal y emita una alerta a la interfaz de usuario. Si pasa el límite de tiempo configurable sin respuesta del operador, el lote se descarta.
*   **¿Cómo sé que está hecho?**
    - [ ] Al cumplirse el plazo sin aprobación, las estrategias temporales son eliminadas y el estado pasa a "Expirado" en el historial de base de datos.
*   **¿Qué no puede pasar?**
    - No se deben promover estrategias al portafolio activo si no hay firma de aprobación del usuario registrada.

### **TTR-003: Daemon de Recurrencia — Ciclo de Vida de Pipeline en Bucle (ADR-0150, resolución `DEBT-024`)**
*   **¿Cuál es el problema?** El usuario quiere que un Pipeline (ej. "generación nocturna de candidatas") corra indefinidamente — una iteración tras otra — sin relanzarlo manualmente cada vez, y poder pararlo, ajustar su topología, y reanudarlo sin perder el histórico de corridas ya hechas.
*   **¿Qué tiene que pasar?** El usuario activa una política de recurrencia sobre un Pipeline vigente (`pipeline_version_hash`), eligiendo `RECURRENCE_MODE` (continuo o por intervalo). El daemon mantiene un **run-state** por Pipeline: `RUNNING` (el bucle está activo, lanza Expeditions), `PAUSED` (el bucle no lanza nuevas Expeditions; el usuario puede modificar la topología del Pipeline mientras está en este estado, lo que crea una nueva versión en `pipeline-registry`), `STOPPED` (el usuario terminó el bucle deliberadamente, no se reanuda sin volver a activarlo). Cada iteración del bucle en estado `RUNNING` lanza una **Expedition nueva** (vía `expedition-ledger`, ADR-0150) referenciando el `pipeline_version_hash` vigente en ese momento — nunca reutiliza ni extiende la Expedition anterior.
*   **¿Cómo sé que está hecho?**
    - [ ] El usuario activa recurrencia `CONTINUOUS` sobre un Pipeline → cada Expedition que termina dispara automáticamente el lanzamiento de la siguiente, sin acción manual.
    - [ ] El usuario pausa el bucle (`RUNNING → PAUSED`) → ninguna Expedition nueva se lanza; las Expeditions ya completadas siguen consultables normalmente.
    - [ ] Estando `PAUSED`, el usuario modifica la topología del Pipeline (agrega/quita un nodo) → se crea una nueva versión en `pipeline-registry` (diffeable contra la anterior, ADR-0005/ADR-0150); el run-state sigue `PAUSED`.
    - [ ] El usuario reanuda (`PAUSED → RUNNING`) → la siguiente iteración lanza una Expedition sobre la versión vigente (la nueva, si se modificó).
    - [ ] Se consulta "cuántas Expeditions ha producido este Pipeline en total, y en qué versión estaba cada una" — es una query directa sobre `expedition-ledger`, sin mecanismo aparte.
*   **¿Qué no puede pasar?**
    - Una Expedition nunca queda huérfana de su iteración: cada una queda atada por linaje a la versión de Pipeline vigente en el momento exacto de su lanzamiento.
    - El run-state de recurrencia nunca se confunde con el estado de UNA Expedition (`PENDING`/`RUNNING`/`DONE`/`FAILED`/`CANCELLED`, ADR-0150) — son dos máquinas de estado distintas en capas distintas (la del bucle, la de cada corrida individual).

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Las reglas, la base de datos de disparadores y la ejecución de los pipelines ocurren exclusivamente en la máquina del usuario.
- **Fidelidad (ADR-0017):** No aplica de manera directa, pero los pipelines disparados invocan los motores de backtesting con la fidelidad correspondiente.

### Perfil Ops / Auditoría (ADR-0020)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de la ejecución del trigger |
| | `created_at` | Timestamp de disparo en nanosegundos |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de la configuración de reglas evaluada |
| | `audit_chain_hash` | Hash encadenado del historial de disparos |
| | `event_sequence_id` | Secuencia de recuperación del disparo |
| **II. Soberanía** | `owner_id` | Identificador del operador local |
| **IV. Hardware** | `node_id` | Identificador de hardware físico del host |
| | `process_id` | PID del daemon de monitoreo |
| **Rastro de Evidencia:** | Emite registros de inicio de pipeline, métrica de disparo causante y veredicto final para auditoría en el módulo `feedback`. |

**Tabla de política de recurrencia (TTR-003, propiedad de esta feature):** `id`, `pipeline_version_hash` (referencia vigente, actualizable al reanudar tras una modificación), `run_state` (`RUNNING`\|`PAUSED`\|`STOPPED`), `recurrence_mode` (`CONTINUOUS`\|`INTERVAL`), `recurrence_interval_secs` (solo si `INTERVAL`), `last_expedition_id` (referencia informativa a la última Expedition lanzada), `owner_id`. Mutable (`row_version`) — el run-state cambia en sitio; el histórico de qué Expeditions produjo vive en `expedition-ledger`, nunca duplicado aquí.

---

## Dependencias y Bloqueantes

**Depende de:**
- [`pipeline-registry`](../features/pipeline-registry.md) — resuelve la definición/versión vigente del Pipeline que se dispara.
- [`expedition-ledger`](../features/expedition-ledger.md) — cada disparo (reactivo o recurrente) lanza una Expedition nueva; esta feature nunca ejecuta el Pipeline directamente, solo lo dispara.
- [`async-job-executor`](../features/async-job-executor.md) — el lanzamiento de cada Expedition es un job asíncrono.

**Consumido por:**
- El usuario, vía Canvas — activa/pausa/detiene la recurrencia sobre un nodo Pipeline.
