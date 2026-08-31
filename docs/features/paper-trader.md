# Paper Trader

**Carpeta:** `./features/paper-trader/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Es el componente encargado de ejecutar una estrategia en tiempo real sin riesgo de capital. Su misión es la **Simulación de Alta Fidelidad**: operar sobre feeds de datos vivos capturando la fricción real del mercado (spreads, liquidez) para que los resultados sean estadísticamente indistinguibles de una cuenta real.

---

## Narrativa de Producto

**Capacidad:** `CAP-05` — Cuarentena antes del dinero real

**Historia de usuario:** Como trader, quiero que mi estrategia opere contra el mercado en vivo sin arriesgar capital, para ver cómo se comporta con las condiciones de hoy antes de confiarle dinero real.

**El problema (técnico):** un backtest histórico no contiene el presente. No sabe cómo está la liquidez esta semana, ni cómo se está portando el bróker este mes, ni la latencia real del momento. La única forma de descubrir esa brecha revisando el pasado con más cuidado es imposible — se descubre operando, y la única manera de operar sin pagar el precio de estar equivocado es hacerlo sin capital real de por medio.

**La solución (técnica):** un motor que evalúa la lógica de la estrategia contra cada actualización del feed de mercado en vivo, genera órdenes virtuales cuando hay señal, y calcula el precio de llenado capturando el spread dinámico real del instante — no un spread promedio ni idealizado. Lleva un balance virtual inmutable y auditado por sesión, construido para que sus resultados sean estadísticamente indistinguibles de los de una cuenta real.

**Explicado sin tecnicismos:** es un simulador de manejo, pero conectado al tráfico real de la calle en vez de a un circuito grabado de antemano. El alumno no puede chocar el coche de verdad, pero tampoco está practicando contra un video de hace un año — está reaccionando al tráfico de ahora mismo, con sus baches y sus imprevistos de hoy.

Por eso esta pieza no reemplaza al examen histórico (`CAP-04`): lo complementa con la única prueba que el pasado no puede ofrecer, que es el presente.

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
## Persistencia (Inundación de Fundamentos — ADR-0020)

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
| | `institutional_tag` | Etiqueta de entorno (ADR-0020) |
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
