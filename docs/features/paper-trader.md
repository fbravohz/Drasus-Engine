# Paper Trader

**Carpeta:** `./features/paper-trader/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Es el componente encargado de ejecutar una estrategia en tiempo real sin riesgo de capital. Su misión es la **Simulación de Alta Fidelidad**: operar sobre feeds de datos vivos capturando la fricción real del mercado (spreads, liquidez) para que los resultados sean estadísticamente indistinguibles de una cuenta real.

---

## Comportamientos Observables

- [ ] Genera señales basadas en datos de mercado en tiempo real.
- [ ] Simula ejecuciones de órdenes (fills) aplicando spreads y slippage variables.
- [ ] Mantiene un balance virtual inmutable y auditado por cada sesión.

---

## Ciclo de Vida de la Feature — Paper Trader

### Entrada
- Estrategia validada y parámetros fijos.
- Feed de datos en vivo (Ticks/Barras).
- Capital inicial virtual configurado.

### Proceso
- Evalúa la lógica de la estrategia sobre cada nueva actualización de datos.
- Si hay señal, genera una "Orden Virtual" enviada a `order-fsm`.
- Calcula el "Precio de Ejecución Virtual" capturando el Spread dinámico del feed.

### Salida
- Record inmutable de trades virtuales (Fills).
- Curva de equidad virtual con timestamps de alta resolución.
- Veredicto de "Operatibilidad" para el módulo de `feedback`.

### Contextos de Uso
**Contexto 1: Incubación (Módulo Incubate)**
- Prueba de fuego antes de arriesgar capital.
**Contexto 2: Pruebas de Estrategia Manual**
- Permite a un usuario "sentir" el mercado con una estrategia nueva sin entrar a producción.

---

## Tareas (TTRs) — Herencia de Incubadora

### TTR-001: Implementar Motor de Ejecución en Tiempo Real
*   **Descripción:** Recibe el feed en vivo, procesa señales de la estrategia y emite órdenes virtuales. 
*   **Criterio de Éxito:** Los fills simulados coinciden con los precios de mercado en el momento exacto de la orden.

### TTR-002: Simulación de Spread Dinámico
*   **Descripción:** Captura el bid/ask real del mercado al momento de la orden virtual para calcular el fill.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. El procesamiento de señales vivas debe ocurrir en el hardware local para asegurar fidelidad técnica.
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda cuenta y trade virtual registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del trade virtual |
| | `created_at` | Timestamp de ejecución (nanosegundos) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del balance virtual |
| | `audit_chain_hash` | Hash de la secuencia de trades de la sesión |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Dueño de la IP de la estrategia |
| | `manifest_id` | ID del contrato de diseño legal |
| | `institutional_tag` | Etiqueta de entorno (ADR-0020 V2) |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la lógica de trading inyectada |
| | `data_snapshot_id` | Puntero al feed de datos vivos (L2) |
| | `indicator_state_hash` | Snapshot del estado técnico T-0 |
| | `version_node_id` | ID de la versión en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de incubación |
| | `execution_latency_ms` | Latencia simulada de ejecución |


---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Lógica de cálculo de llenado (fill engine) basado en spread.
- **Shell (Infraestructura):** Listener de feeds de datos y manejador de balance virtual.
- **Frontera Pública:** Interfaz de inicio de sesión de papel `start_incubation(strategy)`.

---

## Dependencias
**Consumido por:** `incubate` (Orquestador).
**Depende de:** `order-fsm`, `clock`.
