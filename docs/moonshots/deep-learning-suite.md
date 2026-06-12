# Deep Learning Suite

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0031 (IA Híbrida), ADR-0032 (Hardware Soberano)

## ¿Qué es esta feature?

La **Suite de Aprendizaje Profundo** proporciona los bloques de construcción para modelos predictivos y de toma de decisiones avanzados. Incluye arquitecturas de secuencia (LSTM, Transformadores) para predicción de precios, agentes de Aprendizaje por Refuerzo Profundo (DRL) para la gestión de portafolio, optimización de arquitectura neuronal (DARTS) y un canal de comunicación seguro para APIs externas (Cloud LLM Gateway).

## Comportamientos Observables

- [ ] **DRL Genesis:** Entrenamiento de agentes (PPO/DQN) para descubrir regímenes de recompensa autónomos (No-Template).
- [ ] Inferencia en tiempo real (< 50ms) para predicción de próximo tick/barra.
- [ ] **Sinergia Híbrida:** Delegación de la "Tesis de Alpha" al motor genético para el ajuste de umbrales operativos.
- [ ] Optimización de políticas de ejecución (PPO) mediante agentes que aprenden de la fricción del mercado.
- [ ] **Desvío LLM Seguro:** El sistema enruta las solicitudes que requieren razonamiento complejo de LLMs a modelos externos (GPT-4o) a través del Cloud LLM Gateway únicamente si el usuario aprueba explícitamente el uso de tokens remotos.

## Restricciones

- **OBLIGATORIO:** Respetar los límites de VRAM definidos en ADR-0032 (Fallback automático a CPU/Rust SIMD-Rayon si > 6GB).
- **NUNCA** usar modelos de caja negra para ejecución final sin una capa de validación simbólica o de lógica difusa que explique la decisión (IA Explicable).
- **NUNCA** enviar código de estrategia o datos de transacciones reales sin cifrado y anonimización previa a través del Cloud LLM Gateway.
- Los modelos deben ser serializables a formatos portátiles (ONNX/TorchScript) para garantizar la soberanía de ejecución.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| DL_BATCH_SIZE | 1024 | 128 - 4096 | Tamaño de lote para entrenamiento/inferencia | CONFIG |
| USE_CUDA | True | True/False | Forzar o deshabilitar aceleración por GPU | CONFIG |
| FALLBACK_TO_CPU | True | [FIJO] | Activa uso de Rust SIMD-Rayon/CPU si falla la GPU | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Definición de grafos en `burn`/`candle` (Rust puro, sin libtorch — ADR-0112), funciones de pérdida (Loss) especializadas en finanzas (ej: Sharpe Loss).
- **Shell (Infraestructura):** Manejador de memoria GPU, carga de datos via Apache Arrow, orquestador de entrenamiento asíncrono.

## Ciclo de Vida de la Feature — Deep Learning

### Entrada
- Tensores de datos normalizados (escala robusta vs outliers).
- Hiperparámetros de arquitectura (layers, heads, attention).

### Proceso
1. **Forward Pass:** Predicción según el estado actual del mercado.
2. **Backpropagation:** Actualización de pesos según error cometido.
3. **Architecture Search:** (Si DARTS activo) Selección de la mejor ruta neuronal.

### Salida
- `PredictionTensor` (probabilidades de movimiento, valor esperado).
- `ActionVector` (en el caso de DRL: comprar, vender, mantener).

## Tareas (TTRs)

### TTR-001: Implementación de Transformadores de Atención Temporal
- **Problema:** Las LSTMs tienen dificultades con dependencias de muy largo plazo.
- **Qué tiene que pasar:** Crear bloque de Atención para series temporales financieras.
- **Criterio de éxito:** Mejorar precisión en predicción de volatilidad a 24h vs baseline ARIMA/LSTM.

### TTR-002: Agente de Ejecución DRL (PPO)
- **Problema:** El slippage y el impacto de mercado son difíciles de modelar con reglas fijas.
- **Qué tiene que pasar:** Entrenar agente PPO en el simulador tick-by-tick.
- **Criterio de éxito:** Reducir slippage acumulado en un 10% comparado con ruteo simple.

### TTR-003: Cloud LLM Gateway
- **Problema:** Los LLMs locales pueden carecer del razonamiento necesario para resumir veredictos complejos o corregir topologías raras de AST, requiriendo modelos frontera remotos sin filtrar secretos.
- **Qué tiene que pasar:** Desarrollar un cliente gRPC que envíe prompts anonimizados (con alias y IDs lógicos en lugar de fórmulas explícitas) a APIs de modelos frontera (GPT-4o).
- **Criterio de éxito:** Respuestas completadas e integradas a la UI local en < 3s, con 100% de datos sensibles anonimizados en origen.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada entrenamiento e inferencia de Deep Learning registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del modelo/Job |
| | `created_at` | Timestamp de inicio de cómputo |
| | `audit_hash` | Hash de integridad del modelo/pesos |
| | `audit_chain_hash` | Hash de integridad del dataset de entrenamiento |
| **II. Soberanía** | `owner_id` | Dueño de la IP del modelo |
| | `institutional_tag` | Tag de cumplimiento institucional |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la arquitectura (`burn`/`candle`) |
| | `data_snapshot_id" | Ref al snapshot de datos market/trades |
| | `indicator_state_hash` | Snapshot de los pesos de la red |
| | `version_node_id` | Versión del modelo en el DAG |
| **IV. Hardware** | `node_id` | Hardware ID (Cuda Device) |
| | `process_id` | PID del proceso worker GPU |
| | `execution_latency_ms` | Latencia total de entrenamiento/inferencia |

## Gobernanza y Estándares (Fijos)
