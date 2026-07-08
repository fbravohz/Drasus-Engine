# Data Portability (Portabilidad y Exportación de Datos)

> 🔴 **En Diseño** 2026-07-07 · Cimiento del substrato aprobado por el propietario, aún sin orden de trabajo. Se materializa el catálogo + registro de solicitudes ahora; el generador de export real y la Cáscara Visual quedan diferidos (ver Dependencias).

**Carpeta:** `./features/data-portability/`
**Estado:** 🔴 En Diseño (cimiento #13, pendiente de Story)
**Última actualización:** 2026-07-07
**Decisión Arquitectónica Asociada:** ADR-0148 (cimiento #13) · ADR-0144 (substrato) · ADR-0020 (Inundación de Fundaciones — `owner_id` universal) · ADR-0093 (secretos jamás salen) · ADR-0141 (append-only, pseudonimización sobre DELETE físico)

## ¿Qué es esta feature?

El puerto que, dado un `owner_id` autenticado, permite (1) **exportar** en formato legible todos los datos que Drasus tiene ligados a esa identidad (derecho de acceso/portabilidad, GDPR Art. 15/20) y (2) **solicitar el olvido** de esos datos, con las excepciones de retención legal que correspondan (derecho de supresión, GDPR Art. 17). No es una feature de dominio de trading: es infraestructura de cumplimiento legal transversal, mismo nivel que `consent-registry` (#5).

- **Problema:** Drasus retiene datos de trabajo del usuario en la Cabina de Mando desde el cimiento #1 (`central-identity`). Sin un catálogo de qué tablas los contienen, la primera solicitud real de un usuario europeo exige auditar manualmente todo el esquema bajo un plazo legal de 30 días.
- **Comportamiento observable:** el usuario pide desde su panel de cuenta "dame mis datos" o "olvídame"; recibe un archivo estructurado o la confirmación de que su identidad fue pseudonimizada (con el detalle de qué se retuvo por ley y por qué).
- **Por qué:** es barato de cimentar ahora porque `owner_id` (Grupo II, ADR-0020) ya es universal en cualquier tabla relevante; el catálogo es, en esencia, un índice de lo que ya existe — y carísimo de reinstrumentar bajo presión regulatoria con plazo legal encima.

## Comportamientos Observables

- Cuando una feature nueva declara una tabla con `owner_id` → se auto-registra en el catálogo de datos exportables (mismo mecanismo de propagación que ADR-0020).
- Cuando el usuario solicita su export → se crea una solicitud append-only con estado `RECIBIDA`; el sistema recorre el catálogo y arma el archivo estructurado (JSON/CSV) con los datos de ese `owner_id` en todas las tablas declaradas.
- Cuando el usuario solicita el olvido → las tablas sin obligación de retención legal se pseudonimizan (se desvincula el `owner_id` identificable); las tablas con retención obligatoria (auditoría financiera, atestaciones de #10, libro de nocional de #4) se pseudonimizan también, pero **conservan el registro** por integridad del ledger — el usuario ve exactamente cuáles y por qué.
- Cuando se audita una solicitud pasada → el registro append-only prueba qué se exportó/pseudonimizó y cuándo.

## Restricciones

- **NUNCA (FIJO)** se exportan secretos: credenciales de bróker, claves de cifrado, IPs live (ADR-0093) — mismo filtro que `instance-continuity` (#11).
- **NUNCA** el export incluye la lógica/IP de terceros con quienes el usuario haya interactuado (ej. estrategias ajenas vistas vía marketplace) — solo los datos propios del `owner_id` solicitante.
- **NUNCA** se sirve un export a un tercero, aunque tenga acceso administrativo, salvo excepción legal documentada y auditada (orden judicial).
- **NUNCA** el olvido borra físicamente (DELETE) un registro cuya integridad referencial protege a otros — se pseudonimiza, nunca se elimina la fila (mismo criterio de ADR-0141/#12).
- Este catálogo **NO sustituye** al backup cifrado de `instance-continuity` (#11): ese es un blob opaco para disaster recovery, este es un export legible para ejercer un derecho legal — propósitos distintos, nunca se confunden.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| EXPORT_FORMAT | JSON | JSON / CSV / ambos | Formato del archivo de export | CONFIG |
| EXPORT_SLA_DAYS | 30 | 1 – 30 días | Plazo máximo para completar una solicitud (alineado a GDPR Art. 12) | CONFIG |
| RETENTION_EXEMPT_TABLES | (catálogo) | conjunto | Tablas con obligación de retención legal que se pseudonimizan en vez de purgarse en un olvido | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** resolución de qué tablas del catálogo aplican a un `owner_id`, serialización determinista del export, decisión de pseudonimizar vs. purgar por tabla (según si está en `RETENTION_EXEMPT_TABLES`).
- **Shell (Infraestructura):** recorrido real del esquema (adaptador, diferido), generación del archivo, persistencia del registro de solicitudes.
- **Frontera Pública:** puerto "exporta mis datos" + puerto "olvídame" — ambos consumidos desde el panel de cuenta de la Cabina de Mando.

## Ciclo de Vida de la Feature — Data Portability

### Entrada
`owner_id` autenticado y el tipo de solicitud (export u olvido).

### Proceso
Recorre el catálogo de tablas declaradas con `owner_id`, arma el export o aplica pseudonimización según la tabla, registra la solicitud append-only.

### Salida
Un archivo estructurado descargable, o la confirmación de olvido con el detalle de qué se retuvo por ley.

## Tareas (TTRs)

- **TTR-001:** Catálogo declarativo de tablas exportables — cada feature nueva con `owner_id` se auto-registra.
- **TTR-002:** Registro append-only de solicitudes de exportación/olvido con estado (Core puro para la resolución, Shell para la persistencia).
- **TTR-003 (diferido):** Generador real del archivo de export contra el esquema completo — adaptador, se construye ante la primera solicitud real o el lanzamiento en jurisdicción GDPR.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `identity_in` | `AccountIdentity` (plomería, ADR-0144) | Input | `1` | Identidad de cuenta cuyo dato se exporta u olvida, producida por `central-identity`. |
| `export_request_out` | Solicitud de export/olvido (tipo técnico nuevo — plomería, ADR-0148) | Output | `0..1` | Registro append-only de la solicitud con su estado. |
| `data_catalog_out` | Catálogo de tablas exportables (tipo técnico nuevo — plomería, ADR-0148) | Output | `1..N` | Lista declarativa de qué features/tablas tienen `owner_id`, poblada incrementalmente por cada feature nueva. |

> Tipos técnicos nuevos del cimiento #13, registrados en el catálogo de ADR-0137 vía la enmienda de ADR-0148. Nombres canónicos de `struct` los fija el ingeniero.

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: botón "exportar mis datos" / "olvidarme" en el panel de cuenta del cajón de ajustes. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016), re-escopado por ADR-0143:** el catálogo y las solicitudes viven en la Cabina de Mando (es donde está el dato a exportar/olvidar); el motor local no es fuente de verdad de esta feature.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría/Cumplimiento)**: Identidad(I) + Soberanía(II: `owner_id`) + subset V (`compliance_status_id` del estado de la solicitud).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Dos piezas: (1) catálogo declarativo de tablas exportables — no es una tabla de negocio, es metadato de esquema (puede materializarse como tabla ancla análoga a `foundation_master_fields`, ADR-0020). (2) registro **append-only** de solicitudes (`event_sequence_id UNIQUE`) con Grupo I + Perfil D; campos propios fuera del catálogo (marcados): tipo de solicitud (export/olvido), estado, detalle de tablas pseudonimizadas vs. purgadas. `STRICT`, UUIDv7, `audit_chain_hash` encadenado (ADR-0141).

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (#1, `owner_id`), `consent-registry` (#5, forma parte de lo exportable — qué consintió y cuándo).
- **Distinto de (no confundir):** `instance-continuity` (#11, blob opaco de disaster recovery ≠ export legible de derecho legal), `data-aggregation` (#9, agregado anónimo saliente hacia terceros ≠ dato crudo propio saliente hacia el mismo dueño). El **derecho de rectificación** (GDPR Art. 16, corregir un dato inexacto) NO vive aquí: para datos de perfil mutables se resuelve en `central-identity` (#1, sobreescritura directa); para registros append-only (#4/#10) una corrección es una fila nueva que enmienda, nunca una edición retroactiva — este cimiento (#13) solo cubre acceso/portabilidad/olvido (Art. 15/17/20), no rectificación.
- **Bloquea a:** nada operativamente hoy; bloquea el **lanzamiento en jurisdicción GDPR** si no existe antes de tener usuarios de pago reales en la UE.
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** panel de cuenta, sección "mis datos" con export y olvido, en la Cabina de Mando (no en el monolito local).
