# Anomaly Detector & Insight Engine

**Carpeta:** `./features/anomaly-detector/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente encargado de detectar comportamientos atípicos y fallos de modelo. Su misión es el **Aprendizaje de Fallas**: transforma anomalías estadísticas en conocimientos accionables (restricciones) para que el motor de generación no cometa los mismos errores dos veces.

---

## Comportamientos Observables

- [ ] Detecta **Rupturas de Correlación** masivas entre estrategias.
- [ ] **PCA Toxicity Clustering:** Identifica zonas frágiles del portafolio agrupando estrategias por perfiles de riesgo tóxicos (§7.2).
- [ ] **Adversarial Testing:** Genera escenarios patológicos automáticos (ej: "¿Qué pasa si la volatilidad duplica?") para estresar el modelo.
- [ ] Genera **Sugerencias Estructuradas** para el re-diseño genético de estrategias.

---

## Ciclo de Vida de la Feature — Anomaly Detector

### Entrada
- Logs de ejecución y de auditoría de todos los módulos.
- Serie de retornos de las estrategias activas.
- Historial de veredictos del watchdog.

### Proceso
- Aplica algoritmos de detección de outliers sobre el flujo de fills.
- Analiza la matriz de correlación dinámica buscando concentraciones de riesgo.
- Clasifica la anomalía (Ej: Operativa, Estadística, de Datos).

### Salida
- **Anomaly Log:** Registro detallado del evento.
- **Genetic Constraints:** Parámetros de veto para la próxima evolución genómica.

### Contextos de Uso
**Contexto Único: Mejora Continua (Módulo Feedback)**
- Actúa como el puente de inteligencia entre la ejecución real y la creación de nuevos candidatos.

---

## Tareas (TTRs) — Herencia de Módulo Retroalimentar

### TTR-001: Monitor de Correlaciones Rotas
*   **Descripción:** Análisis de dependencia entre estrategias del portafolio en tiempo real.

### TTR-002: PCA Toxicity Clustering (SQX Mod 3.3.3)
*   **Descripción:** Proyecta la población de estrategias en un espacio PCA para identificar clústeres con "Toxicity High" (redundancia peligrosa).
*   **Veredicto:** Sugiere el retiro de estrategias que caen en zonas de fragilidad histórica.

### TTR-003: Generador de Pruebas Adversarias
*   **Descripción:** Crea variaciones sintéticas de los datos de mercado para probar la resistencia ante eventos cisne negro.

### TTR-004: Generador de Restricciones Genéticas
*   **Descripción:** Traduce los fallos detectados en parámetros de veto para el `nsga2-optimizer`.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. El aprendizaje de anomalías es propiedad intelectual crítica.
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda detección de anomalía registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la anomalía |
| | `created_at` | Timestamp de detección |
| | `audit_hash` | Hash del rastro de evidencia causal |
| | `audit_chain_hash` | Hash del timeline de incidentes |
| **II. Soberanía** | `owner_id` | Autor de la IP evaluada |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del modelo de detección (PCA/OLS) |
| | `data_snapshot_id` | Contexto de mercado del evento |
| | `indicator_state_hash` | Score de toxicidad/anomalía |
| | `version_node_id` | Versión de la restricción generada |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del insight engine |


---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Algoritmos de clustering y detección de outliers en `anomaly_logic.rs`.
- **Shell (Infraestructura):** Integración con la base de datos de auditoría histórica.
- **Frontera Pública:** Contrato `get_insights_from_failures(session_data)`.

---

## Dependencias
**Consumido por:** `feedback`.
**Depende de:** `factor-decomposition`, `nsga2-optimizer`.
