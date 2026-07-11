---
name: auditoria-retroactiva-epic0
description: La auditoría retroactiva EPIC-0/STORY-001 contrasta TODOS los ADRs uno por uno (no solo ADR-0141); registra por cada uno si se cumplió/violó/ignoró/no-implementó.
metadata: 
  node_type: memory
  type: project
  originSessionId: 121318b9-21ba-4d1e-a76b-a3648b955c7b
---

> ✅ **EJECUTADA 2026-07-10.** Diagnóstico (6 lotes Sonnet por área) + remediación de código cerrada (QA APTO). Plan `.agents/plans/magical-sprouting-quasar.md`; Orden `docs/execution/STORY-045-foundation-audit-remediation.md`. **Veredicto: substrato arquitectónicamente sano, sin deriva sistémica.** Hallazgos concentrados en el **esquema base EPIC-0** (pre-ADR-0141): `foreign_keys=ON` estaba inactivo (única FK real inerte), faltaba STRICT/UUIDv7/triggers en el baseline, y 3 ledgers append-only de la plomería sin atomicidad. Corregido en STORY-045/046 (greenfield in-situ). **DEBT-018 saldada** por STORY-047 (retrofit de mutación 0 `missed` en #4–#12). Abierta **DEBT-019** (cobertura de mutación de la plomería EPIC-0, no bloqueante). Lo NO-código (desincronizaciones ADR/SAD/CLAUDE.md, FK física `owner_id`, proptest vs enumeración, reconciliación de Canvas — cuya infra genérica YA existe) → **TASK-049**, paquete de escalamiento al Architect pendiente de que el usuario lo invoque. **PENDIENTE: commits de la remediación (autorización del usuario).** El texto siguiente conserva el criterio metodológico con que se ejecutó.

La auditoría retroactiva desde EPIC-0 / STORY-001 tuvo alcance **AMPLIADO** por decisión del usuario (2026-07-03): **no se contrastó solo ADR-0141** (Modelado Relacional Soberano) sino también **cada feature construida vs. su spec funcional** (regla del usuario 2026-07-10). Se recorre el **catálogo completo de ADRs** (índice `docs/ADR.md`) y, **ADR por ADR + feature por feature**, se verifica en el código YA construido si cada decisión **se cumplió, se violó o se ignoró**, y **si está registrada como tal** (¿el ADR está sellado como implementado en su `docs/adr/ADR-XXXX.md`? ¿el código realmente lo respeta?).

**Cada hallazgo se registra con veredicto:** cumplido / violado / ignorado / no-implementado.

**Why:** el problema raíz que motiva todo esto (documentado en el skill del Tech-Lead) es que existen decisiones arquitectónicas que se tomaron pero **nunca se aplicaron** en el código. La auditoría cierra ese hueco de forma sistemática, ADR por ADR, no solo para el de esquema.

**How to apply:** vara de medición = **contraste bidireccional** (retar el código Y el ADR; cualquiera de los dos puede estar mal/obsoleto — Gate de Coherencia del `tech-lead/SKILL.md`). Procedimiento barato de tokens: recorrer el índice `docs/ADR.md` (una línea por ADR), abrir cada `docs/adr/ADR-XXXX.md` bajo demanda, y contrastar contra el código/migraciones reales. Hallazgos que sean defecto de implementación → Story de corrección; hallazgos que revelen ADR obsoleto/equivocado → escalar al Architect. Anomalías de esquema de arranque (heredadas del Architect): A5 PRAGMAs `pool.rs` (urgente), A6 baseline `STRICT`+UUIDv7, A4 `audit_chain_hash`→NULL, A3 `event_sequence_id`→`row_version`, A2 destino de `foundation_master_fields.event_sequence_id`.
