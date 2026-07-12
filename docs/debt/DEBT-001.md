# DEBT-001 · Ledgers append-only sin transacción atómica ni reintento
- **Severidad:** 🟠 Media
- **Origen:** observación de QA en STORY-030 (`usage-metering`); patrón preexistente desde `audit_log` (EPIC-0).
- **Descripción:** los ledgers append-only asignan `event_sequence_id` con `SELECT MAX(...)+1` e `INSERT` en **sentencias separadas**, sin envolverlas en una transacción `BEGIN IMMEDIATE`. Bajo escritura concurrente, dos escritores pueden derivar el mismo `event_sequence_id`; el `UNIQUE` rechaza a uno (fallo seguro) pero **el evento perdedor se pierde** si no hay reintento.
- **Impacto actual:** nulo — SQLite serializa escritores a nivel de archivo, el motor es local/monoproceso, y los tests corren monohilo sobre `:memory:` (nunca ejercen concurrencia). Se vuelve real con jobs concurrentes (`async-job-executor`, ejecución de varias estrategias).
- **Causa raíz (instrucciones):** el skill `rust-engineer` exigía los invariantes de tamper-evidence (UNIQUE, triggers, hash chain) pero **no** exigía (a) atomicidad transaccional en *read-then-write*, ni (b) prueba de 2 escritores. Vacío de plantilla, no descuido del agente.
- **Disparador de pago / plan:**
  1. Regla permanente en skills `rust-engineer` + `qa-engineer` (transacción `BEGIN IMMEDIATE` + `busy_timeout` + reintento acotado; prueba de 2 escritores obligatoria en todo ledger). → **hecho 2026-07-04**.
  2. `consent-registry` (#5) nace correcto (arreglado en STORY-031 antes de cerrar).
  3. **STORY-032 de endurecimiento** para los ledgers ya commiteados (`audit_log` #0002, `usage_records` #0010), con su propio QA. Recomendado: entre #5 y #6.
- **Estado:** ✅ **Pagada** — [STORY-032](../execution/STORY-032-ledger-atomicity-hardening.md) (2026-07-05). Los 3 puntos del plan completados: regla permanente en skills (2026-07-04), `consent-registry` (#5) nació correcto (STORY-031), y `audit_events`+`usage_records` endurecidos con append atómico (`BEGIN IMMEDIATE` + reintento + `WriteContention`), QA APTO por mutación (quitar la transacción tumba las pruebas de concurrencia).
