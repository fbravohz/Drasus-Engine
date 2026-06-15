## 5. Requisitos No-Funcionales (8 Leyes de Drasus Engine)

Drasus Engine adhiere a 8 leyes fundamentales que garantizan su rigor científico y operativo:

1. **Event-Driven:** Operación sobre flujos de eventos tipados (NautilusTrader).
2. **Deterministic Replay:** Backtests reproducibles bit-a-bit (Seeds PRNG documentados).
3. **Fail-Safe by Default:** Circuit breakers, kill switches y límites de riesgo integrados en el core.
4. **High-Performance FFI/gRPC:** Uso de Apache Arrow y Polars para zero-copy entre módulos.
5. **CPU-Centric Efficiency:** Rust Native y Polars lazy evaluation; GPU para cómputo masivo.
6. **Zero-Trust Validation:** Esquemas Serde estrictos en todas las fronteras de módulos.
7. **Absolute Parameterization:** Cero hardcoding; 100% configurable dinámicamente.
8. **Data Sovereignty:** Arquitectura Local-First; soberanía total de datos y capital.

### 5.1 KPIs de Rendimiento y Escalabilidad

| Métrica | Target | Justificación |
|---------|--------|---|
| **Backtest Throughput** | Más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114; sin KPI absoluto) | Exploración masiva de Alpha. |
| **Live Order Latency** | ≤100ms (end-to-end) | Ejecución competitiva institucional. |
| **Monte Carlo (CPU)** | 10K iteraciones en tiempo acotado vía `ndarray`/Rayon | CPU-first; GPU `candle` opcional (ADR-0061/0112). |
| **Watchdog Kill** | ≤5s detección | Supervivencia ante fallos sistémicos. |
| **Data Size Support** | 100GB+ | Gestión eficiente mediante DuckDB/Parquet. |

---

