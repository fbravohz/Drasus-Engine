## 21. Fusión de Datos Fundamentales (Capa de Eventos)

### Introducción

Drasus Engine es un motor cuantitativo basado en indicadores y matemáticas. Esta capa le añade la capacidad de usar **información del mundo** — noticias fundamentales, análisis de economías y empresas, indicadores macro y micro, resultados corporativos — **sin abandonar el determinismo**. El principio rector: un evento del mundo se convierte en un **indicador numérico** que el resto del motor consume exactamente igual que cualquier otro indicador técnico (mismo contrato, sin lógica especial en la ruta caliente).

La capa **no** interpreta texto con opiniones. Convierte hechos medibles en números por fórmula. Combina el poder cuantitativo con el poder fundamental, pero el fundamental entra ya **determinizado**.

### Decisiones Base

| ADR | Qué fija |
|---|---|
| [ADR-0125](../adr/ADR-0125.md) | Frontera determinista: Event Study + Surprise como métodos canónicos; la extracción NLP de texto libre vive en `moonshots`. |
| [ADR-0126](../adr/ADR-0126.md) | El hecho crudo se obtiene de proveedores estructurados externos; el scoring es 100% propio; prohibido consumir scores de terceros. |
| [ADR-0127](../adr/ADR-0127.md) | PIT extendido a eventos: instante de publicación + versionado vintage/as-of (first-print vs revisiones). |
| [ADR-0128](../adr/ADR-0128.md) | Relevancia evento→activo por mapa de exposición; indicador normalizado por instrumento (resuelve el multi-mercado). |

### Flujo de Datos

```
1. Ingesta del evento crudo (módulo ingest)
   ├─► Fuente estructurada externa (calendario macro, resultados, política monetaria)
   ├─► Persistencia local con linaje (proveedor, instante de publicación, licencia, latencia)
   ├─► Versionado vintage/as-of: first-print + revisiones como versiones nuevas (nunca sobrescribe)
   └─► Guardia PIT (pit-data-validator): el evento solo es visible desde su instante de publicación

2. Scoring determinista del impacto (feature event-impact-scorer)
   ├─► Event Study: retorno anormal del activo alrededor del evento ("cuánto impactó")
   ├─► Surprise: distancia estandarizada real vs consenso previo ("cuánto sorprendió")
   └─► Coeficiente de impacto reproducible y auditable (misma entrada → mismo número)

3. Resolución de relevancia por activo (feature asset-exposure-map)
   ├─► Vector de exposición del instrumento (emisor, sector, país, divisa, correlación, cadena de suministro)
   ├─► Etiquetas del evento (entidad, alcance global/país/sector/emisor, región)
   └─► Relevancia = solape determinista; el alcance modula la difusión

4. Proyección a indicador (feature fundamental-indicator-projector)
   ├─► Combina coeficiente de impacto × relevancia → serie numérica acotada y normalizada por activo
   └─► Expuesta en el contrato estándar de indicador del motor

5. Consumo aguas abajo (sin lógica especial)
   ├─► generate: las estrategias pueden referenciar el indicador fundamental
   ├─► validate: backtest PIT-correcto sobre el histórico de eventos
   ├─► execute: lectura en tiempo real del indicador
   └─► manage: ponderación de riesgo/posición según el indicador
```

### Invariantes

* **Determinismo:** el mismo evento produce el mismo indicador cada vez que se recalcula. Razón: el coeficiente de impacto se calcula por fórmula, nunca por opinión de un modelo.

* **PIT innegociable:** ningún consumidor ve un evento antes de su instante de publicación, ni usa cifras revisadas en un backtest. Razón: usar el dato revisado o anticipado es look-ahead silencioso que invalida el backtest.

* **Indicador relativo al activo:** nunca se inyecta un valor fundamental global idéntico a todos los activos. Razón: el mismo evento impacta distinto según la exposición de cada instrumento; la misma estrategia en distintos activos recibe indicadores distintos.

* **Soberanía:** solo entra el hecho crudo medible (cifra real, consenso, fecha/hora, entidad); jamás un score interpretado por un tercero. Razón: una caja negra ajena en el núcleo rompe el determinismo y la auditoría.

* **Contrato estándar de indicador:** la salida respeta el mismo contrato que cualquier indicador técnico; el hot-path no contiene lógica fundamental, solo lee una serie numérica precalculada. Razón: cero coste de latencia y cero acoplamiento especial.

### Propiedades

* **Alcance inicial:** eventos estructurados con medida objetiva (programados con consenso, o con reacción de precio observable). El texto libre / NLP queda en R&D (`moonshots`) hasta validarse.
* **Reproducibilidad:** todo indicador es auditable — fórmula y datos de entrada trazados vía linaje (ADR-0020 V2).
* **Diferenciación:** la fusión fundamental determinista es una capacidad de categoría ausente en las herramientas puramente técnicas.

---
