# Instance Continuity (Continuidad y Portabilidad de Instancia)

> 🟡 **Parcial** 2026-07-06 · Orden [STORY-039](../execution/STORY-039-instance-continuity.md) · Core (KDF Argon2id + cifrado autenticado AES-256-GCM con nonce sembrado inyectado + filtro que EXCLUYE secretos de bróker/IPs live + gate de titularidad exclusiva `custody_epoch`→`CustodyConflict`) + esquema (`migrations/0017_instance_continuity.sql`: `instance_backups` append-only atómica + `custody_state` mutable) + puertos `identity_in`/`backup_blob_out`/`custody_status_out` + CLI `verify instance-continuity`. Consume `AccountIdentity` real de #1. **Auditoría TL independiente aprobada + QA APTO por mutación** (32/38 cazados; seguridad 100% cazada; 3 sobrevivientes = `canonical_delta_bytes` sin valor-dorado → DEBT-015, no bloqueante). Pendiente (diferido, disparador "primer cobro real, tras EPIC-5"): adaptador de almacén de objetos (S3/R2), liberación forzada de titularidad desde la Cabina de Mando, y la Cáscara Visual (toggle de respaldo + indicador de titularidad).

**Carpeta:** `./features/instance-continuity/`
**Estado:** Core + esquema + puertos implementados (adaptador de red y UI diferidos)
**Última actualización:** 2026-07-06
**Decisión Arquitectónica Asociada:** ADR-0146 (cimiento #11) · ADR-0093 (secretos jamás salen, cifrado client-side) · ADR-0143 (tres planos) · ADR-0145 (motivo de urgencia — atestación soberana irremplazable)

## ¿Qué es esta feature?

Dos comportamientos relacionados bajo un mismo mecanismo: (1) un **respaldo cifrado** de la base de datos local del usuario hacia el almacenamiento de objetos del proveedor, cifrado del lado del cliente de modo que el proveedor jamás puede leerlo; y (2) un **relevo de custodia** que permite que la misma cuenta maestra opere alternadamente desde varias máquinas activadas del usuario (ej. laptop y PC de escritorio), llevando su estado consigo, sin que dos máquinas escriban la cadena de auditoría al mismo tiempo.

- **Problema:** un disco muerto o un robo de laptop borra el trabajo del usuario. El tier de pago (Sovereign) suprime la telemetría de trabajo en origen (ADR-0143) — su historial NO está en los servidores del proveedor, así que un fallo de hardware lo pierde por completo. Además, cambiar de máquina o alternar entre dos máquinas propias hoy no tiene un mecanismo limpio: solo existe la transferencia de licencia (libera un cupo), que no recupera datos.
- **Comportamiento observable:** el usuario activa "respaldo en la nube"; al cerrar sesión en una máquina, sube un blob cifrado incremental y cede la titularidad de la cadena; al abrir en otra máquina activada, la reclama y continúa exactamente donde quedó.
- **Por qué:** convierte la garantía de soberanía en un producto vendible ("respaldamos tu trabajo y ni siquiera nosotros podemos verlo") y resuelve, sin construir nada nuevo de licenciamiento, el caso más común de portabilidad — cambiar o alternar de computadora — sin abuso de licencia ni pérdida de historial.

## Comportamientos Observables

- Cuando el usuario activa el respaldo → la app cifra la DB local con una clave derivada de su secreto maestro y sube el blob incremental.
- Cuando el proveedor recibe el blob → almacena bytes opacos (sin clave, sin capacidad de descifrar).
- Cuando el usuario cierra sesión en una máquina que era titular de la cadena → sube el snapshot y marca esa máquina como **no-titular**.
- Cuando el usuario abre Drasus en otra máquina activada de la misma cuenta → descarga el último snapshot, lo descifra localmente, y esa máquina **reclama la titularidad** — pasa a ser el único escritor activo de la cadena de auditoría.
- Cuando el usuario intenta tener dos máquinas titulares a la vez → la segunda queda bloqueada ("Esta cuenta está activa en otra máquina") hasta un reclamo explícito — nunca operan en paralelo silenciosamente.
- Cuando la máquina anterior está destruida (disco muerto) → el usuario reclama la titularidad desde el panel de cuenta en la Cabina de Mando (diferido), tras verificación de identidad — mismo flujo que desvincular una máquina muerta en `licensing-system`.
- Cuando el usuario pierde el secreto maestro → el respaldo es irrecuperable **por diseño** (el proveedor no tiene puerta trasera). Se advierte explícitamente en la UI.

## Restricciones

- **NUNCA (FIJO)** la clave de cifrado ni el secreto maestro salen de la máquina del usuario (ADR-0093). El cifrado es client-side; el servidor solo ve bytes opacos.
- **NUNCA** se incluyen en el blob los secretos de bróker ni las IPs de servidores live (ADR-0093).
- **NUNCA** dos máquinas escriben la cadena de auditoría de la misma cuenta maestra al mismo tiempo — la custodia es exclusiva y detectable (conflicto de titularidad).
- **NUNCA** el relevo de custodia exige que la máquina anterior esté viva — puede forzarse desde el panel de cuenta central si la máquina murió.
- El proveedor **no puede** ofrecer recuperación de contraseña del blob (no tiene la clave): la pérdida del secreto maestro implica pérdida del respaldo.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| BACKUP_ENABLED | false | true/false | Activa el respaldo cifrado (opt-in explícito) | CONFIG |
| BACKUP_INTERVAL | 24 h | 1 h – 30 d | Cadencia de subida del blob incremental | CONFIG |
| CLIENT_SIDE_ENCRYPTION | AES-256-GCM | (fijo) | Algoritmo de cifrado autenticado del blob | FIJO |
| BACKUP_RETENTION | 30 días | 1 – 365 días | Cuántas versiones del blob se conservan en el objeto remoto | CONFIG |
| CUSTODY_HANDOFF_MODE | manual | manual / automático-al-cerrar | Si el relevo de custodia se dispara al cerrar sesión o requiere confirmación explícita | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** derivación de clave desde el secreto maestro (KDF estándar), cifrado/descifrado AES-256-GCM autenticado, cálculo del delta a respaldar, verificación determinista de titularidad de custodia (¿esta máquina es la titular vigente?).
- **Shell (Infraestructura):** cliente de almacenamiento de objetos (S3/R2, adaptador diferido), programador de la cadencia, lectura consistente de la DB local (snapshot), gate de titularidad consultado al iniciar la app.
- **Frontera Pública:** puerto "respalda/restaura este entorno" + puerto "¿soy la máquina titular?" — el blob cifrado y el estado de titularidad son los únicos artefactos que cruzan a la Cabina de Mando.

## Ciclo de Vida de la Feature — Instance Continuity

### Entrada
Snapshot consistente de la base de datos local, el secreto maestro del usuario, y el estado de titularidad vigente (qué máquina es la escritora activa).

### Proceso
Deriva la clave, cifra el delta, lo sube; al cerrar sesión cede la titularidad; al abrir en otra máquina la reclama tras descargar y descifrar el último snapshot.

### Salida
Un blob cifrado remoto restaurable, y en cada momento exactamente una máquina marcada como titular de la cadena de auditoría de la cuenta.

## Tareas (TTRs)

- **TTR-001:** Cifrado/descifrado client-side del snapshot y subida incremental (adaptador de almacén de objetos diferido).
- **TTR-002:** Gate de titularidad de custodia — una sola máquina escritora activa, con detección de conflicto.
- **TTR-003:** Liberación forzada de titularidad desde el panel de cuenta central cuando la máquina anterior no puede ceder (diferido junto con la Cabina de Mando).

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `identity_in` | `AccountIdentity` (plomería, ADR-0144) | Input | `1` | Identidad de cuenta a la que pertenece el blob y la titularidad, producida por `central-identity`. |
| `backup_blob_out` | Blob cifrado de respaldo (tipo técnico nuevo — plomería, ADR-0146) | Output | `0..1` | Snapshot cifrado listo para subir al almacén de objetos (adaptador diferido). |
| `custody_status_out` | Estado de titularidad (tipo técnico nuevo — plomería, ADR-0146) | Output | `1` | ¿Esta máquina es la titular vigente de la cadena de auditoría de la cuenta? Consumido por el arranque de la app y por `licensing-system`. |

> Tipos técnicos nuevos del cimiento #11, registrados en el catálogo de ADR-0137 vía la enmienda de ADR-0146. Nombres canónicos de `struct` los fija el ingeniero.

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: toggle de "respaldo en la nube" + indicador de titularidad de la máquina actual, en el cajón de ajustes. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016), re-escopado por ADR-0143:** coherente con el tier de pago — el proveedor almacena pero no lee. En el tier gratuito el trabajo ya fluye en claro por el firehose; el respaldo cifrado es un perk especialmente valioso del tier de pago.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría)**: Identidad(I) + Soberanía(II: `owner_id`) + Hardware(IV: `node_id` — qué máquina es la titular).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Registro local de respaldos con Grupo I + Perfil D; campos propios fuera del catálogo (marcados): marca de tiempo del último snapshot, hash del blob, tamaño, `node_id` titular vigente — nunca la clave de cifrado ni el secreto maestro. `STRICT`, UUIDv7.

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (#1, a qué cuenta pertenece el blob y la titularidad), el almacén de objetos de la Cabina de Mando (adaptador de red diferido).
- **Comparte flujo con:** `licensing-system` (#2) — el reclamo forzado de titularidad desde el panel central reutiliza el mismo self-service que liberar un cupo de activación de una máquina muerta.
- **Bloquea a:** nada — es la garantía de continuidad de todo el resto del trabajo del usuario.
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** toggle de respaldo + indicador de titularidad en el cajón de ajustes (Inspector de Ajustes).
