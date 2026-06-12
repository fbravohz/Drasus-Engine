# Visual Downsampling Service — Reducción de Resolución Local

**Carpeta:** `./features/visual-downsampling/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0028 (ZUI Fractal Navigation)

---

## ¿Qué es esta feature?

Es un servicio de procesamiento en el backend que reduce el número de puntos de datos de una serie temporal masiva (ej: un millón de ticks) a una representación visualmente idéntica pero mucho más ligera (ej: mil puntos) antes de enviarla a la UI.

**Problema que resuelve:** El navegador no puede renderizar un millón de puntos en un gráfico sin colapsar. Enviar todos los datos es un desperdicio de ancho de banda local y CPU. Este servicio asegura que la UI sea fluida en cualquier nivel de zoom.

## Comportamientos Observables

- [ ] El usuario hace zoom out en un gráfico de 2 años y el sistema responde instantáneamente mostrando la forma general de la curva sin perder los picos de precio máximos y mínimos.
- [ ] Cuando el usuario hace zoom in (hacia el detalle), el backend envía dinámicamente datos con mayor resolución solo para el área visible.
- [ ] La curva visualmente se ve exactamente igual que la curva cruda, sin "suavizados" artificiales que oculten eventos críticos.

## Restricciones

- **NUNCA** usar promedios simples; se deben usar algoritmos que preserven los extremos (picos y valles) como **LTTB** (Largest Triangle Three Buckets).
- **Límite Técnico:** El downsampling debe ejecutarse en menos de 50ms para no penalizar la sensación de interactividad.
- **Invariante:** Los valores retornados deben existir en el dataset original (prohibido inventar puntos interpolados para visualización).

---

## Ciclo de Vida de la Feature — Visual Downsampling Service

### Entrada
- Serie temporal masiva (Precios, PnL, Indicadores).
- `target_resolution` (Número de puntos deseados, ej: 1500).
- Ventana visible (opcional, para zoom).

### Proceso
- Aplicación de algoritmo de reducción de puntos (LTTB o Min-Max Bucketing).
- Preservación de los puntos extremos de la serie dentro de cada segmento de tiempo.
- Empaquetado en formato Arrow ligero.

### Salida
- Serie temporal reducida lista para el Canvas nativo Flutter CustomPainter/Impeller.

### Contextos de Uso

**Contexto 1: Dashboard de Flota (Nivel 1)**
- Muestra minicarpas (sparklines) de rendimiento de 100 portafolios procesando miles de trades en milisegundos.

**Contexto 2: Inspector de Estrategia (Nivel 3)**
- Permite el scroll infinito y zoom en curvas de equidad detalladas de miles de barras.

---

## Tareas (TTRs)

### **TTR-001: Implementación de Algoritmo LTTB (Rust SIMD/Rayon)**
*   **¿Cuál es el problema?** El downsampling de 1M de puntos en Rust puro es lento.
*   **¿Qué tiene que pasar?** Una función acelerada con Rust SIMD/Rayon que implemente *Largest Triangle Three Buckets* para reducir la resolución preservando la forma visual.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo procesar 1M de puntos en < 20ms en una CPU estándar.

### **TTR-002: Proveedor Dinámico por Viewport**
*   **¿Cuál es el problema?** El backend necesita saber qué parte de la serie está viendo el usuario para no procesar datos innecesarios.
*   **¿Qué tiene que pasar?** Un endpoint que reciba `zoom_start`, `zoom_end` y devuelva el subset procesado.
*   **¿Cómo sé que está hecho?**
    - [ ] Al mover el zoom en la UI, el backend solo envía los datos necesarios para esa ventana.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada reducción visual registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del proceso |
| | `created_at` | Timestamp de ejecución |
| | `audit_chain_hash` | Hash de integridad visual (LTTB state) |
| **II. Soberanía** | `owner_id` | Usuario que visualiza los datos |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del algoritmo de downsampling |
| | `data_snapshot_id` | Ref al dataset crudo (input) |
| **IV. Hardware** | `node_id` | ID del hardware físico ejecutor |
| | `process_id` | PID del servicio visual |

## Dependencias y Bloqueantes
**Depende de:** `binary-arrow-transport` (para el envío).
**Bloquea:** `equity-curve-tracker`, `zui-navigation`.
