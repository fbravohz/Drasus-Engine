# Verified Account Registry (Cuentas Verificadas Drasus)

**Carpeta:** `./features/verified-account-registry/`
**Estado:** En Diseño
**Última actualización:** 2026-07-04
**Decisión Arquitectónica Asociada:** ADR-0145 (pilar de Cuentas Verificadas, cimiento #10) · ADR-0143 (tres planos + telemetría clase 5) · ADR-0093 (secretos nunca salen) · ADR-0141 (modelado) · ADR-0137 (puertos)

## ¿Qué es esta feature?

El registro que unifica bajo una identidad Drasus (`central-identity`, #1) las **N cuentas de trading** del usuario (fondeo, prop, capital propio con ICMarkets/Binance/IBKR), cada una con su **track record verificado** y su **ámbito de atestación**. Es el cimiento del pilar análogo a myFXbook / MT5 Signals, con el diferenciador soberano: Drasus atestigua criptográficamente lo que **su propio motor ejecutó** (cadena de hash + append-only), no solo lo que el bróker reporta. Ahora se entrega el **puerto + el esquema**; el portal público y su render son un repo aparte, diferido (ADR-0145).

- **Problema:** myFXbook/MT5 confían en la conexión read-only al bróker. Drasus puede además probar, con la cadena de hash inmutable ya construida, que **fue su motor** quien ejecutó y que el track no se alteró. Sin capturar los eventos correctos desde el cimiento #6, no se puede reconstruir esa pista después.
- **Comportamiento observable:** el usuario ve todas sus cuentas bajo una identidad; cada una con su curva de equidad, gain%, drawdown y estadística; puede publicar (opt-in) las que elija.
- **Por qué:** es prueba social verificada — un motor de distribución que ni SQX ni el resto del mercado ofrecen con atestación soberana.

## Comportamientos Observables

- Cuando el usuario registra una cuenta de trading → queda vinculada a su `owner_id` con bróker, apalancamiento, divisa y tipo (fondeo/prop/propio).
- Cuando una operación se ejecuta por el motor Drasus → su track queda **atestado soberanamente** (cadena de hash del audit-log); se marca "Ejecución Verificada por Drasus".
- Cuando el usuario conecta una cuenta read-only al bróker (investor password/API) → el motor **local** computa la estadística de cuenta-completa y la marca "Reportado por el Bróker"; la credencial **nunca** sube a la Cabina de Mando (ADR-0093).
- Cuando el usuario opta por publicar una cuenta → su track record (curvas, gain%, drawdown, estadística) se emite al servidor central para el portal; sin opt-in, permanece privado.
- Cuando se recalcula el track record → el gain% excluye depósitos/retiros (usa los eventos de flujo de capital de #6), reproduciendo la métrica de crecimiento estilo myFXbook/MT5.

## Restricciones

- NUNCA se presenta un dato "reportado por el bróker" como "verificado por Drasus": el ámbito de atestación es inviolable y visible.
- NUNCA vive una credencial de bróker ni una investor password en este registro: se referencia la cuenta por un identificador **no secreto**; los secretos siguen en `broker_connections` (cifrados, locales, ADR-0093).
- NUNCA se publica sin consentimiento vigente por cuenta (`consent-registry`, #5). Publicar es opt-in; el default es privado.
- Publicar **resultados/estadísticas** no expone el **trabajo/PI**: el track record no incluye la lógica (estrategia, AST, parámetros) que lo generó. Es telemetría clase 5 (ADR-0143 enmendado), independiente del tier.
- El track record publicable es **identificable por diseño**; NO pasa por la anonimización de `data-aggregation` (#9), que sirve al canal anónimo hacia terceros.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PUBLICATION_DEFAULT | privado | privado/público | Estado de publicación al registrar una cuenta | FIJO (privado) |
| ATTESTATION_SCOPES | soberana, read-only | conjunto | Ámbitos de atestación soportados por cuenta | CONFIG |
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
- **Inundación de Fundaciones (ADR-0020 V2):** Grupo I completo + **Perfil D (Ops/Auditoría/Forense)**: Identidad(I) + Soberanía(II: `owner_id`, `institutional_tag`) + Hardware(IV: `node_id`) + subset V (`signature_hash` del track atestado).

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Dos tablas: (1) registro de cuentas —tabla **mutable** (`row_version`, no `event_sequence_id`)— con Grupo I + Perfil D; campos propios fuera del catálogo (marcados): bróker/venue, apalancamiento, divisa, tipo de cuenta, estado de publicación, ámbito(s) de atestación, referencia **no secreta** a la conexión de bróker. (2) track record atestado con Grupo I + Perfil D; campos propios (marcados): tipo/ventana, `signature_hash`, ámbito, referencia a la cuenta. Montos monetarios como **entero ×10⁸** (ADR-0141), nunca `REAL`. `STRICT`, UUIDv7. Multi-tenancy real solo en la Cabina de Mando: se reutiliza `owner_id`, prohibido calcar `tenant_id` (ADR-0144).

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (#1, `owner_id`), `enriched-domain-events` (#6, flujo de capital + snapshot + orden reforzada), `consent-registry` (#5, opt-in de publicación), `broker-connector` (conexión read-only, en el Plano de Ejecución), audit-log (cadena de hash de la atestación soberana).
- **Bloquea a:** el portal público de Cuentas Verificadas (repo aparte, futuro) y su contrato de reporte.
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** panel de cuentas verificadas. SVF: tras ejecutar operaciones reales, el panel muestra el track record calculado por el Core (curvas, gain% sin depósitos) con su ámbito de atestación; al activar la publicación, el registro de consentimiento se dispara; tras recargar, el estado persiste.
