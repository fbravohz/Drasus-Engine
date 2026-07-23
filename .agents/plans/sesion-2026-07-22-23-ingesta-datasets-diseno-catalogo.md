# Sesión 2026-07-22/23 · Frente de ingesta: datasets reales, spread/fill/DST y catálogo de entidades

> **Propósito de este documento:** handoff completo para el siguiente Tech-Lead. El chat de origen se elimina; esto + `PROGRESS.md` + el ROADMAP deben bastar para retomar sin pérdida. Fecha verificada con `scripts/now.sh`: 2026-07-23 (tarde, hora de México).

---

## 1. Estado en una línea

El **frente de diseño de ingesta está SELLADO** (ADR-0162 + 5 enmiendas + 2 features nuevas + 4 extendidas), **sin commitear**, listo para revisión del propietario. El siguiente paso real es **código**: despachar ingenieros a construir el adaptador de ingesta MetaTrader (CSV M1) contra el esquema canónico.

---

## 2. Qué se decidió y por qué (contexto de dominio)

**Fuente de datos.** El propietario entregó `datasets/` (~900 MB, en `.gitignore`, jamás versionar):
- `EURUSD_M1_2000-01-01_2025-05-01.csv` (561 MB) — **la fuente principal**: barras M1 de MetaTrader, 2000–2025, con columna SPREAD. Delimitador TAB, header `<DATE> <TIME> <OPEN> <HIGH> <LOW> <CLOSE> <TICKVOL> <VOL> <SPREAD>`.
- `EURUSD_TICK.csv` (203 MB) — ticks bid/ask reales, solo abril 2025. **El propietario lo va a BORRAR.** Nada depende de él.
- 3× `.dat` (Dukascopy/IC Markets) — binario propietario de QuantDataManager, **no ingestibles sin re-exportar a CSV** (follow-up). Uno mal nombrado: `dukas_H1` es H1, no M1.
- Ningún archivo declara zona horaria ni bróker → se DECLARAN al importar.

**Reconciliación bid/ask (clave).** El propietario propuso "operamos en el open y aplicamos el spread". Esto **NO revierte** el invariante Bid/Ask Exclusivo (ADR-0114 regla c) — lo **cumple por reconstrucción**: `open` MT5 = bid, `ask = open + spread_reported`, compra al ask, vende al bid, nunca punto medio. La columna SPREAD es lo que hace ingestibles los 25 años para fills.

**Correcciones del propietario a las primeras propuestas de los agentes** (todas ya aplicadas en los tickets/ADR):
1. El piso de spread NO se calibra contra un dataset fugaz (abril 2025). Se calibra **por instrumento**: `default_spread` declarado del catálogo + distribución de spreads no-cero del propio archivo.
2. La zona horaria NO se infiere de una noticia (Nóminas) ni de la volatilidad. Se **declara** (selector mundial con bandera DST, patrón SQX). Se persiste como identificador **IANA** (`America/New_York`), no como `EST+07`. Hueco de fin de semana = verificación estructural opcional.
3. El catálogo se productiza en **FEATURES hexagonales, no tablas sueltas**.
4. El gate de coherencia sesión/activo es **N×N por huella de calendario** (no lista de pares): compara huella declarada vs. observada; simétrico; `INDETERMINADO` si el archivo es muy corto.

---

## 3. Artefactos producidos (TODO SIN COMMITEAR)

**Tickets de análisis:**
- `docs/execution/TASK-064-politica-spread-fill-model-dst.md` (Quant) — política spread nulo/cero, selector real/aleatorio/fijo, DST declarado, plan de validación opcional abril 2025.
- `docs/execution/TASK-065-catalogo-broker-instrumento-sesion.md` (Architect) — catálogo + descomposición en features (§2-bis) + gate N×N (§3-bis).

**ADR nuevo + índice:**
- `docs/adr/ADR-0162.md` — Catálogo Instrumento × Bróker × Sesión. Activa ADR-0140, extiende 0154/0155/0137. Contiene 3 secciones "Corrección del Architect" (retó la premisa, GOVERNANCE-4.1.7): clave técnica UUIDv7 vs. `instrument_id` de negocio; `row_version` (no `event_sequence_id`, son tablas mutables); zona horaria IANA (no `EST+07`).
- `docs/ADR.md` — índice actualizado con 0162.

**ADRs enmendados:** `ADR-0114` (reconstrucción de ask desde spread), `ADR-0140` (activado), `ADR-0154`, `ADR-0155`, `ADR-0137`.

**Features nuevas:** `docs/features/instrument-catalog.md`, `docs/features/session-catalog.md`.

**Features extendidas:** `docs/features/pit-data-validator.md` (gate de coherencia N×N), `docs/features/data-import-wizard.md` (selector de zona + gate), `docs/features/data-sanitizer-pipeline.md` (política de piso de spread), `docs/features/backtest-engine.md` (selector `SPREAD_SOURCE_MODE`).

**Retrofit de ledger commiteado al inicio de sesión** (`18e4cf0`..`5814034`): persistence.rs + integration_tests.rs + migración 0006 (DEBT-032/034), docs de escala/barras/ingesta, `.gitignore` de `datasets/`.

**Instrucciones y memoria (sin commitear):**
- `.agents/knowledge/governance.md` — regla nueva **GOVERNANCE-4.1.7 "Reta la Premisa Recibida"** (imperativa, todo agente).
- `.agents/skills/tech-lead/SKILL.md` — regla "no precocines la solución en el prompt de despacho".
- `.agents/memory/despacho-no-precocinar-solucion.md` (+ MEMORY.md).
- `scripts/now.sh` — reloj de pared determinista.

---

## 4. Lección de proceso de esta sesión (crítica)

Dos subagentes (Quant, Architect en su primera pasada) entregaron trabajo defectuoso porque **el Tech-Lead les precocinó la solución en el prompt** (dataset fugaz + noticia; tablas en vez de features). No fue capacidad del modelo, ni contexto, ni la división de governance. Se canonizó en dos planos: el que despacha no precocina (`tech-lead/SKILL.md`); el que recibe reta la premisa (`GOVERNANCE-4.1.7`). En la segunda pasada, el Architect **sí** retó la premisa (3 correcciones) — el encuadre correcto funcionó.

---

## 5. Siguiente paso para el próximo Tech-Lead

1. **Revisión del propietario** de ADR-0162 y las features (pendiente — él lo pidió antes de sellar formalmente).
2. **Commitear el lote** (requiere autorización explícita del propietario, regla "autorizar cada commit"): agrupar `docs` (tickets + ADRs + features), `chore(agents)` (governance + skill + memoria + `scripts/now.sh`). NO push salvo que lo pida.
3. **Despachar ingenieros** a construir el adaptador de ingesta MT5 → esquema canónico contra el CSV M1: parser TAB, escala ×10⁸, timestamps ns UTC con zona IANA declarada, reconstrucción ask=open+spread, política de spread nulo/cero, gate de coherencia de calendario. Aplicar `GOVERNANCE-4.1.7` en cada despacho: problema + restricciones + doc + entregable, NO el método masticado.
4. **Re-exportar los `.dat`** a CSV desde QuantDataManager (follow-up, no bloqueante).

---

## 6. Bloqueos vigentes

- **Alcance de día 1 del catálogo de sesiones** (`session-catalog` §Dependencias): cuántas plantillas preconfiguradas salen al arrancar más allá de Forex 24/5 y Cripto 24/7 → depende de qué brókers soporta el primer conector real. `🔴 pendiente de confirmación del propietario` (relacionado con lo que fue TASK-060).
- **TASK-060 (muestras reales)**: efectivamente **satisfecho** para el primer build de Forex (el CSV M1 basta). Queda como follow-up el re-export de los `.dat`.
- **Límite de sesión**: golpeado 2026-07-23 madrugada, ya reseteado. Usar `scripts/now.sh` para no estimar mal; tratar el error de límite como señal de pausa (no hay API de cuota consultable).
