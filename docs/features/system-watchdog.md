# Shadow Watchdog & Kill Switch (P1 Crítica)

**Carpeta:** `./features/system-watchdog/`
**Estado:** Especificación / Crítica
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

El protector de última instancia del sistema. Su misión es la **Prevención de Ruina**: monitorea continuamente la salud técnica (latencia, conexión) y financiera (equity) para ejecutar un cierre de emergencia (Kill Switch) ante anomalías críticas.

---

## Comportamientos Observables

- [ ] **Heartbeat Monitoring:** Detecta desconexión o congelamiento del daemon de ejecución en < 5s.
- [ ] **Pardo Profile Comparison:** Compara spreads y deslizamientos reales vs esperados históricos; alerta si spread > 3σ.
- [ ] **Kill Switch Proactivo:** Activa el cierre global si el DrawDown vivo excede `HistMaxDD × safety_factor` (default: 1.1).
- [ ] Cierra todas las posiciones y limpia el libro de órdenes en milisegundos.

---

## Ciclo de Vida de la Feature — System Watchdog

### Entrada
- Estado del balance y posiciones en tiempo real.
- Status de conectividad del API del broker.
- Umbral de Kill Switch (ej: 1.5x DD máximo histórico).

### Proceso
- Evalúa la salud del sistema cada N milisegundos (Heartbeat).
- Si se detecta brecha de riesgo o desconexión prolongada:
  1. Bloquea la entrada de nuevas órdenes.
  2. Dispara el protocolo de cierre de emergencia a mercado.
  3. Envía notificación de pánico.

### Salida
- **Estado del Sistema:** LOCKED / OPERATIONAL.
- **Reporte de Incidente:** Log de la razón exacta del disparo del Kill Switch.

### Contextos de Uso
**Contexto Único: Seguridad Operativa (Módulo Execute)**
- Es el guardián de la "Llave de Paso" del flujo de órdenes.

---

---

## Tareas (TTRs)

### **TTR-001: Monitor de Salud Técnica y Financiera (Shadow Heartbeat)**
* **¿Cuál es el problema?** Si el daemon de ejecución en vivo se bloquea o tiene un crash silencioso, las órdenes abiertas pueden quedar desprotegidas ante movimientos violentos del mercado.
* **¿Qué tiene que pasar?** Implementar un monitor supervisor independiente (Puerto 8002) que verifique la latencia de respuesta del corazón del sistema y la coherencia del capital en tiempo real.
* **¿Cómo sé que está hecho?**
    - [ ] Si el latido cesa por > 5s, el sistema activa automáticamente el `EMERGENCY_LOCK`.
    - [ ] El Pardo Check detecta spreads anómalos (>3σ) y emite un `WARNING`.
* **¿Qué no puede pasar?** NUNCA permitir que el Watchdog comparta el mismo hilo de ejecución que el motor de órdenes.

### **TTR-002: Ejecución de Kill Switch y Barrido de Pánico**
* **¿Cuál es el problema?** Ante una pérdida que excede los límites históricos (Drawdown catastrófico), la reacción humana es demasiado lenta para evitar la ruina.
* **¿Qué tiene que pasar?** Una función atómica que, al recibir la señal de pánico, envíe órdenes de cierre a mercado para todas las posiciones abiertas y limpie el libro de órdenes pendiente.
* **¿Cómo sé que está hecho?**
    - [ ] El cierre total de posiciones ocurre en milisegundos tras cruzar el umbral de `MAX_DD * 1.1`.
    - [ ] El sistema se bloquea físicamente y requiere reinicio manual para operar de nuevo.
* **¿Qué no puede pasar?** PROHIBIDO intentar reabrir posiciones tras un disparo de Kill Switch sin autorización humana expliciata.

### **TTR-003: Shadow Mode Validation (Validación Paralela Inerte)**
* **¿Cuál es el problema?** Probar una estrategia en producción viva directamente con capital puede inducir pérdidas catastróficas inesperadas debido a diferencias microscópicas de alimentación de datos (data feed drift).
* **¿Qué tiene que pasar?** Habilitar la ejecución de estrategias en paralelo con volumen estrictamente igual a cero (Shadow Mode). El Watchdog asocia los disparos de órdenes simuladas y compara el rendimiento live-ficticio en tiempo real vs el modelo de backtest teórico.
* **¿Cómo sé que está hecho?**
    - [ ] El sistema procesa los ticks de mercado en paralelo sin enviar órdenes reales al broker.
    - [ ] Mide y persiste métricas de drift de PnL ficticio y latencias teóricas en DuckDB.
* **¿Qué no puede pasar?** PROHIBIDO enviar órdenes con volumen mayor a cero cuando una estrategia está marcada como `Shadow_Active`.

### **TTR-004: Emergency PWA (Mando Móvil Remoto)**
* **¿Cuál es el problema?** El operador puede no estar frente a la laptop/consola de trading en el momento en que se produce un fallo de infraestructura o comportamiento anómalo.
* **¿Qué tiene que pasar?** Proveer un portal web progresivo (Emergency PWA) ultra-rápido, ligero y seguro, compilado directamente a la UI de Flutter y enlazado de forma segura con el gRPC de Rust de Drasus Engine. Permite el monitoreo remoto en tiempo real de la equidad de la cuenta y el disparo inmediato del manual `FlattenAll()` (Kill Switch remoto).
* **¿Cómo sé que está hecho?**
    - [ ] Carga instantánea de la interfaz PWA y comunicación segura mediante gRPC.
    - [ ] Detonación instantánea y atómica de la limpieza de órdenes al presionar el interruptor de pánico.
* **¿Qué no puede pasar?** PROHIBIDO permitir el acceso a la PWA sin autenticación cifrada por token de un solo uso local (SLA de seguridad).

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda alerta de riesgo y ejecución de Kill Switch registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del incidente |
| | `created_at` | Timestamp del disparo (Heartbeat) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del sistema T-0 |
| | `audit_chain_hash` | Hash de la secuencia de incidentes |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Autor responsable del capital |
| | `access_token_id` | Token de autorización de pánico |
| **IV. Hardware** | `node_id` | ID del hardware físico supervisor |
| | `process_id` | PID del watchdog independiente |
| **V. Forense & Ejecución** | `execution_latency_ms` | Reacción al pánico (milisegundos) |
| | `heartbeat_latency_ms` | Variante específica: latencia del último latido detectado; medida derivada de `execution_latency_ms` aplicada al heartbeat |

- **Decisión Arquitectónica Asociada:**
    - ADR-0010: Reglas Dinámicas (Watchdog como ejecutor de Hard Limits).
    - ADR-0015: Arquitectura de Causalidad (Análisis de Causa Raízs de liquidación).
    - ADR-0020: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`broker-connector`](../features/broker-connector.md) — para la comunicación con el exchange.
- [`notification`](../features/notification.md) — para alertas de pánico.

**Consumido por:**
- [`execute`](../modules/execute.md) — para la protección de la sesión de trading en vivo.
