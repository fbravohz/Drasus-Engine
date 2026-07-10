---
name: debt-registry-y-atomicidad
description: "DEBT.md es el registro canónico de deuda rastreada (regla: si no está ahí, no está rastreada). Regla permanente nacida de DEBT-001: todo ledger append-only nace atómico (BEGIN IMMEDIATE + reintento + WriteContention + prueba de 2 escritores)."
metadata:
  node_type: memory
  type: feedback
  originSessionId: 121318b9-21ba-4d1e-a76b-a3648b955c7b
---

Dos disciplinas establecidas en la construcción del substrato de monetización (2026-07-04/06):

**1. `docs/DEBT.md` = registro canónico de deuda técnica rastreada.** Análogo a `TEST.md`. Regla de oro: *si una deuda no está en DEBT.md, no está rastreada.* Cada entrada lleva severidad (🔴/🟠/🟡), origen, descripción, impacto actual, disparador de pago y estado (`Abierta`/`En pago`/`Pagada` con enlace a la Story que la saldó). `PROGRESS.md` (bitácora TL) *narra* cuándo se halló; DEBT.md es el registro. **Sistema de 3 registros:** ROADMAP (fases/diferidos arquitectónicos, lo edita el Architect) + DEBT.md (deuda granular hallada en implementación, lo edita el TL) + banners "Pendiente:" en cada feature. No duplicar: el adaptador de red a la Cabina / servidor gRPC / panel productivo viven en el ROADMAP, NO en DEBT.

**2. Regla permanente de atomicidad de ledgers append-only (causa raíz DEBT-001).** Todo ledger que asigne `event_sequence_id` con read-then-write DEBE envolver el `load_tail`+`INSERT` en UNA transacción `self.pool.begin_with("BEGIN IMMEDIATE")`, con `busy_timeout=5s` en el pool y un reintento acotado (`MAX_*_ATTEMPTS=5`) que delega a `try_X_once`; al agotarse, error tipado `WriteContention { attempts }` — **nunca pérdida silenciosa de evento**. `BEGIN IMMEDIATE` toma el lock de escritura al entrar (evita el deadlock de upgrade de DEFERRED). El `UNIQUE(event_sequence_id)` es cinturón-y-tirantes, no la guarda primaria. **Prueba de 2 escritores obligatoria** con DB en archivo temporal (NO `:memory:`, que serializa y no ejerce concurrencia). Ya vive en skills `rust-engineer` §4 y `qa-engineer` §2.

**Why:** DEBT-001 se dejó como deuda por un vacío de plantilla (el skill exigía tamper-evidence pero no atomicidad ni prueba de concurrencia), no por descuido del agente. El usuario objetó con razón que perder eventos no es deuda menor y que debe haber reintento, no solo excepción. Desde #5 en adelante todo ledger nace correcto; STORY-032 endureció los viejos (`audit_log`, `usage_records`).

**How to apply:** al abrir/saldar cualquier deuda, edita DEBT.md primero. Al revisar cualquier ledger append-only nuevo, exige el patrón atómico completo o es defecto (no APTO en QA). Falsación empírica válida: quitar el `BEGIN IMMEDIATE` dejando solo el reintento debe tumbar la prueba de 2 escritores. Enlaza con [[pricing-foundations-saas]], [[politica-de-pruebas-y-validacion]].
