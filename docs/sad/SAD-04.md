## 4. Vistas del Sistema (Modelo C4)

### 4.1 Nivel 1: Contexto
```
    ┌───────────────────────────┐   ┌───────────────────────────┐
    │       Flutter UI          │   │  Agente LLM (Claude, etc.) │
    │ (Dart + Impeller Engine)  │   │  Cliente MCP — Cabina Dual │
    └────────┬──────────────────┘   └────────┬──────────────────┘
             │ (Local: FFI / Remoto: gRPC)    │ (MCP, vía Agentic
             │                                │  Gateway — ADR-0123)
             └───────────────┬────────────────┘
    ┌────────────────────────▼──────────┐      ┌─────────────────────────┐
    │   Drasus Engine Backend   │◄────►│       Brokers           │
    │        (Rust Core)        │ API/ │  (Binance, Interactive  │
    │   [broker-connector]      │  WS  │   Brokers, etc.)        │
    └────────┬──────────────────┘      └─────────────────────────┘
             │
    ┌────────▼──────────────────┐
    │      SQLite Local         │
    │   (Historial, States)     │
    └───────────────────────────┘
```

**Tercer cliente — Agente LLM vía MCP (Cabina Dual, ADR-0123):** igual que Flutter, es un Shell-cliente más sobre la misma `public_interface`. No reemplaza al cliente humano: el usuario decide en cada momento si opera la interfaz o delega en el agente. Sus permisos por defecto se gradúan por riesgo de pipeline (abiertos en descubrimiento/simulación, bloqueados en producción real salvo activación explícita) — el detalle vive en [`agentic-mcp-gateway.md`](../features/agentic-mcp-gateway.md).

### 4.2 Nivel 2: Contenedores (Features Hexagonales + Presets de Pipeline)

El sistema organiza las features como crates hexagonales independientes (ADR-0137), agrupadas por dominio. El pipeline recomendado es: **Ingestar → Generar → Validar → Incubar → Gestionar → Ejecutar → Retroalimentar → Retirar**.

**Estructura de Carpetas (workspace Cargo):**
```
raíz del proyecto
├── crates/
│   ├── shared/                     # Tipos ADR-0137 (109 tipos) + plumbing cross-cutting
│   │   └── src/types/              # Catálogo canónico: Bars, Signal, Order...
│   ├── features/                   # Un crate por feature — cada uno es un hexágono
│   │   ├── data/                   # Dominio: ingesta y preparación de datos
│   │   ├── generation/             # Dominio: generación de estrategias
│   │   ├── validation/             # Dominio: backtest, robustez, scoring
│   │   ├── execution/              # Dominio: órdenes, broker, kill-switch
│   │   ├── portfolio/              # Dominio: optimización, correlación, rebalanceo
│   │   ├── lifecycle/              # Dominio: incubación, monitoreo, archivo
│   │   ├── infrastructure/         # Dominio: plumbing (clock, audit, telemetry)
│   │   └── _TEMPLATE/              # Plantilla canónica de feature crate
│   ├── presets/                    # Crates de cableado — CERO lógica
│   │   └── standard-pipeline/      # Preset recomendado (los 8 pasos)
│   ├── app/                        # Binario + entry point
│   ├── bridge/                     # FFI flutter_rust_bridge
│   └── nautilus_compat/            # Anticorruption layer (NautilusTrader)
└── migrations/                     # Migraciones SQLx centralizadas (ADR-0006)
```

**Arquitectura de Feature Crate (cada feature implementa esta estructura):**
```
crates/features/<dominio>/<feature>/
├── Cargo.toml              # ÚNICA dependencia: shared
├── src/
│   ├── lib.rs              # Declara módulos; solo public_interface es pub
│   ├── public_interface.rs # [Shell] Puertos tipados InputPorts/OutputPorts (ADR-0137)
│   ├── domain/             # [Core] Lógica pura, sin I/O
│   ├── orchestrator.rs     # [Shell] Implementación concreta de puertos
│   ├── persistence/        # [Shell] Acceso a datos (solo si aplica)
│   └── schemas.rs          # Modelos de datos / contratos
```

**Árbol Visual del Sistema C4 Nivel 2 (hexagonal — ADR-0137):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│                    shared: Tipos ADR-0137 + plumbing                        │
│  (109 tipos de puerto: Bars, Signal, Order, BacktestResult...)             │
│  + Clock, JobExecutor, AuditLog, Telemetry, MCP Gateway                    │
└────────────────────────────────────────────────────────────────────────────┘
       ▲                        ▲                        ▲
       │ (solo shared)          │ (solo shared)          │ (solo shared)
       │                        │                        │
┌──────┴──────────┐   ┌─────────┴──────────┐   ┌────────┴──────────┐
│ Feature crate   │   │ Feature crate      │   │ Feature crate     │
│ backtest-engine │   │ monte-carlo-sim    │   │ wfa-analyzer      │
│ ┌─────────────┐ │   │ ┌───────────────┐  │   │ ┌──────────────┐  │
│ │InputPorts   │ │   │ │InputPorts     │  │   │ │InputPorts    │  │
│ │ Bars        │ │   │ │ BacktestResult│  │   │ │BacktestResult│  │
│ │ Exec.Cont.  │ │   │ │ Exec.Container│  │   │ │Exec.Container│  │
│ └─────────────┘ │   │ └───────────────┘  │   │ └──────────────┘  │
│ ┌─────────────┐ │   │ ┌───────────────┐  │   │ ┌──────────────┐  │
│ │OutputPorts  │ │   │ │OutputPorts    │  │   │ │OutputPorts   │  │
│ │BacktestRes. │ │   │ │MonteCarloRes. │  │   │ │WFAMatrix      │  │
│ │EquityCurve  │ │   │ │               │  │   │ │               │  │
│ └─────────────┘ │   │ └───────────────┘  │   │ └──────────────┘  │
└─────────────────┘   └───────────────────┘   └───────────────────┘

        ┌──────────────────────────────────────────────────────────┐
        │            preset: standard-pipeline                     │
        │  (CERO lógica — solo cablea features en orden canónico)  │
        │  depende de: backtest-engine, monte-carlo-sim, wfa...    │
        └─────────────────────┬────────────────────────────────────┘
                              │
        ┌─────────────────────┴────────────────────────────────────┐
        │                    app (binario)                          │
        │  CLI + entry point + presets de wiring                    │
        └──────────────────────────────────────────────────────────┘
```
Cada feature crate es independiente — compila y se testea aislada. El preset `standard-pipeline` agrupa features en el orden recomendado del pipeline sin añadir lógica propia. Los módulos como dueños runtime no existen: las features se conectan directamente en el Canvas [Forge/Reactor] por sus puertos tipados.

---

