# Anomaly Detector & Insight Engine

**Carpeta:** `./features/anomaly-detector/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente encargado de detectar comportamientos atípicos y fallos de modelo. Su misión es el **Aprendizaje de Fallas**: transforma anomalías estadísticas en conocimientos accionables (restricciones) para que el motor de generación no cometa los mismos errores dos veces.

---

## Narrativa de Producto

**Capacidad:** `CAP-08` — El ciclo que se cierra solo

**Historia de usuario:** Como operador de una flota de estrategias, quiero que el sistema detecte por sí mismo comportamientos atípicos —correlaciones que se rompen, familias enteras de estrategias que fallan igual— y traduzca ese fallo en una restricción concreta para el motor de búsqueda, para que la siguiente generación no repita el mismo error.

**El problema (técnico):** un fallo aislado es una anécdota; un patrón de fallos es información. Pero casi ninguna herramienta conecta lo que pasó en producción de vuelta con lo que el motor de búsqueda genera después: el conocimiento se queda atrapado en un reporte que alguien tendría que leer, interpretar y traducir a mano en una regla nueva. Mientras tanto, el generador sigue produciendo candidatas con el mismo defecto estructural, generación tras generación.

**La solución (técnica):** un detector que vigila rupturas de correlación entre estrategias activas, agrupa por similitud de comportamiento para encontrar familias enteras con el mismo perfil de riesgo frágil (agrupamiento por componentes principales), y somete al sistema a escenarios patológicos generados a propósito para encontrar dónde se rompe. Cada hallazgo se traduce, sin intervención manual, en una restricción genética concreta —un veto— que el motor evolutivo respeta desde la siguiente corrida.

**Explicado sin tecnicismos:** imagina un maestro que corrige exámenes y encuentra que diez alumnos distintos cometieron exactamente el mismo error de razonamiento. Un maestro mediocre pone una tacha a cada examen y sigue. Un maestro que de verdad enseña se pregunta *por qué* se repite el error, y cambia la próxima clase para que no vuelva a pasar.

Esta pieza es ese segundo maestro, pero automático. No solo detecta que una estrategia falló: encuentra el patrón detrás de varios fallos parecidos y le dice al criadero de `CAP-03`, en el lenguaje que entiende, "deja de producir esto". Es la diferencia entre una fábrica que tropieza con la misma piedra mil veces y una que aprende a esquivarla.

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
## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda detección de anomalía registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la anomalía |
| | `created_at` | Timestamp de detección |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del rastro de evidencia causal |
| | `audit_chain_hash` | Hash del timeline de incidentes |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
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
