# Respaldo Cifrado de la DB Local (Encrypted Local Backup)

> ⚠️ **Superado por ADR-0146 (2026-07-06) — promovido de moonshot a cimiento #11.** Ya no es un adaptador diferido sin fecha: se construye su contrato/esquema ahora, en la misma pasada de fundaciones que el resto del substrato de monetización, con un mecanismo adicional (maestro itinerante / relevo de custodia). La especificación canónica vive en [`instance-continuity`](../features/instance-continuity.md). Este archivo queda como registro histórico de la idea original; no se actualiza más.

**Carpeta:** `./moonshots/encrypted-local-backup/`
**Estado:** Archivada como Moonshot (adaptador de durabilidad — puerto/contrato listo, sin implementación)
**Última actualización:** 2026-07-05
**Decisión Arquitectónica Asociada:** ADR-0143 (Tres Planos + Soberanía por Tier) · ADR-0093 (los secretos jamás salen) · ADR-0141 (persistencia)

---

> 🔗 **Nota cruzada:** NO confundir con el *Cold Storage / S3 Sync* de [`saas-cloud-engine`](saas-cloud-engine.md) ni con la DR por nodo de [`distributed-edge-execution`](distributed-edge-execution.md). Aquellos replican el WAL del **motor headless** (VPS del usuario) para recuperación de nodo. Este moonshot es el **respaldo de la base de datos soberana del escritorio del usuario individual**, cifrado en el cliente de modo que el proveedor **no puede leerlo** — es un perk del tier de pago, no infraestructura del clúster.

## ¿Qué es esta feature?

Un respaldo opcional de la base de datos local del usuario (estrategias, backtests, portafolios) hacia almacenamiento de objetos del proveedor (S3 / Cloudflare R2), **cifrado del lado del cliente** antes de salir de la máquina. El proveedor almacena un blob opaco que jamás puede descifrar.

- **Problema:** un disco muerto o un robo de laptop borra meses de trabajo del usuario. El tier gratuito ya vive en el firehose del proveedor (recuperable), pero el usuario de pago suprime esa telemetría en origen (ADR-0143) — su trabajo NO está en los servidores del proveedor, así que un fallo de hardware sí lo perdería. Este respaldo cierra ese hueco **sin romper la promesa de soberanía**.
- **Comportamiento observable:** el usuario activa "respaldo en la nube"; la app sube un blob cifrado periódicamente; ante un equipo nuevo, restaura desde el blob con su secreto maestro.
- **Por qué:** convierte la garantía de soberanía en un producto vendible ("respaldamos tu trabajo y ni siquiera nosotros podemos verlo"), coherente con el tier de pago en vez de contradecirlo.

## Comportamientos Observables

- Cuando el usuario activa el respaldo → la app cifra la DB local con una clave derivada de su secreto maestro y sube el blob.
- Cuando el proveedor recibe el blob → almacena bytes opacos (sin clave, sin capacidad de descifrar).
- Cuando el usuario restaura en una máquina nueva → introduce su secreto maestro, descarga el blob y lo descifra localmente.
- Cuando el usuario pierde el secreto maestro → el respaldo es irrecuperable **por diseño** (el proveedor no tiene puerta trasera). Se advierte explícitamente en la UI.

## Restricciones

- **NUNCA (FIJO)** la clave de cifrado ni el secreto maestro salen de la máquina del usuario (ADR-0093). El cifrado es client-side; el servidor solo ve bytes opacos.
- **NUNCA** se incluyen en el blob los secretos de bróker ni las IPs de servidores live (ADR-0093) — se excluyen del respaldo igual que se excluyen de la telemetría.
- El proveedor **no puede** ofrecer recuperación de contraseña del blob (no tiene la clave): la pérdida del secreto maestro implica pérdida del respaldo, y así se comunica.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| BACKUP_ENABLED | false | true/false | Activa el respaldo cifrado (opt-in explícito). | CONFIG |
| BACKUP_INTERVAL | 24 h | 1 h – 30 d | Cadencia de subida del blob incremental. | CONFIG |
| CLIENT_SIDE_ENCRYPTION | AES-256-GCM | (fijo) | Algoritmo de cifrado autenticado del blob. | FIJO |
| BACKUP_RETENTION | 30 días | 1 – 365 días | Cuántas versiones del blob se conservan en el objeto remoto. | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** derivación de clave desde el secreto maestro (KDF estándar), cifrado/descifrado AES-256-GCM autenticado, cálculo del delta a respaldar.
- **Shell (Infraestructura):** cliente de almacenamiento de objetos (S3/R2), programador de la cadencia, lectura consistente de la DB local (snapshot).
- **Frontera Pública:** puerto "respalda/restaura este entorno" — el blob cifrado es el único artefacto que cruza a la Cabina de Mando.

## Gobernanza y Estándares (Fijos)

- **Soberanía por Tier (ADR-0143):** este respaldo es coherente con el tier de pago: el proveedor almacena pero no lee. En el tier gratuito el trabajo ya fluye en claro por el firehose (dueño por ToS), así que el respaldo cifrado es un perk especialmente valioso para el tier de pago.
- **Secretos (ADR-0093):** credenciales de bróker e IPs live jamás entran al blob.
- **Inundación de Fundaciones (ADR-0020):** Perfil D si se persiste un registro local de respaldos (marca de tiempo, hash del blob, tamaño) — nunca la clave.

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (a qué cuenta pertenece el blob), el almacén de objetos de la Cabina de Mando (adaptador de red diferido con el servidor central).
- **Por qué es moonshot y no cimiento:** no necesita un puerto local vivo AHORA (a diferencia del metering); es un adaptador de durabilidad que se activa cuando exista la Cabina de Mando y haya demanda. El contrato queda listo para responder "sí, lo tenemos" sin 6 meses de diseño.
