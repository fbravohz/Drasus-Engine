# Fractional Differencer (Memory Preservation)

**Carpeta:** `./features/fractional-differencer/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0064 (Preservación de Memoria Estadística via Diferenciación Fraccional)

## ¿Qué es esta feature?

El **Fractional Differencer** es una herramienta de procesamiento de series temporales que permite transformar una serie no-estacionaria (como los precios) en una serie estacionaria **preservando la máxima memoria estadística posible**. 

A diferencia de la diferenciación tradicional (entera, $d=1$), que elimina la tendencia pero también gran parte de la señal predictiva, la diferenciación fraccional ($0 < d < 1$) utiliza una ventana de pesos decrecientes para mantener la correlación histórica necesaria para el Alpha.

## Comportamientos Observables

- [ ] El usuario aplica diferenciación fraccional a una serie de precios de Bitcoin.
- [ ] El sistema genera una nueva serie que pasa el test de ADF (estacionaria) pero cuya gráfica aún conserva visualmente "ecos" de la estructura original del precio.
- [ ] Si se usa $d=0.4$, la serie resultante tiene una correlación mucho mayor con la original que si se usara $d=1.0$ (diferenciación estándar).
- [ ] El sistema calcula automáticamente el valor óptimo de $d$ que minimiza la pérdida de varianza mientras garantiza estacionariedad.

## Restricciones

- **NUNCA** usar diferenciación entera si la serie puede ser estacionarizada con un $d < 1$ (para evitar pérdida de Alpha).
- **Límite de Ventana:** La ventana de pesos debe estar truncada para evitar costos de cómputo infinitos (uso de pesos de tolerancia configurable).
- **PROHIBIDO** el uso de datos futuros para el cálculo de los pesos (Cero look-ahead bias).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DIFF_ORDER_D | 0.45 | 0.1 - 1.0 | El grado de diferenciación fraccional | CONFIG |
| WEIGHT_TOLERANCE | 1e-4 | 1e-6 - 1e-2 | Valor por debajo del cual se ignoran los pesos de la ventana | CONFIG |
| ADF_THRESHOLD | -2.86 | -4.0 - -2.0 | Valor crítico para el test de Dickey-Fuller (estacionariedad) | CONFIG |
| WINDOW_TYPE | fixed | fixed / expanding | Tipo de ventana para el cálculo de pesos | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Generador de coeficientes binomiales de pesos (expansión de Taylor), operación de convolución sobre la serie.
- **Shell (Infraestructura):** Integración con Polars para procesamiento vectorial rápido, almacenamiento de la serie diferenciada en Parquet efímero.
- **Frontera Pública:** Recibe una Serie Temporal (Precios); produce una Serie Diferenciada Estacionaria.

## Ciclo de Vida de la Feature

### Entrada
- Serie temporal de precios crudos (Float).
- Parámetro de orden $d$ o solicitud de auto-optimización.

### Proceso
1. Calcula los pesos decrecientes basados en el orden $d$ y la tolerancia.
2. Aplica la convolución de los pesos sobre la serie histórica.
3. Valida la estacionariedad mediante el test ADF.

### Salida
- Serie transformada (estacionaria).
- Reporte de varianza preservada vs varianza original.

### Contextos de Uso
**Contexto: Módulo Ingest**
- Pre-procesamiento de datos para alimentar modelos de Machine Learning o generadores de señales que requieren estacionariedad.

**Contexto: Módulo Generate**
- Uso en el Genetic Builder para crear indicadores "Memory-Aware".

## Tareas (TTRs)

### **TTR-001: Generador de Pesos de Diferenciación**
Implementación del algoritmo que genera la serie de pesos binomiales decrecientes. Debe manejar la precisión numérica para evitar degradación en ventanas largas.

### **TTR-002: Motor de Convolución Vectorial**
Aplicación de los pesos sobre la serie de precios. Debe usar Polars o Rust SIMD/Rayon para garantizar que el procesamiento de millones de barras ocurra en milisegundos.

### **TTR-003: Auto-Optimización de Orden d**
Lógica iterativa que busca el menor valor de $d$ que logre superar el umbral de ADF, maximizando la retención de memoria.

## Gobernanza y Estándares
- **Inundación de Fundaciones (ADR-0020 V2): Perfil A (Datos / Ingest)** — persiste series transformadas + linaje del orden $d$ (I + III + IV).

  | Categoría | Campo | Descripción |
  | :--- | :--- | :--- |
  | **I. Identidad** | `id` | Identificador único de la serie diferenciada |
  | | `created_at` | Timestamp de la transformación |
  | | `updated_at` | Timestamp de última modificación del registro |
  | | `audit_hash` | Hash de integridad de la serie resultante |
  | | `audit_chain_hash` | Hash encadenado del historial de transformaciones |
  | | `event_sequence_id` | Secuencia de recuperación |
  | **III. Linaje** | `data_snapshot_id` | Puntero a la serie cruda de origen (PIT) |
  | | `transformation_id` | ID del paso de diferenciación fraccional aplicado |
  | | `parent_id` | Serie padre de la que deriva (linaje de la transformación) |
  | | `version_node_id` | Versión del transformador en el DAG |
  | **IV. Hardware** | `node_id` | ID del hardware físico (aceleración vectorial) |
  | | `process_id` | PID del motor de convolución |
- **Local-First:** 100% Local.
- **Rastro de Evidencia:** Emite el valor de $d$ óptimo y la estadística ADF al módulo de `feedback`.
