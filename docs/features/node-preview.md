# Micro-Backtest Node Preview — Vista Previa Local de Nodos

**Carpeta:** `./features/node-preview/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0096 (Caché de Previews Locales de Nodo para Iteración Rápida)

---

## ¿Qué es esta feature?

El Micro-Backtest Node Preview provee al operador de Drasus Engine una retroalimentación visual instantánea de la curva de balance y métricas de rendimiento simplificadas de una estrategia directamente al seleccionar un nodo lógico en el Strategy Inspector del Nivel 3.

Para evitar la demora asociada a un pipeline de validación completo (que requiere correr Monte Carlo y análisis Walk-Forward exhaustivos en datasets multi-año), esta característica genera una simulación rápida sobre una ventana acotada y almacena los resultados en una caché local (`node_preview_cache` JSON blob) en SQLite. Si se edita cualquier parámetro de un nodo, el sistema invalida reactivamente esta caché y solicita su regeneración de forma asíncrona.

---

## Comportamientos Observables

- [ ] Al seleccionar un nodo en el Strategy Inspector del Nivel 3, el inspector contextual muestra un mini-gráfico de equidad (línea simplificada de 50 puntos) y métricas básicas (Sharpe, Profit Factor).
- [ ] Si los datos de vista previa existen en SQLite, la renderización de la curva en el frontend de Flutter se completa de forma instantánea (<5ms) desde la memoria local.
- [ ] Si el operador modifica algún parámetro micro de un nodo (ej: aumentar el período de un indicador), la curva de equidad del nodo y sus sucesores lógicos en el grafo se vuelven opacos (gris/rojo) con el mensaje "Caché de Vista Previa Inválido".
- [ ] Al presionar el botón "Regenerate Preview" en un nodo invalidado, se inicia una tarea asíncrona en segundo plano. Un indicador de carga (spinner) se muestra en el nodo sin bloquear el desplazamiento o interacción del usuario en el lienzo.
- [ ] Al finalizar la simulación, el mini-gráfico se actualiza con la nueva curva de equidad y las métricas calculadas.

---

## Restricciones

- **NUNCA** ejecutar la simulación de vista previa dentro del hilo principal de la interfaz de usuario; todo cálculo de backtest rápido debe delegarse de forma asíncrona a un hilo de baja prioridad del orquestador Rust.
- **NUNCA** permitir la promoción de una estrategia a incubación o producción si tiene bloques lógicos con caché de vista previa invalidada o desactualizada.
- **Límite Técnico:** La generación asíncrona de la vista previa sobre la ventana de backtest rápido (30 días, M15) debe completarse en menos de 10 segundos.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PREVIEW_HISTORY_DAYS | 30 | 10 - 90 | Días de histórico de datos de mercado utilizados para la simulación rápida | CONFIG |
| PREVIEW_TIMEFRAME | M15 | M1 - H1 | Temporalidad de barras OHLCV para la ejecución de la vista previa rápida | [FIJO] |
| DOWNSAMPLE_POINTS | 50 | 20 - 200 | Número de puntos de la equidad proyectados en el mini-gráfico de la interfaz | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmo de invalidación de dependencias del grafo AST (invalidación en cascada) y interpolación/downsampling de curvas de equidad.
- **Shell (Infraestructura):** Motor de micro-backtesting simplificado (ejecución sin fricción de slippage dinámico ni Monte Carlo), consultas y almacenamiento de blobs JSON en la tabla `strategies` de SQLite local.
- **Frontera Pública:** API FFI/gRPC para consultar la caché de vista previa (mapeado a `get_node_preview` en FFI/gRPC), despachar la regeneración (`generate_node_preview`) y emitir eventos de invalidación del AST.

---

## Ciclo de Vida de la Feature

### Entrada
- El identificador único de la estrategia y la topología del nodo seleccionado.
- Datos de mercado OHLCV pre-cargados (30 días históricos en M15).
- Evento de edición de parámetros emitido por el inspector de Flutter.

### Proceso
- Valida la integridad del nodo en el AST global.
- Recupera el blob JSON cacheado si está marcado como válido.
- En caso de invalidación, ejecuta la simulación determinista rápida y reduce (downsamples) el vector de equidad resultante a $N$ puntos coordenados.
- Actualiza el registro de persistencia local.

### Salida
- Curva de equidad simplificada y métricas (Sharpe, Profit Factor) listas para FFI.
- Estado de validez del nodo en el canvas de Flutter.

---

## Tareas (TTRs)

### **TTR-001: Gestor de Estado e Invalidación en Cascada del AST (Rust Core)**
*   **¿Cuál es el problema?** Si un usuario modifica un nodo base, los nodos descendientes siguen mostrando resultados desactualizados, lo que genera conclusiones de optimización erróneas.
*   **¿Qué tiene que pasar?** Implementar la lógica del grafo en Rust que, al recibir un cambio de parámetro en un nodo, marque recursivamente como inválidos (`preview = null`) los cachés de vistas previas de ese nodo y de todos sus dependientes directos e indirectos en el AST.
*   **¿Cómo sé que está hecho?**
    - [ ] Al cambiar el input del nodo A, las consultas de vista previa de los nodos descendientes B y C devuelven `cached: false` de inmediato.

### **TTR-002: Motor de Simulación Rápida Local (Rust Shell)**
*   **¿Cuál es el problema?** El backtesting tradicional de Drasus Engine es demasiado robusto y pesado para ofrecer feedback rápido a nivel de edición de nodos.
*   **¿Qué tiene que pasar?** Crear un motor de simulación rápida en Rust que consuma barras OHLCV indexadas en SQLite/Parquet, omitiendo simulaciones de latencia, comisiones de broker y simulaciones estocásticas Monte Carlo para maximizar el throughput.
*   **¿Cómo sé que está hecho?**
    - [ ] La ejecución del micro-backtest de 30 días en M15 para una estrategia estándar se completa en menos de 5 segundos.

### **TTR-003: Integración de Persistencia y Caché SQLite (Rust persistence)**
*   **¿Cuál es el problema?** Recalcular las curvas de equidad cada vez que el usuario navega o selecciona nodos en el editor introduce una latencia inaceptable.
*   **¿Qué tiene que pasar?** Agregar la columna `node_preview_cache` de tipo TEXT/JSON a la base de datos SQLite transaccional local, y desarrollar los métodos de repositorio para guardar y extraer la curva reducida y métricas de rendimiento asociadas.
*   **¿Cómo sé que está hecho?**
    - [ ] Al consultar la vista previa de un nodo previamente calculado, los datos se leen de SQLite en menos de 2ms.

### **TTR-004: Componente Mini Chart y Hooks en Flutter (Dart UI - NodePreviewChart & useNodePreview)**
*   **¿Cuál es el problema?** La visualización del rendimiento del nodo debe estar contenida en el espacio limitado del inspector sin degradar los FPS de renderizado del lienzo, y conectarse reactivamente al backend.
*   **¿Qué tiene que pasar?** Implementar el widget `NodePreviewChart.dart` utilizando CustomPainter de Flutter para renderizar curvas de equidad simplificadas, y el hook/provider `useNodePreview.dart` para conectarse a las llamadas FFI/gRPC del orquestador Rust, reaccionando a los estados de validez (coloración gris/rojo con botón de regeneración asíncrona).
*   **¿Cómo sé que está hecho?**
    - [ ] El inspector de Flutter renderiza el gráfico simplificado y el spinner asíncrono manteniendo el lienzo del DAG a 120 FPS estables.
    - [ ] El hook maneja de forma reactiva la invalidación y el spinner de carga al vuelo.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. No realiza llamadas de red; consume el databank local SQLite.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil IA / R&D. Registra `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`, `version_node_id`, `logic_hash`, `node_id`, `execution_latency_ms`.
- **Rastro de Evidencia:** Emite logs de telemetría sobre la duración del cálculo rápido y la tasa de aciertos de la caché para el módulo de `feedback`.

---

## Dependencias

**Depende de:**
- [`visual-dag-editor`](../features/visual-dag-editor.md) — para la visualización en el lienzo Nivel 3.
- [`databank-manager`](../features/databank-manager.md) — para la lectura local de datos históricos.

**Bloquea:**
- [`generate`](../modules/generate.md) — la optimización interactiva de nodos en el lienzo visual requiere este feedback.
