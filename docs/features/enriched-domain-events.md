# Enriched Domain Events

> 🟡 **Parcial** 2026-07-05 · Orden de trabajo [STORY-033](../execution/STORY-033-enriched-domain-events.md) · Cimiento local completo: event-store append-only **atómico** `domain_events` (migración `0012`, `event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE, Grupo I + Perfil D), Core `domain/enriched_domain_events.rs` (enum `EnrichedDomainEvent` de 8 variantes con los **3 de ADR-0145** — flujo de capital, snapshot de cuenta, orden reforzada; montos `i64` ×10⁸, serialización canónica `BTreeMap`, `decide_replication`, hash encadenado), Shell append-only atómico (`BEGIN IMMEDIATE` + reintento + `WriteContention`) que consume el `ExecutionGate` **real** de #2 para derivar `replicate`, puerto `event_out`/`gate_in`, CLI `verify enriched-domain-events`. Crate `crates/shared` (excepción bendecida ADR-0137). QA APTO (5 mutaciones). Pendiente: fan-out al bus (ADR-0085), envío a la Cabina de Mando (adaptador de red diferido) y mapeo de las acciones reales de `execute` (EPIC-5).

**Carpeta:** `./features/enriched-domain-events/`
**Estado:** 🟡 Parcial (event-store local completo; bus, envío al proveedor y mapeo de `execute` diferidos)
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (cimiento #6) · ADR-0085 (bus) · ADR-0027 (event sourcing) · ADR-0143 (supresión por tier) · **ADR-0145 (enriquecimiento para Cuentas Verificadas: flujo de capital + snapshot de cuenta + refuerzo de orden)**

## ¿Qué es esta feature?

La instrumentación temprana: tipos de evento **inmutables y ricos** que el motor emite al bus existente (ADR-0085) por cada acción significativa, con los datos que los productos de monetización futuros van a necesitar. Es la **raíz** del substrato (SAD-22 §22.4): sin eventos estructurados no hay telemetría, ni agregación, ni reportes, ni billing.

- **Problema:** si no se emiten estos eventos desde el día 1, cada producto futuro exige reabrir la capa de ejecución para instrumentarla. Es más barato emitir eventos que nadie consume aún.
- **Comportamiento observable:** cada orden ejecutada, backtest completado, régimen detectado, etc., produce un evento estructurado en el bus y en el audit-log.
- **Por qué:** el leverage arquitectónico entero depende de esta captura.

## Comportamientos Observables

- Cuando se ejecuta una orden → emite un evento con instrumento, lado, cantidad, precio, slippage, tiempo de fill, bróker, nocional y —refuerzo ADR-0145— `account_id`, PnL realizado, MAE, MFE y duración del trade (para derivar % de trades rentables, tiempo medio de espera y días de trading por cuenta).
- **Cuando hay un movimiento de capital (depósito / retiro / transferencia) —ADR-0145—** → emite un evento de flujo de capital con signo, monto (entero ×10⁸), divisa, cuenta y timestamp. Es imprescindible para calcular el gain% separado del capital aportado (el crecimiento excluye depósitos).
- **Cuando cambia el estado de una cuenta de bróker —ADR-0145—** → emite un snapshot con equity, balance y margen disponible/requerido por cuenta (cadencia por-fill o periódica). Alimenta las curvas de equidad y balance del track record.
- Cuando termina un backtest → emite un evento con las métricas completas (Sharpe, drawdown, PBO, régimen).
- Cuando se detecta un régimen, un drawdown, estrés de liquidez o un cambio de correlación → emite su evento respectivo.
- Cuando la licencia ordena supresión (tier de pago, ADR-0143) → el emisor hacia la Cabina de Mando se apaga en origen, pero el evento sigue disponible **localmente** para el propio usuario.

## Restricciones

- NUNCA un evento enriquecido incluye secretos (credenciales de bróker, IPs live) — ADR-0093.
- Los eventos son inmutables (append-only, encadenados con `audit_chain_hash`).
- La supresión por tier afecta el **envío al proveedor**, no la emisión local (el usuario de pago conserva sus propios eventos).

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| EVENT_TYPES_ENABLED | todos | conjunto | Qué tipos de evento se emiten | CONFIG |
| LOCAL_RETENTION | 90 d | 0 – ∞ | Retención local de eventos antes de poda | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** construcción del evento a partir del estado observado (sin I/O), cálculo de su hash encadenado.
- **Shell (Infraestructura):** publicación en el bus (ADR-0085), persistencia en el event-store local, envío a la Cabina de Mando según el gate de supresión.
- **Frontera Pública:** puerto que expone el flujo de eventos enriquecidos; consumido por `usage-metering`, `data-aggregation`, `institutional-report-engine` y la telemetría.

## Ciclo de Vida de la Feature — Enriched Domain Events

### Entrada
El estado observado de una acción del motor (orden, backtest, régimen, drawdown…) y el veredicto de supresión del gate de licencia.

### Proceso
Construye el evento inmutable rico, lo publica en el bus y lo persiste; decide si además lo envía al proveedor.

### Salida
Un evento estructurado en el bus + en el event-store local, opcionalmente replicado a la Cabina de Mando.

## Tareas (TTRs)

- **TTR-001:** Catálogo de tipos de evento enriquecidos y su construcción (Core puro, hash encadenado).
- **TTR-002:** Publicación en el bus (ADR-0085) + persistencia append-only local.
- **TTR-003:** Envío a la Cabina de Mando gobernado por el gate de supresión (ADR-0143).
- **TTR-004 (ADR-0145):** Evento de flujo de capital (depósito/retiro/transferencia; monto entero ×10⁸, con signo, divisa, cuenta).
- **TTR-005 (ADR-0145):** Evento de snapshot de estado de cuenta (equity/balance/margen por cuenta de bróker) + refuerzo de la orden-con-fricción con `account_id`/PnL/MAE/MFE/duración.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `gate_in` | `ExecutionGate` (plomería, ADR-0144) | Input | `1` | Veredicto de supresión de telemetría. |
| `event_out` | `EnrichedDomainEvent` (tipo técnico nuevo — plomería, ADR-0144) | Output | `1..N` | Flujo de eventos ricos; consumido por medición, agregación, reportes y telemetría. |

## Cáscara Visual (Thin Shell)

> Plomería (Ventana de Verificación). El UI-Designer escribe solo la nota de observable. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** emisión y persistencia local; envío al proveedor condicionado por tier.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría)**: Identidad(I) + Soberanía(II: `owner_id`, `institutional_tag`) + Hardware(IV: `node_id`, `process_id`, `session_id`).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Event-store append-only (`event_sequence_id UNIQUE`, `audit_chain_hash` encadenado, NULL en génesis) con Grupo I + Perfil D. Campo propio fuera del catálogo (marcado): tipo de evento (`TEXT` con `CHECK`), payload estructurado (`TEXT` con `json_valid`). El `CHECK` del tipo de evento incluye los subtipos nuevos de ADR-0145 (flujo de capital, snapshot de estado de cuenta) además de los previos. Montos monetarios (flujo de capital, equity/balance/margen del snapshot) como **entero ×10⁸**, nunca `REAL` (ADR-0141). Reutiliza el patrón de `audit-log`/`telemetry` (ADR-0141). `STRICT`, UUIDv7.

**Rastro de Evidencia:** es la fuente primaria de causalidad para el módulo `feedback` y para todos los productos de monetización.

## Dependencias y Bloqueantes

- **Depende de:** bus de eventos (ADR-0085), audit-log (construido), `licensing-system` (gate de supresión).
- **Bloquea a:** `usage-metering`, `data-aggregation`, `institutional-report-engine`, `verified-account-registry` (#10, ADR-0145 — consume flujo de capital + snapshot de cuenta + orden reforzada).
- **Contrato de Integración UI (ADR-0117) — Ventana de Verificación:** su observable (conteo y último timestamp de eventos por tipo) queda visible en el tab de verificación de una feature consumidora (p. ej. el panel de consumo de `usage-metering`); hasta entonces, deuda de integración registrada.
