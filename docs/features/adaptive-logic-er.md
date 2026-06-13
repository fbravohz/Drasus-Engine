# Adaptive Logic (Efficiency Ratio)

**Carpeta:** `./features/adaptive-logic-er/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020 V2

---

## ¿Qué es?

El Adaptive Logic basado en el Efficiency Ratio (ER) de Kaufman es un filtro de calidad de la señal. Su objetivo es asegurar que el Alpha detectado explote una ineficiencia real del mercado y no sea simplemente ruido aleatorio. El ER mide la "rectitud" de una tendencia comparando el movimiento neto con la suma de todos los movimientos absolutos.

**Problema que resuelve:** Muchas estrategias entran al mercado en zonas de "ruido" o volatilidad sin dirección, lo que genera señales falsas. Este componente filtra esas señales permitiendo operar solo cuando el mercado muestra una eficiencia direccional clara o, por el contrario, adaptando la sensibilidad del indicador a la falta de ella.

---

## Comportamientos Observables

- [ ] El sistema calcula el Efficiency Ratio (ER) de Kaufman para la ventana temporal de la señal.
- [ ] Si `ER < umbral_minimo`, el sistema bloquea la señal por considerarla "ruido estocástico".
- [ ] El sistema puede adaptar dinámicamente los parámetros de otros indicadores basándose en el ER (lógica adaptativa).
- [ ] En el reporte de validación, se muestra el "Average ER per Trade" para certificar que la estrategia opera en condiciones de mercado reales.

---

## Restricciones

- **FIJO:** El cálculo del ER debe ser vectorizado para no penalizar la latencia del backtest.
- Un ER de 1.0 representa una línea recta perfecta; un ER cercano a 0 representa ruido puro. El sistema nunca aceptará señales con ER < 0.1 por defecto.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| KAUFMAN_ER_PERIOD | 10 | 5 - 50 | Ventana de barras para calcular la eficiencia. | CONFIG |
| MIN_ER_THRESHOLD | 0.3 | 0.1 - 0.8 | Umbral mínimo para permitir la ejecución de una señal. | CONFIG |

---

## Ciclo de Vida de la Feature — Adaptive Logic (ER)

### Entrada
- Serie de precios (OHLCV).
- Señales generadas por la estrategia.

### Proceso
- Calcula la diferencia entre el precio actual y el precio de hace N barras.
- Calcula la suma de los cambios absolutos barra a barra en esa misma ventana.
- Divide el movimiento neto por el movimiento total para obtener el ER.

### Salida
- `efficiency_ratio_score`.
- Estado de bloqueo (VETO / ALLOW).

---

## Tareas (TTRs)

### **TTR-001: Calculador Vectorizado de Efficiency Ratio**
*   **¿Cuál es el problema?** El cálculo barra a barra en Rust es lento.
*   **¿Qué tiene que pasar?** Implementar el cálculo usando Polars/Rust SIMD-Rayon para procesar millones de barras en milisegundos.
*   **¿Cómo sé que está hecho?**
    - [ ] El test de velocidad muestra < 1ms para 100,000 barras.
    - [ ] Los valores coinciden con el estándar de Kaufman.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
