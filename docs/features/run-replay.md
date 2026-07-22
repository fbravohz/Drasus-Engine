# Repetición de Corrida

**Carpeta:** `./features/run-replay/`
**Estado:** Lista para implementar
**Última actualización:** 2026-07-21
**Decisión Arquitectónica Asociada:** ADR-0157 (Visualización de Estrategia — un motor de render, dos entradas separadas), ADR-0027 (Event Sourcing), ADR-0011 (Operaciones Asincrónicas)

> **Renombre 2026-07-21:** esta feature se llamaba "Time-Warp Debugger — Simulación Forense". El nombre viejo viola la prohibición del propietario sobre el campo semántico de la muerte, y su especificación estaba atada al canvas descontinuado de tres niveles. Se refresca al canvas actual (ADR-0136/0137), se vuelve agnóstica al tipo de barra y suma el inspector de barra estilo StrategyQuant X.

---

## ¿Qué es esta feature?

Repetición de Corrida es el motor de **reproducción y revisión** de Drasus Engine: rebobinar una corrida **ya terminada** (backtest o sesión), plantar el cabezal en cualquier instante — hasta el milisegundo — y ver el estado exacto reconstruido en ese momento. Es hacia atrás: *"déjame re-examinar qué pasó"*, incluido cazar un error o auditar por qué una estrategia hizo lo que hizo.

Para evitar la explosión de RAM que causaría mantener millones de barras y variables en memoria, opera con un diseño **Event-Driven**: al posicionarse en un instante exacto, el backend recarga perezosamente el valor puntual de los indicadores y el estado del libro desde el almacén de eventos persistido (Event Store), reconstruyendo el estado instantáneamente. Comparte el motor de render con [`live-strategy-view`](./live-strategy-view.md) (Vista en Vivo), pero se invoca distinto y sirve otro propósito.

---

## Comportamientos Observables

- [ ] El usuario abre un nodo **Repetición de Corrida** sobre una corrida terminada y ve una línea de tiempo con marcadores de color por tipo de evento (compra, venta, cambio de régimen).
- [ ] Al desplazar el cabezal, la UI actualiza en tiempo real los valores numéricos de los indicadores en el inspector.
- [ ] Al pasar el cursor sobre una barra, una tabla muestra sus valores (`open/high/low/close`, volumen, metadatos) **y el cálculo exacto de cada indicador de la estrategia en esa barra** (inspector estilo StrategyQuant X).
- [ ] El render funciona sobre **cualquier tipo de barra** (velas, Renko, Rango, Volumen, Tick, Dólar…), no solo velas japonesas.
- [ ] El usuario puede darle "Play" para ver la corrida grabada animarse a velocidad controlada.
- [ ] El consumo de RAM se mantiene estable sin importar el tamaño del histórico navegado.

---

## Restricciones

- **NUNCA** cargar datasets completos en RAM para la sesión; toda consulta indexada y paginada desde el Event Store.
- **NUNCA** discrepancia entre los valores reconstruidos y los ejecutados históricamente (reproducibilidad determinista, hereda ADR-0114).
- **NUNCA** términos del campo semántico de la muerte en el nombre de la feature o sus nodos.
- **Límite Técnico:** recargar y renderizar un punto de la línea de tiempo tras mover el cabezal debe ser inferior a 150ms.

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PLAYBACK_SPEED | 1.0 | 0.1 - 10.0 | Multiplicador de velocidad de reproducción automática | CONFIG |
| PRE_FETCH_EVENTS | 50 | 10 - 500 | Eventos pre-cargados alrededor del cabezal | CONFIG |
| MAX_REPLAYABLE_EVENTS | 100000 | 1000 - 1000000 | Límite de eventos indexados por sesión | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** reconstrucción de estado a partir de deltas de eventos, mapeo de marcas temporales y downsampling de curvas de equidad.
- **Shell (Infraestructura):** consultas vectorizadas en DuckDB con lectura selectiva de particiones Hive-style en Parquet, integradas con el Event Store; serialización Arrow hacia el puente FFI.
- **Frontera Pública:** interfaz para posicionar el cabezal en un evento/instante, obtener el log de eventos y emitir streams de reproducción.

---

## Ciclo de Vida de la Feature — Repetición de Corrida

### Entrada
- Log de eventos históricos inmutables (Event Store) de una corrida terminada.
- Archivos Parquet de mercado histórico particionados en Hive-style.
- Comandos de navegación del usuario (arrastre del cabezal, play/pausa, rangos).

### Proceso
- Poda de particiones a nivel de sistema de archivos para ignorar carpetas fuera del rango.
- Recupera el snapshot más cercano al instante seleccionado y aplica deltas.
- Re-hidrata el estado de las variables lógicas y proyecta el render sobre el tipo de barra activo.

### Salida
- Estado exacto de indicadores en el instante seleccionado, con inspector de barra.
- Trades y curva de equidad con downsampling aplicado.

### Contextos de Uso
- **Revisión y depuración (feedback):** reproducir una corrida para entender un fallo o una anomalía.
- **Revisión de paper trades (incubate):** revisar lo ocurrido en incubación.
- **Navegación temporal (validate):** inspeccionar una corrida de validación.

---

## Tareas (TTRs)

### **TTR-001: Poda de Particiones y Consulta DuckDB (Rust Shell)**
*   **¿Cuál es el problema?** Un escaneo completo sobre gigabytes de Parquet al cambiar el rango introduce latencias de segundos.
*   **¿Qué tiene que pasar?** Traducir el rango de fechas del usuario en una consulta DuckDB que aproveche la estructura Hive-style, cargando solo las particiones del rango.
*   **¿Cómo sé que está hecho?**
    - [ ] Consultas de trades para rangos acotados se completan en menos de 200ms sobre datasets multi-GB.
    - [ ] El motor retorna el badge de "particiones escaneadas".

### **TTR-002: Motor de Re-hidratación Event-Driven**
*   **¿Cuál es el problema?** Mantener el histórico completo en RAM causa desbordamientos.
*   **¿Qué tiene que pasar?** Leer el snapshot y aplicar la secuencia de deltas para reconstruir el estado en un instante seleccionado.
*   **¿Cómo sé que está hecho?**
    - [ ] Al saltar a un trade, los indicadores recalculados coinciden con la bitácora histórica.

### **TTR-003: Línea de Tiempo Interactiva con Inspector de Barra (Flutter Canvas)**
*   **¿Cuál es el problema?** El operador necesita arrastrar el cabezal por millones de eventos de forma fluida y ver el cálculo de indicadores al pasar el cursor sobre una barra.
*   **¿Qué tiene que pasar?** Barra de progreso en CustomPainter con downsampling de densidad de eventos; tabla de inspector de barra con OHLCV + indicadores, agnóstica al tipo de barra.
*   **¿Cómo sé que está hecho?**
    - [ ] Desplazar el slider actualiza la UI en menos de 150ms de forma consistente.
    - [ ] El inspector muestra el valor exacto de cada indicador en la barra bajo el cursor.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `bars_in` | `Bars` | Input | `1..N` | Series de barras de la corrida reconstruida (cualquier `bar_type`) |
| `signal_in` | `Signal` | Input | `0..N` | Señales históricas de la corrida |
| `order_in` | `Order` | Input | `0..N` | Órdenes ejecutadas en la corrida |
| `position_in` | `Position` | Input | `0..N` | Estado de posición reconstruido por instante |

> Observador terminal (nodo de canvas / Inspector Panel): reconstruye y renderiza; no produce un dato de puerto hacia el pipeline.

## Cáscara Visual (Thin Shell)

> **Pendiente de la Etapa 0.5** — la produce el skill `ui-designer` leyendo `docs/DESIGN.md`. No rellenar a mano.

### Contexto de superficie (ADR-0136)
**Canvas — Vista Interior / Inspector Panel** — nodo "Repetición de Corrida" con línea de tiempo, cabezal e inspector de barra.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil D (Ops / Auditoría) — Grupo I completo (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) + `event_sequence_id`, `process_id`, `node_id`.
- **Rastro de Evidencia:** emite telemetría de rendimiento de consultas (particiones escaneadas, latencia) para optimización en `feedback`.

## Dependencias y Bloqueantes
**Depende de:**
- [`audit-event-store`](./audit-event-store.md) — recuperación de eventos inmutables.
- [`hive-partition-manager`](./hive-partition-manager.md) — estructura Hive-style en disco.
- [`live-strategy-view`](./live-strategy-view.md) — motor de render compartido.

**Contrato de Integración UI (ADR-0117):**
- **Superficie propia:** nodo/pestaña "Repetición de Corrida". SVF: abrir una corrida terminada reconstruye su estado vía `public_interface`; el usuario arrastra el cabezal y ve indicadores y el inspector de barra; tras recargar, la corrida sigue disponible para reproducción.
