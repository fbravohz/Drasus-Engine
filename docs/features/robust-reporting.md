# Robust Reporting

**Carpeta:** `./features/robust-reporting/`
**Estado:** En Diseño
**Última actualización:** 2026-04-29
**Decisión Arquitectónica Asociada:** ADR-0020 (Inundación de Fundaciones)

## 1. ¿Qué es esta feature?

Genera reportes estáticos (JSON/HTML) hiper-detallados de una estrategia o portafolio, incluyendo curvas de equity hiper-resolución, distribuciones de Montecarlo completas y un manifiesto de parámetros del backtest.
Es una exportación offline y soberana que no depende de interfaces web pesadas.

## 2. Comportamientos Observables

- Cuando se solicita un reporte, el sistema exporta un archivo HTML encapsulado (sin dependencias externas) y un JSON puro de métricas.
- El HTML contiene gráficos interactivos pre-renderizados (Equity, Drawdown, Montecarlo).
- **Generador de Tearsheet / Pitchbook Institucional:** El usuario selecciona un portafolio y pulsa "Generar Prospecto para Inversores". Además de los gráficos, el sistema redacta (vía LLM local) una narrativa financiera profesional de nivel institucional (tesis del fondo, comportamiento en regímenes de estrés, descorrelación vs benchmark) y entrega un PDF presentable a inversores.

## 3. Restricciones

- NUNCA incluye llamadas a APIs externas en el HTML (Zero-Docker / Soberanía).
- NUNCA expone el genoma/código fuente si la estrategia está marcada como privada.

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| EXPORT_FORMAT | html_json | html / json | Formato de salida del reporte | CONFIG |
| EMBED_DATA | true | boolean | Si incrusta todos los data points en el HTML (puede pesar varios MB) | CONFIG |

## 5. Estructura Interna (FCIS — ADR-0002)

- **Core:** Plantillas Tera y serializador Serde.
- **Shell:** Orquestador de escritura a disco.
- **Frontera Pública:** API `generate_report(strategy_id)`.

## 6. Ciclo de Vida de la Feature

### Entrada
- IDs de estrategias, resultados de simulaciones Montecarlo, métricas institucionales.

### Proceso
- Agrega datos de series temporales.
- Renderiza componentes estáticos HTML.
- Serializa metadatos técnicos en JSON.

### Salida
- Paquete ZIP con Reporte HTML y `metadata.json`.

### Contextos de Uso
**Contexto 1: Veredicto de Validación**
- Tras certificar en `validate`, se emite su "Certificado de Nacimiento" en HTML.

**Contexto 2: Autopsia de Feedback**
- Tras detectar anomalía, se emite su "Reporte de Defunción".

## 7. Tareas (TTRs)

### **TTR-001: Motor de Renderizado Estático HTML**
* **¿Cuál es el problema?** Requerimos compartir reportes o analizarlos offline sin levantar el backend completo.
* **¿Qué tiene que pasar?** El sistema toma métricas complejas (Montecarlo, Equity) y las consolida en un único archivo HTML estático e independiente.
* **¿Cómo sé que está hecho?**
  - [ ] Genero reporte y abro el `.html` sin conexión a internet y veo los gráficos renderizados.
* **¿Qué no puede pasar?**
  - No puede descargar librerías JS desde CDNs externos.

### **TTR-002: Exportador JSON Institucional**
* **¿Cuál es el problema?** Otros sistemas o scripts necesitan parsear los resultados exactos del backtest.
* **¿Qué tiene que pasar?** Volcado determinista de todas las métricas en un JSON indexable.
* **¿Cómo sé que está hecho?**
  - [ ] El JSON tiene llaves predecibles y coincide hash con la base de datos.
* **¿Qué no puede pasar?**
  - No puede faltar el `audit_hash` del reporte.

### **TTR-003: Generador de Pitchbook / Tearsheet con Narrativa IA**
* **¿Cuál es el problema?** Un reporte crudo de métricas (Profit Factor, Sharpe, SQN) sirve a un ingeniero, pero no convence a un inversor de aportar capital. El creador necesita un folleto de inversión profesional para monetizar su portafolio.
* **¿Qué tiene que pasar?** A partir del portafolio y sus métricas certificadas, el sistema genera un PDF con gráficos institucionales y una narrativa financiera redactada por el LLM local: tesis de la estrategia, resiliencia en regímenes de estrés (incl. resultados de gemelos digitales si existen) y nivel de descorrelación frente al benchmark.
* **¿Cómo sé que está hecho?**
  - [ ] Selecciono un portafolio, pulso "Generar Prospecto" y obtengo un PDF con narrativa coherente y gráficos.
  - [ ] La narrativa cita exclusivamente métricas reales del portafolio (sin inventar cifras).
* **¿Qué no puede pasar?**
  - El texto generado NUNCA puede afirmar métricas que no existan en los datos certificados (cero alucinación numérica).
  - Si el LLM local no está disponible, se genera el PDF con métricas y gráficos sin la capa narrativa (fallback).
* **Dependencia:** reutiliza el LLM local de [`robustness-verdict-engine`](./robustness-verdict-engine.md) para la generación de lenguaje natural.

## 8. Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Los recursos web (JS/CSS) están incrustados inline.
- **Inundación de Fundaciones (ADR-0020):**
  - **Perfil D (Ops / Auditoría):** reportes/exportación forense (la etiqueta "C. Auditoría" era mixta e inválida).
  - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash` (del contenido estático), `audit_chain_hash`, `event_sequence_id`.
  - **II. Soberanía & Propiedad:** `owner_id` (visible en metadata), `institutional_tag`.
  - **IV. Infraestructura & Ops:** `node_id` (nodo que renderizó el archivo), `process_id`.
  - **V. Forense (Gobernanza, cuando aplica):** `risk_audit_id`, `signature_hash` del reporte sellado.

## 9. Decisión Arquitectónica Asociada
- ADR-0020.
