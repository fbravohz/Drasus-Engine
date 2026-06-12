# Advanced Trade Management (ATM)

**Carpeta:** `./features/advanced-trade-management/`
**Estado:** En Diseño
**Decisiones Arquitectónicas Asociadas:** ADR-0081, ADR-0009, ADR-0020 V2, ADR-0108, ADR-0109

---

## ¿Qué es? (Explicado Simple)

Es el **gestor operativo base de transacciones**. Implementa reglas tradicionales de control de órdenes que permiten estructurar operaciones complejas más allá del simple disparo de una orden única. 

Este componente provee:
- **Grid Trading & Scaling (In/Out):** Entradas y salidas en capas o niveles escalonados.
- **Hedging Real & Anti-Martingala Clásica:** Incrementos de volumen tras operaciones ganadoras.
- **Trailing Stop Mecánico:** Modificación continua de los niveles de Stop Loss atado a un parámetro de volatilidad o porcentaje fijo.

**Primitivas de Acción del Genoma de Riesgo y Gestión de Posición (ADR-0108/ADR-0109):** `anti_martingale_scaling_factor`, `max_grid_levels` y `trailing_stop_atr_multiplier` son, respectivamente, Primitivas de Acción de Mutación de Sizing por Multiplicador, de Morfología de Salida (`Split_Position(N_Fases)`) y de Morfología de Salida (`Move_SL_To_Target`) del Dominio de Riesgo y Gestión de Posición. Cuando ese genoma no está activo en el Manifest, operan con los valores por defecto de la tabla de configuración (comportamiento actual, sin cambios). Cuando está activo, el motor evolutivo direcciona estos parámetros como nodos `wildcard_group` del dominio, disparados por sus Genes de Condición de Estado (p. ej. `Consecutive_Wins` para `anti_martingale_scaling_factor`).

---

## Comportamientos Observables

- **Modificación Dinámica de Niveles:** El sistema recalcula la distancia del Trailing Stop al cierre de cada barra y transmite la actualización al broker.
- **Pirámide de Posición (Scaling In):** A medida que el mercado avanza a favor de la posición, el gestor inyecta lotaje adicional según niveles predefinidos sin exceder el riesgo global.
- **Gestión de Cobertura (Hedging):** Permite abrir posiciones opuestas (Cortos vs Largos) sobre el mismo activo en subcuentas o tickets aislados.
- [ ] Cuando el Genoma de Riesgo y Gestión de Posición (ADR-0109) está activo, `anti_martingale_scaling_factor`, `max_grid_levels` y `trailing_stop_atr_multiplier` pueden ser resueltos por el motor evolutivo en lugar de tomar su valor por defecto.

---

## Parámetros Configurables (Configuración Tipada Serde)

Se gestiona mediante el objeto `AdvancedTradeManagementConfig`:

| Parámetro | Valor por Defecto | Qué Mide |
|---|---|---|
| `trailing_stop_atr_multiplier` | 2.5 | Multiplicador de la volatilidad para la distancia del stop |
| `max_grid_levels` | 5 | Número máximo de estratos permitidos en Grid Trading |
| `anti_martingale_scaling_factor` | 1.5 | Factor de incremento del lotaje tras una operación ganadora |

---

## Tareas (TTRs)

### TTR-001: Implementación del Trailing Stop Mecánico
- **Descripción:** Desarrollar el módulo de trailing stop que evalúe el precio actual frente al precio de entrada y mueva el nivel de Stop Loss conforme avanza el beneficio, manteniendo una distancia mínima proporcional al ATR.
- **Criterio de Éxito:** Bajo simulación de backtest, el nivel de Stop Loss aumenta estrictamente hacia arriba en posiciones de compra (Largos) sin retroceder ante correcciones temporales.

### TTR-002: Orquestador de Grid Escalonado
- **Descripción:** Diseñar el despachador de múltiples estratos operacionales. Cuando la orden principal se ejecuta, el sistema calcula automáticamente las órdenes complementarias de entrada escalonada y las envía como órdenes pendientes al broker.
- **Criterio de Éxito:** La apertura de una posición principal de 1.0 lote dispara automáticamente 4 órdenes complementarias de 0.2 lotes en niveles separados por 20 pips.

### TTR-003: Exposición como Primitivas de Acción del Genoma de Riesgo y Gestión (ADR-0108/ADR-0109)
- **¿Cuál es el problema?** Las mecánicas de Grid/Scaling, Anti-Martingala Clásica y Trailing Stop de esta feature hoy operan con valores fijos por configuración global. El Dominio de Riesgo y Gestión de Posición necesita poder direccionar estos mismos parámetros como Genes de Acción evolutivos, condicionados a sus Genes de Condición de Estado.
- **¿Qué tiene que pasar?** Cada parámetro de la tabla de configuración debe ser direccionable individualmente como un nodo `wildcard_group` del dominio de Riesgo y Gestión de Posición (ADR-0108), de forma que el motor evolutivo pueda, por ejemplo, asociar `anti_martingale_scaling_factor` a una combinación específica de `Consecutive_Wins` y `ATR_Ratio`.
- **¿Cómo sé que está hecho?**
    - [ ] Un Manifest sin Genoma de Riesgo y Gestión activo opera exactamente con los valores por defecto de esta tabla (sin regresión de comportamiento).
    - [ ] Un Manifest con Genoma de Riesgo y Gestión activo puede resolver `anti_martingale_scaling_factor` a un valor distinto del default sin afectar `trailing_stop_atr_multiplier` si este último no fue seleccionado por el genoma.
- **¿Qué no puede pasar?** Ningún parámetro de esta tabla puede ser modificado en LIVE fuera del proceso de resolución de `wildcard_group` y re-compilación del Manifest (ADR-0043).

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2):**
    - Obligatorio incluir en cada guardado: `indicator_state_hash` (estado técnico exacto al ajustar el stop), `source_signal_id` (vínculo causal con la señal original).
- **Genomas Modulares por Dominio (ADR-0108/ADR-0109):** Esta feature es Primitiva de Acción del Dominio de Riesgo y Gestión de Posición. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
- **Dependencias:** Utilizado primordialmente en `execute` y `manage`.
