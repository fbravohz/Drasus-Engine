---
name: verification-surface-svf
description: "Toda feature (incluida plomería) entrega una Superficie de Verificación Funcional (tab SVF) para que el humano pruebe el flujo front→back→DB sin leer código. ADR-0117. Canal de debug #1."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 121318b9-21ba-4d1e-a76b-a3648b955c7b
---

El usuario (perfil frontend, con poco tiempo: maneja redes sociales, su empleo que financia el emprendimiento, y la dirección macro/micro con los agentes) tiene HOY un solo canal práctico para verificar que una Story funciona: **una superficie en la UI Flutter** — read-only o un widget que dispare el input y muestre el output. Por eso, por ADR-0117, **cada feature entrega su SVF (Cáscara Delgada) en la MISMA Story que su backend** — incluida la plomería (el reloj es plomería y aun así tiene `ui/lib/tabs/clock_tab.dart`).

**Why:** sin esa superficie, el humano tendría que meterse a Rust/Flutter para verificar, lo cual es inviable con su carga. La SVF es la evidencia del round-trip front→FFI→back→DB→front. Es también el error que cometí en STORY-024: sellé el backend del `sovereign-data-fetcher` SIN su SVF, violando el Gate de Integración Anti-Deuda de ADR-0117, y encima propuse diferir la UI a una "Story separada" (también prohibido: ADR-0117 exige misma Story).

**How to apply:** ninguna Story de feature se marca Completada sin (a) su tab SVF si tiene Superficie propia, o (b) su observable visible en el tab de una feature consumidora si es plomería (Ventana de Verificación). Patrón canónico: binding FFI en `ui/lib/src/rust/api/<feature>.dart` + tab en `ui/lib/tabs/<feature>_tab.dart`, imitando `clock_tab.dart`. Distinto del Dashboard widget (bento) y del nodo Canvas DAG (EPIC-8) — ver [[arquitectura-visual-canvas]].

**Canal de debug #2 (futuro, no definido aún):** un probador de gRPC/CLI (tipo Postman-para-gRPC o grpcurl) para disparar inputs/outputs del backend desde terminal sin UI. El usuario lo ve deseable; queda como propuesta a formalizar (ADR) cuando se priorice.
