# Time-Warp Debugger — Simulación Forense y Línea de Tiempo Event-Based

**Carpeta:** `./features/time-warp-debugger/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0011 (Operaciones Asincrónicas), ADR-0027 (Event Sourcing)

---

## ¿Qué es esta feature?

El Time-Warp Debugger es el motor de reproducción forense y depuración de Drasus Engine. Provee una línea de tiempo interactiva en la UI que permite al operador desplazar el cabezal temporal (Time-Travel) para viajar secuencialmente por la lista de **Eventos Críticos** del sistema (ej: trades ejecutados, señales de orden disparadas, y cambios de régimen de mercado HMM).

Para evitar la explosión de consumo de memoria RAM que causaría mantener millones de barras y variables del simulador activas, el debugger opera con un diseño **Event-Driven**: al posicionarse en un milisegundo exacto, el backend recarga perezosamente el valor puntual de los indicadores y el estado del libro del log de auditoría operativo persistido (SQLite WAL/Event Store), reconstruyendo el estado del sistema instantáneamente.

---

## Comportamientos Observables

- [ ] El usuario ve una línea de tiempo interactiva con marcadores de color para cada tipo de evento crítico (verde para compras, rojo para ventas, azul para cambios de régimen).
- [ ] Al desplazar el cabezal, la UI actualiza en tiempo real los valores numéricos de los indicadores en el Strategy Inspector del Nivel 3.
- [ ] Si el usuario salta a un evento de trade, el sistema muestra el estado exacto del libro de órdenes (Footprint/BBO) en ese milisegundo preciso.
- [ ] El perfil de consumo de RAM se mantiene estable y por debajo del umbral de alerta sin importar el tamaño del dataset histórico que se navegue.
- [ ] El usuario puede hacer reproducir en "Play" la línea de tiempo a velocidad controlada para ver el flujo de ejecución animado en el canvas DAG.

---

## Restricciones

- **NUNCA** cargar en memoria RAM datasets completos para la sesión de depuración; toda consulta debe ser indexada y paginada desde el Event Store.
- **NUNCA** permitir discrepancia entre los valores reconstruidos en depuración y los ejecutados históricamente (reproducibilidad determinista).
- **Límite Técnico:** Tiempo de recarga y renderizado de un punto de la línea de tiempo tras mover el cabezal debe ser inferior a 150ms.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PLAYBACK_SPEED | 1.0 | 0.1 - 10.0 | Multiplicador de velocidad de reproducción automática | CONFIG |
| PRE_FETCH_EVENTS | 50 | 10 - 500 | Cantidad de eventos pre-cargados alrededor del cabezal | CONFIG |
| MAX_DEBUGEABLE_EVENTS | 100000 | 1000 - 1000000 | Límite máximo de eventos indexados por sesión | [FIJO] |
| DART_STATE_CACHE_DURATION_MINS | 5 | 1 - 60 | Duración del caché de estados locales en la UI de Dart | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmo de reconstrucción de estado a partir de deltas de eventos, mapeo de marcas temporales y downsampling de curvas de equidad.
- **Shell (Infraestructura):** Consultas SQL vectorizadas en DuckDB con lectura selectiva de particiones Hive-style en Parquet (`hive_partitioning=true`), integradas con el Event Store en SQLite WAL y serialización Arrow hacia el puente FFI de Flutter.
- **Frontera Pública:** Interfaz para `seek_to_event(event_id)`, `get_event_log(timestamp)`, `get_backtest_results(start_date, end_date)` y emisión de streams de reproducción.

---

## Ciclo de Vida de la Feature

### Entrada
- Log de eventos históricos inmutables (Event Store).
- Archivos Parquet de mercado histórico particionados en Hive-style (`year=YYYY/month=MM/`).
- Comandos de navegación del usuario (drag del cabezal, play/pause, selectores de rango temporal y sliders).

### Proceso
- Realiza la poda de particiones (partition pruning) a nivel de sistema de archivos para ignorar carpetas Parquet fuera del rango de fechas.
- Ejecuta la consulta SQL en DuckDB sobre el path particionado, retornando Arrow tables.
- Recupera el snapshot del Event Store más cercano a la marca temporal seleccionada y aplica deltas.
- Re-hidrata el estado de las variables lógicas en memoria.

### Salida
- Estado exacto de indicadores en T-debug.
- Trades filtrados y curva de equidad con downsampling aplicado.
- Estadísticas de consulta (Particiones escaneadas, latencia).

---

## Tareas (TTRs)

### **TTR-001: Poda de Particiones y Consulta DuckDB (Rust Shell)**
*   **¿Cuál es el problema?** Realizar un escaneo completo (table scan) sobre gigabytes de archivos Parquet de transacciones históricas al cambiar el rango de fechas en la UI introduce latencias de varios segundos.
*   **¿Qué tiene que pasar?** Escribir la lógica en Rust que traduzca el rango de fechas del usuario en una consulta DuckDB filtrada que aproveche la estructura de carpetas Hive-style `/trades.parquet/year=YYYY/month=MM/` con la directiva `hive_partitioning=true`, cargando exclusivamente los archivos correspondientes a las particiones del rango.
*   **¿Cómo sé que está hecho?**
    - [ ] Las consultas de trades para rangos acotados (ej: 1 mes) se completan en menos de 200ms sobre datasets multi-GB.
    - [ ] El motor retorna en los metadatos el badge de "particiones escaneadas" indicando que solo se leyeron las carpetas correspondientes al filtro temporal.

### **TTR-002: Motor de Re-hidratación Event-Driven**
*   **¿Cuál es el problema?** Mantener el histórico completo en RAM causa desbordamientos.
*   **¿Qué tiene que pasar?** Desarrollar lógica en Rust que lea el snapshot y aplique la secuencia de deltas del log para reconstruir el estado operativo en un milisegundo seleccionado.
*   **¿Cómo sé que está hecho?**
    - [ ] Al saltar a un trade, el sistema recalcula los indicadores del nodo RSI y coinciden con la bitácora histórica.

### **TTR-003: Línea de Tiempo Interactiva (Flutter Canvas)**
*   **¿Cuál es el problema?** El operador necesita arrastrar con precisión el cabezal por millones de eventos de forma fluida.
*   **¿Qué tiene que pasar?** Implementar barra de progreso física en Flutter CustomPainter que use downsampling de densidad de eventos para pintar marcas calientes sin sobrecargar la GPU.
*   **¿Cómo sé que está hecho?**
    - [ ] Desplazar el slider temporal actualiza la UI en menos de 150ms de forma consistente.

### **TTR-004: Virtual Scrolling y Downsampling de Equidad en Flutter (Dart UI)**
*   **¿Cuál es el problema?** Intentar pintar y desplazar tablas con miles de transacciones o curvas de equidad hiper-densas causa caídas drásticas de FPS y desborda la memoria RAM en el frontend de Flutter.
*   **¿Qué tiene que pasar?** Implementar un listado de transacciones con desplazamiento virtual (virtual scrolling) en Flutter y aplicar un algoritmo de downsampling a la curva de equidad (reduciendo de 10,000 a 500 puntos antes de enviar vía FFI) para mantener el renderizado en menos de 16ms.
*   **¿Cómo sé que está hecho?**
    - [ ] La interfaz se mantiene a 120 FPS estables al arrastrar el slider temporal por un histórico de 50,000 trades.
    - [ ] El consumo de memoria RAM de la interfaz UI de Flutter se mantiene estable por debajo de 50MB durante la navegación.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil Ops / Auditoría. Registra `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`, `process_id`, `node_id`.
- **Rastro de Evidencia:** Emite logs de telemetría de rendimiento de consultas (número de particiones y latencia) para optimización en `feedback`.

---

## Dependencias
**Depende de:**
- [`audit-event-store`](../features/audit-event-store.md) — para la recuperación de eventos inmutables.
- [`zui-navigation`](../features/zui-navigation.md) — para renderizar en el Strategy Inspector del Nivel 3.
- [`hive-partition-manager`](../features/hive-partition-manager.md) — para la estructura Hive-style en disco.

**Bloquea:**
- Simulación forense visual interactiva en el frontend.
