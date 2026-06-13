# Quality Heatmap Generator

**Carpeta:** `./features/quality-heatmap-generator/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0020 V2 (Inundación de Fundaciones)

## ¿Qué es?

Es el componente responsable de auditar la integridad de los datos históricos y generar una representación visual (Heatmap) de su calidad. Permite al usuario identificar rápidamente zonas con gaps, datos interpolados o anomalías de volatilidad antes de confiar en un backtest.

**Problema:** Una serie temporal de un año puede parecer buena, pero tener micro-gaps invisibles que alteran el resultado de la estrategia.
**Solución:** Un escáner que calcula un score de calidad por cada bloque de tiempo y lo envía al frontend para su visualización cromática.

## Comportamientos Observables

- [ ] El usuario ve un calendario o línea de tiempo de un activo:
  - **Verde:** Datos 100% íntegros y validados.
  - **Amarillo:** Presencia de micro-gaps rellenados (interpolados).
  - **Rojo:** Gaps masivos o datos corruptos/faltantes.
- [ ] Al pasar el ratón por una zona, el sistema muestra el % exacto de integridad estructural.
- [ ] Permite filtrar el backtest para que solo use zonas "Verdes".

## Restricciones

- NUNCA se realiza el cálculo del heatmap en el hilo principal; es una tarea analítica delegada a DuckDB/Polars.
- NUNCA se guarda el heatmap como dato estático; debe poder regenerarse bajo demanda para reflejar nuevas sanitizaciones.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| HEATMAP_RESOLUTION | Monthly | Daily/Monthly | Granularidad de la auditoría visual | CONFIG |
| MIN_GREEN_SCORE | 99.9% | 90% - 100% | Umbral para marcar zona como íntegra | CONFIG |
| ANALYZE_VOLATILITY | False | True/False | Detectar anomalías de precios (outliers) además de gaps | CONFIG |

## Ciclo de Vida de la Feature — Quality Heatmap Generator

### Entrada
- Serie temporal de Parquet (Símbolo + Rango).
- Reglas de calidad del `Data Sanitizer Pipeline`.

### Proceso
- Ejecuta una agregación DuckDB para contar barras esperadas vs barras reales.
- Calcula el ratio de integridad por celda del heatmap.
- Identifica zonas con flags de auditoría forense (`is_synthetic`, `correction_applied`).

### Salida
- Objeto JSON con la matriz de calidad para el frontend.
- Score global de integridad del activo (`Asset Integrity Score`).

### Contextos de Uso

**Contexto 1: Inventory (Módulo Ingest)**
- Visión general del "almacén de datos" del usuario.

**Contexto 2: Configuración de Backtest (Módulo Validate)**
- Advertencia al usuario si intenta validar en una zona de baja calidad.

## Tareas (TTRs)

### **TTR-001: Escáner de Integridad Estructural**
- Implementa la lógica DuckDB para detectar discontinuidades cronológicas masivas.

### **TTR-002: Generador de Matriz Visual**
- Desarrolla el servicio que traduce los hallazgos técnicos en coordenadas de color para el componente Heatmap de Flutter.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada escaneo de calidad genera un registro de persistencia filtrado para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del escaneo |
| | `created_at` | Timestamp de generación |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Correlación forense con el CSV/Parquet origen |
| | `audit_chain_hash` | Hash de integridad del bloque analizado |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | ID de la fuente (Sovereign Fetcher ID) |
| | `transformation_id` | ID del pipeline de sanitización usado |
| | `logic_hash` | Hash de las reglas de calidad activas |
| **IV. Hardware** | `node_id` | ID del hardware físico escaneador |
| | `process_id` | PID del worker DuckDB/Polars |
