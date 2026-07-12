# DEBT-008 · `enriched-domain-events` (#6) sin fan-out al bus (ADR-0085)
- **Severidad:** 🟡 Baja
- **Origen:** STORY-033 (`enriched-domain-events`).
- **Descripción:** el cimiento #6 persiste cada evento en el event-store append-only local (atómico), pero **no lo publica al bus** (ADR-0085). Hoy la Shell escribe a la tabla; el `event_out` fan-out a los consumidores vivos (medición, agregación, reportes) no está cableado.
- **Impacto actual:** nulo — los consumidores del substrato (#4, #7, #9) leen del event-store por puerto, no del bus; el bus solo se necesita para reacción en tiempo real (feedback, telemetría).
- **Disparador de pago:** al cablear el primer consumidor que exija push en vivo (módulo `feedback` / telemetría), o al construir el bus como infraestructura. Distinto del **adaptador de red a la Cabina de Mando**, que vive en el ROADMAP (no aquí).
- **Estado:** Abierta.
