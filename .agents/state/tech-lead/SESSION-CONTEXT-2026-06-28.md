# Volcado de Contexto de Sesión — 2026-06-27/28

> Respaldo de continuidad solicitado por el usuario (no implica eliminar ni compactar la conversación). Captura decisiones, estado y trabajo pendiente para poder retomar en frío.

## ⏸️ PUNTO DE PAUSA (2026-06-28 — cerca del límite semanal, agentes detenidos)

Estado limpio y compilable. **Terminado esta sesión:** CLI `drasus verify` (Canal #2 Fase 1, verificado, descarga real de Binance) + capa FFI del fetcher (Bridge: `crates/bridge/src/api/data_fetcher.rs` + `ui/lib/src/rust/api/data_fetcher.dart`, patrón await) + `## Cáscara Visual` del fetcher + Spike gRPC (verde) + enmiendas ADR-0117/0136 + ADR-0142 + ADR-0141 + greenfield. **Detenido sin escribir nada (re-despachar limpio):** (1) Flutter SVF (Banco de Verificación + sección fetcher + Dashboard widget — la spec está en el feature doc §Cáscara Visual; los gaps G1/G2/G3 y los métodos FFI están en PROGRESS.md y en `crates/bridge/src/api/data_fetcher.rs`); (2) Architect reconciliación ROADMAP infra Canvas (subir de EPIC-8 a inicio de auditoría + 3 manifestaciones siempre).
**Resume aquí:** (a) re-despachar Flutter SVF para cerrar STORY-024; (b) commit acumulado (NADA commiteado desde `66da21e`); (c) auditoría retroactiva; (d) ADR-0143 + Story gRPC Fase 2 (no urgente).

## 1. STORY-024 — Sovereign Data Fetcher

- **Backend (motor de descarga, TTR-001 Bulk + TTR-002 Delta): TERMINADO y commiteado.** Primer crate hexagonal de dominio (`crates/features/data/sovereign-data-fetcher/`). Concurrencia real (`JoinSet` + `Arc<dyn BulkSource>`), reconciliación sin duplicados, Jobs durables (ADR-0011), recuperación tras crash, migración 0006 Perfil A. 20/20 tests, QA APTO.
- **Ciclo:** 1ª entrega → verificación TL → QA NO APTO (concurrencia falsa, `Semaphore` decorativo) → regresión (JoinSet + test honesto `peak>=2`) → re-auditoría TL → QA APTO → sellada.
- **⚠️ INCOMPLETA por ADR-0117:** se selló SIN su **SVF (Superficie de Verificación Funcional)**. El fetcher NO es plomería: produce `Tick`/`Bars` y toma configuración del usuario (broker/símbolo/fechas/timeframe) → tiene **Superficie propia**. Falta su superficie de verificación en la UI. PENDIENTE: dispatch Designer → Bridge (FFI `data_fetcher.dart`) → Flutter (entrada del fetcher en el banco de SVF).
- **Decisión pendiente registrada:** reintentos Bulk fijos en 3 (no configurables aún).

## 2. ADR-0141 — Modelado Relacional Soberano (RATIFICADO 2026-06-28)

Cierra el vacío de modelado relacional. Decisiones del usuario:
- `audit_chain_hash` = **NULL** canónico en la fila génesis (corregir `permission_decisions`).
- **STRICT** en TODAS las tablas (incl. las 6 existentes) vía edición in-situ del baseline (legítimo en greenfield).
- **UUIDv7** en TODAS las PKs (cambio en Rust: `uuid` feature `v7` + `Uuid::now_v7()`; el tipo de columna no cambia).
- Sesiones de mercado: **derivar en runtime** (sin columna).
- Umbral: "crece con el mercado → Parquet; crece con acciones del usuario → SQLite".
- Precios `INTEGER ×10⁸` (nunca `REAL`); timestamps `INTEGER` ns UTC.
- Checks del Gate: M1–M12 (migraciones), R1–R7 (Rust), F1–F3 (specs) — en ADR-0141 §"Gate del Tech Lead".
- Archivos: ADR-0141 creado; ADR-0006 enmendado; ADR-0030/0035/0036 banners; CLAUDE.md flag; SAD-08/11/20.

## 3. Fase del proyecto: GREENFIELD

Monolito de **escritorio**; "producción" = la instancia en la máquina de cada usuario individual (no servidor central). Baseline de migraciones editable in-situ hasta el **primer release distribuido**, que dispara BROWNFIELD (migraciones forward-only, robustas a saltos de versión). Declarado en CLAUDE.md §1 + ADR-0006 enmendado.

## 4. Reglas nuevas grabadas en skills (esta sesión)

- **tech-lead:** (1) contraste bidireccional en el Gate (retar feature/ADR/SAD); (2) **Etapa 7 "¿Qué aprendimos y cómo mejoramos?"** al cerrar cada iteración; (3) TDD/prueba discriminante; (4) Ventana de Verificación / SVF; (5) trazabilidad ADR→checks del Gate; (6) checks ADR-0141; (7) **Definición de Terminado innegociable**: no se sella sin SVF, UI en la misma Story (prohibido diferir a Story separada o EPIC-8); (8) test afilado plomería vs Superficie propia.
- **rust-engineer / qa-engineer:** prueba discriminante (debe poder fallar; medir el comportamiento).
- **architect:** regla fase greenfield/brownfield.
- **flutter-engineer:** patrón SVF canónico (imitar `ui/lib/tabs/clock_tab.dart`).

## 5. Memoria actualizada

- `verification-surface-svf.md` (NUEVO): toda feature entrega su SVF en la misma Story; canal de debug #1; canal #2 (gRPC/CLI) futuro.
- `arquitectura-visual-canvas.md` (enriquecido): el Canvas DAG es CENTRAL (drag-drop de nodos, extraer features sin la orquestación monolítica del módulo, flujos custom); 3 manifestaciones de UI.

## 6. Estrategia de UI y Verificación (decisiones del usuario 2026-06-28 — EN FORMALIZACIÓN)

1. **SVF = un único tab "Banco de Verificación"** con menú lateral (estilo galería) para seleccionar features y probarlas; NO un tab por feature. (Refina ADR-0117.)
2. **Las 3 manifestaciones de UI se desarrollan EN LA MISMA STORY que el backend** (SVF + Dashboard widget + nodo Canvas DAG), no diferidas a EPIC-8. Razón: contenido para redes + progreso visual continuo. **EPIC-8 queda solo como unificación/pulido.**
   - ⚠️ **Bloqueo real a resolver:** el sistema de card-nodes del Canvas (infraestructura drag-drop) NO existe aún. No se puede "añadir el nodo de una feature" sin esa infra. Opciones a decidir: (A) construir la infra del Canvas como Story fundacional próxima; (B) por ahora SVF bench + Dashboard widget por feature, nodo Canvas cuando exista la infra.
3. **UI-Designer audita la UI de cada feature** (propone y aplica mejoras) — Etapa 0.5, antes de Flutter.
4. **Canal de debug #2 (gRPC/CLI tipo Postman): PRIORITARIO ("lo quiero ya").** El desktop usa FFI (flutter_rust_bridge); gRPC es el modo headless/SaaS (CLAUDE.md). Requiere decisión de arquitectura (servidor gRPC tonic + protos, o harness CLI interino). ESCALADO al Architect.

## 7. Cola de trabajo pendiente

1. **Estrategia UI+gRPC** (escalada al Architect): formalizar SVF bench, scope de 3-manifestaciones-por-Story + sequencing de infra Canvas, rol de auditoría del Designer, harness gRPC/CLI.
2. **Completar STORY-024 con su SVF** (Designer → Bridge → Flutter) — tras formalizar la estrategia.
3. **Auditoría retroactiva desde STORY-001/EPIC-0** con ADR-0141 como vara. Primer ítem: `pool.rs` PRAGMAs (A5). Luego baseline STRICT+UUIDv7 (A6), `permission_decisions.audit_chain_hash`→NULL (A4), `jobs.event_sequence_id`→`row_version` (A3), destino de `foundation_master_fields.event_sequence_id` (A2). + verificar SVF de cada feature ya cerrada.
4. **Infra del Canvas DAG** (decisión A/B pendiente del usuario).
5. **Dashboard widgets** (`dashboard_registry.dart` hoy `available:false`).

## 8. Commits de la sesión (en main)

- `4523003` feat(ingest): STORY-024 motor de descarga.
- `fd767bd` docs: ADR-0141 + ADR-0006 + SAD + STORY-024.
- `66da21e` chore(skills): retroalimentación de iteración.
- (posteriores: correcciones de clasificación UI + estas actualizaciones, sin commitear aún salvo indicación.)

## 9. Workstream paralelo del usuario (no tocar)

ADR-0140 (opciones financieras post-MVP), `options-analysis.md` (notas Black-Scholes del usuario), prep de 9 features para opciones. Commits `9517996`/`953dfe2`/`63c5b80`. Es trabajo del usuario; no incluir en commits del TL.
