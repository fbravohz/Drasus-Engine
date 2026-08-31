# Sovereign Security — Seguridad Soberana del Sistema

**Carpeta:** `./features/sovereign-security/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-04
**Decisión Arquitectónica Asociada:** ADR-0093 (Arquitectura de Seguridad Soberana)

---

## ¿Qué es esta feature?

Sovereign Security establece el marco de ciberseguridad local e integridad de datos para Drasus Engine. Protege las credenciales sensibles del bróker mediante criptografía fuerte, mantiene un registro de transacciones inmutable a prueba de manipulación mediante encadenamiento de hashes, y garantiza la privacidad total del usuario al deshabilitar cualquier telemetría.

---

## Narrativa de Producto

**Capacidad:** `CAP-06` — Ejecución real y defensa del capital

**Historia de usuario:** Como trader, quiero que las contraseñas de mi bróker nunca salgan de mi computadora, sin importar el plan que tenga contratado, para que ni un error de Drasus ni una decisión de negocio futura puedan exponer el acceso a mi cuenta real.

**El problema (técnico):** una credencial de bróker guardada en texto plano, o transmitida a cualquier servidor externo, es la llave completa a la cuenta de un usuario. Un registro de auditoría que se puede editar después de escrito es, además, inútil como evidencia — cualquier manipulación posterior se vuelve indetectable.

**La solución (técnica):** las credenciales del bróker se cifran localmente con un algoritmo de cifrado fuerte antes de guardarse, usando una llave que nunca sale de la máquina del usuario — invariante fijo, sin excepción por nivel de suscripción ni por estado de pago. El registro de auditoría es de solo-agregado: cada fila lleva el sello de la anterior encadenado por hash, así que alterar un registro pasado rompe visiblemente la cadena.

**Explicado sin tecnicismos:** es como la caja fuerte de tu propia casa, no la de un banco. Nadie más tiene la combinación, ni siquiera Drasus, y no importa si dejas de pagar la mensualidad de un servicio — la caja sigue siendo tuya y sigue en tu casa.

Ninguna contraseña de tu bróker sale nunca de tu computadora, en ningún plan, gratis o de paga. Y cada operación que haces queda anotada en una bitácora que nadie puede corregir después sin que se note — como un cuaderno donde cada página lleva pegado un pedazo de la anterior.

---

## Comportamientos Observables

- [ ] El usuario registra credenciales de un bróker
  → El sistema lee la Master Key desde variables de entorno
  → Encripta el API Secret usando AES-256-GCM antes de guardarlo en `broker_connections`
  → El registro guardado en la base de datos contiene solo bytes cifrados y no texto plano

- [ ] El módulo de ejecución realiza un trade
  → Registra de forma secuencial la transacción en `audit_log`
  → El sistema calcula el hash SHA-256 de la fila actual y lo vincula con el hash del registro anterior (`audit_chain_hash`)
  → Genera un registro de replay inmutable con el indicador técnico en `indicator_snapshots`

- [ ] El sistema inicia su ciclo operativo
  → No realiza ninguna petición HTTP/WebSocket externa que no sea hacia los endpoints configurados de brokers/datos
  → No se envían estadísticas de uso, métricas de hardware o datos de estrategias a ningún servidor central (Cero Telemetría)

---

## Restricciones

- **NUNCA guardar llaves de API en texto plano** en base de datos o archivos de configuración.
- **NUNCA modificar o eliminar filas** de las tablas `audit_log` y `events` (Append-Only absoluto).
- **NUNCA activar servicios de telemetría o analíticas** en segundo plano.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| `ENCRYPTION_ALGORITHM` | "AES-256-GCM" | Fijo | Algoritmo de cifrado de credenciales | **[FIJO]** |
| `MASTER_KEY_ENV_VAR` | "DRASUS_MASTER_KEY" | Cadena | Variable de entorno que contiene la llave maestra | **[FIJO]** |
| `SOVEREIGN_PRIVACY_MODE` | true | true | Forzar cero telemetría externa | **[FIJO]** |

---

## Ciclo de Vida de la Feature

### Entrada
- Credenciales de API (texto plano)
- Acciones de trading y respuestas del bróker
- Estado de indicadores al momento de la orden

### Proceso
- Encriptación simétrica AES-256-GCM de textos planos usando Master Key de entorno.
- Generación de hashes de cadena SHA-256 (`audit_chain_hash`) para ligar registros contiguos de auditoría.
- Bloqueo de peticiones externas no solicitadas.

### Salida
- Credenciales encriptadas en `broker_connections`.
- Historial inmutable encadenado en `audit_log` y `events`.
- Snapshots selectivos guardados en `indicator_snapshots`.

### Contextos de Uso

**Contexto 1: Conexión con Brókers**
- Proporciona desencriptación segura en memoria al vuelo durante llamadas API.

**Contexto 2: Auditoría y Cumplimiento**
- Proporciona un rastro de auditoría inalterable útil para la reconciliación de NautilusTrader y verificación de operaciones.

---

## Tareas (TTRs)

### **TTR-001: Cifrado y Descifrado de API Keys (Broker Key Safety)**
*   **Descripción:** Implementa el cifrado/descifrado simétrico de las credenciales de broker persistidas.
*   **Reglas de Negocio:**
    * Cifrado con AES-256-GCM usando la variable de entorno `DRASUS_MASTER_KEY`.
    * Lanzar error crítico y abortar inicio si la variable de entorno no está definida.
*   **Entrada:** API Key / Secret (texto plano).
*   **Salida:** API Key / Secret (cifrado, bytes) y nonce de cifrado.
*   **Precondición:** Master Key inyectada y válida.
*   **Postcondición:** Datos ilegibles persistidos en `broker_connections`.

### **TTR-002: Encadenamiento de Auditoría Criptográfica (Immutable Audit Trail)**
*   **Descripción:** Garantiza el enlace y validación secuencial inmutable de los registros de trading.
*   **Reglas de Negocio:**
    * Cada fila de `audit_log` y `events` calcula su `audit_hash` con SHA-256.
    * El campo `audit_chain_hash` debe almacenar el `audit_hash` del registro anterior.
*   **Entrada:** Fila de auditoría con datos de la transacción.
*   **Salida:** `audit_hash` y `audit_chain_hash` inyectados en la fila.
*   **Precondición:** Transacción de base de datos activa.
*   **Postcondición:** Fila persistida de forma inmutable.

### **TTR-003: Persistencia Selectiva de Snapshots de Indicadores**
*   **Descripción:** Guarda los estados de los indicadores en momentos específicos de señal.
*   **Reglas de Negocio:**
    * Solo registrar en eventos de entrada/salida de señal o apertura/cierre de trades.
*   **Entrada:** `strategy_id`, `trade_id`, `event_type`, valores de indicadores y parámetros.
*   **Salida:** Fila insertada en `indicator_snapshots`.
*   **Precondición:** Señal detectada o trade ejecutado.
*   **Postcondición:** Registro disponible para Time-Travel Debugging futuro.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. No depende de ningún KMS o almacenamiento en la nube externo.
- **Inundación de Fundaciones (ADR-0020):**
  - Aplica Perfil **Ops / Auditoría** para las tablas de auditoría y eventos.
  - Campos de persistencia obligatorios: `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`, `owner_id`, `institutional_tag`, `process_id`, `session_id`, `node_id`.

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para marcas de tiempo deterministas.

**Consumido por:**
- [`execute`](../modules/execute.md) — para encriptar credenciales de broker y auditar ejecuciones.
- [`feedback`](../modules/feedback.md) — para reconstruir auditorías.
