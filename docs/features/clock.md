# Clock — Abstracción del Reloj del Sistema

**Carpeta:** `./features/clock/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

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
*   **Postcondición:** Registro del `ntp_sync_offset` en el rastro de auditoría (ADR-0020 V2).

### **TTR-002: Simulación de Reloj Determinista (Backtest-Ready)**
*   **Descripción:** Proporciona un reloj controlado para simulaciones reproducibles 100%.
*   **Reglas de Negocio:**
    * El reloj solo avanza mediante llamadas explícitas `advance(ns)`.
    * Toda lectura de reloj falso debe incluir el `virtual_process_id` (ADR-0020 V2).
*   **Entrada:** `initial_timestamp_ns`, `step_ns`.
*   **Salida:** `virtual_timestamp_ns`.
*   **Precondición:** Modo de ejecución `SIMULATION` activo.
*   **Postcondición:** El rastro de evidencia muestra la delta entre tiempo real y virtual.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Toda sincronización de tiempo y estado del reloj registra el set completo de **25 campos mandatorios** (ver ADR-0020 V2 V2).
    - Metadatos de precisión y sincronía: `audit_chain_hash` (Secuencia temporal), `logic_hash` (Time provider hash), `node_id`, `event_sequence_id`.
    - Integridad: `data_snapshot_id` (External NTP snapshot reference).

- **Decisión Arquitectónica Asociada:**
    - ADR-0002: Desacoplamiento de Persistencia (Timestamps como int64).
    - ADR-0013: Stack Tecnológico (NautilusTrader precision).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- Ninguna. Es una primitiva base.

**Consumido por:**
- **Todos los Módulos y Features:** Para la línea de tiempo inmutable del sistema.
