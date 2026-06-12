# Interactive Chat Loop

**Carpeta:** `./moonshots/interactive-chat-loop/`
**Estado:** Incubación
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión)

---

## ¿Qué es esta feature?

El `Interactive Chat Loop` es una interfaz conversacional integrada de estilo ChatGPT que permite al operador refinar, modificar y auditar sus estrategias cuantitativas utilizando lenguaje natural. Traduce las intenciones del usuario en modificaciones directas y seguras sobre el Árbol de Sintaxis Abstracta (JSON AST) de la estrategia, cerrando el ciclo de retroalimentación de forma conversacional.

---

## Comportamientos Observables

- [ ] El usuario abre la consola de chat e introduce una orden (ej: "Agrega un filtro de spread máximo de 2 pips a las señales de compra").
- [ ] La interfaz procesa la intención, muestra una confirmación estructurada del cambio propuesto y resalta visualmente el nodo modificado en el lienzo del DAG (Nivel 2 o 3).
- [ ] El chat permite auditar y pedir aclaraciones sobre por qué una estrategia tomó una decisión específica o por qué falló un test.

---

## Restricciones

- **NUNCA** permitir que el LLM modifique la lógica del AST de forma directa o autónoma sin la confirmación y veto final explícito del usuario.
- **NUNCA** utilizar modelos remotos (nube) ni runtimes externos (Ollama) para procesar estrategias propietarias del usuario, preservando la confidencialidad de la propiedad intelectual mediante LLM local soberano embebido (`candle` con modelos cuantizados — ADR-0115).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| LOCAL_LLM_MODEL | (modelo cuantizado local) | - | Identificador del modelo local soberano a ejecutar vía `candle` embebido (sin Ollama) | CONFIG |
| TEMPERATURE | 0.0 | 0.0 - 0.7 | Grado de creatividad del modelo (fijado en 0.0 para determinismo de código) | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Parseadores de lenguaje natural e intenciones y generadores de parches de AST.
- **Shell (Infraestructura):** Inferencia in-process vía LLM local soberano embebido (`candle`, sin servidor externo) y persistencia de logs de conversación en SQLite.
- **Frontera Pública:** Interfaz de procesamiento que recibe texto de usuario e interactúa con el orquestador de la estrategia.

---

## Tareas (TTRs)

### **TTR-001: Inferencia Local Soberana (`candle` embebido)**
*   **¿Cuál es el problema?** Enviar datos de estrategias a APIs externas o depender de runtimes externos compromete la soberanía intelectual del operador.
*   **¿Qué tiene que pasar?** Integrar inferencia in-process de bajo retardo vía `candle` embebido (sin Ollama ni servidor) para procesar prompts estructurados.
*   **¿Cómo sé que está hecho?**
    - [ ] El chat responde a prompts de prueba localmente sin requerir conectividad a Internet.

### **TTR-002: Traductor Semántico a AST (JSON)**
*   **¿Cuál es el problema?** Traducir lenguaje natural difuso a código operativo o AST puede introducir errores de sintaxis catastróficos.
*   **¿Qué tiene que pasar?** Implementar un compilador intermedio que valide gramaticalmente las modificaciones sugeridas por la IA antes de aplicarlas al AST.
*   **¿Cómo sé que está hecho?**
    - [ ] Una solicitud de texto válida genera exactamente el parche de nodos esperado en el JSON AST.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local (Sovereign AI).
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil IA / R&D. Registra `node_id`, `logic_hash`, `access_token_id`.
- **Rastro de Evidencia:** Emite logs de comandos procesados al módulo de `feedback`.
