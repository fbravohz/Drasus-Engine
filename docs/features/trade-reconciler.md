# Trade Reconciler

**Carpeta:** `./features/trade-reconciler/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente encargado de la **Feedback de ejecución de la Operativa**. Su misión es la reconciliación diaria: compara la realidad cruda del broker (Fills reales) contra la expectativa del sistema (Backtest/Paper), calibrando los modelos de fricción para que la simulación futura sea siempre más exacta.

---

## Narrativa de Producto

**Capacidad:** `CAP-08` — El ciclo que se cierra solo

**Historia de usuario:** Como operador con estrategias en cuenta real, quiero que el sistema compare cada operación ejecutada contra lo que el backtest predijo, para saber si la fricción real se parece a la que asumí y corregirla antes de que la brecha crezca sola.

**El problema (técnico):** todo backtest asume un modelo de fricción (deslizamiento, comisión, spread) calibrado con datos pasados o supuestos razonables. Ese modelo empieza a envejecer desde el primer día de operación real: la liquidez cambia, el bróker cambia su ejecución, el spread se comporta distinto en momentos de alta volatilidad. Sin una comparación diaria y sistemática entre lo ejecutado y lo esperado, la brecha entre simulación y realidad crece en silencio y nadie la nota hasta que el resultado en vivo decepciona sin explicación aparente.

**La solución (técnica):** un proceso de reconciliación diaria (fin de sesión) que alinea cada orden solicitada con su ejecución real en el bróker, calcula el deslizamiento real y el spread real promedio incurrido, y compara los costos de transacción reales contra los que el motor de simulación estimó. Cuando la desviación es sistemática, produce sugerencias de calibración concretas —nuevos valores de deslizamiento y spread— que alimentan de vuelta al motor de backtest, para que la próxima simulación sea más honesta que la anterior.

**Explicado sin tecnicismos:** imagina que calculaste el costo de un viaje en taxi usando la tarifa del año pasado. El primer día que subes al taxi real, pagas un poco más — el tráfico cambió, el precio de la gasolina cambió. Si no comparas nunca el recibo real contra tu cálculo, sigues presupuestando mal viaje tras viaje, y con el tiempo la diferencia se hace grande sin que la notes.

Esta pieza es la que revisa el recibo real todas las noches y lo compara contra lo que habías calculado. Si la diferencia se repite, ajusta la calculadora para el día siguiente — así el presupuesto se parece cada vez más a lo que de verdad vas a pagar.

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
## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada reporte de reconciliación registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la reconciliación |
| | `created_at` | Timestamp del análisis de cierre diario |
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
