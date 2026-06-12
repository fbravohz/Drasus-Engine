# Visual DAG Editor — Orquestación de Canales Sin Código

**Carpeta:** `./features/visual-dag-editor/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0022 (Pipeline No-Lineal (DAG Multiflujal)), ADR-0028 (ZUI Fractal Navigation)

---

## ¿Qué es esta feature?

El Visual DAG Editor es la herramienta de diseño gráfico y configuración de alto nivel de Drasus Engine. Utiliza un lienzo interactivo renderizado por GPU mediante **Flutter CustomPainter** (Impeller) que permite al usuario arrastrar, conectar y bifurcar bloques lógicos (nodos) de la estrategia. 

Mapea visualmente la lógica a través de tres niveles de profundidad (ZUI):
1. **Nivel 1 (Macro): Fleet Command:** Vista de gestión de portafolios agregada.
2. **Nivel 2 (Meso): La Fábrica Visual (Pipeline CI/CD Quant):** Un pipeline horizontal continuo donde se arrastran bloques de control para la validación progresiva de estrategias: Generador Genético, Filtro Base y Optimizador Secuencial (con Coordenadas Paralelas y UMAP 3D), Walk-Forward Analysis (WFA) y Simulador de Montecarlo.
3. **Nivel 3 (Micro): El ADN y los Indicadores:** Editor de Visual Scripting puro que expone bloques micro (`Data Source`, `Signal Blocks`, `Indicator Blocks`, `Execution Blocks`), un inspector de ultra-bajo nivel con toggle de optimización, plantillas base (Templates) y un catálogo completo de nodos clasificados.

La interfaz implementa un **Motor de Estado UI** acoplado al JSON AST global con un **Mecanismo de Invalidación Estricta**: si se edita cualquier parámetro micro en el Nivel 3, el sistema invalida y marca visualmente en rojo los resultados de robustez asociados en el Nivel 2, obligando al operador a re-evaluar la estrategia.

---

## Comportamientos Observables

- [ ] El usuario puede arrastrar bloques macro en la Fábrica Visual (Nivel 2) para reconfigurar el pipeline de validación (ej. mover Montecarlo antes de WFA).
- [ ] Al hacer doble clic en una estrategia aprobada en el Nivel 2, la UI realiza zoom dinámico al Nivel 3 expeliendo el lienzo de Visual Scripting micro.
- [ ] Si el usuario modifica el período de un RSI en el Nivel 3, todos los bloques del pipeline de la Fábrica Visual en el Nivel 2 para esa estrategia se colorean en rojo indicando "Caché de Validación Invalido".
- [ ] En el Nivel 3, el usuario puede arrastrar nodos de categorías como `Candle Patterns` o `Time Filters` y conectarlos mediante flujos tipados.
- [ ] Cada parámetro en el inspector de ultra-bajo nivel posee un toggle "Exponer al Optimizador"; al activarlo, dicho parámetro se inyecta en el espacio de búsqueda del Generador Genético del Nivel 2.
- [ ] La UI renderiza gráficos de Coordenadas Paralelas para visualizar agrupaciones de parámetros optimizados y dispersión UMAP 3D para análisis de estabilidad.

---

## Restricciones

- **NUNCA** permitir la ejecución de estrategias en vivo con bloques de validación marcados como invalidantes (en rojo).
- **NUNCA** mezclar tipos de datos incompatibles en las conexiones micro sin un nodo convertidor explícito.
- **NUNCA** utilizar intérpretes externos en caliente (Python u otros lenguajes interpretados) para la ejecución matemática; todo cálculo en el lienzo se compila a código nativo de Rust (Rhai scripting seguro o AST optimizado).
- **Límite Técnico:** Invalidación de estados y actualización visual en la UI Flutter debe completarse en menos de 16ms (1 frame a 60fps).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| AUTO_INVALIDATE_CACHE | true | true/false | Controla si la edición micro invalida automáticamente los tests de robustez | [FIJO] |
| MAX_OPT_PARAMETERS | 20 | 5 - 100 | Límite de parámetros expuestos simultáneamente al optimizador | CONFIG |
| UMAP_DIMENSIONS | 3 | 2 - 3 | Dimensiones del gráfico de dispersión de estabilidad | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Gestor de estados de invalidación, validación de tipos del AST global y mapeo de parámetros expuestos.
- **Shell (Infraestructura):** Componentes gráficos de Coordenadas Paralelas, visualizador UMAP WebGL en Flutter, y motor de plantillas JSON AST.
- **Frontera Pública:** Contrato de importación/exportación de `Pipeline_JSON_Manifest` y bus de eventos de invalidación de estado.

---

## Ciclo de Vida de la Feature

### Entrada
- Interacciones del usuario en los tres niveles de la interfaz.
- Manifiestos de plantillas base (Templates JSON AST).
- Resultados y métricas de ejecución de DuckDB/Parquet.

### Proceso
- Mapea elementos gráficos 1:1 con nodos del JSON AST global.
- Evalúa dependencias de parámetros y marca invalidaciones en cascada.
- Renderiza dispersión espacial de parámetros optimizados.

### Salida
- JSON AST sincronizado y validado.
- Viewports gráficos con marcas de estado.

---

## Tareas (TTRs)

### **TTR-001: Motor de Diagrama de Fábrica Visual (Nivel 2 Meso)**
*   **¿Cuál es el problema?** El operador requiere configurar pipelines de validación complejos y ver el embudo de estrategias de forma continua y clara.
*   **¿Qué tiene que pasar?** Implementar el lienzo horizontal en Flutter con bloques móviles que representen Ingesta, NSGA-II, WFA y Montecarlo, mostrando tasas de filtrado dinámicas.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo conectar y reordenar visualmente los 4 bloques de validación principales.

### **TTR-002: Editor de Visual Scripting Micro (Nivel 3 Micro)**
*   **¿Cuál es el problema?** Diseñar reglas de señal específicas requiere granularidad de indicadores, patrones de velas y operadores matemáticos.
*   **¿Qué tiene que pasar?** Desarrollar la paleta de micro-bloques clasificados (Candle Patterns, Time Filters, Position State) y conectores con validación de tipos en lienzo CustomPainter.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo diseñar un cruce de EMA con filtrado por patrón Doji y día de la semana.

### **TTR-003: Validador y Motor de Invalidación Reactiva (Flutter JSON AST)**
*   **¿Cuál es el problema?** Alterar parámetros de bajo nivel sin re-evaluar la robustez conduce a falsos positivos en producción.
*   **¿Qué tiene que pasar?** Implementar un motor de invalidación en Flutter que compare hashes lógicos de la estrategia; cualquier cambio micro marca en rojo y descarta el caché de validación en Nivel 2.
*   **¿Cómo sé que está hecho?**
    - [ ] Cambio una variable de entrada en el Nivel 3 y observo que el bloque WFA del Nivel 2 pasa de verde a rojo parpadeante con advertencia "Re-evaluación Mandatoria".

### **TTR-004: Visualizador de Coordenadas Paralelas y UMAP 3D**
*   **¿Cuál es el problema?** Identificar el agrupamiento y estabilidad de parámetros optimizados entre 10,000 iteraciones es inmanejable en tablas.
*   **¿Qué tiene que pasar?** Integrar gráficos vectoriales de Coordenadas Paralelas en la UI para evaluar regímenes estables, y gráfico WebGL UMAP 3D para dispersión espacial de robustez.
*   **¿Cómo sé que está hecho?**
    - [ ] El Nivel 2 renderiza interactivamente las coordenadas paralelas y dispersión 3D de optimización en menos de 50ms al cargar datos.

### **TTR-005: Escape Hatch (Editor de Código Embebido Nativo Flutter y Rhai Scripting)**
*   **¿Cuál es el problema?** El Grafo visual puede carecer de operadores específicos que el operador desea programar a mano.
*   **¿Qué tiene que pasar?** Crear un nodo inyectable especial con un editor de código embebido nativo Flutter en el frontend, que permita escribir código de cálculo matemático evaluado de forma segura en Rust mediante el motor de scripting Rhai.
*   **¿Cómo sé que está hecho?**
    - [ ] Escribo una función de filtro matemático personalizada en el editor de código embebido nativo Flutter del nodo, expongo dos pines numéricos, y se ejecuta a velocidad nativa sin depender de intérpretes externos.

### **TTR-006: Visor de Latencia de Nodos (Heatmap)**
*   **¿Cuál es el problema?** Identificar cuellos de botella de latencia entre decenas de nodos en el lienzo visual es difícil sin perfiles visuales.
*   **¿Qué tiene que pasar?** Colorear los nodos del lienzo con un mapa de calor (verde a rojo) mostrando microsegundos de ejecución/backtest medidos en el core.
*   **¿Cómo sé que está hecho?**
    - [ ] Al activar el "Modo Rendimiento", los nodos lentos se colorean en rojo indicando visualmente el retardo en microsegundos.

### **TTR-007: Git Visual Gráfico (Diffs de Grafo)**
*   **¿Cuál es el problema?** Comparar cambios entre versiones lógicas del lienzo a nivel de texto JSON es propenso a errores cognitivos.
*   **¿Qué tiene que pasar?** Proveer una vista de diferencias del lienzo que resalte gráficamente los nodos eliminados en rojo y los añadidos en verde.
*   **¿Cómo sé que está hecho?**
    - [ ] Al comparar dos versiones lógicas del DAG, la UI resalta claramente el diff gráfico de nodos.

### **TTR-008: Meta-Estrategias (Lienzo Macro Nodal)**
*   **¿Cuál es el problema?** Orquestar múltiples estrategias con lógicas de control de capital cruzadas requiere un nivel superior de abstracción visual.
*   **¿Qué tiene que pasar?** Implementar un lienzo de alto nivel donde los nodos representen estrategias completas y permitan trazar lógica de rebalanceo y distribución de margen (ej: si Estrategia A Drawdown > 15%, desactiva Estrategia B y transfiere margen a C).
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo diseñar un grafo de portafolio conectando dos bloques de estrategia independientes a una regla global de balance.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil IA / R&D. Registra `logic_hash`, `manifest_id`, `version_node_id`, `node_id`.
- **Rastro de Evidencia:** Emite hash de invalidación y auditoría de parámetros expuestos para el módulo de `feedback`.

---

## Dependencias
**Depende de:**
- [`strategy-versioning`](../features/strategy-versioning.md) — para la gestión de versiones del AST.
- [`robustness-score-aggregator`](../features/robustness-score-aggregator.md) — para actualizar estados del Nivel 2.

**Bloquea:**
- [`generate`](../modules/generate.md) — requiere la UI micro/meso.
