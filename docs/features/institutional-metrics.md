# Institutional Metrics Suite

**Carpeta:** `./features/institutional-metrics/`
**Estado:** En Diseño
**Decisiones Arquitectónicas Asociadas:** ADR-0009, ADR-0020, ADR-0023, ADR-0047

---

## ¿Qué es? (Explicado Simple)

Es la "Calculadora Maestra" del sistema. Mide qué tan buena, mala o riesgosa es una estrategia. En lugar de calcular todo al mismo tiempo y trabar la computadora, usa dos motores separados (Computación Asimétrica - ADR-0047):
1. **El Motor Ferrari (Hot-Path):** Usa Rust (`NautilusTrader`) para calcular cosas básicas en milisegundos mientras la estrategia opera (ej. WinRate, Dinero Ganado/Perdido, MAE/MFE).
2. **El Laboratorio Pesado (R&D):** Usa el motor vectorizado Polars/Rust y Rust SIMD/Rayon para calcular las métricas matemáticas complejas (Sharpe, Monte Carlo, Probabilidad Z) *después* de que termina la simulación, sin afectar la velocidad de evolución genética.

También usa un **Selector Dinámico (ADR-0023)**: si solo pides 2 métricas, no calcula las otras 30.

---

## Narrativa de Producto

**Capacidad:** `CAP-02` — Un simulador que no miente

**Historia de usuario:** Como investigador de estrategias, quiero un conjunto de métricas de nivel institucional —no solo cuánto gané, sino cuánto riesgo corrí para ganarlo— calculadas sin que el cómputo pesado retrase la búsqueda de nuevas estrategias, para poder comparar candidatas con el mismo rigor que usaría un fondo profesional.

**El problema (técnico):** "gané dinero" no dice si la estrategia es buena. Dos estrategias con la misma ganancia final pueden tener perfiles de riesgo completamente distintos: una avanza en línea casi recta, la otra sube a trompicones con caídas violentas de por medio. Medir solo el resultado final y no el camino que tomó para llegar ahí esconde justo la información que decide si una estrategia es operable con dinero real. Y calcular decenas de métricas pesadas —simulaciones, probabilidades— en el mismo instante en que el motor evolutivo genera miles de candidatas por segundo frenaría la búsqueda a paso de tortuga.

**La solución (técnica):** dos motores separados. Uno veloz calcula en el mismo instante de la operación las métricas básicas (cuánto se ganó, cuántas operaciones ganadoras). Otro, más pesado, calcula después y en paralelo las métricas que exigen más cómputo (el ratio que mide retorno contra riesgo, el que solo penaliza las caídas, cuánto tiempo tardó la curva en recuperarse de su peor momento). Y un selector inteligente solo calcula las métricas que realmente se piden, en vez de las treinta completas cada vez.

**Explicado sin tecnicismos:** imagina que evalúas a dos corredores que terminaron una maratón en el mismo tiempo exacto. Uno mantuvo un ritmo parejo de principio a fin. El otro salió disparado, se quedó sin fuerzas a la mitad, casi se retira, y remontó al final con el último aliento. Llegaron igual, pero como apuesta para la próxima carrera, no son lo mismo ni de cerca.

Esta pieza es el cronómetro completo de la carrera, no solo el tiempo final: cuánto sufrió el corredor en el peor tramo, qué tan parejo fue su ritmo, cuánto tardó en recuperar el aliento después de un bajón. Con esa información completa puedes elegir de verdad a quién apostarle, en vez de fiarte solo del resultado final que, por sí solo, no cuenta la historia completa.

---

## El Muestrario de Métricas

Para evitar la "sobreingeniería", organizamos las métricas en capas lógicas:

1. **Lo Básico (Performance Base):** Win Rate, Beneficio Neto, Expectativa Matemática (cuánto ganas en promedio por trade).
2. **El Riesgo (Institucional):** Ratio de Sharpe (rendimiento vs riesgo), Sortino (penaliza solo el riesgo negativo), VaR (Valor en Riesgo).
3. **El Dolor (Drawdown):** Máximo Drawdown (cuánto dinero máximo perdiste desde el punto más alto), Tiempo bajo el agua (cuánto tardaste en recuperarte).
4. **La Calidad de la Curva:** Davey's Linearity Score (qué tan recta y estable es la curva de ganancias hacia arriba, buscando R² = 1.0).
5. **Microestructura (Respiración del Trade):** 
   - **MAE (Maximum Adverse Excursion):** Cuánto dinero estuviste perdiendo en rojo antes de ganar. Ayuda a ver si el Stop Loss está muy apretado.
   - **MFE (Maximum Favorable Excursion):** Cuánto dinero flotante llegaste a tener antes de que cayera. Ayuda a optimizar los Take Profit.

---

## Parámetros Configurables

| Parámetro | Default | Descripción |
|---|---|---|
| `risk_free_rate` | 0.0 | Tasa de ganancia de un bono seguro (para Sharpe/Sortino) |
| `sqn_min_trades` | 30 | Mínimo de operaciones necesarias para que las estadísticas no sean ruido o suerte |

---

## Tareas (TTRs)

### TTR-001: Motor Dual de Cálculo (El Secreto de la Velocidad)
- **Descripción:** Implementar la barrera técnica. Obligar a que `NautilusTrader` calcule las métricas transaccionales (PnL, WinRate) en su ledger interno (Hot-Path), y que el motor vectorizado Polars/Rust con Rust SIMD/Rayon calculen el Sharpe, Sortino y DD en bloques matriciales post-simulación (R&D).

### TTR-002: Selector Dinámico y Lazy Evaluation
- **Descripción:** Implementar el mecanismo que lee la solicitud de métricas de la interfaz y únicamente dispara las funciones matemáticas que corresponden a lo pedido.

### TTR-003: Extractor de Microestructura (MAE/MFE)
- **Descripción:** Extraer del ledger de operaciones cuánto "respiró" el trade (Drawdown interno de la operación) para alimentar futuros optimizadores de Stop Loss y Take Profit.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020):** **Perfil B (IA / R&D), híbrido B+latencia** (cálculo de métricas con extractor de microestructura sensible a latencia).

  | Categoría | Campo | Descripción |
  | :--- | :--- | :--- |
  | **I. Identidad** | `id` | Identificador único del cálculo de métricas |
  | | `created_at` | Timestamp del cálculo |
  | | `updated_at` | Timestamp de última modificación del registro |
  | | `audit_hash` | Hash de integridad del set de métricas |
  | | `audit_chain_hash` | Hash encadenado del historial |
  | | `event_sequence_id` | Secuencia de recuperación |
  | **II. Soberanía** | `owner_id` | Dueño de la configuración |
  | | `manifest_id` | Estrategia evaluada |
  | **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor dual de cálculo |
  | | `data_snapshot_id` | Snapshot de trades de origen (MAE/MFE) |
  | | `version_node_id` | Versión del muestrario de métricas |
  | **IV. Hardware** | `node_id` | ID del hardware físico |
  | | `process_id` | PID del proceso de cálculo |
  | **V. Auditoría & Ejecución (latencia, híbrido)** | `execution_latency_ms` | Latencia del extractor de microestructura |
- **Dependencias:** Consumido masivamente por los módulos `validate`, `manage` y `feedback`.
