# Microestructura L3 (Level 3 Market-by-Order)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental - Prioridad P4)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0100 (Relegación de Microestructura L3 a SaaS Institucional y Proxies Client Zero)

---

## ¿Qué es?

Explora la simulación y análisis de estrategias cuantitativas basadas en datos de **Nivel 3 (L3) - Market-by-Order (MBO)**. A diferencia de L1 (Bid/Ask) y L2 (Order Book por niveles de precio agregados), los datos de Nivel 3 permiten rastrear el ciclo de vida de cada orden individual y cola de ejecución en el matching engine del exchange. Esta característica está diseñada exclusivamente para fondos HFT con presupuestos institucionales capaces de absorber costos prohibitivos de feeds de datos e infraestructura de hardware dedicada en la nube.

---

## ¿Por qué NO es viable para la fase local del Client Zero?

1. **Costo Prohibitivo de Feeds de Datos:**
   - La suscripción a feeds L3 de exchanges regulados (ej: CME/ICE) oscila entre $5K y $20K mensuales por instrumento, violando el principio del Client Zero de costos fijos recurrentes nulos.
2. **Requisitos de Almacenamiento Locales Inmanejables:**
   - El almacenamiento de tick L3 consume de 10 a 50 GB diarios por símbolo. Un portafolio estándar requeriría más de 200TB anuales, inviable en unidades de estado sólido de consumo doméstico (1-2 TB).
3. **Complejidad de Infraestructura Dedicada:**
   - Requiere clusters de bases de datos distribuidas (ClickHouse/TimescaleDB) operando de forma continua para ingesta masiva en vivo, rompiendo el patrón "Zero-Docker".

---

## Alternativa Client Zero Aprobada

- **Datos de Nivel 1 (Bid/Ask):** Obtenidos de Binance/exchanges de forma gratuita para simulación tick-a-tick estándar.
- **Datos de Nivel 2 (DOM):** Suficientes para capturar presión institucional y absorción de volumen con costos de almacenamiento optimizados.
- **Análisis MAE/MFE:** Sirve como un proxy de la eficiencia de colas de ejecución en la microestructura sin requerir la inmensa carga de datos L3.

---

## Comportamientos Observables (SaaS Institucional Futuro)

- [ ] **Simulador de Colas de Ejecución (FIFO / Pro-Rata):** Simula el orden exacto de prioridad de la orden del usuario en la cola del exchange, calculando con precisión de microsegundos el tiempo de ejecución.
- [ ] **Cancelación e Inyección en DOM:** Rastreabilidad de adición de volumen de creadores de mercado institucionales y cancelación de órdenes al milisegundo.

---

## Restricciones

- **NUNCA** habilitar L3 en despliegues locales-first tradicionales si no se cumple el prerrequisito de hardware y conectividad de alta densidad.
- **FIJO:** El motor de NautilusTrader utiliza su backend nativo `L3_MBO` para simular colas de prioridad de forma determinista.

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmos de ordenación de colas en el matching engine y reconstrucción del libro de órdenes orden por orden.
- **Shell (Infraestructura):** Integración con servidores ClickHouse masivos y pipelines de ingesta a nivel de socket de baja latencia.

---

## Tareas (TTRs)

### **TTR-001: Reconstrucción de Libro Market-by-Order**
*   **¿Cuál es el problema?** L2 solo proporciona fotos del libro agregadas; no permite saber si nuestra orden está al frente o detrás de un muro institucional grande.
*   **¿Qué tiene que pasar?** Diseñar un parser en Rust nativo que tome las actualizaciones de órdenes individuales del feed (Order Add, Order Modify, Order Cancel, Order Execute) y mantenga el estado de colas prioritarias por ID de orden.
*   **¿Cómo sé que está hecho?**
    - [ ] Se reconstruye la cola completa del libro con IDs de órdenes verificables.
    - [ ] El simulador de NautilusTrader procesa la prioridad de ejecución respetando las posiciones de las órdenes.
*   **¿Qué no puede pasar?**
    - No puede haber pérdidas de eventos L3 que corrompan el matching engine (descuadres de volumen total).

---

## Gobernanza y Estándares (ADR-0020)

### Perfil Datos / Ingesta
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de flujo L3 |
| | `created_at` | Timestamp de ingesta del tick |
| | `audit_hash` | Hash de integridad del bloque L3 Parquet |
| **III. Linaje Alpha** | `data_snapshot_id` | Identificador del PIT Snapshot de datos L3 |
| **IV. Hardware** | `node_id` | ID del servidor ClickHouse de ingesta |
| | `process_id` | PID de la tarea de socket de red |
| | `execution_latency_ms` | Latencia de decodificación de socket a base de datos |
