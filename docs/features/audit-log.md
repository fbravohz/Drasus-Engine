# Audit Log — Registro de Auditoría Inmutable

**Carpeta:** `./features/audit-log/`
**Estado:** 🟡 Parcial — TTR-001 (registro inmutable append-only + hash chain) implementado. TTR-002 (reconciliación Nautilus) diferido a EPIC-2+.
**Última actualización:** 2026-06-12

> 🟡 **Parcial** 2026-06-12 · TTR-001 en código (`crates/shared`, migración `0002_audit_log.sql`, triggers append-only + verificación de cadena) · Orden de trabajo [STORY-004](../execution/STORY-004-audit-log.md) · Pendiente: TTR-002 (EPIC-2+).

---

## ¿Qué es?

El Audit Log es el registro histórico inmutable de todos los eventos significativos del sistema. Cada cambio de estado, cada decisión de trading, cada anomalía detectada queda registrada para siempre.

**Problema:** Si los módulos escriben logs directamente, cada uno usa su propio formato. Los logs se pierden al reiniciar. No hay forma unificada de investigar qué pasó.

**Solución:** El Core nunca escribe logs. En su lugar, dispara eventos al puerto de auditoría injected. El Shell es quien persiste esos eventos de forma inmutable (append-only: solo agregar, nunca borrar ni modificar).

**Resultado observable:** Auditoría completa del sistema basada en el **Principio de Inundación de Fundaciones (ADR-0020)**. Cada evento incluye metadatos para cumplimiento institucional (Institutional Compliance) y reconciliación de Nautilus.

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
    * Toda entrada DEBE incluir `process_id` y `institutional_tag` (ADR-0020).
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

## Puertos de Integración (ADR-0137)

> Obligatorio en toda feature. Define los tipos de dato que la feature acepta (inputs) y produce (outputs).
> Los IDs de tipo deben pertenecer al catálogo de ADR-0137. Un puerto sin tipo declarado es inválido en el Canvas [Forge/Reactor].

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `audit_content_in` | `(interno)` | Input | 1 | Payload de auditoría que el llamador entrega: `action_type`, `entity_kind`, `entity_id`, `details_json`, `process_id`. No es un tipo de canvas; es la firma Rust de la API de la feature (Enmienda 2026-06-24 de ADR-0137). |
| `audit_event_out` | `AuditEvent` | Output | 1..N | Evento auditado persistido de forma inmutable (append-only) con `audit_hash` y `audit_chain_hash`. Representa la salida canónica de la bitácora. |

> **Cardinalidad:** `1` = exactamente uno · `0..1` = opcional · `0..N` = múltiple · `1..N` = al menos uno.
> **Puerto interno (ADR-0137 Enmienda 2026-06-24):** `audit_content_in` es un puerto interno — plomería `textLabel`, payload específico de esta feature, nunca cableable en el canvas. No requiere tipo de catálogo.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020) — Perfil D (Ops/Auditoría):** `audit_events` aplica el Grupo I (universal) + Soberanía (II) + Hardware (IV). Los Grupos III y V (linaje Alpha, forense de ejecución) NO aplican a este perfil y se omiten — ver `migrations/0002_audit_log.sql`, que cita expresamente "PROHIBIDO copy-paste masivo" de ADR-0020.

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad & Integridad** | `id` | UUID del evento (`log_id`) |
| | `created_at` | Timestamp del evento (nanosegundos) |
| | `updated_at` | Igual a `created_at` (append-only) |
| | `audit_hash` | SHA-256 del contenido + enlace previo |
| | `audit_chain_hash` | `audit_hash` de la fila anterior (NULL solo en la fila génesis) |
| | `event_sequence_id` | Posición monótona en la cadena (orden de replay) |
| **II. Soberanía & Propiedad** | `owner_id` | Dueño del capital/IP (nullable: no todo evento tiene uno) |
| | `institutional_tag` | Entorno (PROD/PAPER/CHALLENGE) — obligatorio |
| | `manifest_id` | Contrato de diseño vinculado (nullable) |
| | `access_token_id` | Rastreo de autenticación (nullable) |
| **IV. Infraestructura & Ops** | `process_id` | Ancla del job que disparó el evento — obligatorio |
| | `session_id` | Agrupación de runtime (nullable) |
| | `node_id` | ID del hardware físico (nullable) |

- **Campos propios de la feature (TTR-001):** `action_type`, `entity_type`, `entity_id`, `details_json` — definidos en "Entrada"/"Restricciones" de este documento, no en ADR-0020.

- **Decisión Arquitectónica Asociada:**
    - ADR-0015: Arquitectura de Causalidad (Audit Log como fuente de verdad para Feedback).
    - ADR-0016: Local-First (Soberanía de datos de auditoría).
    - ADR-0020: Inundación de Fundaciones — Perfil D (Ops/Auditoría).

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para timestamps deterministas de alta precisión.

**Consumido por:**
- **Todos los Módulos del Pipeline:** Para registro de transiciones de estado y decisiones críticas.
- [`feedback`](../modules/feedback.md) — para análisis de causalidad y autopsias.
