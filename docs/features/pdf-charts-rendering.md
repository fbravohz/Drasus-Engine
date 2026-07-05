# PDF Charts Rendering

**Carpeta:** `./features/pdf-charts-rendering/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión)

---

## ¿Qué es esta feature?

El `PDF Charts Rendering` es el componente de backend (server-side/headless) encargado de generar y renderizar gráficos vectoriales estáticos de alto rendimiento para su inclusión directa en los reportes analíticos inmutables en formato PDF. Permite exportar curvas de equidad, histogramas de distribución y matrices mensuales sin depender de la UI de Flutter o navegadores externos.

---

## Comportamientos Observables

- [ ] El usuario presiona "Exportar Reporte PDF" desde el Strategy Inspector o Fleet Command.
- [ ] El sistema genera un archivo PDF detallado que incluye:
  - Curva de equidad vectorizada limpia.
  - El Monthly Performance Heatmap renderizado como tabla PDF con colores correspondientes.
  - Histogramas de distribución horaria de trades y scatter plot simplificado.
- [ ] Los gráficos en el PDF conservan su nitidez vectorial al aplicar zoom (sin pixelación).

---

## Restricciones

- **NUNCA** utilizar Chromium headless (Puppeteer/Playwright) o dependencias pesadas similares para el renderizado de gráficos en reportes; todo el pintado vectorial se genera mediante código nativo en Rust (ej. usando la librería `plotters` u otras herramientas de dibujo nativas).
- **NUNCA** bloquear el hilo principal de ejecución en vivo al compilar y renderizar reportes PDF; se procesa de forma asíncrona mediante la cola de tareas del orquestador.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PDF_GRAPH_DPI | 300 | 150 - 600 | Resolución de las imágenes rasterizadas incrustadas (si aplica) | CONFIG |
| VECTOR_EXPORT_FORMAT | SVG | SVG / PNG | Formato vectorial intermedio para incrustar en el PDF | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Motor de dibujo y layout de curvas, ejes y rejillas vectoriales a partir de arrays de datos crudos.
- **Shell (Infraestructura):** Generador de archivos binarios PDF y persistencia local en el directorio de reportes del usuario.
- **Frontera Pública:** Puerto de generación de PDF que recibe el catálogo analítico de resultados y escribe el reporte en disco.

---

## Ciclo de Vida de la Feature — PDF Charts Rendering

### Entrada
- Datos de balance históricos y estadísticas de transacciones procesados por Rust.
- Parámetros de estilo (paleta de colores, márgenes).

### Proceso
- Traduce los puntos de la curva de equidad a trazos vectoriales.
- Genera la estructura tabular del heatmap con colores de fondo.
- Ensambla y pagina los elementos gráficos en el formato de destino PDF.

### Salida
- Archivo binario PDF persistido localmente y ruta del archivo devuelta al emisor.

---

## Tareas (TTRs)

### **TTR-001: Renderizador Vectorial Headless (Rust)**
*   **¿Cuál es el problema?** El renderizado de gráficos estáticos para reportes no puede depender de la presencia de una GPU o de la interfaz gráfica Flutter activa.
*   **¿Qué tiene que pasar?** Implementar un motor de dibujo nativo en Rust que pinte curvas de equidad e histogramas en formatos vectoriales SVG/PDF.
*   **¿Cómo sé que está hecho?**
    - [ ] El motor Rust genera archivos de imagen vectorial correctos sin levantar servicios de UI.

### **TTR-002: Compilador de Reportes Analíticos PDF**
*   **¿Cuál es el problema?** El operador requiere reportes consolidados y portátiles para documentar el comportamiento de las estrategias y portafolios.
*   **¿Qué tiene que pasar?** Desarrollar el servicio que concatena texto analítico y gráficos vectoriales en páginas PDF formateadas.
*   **¿Cómo sé que está hecho?**
    - [ ] El sistema exporta un archivo PDF con la curva de equidad, el heatmap mensual y las tablas estadísticas correctas.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil Ops / Auditoría.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía:** `owner_id` (usuario que solicitó el reporte).
    - **IV. Infra & Ops:** `node_id`, `process_id` (worker que generó el PDF).
- **Rastro de Evidencia:** Emite un registro de auditoría de reportes generados para el módulo de `feedback`.

---

## Dependencias
- **Depende de:** `/features/robust-reporting.md`, `/features/institutional-metrics.md`
- **Bloquea:** `/modules/validate.md`
