# Portfolio Data Preparation

**Carpeta:** `./features/portfolio-data-preparation/`
**Estado:** En Diseño
**Última actualización:** 2026-04-29
**Decisión Arquitectónica Asociada:** ADR-0056 (Portfolio Data Preparation)

## 1. ¿Qué es esta feature?

Es el sistema encargado de preparar las fundaciones matemáticas y de datos para el análisis de portafolio multi-estrategia. Esta preparación ocurre después de generar estrategias individuales y antes de combinarlas en un portafolio.

**Problema:** Combinar estrategias basándose únicamente en Sharpe o PnL ignorando la correlación o el régimen del mercado lleva a un sobreajuste del portafolio (Curve Fitting) y vulnerabilidad en cambios de régimen.
**Solución:** Estandarizar las curvas de rendimiento, calcular la matriz de correlación temporal y asignar etiquetas de régimen de mercado para permitir una asignación jerárquica de riesgo (HRP) adaptativa.

## 2. Comportamientos Observables

- [ ] Durante la limpieza de datos, el sistema clasifica automáticamente cada barra histórica con una etiqueta de régimen de mercado (Trending, Ranging, Crash).
- [ ] Tras un bloque de generación de estrategias, el sistema procesa todas las curvas de rendimiento y las escala al mismo tamaño (normalización 0 a 1) para que sean comparables de forma justa.
- [ ] El sistema cruza las curvas normalizadas y construye una matriz matemática que indica qué tan parecida es la estrategia A a la estrategia B, guardándola en DuckDB.
- [ ] **Visualización de Correlaciones:** Genera un mapa de calor y un dendrograma interactivo basado en clustering single-linkage para visualización.

## 3. Restricciones

- La matriz de correlación NUNCA debe calcularse dinámicamente en memoria durante la ejecución en vivo; debe leerse estáticamente desde DuckDB.
- NUNCA se debe normalizar curvas de rendimiento usando métricas relativas que introduzcan *look-ahead bias* (sesgo de mirar al futuro).
- La asignación del régimen (HMM) NUNCA debe usar datos del día siguiente para clasificar el día actual.

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| HMM_STATES | 3 | 2-5 | Cantidad de regímenes de mercado detectables | CONFIG |
| NORMALIZATION_METHOD | min_max | min_max / z_score | Forma en la que se estandarizan las curvas de equidad | CONFIG |

## 5. Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos matemáticos HMM y matriz de covarianza de Pearson sin interacción con bases de datos.
- **Shell (Infraestructura):** Lectura masiva desde Parquet, escritura temporal en DuckDB y SQLite para almacenamiento en caché.
- **Frontera Pública:** Exposición del estado del régimen actual y consulta de vecindario de correlaciones.

## 6. Ciclo de Vida de la Feature

### Entrada
- Datos OHLCV históricos
- Curvas de rendimiento en bruto (P&L por trade o barra) de múltiples estrategias.

### Proceso
- Evalúa la volatilidad e inercia del mercado para asignar una etiqueta de régimen de mercado.
- Convierte las ganancias absolutas de múltiples estrategias en series temporales porcentuales relativas.
- Calcula la distancia de Pearson entre cada par de estrategias.

### Salida
- `regime_label` inyectado en cada barra de datos (Parquet).
- Matriz de Correlación temporal persistida en DuckDB.
- `equity_curve_path` (curva normalizada) inyectada en SQLite/Databank.

### Contextos de Uso
**Contexto 1: Optimización de Portafolio (Manage)**
- Entrada: Matriz de correlación y regímenes.
- Impacto: Define qué estrategias se descartan por colinealidad (mismo comportamiento) y qué peso recibe cada una (HRP).

**Contexto 2: Etiquetado Adaptativo (Generate / Validate)**
- Entrada: Etiquetas de régimen por barra.
- Impacto: Permite evaluar el comportamiento de una estrategia aislada en regímenes específicos.

## 7. Tareas (TTRs)

### **TTR-001: Implementación del Etiquetador HMM (Regime Labeling)**
* **Problema:** Necesitamos etiquetar cada día de mercado con un "estado" pre-entrenado.
* **Comportamiento:** Cada barra recibe una etiqueta `Trending`, `Ranging`, `Crash`.
* **Criterio de Éxito:** Las etiquetas generadas carecen de información del futuro y se guardan junto a la barra de precio.
* **Restricción:** Entrenamiento one-time.

### **TTR-002: Cálculo y Cacheo de Matriz de Correlación (Pearson)**
* **Problema:** Re-calcular correlaciones en tiempo real es inviable para miles de estrategias.
* **Comportamiento:** Se genera una matriz N x N estática en batch, almacenada en DuckDB.
* **Criterio de Éxito:** Recuperación de matriz completa de 1000 estrategias en <1s.
* **Restricción:** Solo se comparan curvas normalizadas en las mismas ventanas de tiempo.

### **TTR-003: Normalización de Curvas de Equidad**
* **Problema:** Comparar una curva que gana $1M con una que gana $10 es matemáticamente distorsionado sin normalizar.
* **Comportamiento:** Transforma curvas de precios absolutos en vectores de 0-1.
* **Criterio de Éxito:** La forma de la curva y el drawdown se mantienen proporcionales, pero su escala desaparece.

### **TTR-004: Generación de Dendrograma y Matriz de Calor**
* **Problema:** Los usuarios necesitan visualizar los clústeres de riesgo de las estrategias.
* **Comportamiento:** Calcula las distancias euclidianas y jerárquicas y exporta los datos para visualización.
* **Criterio de Éxito:** Genera un JSON estructurado con la matriz de distancias y la estructura del dendrograma.

## 8. Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** Baja (Basado en barras de cierre de equity o OHLCV diarios).
- **Inundación de Fundaciones (ADR-0020 V2):**
  - **Perfil Datos / IA:** Foco en Identidad + Linaje de pesos.
  - Campos a inyectar: Grupo I completo (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) + `version_node_id`, `data_snapshot_id`.
  - **Hooks Forenses:** `indicator_state_hash` para almacenar la foto de la matriz de correlación calculada.
- **Rastro de Evidencia:** Informa a `feedback` en caso de detectar rupturas dramáticas de correlación de un día para otro (colapso de matriz).

## 9. Decisión Arquitectónica Asociada
ADR-0056: Portfolio Data Preparation (HMM & Matriz Pearson)

## 10. Dependencias y Bloqueantes
**Depende de:** `ingest` (para OHLCV) y `generate` (para curvas de equidad).
**Bloquea:** `manage` (requiere la matriz para rebalanceo HRP).
