# Strategy AST Copilot

**Carpeta:** `./features/strategy-ast-copilot/`
**Estado:** En Diseño
**Última actualización:** 2026-04-28
**Decisión Arquitectónica Asociada:** ADR-0051

## 1. ¿Qué es esta feature?
Asistente manejado por LLM que traduce lenguaje natural en la estructura de árbol determinista (AST) que gobierna las reglas de una estrategia. 
Resuelve la frustrante curva de aprendizaje de ensamblar bloques visuales manualmente, actuando como el "fontanero" que conecta la lógica, pero sin el peligro de que el LLM alucine código ejecutable directo.

## Narrativa de Producto

**Capacidad:** `CAP-09` — Una interfaz donde se ve el razonamiento

**Historia de usuario:** Como usuario que no programa, quiero describir en lenguaje natural la regla que quiero agregar a mi estrategia y ver aparecer el bloque correspondiente ya conectado en el lienzo, para no tener que ensamblarlo a mano clic por clic.

**El problema (técnico):** ensamblar manualmente cada bloque de una regla nueva —arrastrar, conectar, definir parámetros— es lento, y la alternativa de dejar que un modelo de lenguaje genere directamente la lógica ejecutable es peligrosa: un modelo de lenguaje puede inventar algo que no existe en el sistema y comprometer la lógica de la estrategia si nadie lo filtra antes.

**La solución (técnica):** el usuario describe la regla en lenguaje natural ("crea un filtro que rechace entradas los viernes a partir de las 14:00"), un asistente propone una estructura de reglas expresada en un formato validado que solo puede mapear a piezas que ya existen en el catálogo del sistema, y esa propuesta se valida antes de instanciarse en el lienzo. El asistente propone; el motor determinista dispone — si la propuesta rompe una regla de encadenamiento válida, se rechaza y el motivo se le devuelve al asistente para que corrija.

**Explicado sin tecnicismos:** es como dictarle una instrucción a un ayudante que solo tiene permiso de usar piezas de un catálogo aprobado, nunca de inventar materiales nuevos sobre la marcha. Le dices "pon un filtro que bloquee esto los viernes por la tarde" y él arma la pieza correcta con el catálogo que ya existe — pero si se equivoca y propone algo que no está en el catálogo, un inspector se lo rechaza antes de que se instale, y el ayudante tiene que intentarlo de nuevo.

Nunca es al revés: el ayudante sugiere, pero quien decide si la pieza entra es el sistema de reglas fijas, no el ayudante.

## 2. Comportamientos Observables
- [ ] Usuario dicta: "Crea un filtro que rechace entradas los viernes a partir de las 14:00".
- [ ] El sistema genera un sub-árbol en el lienzo visual validado e instanciado.

## 3. Restricciones
- PROHIBIDA la inyección de código Rust crudo; el sistema exige una respuesta en JSON puro validado vía esquemas Serde que mapea exclusivamente a nodos existentes en el core de Drasus Engine.

## 4. Parámetros Configurables
| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| LLM_PROVIDER | local | local / cloud | Motor de inferencia local soberano embebido (`candle`); cloud solo para asistencia de diseño offline, nunca veredictos. Sin runtimes externos (ADR-0115). | CONFIG |
| AST_SCHEMA_VERSION | v1 | v1 | Versión de mapeo JSON -> Serde | FIJO |

## 5. Estructura Interna (FCIS)
- **Core (Lógica Pura):** Validador Serde del JSON generado; compresión de esquemas de nodos para el Prompt.
- **Shell (Infraestructura):** Conectores API gRPC/REST hacia los LLMs.
- **Frontera Pública:** Endpoint de traducción semántica.

## 6. Ciclo de Vida de la Feature
### Entrada
- Instrucción en texto natural del usuario.
- Catálogo de Nodos AST soportados y reglas de encadenamiento.
### Proceso
- El LLM genera una topología en formato de Grafo.
- El validador rechaza y reintenta si el JSON rompe las invariantes lógicas (ej. un nodo booleano conectado a un input numérico).
### Salida
- Modelo en memoria Serde validado.
### Contextos de Uso
- **Contexto 1: Orchestrator Visual (generate / edit)**
  - Creación acelerada de andamiajes de estrategias.

## 7. Tareas (TTRs)
### **TTR-001: Fontanería LLM -> AST Serde**
* **¿Cuál es el problema?** Ensamblar ASTs complejos a mano requiere demasiados clics; programarlos requiere Rust.
* **¿Qué tiene que pasar?** Un LLM hace de puente y mapea texto a JSON compatible con el schema de Drasus Engine.
* **¿Cómo sé que está hecho?**
  - [ ] Escribo "Si RSI > 70 y es martes, vende" y aparece el árbol de 4 nodos listo para modificar en la UI.
  - [ ] Si el LLM inventa un nodo `Magia()`, el validador lo rechaza e inyecta el error de vuelta al modelo.

## 8. Gobernanza y Estándares (Fijos)
- **Soberanía Asistida:** Permite LLM en Cloud pero el backend de validación y la lógica final operan localmente.
- **Inundación de Fundaciones (ADR-0020): Perfil B (IA / R&D)** — linaje prompt→AST (II + III subset + IV).

  | Categoría | Campo | Descripción |
  | :--- | :--- | :--- |
  | **I. Identidad** | `id` | Identificador único de la generación AST |
  | | `created_at` | Timestamp de la generación |
  | | `updated_at` | Timestamp de última modificación del registro |
  | | `audit_hash` | Hash de auditoría del prompt + AST resultante |
  | | `audit_chain_hash` | Hash encadenado del historial de iteraciones |
  | | `event_sequence_id` | Secuencia de recuperación |
  | **II. Soberanía** | `owner_id` | Autor del prompt |
  | | `manifest_id` | Estrategia generada |
  | **III. Pesos/Arquitectura** | `logic_hash` | Hash del AST producido |
  | | `parent_id` | Semilla (prompt) original de la que deriva el AST (linaje) |
  | | `version_node_id` | Versión del AST en el DAG |
  | **IV. Hardware** | `node_id` | ID del hardware/host de generación |
  | | `process_id` | PID del proceso del copiloto |
