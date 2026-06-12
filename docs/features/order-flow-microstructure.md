# Order Flow & Microstructure (Flujo de Órdenes y Microestructura)

**Carpeta:** `./features/order-flow-microstructure.md`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0004 (Máquina de Estados), ADR-0017 (Simulación de Alta Fidelidad)

## ¿Qué es esta feature?

Esta feature provee las métricas de alta frecuencia necesarias para detectar la presión institucional y la absorción de liquidez. A diferencia de los indicadores de precio, el flujo de órdenes analiza la batalla directa entre compradores y vendedores en el libro de órdenes (DOM) y en el registro de transacciones ejecutadas.

### Métricas Incluidas:
- **CVD (Cumulative Volume Delta):** Suma acumulada de la diferencia entre el volumen de compra y venta agresivo. Revela quién tiene el control del mercado.
- **VWAP (Volume Weighted Average Price):** Precio promedio ponderado por volumen, actuando como el "valor justo" institucional.
- **OFI (Order Flow Imbalance):** Desequilibrio entre las órdenes limitadas de compra y venta en los niveles superiores del libro de órdenes (L2).

## Comportamientos Observables

- [ ] **Detección de Absorción:** El sistema identifica cuando el precio no se mueve a pesar de un CVD alto (indicando que una institución está absorbiendo todas las órdenes agresivas).
- [ ] **Validación por VWAP:** Las señales de ejecución se validan contra la distancia al VWAP; se rechazan compras si el precio está excesivamente alejado (sobre-extensión).
- [ ] **Presión en el DOM:** El sistema monitorea el OFI y cancela órdenes pendientes si la presión del libro de órdenes cambia drásticamente en contra de la posición.

## Restricciones

- **NUNCA** operar con métricas de flujo de órdenes en activos con volumen extremadamente bajo (señales ruidosas).
- El cálculo de CVD debe ser bit-a-bit idéntico entre el backtest y el trading en vivo (requiere feeds de ticks reales).
- La latencia para el cálculo de OFI no debe exceder los 5ms.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
| :--- | :--- | :--- | :--- |
| VWAP_DEVIATION_LIMIT | 2.5 | 1 - 5 | Desviaciones estándar máximas para permitir entrada |
| OFI_THRESHOLD | 0.65 | 0.5 - 0.9 | Grado de desequilibrio para validar señal |
| CVD_LOOKBACK | 100 | 20 - 500 | Velas para el cálculo de la tendencia del Delta |

## Tareas (TTRs)

### **TTR-001: Motor de Cálculo de Delta de Volumen (CVD)**
- **Qué tiene que pasar:** Programar un agregador que clasifique cada tick como "Compra" o "Venta" usando la regla del tick o la clasificación Bid/Ask.
- **Criterio de éxito:** Paridad total entre el indicador visual y el motor de ejecución.

### **TTR-002: Integración con NautilusTrader L2/DOM**
- **Qué tiene que pasar:** Conectar la feature al libro de órdenes profundo de Nautilus para extraer el desequilibrio de niveles (OFI).

### **TTR-003: Volume Imbalance BBO Node**
- **Qué tiene que pasar:** Implementar un nodo de microestructura que monitoree el DOM L2 en tiempo real y dispare una señal de compra/venta cuando el volumen de órdenes limite en el Best Bid supere al del Best Offer por más de 300% (o viceversa).

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del snapshot de flujo |
| | `created_at` | Timestamp de alta resolución (ns) |
| | `audit_hash` | Hash de integridad del DOM snapshot |
| **II. Soberanía** | `owner_id` | Identificador del dueño del feed de datos |
| **III. Linaje** | `data_snapshot_id` | Snap del libro de órdenes (DOM) en el momento del trade |
| | `indicator_state_hash` | Estado del CVD y VWAP en la ejecución |
| **IV. Hardware** | `node_id" | ID del hardware físico (Network Interface Card) |
| | `process_id` | PID del motor de ticks |
| | `execution_latency_ms` | Latencia de actualización del DOM / Pipeline |
| **V. Forense** | `event_sequence_id` | Índice secuencial de ticks para reconstrucción |
