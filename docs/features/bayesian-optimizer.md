# Bayesian Optimizer

**Carpeta:** `./features/bayesian-optimizer.md`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0013 (Stack Tecnológico - Rust SIMD/Rayon)

## ¿Qué es esta feature?

El **Optimizador Bayesiano** es un motor de búsqueda inteligente de parámetros que utiliza modelos probabilísticos (Procesos Gaussianos) para encontrar la combinación óptima de configuración sin tener que probar todas las posibilidades (Grid Search). Su principal ventaja es la eficiencia: requiere un 70-90% menos de iteraciones para converger a un resultado global.

## Narrativa de Producto

**Capacidad:** `CAP-03` — Búsqueda automática en el espacio de estrategias

**Historia de usuario:** Como investigador de estrategias, quiero afinar los parámetros de una estrategia costosa de evaluar sin tener que probar miles de combinaciones a fuerza bruta, para llegar a una configuración sólida gastando una fracción del cómputo.

**El problema (técnico):** cuando cada evaluación es cara —una simulación larga, un dataset pesado— probar todo el espacio de parámetros por fuerza bruta es inviable, y elegir a mano solo veinte combinaciones deja fuera casi con certeza a las mejores.

**La solución (técnica):** un modelo probabilístico —un proceso gaussiano— que aprende de cada evaluación dónde es probable que esté el óptimo, y decide el siguiente punto a probar en función de esa evidencia acumulada, en vez de muestrear al azar o recorrer una rejilla completa. Es una implementación par del puerto de búsqueda: útil cuando el presupuesto de evaluaciones es limitado, no un sustituto del motor evolutivo.

**Explicado sin tecnicismos:** imagina que buscas el punto más alto de una colina cubierta de niebla, y cada vez que das un paso y mides la altura te cuesta caro —tienes que esperar un rato para saber cuánto subiste—.

En vez de caminar al azar o recorrer la colina en cuadrícula, vas anotando cada medición y usas el patrón que se forma para adivinar hacia dónde conviene dar el siguiente paso. Cada intento te acerca más rápido a la cima que probar puntos al azar, porque aprendes de los pasos anteriores en vez de ignorarlos.

---

## Comportamientos Observables

- [ ] El usuario define rangos de parámetros (ej: RSI Period 5-50) y el objetivo (Optimizer Target: Sharpe Ratio).
- [ ] El sistema ejecuta backtests secuenciales "inteligentes", aprendiendo de cada resultado para predecir dónde está el mejor Sharpe.
- [ ] Visualización del mapa de calor de probabilidad y convergencia.
- [ ] Capacidad de optimización multi-dimensional (hasta 20-30 parámetros simultáneamente).

## Restricciones

- **NUNCA** permitir la optimización masiva sin validación Out-of-Sample (WFA) para evitar el ajuste excesivo.
- **OBLIGATORIO:** Paralelización de los backtests individuales generados por el proceso bayesiano.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| OPTIMIZER_ITERATIONS | 50 | 20 - 500 | Cuántas iteraciones inteligentes realizar | CONFIG |
| SURROGATE_MODEL | GAUSSIAN_PROCESS | RF, GP, TPE | Modelo probabilístico subyacente | CONFIG |
| AQUISITION_FUNCTION | EXPECTED_IMPROVEMENT | EI, LCB, PI | Cómo decide el próximo punto a probar | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmo de adquisición y actualización de la distribución posterior (Posterior Distribution).
- **Shell (Infraestructura):** Orquestador de tareas asíncronas que lanza los backtests en el `async-job-executor`.

## Ciclo de Vida de la Feature — Bayesian Optimizer

### Entrada
- Definición de espacio de búsqueda (Search Space).
- Función objetivo (ej. Maximizar Sharpe × WinRate).

### Proceso
1. **Initial Sampling:** Pruebas aleatorias iniciales para "calentar" el modelo.
2. **Surrogate Fitting:** Crea un modelo matemático de la superficie de rendimiento.
3. **Point Selection:** Elige el mejor punto usando la función de adquisición.
4. **Update:** Ejecuta backtest y actualiza el modelo.

### Salida
- `OptimalParametersDict`: La mejor configuración encontrada.
- Reporte de sensibilidad de parámetros.

### Contextos de Uso
- **Generate:** Para el ajuste fino (fine-tuning) de estrategias supervivientes.
- **Manage:** Para optimizar dinámicamente los pesos del portafolio.

## Tareas (TTRs)

### TTR-001: Integración con Scikit-Optimize / Optuna
- **Problema:** No queremos reinventar algoritmos bayesianos probados.
- **Qué tiene que pasar:** Integrar el backend de Optuna o Scikit-Optimize con el motor de backtesting local.
- **Criterio de éxito:** Ejecutar 50 iteraciones en < 1/10 del tiempo de un Grid Search completo.

### TTR-002: Visualizador de Superficie de Optimización
- **Problema:** Los usuarios necesitan confiar en que el optimizador no está atrapado en un máximo local insignificante.
- **Qué tiene que pasar:** Crear gráficos de contorno de 2D/3D que muestren la relación entre parámetros y fitness.
- **Criterio de éxito:** Renderizado interactivo en la UI via WebGL.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada corrida de optimización inteligente registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del Job de optimización |
| | `created_at` | Timestamp de inicio |
| | `updated_at` | Última actualización del job |
| | `audit_hash` | Hash de los parámetros óptimos finales |
| | `audit_chain_hash` | Hash de la secuencia de puntos evaluados |
| | `event_sequence_id` | Secuencia del evento de optimización |
| **II. Soberanía** | `owner_id` | Usuario que solicitó la optimización |
| | `manifest_id` | ID del diseño evaluado |
| | `institutional_tag` | Etiqueta de entorno de ejecución |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor Bayesiano |
| | `data_snapshot_id` | Puntero a los datos históricos usados |
| | `indicator_state_hash` | Snapshot de la superficie de respuesta |
| | `version_node_id` | Versión de la estrategia en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del proceso orquestador |

## Gobernanza y Estándares (Fijos)
