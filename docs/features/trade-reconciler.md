# Trade Reconciler

**Carpeta:** `./features/trade-reconciler/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente encargado de la **Feedback de ejecución de la Operativa**. Su misión es la reconciliación diaria: compara la realidad cruda del broker (Fills reales) contra la expectativa del sistema (Backtest/Paper), calibrando los modelos de fricción para que la simulación futura sea siempre más exacta.

---

## Comportamientos Observables

- [ ] Compara cada precio de ejecución real contra el precio solicitado (slippage real).
- [ ] Calcula el **Spread Real Promedio** incurrido durante la sesión.
- [ ] Sugiere ajustes automáticos a los parámetros de fricción del sistema.

---

## Ciclo de Vida de la Feature — Trade Reconciler

### Entrada
- Lista de órdenes ejecutadas en el Broker (Fills).
- Lista de señales/órdenes solicitadas por las estrategias.
- Configuración de tolerancia de spread.

### Proceso
- Alínea cada orden enviada con su ejecución correspondiente en el broker.
- Calcula la diferencia de precio (Slippage) y la desviación temporal.
- Compara los costos de transacción reales (comisiones + spread) vs los estimados.

### Salida
- **Reporte de Reconciliación Diario:** Consolidado de costos.
- **Sugerencias de Calibración:** Valores recomendados de slippage/spread para el próximo ciclo.

### Contextos de Uso
**Contexto Único: Retroalimentación (Módulo Feedback)**
- Cierra el círculo de aprendizaje corrigiendo los supuestos de los módulos de generación y validación.

---

## Tareas (TTRs) — Herencia de Módulo Retroalimentar

### TTR-001: Reconciliación Diaria de Fills
*   **Descripción:** Proceso EOD que barre el diario de órdenes.
*   **Criterio de Éxito:** Detección de desviaciones de costo sistemáticas.

### TTR-002: Calibración de Modelos de Fricción
*   **Descripción:** Traducción de realidad en parámetros para el `backtest-engine`.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. La reconciliación de costos es un dato operativo sensible.
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada reporte de reconciliación registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la reconciliación |
| | `created_at` | Timestamp de autopsia (EOD) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del reporte de discrepancias |
| | `audit_chain_hash` | Hash de la secuencia de auditoría |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Dueño operativo del capital |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor reconciliador |
| | `data_snapshot_id` | Ref al snapshot del Broker vs Local |
| | `indicator_state_hash` | Delta de slippage/comisiones calculado |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de auditoría |


---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Algoritmos de alineación de órdenes y cálculo de dispersión en `reconciler.rs`.
- **Shell (Infraestructura):** Integrador de logs de broker y logs de sistema.
- **Frontera Pública:** Contrato `reconcile_session(session_id)`.

---

## Dependencias
**Consumido por:** `feedback`.
**Depende de:** `institutional-metrics`, `audit-log`.
