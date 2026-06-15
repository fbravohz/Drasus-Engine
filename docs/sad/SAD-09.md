## 9. La Fontanería

### Seguridad
* **Validación Desconfiada:** Validación de toda entrada externa (FFI/gRPC, WebSockets externos, base de datos).
* **Máquina de Estados:** Estados definidos exhaustivamente; transiciones imposibles no pueden ocurrir.
* **Auditoría:** Cada cambio de estado guardado con marca de tiempo.
* **Soberanía y Criptografía (ADR-0093):**
    * **Protección de Llaves:** Encriptación AES-256-GCM de claves de API en `broker_connections` con Master Key en variable de entorno.
    * **Auditoría Inmutable:** Registro secuencial encadenado mediante hash de transacciones en SQLite.
    * **Privacidad Soberana:** Cero telemetría externa para proteger IP de estrategias y datos operativos del usuario.

### Ejecución Automática con Auditoría

Ver **ADR-0010: Reglas Dinámicas (Hard Limits vs Soft Alerts)** para el mecanismo completo de ejecución automática, auditoría y veto.

### Concurrencia y Parallelismo
* **Monitoreo de degradación en paralelo con ejecución:** Monitoreo continuo de cambios mientras el portafolio ejecuta. Sin bloqueos; detección de cambios PnL/drawdown/régimen activable en cualquier momento.
* **Retroalimentación (Bucle Pardo) en paralelo y al cierre:** Control de calidad estadístico continuo. Compara paper/vivo vs backtest. Sugerencias se retroalimentan a generar para nueva evolución.
* **Arquitectura:** Tareas independientes asincrónicas; eventos entre módulos via bus de eventos.

### Operaciones Asincrónicas

Para operaciones costosas (backtesting masivo, generación de candidatas, optimización de portafolio), ver **ADR-0011: Operaciones Asincrónicas (Async Job Pattern)**.

### Multi-Pipeline Paralela

Para ejecución de N pipelines simultáneamente con reserva de recursos para live trading (SLA de reserva), ver **ADR-0012: Arquitectura Multi-Pipeline Paralela**. El sistema garantiza que al menos 2 núcleos de CPU y 4GB de RAM estén siempre disponibles exclusivamente para el pipeline de ejecución en vivo.

### Contratos de Intercambio (Signal Contracts)
Para asegurar que el pipeline sea reproducible y agnóstico, se definen los siguientes objetos de intercambio inmutables:
* **SignalEquation:** Representación simbólica o lógica de una señal de entrada.
* **FitnessVector:** Vector multi-objetivo (Sharpe, MaxDD, WinRate) usado para la selección natural de estrategias.

### Observabilidad

- **Propiedades:** Latencia p99 < 100ms (NautilusTrader), Throughput competitivo (más rápido que MT5/SQX/QuantConnect en igual hardware vía Polars Native, sin KPI absoluto — ADR-0114), disponibilidad local 100%.
- **Foundation Inundation (ADR-0020 V2):** Inyección de hooks tempranos para evitar refactorizaciones futuras. El esquema de base de datos es **Distribuidor y Basado en Requisitos**: cada Feature define su propio contrato de persistencia en su archivo `.md`, pero todas deben obedecer el **Contrato Global** definido en [ADR-0020 V2](../ADR.md#adr-0020-principio-de-inundación-de-fundaciones-v2-foundation-inundation).
- **Registro de eventos Estructurado:** Definido en [`telemetry.md`](../features/telemetry.md) (en JSON, rastreable).
- **Métricas:** Latencia de ingesta, rendimiento de señales, órdenes por segundo, cambio de rendimiento, drawdown actual.
* **Métricas Desacopladas por Módulo:** Cada módulo expone sus propias métricas de forma independiente; la feature `telemetry` las recolecta dinámicamente. 
* **Consumidor Maestro:** El módulo de `feedback` consume todos estos puntos de evidencia (causalidad) para generar los veredictos de salud de Pardo. **Ver ADR-0015**.
* **Supervisión Activa:** Latido de vida cada N segundos desde ejecución; botón de emergencia automático si drawdown excede límite crítico.

### Despliegue
* **Un solo binario ejecutable:** Binario nativo Rust compilado.
* **CI/CD:** Automatización (tests + validación de cambios de esquema).
* **Reversión:** Reversión de base de datos (control de cambios), reversión de código (historial de versiones).

---

