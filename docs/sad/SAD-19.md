## 19. Glosario y Apéndices

### 19.1 Stack Técnico Detallado
* **Backend:** Rust, Tokio, NautilusTrader nativo (crates v2), Polars, DuckDB, `ndarray`/Rayon (cómputo numérico CPU-first; `candle` opcional, ADR-0112).
* **Frontend:** Flutter, Flutter FFI, Flutter CustomPainter (Impeller GPU rendering).
* **Persistencia:** SQLite (WAL), Parquet (Hive), Apache Arrow.

### 19.2 Nomenclatura de Tareas (Traceability)
Para asegurar el 100% de trazabilidad entre el PRD y la implementación, se utiliza el formato `FASE.MODULO.TAREA`:
* **MOD-X.Y:** Módulo técnico (ej: MOD-01 Ingest).
* **TASK-X.Y.Z:** Requisito funcional específico (ej: TASK-1.1.1 NSGA-II).
* **FEAT-X:** Feature arquitectónica compartida (ADR-0003).

---

