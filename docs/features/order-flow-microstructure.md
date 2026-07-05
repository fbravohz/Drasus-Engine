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

## Entrega por Fases (Split por Dependencia Dura — ADR-0118)

Esta feature se construye en **dos partes**, en módulos y fases distintas, por una dependencia dura de datos: la parte en vivo necesita el libro de órdenes en tiempo real, que solo existe en ejecución.

- **Parte histórica (CVD sobre datos almacenados) → `ingest` / EPIC-1.** Calcula el Cumulative Volume Delta a partir del registro de transacciones ya ingestado (clasificación Compra/Venta por la regla del tick). Requiere datos a nivel de operación (ej.: Binance aggTrades). Cubre el TTR-001. Enriquece la golden source para backtesting (`validate`) y generación de señales (`generate`).
- **Parte en vivo (OFI / presión del DOM L2) → `execute` / EPIC-6.** El desequilibrio del libro de órdenes profundo y la guardia pre-trade necesitan el feed L2 en tiempo real vía NautilusTrader. Cubren los TTR-002 y TTR-003. Es la guardia de microestructura defensiva de la ejecución nativa.

La paridad bit-a-bit del CVD entre backtest y vivo (ver Restricciones) es el contrato que une ambas partes.

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

## Preparación para Opciones (Post-MVP — ADR-0140)

> **Estado:** Diferido. No implementar hasta que los cinco prerrequisitos de ADR-0140 se cumplan.

El flujo de órdenes actual (CVD, OFI, VWAP) opera sobre instrumentos lineales donde el volumen de transacciones es la métrica central. En opciones, cada transacción conlleva el pago de una **prima** (el precio de la opción × multiplicador del contrato), lo que habilita una métrica superior: el **Net Premium Flow (NPF)**.

### Net Premium Flow (NPF)

El NPF consolida el balance neto de capital que entra al mercado de opciones, clasificando cada transacción por intención institucional:

| Flujo | Clasificación |
|---|---|
| **Alcista (+)** | Compra de Calls al Ask + Venta de Puts al Bid |
| **Bajista (−)** | Compra de Puts al Ask + Venta de Calls al Bid |

**NPF = Flujo Alcista Total − Flujo Bajista Total**

La clasificación de cada transacción como "compra agresiva" o "venta agresiva" sigue el algoritmo de Lee-Ready: órdenes ejecutadas en el Ask son compras iniciadas por el comprador; órdenes en el Bid son ventas iniciadas por el vendedor.

### Señales derivadas del NPF

- **Detección de Sweeps institucionales:** órdenes que barren la liquidez de múltiples exchanges simultáneamente. Un Sweep al Ask con prima millonaria delata Smart Money operando con urgencia antes de un movimiento brusco del subyacente.
- **Gamma Squeeze prediction:** NPF hiper-agresivo concentrado en Calls OTM de corto vencimiento obliga a los creadores de mercado (cortos de Gamma) a comprar el subyacente en masa, creando un bucle de retroalimentación alcista.
- **Divergencia NPF vs Precio:** si el precio cae pero el NPF se mantiene positivo (acumulación oculta de Calls o venta agresiva de Puts), el sistema interpreta divergencia alcista.
- **Contraste NPF vs Open Interest:** si el NPF masivo en el Ask supera al Open Interest previo, es apertura de posiciones agresivas (momento); si el volumen es alto pero cerca del Bid, es toma de ganancias o rebalanceo.

### Filtrado de estrategias multi-pata

El NPF bruto produce señales falsas cuando las transacciones son patas de estrategias multi-pata (spreads, straddles). El motor debe filtrar transacciones que forman parte de una combo order conocida (mismo timestamp, subyacente, y strikes correlacionados) antes de agregar al NPF neto.

**Refactorización necesaria:** añadir el motor de NPF como una métrica paralela al CVD, consumiendo el feed de opciones del [`option-data-ingestor`](../moonshots/option-data-ingestor.md) y clasificando transacciones con el algoritmo de Lee-Ready adaptado a opciones.

**Moonshots asociados:** [`option-data-ingestor`](../moonshots/option-data-ingestor.md), [`greeks-monitor`](../moonshots/greeks-monitor.md) (para el cálculo de Gamma del mercado agregado).

---

## Persistencia (Inundación de Fundamentos — ADR-0020 · Perfil C Hot-Path, híbrido C+III)

Híbrido: Perfil C (Ops/Hot-Path) + linaje III legítimo (snapshot DOM/tick reproducible).

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del snapshot de flujo |
| | `created_at` | Timestamp de alta resolución (ns) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del DOM snapshot |
| | `audit_chain_hash` | Hash encadenado de la secuencia de snapshots |
| | `event_sequence_id` | Índice secuencial de ticks para reconstrucción (recovery) |
| **II. Soberanía** | `owner_id` | Identificador del dueño del feed de datos |
| **III. Linaje (híbrido)** | `data_snapshot_id` | Snap del libro de órdenes (DOM) en el momento del trade |
| **IV. Hardware** | `node_id` | ID del hardware físico (Network Interface Card) |
| | `process_id` | PID del motor de ticks |
| **V. Forense & Ejecución** | `execution_latency_ms` | Latencia de actualización del DOM / Pipeline (hot-path) |
| | `source_signal_id` | Señal/tick de origen del snapshot |
| | `indicator_state_hash` | Estado del CVD y VWAP en la ejecución (Grupo V) |
