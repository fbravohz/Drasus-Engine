# Regulatory Sign-Off Service (Validación Acreditada)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot — ⚠️ **ZIZAÑA** (el motor sí; la firma acreditada exige acreditación que no se tiene)
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (clasificación de modelos)

---

## ¿Qué es?

La idea (del documento-semilla): vender **validación de modelos con firma de tercero independiente** que satisface obligaciones regulatorias — SR 11-7 (Fed/OCC), UCITS/ESMA, Basel III, Solvency II. Cliente: fondos regulados, bancos. Ticket $20K–$500K/año.

## ⚠️ Por qué es zizaña (no va al núcleo tal cual)

- **La distinción crítica:** el **motor** puede generar el reporte de validación (PBO, CPCV, WFA, robustez) — eso ES un cimiento (`institutional-report-engine` + moonshot `institutional-report-products` / "Model Validation herramienta"). Pero el **sello que el regulador acepta** exige ser un **validador acreditado/reconocido** ante ese regulador. Vender "cumplimiento regulatorio" sin la acreditación es vender humo y expone a responsabilidad.
- **No es un problema técnico, es de acreditación legal:** ninguna cantidad de rigor estadístico sustituye el reconocimiento formal del regulador.

## Qué SÍ se construye ahora

La herramienta de validación y el reporte firmado (integridad criptográfica) se construyen como cimiento. Lo que queda archivado es la **promesa de validez regulatoria acreditada**, condicionada a obtener la acreditación correspondiente en cada jurisdicción.

## Dependencias

- Requeriría: acreditación como validador independiente ante cada regulador (SEC/ESMA/etc.). Trámite legal, no desarrollo.
- **Depende técnicamente de:** `institutional-report-engine` (ya cimiento).
