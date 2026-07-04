# Institutional Report Products (Suite de Reportes de Alto Valor)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (adaptadores del cimiento — se activan bajo demanda del mercado)
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (substrato) · cimiento `institutional-report-engine`

---

## ¿Qué es?

Un conjunto de **adaptadores** sobre el cimiento `institutional-report-engine` (ADR-0144 #7) que empaquetan capacidades ya existentes del guantelete de validación como productos vendibles a entidades (fondos, family offices, aseguradoras). Ninguno es un motor nuevo: cada uno consume el puerto de reportes + los motores que ya existen (Monte Carlo, WFA, CPCV, PBO, robustez decagonal, análisis de régimen).

Productos incluidos:
- **Stress Testing as a Service** — un portafolio → guantelete completo (Monte Carlo masivo, crisis históricas, estrés de liquidez, correlación en pánico) → reporte institucional. Cliente: fondos, pensiones, aseguradoras. Ticket $10K–$50K.
- **Model Validation (herramienta)** — validación independiente de una estrategia (PBO, CPCV, WFA, ablation, cross-market). Cliente: fondos regulados, bancos. Ticket $20K–$75K. **Caveat:** se vende la *herramienta y el reporte*, NO el sello acreditado (ver moonshot `regulatory-signoff-service`).
- **Backtest Certification** — score de legitimidad estadística (PBO, robustez) con sello "Drasus Certified" para pitch decks. Cliente: traders/fondos levantando capital. Ticket $5K–$50K.
- **Drawdown Forensics** — análisis forense de un evento de drawdown (descomposición de PnL, régimen, microestructura, slippage). Cliente: fondos en crisis. Ticket $25K–$100K.

## Comportamientos Observables

- [ ] El cliente sube su portafolio/estrategia por la API de terceros → el motor ejecuta el guantelete → devuelve un reporte firmado y trazable.
- [ ] El reporte enlaza cada dato a su evento en el audit-log (ADR-0027).
- [ ] El sello de certificación es reproducible: mismo input → misma firma.

## Restricciones

- **NUNCA** se promete validez regulatoria acreditada aquí: eso vive en `regulatory-signoff-service` (moonshot zizaña, condicionado a acreditación).
- Reutiliza el guantelete existente; PROHIBIDO duplicar motores de validación.
- El dato del cliente se procesa bajo su consentimiento; no alimenta agregados sin permiso.

## Dependencias

- **Depende de:** `institutional-report-engine`, `third-party-api-gateway`, `consent-registry`, y los motores de validación existentes (módulo `validate`).
- **Se activa:** cuando el primer cliente institucional lo pida (los contratos ya existen; el adaptador es trabajo de días).

## Por qué es moonshot

El motor y el puerto se construyen ahora (cimiento); el empaquetado como producto, el pricing y el go-to-market institucional son posteriores y dependen de reputación/track record. No bloquea el MVP.
