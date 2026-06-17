## 3. Decisiones Técnicas Clave (ADRs)

> **Nota:** Esta tabla es un resumen curado de los ADRs más representativos para la comprensión arquitectónica general; no es exhaustiva. El registro completo y vigente de todas las Decisiones de Arquitectura (incluyendo ADR-0014 a ADR-0119 y subsiguientes) es [`ADR.md`](../ADR.md).

| ID | Decisión | Propósito |
|---|---|---|
| **ADR-0001** | Un solo binario con módulos independientes + separación lógica pura vs. interacción | Evitar latencia de red de microservicios; mantener testabilidad de funciones sin efectos secundarios. |
| **ADR-0002** | Estructuras de datos puras + librerías de procesamiento vectorial | Desacoplar lógica de base de datos; permitir acceso a memoria compartida sin copias (zero-copy) y cálculos en paralelo en CPU con compilación JIT. |
| **ADR-0003** | Estructura de Carpetas (separación clara lógica pura / interacción) | Forzar aislamiento físico entre lógica pura e infraestructura; escalabilidad infinita sin archivos gigantes. |
| **ADR-0004** | Máquina de estados con números enteros (int64) | Garantizar que dos ejecuciones del mismo cálculo den idéntico resultado numérico; permitir optimización automática en cambios de estado. |
| **ADR-0005** | Versionado reproducible con historial completo en grafo | Reproducibilidad 100%, auditoría completa, pruebas A/B en vivo. |
| **ADR-0006** | Control de cambios de esquema centralizado | Una única fuente de verdad para cambios en tablas; reversión atómica de cambios; auditoría de quién cambió qué. |
| **ADR-0007** | Inyección dinámica de comportamiento (Feature Router) | Permitir coexistencia de variantes de features sin hardcoding; activar/desactivar por configuración. |
| **ADR-0008** | Todo es configurable excepto las reglas invariables | Cada usuario/equipo puede ajustar parámetros; solo las restricciones arquitectónicas son fijas. |
| **ADR-0009** | Interfaz Unificada Strategy-Portfolio (ExecutableContainer) | Strategy y Portfolio comparten contrato idéntico; módulos operan con lógica única, sin duplicación. |
| **ADR-0010** | Reglas Dinámicas (Hard Limits vs Soft Alerts) | Autonomía operativa: hard limits ejecutan automáticamente, soft alerts notifican. Usuario es autoridad final, no bloqueador. |
| **ADR-0011** | Operaciones Asincrónicas (Async Job Pattern) | Operaciones costosas se ejecutan en background con patrón Disparo→Monitoreo→Recuperación. Interfaz no se bloquea. |
| **ADR-0012** | Arquitectura Multi-Pipeline Paralela | N pipelines simultáneamente con recursos reservados para live trading. Usuario configura % CPU/RAM para exploración. |
| **ADR-0013** | Selección de Stack Tecnológico | Rust, SQLite+WAL, Polars/DuckDB, Tokio, Flutter (UI), flutter_rust_bridge (FFI). |
| **ADR-0029** | Patrón Todo en Uno (Rust + Flutter FFI) | Aplicación compilada nativamente como un único ejecutable, eliminando latencia FFI/gRPC/DOM. |
| **ADR-0086** | Minería Descentralizada (La Colmena) | Arquitectura cliente-servidor para crowdsourcing de cómputo GPU/CPU de exploración de estrategias. |
| **ADR-0087** | El Guardián & El Centinela | Validador de riesgo pre-trade global (<1ms) y Shadow Watchdog e interruptor automático de emergencia en Rust. |
| **ADR-0088** | Protocolo de Incubación & Cono de Silencio | Cuarentena acelerada de 7 días con Eutanasia Predictiva (MAE), bandas de confianza Monte Carlo y Broken Strategy Flag. |
| **ADR-0089** | Optimización & Rebalanceo de Portfolio | Asignación adaptativa HRP/Markowitz, Hedging Cointegrativo Tick-by-Tick (+0.85), Router de Liquidez y Auto-Rebalancing Daemon. |


### 3.1 Patrones Descubiertos (Insights de Arquitectura)

1. **Hexagonal > Performance:** Si una optimización de velocidad (hot path) rompe la modularidad o el desacoplamiento, se rechaza. La modularidad es la base de la longevidad del sistema.
2. **Clarificación Previa a Abstracción:** Antes de proponer una nueva interfaz (ABC/Protocol), se debe clarificar el problema de negocio. La mayoría de las "necesidades" de auditoría se resuelven con `test_results` (inmutable) + `live_results` (runtime), sin módulos extra.
3. **Versión Inmutable + Rama:** Las estrategias/portafolios se tratan como repositorios Git. Todo cambio es una nueva versión; los resultados de pruebas son acumulativos y heredables.
4. **Soberanía "Zero-Docker":** El sistema debe ser funcional localmente sin dependencias de red pesadas. SQLite WAL + Parquet es el estándar de oro para el Client Zero.
5. **IDs Mandatorios para Trazabilidad:** Todo (Módulo, Tarea, Feature) tiene un ID único (MOD-X, TASK-X, FEAT-X) para evitar redundancia y asegurar coherencia en auditorías forenses.

---

