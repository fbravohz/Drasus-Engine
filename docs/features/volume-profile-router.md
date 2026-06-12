# Volume Profile Router

**Carpeta:** `./features/volume-profile-router.md`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0013 (Stack Tecnológico - DuckDB/Parquet)

## ¿Qué es esta feature?

El **Ruteador por Perfil de Volumen** es una capa de seguridad en la ejecución que suspende automáticamente las órdenes ante caídas de liquidez detectadas mediante el análisis del perfil de volumen (Volume at Price). Su objetivo es evitar deslizamientos (*slippage*) excesivos al intentar operar en "huecos" de liquidez o durante periodos de pre-volumen anómalo.

## Comportamientos Observables

- [ ] Monitoreo en tiempo real del mapa de liquidez (Order Book + Trades).
- [ ] Suspensión de órdenes pendientes si la liquidez proyectada en el precio objetivo es inferior al umbral mínimo.
- [ ] Reanudación automática cuando el perfil de volumen vuelve a niveles institucionales saludables.
- [ ] Reporte de "Liquidity Veto" en los logs de ejecución.

## Restricciones

- **NUNCA** permitir una orden si la profundidad de mercado (*depth*) es insuficiente para absorber al menos el 200% del tamaño de la posición sin mover el precio más de un 0.05% (configurable).
- **OBLIGATORIO:** Integración con el `audit-log` para registrar cada veto por liquidez.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| LIQUIDITY_THRESHOLD | 0.05% | 0.01% - 0.5% | Deslizamiento máximo permitido por perfil | CONFIG |
| MIN_VOLUME_AVG | 1.5x | 0.5x - 5.0x | Volumen mínimo vs promedio móvil para operar | CONFIG |
| ADAPTIVE_HALT | True | True/False | Si activa la suspensión automática | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmo de cálculo de perfil de volumen (VA, POC, High Volume Nodes vs Low Volume Nodes) en Rust SIMD/Rayon.
- **Shell (Infraestructura):** Suscripción a WebSockets de Nivel 2 (Quotes) y Nivel 1 (Trades).

## Ciclo de Vida de la Feature — Volume Profile Router

### Entrada
- Datos de trades recientes (Tick-by-Tick).
- Estado del Order Book (opcional, si disponible).

### Proceso
1. **Profile Construction:** Agrupa volumen por niveles de precio.
2. **Anomaly Detection:** Identifica "huecos" de liquidez (Low Volume Nodes).
3. **Execution Veto:** Cruza el precio de la orden con el mapa de liquidez.

### Salida
- `Boolean` (CanTrade / HaltNow).
- `SlippageProjection` (estimado de costo de impacto).

### Contextos de Uso
- **Execute:** Como guardián final de la orden en vivo.
- **Validate:** Para determinar si los backtests son realistas según el volumen histórico.

## Tareas (TTRs)

### TTR-001: Motor de Perfil de Volumen Vectorizado
- **Problema:** Calcular el perfil de volumen barra por barra es ineficiente.
- **Qué tiene que pasar:** Crear una estructura de datos Parquet/Rust que mantenga el perfil de forma incremental.
- **Criterio de éxito:** Actualizar el perfil en < 0.5ms por trade.

### TTR-002: Lógica de Veto por Deslizamiento Proyectado
- **Problema:** Los deslizamientos inesperados destruyen el Alpha.
- **Qué tiene que pasar:** Implementar la lógica que calcula el llenado (*fill*) proyectado sobre el perfil actual.
- **Criterio de éxito:** Bloquear órdenes que tendrían un slippage > 2x el spread promedio.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada decisión de ruteo registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del evento de ruteo |
| | `created_at` | Timestamp de la señal entrante |
| | `audit_hash` | Hash del veredicto de ruteo |
| | `audit_chain_hash` | Hash de la secuencia de perfiles |
| **II. Soberanía** | `owner_id` | Dueño del algoritmo de riesgo |
| | `manifest_id` | ID del diseño evaluado |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor de cálculo de perfil |
| | `data_snapshot_id` | Puntero a los ticks/quotes usados |
| | `indicator_state_hash` | Snapshot del perfil de volumen T-0 |
| | `version_node_id` | Versión de la política de riesgo |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del proceso de ejecución |

## Gobernanza y Estándares (Fijos)
