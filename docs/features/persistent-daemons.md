# Persistent Daemons (LiveNode Isolation)

**Carpeta:** `./features/persistent-daemons/`
**Estado:** Lista para implementar
**Última actualización:** 2026-05-12
**Decisión Arquitectónica Asociada:** ADR-0084 (Daemons Persistentes y Aislamiento de Núcleo)

## ¿Qué es esta feature?

En un entorno de trading institucional, la latencia y la estabilidad son críticas. Mientras que los procesos de Investigación y Desarrollo (R&D) utilizan "Workers" efímeros que se crean para una tarea específica y luego desaparecen, la ejecución en vivo requiere **Procesos Persistentes (Daemons)**. 

Esta feature implementa hilos de ejecución de larga duración en Rust (Tokio tasks) dedicados exclusivamente al **LiveNode** de NautilusTrader. Estos procesos se mantienen vivos durante toda la sesión de trading y cuentan con aislamiento de hardware mediante **Core Pinning** (afinidad de CPU) para garantizar que la ejecución de órdenes reales nunca se vea afectada por el ruido o la carga de otros módulos del sistema.

## Comportamientos Observables

- [ ] Cuando el usuario activa el modo "Live Trading", el sistema inicia un hilo persistente que no se detiene hasta que el usuario lo solicita explícitamente o el sistema detecta un error crítico.
- [ ] Si el sistema está realizando una optimización genética masiva (uso de CPU al 100% en otros núcleos), la latencia de respuesta del LiveNode se mantiene estable en el rango de microsegundos.
- [ ] El usuario puede ver en el Dashboard qué núcleo físico de su procesador está dedicado exclusivamente a la ejecución en vivo.
- [ ] Si el proceso persistente falla por causas externas, el sistema intenta un reinicio controlado y notifica inmediatamente al usuario.

- **NUNCA** se debe ejecutar lógica de R&D (backtesting pesado, optimización) en el mismo núcleo asignado al LiveNode.
- **NUNCA** se debe permitir que el LiveNode sea recolectado o pausado por el planificador de tareas de forma arbitraria (uso de afinidad obligatoria).
- **Límite Técnico:** El número de daemons persistentes está limitado por el número de núcleos físicos disponibles en el hardware del usuario (SLA de reserva).
- **Ejecución Remota Delegada (HybridComputeCooperative):** El daemon persistente de LiveNode puede ser desplegado de forma delegada en un VPS remoto. El backend Rust local (FFI) actúa como orquestador de control enviando comandos y recibiendo telemetría por gRPC/WebSockets, permitiendo la desconexión del cliente local sin interrumpir la operación del daemon remoto.


## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| RESERVED_CORE_ID | 1 | 0 - N | ID del núcleo de CPU reservado para el LiveNode | CONFIG |
| DAEMON_HEARTBEAT_MS | 1000 | 100 - 5000 | Intervalo de verificación de salud del proceso | CONFIG |
| AUTO_RESTART_LIVE | true | true/false | Reiniciar automáticamente el daemon si falla | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Gestión de la máquina de estados del LiveNode (Starting, Running, Error, Stopped).
- **Shell (Infraestructura):** Orquestador de hilos de Rust (Tokio), gestión de afinidad de CPU mediante `core_affinity`.
- **Frontera Pública:** Comandos para `start_live_node`, `stop_live_node` y consulta de `live_node_status`.

## Ciclo de Vida de la Feature

### Entrada
- Configuración de la estrategia promovida.
- Credenciales de broker cifradas.
- Especificación del núcleo de CPU a reservar.

### Proceso
- Reserva el núcleo físico especificado.
- Inicializa el entorno de NautilusTrader en el hilo persistente.
- Mantiene un bucle de eventos determinista para la recepción de ticks y envío de órdenes.
- Emite latidos de vida (heartbeats) hacia el orquestador principal.

### Salida
- Ejecución de órdenes en tiempo real.
- Registro de auditoría (audit trail) de cada micro-decisión.
- Métricas de salud del daemon (CPU usage, latency spikes).

## Tareas (TTRs)

### **TTR-001: Implementación de Core Pinning**
*   **¿Cuál es el problema?** El sistema operativo mueve los procesos entre núcleos de CPU según le conviene, lo que causa variaciones en la latencia (jitter). Para trading de alta frecuencia, necesitamos que el LiveNode se quede en un solo sitio.
*   **¿Qué tiene que pasar?** El sistema debe identificar los núcleos disponibles y anclar el hilo del LiveNode al núcleo configurado, impidiendo que otras tareas usen ese núcleo si es posible.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo verificar en el monitor del sistema que el hilo del LiveNode siempre corre en el mismo CPU ID.
    - [ ] La latencia de respuesta es constante bajo carga de otros procesos.

### **TTR-002: Watchdog de Daemons Persistentes**
*   **¿Cuál es el problema?** Si un daemon de ejecución se queda colgado o muere sin avisar, el usuario puede perder dinero por no gestionar sus posiciones.
*   **¿Qué tiene que pasar?** Un monitor externo (Shadow Watchdog) debe verificar cada segundo que el daemon sigue respondiendo. Si no hay respuesta, dispara el protocolo de emergencia.
*   **¿Cómo sé que está hecho?**
    - [ ] Si fuerzo la detención del proceso, el sistema lanza una alerta visual y sonora en < 2 segundos.
    - [ ] Hay un log que registra cada latido de vida exitoso.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2 · Perfil C Ops/Hot-Path)

Daemon de ejecución real (core pinning, latencia crítica) → Perfil C (I + II + IV + V latencia):

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del daemon/sesión |
| | `created_at` | Timestamp de arranque (nanosegundos) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del estado del daemon |
| | `audit_chain_hash` | Hash encadenado del historial de heartbeats |
| | `event_sequence_id` | Secuencia de recuperación del daemon |
| **II. Soberanía** | `owner_id` | Propietario de la cuenta/estrategia en ejecución |
| | `institutional_tag` | Tag de entorno (LIVE / PAPER) |
| **IV. Hardware** | `node_id` | ID del hardware físico (núcleo fijado / core pinning) |
| | `process_id` | PID del daemon persistente |
| | `session_id` | Agrupación de runtime |
| **V. Forense & Ejecución (latencia)** | `execution_latency_ms` | Latencia de ciclo del daemon en hot-path |
| | `source_signal_id` | Señal/evento de mercado procesado |

- **Rastro de Evidencia:** Emite heartbeats y latencias de ciclo al módulo `feedback`.

---

## Dependencias

**Depende de:**
- Módulo `manage` (para recibir la estrategia promovida).
- Motor `NautilusTrader` (núcleo de ejecución).

**Bloquea:**
- Ejecución real en mercados externos.
