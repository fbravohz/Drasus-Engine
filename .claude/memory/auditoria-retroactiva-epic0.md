---
name: auditoria-retroactiva-epic0
description: La auditoría retroactiva EPIC-0/STORY-001 contrasta TODOS los ADRs uno por uno (no solo ADR-0141); registra por cada uno si se cumplió/violó/ignoró/no-implementó.
metadata: 
  node_type: memory
  type: project
  originSessionId: 121318b9-21ba-4d1e-a76b-a3648b955c7b
---

La auditoría retroactiva desde EPIC-0 / STORY-001 (pendiente, va después de los cimientos de pricing — ver [[pricing-foundations-saas]]) tiene alcance **AMPLIADO** por decisión del usuario (2026-07-03): **no se contrasta solo ADR-0141** (Modelado Relacional Soberano). Se recorre el **catálogo completo de ADRs** (índice `docs/ADR.md`) y, **ADR por ADR**, se verifica en el código YA construido si cada decisión **se cumplió, se violó o se ignoró**, y **si está registrada como tal** (¿el ADR está sellado como implementado en su `docs/adr/ADR-XXXX.md`? ¿el código realmente lo respeta?).

**Cada hallazgo se registra con veredicto:** cumplido / violado / ignorado / no-implementado.

**Why:** el problema raíz que motiva todo esto (documentado en el skill del Tech-Lead) es que existen decisiones arquitectónicas que se tomaron pero **nunca se aplicaron** en el código. La auditoría cierra ese hueco de forma sistemática, ADR por ADR, no solo para el de esquema.

**How to apply:** vara de medición = **contraste bidireccional** (retar el código Y el ADR; cualquiera de los dos puede estar mal/obsoleto — Gate de Coherencia del `tech-lead/SKILL.md`). Procedimiento barato de tokens: recorrer el índice `docs/ADR.md` (una línea por ADR), abrir cada `docs/adr/ADR-XXXX.md` bajo demanda, y contrastar contra el código/migraciones reales. Hallazgos que sean defecto de implementación → Story de corrección; hallazgos que revelen ADR obsoleto/equivocado → escalar al Architect. Anomalías de esquema de arranque (heredadas del Architect): A5 PRAGMAs `pool.rs` (urgente), A6 baseline `STRICT`+UUIDv7, A4 `audit_chain_hash`→NULL, A3 `event_sequence_id`→`row_version`, A2 destino de `foundation_master_fields.event_sequence_id`.
