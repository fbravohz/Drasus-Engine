# Event-Driven Pipeline Triggers

**Carpeta:** `./features/event-driven-pipeline-triggers/`
**Estado:** En Diseño
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0011 (Operaciones Asincrónicas), ADR-0012 (Arquitectura Multi-Pipeline Paralela)

---

## ¿Qué es esta feature?

El sistema de disparadores de pipelines basado en eventos permite automatizar la ejecución de flujos de descubrimiento y validación de estrategias (pipelines de QUANTOPS) en respuesta a condiciones específicas del mercado o del portafolio en tiempo real. 

**Problema:** Tradicionalmente, la ejecución de la exploración, optimización o rebalanceo se realiza de manera manual mediante la interfaz de usuario. Con esta feature, el sistema se vuelve proactivo: ante eventos como el incremento de la volatilidad (ej: VIX cruzando un umbral) o cambios en el régimen de mercado, se disparan de forma autónoma secuencias de ingestión, búsqueda genética y validación, notificando al operador únicamente para la aprobación del despliegue final.

---

## Comportamientos Observables

- [ ] **Daemon de Escucha de Eventos:** Un proceso persistente en segundo plano monitorea las métricas del mercado (volatilidad, spreads) y el estado del portafolio (drawdown diario, desviación de pesos).
- [ ] **Definición de Reglas de Disparo:** El usuario define reglas lógicas de condición y acción (ej: "SI la volatilidad excede cierto umbral, ENTONCES ejecutar el pipeline de generación de reversión a la media").
- [ ] **Máquina de Estados de Ejecución:** El sistema registra y hace seguimiento a los estados de los pipelines disparados (pendiente, en ejecución, completado, fallido).
- [ ] **Flujo de Aprobación Manual:** Al finalizar un pipeline disparado por eventos, si los candidatos generados superan los criterios de calidad mínimos, el sistema notifica al usuario con un resumen y solicita aprobación explícita para la promoción, en lugar de realizar un autodespliegue automático.
- [ ] **Registro de Auditoría e Historial:** Cada trigger evaluado y ejecutado se escribe de manera persistente con sus resultados y latencia de respuesta en el registro inmutable.

---

## Restricciones

- **NUNCA** permitir el despliegue automático directo en cuentas vivas sin la confirmación manual explícita del operador a través del flujo de aprobación.
- **NUNCA** bloquear el hilo principal de procesamiento de órdenes o recepción de cotizaciones del bróker durante la ejecución de los pipelines disparados.
- **FIJO:** Los disparadores múltiples que coincidan en la misma ventana temporal se evalúan de forma secuencial o con concurrencia controlada para evitar la saturación de los recursos de hardware de la máquina local.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| EVALUATION_INTERVAL_SECS | 60 | 5 - 3600 | Intervalo de tiempo para evaluar las condiciones de disparo | CONFIG |
| MAX_PARALLEL_PIPELINES | 2 | 1 - 8 | Límite máximo de pipelines automatizados que se pueden ejecutar en paralelo | CONFIG |
| APPROVAL_TIMEOUT_HOURS | 24 | 1 - 168 | Tiempo que permanece activa la notificación de aprobación antes de descartar el resultado | CONFIG |

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

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Las reglas, la base de datos de disparadores y la ejecución de los pipelines ocurren exclusivamente en la máquina del usuario.
- **Fidelidad (ADR-0017):** No aplica de manera directa, pero los pipelines disparados invocan los motores de backtesting con la fidelidad correspondiente.

### Perfil Ops / Auditoría (ADR-0020 V2)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de la ejecución del trigger |
| | `created_at` | Timestamp de disparo en nanosegundos |
| | `audit_hash` | Hash de la configuración de reglas evaluada |
| **II. Soberanía** | `owner_id` | Identificador del operador local |
| **IV. Hardware** | `node_id` | Identificador de hardware físico del host |
| | `process_id` | PID del daemon de monitoreo |
| **Rastro de Evidencia:** | Emite registros de inicio de pipeline, métrica de disparo causante y veredicto final para auditoría en el módulo `feedback`. |
