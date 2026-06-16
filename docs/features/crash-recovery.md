# State Recovery Protocol (Crash Recovery)

**Carpeta:** `./features/crash-recovery/`
**Estado:** Especificación / Crítica
**Última actualización:** 2026-05-29
**Decisión Arquitectónica Asociada:** ADR-0027 (Event Sourcing & Inventory Reconstruction)

---

## ¿Qué es?

Módulo de contingencia ante fallos físicos de infraestructura (cortes de luz, desconexión de red, reinicios inesperados del sistema operativo). Su objetivo es garantizar la resiliencia operativa y la prevención de ruina al restaurar con fidelidad absoluta el estado transaccional e indicadores en caliente del LiveNode en un lapso `<= 10 segundos` tras el reinicio.

---

## Comportamientos Observables

- [ ] **Persistencia Transaccional:** Guarda inmutablemente cada fill y cambio de estado de orden en el SQLite Event Store en el hot path.
- [ ] **Modo Recovery en Arranque:** Detiene la operativa en vivo al arrancar y bloquea el estado hasta completar la conciliación.
- [ ] **Sincronización Bidireccional:** Consulta las posiciones y órdenes reales del broker mediante API REST en el arranque para compararlas con SQLite local.
- [ ] **Re-alineación de Parámetros:** Recalcula indicadores cinéticos (como Trailing Stops activos), sincroniza límites en el broker y reabre WebSockets una vez validada la paridad de inventario.
- [ ] **Rendimiento de Restauración:** Completa la secuencia de conciliación total y vuelve al estado `ONLINE` en `<= 10 segundos`.

---

## Ciclo de Vida de la Feature — Crash Recovery

### Entrada
- Estado transaccional inmutable persistido en SQLite (`audit-event-store`).
- Datos de posiciones y órdenes pendientes descargados de la API REST del bróker.
- Configuraciones y estados de estrategias activas.

### Proceso
1. **Trigger de Arranque:** El LiveNode despierta y detecta que la sesión no se cerró limpiamente.
2. **Activación de Modo Recovery:** Bloquea el envío de nuevas órdenes.
3. **Descarga de Broker State:** Consulta la API REST del broker.
4. **Validación de Paridad:** Compara posiciones reales del bróker vs ledger de SQLite.
5. **Rehidratación de Indicadores:** Re-construye el estado de indicadores de salida dinámicos (Trailing Stops, Grid Levels) basados en la serie temporal actual.
6. **Ajuste Físico:** Modifica órdenes pendientes en el bróker si detecta divergencias insalvables.

### Salida
- **Estado del Sistema:** `ONLINE` (Conectividad WebSocket restaurada) / `EMERGENCY_LOCK` (Si hay divergencia crítica no conciliable).
- **Audit Trace ID:** Firma criptográfica del reporte de conciliación de arranque.

---

## Tareas (TTRs)

### **TTR-001: Persistencia Transaccional e Inmutabilidad (Hot-Path SQLite)**
* **¿Cuál es el problema?** Si ocurre un corte de luz y el estado de la posición solo reside en memoria RAM, el sistema pierde el rastro operativo al reiniciarse.
* **¿Qué tiene que pasar?** Persistir atómicamente cada cambio de estado confirmado por el exchange en el `audit-event-store` en el hot-path mediante SQLite WAL.
* **¿Cómo sé que está hecho?**
    - [ ] Cada fill confirmado produce una entrada inmutable de solo lectura firmada digitalmente con `audit_hash`.
* **¿Qué no puede pasar?** PROHIBIDO diferir la escritura de base de datos en segundo plano sin usar el modo WAL síncrono para eventos transaccionales críticos.

### **TTR-002: Modo Recovery en el Arranque (Wake-up Call)**
* **¿Cuál es el problema?** Si la computadora se apaga y el sistema arranca enviando órdenes basándose en datos obsoletos de memoria RAM, se pueden duplicar posiciones o violar límites de riesgo.
* **¿Qué tiene que pasar?** Al arrancar el binario, el LiveNode detecta la interrupción de la sesión anterior, entra inmediatamente en el estado temporal `RECOVERY` y deshabilita la emisión de señales de los agentes.
* **¿Cómo sé que está hecho?**
    - [ ] El sistema bloquea el hot-path de despacho de órdenes y el Dashboard de Flutter muestra `STATUS: RECOVERY` en rojo.
* **¿Qué no puede pasar?** PROHIBIDO abrir la conexión WebSocket de datos de mercado en vivo antes de finalizar el proceso de reconciliación.

### **TTR-003: Sincronización Bidireccional Activa (Broker REST Audit)**
* **¿Cuál es el problema?** Las discrepancias entre lo que el sistema "cree" que tiene y las posiciones reales en el bróker pueden causar liquidaciones forzosas catastróficas.
* **¿Qué tiene que pasar?** Realizar peticiones REST síncronas de alta velocidad hacia el bróker para descargar la lista de órdenes activas y posiciones abiertas. Comparar este inventario contra la base de datos SQLite local.
* **¿Cómo sé que está hecho?**
    - [ ] Si las posiciones coinciden 1:1, el validador emite un `RecoveryVerdict::SUCCESS`.
    - [ ] Si hay discrepancia no conciliable automáticamente (ej. órdenes llenadas offline sin registrar), dispara el `EMERGENCY_LOCK` y notifica al usuario.
* **¿Qué no puede pasar?** NUNCA permitir que la conciliación tome más de `10 segundos` en condiciones de conexión normales.

### **TTR-004: Re-alineación Paramétrica e Hidratación de Indicadores**
* **¿Cuál es el problema?** Los trailing stops o grillas de salida pierden su rastro de cálculo si el sistema se apaga intempestivamente, dejando posiciones vivas desprotegidas en el exchange.
* **¿Qué tiene que pasar?** Re-calcular la serie temporal histórica inmediata y actualizar la posición del Stop Loss dinámico en el bróker si el precio avanzó durante la inactividad.
* **¿Cómo sé que está hecho?**
    - [ ] Los objetos de estrategia rehidratan sus estados internos de indicadores de salida en base a los últimos ticks reconstruidos.
* **¿Qué no puede pasar?** PROHIBIDO reanudar la operativa WebSocket sin actualizar las órdenes de stop activas en el bróker con los parámetros correctos.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda restauración de sesión de emergencia registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del evento de recuperación |
| | `created_at` | Timestamp de arranque (nanosegundos) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de la conciliación exitosa |
| | `audit_chain_hash` | Firma del rastro de incidentes encadenados |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Propietario responsable de la cuenta |
| | `compliance_status_id` | Código de veredicto del reconciliador |
| **IV. Hardware** | `node_id` | Identificador del hardware físico local |
| | `process_id` | PID del daemon LiveNode |
| **V. Forense & Ejecución** | `execution_latency_ms` | Latencia de la operación de recuperación (subset Latencia) |
| | `recovery_latency_ms` | Variante específica: tiempo total de reconciliación (Target <= 10s); medida derivada de `execution_latency_ms` aplicada al flujo de recuperación |

---

## Dependencias
**Depende de:**
- [`broker-connector`](../features/broker-connector.md) — para peticiones REST al exchange.
- [`audit-event-store`](../features/audit-event-store.md) — para la persistencia transaccional SQLite.

**Consumido por:**
- [`execute`](../modules/execute.md) — para garantizar el arranque seguro de la sesión live.
