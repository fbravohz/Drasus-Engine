## 6. Experiencia de Usuario y Flujos (Dashboard + Canvas)

> ⚠️ ADR-0028 (ZUI Fractal de 3 niveles nominados) fue **supersedido por ADR-0136** (2026-06-23). Los términos Fleet Command (Macro), Orchestrator (Meso) y Strategy Inspector (Micro) como denominaciones de pantalla están descontinuados. Ver ADR-0136 para la especificación completa.

El sistema se organiza en **dos superficies**:

1. **Dashboard:** Centro de monitoreo read-only. Bento grid de widgets arrastrables que muestran métricas, KPIs y estado del sistema. Cada elemento clickeable abre el Canvas en el contexto correspondiente. No hace zoom ni forma parte del Canvas.
2. **Canvas [Forge/Reactor — TBD]:** Lienzo nodal unificado con card-nodes estilo N8N / React Flow y puertos tipados (ADR-0137). Implementa layout automático (Dagre), validación estricta de aciclicidad en backend Rust (`petgraph`), y motor de invalidación reactiva de caché de validación. Tiene dos estados de zoom continuo:
   - **Vista Relacional:** nodos pequeños con conexiones visibles — conectar, reordenar, crear pipelines.
   - **Vista Interior:** zoom in-place en un nodo — editar lógica interna, Logic Blocks, parámetros.

   El Canvas expresa la jerarquía de entidades (Cluster → Portfolio → Strategy → Logic Blocks) mediante anidación in-place, con breadcrumb flotante. Los feature nodes (hojas atómicas) abren Inspector Panel lateral sin salir del canvas.

### 6.1 El "Happy Path" (Máxima Confianza)
`Dashboard` (Detección oportunidad) → `Canvas — Vista Relacional` (Construir / ejecutar Generate/Validate) → `Canvas — Inspector Panel` (Inspección robustez) → `Deploy` (Incubación; perfil configurable: Cuarentena 7 días / Extendido 21 días / Legacy 3-6 meses — ADR-0088) → `Live Trading`.

---

### 6.2 Puertos Públicos de Módulos

| Puerto | Módulo | Propósito | Operaciones |
|---|---|---|---|
| **DataPort** | ingest | Ingesta y consulta de datos de mercado | Descargar barras históricas, consultar régimen actual, suscribirse a datos en tiempo real |
| **StrategyPort** | generate | Generación y catálogo de estrategias | Generar candidatos vía NSGA-II, crear ramas de experimentación, listar estrategias |
| **ValidationPort** | validate | Pruebas estadísticas rigurosas | Ejecutar backtest, suite de validación (WFA/MC/CPCV), obtener veredicto final |
| **PaperTradingPort** | incubate | Ejecución forward sin dinero real | Iniciar sesión de paper trading, comparar con backtest (Pardo), obtener métricas |
| **PortfolioPort** | manage | Gestión de portafolios versionados | Crear portafolio, optimizar pesos (HRP), definir reglas, rebalanceo Walk-Forward, backtesting de portafolio |
| **ExecutionPort** | execute | Ejecución real y Veto Power | Colocar órdenes, cancelar órdenes, vetar decisiones, activar kill switch |
| **RetirementPort** | withdraw | Monitoreo de degradación | Evaluar salud de estrategia, retirar controladamente, reactivar |
| **FeedbackPort** | feedback | Control de Calidad y Cierre de Bucle | Detectar drift (Pardo), vetar estrategias degradadas, disparar refinamiento |

**Características Clave de los Puertos:**
- **Asincronía Transparente:** Las operaciones costosas (generación, backtests) retornan un `job_id` inmediatamente. El cliente monitorea el progreso consultando el puerto repetidamente o escuchando eventos de finalización.
- **Determinismo Garantizado:** Dentro de un Puerto, mismo input → mismo output, siempre. (Excepto operaciones que explícitamente muestrean aleatoriedad, donde la semilla es configurable.)
- **Sin Efectos Secundarios en Core:** Los Puertos pueden loguear, persistir, notificar. El Core no.
- **Inyectables para Testing:** Los Puertos pueden reemplazarse con implementaciones fake en tests. Ej: `BacktestEngine` puede ser un `FakeBacktestEngine` que devuelve resultados precalculados.

**Pipeline de Ejemplo: Flujo Completo a través de Puertos**
```
Cliente (FFI/gRPC)
    │
    ├─► DataPort.ingest(symbol="BTC", start=2020-01-01) 
    │      → "job_id=12345" (asincrónico)
    │
    ├─► StrategyPort.generate(method=NSGA2, pop_size=100)
    │      → "job_id=12346" (asincrónico)
    │
    ├─► ValidationPort.run_validation(strategy_id="S1", tests=[WFA, MC, CPCV])
    │      → "job_id=12347" (asincrónico)
    │      → Cuando finaliza: ValidationPort.get_verdict(strategy_id="S1") → APROBADA
    │
    ├─► PaperTradingPort.start_session(strategy_id="S1", profile=EXTENDED)   # perfil de incubación configurable (ADR-0088)
    │      → "session_id=sess_001" (sesión iniciada)
    │
    ├─► ExecutionPort.start_execution(portfolio_id="P1", broker="ibkr")
    │      → "execution_id=exec_001" (trading en vivo iniciado)
    │
    ├─► ExecutionPort.get_pending_decisions(execution_id="exec_001")
    │      → Lista de órdenes pendientes de veto del usuario
    │
    └─► RetirementPort.retire_strategy(strategy_id="S1", reason="drawdown > -40%")
           → Estrategia marcada RETIRADA en DAG de versiones
```
### 6.3 El Motor de Producción: Nautilus LiveNode

#### 6.3.1 Procesos Persistentes (Daemons)
En R&D se usan Workers efímeros que nacen, calculan y mueren. En Producción se necesitan **Procesos Persistentes (Daemons)**. El Core en Rust orquesta un hilo en segundo plano (Tokio task) dedicado exclusivamente a la ejecución en vivo y paper trading, inicializando el componente **LiveNode** de NautilusTrader.

- **Aislamiento de Entorno:** El proceso en vivo corre en su propio núcleo lógico mediante afinidad de CPU (*Core Pinning*), totalmente aislado de los Workers de R&D. Si se lanza una optimización genética masiva que consume el 99% del hardware, el sistema operativo garantiza que el núcleo reservado para el LiveNode mantenga latencia de microsegundos para ejecutar órdenes reales.
- **Componentes del LiveNode:** Conectividad nativa con brokers (Binance, IBKR, Oanda), loop de eventos determinista (Local-First) y gestión de órdenes mediante el FSM operativo descrito en [la sección 12](./SAD-12.md).
- **Reconstrucción de Inventario:** El Event Store (persistencia local en modo WAL) registra cada evento de ejecución, permitiendo reconstruir el estado del inventario tras un reinicio o caída del proceso.

#### 6.3.2 Multiplexación de Datos (El Bus Pub/Sub)
Si se abren 50 conexiones individuales con un mercado para 50 agentes en el mismo símbolo, la IP se banea instantáneamente. La solución:

1. **Conexión Única (Single Data Client):** El LiveNode levanta un solo cliente de datos hacia el mercado por símbolo.
2. **El Bus de Mensajes (Message Bus):** Los ticks y actualizaciones del Order Book llegan al Message Bus central de NautilusTrader, de altísimo rendimiento en memoria RAM (Cero-Copias).
3. **Suscripción de Agentes:** Cuando se "promueve" una estrategia a Producción, el motor lee su configuración inmutable (AST) y suscribe el agente pasivamente al bus.
4. **Distribución (Fan-out):** Un solo evento del mercado se distribuye por referencia en memoria a los 50 agentes en nanosegundos. Cero duplicación de red, cero clonación innecesaria de objetos.


**Pipeline de Ejecución (Happy Path):**
```
Datos del Mercado
    ↓
Módulo ingest: Ingesta barras ──► Guardar precios y régimen de mercado
    ↓
Módulo generate: Generar candidatos ──► Crear planos de estrategia (descubrimiento)
    ↓
Módulo validate: Validar estrategia ──► Suite completa de pruebas → APROBADA
    ↓
Módulo incubate: Ejecución paper trading ──► Test forward (perfil configurable: 7/21 días o 3-6 meses, ADR-0088) → PROMOVIDA
    ↓
Módulo manage: Optimizar portafolio ──► Combinar estrategias, establecer reglas
    ↓
Módulo execute: Colocar orden ──► Ejecutar en mercado vivo [validación <1ms; orden end-to-end ≤100ms]
    ↓
Módulo feedback: Veredicto Pardo ──► Control de calidad estadístico, decisión de retiro
    ↓
Módulo withdraw: Retiro controlado ──► Archivo definitivo, fin de ciclo ──► [Volver a Generar via Feedback]
```

### 6.4 El Frontend: Paradigma de Interfaz de Usuario (UI/UX) - Dashboards y Visualización

La interfaz gráfica de Drasus Engine se diseña sobre un paradigma responsivo y de ultra-bajo retardo visual:

*   **Entrega Progresiva por Feature — Cáscara Delgada (ADR-0117):** ninguno de los componentes listados abajo espera a EPIC-8. Cada uno se entrega progresivamente, como Cáscara Delgada, dentro de la misma Story que implementa su Feature correspondiente, acotado por un Techo Fijo: un control que dispara la operación real vía `public_interface`, una visualización del resultado real (FFI/gRPC) y un observable persistido visible tras recargar. Las Features sin superficie propia ("plomería") declaran su Ventana de Verificación en la Feature consumidora que sí tiene Cáscara Delgada. Todas las piezas viven en un único shell compartido — el Panel Operativo Fundacional (nacido en EPIC-0/SPIKE-006) — al que EPIC-8 aplica la integración Canvas [Forge/Reactor] (ADR-0136) y el pulido visual.
*   **Visualización High Precision (Impeller nativo):** Renderizado en GPU de alto rendimiento para interactuar con cientos de miles de puntos de datos sin congelamientos de UI. El lienzo se reserva para la representación de la topología del grafo.
*   **Micro-Backtest Node Preview:** Visualizador integrado en los nodos del Inspector Panel de una Strategy. Permite la visualización de curvas de equidad reducidas y métricas clave precargadas desde SQLite local, con invalidación visual y regeneración asíncrona manual ante la edición de parámetros.
*   **Time-Warp UI:** Selector de rango temporal y slider interactivo para navegación forense rápida. Realiza consultas optimizadas con DuckDB mediante partition pruning sobre el data lake en Parquet para minimizar la latencia de carga (<200ms) y la carga de memoria RAM en el frontend.
*   **UMAP Scatter Visualizer:** Scatter plot 2D/3D interactivo en Flutter nativo (CustomPainter/GPU) para identificar clústeres de robustez mediante reducción de dimensionalidad UMAP, con soporte de brushing (lasso) para drill-down directo de estrategias.
*   **Toxicity Purifier UI:** Panel interactivo para la purga masiva de clústeres de estrategias tóxicas detectadas por PCA. Permite previsualizar el impacto, realizar soft-delete por lote con firma en el log de auditoría y rollbacks mediante snapshots automatizados.
*   **Efficiency & Incubation Dashboard:** Monitoreo y control del Cono de Silencio y métricas clave de incubación.
*   **Data Manager UI:** Interfaz que incorpora el asistente de importación (Import Wizard) y el mapa cromático de calidad de datos.
*   **Throttling Metrics Dashboard:** Monitoreo en tiempo real de la latencia inducida por colas de brokers y conectores.
*   **Gráficos en PDF:** Motor de renderizado en el backend (headless) para la generación de reportes de auditoría y análisis inmutables.
*   **Monthly Performance Heatmap:** Matriz visual interactiva de Años × Meses segmentable por dirección (Long/Short) y tipo de muestra (IS/OOS).
*   **Trade Analysis BI Suite:** Dashboard de análisis de transacciones históricas (cascada P/L, Wins/Losses semanales y correlación duración vs rentabilidad).
*   **Strategy Config Diff:** Visor de diferencias entre la configuración del último test válido y la actual.
*   **AI Experience:** Incorporación de asistentes contextuales (Interactive Chat Loop, Hybrid Prompting UI, Natural Language Explanation) y auditorías regulatorias (Compliance Dashboard).
*   **Workflow Configurators:** Diseñador visual de pipelines automatizados (Visual Workflow Builder) y selectores dinámicos de universos accionarios (Visual StockPicker Configurator).

---

