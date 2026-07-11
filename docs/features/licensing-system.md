# Sistema de Licenciamiento (Licensing System)

> 🟡 **Parcial** 2026-07-04 · Orden de trabajo [STORY-028](../execution/STORY-028-licensing-system.md) · Gate local completo: migración `0008_licensing_system.sql` (Grupo I + Perfil D + `row_version`, tabla `licenses`), Core puro (`domain/licensing_system.rs`: verificación de firma **Ed25519 asimétrica** — NO HMAC —, comparación de huella reutilizada de `AccountIdentity`, heartbeat/gracia determinista, supresión de telemetría por tier, derivación de `ExecutionGate`), Shell (`persistence/licensing_system.rs`: repositorio con concurrencia optimista; `orchestrator/licensing_system.rs`: emisor de licencias de desarrollo stub Ed25519, proveedor de `PlanLimits` stub, caché del veredicto con TTL), puerto `execution_gate_out` → `ExecutionGate` en `public_interface.rs`, CLI `verify licensing-system` (ADR-0142). Crate: `crates/shared` (excepción bendecida ADR-0137). Pendiente: emisor real de licencias en la Cabina de Mando, adaptador real de `plan-tier-quota` (#3) para `plan_limits_in`, UI del panel de licencia/tier (Superficie propia, deuda de integración).

**Carpeta:** `./features/licensing-system/`  
**Estado:** 🟡 Parcial (gate local completo; emisor central, `plan-tier-quota` real y UI diferidos)  
**Última actualización:** 2026-07-04  
**Decisión Arquitectónica Asociada:** ADR-0020 (Inundación de Fundaciones) · ADR-0143 (Soberanía Condicionada por Tier) · ADR-0144 (Substrato de Monetización, cimiento #2)

> 🔶 **Enmendado por ADR-0143 (2026-07-03)** — el modelo dual Sovereign/Explorer se re-encuadra como el modelo de tiers de ADR-0143. **"Cero telemetría absoluta" queda derogado:** toda instancia mantiene un canal de control obligatorio (identidad/licencia/heartbeat); lo que el tier de pago obtiene es la **supresión en origen de la telemetría de trabajo**, no la ausencia total de canal. Esta feature es el **cimiento #2** del substrato: además de validar la licencia, actúa como el **gate** que ordena suprimir/reactivar la telemetría y cuenta las **activaciones simultáneas por tier**.

---

## 1. ¿Qué es esta feature?

El sistema de licenciamiento regula los niveles de acceso del usuario al ecosistema Drasus Engine y es el **gate** que decide, antes de cada operación sensible, si se ejecuta y si se suprime la telemetría de trabajo (ADR-0143). Vincula la licencia a la identidad (`central-identity`) y a la huella de hardware, y controla las activaciones simultáneas por tier.

* **Problema:** el negocio necesita cobrar y prevenir el abuso multi-instancia sin poner una llamada de red síncrona en el hot-path ni bloquear al usuario honesto cuando pierde conexión.
* **Comportamiento observable:** el usuario puede operar offline durante un período de gracia; el sistema valida la licencia de forma asíncrona y, según el tier, apaga o enciende la emisión de telemetría de trabajo en su máquina.
* **Niveles de Licencia (tiers de ADR-0143):**
  * **Sovereign Tier (pago al corriente):** privacidad real — la telemetría de **trabajo** (estrategias, backtests, portafolios, resultados) se **suprime en origen**. Se conserva solo el canal mínimo de control (licencia/heartbeat/anti-abuso). Los secretos nunca salen, en ningún tier (ADR-0093).
  * **Explorer Tier (gratuito):** costo cero a cambio de que el trabajo del usuario alimente a la Cabina de Mando del proveedor (firehose, dueño por ToS — `consent-registry`). Si un usuario de pago deja de pagar, degrada a este comportamiento (sin borrar su entorno).

---

## 2. Comportamientos Observables

* **Validación de Huella de Hardware:**
  * Al iniciar, la aplicación lee los identificadores físicos de la máquina (placa base, CPU) y genera una firma criptográfica única.
  * Si los identificadores no coinciden con la firma registrada en el archivo local de licencia, el sistema deshabilita las operaciones de trading en vivo y muestra una alerta al usuario.

* **Validación de Heartbeat (Periodo de Gracia):**
  * El sistema permite la operación sin conexión a internet durante un periodo configurable.
  * Al aproximarse al límite sin conexión, la interfaz muestra notificaciones preventivas sugiriendo al usuario una conexión momentánea para el refresco del certificado de la licencia.
  * Si se supera el límite absoluto sin validación, el motor restringe la creación de nuevos backtests y operaciones en vivo hasta que se valide la firma.

* **Transferencia de Licencia entre Máquinas (activaciones):**
  * El usuario puede **desvincular** una máquina (huella de hardware) desde su Plano de Control para liberar un cupo de activación y reclamarlo en otra máquina (self-service, sin intervención del proveedor).
  * La invalidación de la máquina desvinculada es autoritativa en la Cabina de Mando (adaptador de red diferido); localmente, la máquina liberada degrada su gate en el siguiente heartbeat.
  * Un cupo liberado no se puede reclamar hasta pasado `TRANSFER_COOLDOWN`, evitando la rotación de una licencia por muchas máquinas.
  * **Si la máquina desvinculada está destruida (disco muerto, ADR-0146):** el usuario no puede operar esa máquina para desvincularla — la liberación se hace desde el **panel de cuenta en la Cabina de Mando** (diferido), tras verificación de identidad. [`instance-continuity`](instance-continuity.md) (#11) reutiliza este mismo flujo self-service para liberar, además del cupo de activación, la **titularidad de custodia** de la cadena de auditoría.

* **Cambio de Tier a Mitad de Ciclo (downgrade — garantía reduce-only):**
  * Si el usuario baja de tier a mitad de ciclo, el nuevo límite aplica **hacia adelante**; el contador de nocional del ciclo (`usage-metering`, append-only) **no se reinicia** por el cambio.
  * El `ExecutionGate` restringe **solo la exposición nueva o incremental**. **NUNCA bloquea el cierre de una posición abierta** (órdenes reduce-only): degradar un tier o alcanzar la cuota jamás atrapa el capital ya desplegado del usuario. El bloqueo por cuota/tier es sobre "abrir más", no sobre "salir".

---

## 3. Restricciones

* **PROHIBIDO** realizar llamadas síncronas de validación de red en el bucle principal de ejecución de órdenes (*Hot-Path*).
* **PROHIBIDO** almacenar claves privadas de firma de licencias dentro del ejecutable o código fuente del cliente.
* **PROHIBIDO** deshabilitar el funcionamiento del actualizador de emergencia o de la auditoría local si la licencia expira.
* **PROHIBIDO (FIJO)** que el `ExecutionGate` bloquee una orden **reduce-only** (de cierre): ningún estado de licencia, cuota o tier puede impedir salir de una posición abierta. El gate restringe abrir/incrementar exposición, jamás atrapa capital ya desplegado.
* **Una sola instancia por máquina (FIJO):** corre **una** instancia de Drasus por máquina, identificada por su huella de hardware. Un segundo arranque en la misma máquina comparte la huella y NO cuenta como una segunda activación. Las *activaciones* del tier cuentan **máquinas distintas** (p. ej. 1 laptop personal + 2 nodos VPS headless), no procesos; esas máquinas se fusionan en **una sola interfaz** vía el Plano de Control del usuario (ADR-0119).

---

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| HEARTBEAT_INTERVAL | 90 días | 30 - 360 días | Tiempo límite permitido de ejecución local antes de requerir un refresco en línea. | CONFIG |
| RECHECK_WINDOW | 5 días | 1 - 15 días | Ventana previa al vencimiento del heartbeat donde se inician las alertas visuales en la interfaz. | CONFIG |
| GRACE_PERIOD | 7 días | 0 - 30 días | Días adicionales de ejecución permitida tras vencer el heartbeat antes del bloqueo funcional. | CONFIG |
| ACTIVATIONS_PER_TIER | Explorer 1 / Sovereign 3 | 1 - N | **Máquinas distintas** (por huella de hardware) autorizadas en simultáneo — una instancia por máquina. Explorer = 1; Sovereign = 3 (típico: 1 laptop personal + 2 nodos VPS headless), fusionadas en una sola interfaz (ADR-0119). El límite real por plan lo fija `plan-tier-quota`. | CONFIG |
| SUPPRESS_WORK_TELEMETRY_ON_PAID | true | true/false | En Sovereign Tier al corriente, apaga en origen la emisión de telemetría de trabajo (ADR-0143). | FIJO |
| TRANSFER_COOLDOWN | 7 días | 0 - 30 días | Enfriamiento tras liberar una activación (desvincular una máquina) antes de poder reclamar ese cupo en otra — frena la rotación abusiva de activaciones (una licencia rotando por muchas máquinas). | CONFIG |

---

## 5. Estructura Interna (FCIS)

* **Core (Lógica Pura):**
  * Algoritmo de hashing y firma criptográfica para validar el archivo de licencia contra los identificadores de hardware.
  * Comparador determinista de marcas de tiempo y validez del certificado.
* **Shell (Infraestructura):**
  * Lectores de datos físicos del sistema operativo (interfaz con el hardware local).
  * Gestor de persistencia del archivo de licencia en la base de datos local y almacenamiento seguro del sistema.
* **Frontera Pública:**
  * Interfaz de consulta para comprobar la validez de la licencia y el tier activo (Sovereign / Explorer).

---

## 6. Ciclo de Vida de la Feature

### Entrada
* Identificadores crudos del hardware de la máquina local.
* Archivo de licencia firmado criptográficamente.
* Reloj del sistema (validado contra fuentes de tiempo locales protegidas).

### Proceso
* Verifica la firma **asimétrica Ed25519** del archivo de licencia con la clave PÚBLICA incrustada en el cliente (la clave PRIVADA firma solo en el emisor — Cabina de Mando real, o el stub local de desarrollo — y jamás sale de ahí; ADR-0093 §3, corrección obligatoria de STORY-028: HMAC quedó descartado por ser simétrico).
* Compara el `node_id` (huella de hardware) del archivo de licencia contra el `node_id` que trae `AccountIdentity` (puerto `identity_in`, producido por `central-identity`) — sin recalcular la huella.
* Verifica si la fecha actual es menor a la fecha de expiración del heartbeat local (con reloj determinista inyectado).

### Salida
* Veredicto de validación: LICENCIA_VÁLIDA / LICENCIA_INVÁLIDA / REQUIERE_REFRESCO.
* Nivel de acceso autorizado (Sovereign o Explorer).

---

## 7. Tareas (TTRs)

### TTR-001: Generación de Huella Digital de Hardware
* **¿Cuál es el problema?**  
  Necesitamos ligar la licencia a una máquina específica para evitar la clonación no autorizada del software comercial en múltiples servidores, sin violar la privacidad del usuario ni almacenar datos personales.
* **¿Qué tiene que pasar?**  
  El sistema recopila datos de hardware locales estables y genera un hash único con `SHA-256` (huella de máquina = `node_id`, digest de una vía sin clave — es la huella reutilizada de `central-identity`/`AccountIdentity`, "SHA-256 de identificadores de máquina"). **No es `HMAC`** (que sería un hash con clave simétrica): esta huella es un digest público, distinto de la **firma de licencia**, que sí es asimétrica `Ed25519` (clave privada solo en el emisor, ADR-0093). Este hash se valida contra el archivo de licencia importado por el usuario.
* **¿Cómo sé que está hecho?**  
  * [ ] El hash se genera de manera idéntica en el mismo equipo en múltiples arranques del sistema.
  * [ ] Si se altera el archivo de configuración de hardware simulado, el sistema detecta el cambio de firma.
* **¿Qué no puede pasar?**  
  * No se pueden transmitir los identificadores de hardware en crudo a ningún servidor externo.

### TTR-002: Verificación de Heartbeat Temporal
* **¿Cuál es el problema?**  
  El software debe verificar periódicamente que la licencia no ha sido cancelada o modificada (ej. reembolsos de Stripe), pero debe hacerlo de forma silenciosa e invisible para no molestar a los usuarios honestos.
* **¿Qué tiene que pasar?**  
  El sistema mantiene una fecha límite en el archivo de licencia local. Si el sistema detecta que la fecha límite se aproxima, activa alertas en la interfaz gráfica. Si se excede el periodo de gracia, suspende operaciones comerciales.
* **¿Cómo sé que está hecho?**  
  * [ ] El sistema inicia alertas cuando el tiempo restante es menor que la ventana de verificación.
  * [ ] El sistema desactiva el trading en vivo al llegar al límite absoluto si no hay conexión para revalidar.
* **¿Qué no puede pasar?**  
  * No se puede bloquear la aplicación de inmediato ante una pérdida repentina de conexión a internet.

---

## 8. Gobernanza y Estándares (Fijos)

* **Local-First (ADR-0016):** 100% Local. La validación se realiza en la máquina del usuario; la red solo se utiliza asíncronamente para refrescar el token de heartbeat.
* **Inundación de Fundaciones (ADR-0020):**
  * **Perfil D (Ops / Auditoría):** Foco en Identidad del Hardware, Soberanía de los Datos del Cliente y Auditoría Local de Accesos.
  * **I. Identidad & Integridad (Grupo I completo):** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, **`row_version`**. Tabla **mutable** (el heartbeat refresca la validez en sitio) → `row_version` para concurrencia optimista, NO `event_sequence_id UNIQUE` (ese patrón es solo para tablas append-only; ADR-0141). El historial de cambios de licencia va al `audit-log` existente, no a esta tabla.
  * **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `access_token_id`.
  * **IV. Infraestructura & Ops:** `node_id` (huella de hardware), `process_id`.
  * **V. Forense (Gobernanza):** `signature_hash` (firma de hardware), `compliance_status_id` (estado de la licencia).
  * **Hooks Forenses:** Registro de intentos fallidos de validación de firma de hardware en el log local protegido.
* **Contrato de Persistencia:**  
  Los metadatos de la licencia se guardan cifrados en el almacén local del sistema utilizando claves derivadas de la huella digital.

---

## 9. Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `identity_in` | `AccountIdentity` (plomería, ADR-0144) | Input | `1` | Identidad y huella de hardware (`node_id`) de la instancia, producida por [`central-identity`](central-identity.md). La licencia se **valida contra** esta huella; NO se re-deriva aquí (reutilización, ADR-0144 FIJO). |
| `plan_limits_in` | `PlanLimits` (plomería, ADR-0144) | Input | `1` | Límites vigentes del plan (activaciones, volumen nocional, features), producidos por `plan-tier-quota` (cimiento #3). **Aún no construido → se cablea a un stub local**; el adaptador real llega con #3 (puerto ahora, adaptador después, ADR-0144). |
| `execution_gate_out` | `ExecutionGate` (plomería, ADR-0144) | Output | `1` | Veredicto de ejecución **`{Allow / Deny / UpgradeRequired}`** + orden de supresión/reactivación de telemetría de trabajo por tier (ADR-0143). Consumido por el hot-path de `execute` (¿puedo operar?) y por `telemetry` (¿debo suprimir la emisión?) → ≥2 consumidores. |

> Tipos técnicos del substrato (plomería del Plano de Control del proveedor, no de dominio del canvas), análogos a `AuditEvent`/`TelemetrySample`. Registrados en el catálogo de ADR-0137 vía la enmienda de ADR-0144 (2026-07-03). Residencia en `crates/shared` (mismo criterio bendecido que `central-identity`/`audit-log`/`telemetry`).

> **Orden de dependencia:** este cimiento (#2) consume `AccountIdentity` de #1 (real) y `PlanLimits` de #3 (stub hasta que exista). No introduce acoplamiento entre features de dominio: todo pasa por puertos tipados sobre `shared`.
