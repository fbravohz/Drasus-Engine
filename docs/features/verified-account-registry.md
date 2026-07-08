# Verified Account Registry (Cuentas Verificadas Drasus)

> 🟡 **Parcial** 2026-07-06 · Orden de trabajo [STORY-037](../execution/STORY-037-verified-account-registry.md) · Cimiento local completo: dos tablas (migración `0016`) — `verified_accounts` **MUTABLE** (`row_version`→`VersionConflict`, `broker_connection_ref` NO secreto ADR-0093, `account_type`/`publication_status` CHECK, `attestation_scopes` json_valid) y `attested_track_records` **APPEND-ONLY atómica** (`event_sequence_id UNIQUE` + triggers, `BEGIN IMMEDIATE`+reintento+`WriteContention`, subset V `signature_hash NOT NULL`). Core `domain/verified_account_registry.rs` (`compute_track_record` con **gain% que EXCLUYE el flujo de capital** — el diferenciador ADR-0145; Eje A ámbitos `SOVEREIGN`/`BROKER_READONLY` inviolables vía `is_sovereign_attestation`, **cruzado con Eje B `CapitalReality` `LIVE`/`PAPER`/`DEMO`/`CHALLENGE` (`is_real_capital`, ortogonal — STORY-038/DEBT-014)**; `compute_track_record_signature` reproducible; montos `i64` ×10⁸). Orquestador (`register_account`, `attest_track_record`, `request_publication` con `consent_out` **REAL** de #5, default PRIVATE). Guardarraíl estructural: secretos nunca en el registro. Puertos `event_in`/`consent_in`/`registry_out`/`track_record_out`, CLI `verify verified-account-registry`. Crate `crates/shared`. **QA APTO** (mutación: 84/118 cazados + `BEGIN IMMEDIATE` manual; huecos no bloqueantes → DEBT-013). Pendiente: portal público (repo aparte), contrato de reporte al servidor central, conexión read-only real al bróker, panel de UI (Superficie propia, DEBT-005).
>
> ✅ **Corrección de ADR-0145 (2026-07-06) RESUELTA por [STORY-038](../execution/STORY-038-verified-account-capital-reality.md) — el eje de realidad de capital ya está modelado.** El código ahora modela **ambos ejes ortogonales**: **Eje A** (`SOVEREIGN`/`BROKER_READONLY` — quién ejecutó, `is_sovereign_attestation`) × **Eje B** (`LIVE`/`PAPER`/`DEMO`/`CHALLENGE` — realidad del capital, enum `CapitalReality` con `is_real_capital`). Una cuenta en `PAPER`/`DEMO`/`CHALLENGE` corre en el mismo entorno determinista que `LIVE` (NO backtesting) y **sí es atestiguable** con capital virtual — nunca se presenta sin esa etiqueta. Entregado: el Eje B vive en `verified_accounts` y se estampa en cada `attested_track_records` (columna + CHECK); firma reproducible y ambos `audit_hash` lo encadenan; la proyección de puerto expone `capital_reality` + `is_real_capital` (derivado SOLO del Eje B) junto al Eje A, siempre; test discriminante `SOVEREIGN`+`PAPER` (atestado pero capital virtual) en 3 capas. Auditoría TL aprobada + QA APTO por mutación. **DEBT-014 pagada.**
>
> ✅ **Retrabajo de Inundación de Fundaciones RESUELTO por [STORY-041](../execution/STORY-041-verified-account-eje-b-consolidation.md) (2026-07-07) — DEBT-016 pagada.** La columna duplicada `capital_reality` se **eliminó** de ambas tablas (migración `0016` editada in-situ, greenfield); el Eje B ahora vive en `institutional_tag` (Grupo II, ADR-0020) con `CHECK (institutional_tag IN ('LIVE','PAPER','DEMO','CHALLENGE'))` en ambas tablas — una sola columna, reutilización del campo canónico. El tipo de dominio `CapitalReality` se conservó como **intérprete** de `institutional_tag` (mantiene `is_real_capital`, `true` solo para `LIVE`); validación fail-typed `UnknownInstitutionalTag` (sin default silencioso) + dos tests-guardarraíl (`pragma_table_info` verifica que `capital_reality` NO existe y que `institutional_tag` porta el CHECK del Eje B). Cero cambio de comportamiento observable: gain% sigue excluyendo el flujo de capital, ambos ejes se exponen siempre juntos, discriminante `SOVEREIGN`+`PAPER` intacto. Auditoría TL aprobada + QA APTO por mutación (80/92; 6 sobrevivientes = bordes de DEBT-013, ninguno nuevo — cobertura preservada).

**Carpeta:** `./features/verified-account-registry/`
**Estado:** 🟡 Parcial (registro + track record + firma + gate de publicación local completos; portal, contrato de red y read-only del bróker diferidos)
**Última actualización:** 2026-07-06
**Decisión Arquitectónica Asociada:** ADR-0145 (pilar de Cuentas Verificadas, cimiento #10) · ADR-0143 (tres planos + telemetría clase 5) · ADR-0093 (secretos nunca salen) · ADR-0141 (modelado) · ADR-0137 (puertos)

## ¿Qué es esta feature?

El registro que unifica bajo una identidad Drasus (`central-identity`, #1) las **N cuentas de trading** del usuario (fondeo, prop, capital propio con ICMarkets/Binance/IBKR), cada una con su **track record verificado** y su **ámbito de atestación**. Es el cimiento del pilar análogo a myFXbook / MT5 Signals, con el diferenciador soberano: Drasus atestigua criptográficamente lo que **su propio motor ejecutó** (cadena de hash + append-only), no solo lo que el bróker reporta. Ahora se entrega el **puerto + el esquema**; el portal público y su render son un repo aparte, diferido (ADR-0145).

- **Problema:** myFXbook/MT5 confían en la conexión read-only al bróker. Drasus puede además probar, con la cadena de hash inmutable ya construida, que **fue su motor** quien ejecutó y que el track no se alteró. Sin capturar los eventos correctos desde el cimiento #6, no se puede reconstruir esa pista después.
- **Comportamiento observable:** el usuario ve todas sus cuentas bajo una identidad; cada una con su curva de equidad, gain%, drawdown y estadística; puede publicar (opt-in) las que elija.
- **Por qué:** es prueba social verificada — un motor de distribución que ni SQX ni el resto del mercado ofrecen con atestación soberana.

## Comportamientos Observables

- Cuando el usuario registra una cuenta de trading → queda vinculada a su `owner_id` con bróker, apalancamiento, divisa y tipo (fondeo/prop/propio).
- Cuando una operación se ejecuta por el motor Drasus → su track queda **atestado soberanamente** (cadena de hash del audit-log); se marca "Ejecución Verificada por Drasus" **junto con** la etiqueta de realidad de capital ("LIVE" o "PAPER/DEMO/CHALLENGE") — nunca una sola de las dos. Una cuenta `PAPER`/`CHALLENGE` corre en el mismo entorno determinista que `LIVE` (no es backtesting) y es igualmente atestiguable; solo cambia el rótulo de capital.
- Cuando el usuario conecta una cuenta read-only al bróker (investor password/API) → el motor **local** computa la estadística de cuenta-completa y la marca "Reportado por el Bróker"; la credencial **nunca** sube a la Cabina de Mando (ADR-0093).
- Cuando el usuario opta por publicar una cuenta → su track record (curvas, gain%, drawdown, estadística) se emite al servidor central para el portal; sin opt-in, permanece privado.
- Cuando se recalcula el track record → el gain% excluye depósitos/retiros (usa los eventos de flujo de capital de #6), reproduciendo la métrica de crecimiento estilo myFXbook/MT5.

## Restricciones

- NUNCA se presenta un dato "reportado por el bróker" como "verificado por Drasus": el ámbito de atestación (Eje A) es inviolable y visible.
- NUNCA se presenta un track sin su etiqueta de realidad de capital (Eje B: `LIVE` vs `PAPER`/`DEMO`/`CHALLENGE`) — omitirla es tan grave como confundir los valores del Eje A. Un track `SOVEREIGN`+`PAPER` es válido y atestiguable; nunca se muestra como si fuera `LIVE`.
- NUNCA vive una credencial de bróker ni una investor password en este registro: se referencia la cuenta por un identificador **no secreto**; los secretos siguen en `broker_connections` (cifrados, locales, ADR-0093).
- NUNCA se publica sin consentimiento vigente por cuenta (`consent-registry`, #5). Publicar es opt-in; el default es privado.
- Publicar **resultados/estadísticas** no expone el **trabajo/PI**: el track record no incluye la lógica (estrategia, AST, parámetros) que lo generó. Es telemetría clase 5 (ADR-0143 enmendado), independiente del tier.
- El track record publicable es **identificable por diseño**; NO pasa por la anonimización de `data-aggregation` (#9), que sirve al canal anónimo hacia terceros.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PUBLICATION_DEFAULT | privado | privado/público | Estado de publicación al registrar una cuenta | FIJO (privado) |
| ATTESTATION_SCOPES | soberana, read-only | conjunto | Eje A — ámbitos de atestación soportados por cuenta (quién ejecutó) | CONFIG |
| CAPITAL_MODES | LIVE, PAPER, DEMO, CHALLENGE | conjunto | Eje B — realidad del capital soportada por cuenta (qué arriesgó); ortogonal al Eje A, ambos ejes siempre visibles juntos | CONFIG |
| SNAPSHOT_CADENCE | por-fill | por-fill / periódica | Frecuencia del snapshot de estado de cuenta para las curvas | CONFIG |
| TRACK_RECORD_REFRESH | por-evento | rango | Cada cuánto se recalcula el track record publicado | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** cálculo del track record a partir de los eventos (curvas equity/balance, drawdown máximo, gain% excluyendo flujo de capital, % de trades rentables, tiempo medio de espera, días de trading), separación por ámbito de atestación, cálculo de la firma de integridad del track atestado.
- **Shell (Infraestructura):** persistencia del registro y del track, lectura del flujo de eventos enriquecidos (#6), verificación de consentimiento (#5), conexión read-only al bróker (en el Plano de Ejecución del usuario), emisión del track publicado hacia el servidor central.
- **Frontera Pública:** puerto que expone el registro multi-cuenta y el track record por cuenta con su ámbito y estado de publicación; consumido por el portal (repo aparte, futuro) vía el contrato de reporte.

## Ciclo de Vida de la Feature — Verified Account Registry

### Entrada
El flujo de eventos enriquecidos (#6: orden reforzada, flujo de capital, snapshot de cuenta), el veredicto de consentimiento por cuenta (#5), y —para el ámbito read-only— la conexión al bróker en el Plano de Ejecución.

### Proceso
Agrupa los eventos por cuenta bajo el `owner_id`, calcula el track record por ámbito de atestación, firma el track soberano, y —si hay opt-in— lo prepara para publicación.

### Salida
Un registro multi-cuenta y, por cuenta, un track record verificado con su ámbito (soberano y/o read-only) y su estado de publicación, listo para el portal.

## Tareas (TTRs)

- **TTR-001:** Registro multi-cuenta bajo `owner_id` (cuenta de bróker, apalancamiento, divisa, tipo, ámbitos de atestación).
- **TTR-002:** Cálculo del track record atestado soberanamente (Core: curvas, gain% sin depósitos, drawdown, estadística) + firma de integridad.
- **TTR-003:** Ámbito read-only del bróker computado en el Plano de Ejecución del usuario (sin exfiltrar credenciales).
- **TTR-004:** Opt-in de publicación por cuenta (`consent-registry`) + contrato/puerto de reporte del track hacia el servidor central.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `event_in` | `EnrichedDomainEvent` (plomería, ADR-0144/0145) | Input | `0..N` | Órdenes reforzadas, flujo de capital y snapshots de cuenta. |
| `consent_in` | `ConsentVerdict` (plomería, ADR-0144) | Input | `1..N` | Cobertura de consentimiento de publicación por cuenta. |
| `registry_out` | Registro de cuenta verificada (tipo técnico nuevo — plomería, ADR-0145) | Output | `1..N` | Cuentas del usuario con su ámbito de atestación y estado de publicación. |
| `track_record_out` | Track record atestado (tipo técnico nuevo — plomería, ADR-0145) | Output | `1..N` | Pista auditada por cuenta (curvas, gain%, drawdown, estadística) + firma. Consumido por el contrato de reporte hacia el portal. |

> Tipos técnicos nuevos del pilar (no de dominio del canvas), registrados en el catálogo de ADR-0137 vía la enmienda 2026-07-04. Nombres canónicos de `struct` los fija el ingeniero.

## Cáscara Visual (Thin Shell)

> Pendiente Etapa 0.5 (UI-Designer). Superficie prevista en el monolito: panel de cuentas verificadas (lista de cuentas + track por cuenta + toggle de publicación). El **portal público** es un repo aparte, fuera de esta feature. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** el track record se calcula donde están los datos (Plano de Ejecución); solo el resultado publicado (opt-in) viaja al servidor central. La conexión read-only y sus credenciales nunca salen del Plano de Ejecución.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría/Forense)**: Identidad(I) + Soberanía(II: `owner_id`, `institutional_tag`) + Hardware(IV: `node_id`) + subset V (`signature_hash` del track atestado).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Dos tablas: (1) registro de cuentas —tabla **mutable** (`row_version`, no `event_sequence_id`)— con Grupo I + Perfil D; campos propios fuera del catálogo (marcados): bróker/venue, apalancamiento, divisa, tipo de cuenta, estado de publicación, ámbito(s) de atestación (Eje A), referencia **no secreta** a la conexión de bróker. (2) track record atestado con Grupo I + Perfil D; campos propios (marcados): tipo/ventana, `signature_hash`, ámbito (Eje A), referencia a la cuenta. **Realidad de capital `LIVE`/`PAPER`/`DEMO`/`CHALLENGE` (Eje B) = `institutional_tag` (Grupo II, ya heredado por el Perfil D en ambas tablas) — NO un campo propio nuevo** (corrección ADR-0145, auditoría de Inundación de Fundaciones 2026-07-07). El código ya escrito (STORY-038) lo implementó como columna `capital_reality` duplicada de `institutional_tag`; retrabajo pendiente: consolidar en una sola columna. Montos monetarios como **entero ×10⁸** (ADR-0141), nunca `REAL`. `STRICT`, UUIDv7. Multi-tenancy real solo en la Cabina de Mando: se reutiliza `owner_id`, prohibido calcar `tenant_id` (ADR-0144).

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (#1, `owner_id`), `enriched-domain-events` (#6, flujo de capital + snapshot + orden reforzada), `consent-registry` (#5, opt-in de publicación), `broker-connector` (conexión read-only, en el Plano de Ejecución), audit-log (cadena de hash de la atestación soberana).
- **Bloquea a:** el portal público de Cuentas Verificadas (repo aparte, futuro) y su contrato de reporte.
- **Consumido por (garantía de integridad):** [`master-account-hierarchy`](master-account-hierarchy.md) (#12) — la doble atestación de sus comandos de override (fondo↔hija) se apoya en el mismo patrón de cadena de hash inmutable que este registro ya establece; un override sin encadenar en ambos extremos rompería esta garantía.
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** panel de cuentas verificadas. SVF: tras ejecutar operaciones reales, el panel muestra el track record calculado por el Core (curvas, gain% sin depósitos) con su ámbito de atestación; al activar la publicación, el registro de consentimiento se dispara; tras recargar, el estado persiste.
