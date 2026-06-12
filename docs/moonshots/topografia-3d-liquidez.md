# Topografía 3D de Liquidez

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0097 (Renderizado Gráfico Multidimensional Nativo sin WebViews)

---

## ¿Qué es?

Es un modo de visualización avanzado que renderiza el historial del Order Book y las zonas de liquidez acumuladas como un modelo tridimensional (3D) navegable en tiempo real. Los "muros" de órdenes límite (órdenes pendientes en el DOM) se proyectan como montañas topográficas cuya altura y color indican la concentración de volumen, permitiendo al trader identificar visualmente niveles de soporte, resistencia e intenciones de manipulación (spoofing) de forma espacial e intuitiva.

---

## Comportamientos Observables

- [ ] **Lienzo Topográfico Acelerado:** La UI de Flutter dibuja una malla 3D sobre la que el usuario puede realizar paneos, rotaciones y zooms de forma fluida.
- [ ] **Mapeo Espacial de Liquidez:** El eje X representa el tiempo cronológico, el eje Y representa la escala de precios y el eje Z (altura de la malla) representa la densidad de volumen acumulado de órdenes límite.
- [ ] **Gradiente de Calor:** Los niveles se colorean dinámicamente de acuerdo al volumen (ej. amarillo/rojo para alta densidad de liquidez, azul/violeta para zonas frías de baja liquidez).
- [ ] **Filtro de Spoofing Dinámico:** Al deslizar un control de escala temporal, la topografía elimina los muros de liquidez efímeros que duraron menos de $N$ milisegundos en el libro, aislando las intenciones de órdenes reales.

---

## Restricciones

- **OBLIGATORIO:** Utilizar el motor gráfico nativo GPU de Flutter (Impeller) a través de `CustomPainter` o shaders de GPU personalizados. Prohibido el uso de WebViews o scripts de representación 3D de navegadores (Three.js, Plotly).
- **NUNCA** bloquear el hilo de ejecución principal durante el procesamiento de matrices de profundidad; el filtrado y downsampling de datos de liquidez debe completarse en Rust.
- **FIJO:** Los datos transmitidos desde Rust se envían en formato binario Apache Arrow de memoria compartida para evitar sobrecostos de serialización.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| MAX_TOPOGRAPHY_POINTS | 50000 | 10000 - 200000 | Cantidad máxima de vértices 3D a renderizar en pantalla | CONFIG |
| MIN_L2_DURATION_MS | 500 | 50 - 5000 | Duración mínima en el DOM para incluir volumen en la topografía | CONFIG |
| VERTICAL_SCALE_Z | 1.0 | 0.1 - 5.0 | Multiplicador de escala de altura para el volumen de órdenes | CONFIG |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Procesamiento de matrices de orden book histórico en Polars/Rust. Agrupación y filtrado temporal de órdenes pendientes.
- **Shell (Infraestructura):** Envío de buffers binarios vía FFI y renderizado 3D acelerado en Flutter.

---

## Ciclo de Vida de la Feature — Topografía 3D de Liquidez

### Entrada
- Histórico de datos L2 de libro de órdenes (Parquet/Polars).
- Rango temporal y de precios seleccionado por el usuario.

### Proceso
- Rust filtra y reduce la resolución de la matriz bidimensional (Precio x Tiempo) de volumen de órdenes límite.
- Remuestreo y mapeo de altura Z por densidad de volumen.
- Transferencia del array de vértices mediante memoria compartida FFI.
- Flutter dibuja la malla con proyección en perspectiva 3D mediante la API de Impeller GPU.

### Salida
- Representación visual tridimensional interactiva en el Dashboard.

---

## Tareas (TTRs)

### **TTR-001: Procesador de Densidad L2 en Rust**
*   **¿Cuál es el problema?** Los datos crudos de Nivel 2 (DOM) contienen millones de actualizaciones por hora, lo que causaría caídas de fotogramas masivas si se intentan dibujar directamente en la UI.
*   **¿Qué tiene que pasar?** Desarrollar un filtro en el backend Rust utilizando Polars que agrupe los datos de órdenes límite por bloques de precio y ventanas de tiempo (ej. 1 segundo). Se descartan las órdenes con duraciones cortas (spoofing) y se exporta una matriz compacta y estructurada de volumen.
*   **¿Cómo sé que está hecho?**
    - [ ] Rust entrega un buffer de datos remuestreados en menos de 50ms para un rango de 1 hora de datos.
    - [ ] El log de auditoría registra la reducción de registros (ej: de 5 millones a 20,000 puntos de renderizado).
*   **¿Qué no puede pasar?**
    - El buffer no debe contener valores infinitos o nulos que corrompan el renderizado geométrico de la UI.

### **TTR-002: Renderizador de Malla 3D en Flutter con Impeller**
*   **¿Cuál es el problema?** El uso de WebViews con librerías JavaScript consume demasiados recursos de memoria y CPU, y no encaja en la interfaz de usuario fluida del monolito.
*   **¿Qué tiene que pasar?** Implementar un pintor personalizado (`CustomPainter`) o shader de sombreado (GLSL) en Flutter que reciba el vector de vértices binarios desde Rust y dibuje la topografía 3D de forma directa con transformaciones de matriz de perspectiva en la GPU.
*   **¿Cómo sé que está hecho?**
    - [ ] El gráfico 3D gira y responde a los movimientos de arrastre del mouse de manera fluida (>60 FPS).
    - [ ] Se distingue visualmente la elevación montañosa en los precios donde se concentran los muros de órdenes del libro.
*   **¿Qué no puede pasar?**
    - No se deben utilizar hilos de Dart bloqueantes para calcular la proyección tridimensional.

---

## Gobernanza y Estándares (ADR-0020 V2)

### Perfil Ops / Hot-Path
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID del buffer de renderizado 3D |
| | `created_at` | Timestamp de generación del modelo |
| | `audit_hash` | Hash de la matriz de volumen transferida |
| **II. Soberanía** | `owner_id` | Identificador del usuario local |
| **IV. Hardware** | `node_id` | Identificador de dispositivo GPU |
| | `process_id` | PID de ejecución |
| | `execution_latency_ms` | Latencia de cálculo de Rust + dibujo de fotograma |
