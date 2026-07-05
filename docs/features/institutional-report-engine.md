# Institutional Report Engine

**Carpeta:** `./features/institutional-report-engine/`
**Estado:** En Diseño
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (cimiento #7) · ADR-0101 (plantillas Tera) · ADR-0027 (audit trail)

## ¿Qué es esta feature?

El puerto que consume resultados del guantelete de validación/ejecución (ya existente) y produce **reportes institucionales** con firma de integridad, plantilla y trazabilidad al audit-log. Ahora se entrega el **puerto + una plantilla base**; el catálogo de reportes (stress test, validación, forense, certificación) son adaptadores posteriores (moonshot `institutional-report-products`).

- **Problema:** el día que un fondo pida un reporte de stress test, no se puede decir "dame tres meses para construir el generador". Si el motor ya produce los datos, el reporte es solo una plantilla.
- **Comportamiento observable:** dado un resultado de validación/ejecución, produce un documento institucional (PDF/HTML/JSON) firmado y trazable.
- **Por qué:** habilita los productos de 5–6 cifras que son subproductos naturales del guantelete.

## Comportamientos Observables

- Cuando se le pasa un resultado de validación → genera un reporte con las métricas, su firma de integridad y enlaces al audit-log fuente.
- Cuando el reporte se vuelve a generar sobre el mismo resultado → produce la misma firma (determinismo).
- Cuando el cliente pide branding (white-label) → aplica logo/colores sin alterar los datos.

## Restricciones

- NUNCA un reporte altera los datos fuente: solo los presenta.
- Cada dato del reporte enlaza a su evento en el audit-log (trazabilidad, ADR-0027).
- La firma de integridad es criptográfica y reproducible.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| REPORT_FORMATS | PDF, HTML, JSON | conjunto | Formatos de salida disponibles | CONFIG |
| BRANDING_ENABLED | false | true/false | White-label del reporte | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** ensamblado del reporte a partir de los datos (sin I/O), cálculo de la firma de integridad.
- **Shell (Infraestructura):** render de la plantilla (Tera, ADR-0101), lectura del audit-log, escritura del artefacto.
- **Frontera Pública:** puerto `generate_report(input) -> Report` por comportamiento; consumido por los adaptadores de producto y la API de terceros.

## Ciclo de Vida de la Feature — Institutional Report Engine

### Entrada
Un resultado del pipeline (validación, backtest, ejecución) + la plantilla + opciones de branding.

### Proceso
Ensambla el reporte, lo firma y lo enlaza al audit-log.

### Salida
Un documento institucional firmado y trazable, en el formato pedido.

## Tareas (TTRs)

- **TTR-001:** Puerto de generación de reportes + plantilla base (Core: ensamblado + firma).
- **TTR-002:** Enlace de cada dato del reporte a su evento del audit-log (trazabilidad).

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `result_in` | `BacktestResult` / `RobustnessScore` | Input | `1..N` | Resultados del guantelete a reportar. |
| `report_out` | `InstitutionalReport` (tipo técnico nuevo — plomería, ADR-0144) | Output | `1` | Documento firmado y trazable. |

## Cáscara Visual (Thin Shell)

> Plomería con salida documental (Ventana de Verificación). El UI-Designer escribe la nota de observable. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** el reporte se genera donde están los datos (plano de ejecución); puede exponerse por la API de terceros.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría/Forense)**: Identidad(I) + Soberanía(II) + subset V (`signature_hash`, `compliance_status_id`).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Tabla de reportes generados con Grupo I + Perfil D. Campos propios fuera del catálogo (marcados): tipo de reporte, `signature_hash`, referencia al resultado fuente. `STRICT`, UUIDv7 (ADR-0141).

## Dependencias y Bloqueantes

- **Depende de:** el guantelete de validación (existente), `enriched-domain-events`, audit-log.
- **Bloquea a:** los adaptadores de producto (moonshot `institutional-report-products`) y la API de terceros.
- **Contrato de Integración UI (ADR-0117) — Ventana de Verificación:** su observable (reporte generado + su firma) queda visible en el tab de una feature consumidora del módulo `validate`; hasta entonces, deuda de integración registrada.
