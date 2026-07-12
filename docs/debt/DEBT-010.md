# DEBT-010 · Render Tera→PDF/HTML no cableado en #7
- **Severidad:** 🟡 Baja
- **Origen:** STORY-034 (`institutional-report-engine`).
- **Descripción:** #7 produce hoy la **estructura firmada** del reporte (JSON canónico + `signature_hash`); el render a PDF/HTML con plantilla Tera (ADR-0101) y white-label **no** se añadió (Tera no está como dependencia). El catálogo de productos (stress/validación/forense/certificación) es un moonshot aparte (`institutional-report-products`), NO esta deuda.
- **Impacto actual:** ninguno — el dato firmado y trazable ya existe; falta la capa de presentación.
- **Disparador de pago:** al primer cliente que pida el documento renderado, o al construir el moonshot de productos de reporte.
- **Estado:** Abierta.
