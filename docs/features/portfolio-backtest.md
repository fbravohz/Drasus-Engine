# Portfolio Backtest

**Carpeta:** `./features/portfolio-backtest/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-03
**Decisión Arquitectónica Asociada:** ADR-0091 (Simulación de Portafolio Real)

---

## ¿Qué es esta feature?

El **Portfolio Backtest** es el componente de simulación avanzada encargado de evaluar el comportamiento histórico conjunto de múltiples estrategias de trading algorítmico operando simultáneamente sobre una única cuenta virtual consolidada.

*   **Problema que resuelve:** Los backtests tradicionales suman linealmente las curvas de rendimiento individuales asumiendo capital infinito e independencia operativa. Esto oculta riesgos sistémicos reales como llamadas de margen (Margin Calls), colisiones de margen cruzado y distorsiones por capitalización (compounding) mal calculadas.
*   **Comportamiento observable:** El usuario selecciona un conjunto de estrategias, define una política de capitalización y corre la simulación. El sistema ejecuta las órdenes concurrentemente respetando las limitaciones de margen y los horarios reales de negociación de cada activo.
*   **Por qué la necesitamos:** Provee paridad absoluta y rigor científico en la fase de validación de portafolios, garantizando que el rendimiento combinado sea reproducible bajo restricciones reales de cuenta.

---

## Comportamientos Observables

- [ ] **Simulación de Margen Cruzado:** El sistema calcula el margen requerido por cada estrategia activa y lo descuenta del balance común. Si el margen libre cae a cero, detona una llamada de margen (Margin Call) simulada y detiene/liquida la operativa del portafolio.
- [ ] **Interés Compuesto Configurable:** El operador puede habilitar la capitalización dinámica de ganancias, ajustando automáticamente el tamaño de posición de las estrategias según la frecuencia parametrizada.
- [ ] **Sincronización de Sesiones:** El motor de simulación suspende la generación de señales y la ejecución de órdenes de una estrategia cuando su exchange de origen está cerrado, respetando las diferencias horarias entre activos (ej. Forex 24/5 vs Acciones de Nueva York).

---

## Restricciones

- **NUNCA** se permite el cálculo de equidad agregada sin validar los requisitos de margen cruzado de todas las posiciones abiertas en el mismo instante de tiempo.
- **NUNCA** el motor de simulación puede mezclar o ignorar los husos horarios configurados para cada símbolo; el mapeo de horas de apertura y cierre es estricto y determinista.
- **NUNCA** se aplica interés compuesto dinámico sobre operaciones abiertas intra-período; el ajuste de capitalización se realiza estrictamente en el gate de cierre del período parametrizado.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| COMPOUNDING_FREQUENCY | monthly | none, daily, weekly, monthly | Frecuencia de ajuste del tamaño de posición por interés compuesto | CONFIG |
| MARGIN_REQUIREMENT_PCT | 10% | 1% - 100% | Porcentaje de margen requerido para sostener posiciones en simulación | CONFIG |
| CAPITAL_SHARING_MODE | cross_margin | cross_margin, isolated | Método de asignación de margen entre las estrategias | CONFIG |
| SESSION_SYNC_MODE | strict | strict, loose | Tolerancia de discrepancias en alineación temporal de velas | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

### Core (Lógica Pura)
*   **Calculador de Margen del Pool:** Determina el consumo de margen consolidado y el margen libre del balance común en cada tick o barra de la simulación.
*   **Ajustador de Compounding:** Modifica dinámicamente los lotes base de entrada de cada estrategia al inicio de cada período basándose en las ganancias netas retenidas.

### Shell (Infraestructura)
*   **Sincronizador de Reloj de Simulación:** Coordina el avance del tiempo en NautilusTrader para múltiples series de datos de mercado con husos horarios y calendarios heterogéneos.
*   **Persistencia de Historial Analítico:** Almacena los resultados consolidados de margen, balance y equidad en el formato Parquet particionado.

### Frontera Pública (Contrato)
*   `initialize_portfolio_backtest(strategies_list, initial_capital, config)`: Configura el pool y arranca el event-loop.
*   `apply_compounding(period_profit)`: Calcula y actualiza el capital base disponible.
*   `check_margin_limit(margin_intent)`: Valida la viabilidad de la orden propuesta contra el margen libre consolidado.

---

## Ciclo de Vida de la Feature — Portfolio Backtest

### Entrada
*   Lista de estrategias inmutables registradas en el Databank.
*   Feeds históricos de mercado (Parquet) alineados temporalmente.
*   Parámetros del pool de capital, margen y compounding.

### Proceso
*   Ejecuta concurrente y deterministamente el event-loop sobre las estrategias compartiendo balance.
*   Valida los requisitos de margen cruzado antes de emparejar cada orden simulada.
*   Ajusta dinámicamente los lotes según las reglas de capitalización del compounding.

### Salida
*   Curva de equidad consolidada y reporte detallado de uso de margen.
*   Registro inmutable de llamadas de margen y vetos de sesión.
*   Métricas de rendimiento agregadas del portafolio.

### Contextos de Uso

**Contexto 1: Gestión (Módulo Manage)**
*   Permite al optimizador correr backtests realistas sobre combinaciones candidatas para depurar y seleccionar la mejor topología antes de promover a live.

**Contexto 2: Validación (Módulo Validate)**
*   Forma parte del guantelete de pruebas de robustez pesado para descartar portafolios propensos a quiebras de cuenta por llamadas de margen.

---

## Tareas (TTRs)

### **TTR-001: Pool de capital compartido y modelo de margen en simulación**
*   **¿Cuál es el problema?** El sistema no puede simular con fidelidad llamadas de margen o apalancamiento cruzado de múltiples estrategias si asume capital infinito y aislado para cada una.
*   **¿Qué tiene que pasar?** El motor de simulación integra un Ledger consolidado que rastrea el margen requerido y el balance total del pool, rechazando o liquidando posiciones concurrentemente cuando el margen libre se reduce a cero.
*   **¿Cómo sé que está hecho?**
    - [ ] Pruebas unitarias demuestran que una posición apalancada perdedora en la Estrategia A gatilla una llamada de margen y cierra posiciones de la Estrategia B en el backtest.
    - [ ] La equidad final calculada coincide bit-a-bit con el saldo restante del ledger consolidado.
*   **¿Qué no puede pasar?**
    - No se permite que el margen libre tome valores negativos sin detonar la lógica de liquidación forzada.

### **TTR-002: Interés compuesto dinámico configurable**
*   **¿Cuál es el problema?** Los operadores necesitan evaluar el impacto de la capitalización periódica de ganancias sobre el riesgo y la volatilidad del portafolio.
*   **¿Qué tiene que pasar?** El motor calcula al cierre de cada período parametrizado (día, semana, mes) el PnL retenido y escala de forma geométrica el tamaño de lote asignado a las estrategias del portafolio para el siguiente ciclo.
*   **¿Cómo sé que está hecho?**
    - [ ] Con la capitalización activa, el tamaño de las posiciones de entrada aumenta progresivamente a medida que la curva de equidad asciende.
    - [ ] El reporte detalla los momentos de reajuste de compounding alineados con los cortes de calendario.
*   **¿Qué no puede pasar?**
    - No se permite aplicar compounding sobre operaciones en curso; el ajuste solo afecta a nuevas posiciones entrantes.

### **TTR-003: Sincronización de sesiones de mercado por exchange**
*   **¿Cuál es el problema?** Si una estrategia simula operaciones los domingos en un activo de futuros cerrados, el backtest generará falsos positivos imposibles de replicar en vivo.
*   **¿Qué tiene que pasar?** El motor de sincronización de reloj utiliza los calendarios y horarios de negociación configurados inmutablemente por exchange para suspender la aceptación de órdenes del agente correspondiente durante los periodos de mercado cerrado.
*   **¿Cómo sé que está hecho?**
    - [ ] Los logs de la simulación muestran que las señales emitidas fuera de horario de exchange quedan bloqueadas y registradas como "Veto de Sesión".
    - [ ] Se verifica la alineación exacta de eventos temporales al procesar activos en diferentes husos de manera simultánea.
*   **¿Qué no puede pasar?**
    - No se permite ejecutar transacciones en barras históricas correspondientes a horarios de inactividad oficial del exchange de origen.

---

## Gobernanza y Estándares (Fijos)

### Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda corrida de simulación de portafolio registra los metadatos de relevancia técnica para AI/R&D (Perfil IA / R&D):

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la simulación de portafolio |
| | `created_at` | Timestamp de inicio del backtest (nanosegundos) |
| | `audit_hash` | Hash SHA-256 de los parámetros y composición del backtest |
| | `audit_chain_hash` | Hash de enlace temporal con corridas anteriores |
| **II. Soberanía** | `owner_id` | Identificador del dueño de la IP/Estrategias |
| | `manifest_id` | ID del Design Manifest de calidad aplicado |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del binario del motor de simulación en ejecución |
| | `indicator_state_hash` | Hash de la matriz de ponderación y pesos del portafolio |
| **IV. Hardware** | `node_id` | ID único de la máquina local que ejecuta el backtest |
| | `process_id` | PID del worker asíncrono asignado |

*   **Rastro de Evidencia:** Emite el reporte de margin calls, uso de apalancamiento máximo y el porcentaje de degradación de equidad por compounding para alimentar la causalidad de `feedback`.

---

## Dependencias

**Depende de:**
*   [`backtest-engine`](../features/backtest-engine.md) — para la ejecución del event-loop de simulación.
*   [`portfolio-rules`](../features/portfolio-rules.md) — para el control de drawdowns globales.

**Consumido por:**
*   [`manage`](../modules/manage.md) — para la optimización y validación de topologías de portafolio.
