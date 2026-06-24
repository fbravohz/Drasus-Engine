# Clock — Abstracción del Reloj del Sistema

**Carpeta:** `./features/clock/`
**Estado:** ✅ Implementado — puerto del reloj (TTR-001, TTR-002) + rastro de auditoría (3 eventos a la bitácora existente).
**Última actualización:** 2026-06-12

> ✅ **Implementado** 2026-06-12 · Orden de trabajo [STORY-003](../execution/STORY-003-clock.md) · Núcleo determinista + cáscara `SystemClock` (Fase 1) + emisor de auditoría del reloj `clock_audit` (Fase 2: 3 eventos vía `AuditEventContent`). Auditado Tech-Lead: clippy `-D warnings` limpio, 28 tests verdes, FCIS y granularidad del hot-path verificados.
> 🏗️ **Perfil de auditoría resuelto** 2026-06-12 · Architect (escalamiento §3): eventos del reloj → bitácora existente (`audit-log`) vía `AuditEventContent`, Perfil D; campos huérfanos (`ntp_sync_offset`, proceso virtual, delta real/virtual) reclasificados como payload de `details_json`, NO campos de catálogo. Sin cambios a ADR-0020 V2. Granularidad acotada a 3 eventos (ver "Gobernanza y Estándares").

---

## ¿Qué es?

El Clock es un puerto inyectado que proporciona el tiempo actual a cualquier módulo que lo necesite. En producción devuelve el Unix timestamp real. En backtests y tests, puede inyectarse un reloj determinista que devuelve exactamente el tiempo que el test especifique.

**Problema:** Si los módulos llaman directamente a `datetime.now()` o equivalente, no se pueden reproducir backtests exactamente — cada ejecución obtiene un tiempo real diferente. Además, los tests carecen de control sobre el tiempo.

**Solución:** Todos los módulos del Core obtienen el tiempo a través de este puerto inyectado. El Shell es el responsable de proporcionar la implementación real (reloj del sistema) o la de testing (reloj determinista).

**Resultado observable:** Backtests y tests son 100% reproducibles — el mismo input + mismo reloj injected = exactamente mismo output, siempre.

---

## Comportamientos Observables

- [ ] Un módulo necesita saber la hora actual
  → Llama al puerto Clock inyectado
  → Obtiene Unix timestamp (número flotante de segundos desde epoch)
  → Usa ese timestamp para registrar eventos, comparar tiempos, calcular duraciones

- [ ] En producción, el reloj injected devuelve `datetime.now().timestamp()` actualizado
  → Cada llamada a Clock devuelve un valor ligeramente mayor al anterior

- [ ] En backtests, el usuario injected un reloj que devuelve tiempos fijos
  → El reloj comienza en 2020-01-01 09:30:00
  → Con cada simulación de barra, el reloj avanza exactamente 60 segundos (ej: timeframe de 1 minuto)
  → Todas las llamadas a Clock dentro de la barra devuelven el mismo timestamp
  → Al terminar el backtest, el timestamp final es exacto y reproducible

- [ ] En tests unitarios, el reloj injected devuelve tiempos configurables
  → Test 1 establece reloj en "2020-01-01 10:00:00"
  → Test 2 establece reloj en "2020-12-31 16:00:00"
  → Cada test es independiente, sin contaminación de tiempo real

---

## Restricciones

- **NUNCA un módulo llama a `datetime.now()` o equivalente directo.** Siempre a través de Clock.
- **NUNCA Clock devuelve un valor menor al anterior.** El tiempo es monótono creciente dentro de una sesión.
- **NUNCA un reloj inyectado cambia durante la ejecución de una operación atómica.** Si estás dentro de un trade, el reloj no avanza hasta que el trade termina.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| INITIAL_TIMESTAMP | (reloj real) | Cualquier Unix timestamp | En tests: el timestamp inicial que devuelve Clock |
| ADVANCE_PER_STEP | 0 (reloj real) | >= 0 segundos | En backtests: cuántos segundos avanza el reloj con cada barra |
| FROZEN | false | true / false | En tests: si true, Clock siempre devuelve INITIAL_TIMESTAMP (útil para tests de caché) |

---

## Ciclo de Vida de la Feature

### Entrada
- **Quién llama:** Cualquier módulo del Core que necesite el timestamp actual (ingest, generate, validate, incubate, manage, execute, withdraw, feedback)
- **Qué recibe:** Nada. Solo una llamada al puerto para obtener el tiempo.

### Proceso
- **En producción:** Cálculo trivial — convierte `datetime.now()` a Unix timestamp
- **En backtests:** Manejo de estado interno — el reloj mantiene un timestamp actualizado que avanza según la configuración

### Salida
- **Qué produce:** Un número flotante (Unix timestamp en segundos)
- **Cuándo:** Inmediatamente, de forma síncrona

### Contextos de Uso
- **Ingest:** Asigna timestamp a barras cuando se ingestan (para garantizar que la marca de tiempo es consistente en todo el sistema)
- **Execute:** Registra el timestamp de cada orden enviada, cada transición de estado
- **Validate:** Marca timestaps en resultados de tests para auditoría
- **Feedback:** Timestampa reconciliaciones diarias y anomalías detectadas
- **Audit Log:** Cada evento de auditoría incluye el timestamp actual del Clock

---

---

## Tareas (TTRs)

### **TTR-001: Proporcionar Timestamp de Alta Precisión (Nanosegundos)**
*   **Descripción:** Expone el Unix timestamp actual con precisión de nanosegundos (ADR-0013).
*   **Reglas de Negocio:**
    * En producción, utiliza `time.time_ns()` para evitar errores de precisión de punto flotante.
    * El tiempo DEBE ser monótonamente creciente (ADR-0013).
*   **Entrada:** `request_type` (REAL | FAKE).
*   **Salida:** `timestamp_ns` (int64).
*   **Precondición:** Sincronización NTP verificada.
*   **Postcondición:** Al verificarse la sincronía NTP en el arranque (NO en cada lectura — ver "Granularidad de Auditoría" abajo), emite UN evento de auditoría `CLOCK_NTP_SYNC` a través de la bitácora existente (`AuditEventContent`, audit-log.md). El `ntp_sync_offset` (delta NTP en ns, ADR-0013) NO es un campo del catálogo ADR-0020 V2: viaja como payload del evento dentro de `details_json` (campo opaco que ya expone `AuditEventContent`). Los campos del catálogo del evento son los de su Perfil Técnico (ver "Gobernanza y Estándares").

### **TTR-002: Simulación de Reloj Determinista (Backtest-Ready)**
*   **Descripción:** Proporciona un reloj controlado para simulaciones reproducibles 100%.
*   **Reglas de Negocio:**
    * El reloj solo avanza mediante llamadas explícitas `advance(ns)`.
    * Cada sesión de simulación se identifica con el `session_id` (Grupo IV del catálogo ADR-0020 V2), que es el campo canónico para agrupar un runtime — NO existe ningún `virtual_process_id` en el catálogo. El identificador del proceso virtual de la simulación viaja como payload dentro de `details_json` del evento de cierre de sesión.
*   **Entrada:** `initial_timestamp_ns`, `step_ns`.
*   **Salida:** `virtual_timestamp_ns`.
*   **Precondición:** Modo de ejecución `SIMULATION` activo.
*   **Postcondición:** Al cerrarse la sesión de simulación (NO en cada `advance` ni en cada lectura), emite UN evento de auditoría `CLOCK_SESSION_CLOSE` con la delta acumulada entre tiempo real y virtual como payload dentro de `details_json`. La delta real/virtual NO es un campo del catálogo ADR-0020 V2: es payload propio del evento del reloj.

---

## Puertos de Integración (ADR-0137)

> Obligatorio en toda feature. Define los tipos de dato que la feature acepta (inputs) y produce (outputs).
> Los IDs de tipo deben pertenecer al catálogo de ADR-0137. Un puerto sin tipo declarado es inválido en el Canvas [Forge/Reactor].

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `timestamp_out` | `timestamp_ns` | Output | 1 | Unix timestamp en nanosegundos que el Clock entrega en cada solicitud. En producción refleja el tiempo real del sistema; en backtest/test devuelve el tiempo determinista inyectado. |

> **Cardinalidad:** `1` = exactamente uno · `0..1` = opcional · `0..N` = múltiple · `1..N` = al menos uno.
> **Sin puertos internos de entrada:** el Clock no recibe payload externo — es una primitiva pura. El llamador solo invoca la función; no hay dato que inyectar. No aplica la categoría de puerto interno (ADR-0137 Enmienda 2026-06-24) porque no existe contrato de entrada que documentar.

---

## Gobernanza y Estándares (Fijos)

### Persistencia y Perfil de Auditoría
El reloj NO tiene persistencia propia. Sus eventos auditables se emiten a través de la bitácora existente (`audit-log`, STORY-004) usando su interfaz `AuditEventContent` y su repositorio append-only — NO se crea una tabla nueva. Los timestamps que esos eventos llevan (`created_at`) los inyecta el propio reloj vía el puerto `Clock`, igual que para cualquier otro emisor.

- **Perfil Técnico (ADR-0020 V2): D — Ops/Auditoría.** Los eventos del reloj son registro forense de sincronía/reconciliación temporal; encajan en el mismo perfil que `audit-log` (Grupo I universal + Grupo II Soberanía + Grupo IV Infraestructura). Los Grupos III (Linaje Alpha) y V (Forense de Ejecución) NO aplican y se omiten. PROHIBIDO copy-paste de los 25 campos.
- **Campos del catálogo que lleva cada evento del reloj** (los que `AuditEventContent` ya expone para el Perfil D):
    - Grupo I (lo asigna la bitácora al persistir): `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - Grupo II: `institutional_tag` (obligatorio), `owner_id`/`manifest_id`/`access_token_id` (opcionales).
    - Grupo IV: `process_id` (obligatorio), `session_id` (agrupa la sesión de simulación), `node_id` (opcional).
- **Payload propio del reloj** (dentro de `details_json`, opaco al catálogo): `ntp_sync_offset` (delta NTP en ns), identificador del proceso virtual de simulación, y la delta acumulada real/virtual. Ninguno es campo del catálogo ADR-0020 V2.

### Granularidad de Auditoría (Crítico de Rendimiento)
PROHIBIDO auditar cada lectura de `timestamp_ns()` / `advance(ns)`: se invocan en el hot-path millones de veces y auditarlas saturaría la bitácora. El conjunto MÍNIMO de eventos auditables del reloj es:

| Evento (`action_type`) | Cuándo se emite | Payload en `details_json` |
|---|---|---|
| `CLOCK_NTP_SYNC` | Una vez al verificar la sincronía NTP en el arranque (TTR-001) | `ntp_sync_offset` (ns) |
| `CLOCK_MODE_TRANSITION` | Al transicionar de modo `REAL` ↔ `SIMULATION` | modo origen, modo destino |
| `CLOCK_SESSION_CLOSE` | Al cerrar una sesión de simulación (TTR-002) | id del proceso virtual, delta acumulada real/virtual |

`entity_type` = `CLOCK`; `entity_id` = el `session_id` de la sesión activa.

### Decisión Arquitectónica Asociada
- ADR-0002: Desacoplamiento de Persistencia (Timestamps como int64).
- ADR-0013: Stack Tecnológico (precisión NTP/timestamps; el `ntp_sync_offset` es dato de ADR-0013, no campo de catálogo).
- ADR-0015: Arquitectura de Causalidad (el reloj emite a la bitácora, fuente de verdad para Feedback).
- ADR-0020 V2: Inundación de Fundaciones — Perfil D (Ops/Auditoría).

---

## Dependencias
**Depende de:**
- Ninguna a nivel de núcleo. El reloj es una primitiva base: su lógica determinista NO importa ninguna otra feature (es la bitácora la que recibe el puerto `Clock`, no al revés — sin ciclo).
- Solo para su rastro de auditoría (cáscara, no núcleo): emite sus 3 eventos a través de [`audit-log`](./audit-log.md) vía `AuditEventContent`. Esta emisión vive en la capa de orquestación, fuera del núcleo determinista, por lo que no contamina la reproducibilidad bit-a-bit.

**Consumido por:**
- **Todos los Módulos y Features:** Para la línea de tiempo inmutable del sistema.
