# Simulador Adversarial (Arena de Liquidez)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0031 (IA Híbrida), ADR-0032 (Hardware Soberano)

---

## ¿Qué es?

Es un motor de simulación alternativo que, en lugar de evaluar una estrategia contra datos históricos estáticos del mercado, crea y modela un **Libro de Órdenes (Order Book) dinámico sintético** desde cero. La estrategia del usuario se inyecta en este mercado simulado y compite de forma directa contra 50 agentes pre-entrenados (bots con perfiles de creadores de mercado, traders de momento, etc.) o estrategias de otros usuarios. El objetivo es comprobar la viabilidad y supervivencia de la estrategia en entornos con microestructura interactiva donde las órdenes propias modifican el spread y la liquidez del mercado.

---

## Comportamientos Observables

- [ ] **Generación del Entorno:** El sistema inicializa un Order Book simulado con parámetros configurables de liquidez inicial, volatilidad y profundidad del libro.
- [ ] **Agentes Competidores Concurrentes:** 50 bots automatizados con diferentes perfiles de trading (Market Makers, Momentum, Trend Followers, Arbitrageurs) envían órdenes límite y de mercado simuladas.
- [ ] **Interacción y Deslizamiento Realista:** Las órdenes de la estrategia del usuario afectan directamente la oferta y la demanda. Al enviar órdenes grandes, el spread se ensancha y el precio se desplaza, reflejando el impacto en el mercado en tiempo real.
- [ ] **Métricas de Supervivencia:** Visualización en vivo del saldo relativo y supervivencia de la estrategia en el ranking de la arena contra los 50 bots.

---

## Restricciones

- **OBLIGATORIO:** Mantener la ejecución paralela multihilo en CPU/GPU sin bloquear el trading en vivo del hot-path general.
- **NUNCA** asumir ejecuciones instantáneas (fills perfectos) si el volumen de las órdenes excede la profundidad simulada del Order Book.
- **FIJO:** Los agentes operan bajo reglas probabilísticas deterministas basadas en el estado del libro de órdenes actual para mantener la reproducibilidad de la simulación.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| AGENT_COUNT | 50 | 10 - 200 | Cantidad de bots activos en la simulación | CONFIG |
| ORDER_BOOK_DEPTH | 20 | 5 - 100 | Niveles visibles de profundidad en el libro de órdenes | CONFIG |
| ORDER_IMPACT_MULTIPLIER | 1.0 | 0.1 - 5.0 | Factor multiplicativo del impacto de cada orden en el precio | CONFIG |
| RETRY_MATCHING_ENGINE | true | true / false | Re-evaluación del matching engine en microsegundos | [FIJO] |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Lógica del matching engine simplificado en memoria y comportamiento algorítmico de los agentes competidores.
- **Shell (Infraestructura):** Orquestación multihilo utilizando Tokio/Rayon y persistencia de resultados de la simulación en formato Parquet.

---

## Ciclo de Vida de la Feature — Arena de Liquidez

### Entrada
- Estrategia candidata (JSON AST).
- Parámetros de inicialización de la arena (liquidez, profundidad, perfiles de agentes).

### Proceso
- Creación de la cola de eventos del Order Book.
- Simulación concurrente de ticks de negociación donde los bots reaccionan a los precios y a las órdenes del usuario.
- Emparejamiento de órdenes (matching engine) en tiempo de simulación discreto.

### Salida
- Curva de equidad de la estrategia con impacto de mercado realista.
- Registro detallado de transacciones, deslizamiento promedio y métricas de supervivencia en la arena.

---

## Tareas (TTRs)

### **TTR-001: Motor de Libro de Órdenes Local Simulado**
*   **¿Cuál es el problema?** El backtesting tradicional asume que las órdenes del usuario no cambian el precio del activo, lo cual es falso en entornos de baja liquidez o con lotajes altos.
*   **¿Qué tiene que pasar?** Diseñar un matching engine simplificado en memoria (Rust) que procese bid/ask spreads dinámicos. Las órdenes límite consumen liquidez en niveles y las órdenes de mercado mueven la cotización de acuerdo a la elasticidad de oferta/demanda configurada.
*   **¿Cómo sé que está hecho?**
    - [ ] Al lanzar una orden de compra por encima de la liquidez del Nivel 1, el precio medio de ejecución empeora (deslizamiento).
    - [ ] El libro de órdenes muestra el consumo de contratos por nivel en tiempo real.
*   **¿Qué no puede pasar?**
    - No puede haber fugas de memoria al simular millones de órdenes por minuto.

### **TTR-002: Orquestador de Agentes Concurrentes (Arena de Bots)**
*   **¿Cuál es el problema?** Para simular dinámicas reales de mercado se necesitan participantes activos que reaccionen y compitan por la liquidez.
*   **¿Qué tiene que pasar?** Crear un thread pool en Rust (Rayon) donde 50 hilos livianos evalúen reglas de trading de agentes sintéticos basados en la microestructura actual del libro de órdenes, insertando órdenes de forma competitiva.
*   **¿Cómo sé que está hecho?**
    - [ ] Se observa en los registros analíticos una distribución diversa de órdenes pertenecientes a los diferentes identificadores de bots.
    - [ ] La estrategia del usuario compite y su porcentaje de supervivencia fluctúa de acuerdo al rendimiento de sus oponentes.
*   **¿Qué no puede pasar?**
    - Los bots no deben tener acceso a las órdenes pendientes del usuario antes de que entren al libro (prohibido el front-running omnisciente, salvo que sea un bot diseñado con esa trampa para pruebas de estrés).

---

## Gobernanza y Estándares (ADR-0020 V2)

### Perfil IA / R&D
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de ejecución de simulación adversarial |
| | `created_at` | Timestamp de inicio |
| | `audit_hash` | Hash de la configuración inicial de la arena |
| | `audit_chain_hash` | Hash de integridad del resultado final de la simulación |
| **II. Soberanía** | `owner_id` | Usuario que ejecuta la simulación |
| | `institutional_tag` | Entorno de desarrollo/investigación |
| **III. Pesos/Modelos** | `logic_hash` | Hash del código de comportamiento de los bots |
| | `data_snapshot_id` | Identificador de semilla aleatoria del mercado |
| | `indicator_state_hash` | Hash de hiperparámetros de los agentes |
| **IV. Hardware** | `node_id` | ID del procesador o GPU local |
| | `process_id` | PID del worker de simulación |
| | `execution_latency_ms` | Duración del cómputo de simulación |
