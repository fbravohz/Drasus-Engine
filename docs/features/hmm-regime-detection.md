# HMM Regime Detection — Detección de Régimen de Mercado

**Carpeta:** `./features/hmm-regime-detection/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0013 (Stack Tecnológico), ADR-0015 (Causalidad), ADR-0108, ADR-0110

## ¿Qué es?

El motor de detección de regímenes utiliza **Modelos Ocultos de Markov (HMM)** y modelos **ARIMA** para clasificar el entorno macro y micro-estructural del mercado. Funciona como un orquestador que decide qué tipo de lógica es apta para el escenario actual, protegiendo al sistema de operar en condiciones de baja probabilidad estadística o falta de liquidez.

### Capacidades:
1.  **[OLD-SCHOOL] Regime Detector (HMM):** Clasificación en 4 estados (Tendencia, Rango, Volátil, Calmo). Actúa como un **Enrutador (Router)** que desvía el flujo de órdenes hacia estrategias de Momentum o Mean Reversion basándose en el historial de precios. Etiqueta barras de mercado indexadas con el valor `regime_label` para habilitar el enrutamiento y las pruebas adaptativas.
2.  **[NEW-ERA] Filtro Dinámico L1:** Decodificación de las transacciones crudas Nivel 1 (Bid-Ask). Valida si el vector actual tiene similitud dimensional con entornos de éxito previos sin depender de indicadores técnicos retrasados (SMAs).
3.  **[NEW-ERA] Predicción de Estancamiento ARIMA:** Diagnostica anomalías de volatilidad futura. Detecta escenarios de **Escasez de Liquidez** o **Rango Asintótico** (>85%) para desautorizar operaciones y ahorrar costes de ejecución.

**Gen de Condición de Estado del Genoma de Régimen y Filtro de Entorno (ADR-0108/ADR-0110):** el `regime_label` (estado dominante HMM) es uno de los cuatro Genes de Condición de Estado del Dominio de Régimen y Filtro de Entorno (junto con el exponente de Hurst, la entropía de Shannon del volumen y las pendientes multinivel de Hull MA, descritos en ADR-0110). Cuando ese genoma está activo, el motor evolutivo puede condicionar la máscara binaria Permitido/Prohibido sobre el Genoma de Señal congelado al valor de `regime_label` de la barra actual.

## Narrativa de Producto

**Capacidad:** `CAP-03` — Búsqueda automática en el espacio de estrategias

**Historia de usuario:** Como investigador de estrategias, quiero que el motor reconozca en qué tipo de mercado está operando —tendencia, rango, volátil, calmo— y ajuste o vete la señal en consecuencia, para no dejar que una estrategia de tendencia opere ciegamente en un mercado plano donde está condenada a perder por comisiones.

**El problema (técnico):** una estrategia diseñada para un tipo de mercado suele operar igual sin importar el entorno real: sigue buscando tendencia en un mercado lateral, o mide con indicadores retrasados un cambio que ya ocurrió en el flujo de órdenes crudo. El costo no es solo una mala operación: es seguir intentando entrar en un mercado sin liquidez ni dirección, pagando comisión tras comisión sin ninguna ventaja real.

**La solución (técnica):** un clasificador —Modelos Ocultos de Markov y un modelo de series de tiempo ARIMA— que etiqueta cada barra con el régimen dominante del mercado (tendencia, rango, volátil, calmo), decodifica el flujo crudo de compra/venta sin depender de indicadores retrasados, y veta nuevas entradas cuando detecta escasez de liquidez o un rango que no va a ningún lado. La etiqueta de régimen también queda disponible como una condición más que el motor de búsqueda puede usar al construir estrategias, no solo como enrutador automático.

**Explicado sin tecnicismos:** es como un instructor de manejo que reconoce si vas por autopista o por un terreno lleno de baches, y te avisa a tiempo cuál de las dos formas de manejar te conviene ahora, en vez de dejarte usar siempre el mismo estilo sin importar el camino.

Un mercado en tendencia se maneja distinto de un mercado sin dirección. Manejar igual en los dos es la forma más común de perder dinero en comisiones persiguiendo un movimiento que nunca llega. Esta pieza detecta en qué "camino" está el mercado ahora mismo y ajusta el comportamiento de la estrategia, o de plano le dice que se detenga si el terreno se puso imposible de cruzar con ventaja.

---

## Comportamientos Observables

- [ ] **Validación L1:** Si el vector de flujo de órdenes L1 no coincide con el régimen entrenado, la señal se cancela incluso si el indicador técnico es positivo.
- [ ] **Auto-Veto por Estancamiento:** Si ARIMA predice un rango asintótico, el sistema bloquea nuevas entradas y notifica "Mercado Asintótico detectado (Ahorro de Comisiones)".
- [ ] **Enrutamiento Dinámico:** Cambia el conjunto de parámetros de la estrategia (weights) al detectar un cambio de régimen (ej. de Calmo a Volátil) en < 100ms.
- [ ] **Etiquetado de Barras:** Cada barra del dataset histórico/en vivo es enriquecida con una columna `regime_label` (entero que representa el régimen HMM activo) que sirve para segmentar los sub-backtests.
- [ ] Cuando el Genoma de Régimen y Filtro de Entorno (ADR-0110) está activo, `regime_label` se expone como Gen de Condición de Estado evaluable por el motor evolutivo en cada barra, además de su uso como router OLD-SCHOOL.

## Restricciones

- **NUNCA** operar si el modelo ARIMA predice escasez líquida superior al umbral configurado.
- Los modelos HMM requieren un re-entrenamiento (Walk-Forward) periódico para evitar la degradación del modelo ante cambios macro.
- La latencia total de clasificación no debe exceder los 20ms en el hot-path.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| STAGNATION_THRESHOLD | 0.85 | 0.70-0.95 | Probabilidad de ARIMA para activar el veto |
| L1_SIMILARITY_MIN | 0.70 | 0.50-0.90 | Mínima similitud requerida en el filtro L1 |
| REGIME_TRANSITION_LAG | 3 | 0 - 10 | Barras de gracia antes de confirmar cambio de régimen |

## Tareas (TTRs)

### **TTR-001: Implementación del Filtro Dinámico L1**
*   **Problema:** El precio es un indicador retrasado.
*   **Qué tiene que pasar:** Programar un decodificador que compare el flujo de Bid/Ask actual contra el kernel del modelo HMM.
*   **Criterio de Éxito:** Detección de cambios de régimen 2 baras antes que un ADX tradicional.

### **TTR-002: Predictor de Estancamiento ARIMA**
*   **Problema:** Operar en rangos pequeños destruye el capital por comisiones.
*   **Qué tiene que pasar:** Integrar una regresión deductiva ARIMA que evalúe la volatilidad pasiva futura.
*   **Criterio de Éxito:** Reducción del 20% en trades perdedores en zonas de baja volatilidad.

### **TTR-003: Ventanas WFA Adaptativas por Régimen de Volatilidad (HMM Labeler)**
*   **Problema:** Las ventanas fijas no se adaptan a las transiciones rápidas de volatilidad del mercado.
*   **Qué tiene que pasar:** Desarrollar un etiquetador de barras en Rust que, basándose en la inferencia del modelo HMM, añada la columna `regime_label` a cada registro en el dataset. El Walk-Forward Analyzer leerá esta columna `regime_label` para configurar dinámicamente el tamaño de las ventanas In-Sample y Out-of-Sample.
*   **Criterio de Éxito:** Las ventanas IS/OOS se ajustan de manera fluida y adaptativa basándose en los cambios del tag de régimen sin desbordamientos de rango ni errores de índice.

### **TTR-004: Zero-Code HMM Router Node**
*   **Problema:** El usuario requiere condicionar visualmente diferentes lógicas de señal según el estado de mercado actual sin programar.
*   **Qué tiene que pasar:** Crear una representación de nodo visual para el clasificador HMM que exponga cuatro pines lógicos de salida (Tendencia Alcista, Bajista, Rango Volátil, Rango Calmo) para desviar dinámicamente señales en el DAG visual.

### **TTR-005: Exposición del Estado Dominante HMM como Gen de Condición del Genoma de Régimen y Filtro (ADR-0108/ADR-0110)**
*   **¿Cuál es el problema?** El Dominio de Régimen y Filtro de Entorno necesita leer `regime_label` como uno de sus Genes de Condición de Estado para componer la máscara Permitido/Prohibido aplicada al Genoma de Señal congelado, sin depender del enrutamiento OLD-SCHOOL existente.
*   **¿Qué tiene que pasar?** `regime_label` debe quedar disponible como entrada de solo lectura para el motor evolutivo cuando `ACTIVE_GENOME_DOMAINS` incluye Régimen y Filtro de Entorno, en paralelo a su uso como router de estrategias Momentum/Mean Reversion.
*   **¿Cómo sé que está hecho?**
    - [ ] Un Manifest con Genoma de Régimen y Filtro activo puede componer una condición que combine `regime_label` con el exponente de Hurst de la misma barra.
    - [ ] El enrutamiento OLD-SCHOOL basado en `regime_label` sigue funcionando sin cambios cuando ese genoma no está activo.
*   **¿Qué no puede pasar?** El motor evolutivo no puede escribir o alterar `regime_label`; solo puede leerlo como Gen de Condición.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 · Perfil B IA/R&D, híbrido B+latencia)

Híbrido: Perfil B (IA/R&D, lleva linaje III) + latencia V legítima (clasificación ≤20ms en hot-path).

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la inferencia de régimen |
| | `created_at` | Timestamp del evento de mercado |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del resultado combinado |
| | `audit_chain_hash` | Hash encadenado de la secuencia de inferencias |
| | `event_sequence_id` | Secuencia de recuperación de la inferencia |
| **II. Soberanía** | `owner_id` | Dueño del proceso de clasificación |
| **III. Linaje** | `logic_hash` | Hash de los pesos del modelo HMM y ARIMA |
| | `data_snapshot_id` | Referencia a las últimas 1000 transacciones L1 Crudas |
| | `version_node_id` | Versión del modelo en el DAG |
| | `parent_id` | ID del Alpha Blueprint orquestador (linaje jerárquico) |
| **IV. Hardware** | `node_id` | ID del hardware físico ejecutor |
| | `process_id` | PID del proceso de clasificación |
| **V. Auditoría & Ejecución (latencia, híbrido)** | `execution_latency_ms` | Latencia de decodificación L1 (Hot-path ≤20ms) |
| | `compliance_status_id` | Veredicto de Liquidez/Estancamiento (Veto ARIMA) |
| | `indicator_state_hash` | Vector de estados (α, β) del filtro dinámico |

## Gobernanza y Estándares (Fijos)

- **Genomas Modulares por Dominio (ADR-0108/ADR-0110):** `regime_label` es un Gen de Condición de Estado del Dominio de Régimen y Filtro de Entorno. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
