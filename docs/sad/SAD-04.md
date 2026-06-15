## 4. Vistas del Sistema (Modelo C4)

### 4.1 Nivel 1: Contexto
```
    ┌───────────────────────────┐
    │       Flutter UI          │
    │ (Dart + Impeller Engine)  │
    └────────┬──────────────────┘
             │ (Local: FFI / Remoto: gRPC)
    ┌────────▼──────────────────┐      ┌─────────────────────────┐
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

### 4.2 Nivel 2: Contenedores (8 Módulos de Pipeline + Features Reutilizables)

El sistema sigue un pipeline claro: **Ingestar → Generar → Validar → Incubar → Gestionar → Ejecutar → Retroalimentar → Retirar**.

**Estructura de Carpetas:**
```
Directorio raíz del proyecto
├── Archivo principal (Orquestación de módulos)
├── Carpeta shared (Features reutilizables: telemetría, tipos, utilidades)
│   ├── telemetría/ (Registro estructurado, métricas)
│   ├── tipos/ (Estados máquina de 64 bits, Enumeraciones, tipos base)
│   └── utilidades/ (Conversión de datos, serialización, ayudas de tiempo)
├── Carpeta modules (8 módulos con separación clara)
│   ├── ingest/ (Separación clara: API pública, lógica pura, orquestación, acceso datos, modelos DB, esquemas)
│   ├── generate/
│   ├── validate/
│   ├── incubate/
│   ├── manage/
│   ├── execute/
│   ├── withdraw/
│   └── feedback/
└── Carpeta infrastructure
    ├── Configuración base de datos (Mapeo de objetos a SQL + SQLite)
    └── Bus de eventos (Colas asincrónicas para comunicación entre módulos)

Carpeta migraciones (Control de cambios de esquema centralizado)
Carpeta tests (Pruebas unitarias, integración, simulación histórica)
```

**Arquitectura de Módulos (Cada módulo contiene):**
```
Carpeta módulo/
├── mod.rs
├── public_interface.rs  <-- [SHELL] Única entrada que otros módulos ven (API Interna)
├── domain/              <-- [CORE] Lógica pura (Business Logic), sin efectos secundarios
│   └── logic.rs
├── orchestrator.rs      <-- [SHELL] Manejo de flujo, estados, ruteo de eventos
├── persistence/         <-- [SHELL] Acceso a datos (solo tablas del módulo)
│   ├── models.rs        <-- Esquema de base de datos relacional (tablas locales)
│   └── repository.rs    <-- Consultas y conversión Core <-> DB
└── schemas.rs           <-- Modelos de datos (Estructuras / Contratos)
```

**Árbol Visual del Sistema C4 Nivel 2:**
```
┌────────────────────────────────────────────────────────────────────────────┐
│                    Archivo principal: Orquestación                          │
└────────────────────────────────────────────────────────────────────────────┘
       │
       ├─► Carpeta shared (Features Reutilizables)
       │    ├── Telemetría/ (Registro, métricas)
       │    ├── Tipos/ (Máquina de estados 64-bit, Enumeraciones)
       │    └── Utilidades/ (Conversión de datos, serialización)
       │
       ├─► Carpeta módulo-ingest
       │    ├── API pública: Ingesta de barras, obtener régimen de mercado
       │    ├── Lógica pura: Parsing de precios, detección de anomalías
       │    ├── Orquestación: Manejo gRPC/WebSocket, normalización
       │    ├── Acceso datos: Persistencia de barras, detección de régimen
       │    └── Modelos DB: Tablas barras, histórico régimen
       │
       ├─► Carpeta módulo-generate
       │    ├── API pública: Generar candidatos, evaluar aptitud
       │    ├── Lógica pura: Evolución genética, regresión simbólica
       │    ├── Orquestación: Bucle evolutivo, combinación de señales
       │    ├── Acceso datos: Persistencia estrategias, análisis de factores
       │    └── Modelos DB: Tablas planos estrategia, candidatos
       │
       ├─► Carpeta módulo-validate
       │    ├── API pública: Validar estrategia, suite de pruebas
       │    ├── Lógica pura: Análisis walk-forward, Monte Carlo, pruebas de coherencia
       │    ├── Orquestación: Orquestación backtesting, cálculo métricas
       │    ├── Acceso datos: Motor pruebas, resultados validación
       │    └── Modelos DB: Tablas resultados pruebas, métricas
       │
       ├─► Carpeta módulo-incubate
       │    ├── API pública: Ejecución paper trading, comparación con backtest
       │    ├── Lógica pura: Validación Pardo
       │    ├── Orquestación: Simulación de ejecuciones, detección cambios
       │    ├── Acceso datos: Persistencia paper trading
       │    └── Modelos DB: Tablas sesiones, resultados comparación
       │
       ├─► Carpeta módulo-manage
       │    ├── API pública: Optimizar portafolio, establecer reglas, backtesting de portafolio HRP
       │    ├── Lógica pura: Optimización portafolio (HRP), correlaciones, rebalanceo Walk-Forward
       │    ├── Orquestación: Rebalanceo, cálculo correlaciones
       │    ├── Acceso datos: Persistencia portafolio, estrategias
       │    └── Modelos DB: Tablas portafolios, pesos, reglas
       │
       ├─► Carpeta módulo-execute
       │    ├── API pública: Colocar orden, cancelar orden, veto
       │    ├── Lógica pura: Cambios de estado orden (máquina 64-bit)
       │    ├── Orquestación: Conexión broker, 10 validaciones pre-comercio (ADR-0025)
       │    ├── Acceso datos: Persistencia órdenes, posiciones
       │    └── Modelos DB: Tablas órdenes, ejecuciones, eventos supervisión
       │
       ├─► Carpeta módulo-feedback
       │    ├── API pública: Control de Calidad Estadístico (Pardo), Veredicto de salud
       │    ├── Lógica pura: Detección de Drift (Real vs Esperado)
       │    ├── Orquestación: Cierre de ciclo de vida (Veredicto de retiro)
       │    ├── Acceso datos: Historial de veredictos, constraints de aprendizaje
       │    └── Modelos DB: Tablas anomalías, sugerencias, veredictos
       │
       ├─► Carpeta módulo-withdraw
       │    ├── API pública: Detectar degradación, retiro estrategia
       │    ├── Lógica pura: Comparación de perfiles de rendimiento
       │    ├── Orquestación: Flujo retiro controlado, gestión de veto
       │    ├── Acceso datos: Persistencia de estrategias archivadas
       │    └── Modelos DB: Tablas registro retiro, estrategias archivadas
       │
       └─► Carpeta infrastructure
            ├── Configuración base de datos: Mapeo de objetos + SQLite
            └── Bus de eventos: Colas asincrónicas inter-módulos
```

---

