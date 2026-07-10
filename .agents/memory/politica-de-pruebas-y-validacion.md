---
name: politica-de-pruebas-y-validacion
description: "Pirámide de pruebas canónica de Drasus Engine, herramientas obligatorias por capa y activación del QA-Engineer por fase (ADR-0133)."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: a40c0862-5083-4d31-aa60-f45bd583c96d
---

Política formalizada en ADR-0133 (2026-06-20), extiende la decisión original del usuario (2026-06-12).

**Pirámide canónica (7 capas — fuente de verdad: ADR-0133):**
1. **Unitarios** (`cargo test`) — siempre, desde EPIC-0. Lógica pura del `domain/`.
2. **Integración** (`cargo test` en `tests/`) — siempre, desde EPIC-0. Durabilidad → SQLite en archivo, nunca `:memory:`.
3. **Propiedad** (`proptest`) — obligatorio para toda función cuantitativa pura del `domain/`.
4. **Fuzzing** (`cargo-fuzz`, nightly solo en crate `fuzz/`) — obligatorio en fronteras externas declaradas en ADR-0133 (parsers Ingest, PIT Validate, FFI Bridge, deserialización Execute).
5. **Benchmarks** (`criterion`) — obligatorio desde EPIC-2 para rutas con SLA.
6. **Adversariales** (look-ahead injection + simulacro de fallo) — bloqueantes pre-dinero real.
7. **Flutter** (`flutter test`, `integration_test`) — EPIC-8+. No antes.

**Activación del QA-Engineer:**
- EPIC-0: NO obligatorio por Story. Disponible para escalados puntuales del Tech-Lead.
- EPIC-1+: Gate obligatorio (Etapa 5) antes de cerrar cualquier Story de lógica de dominio.
- Pre-dinero real: Pruebas de Guerra (QA §3) bloqueantes de release — sin excepción.

**Herramienta de cobertura:** `cargo llvm-cov --workspace --summary-only`.

**How to apply:** cada ingeniero entrega su propia pirámide en verde (capas 1–4 según apliquen) con mapeo criterio→prueba + cobertura antes de subir al Tech-Lead. El Tech-Lead reproduce; si falta una capa obligatoria, regresa sin cerrar.

Relacionado: [[adr-0020-contrato-logico]], [[roadmap-metodologia-modulo-completo]].
