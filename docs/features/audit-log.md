# Audit Log — Registro de Auditoría Inmutable

**Carpeta:** `./features/audit-log/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

---

## ¿Qué es?

El Audit Log es el registro histórico inmutable de todos los eventos significativos del sistema. Cada cambio de estado, cada decisión de trading, cada anomalía detectada queda registrada para siempre.

**Problema:** Si los módulos escriben logs directamente, cada uno usa su propio formato. Los logs se pierden al reiniciar. No hay forma unificada de investigar qué pasó.

**Solución:** El Core nunca escribe logs. En su lugar, dispara eventos al puerto de auditoría injected. El Shell es quien persiste esos eventos de forma inmutable (append-only: solo agregar, nunca borrar ni modificar).

**Resultado observable:** Auditoría completa del sistema basada en el **Principio de Inundación de Fundaciones (ADR-0020 V2)**. Cada evento incluye metadatos para cumplimiento institucional (Institutional Compliance) y reconciliación de Nautilus.

---

## Comportamientos Observables

- [ ] Se ejecuta una orden en Execute
  → Dispara evento "ORDER_STATE_CHANGE" con detalles
  → El evento se registra en Audit Log (timestamp, módulo origen, detalles)
  → Incluye: qué cambió de estado, de qué a qué, por qué motivo

- [ ] Se detecta una anomalía en Feedback
  → Dispara evento "ANOMALY_DETECTED" al puerto de auditoría
  → Se registra en Audit Log con timestamp, tipo de anomalía, severidad
  → El usuario puede consultar todas las anomalías de los últimos 30 días

- [ ] Un usuario vetea una decisión de ejecución en Execute
  → Dispara evento "USER_VETO" con identificador de usuario, orden vetada, motivo
  → Se registra en Audit Log
  → Es imposible borrar o modificar este registro — está documentado para siempre

- [ ] El usuario consulta "¿qué pasó con la estrategia XYZ el 2026-04-07?"
  → Sistema busca en Audit Log todos los eventos relacionados a estrategia XYZ en esa fecha
  → Devuelve lista cronológica: qué órdenes se enviaron, qué cambios de estado ocurrieron, qué anomalías se detectaron

---

## Restricciones

- **NUNCA un evento se borra del Audit Log.** Append-only absoluto.
- **NUNCA un evento se modifica después de ser grabado.** Inmutable absoluto.
- **NUNCA se graba información sensible sin encripción.** (Ej: contraseñas, tokens; aunque el Core no debería tener eso)
- **NUNCA un evento Audit falta campos obligatorios:** timestamp, tipo de acción, tipo de entidad, identificador de entidad, detalles.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| `RETENTION_DAYS` | 365 | 30-3650 días | Cuántos días guardar Audit Log antes del cleanup | CONFIG |
| `BATCH_SIZE` | 100 | 10-1000 | Cuántos eventos agrupar antes de escribir en disco | CONFIG |
| `COMPRESSION` | false | true / false | Si true, comprime eventos viejos para ahorrar espacio | CONFIG |
| `MAX_FILE_SIZE_MB` | 1024 | 100-10240 | Límite de tamaño de archivo para rotación | CONFIG |

---

## Ciclo de Vida de la Feature

### Entrada
- **Quién llama:** Todos los módulos del Core — cuando ocurre algo significativo (cambio de estado, detección de anomalía, decisión de usuario)
- **Qué recibe el puerto:** Un evento estructurado con campos semánticos: tipo de acción, tipo de entidad, identificador de entidad, detalles del evento, timestamp

### Proceso
- **En el Core:** Dispara un evento al puerto de auditoría injected; no espera confirmación (es un fire-and-forget)
- **En la Shell/persistencia:** Recibe el evento, valida que tenga todos los campos, lo escribe en el log (append-only), no permite modificación

### Salida
- **Qué produce:** El evento persiste en el Audit Log de forma inmutable
- **Cuándo:** Inmediatamente (o en batch, según configuración de persistencia)

### Contextos de Uso
- **Execute:** Auditoria de cada orden enviada, cada cambio de estado, cada veto de usuario
- **Validate:** Auditoría de qué tests corrieron y con qué resultados
- **Manage:** Auditoría de cambios de portafolio, rebalanceos, cambios de reglas
- **Feedback:** Auditoría de anomalías detectadas, intentos de resolución
- **Withdraw:** Auditoría de retiros de estrategias, cambios de estado
- **Cualquier módulo:** Cualquier decisión de riesgo o cambio de configuración

---

---

## Tareas (TTRs)

### **TTR-001: Registro de Evento Inmutable (Append-Only)**
*   **Descripción:** Persiste eventos críticos con rastro de evidencia inmutable.
*   **Reglas de Negocio:**
    * El `audit_hash` debe calcularse usando el hash de la fila anterior + contenido actual (encadenamiento).
    * Toda entrada DEBE incluir `process_id` y `institutional_tag` (ADR-0020 V2).
*   **Entrada:** `action_type`, `entity_type`, `entity_id`, `details_json`, `process_id`.
*   **Salida:** `log_id` (UUID), `audit_hash`.
*   **Precondición:** Evento validado contra esquema de acción permitido.
*   **Postcondición:** Registro bloqueado físicamente para edición o borrado.

### **TTR-002: Reconciliación de Rastro Nautilus (ADR-0013)**
*   **Descripción:** Sincroniza eventos de auditoría con los ticks de Nautilus para Time-Travel Debugging.
*   **Reglas de Negocio:**
    * Los eventos de ejecución deben incluir el `ntp_sync_offset` del momento.
    * Timestamps inmutables en nanosegundos.
*   **Entrada:** `event_id`, `nautilus_tick_id`.
*   **Salida:** `sync_status`.
*   **Precondición:** Reloj sincronizado (ADR-0013).
*   **Postcondición:** Evento indexado para reconstrucción de estado post-crash.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Cada evento en `audit_events` registra el set absoluto de **25 campos mandatorios** (ver ADR-0020 V2 V2).
    - Causalidad Forense: `audit_chain_hash` (encadenamiento físico), `event_sequence_id` (orden de replay), `logic_hash` (código exacto).
    - Soberanía: `owner_id`, `access_token_id`, `manifest_id`.
    - Contexto: `indicator_state_hash`, `data_snapshot_id`.

- **Decisión Arquitectónica Asociada:**
    - ADR-0015: Arquitectura de Causalidad (Audit Log como fuente de verdad para Feedback).
    - ADR-0016: Local-First (Soberanía de datos de auditoría).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para timestamps deterministas de alta precisión.

**Consumido por:**
- **Todos los Módulos del Pipeline:** Para registro de transiciones de estado y decisiones críticas.
- [`feedback`](../modules/feedback.md) — para análisis de causalidad y autopsias.
