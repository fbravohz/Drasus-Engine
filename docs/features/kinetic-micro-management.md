# Kinetic Micro-Management

**Carpeta:** `./features/kinetic-micro-management/`
**Estado:** En Diseño
**Decisiones Arquitectónicas Asociadas:** ADR-0082, ADR-0020, ADR-0108, ADR-0109

---

## ¿Qué es? (Explicado Simple)

Es el **módulo defensivo hostil** de la nueva escuela. Provee protecciones reactivas agresivas y de alta velocidad diseñadas para contrarrestar la asimetría regresiva del mercado y proteger el capital ante límites estrictos de Drawdown Diario de las firmas de fondeo.

Se compone de cuatro mecánicas avanzadas:
1. **Micro-Scale Out Mandatorio:** Extracción agresiva de ganancias flotantes. Tras alcanzar un beneficio validado (ej. +1.0R), el sistema cierra automáticamente un porcentaje del volumen y traslada el Stop Loss al precio de entrada (BreakEven), eliminando el riesgo de la operación.
2. **Intra-Trade Z-Score Trailing:** Vigilancia estadística del beneficio. Si el PnL flotante excede una desviación anómala (+2.5 Z-Score), el sistema intuye una reversión rápida inminente y realiza un cierre masivo market para blindar las ganancias antes de que el mercado se gire.
3. **Tapering Logarítmico (Anti-Martingala Invertida):** Tras una racha de operaciones perdedoras consecutivas, la matriz de transacciones reduce el volumen de operación. Para regresar al volumen base, se exige que la estrategia logre un número configurable de operaciones consecutivas ganadoras.
4. **Salida por Decaimiento Temporal (Time Decay Exit):** Si una posición permanece abierta más allá de un número de barras sin alcanzar su objetivo, el sistema reduce progresivamente su volumen a una tasa configurable hasta cerrarla por completo o hasta que el SL/TP original se active primero.

**Primitivas de Acción del Genoma de Riesgo y Gestión de Posición (ADR-0108/ADR-0109):** las cuatro mecánicas de esta feature son las **Primitivas de Acción de Morfología de Salida y de Mutación de Sizing por Multiplicador** de mayor uso del Dominio de Riesgo y Gestión de Posición. Cuando un Manifest no tiene ese genoma activo, operan con sus valores por defecto configurados aquí (comportamiento actual, sin cambios). Cuando el genoma está activo, el motor evolutivo direcciona individualmente los parámetros de cada mecánica como nodos `wildcard_group` del dominio de Riesgo y Gestión, disparados por sus Genes de Condición de Estado (`Balance_Streak_Losses`, `Consecutive_Wins`, `Unrealized_R_Multiple`, `Trade_Duration_Bars`, `Distance_To_SL_Percent`).

---

## Comportamientos Observables

- **Extracción Relámpago:** El sistema cierra una porción configurable de la posición una vez alcanzado el umbral de recompensa sin esperar al Take Profit original.
- **Detección de Anomalías de Ganancia:** El monitor de PnL vivo evalúa el Z-Score de la posición en cada tick. Si el beneficio es anormalmente alto, el sistema cierra la operación a precio de mercado.
- **Reducción de Riesgo por Rachas:** Tras N Stop Loss seguidos, el sistema restringe el lotaje de las siguientes órdenes en un porcentaje configurable, y exige M operaciones ganadoras consecutivas para restaurar el volumen base.
- **Cierre Progresivo por Tiempo:** Si una posición excede el límite de barras configurado sin alcanzar su objetivo, el sistema reduce su volumen de forma incremental hasta cerrarla.
- [ ] Cuando el Genoma de Riesgo y Gestión de Posición (ADR-0109) está activo, cualquiera de los umbrales anteriores (`scale_out_profit_r_threshold`, `scale_out_volume_percent`, `tapering_consecutive_losses`, `tapering_volume_reduction_pct`, `tapering_recovery_wins_required`, `time_decay_exit_bars_limit`, `time_decay_exit_reduction_rate`, `z_score_trailing_threshold`) puede ser resuelto por el motor evolutivo en lugar de tomar su valor por defecto.

---

## Parámetros Configurables (Configuración Tipada Serde)

Se gestiona a través del objeto `KineticMicroManagementConfig`:

| Parámetro | Valor por Defecto | Qué Mide |
|---|---|---|
| `scale_out_profit_r_threshold` | 1.0 | Nivel de recompensa (R) para activar el Scale Out y BreakEven |
| `scale_out_volume_percent` | 50% | Porcentaje del volumen cerrado al activarse el Scale Out |
| `z_score_trailing_threshold` | 2.5 | Desviación estándar del PnL vivo para ejecutar el cierre masivo |
| `tapering_consecutive_losses` | 3 | Rachas perdedoras necesarias para activar la reducción de volumen |
| `tapering_volume_reduction_pct` | 75% | Porcentaje de reducción de volumen aplicado tras la racha de pérdidas |
| `tapering_recovery_wins_required` | 2 | Operaciones ganadoras consecutivas necesarias para restaurar el volumen base |
| `time_decay_exit_bars_limit` | 20 | Número de barras sin alcanzar el objetivo antes de iniciar el cierre progresivo |
| `time_decay_exit_reduction_rate` | 10% | Porcentaje del volumen restante reducido por cada barra adicional tras superar el límite |

---

## Tareas (TTRs)

### TTR-001: Implementación del Micro-Scale Out
- **Descripción:** Desarrollar el interceptor de beneficios. Cuando el PnL flotante de una posición activa alcanza el valor equivalente al riesgo original de la operación (`scale_out_profit_r_threshold`), el sistema envía una orden a mercado para cerrar `scale_out_volume_percent` del volumen y emite una orden de modificación del Stop Loss hacia el punto de equilibrio (`BreakEven`).
- **Criterio de Éxito:** Una posición de 2 lotes con riesgo de 100 USD cierra 1 lote y mueve el stop al precio de entrada tan pronto como el beneficio flotante alcanza 100 USD (con valores por defecto).

### TTR-002: Monitor Intra-Trade Z-Score Trailing
- **Descripción:** Implementar el monitor analítico que calcule continuamente el Z-Score de la posición abierta frente a la distribución de retornos histórica. Si el PnL de la posición supera `z_score_trailing_threshold`, se ejecuta un cierre inmediato a mercado.
- **Criterio de Éxito:** Un movimiento extremo a favor de la operación gatilla un cierre a mercado al instante sin esperar a que el precio alcance el Take Profit original.

### TTR-003: Exposición como Primitivas de Acción del Genoma de Riesgo y Gestión (ADR-0108/ADR-0109)
- **¿Cuál es el problema?** Las cuatro mecánicas de esta feature (Scale Out, Z-Score Trailing, Tapering, Time Decay Exit) hoy operan con umbrales fijos por configuración global. El Dominio de Riesgo y Gestión de Posición necesita poder direccionar estos mismos umbrales como Genes de Acción evolutivos, condicionados a sus Genes de Condición de Estado.
- **¿Qué tiene que pasar?** Cada parámetro de esta tabla debe ser direccionable individualmente como un nodo `wildcard_group` del dominio de Riesgo y Gestión de Posición (ADR-0108), de forma que el motor evolutivo pueda, por ejemplo, asociar `tapering_volume_reduction_pct` y `tapering_recovery_wins_required` a una combinación específica de `Balance_Streak_Losses` y `ATR_Ratio`.
- **¿Cómo sé que está hecho?**
    - [ ] Un Manifest sin Genoma de Riesgo y Gestión activo opera exactamente con los valores por defecto de esta tabla (sin regresión de comportamiento).
    - [ ] Un Manifest con Genoma de Riesgo y Gestión activo puede resolver `tapering_volume_reduction_pct` a un valor distinto del default sin afectar `scale_out_volume_percent` si este último no fue seleccionado por el genoma.
- **¿Qué no puede pasar?** Ningún parámetro de esta tabla puede ser modificado en LIVE fuera del proceso de resolución de `wildcard_group` y re-compilación del Manifest (ADR-0043).

### TTR-004: Salida por Decaimiento Temporal (Time Decay Exit)
- **Descripción:** Implementar el monitor de duración de posición que, al superar `time_decay_exit_bars_limit` barras sin que la operación alcance su objetivo, reduce el volumen restante en `time_decay_exit_reduction_rate` por cada barra adicional, hasta el cierre completo o hasta que el SL/TP original se active primero.
- **Criterio de Éxito:** Una posición que permanece abierta 5 barras por encima del límite configurado, con una tasa de reducción del 10%, ve su volumen reducido progresivamente en cada una de esas barras sin alterar el SL/TP original mientras este no se alcance.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020):** **Perfil C (Ops / Hot-Path)** — gestión intra-trade en ruta crítica de ejecución.

  | Categoría | Campo | Descripción |
  | :--- | :--- | :--- |
  | **I. Identidad** | `id` | Identificador único del evento de micro-gestión |
  | | `created_at` | Timestamp del ajuste (nanosegundos) |
  | | `updated_at` | Timestamp de última modificación del registro |
  | | `audit_hash` | Hash de integridad del estado del trade |
  | | `audit_chain_hash` | Hash encadenado del historial de ajustes |
  | | `event_sequence_id` | Secuencia de recuperación exacta post-reinicio |
  | **II. Soberanía** | `owner_id` | Dueño de la posición gestionada |
  | | `manifest_id` | Estrategia de origen |
  | **IV. Hardware** | `node_id` | ID del hardware físico ejecutor |
  | | `process_id` | PID del worker de ejecución |
  | **V. Forense & Ejecución (latencia)** | `execution_latency_ms` | Latencia del ajuste intra-trade |
  | | `source_signal_id` | Señal de origen del scale-out / trailing |
  | | `indicator_state_hash` | Z-Score calculado en el momento del ajuste (Grupo V) |
- **Genomas Modulares por Dominio (ADR-0108/ADR-0109):** Esta feature es Primitiva de Acción del Dominio de Riesgo y Gestión de Posición. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
- **Dependencias:** Utilizado primordialmente en `execute` y `feedback`.
