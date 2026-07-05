# Audit Event Store — Inmutabilidad y Reconstrucción por Eventos

**Carpeta:** `./features/audit-event-store/`
**Estado:** Especificación / Crítica (Fase 2)
**Última actualización:** 2026-04-13

---

## ¿Qué es?

El Audit Event Store es el historial inmutable de vida de todas las decisiones, señales y órdenes del sistema. Implementa el patrón **Event Sourcing (SQX Mod 5.3.5)**: el estado actual del inventario no es una tabla estática, sino el resultado de reproducir todos los eventos previos. Su misión es la **Resiliencia Operativa** y la **Auditoría Forense**.

---

## Comportamientos Observables

- [ ] **Persistencia Inmutable:** Cada decisión del Autopilot se persiste en SQLite WAL como un evento bit-a-bit único.
- [ ] **Inventory Reconstruction:** Tras un crash, el sistema lee el Event Store para recuperar órdenes abiertas y posiciones de broker en < 10s.
- [ ] **User Veto Power (REVERT):** El usuario puede revertir cualquier acción ejecutada automáticamente por el Autopilot desde el Dashboard.
- [ ] **Auto-Auditoría (P1):** Compara spreads reales en el momento del evento contra spreads esperados históricos (Pardo Profile).
- [ ] **Compliance Ready:** Exporta el historial forense en formatos legibles para reguladores (ADR-0020).

---

## Tareas (TTRs)

### **TTR-001: Persistencia de Eventos de Ejecución (Event Store)**
* **¿Cuál es el problema?** El estado actual del sistema (órdenes abiertas, balances) es volátil y puede perderse tras un crash. Sin un rastro inmutable, es imposible reconstruir el inventario local con precisión institucional.
* **¿Qué tiene que pasar?** Cada cambio de estado (`OrderCreated`, `OrderFilled`, `AlertTriggered`) debe persistirse inmediatamente como una fila inmutable en SQLite con un `audit_chain_hash` vinculado al evento anterior.
* **¿Cómo sé que está hecho?**
    - [ ] La tabla de eventos en SQLite crece linealmente y es de solo-lectura tras la inserción.
    - [ ] El hash de auditoría (`audit_hash`) coincide con el rastro de la `audit_chain_hash`.
* **¿Qué no puede pasar?** PROHIBIDO editar o borrar eventos del Store (Inmutabilidad física mandatada por ADR-0027).

### **TTR-002: Protocolo de Recuperación Tras Crash (Crash Recovery)**
* **¿Cuál es el problema?** Reiniciar el sistema tras una caída inesperada sin saber exactamente qué órdenes están vivas en el broker puede llevar a duplicación de órdenes o pérdidas descontroladas.
* **¿Qué tiene que pasar?** Una lógica de "Replay" que, al arrancar, lea el Event Store desde el último punto de verificación y reconstruya el inventario de NautilusTrader en memoria en < 10s.
* **¿Cómo sé que está hecho?**
    - [ ] Tras un kill forzado del proceso, el sistema recupera el 100% de las posiciones abiertas registradas.
* **¿Qué no puede pasar?** NUNCA permitir el envío de nuevas órdenes antes de que el proceso de reconstrucción de inventario haya finalizado.

### **TTR-003: Lógica de Veto y Reversión (Veto Power Engine)**
* **¿Cuál es el problema?** El Autopilot puede ejecutar acciones automáticas (ej. cerrar una posición) que el usuario considera erróneas o innecesarias en un contexto específico.
* **¿Qué tiene que pasar?** Implementar el comando `REVERT_ACTION` que genera un evento de compensación (ej: re-abrir posición) vinculado al evento original por `parent_event_id`, con justificación obligatoria.
* **¿Cómo sé que está hecho?**
    - [ ] El panel de auditoría muestra claramente el par Evento/Reversión con el texto de justificación del usuario.
* **¿Qué no puede pasar?** PROHIBIDO realizar una reversión que viole las restricciones de margen o capital actuales de la cuenta.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 · Perfil D Ops/Auditoría)

Es el registro forense en sí. Perfil D (Auditoría = I + II + IV + V Gobernanza/Cumplimiento):

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del evento |
| | `created_at` | Timestamp atómico (nanosegundos) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del cuerpo del evento (JSON) |
| | `audit_chain_hash` | Hash de integridad de la cadena (WAL) |
| | `event_sequence_id` | Secuencia de recuperación / orden del WAL |
| **II. Soberanía** | `owner_id` | Dueño inmutable del rastro |
| | `institutional_tag` | Tag de cumplimiento institucional |
| **IV. Hardware** | `node_id` | ID del hardware físico de escritura |
| | `process_id` | PID del escritor de eventos |
| | `session_id` | Sesión operativa global |
| **V. Forense (Gobernanza)** | `risk_audit_id` | Ticket de riesgo asociado al evento |
| | `signature_hash` | Firma HMAC del evento sellado |

---

## Dependencias
- [`nautilus-integration`](../features/nautilus-integration.md) — para la captura de eventos reales.
- [`databank-manager`](../features/databank-manager.md) — para el almacenamiento de largo plazo si se requiere.
