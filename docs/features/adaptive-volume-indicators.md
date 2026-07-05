# Adaptive and Volume Indicators

**Carpeta:** `./features/adaptive-volume-indicators.md`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0013 (Stack Tecnológico - Rust SIMD/Rayon)

## ¿Qué es esta feature?

Esta suite de indicadores avanzados se aleja de los promedios estáticos para adaptarse a la volatilidad y liquidez del mercado. Incluye indicadores de **volatilidad adaptativa** (KAMA, VIDYA) y métricas de **flujo de dinero** y **sentimiento de mercado** (Open Interest, Herrick Payoff Index).

## Comportamientos Observables

- [ ] Las medias móviles adaptativas se vuelven más rápidas en tendencias fuertes y más lentas (planas) en rangos.
- [ ] El **Herrick Payoff Index** mide la fuerza de la tendencia combinando precio, volumen e interés abierto.
- [ ] Detección de divergencias entre el precio y el flujo de dinero (Money Flow).

## Restricciones

- **NUNCA** recalcular indicadores sobre todo el historial si solo ha cambiado la última barra (uso de modo incremental vectorizado).
- **OBLIGATORIO:** Implementación en Rust SIMD/Rayon para permitir cálculos en tiempo real en miles de activos simultáneamente.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| KAMA_FAST | 2 | 2 - 30 | Ventana rápida para eficiencia | CONFIG |
| KAMA_SLOW | 30 | 10 - 200 | Ventanas lenta para eficiencia | CONFIG |
| VOLATILITY_THRESHOLD | 0.8 | 0.1 - 1.0 | Sensibilidad al cambio de régimen | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Funciones matemáticas de suavizado dinámico (Efficiency Ratio) y lógica de Herrick Payoff ($HPI = K \times (PriceDiff \times Vol \times (1 \pm \frac{2 \times OpenInt}{OpenInt_{prev}}))$).
- **Shell (Infraestructura):** Integración con el `backtest-engine` y el motor de señales `execute`.

## Ciclo de Vida de la Feature — Adaptive Indicators

### Entrada
- Datos OHLCV.
- Datos de Open Interest (si disponibles).

### Proceso
1. **Efficiency Calculation:** Determina si el mercado está tendencial o errático.
2. **Smoothing:** Ajusta el factor de suavizado dinámicamente.
3. **Volume Weighting:** Pesa el resultado según el volumen relativo.

### Salida
- Valores de indicadores en punto flotante (vectorizados).

### Contextos de Uso
- **Generate:** Como base para descubrir señales de alta calidad.
- **Execute:** Para detectar momentos de entrada con baja volatilidad/alto interés.

## Tareas (TTRs)

### TTR-001: Implementación de KAMA y VIDYA Vectorizados
- **Problema:** Los indicadores adaptativos tradicionales en Rust son lentos debido a los bucles recurrentes.
- **Qué tiene que pasar:** Escribir el algoritmo con Rust SIMD/Rayon usando recursividad optimizada.
- **Criterio de éxito:** Latencia de cálculo < 0.1ms para 100K barras.

### TTR-002: Motor de Interés Abierto y Herrick Payoff
- **Problema:** No todos los exchanges proveen Open Interest de la misma forma.
- **Qué tiene que pasar:** Crear una capa de abstracción para Open Interest y calcular el HPI.
- **Criterio de éxito:** Detectar divergencias de volumen 3-5 barras antes de un cambio de tendencia.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada cálculo de indicador porta el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del cálculo |
| | `created_at` | Timestamp de la barra de origen |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del feed |
| | `audit_chain_hash` | Hash de la secuencia de valores previos |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Dueño de la configuración técnica |
| | `institutional_tag` | Etiqueta de cumplimiento/entorno |
| | `manifest_id` | ID del diseño de la estrategia |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la fórmula matemática (JIT) |
| | `data_snapshot_id` | Puntero a los datos OHLCV base |
| | `indicator_state_hash` | Snapshot del estado interno (ER/HPI) |
| | `version_node_id` | Versión del genoma de indicadores |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del proceso de cálculo |
| | `execution_latency_ms` | Latencia de cálculo (microsegundos) |

## Gobernanza y Estándares (Fijos)
