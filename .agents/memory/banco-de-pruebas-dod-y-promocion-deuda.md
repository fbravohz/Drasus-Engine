---
name: banco-de-pruebas-dod-y-promocion-deuda
description: "El Banco de Pruebas (harness SVF genérico) es gate permanente de Definición de Terminado; y una deuda con disparador cumplido debe promoverse al cerrar el hito, no seguir dormida."
metadata:
  node_type: memory
  type: feedback
  originSessionId: 7afc36c0-aab0-4801-8ee7-bd8897008d98
---

Dos correcciones del propietario (2026-07-12), a raíz de que recomendé arrancar EPIC-1 cuando lo que tocaba era la tanda de UI del substrato (DEBT-005):

**1. El Banco de Pruebas es gate permanente de Definición de Terminado.** De ahora en adelante **ninguna feature se considera Terminada si no se hace su conexión al Banco de Pruebas** (el harness SVF genérico). Formalizado por el Architect en `docs/features/verification-bench.md` + ADR nuevo (siguiente libre, 0152) que enmienda ADR-0117. El Banco: selector de feature + input JSON precargado (izq) + botón Enviar (centro) + respuesta real por FFI read-only (der); **debe mostrar visualmente si el input está bien o mal formado** para el puente FFI, no solo el payload de salida. Chasis parcial ya existía en `ui/lib/tabs/verification_bank/`. Sprint STORY-050 lo construye y enchufa las 14 features del substrato. Enlaza con [[verification-surface-svf]] y [[feedback-svf-galeria-transversal]].

**2. Una deuda con disparador cumplido debe promoverse — no quedarse dormida.** DEBT-005 tenía disparador "al cerrar los backends del substrato"; se cumplió el 2026-07-10 pero seguía 🟡 Baja y no saltó como "lo que sigue". Fallo doble: (a) severidad subestimada (un gap de superficie de verificación NO es cosmético, es gap de DoD → mínimo 🟠 Media); (b) nadie convierte "disparador cumplido" en "activo".

**Why:** el propietario es perfil frontend con tiempo limitado; su único canal real de validación es probar en la app. Recomendarle el siguiente EPIC saltándose lo que le permite VER lo construido lo deja a ciegas y le hace perder tiempo.

**How to apply:** (1) al preguntar/responder "¿qué sigue?", barrer `docs/DEBT.md` por disparadores cumplidos ANTES de mirar la lista de EPICs; las deudas de gap de DoD (SVF/Banco de Pruebas) van arriba del siguiente EPIC. (2) al cerrar cualquier hito, barrer DEBT.md y promover las deudas cuyo disparador ese hito cumplió (subir severidad + marcar disparador cumplido + mover a "Siguiente" en PROGRESS). Regla canónica en `.agents/knowledge/debt-management.md` §Regla de Promoción. Enlaza con [[debt-registry-y-atomicidad]] y [[roadmap-metodologia-modulo-completo]].
