# Auto-Auditoría de Portafolios Vivos

**Carpeta:** `./features/auto-auditoria-portafolios-vivos/`
**Estado:** En Diseño
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0010 (Reglas Dinámicas: Hard Limits vs Soft Alerts), ADR-0025 (Pre-Trade Risk 10-Steps Gate), ADR-0026 (Shadow Watchdog & Heartbeat)

---

## ¿Qué es esta feature?

La auto-auditoría de portafolios vivos es un sistema de monitoreo en tiempo real que protege el capital operativo de la degradación de la expectativa de las estrategias en producción. Detecta cambios dinámicos en los costos de transacción (spreads ensanchados, comisiones variables elevadas o swaps penalizadores de los brokers) y recalcula la expectativa matemática ($R$), Sharpe ratio y Max Drawdown en caliente. Si las métricas cruzan los límites tolerables, el sistema emite alertas o pausa de manera automática la operativa para evitar operar con expectativa matemática negativa.

---

## Comportamientos Observables

- [ ] **Lector de Costos en Tiempo Real:** Interacción periódica con las APIs de los brokers (Darwinex, FTMO, Interactive Brokers) para leer spreads y comisiones vigentes.
- [ ] **Recalculador de R Expectancy Dinámico:** Un hilo optimizado en Rust evalúa de forma constante la expectativa matemática del portafolio activo aplicando los costos reales actuales sobre el histórico reciente.
- [ ] **Acción Defensiva de Pausa (Veto Automático):** Si la expectativa matemática calculada desciende del límite configurable (ej: $R < 1.0$), el sistema ejecuta una detención preventiva de nuevas operaciones (Hard Limit) y registra el evento.
- [ ] **Audit Trail de Costos Operativos:** Almacenamiento continuo del spread promedio, comisiones y slippage experimentado por estrategia en la base de datos de auditoría.
- [ ] **Alertas de Degradación:** Notificaciones de advertencia (Soft Alerts) en la interfaz gráfica y canales configurados ante incrementos de costos mayores al umbral definido.

---

## Restricciones

- **NUNCA** permitir que una estrategia continúe abriendo nuevas operaciones si los costos de transacción actuales eliminan por completo la ventaja estadística calculada.
- **NUNCA** bloquear el flujo crítico de ejecución de órdenes de NautilusTrader; el cálculo y análisis de costos debe ser asíncrono y realizarse en hilos secundarios dedicados.
- **FIJO:** Si la conexión a la API de spreads del bróker falla, el sistema aplica políticas de resiliencia usando el último spread conocido o spreads históricos penalizados para evitar la inacción ante desconexiones.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| SPREAD_FETCH_INTERVAL_SECS | 60 | 10 - 600 | Frecuencia con la que se consultan los costos de transacción al bróker | CONFIG |
| MIN_R_EXPECTANCY | 1.1 | 0.8 - 3.0 | Expectativa matemática mínima aceptable para mantener activa la estrategia | CONFIG |
| DEGRADATION_ALERT_THRESHOLD | 20% | 5% - 50% | Porcentaje de reducción de expectativa permitido antes de disparar alerta soft | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos en Rust para el cálculo de R Expectancy y Sharpe ajustado por comisiones y swaps en vivo.
- **Shell (Infraestructura):** Conectores a brókers, base de datos SQLite WAL para registro de auditoría de costos y bus de eventos.
- **Frontera Pública:** API para suspender/reactivar estrategias, leer métricas de degradación y configurar umbrales de coste.

---

## Ciclo de Vida de la Feature — Auto-Auditoría de Portafolios Vivos

### Entrada
- Datos de spreads, swaps y comisiones provistos en vivo por las APIs de los brokers.
- Registro de ejecuciones históricas y P&L actual por estrategia.
- Umbrales de expectativa matemática configurados.

### Proceso
- El sistema recupera los costos actuales del bróker a intervalos fijos.
- Se recalculan las curvas de rendimiento y la expectativa de cada estrategia activa.
- Si los resultados violan los parámetros de seguridad, se activa el interruptor automático (Hard Limit) o la notificación de advertencia.

### Salida
- Desactivación preventiva o restricción de lotaje de las estrategias afectadas.
- Reporte detallado de impacto de costos en el panel de control.

---

## Tareas (TTRs)

### **TTR-001: Lector y Normalizador de Costos de Bróker**
*   **¿Cuál es el problema?** Cada bróker reporta comisiones y spreads a través de formatos y APIs diferentes, lo que dificulta un cálculo estandarizado.
*   **¿Qué tiene que pasar?** Implementar los conectores específicos para extraer el spread actual y estructurarlo en un formato genérico común y normalizado en memoria.
*   **¿Cómo sé que está hecho?**
    - [ ] El sistema recibe cotizaciones de spread cada 60 segundos de Darwinex y FTMO sin fugas de memoria.
*   **¿Qué no puede pasar?**
    - Una desconexión temporal de la API del bróker no debe tumbar el proceso de monitoreo; debe usar valores de respaldo (stale fallback).

### **TTR-002: Recalculador de Expectativa en Caliente**
*   **¿Cuál es el problema?** El recálculo dinámico sobre miles de transacciones puede inducir latencias y retrasar el hilo de ejecución principal.
*   **¿Qué tiene que pasar?** Desarrollar un motor de cálculo en Rust que, al recibir nuevos spreads, actualice de forma asíncrona la expectativa matemática y los ratios de Sharpe usando buffers circulares eficientes.
*   **¿Cómo sé que está hecho?**
    - [ ] La latencia de recálculo es menor a 1ms para una cartera con 10 estrategias activas.
*   **¿Qué no puede pasar?**
    - No se deben alterar los datos del backtest original; los datos recalculados se almacenan por separado en el historial en vivo.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Todo el cálculo de expectativa y toma de decisiones defensivas reside en el nodo local.
- **Fidelidad (ADR-0017):** Fidelidad máxima. Utiliza las cotizaciones en tiempo real del broker.

### Perfil Ops / Hot-Path (ADR-0020)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID del cálculo de expectativa |
| | `created_at` | Timestamp de auditoría en nanosegundos |
| | `updated_at` | Última actualización del cálculo |
| | `audit_hash` | Hash del vector de spreads utilizado |
| | `audit_chain_hash` | Hash de la cadena de cálculos previos |
| | `event_sequence_id` | Secuencia del evento de recálculo |
| **II. Soberanía** | `owner_id` | Identificador del operador local |
| **IV. Hardware** | `node_id` | Identificador del host de ejecución |
| | `process_id` | PID de ejecución |
| | `execution_latency_ms` | Latencia de recálculo (objetivo <1ms) |
| **Rastro de Evidencia:** | Registra spreads consolidados y variaciones de expectativa para el motor de `feedback` diariamente. |
