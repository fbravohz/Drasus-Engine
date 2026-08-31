# Glass-Box AI Translator

**Carpeta:** `./features/glass-box-ai-translator/`
**Estado:** En Diseño
**Última actualización:** 2026-04-29
**Decisión Arquitectónica Asociada:** ADR-0057 (Glass-Box AI Translator)

## 1. ¿Qué es esta feature?

Es el sistema puente que elimina el "código espagueti" y las "cajas negras" propias de las redes neuronales usadas en trading. Transforma la inteligencia oculta de agentes Deep Reinforcement Learning (DRL) y ecuaciones complejas en flujos visuales que un humano puede leer, auditar y modificar.

**Problema:** Una IA puede encontrar un Alpha espectacular, pero como su salida es un set de miles de pesos neuronales, un banco institucional o un trader prudente nunca le daría dinero real ("Falta de Lógica").
**Solución:** Sistema de traducción tri-etapa: Extracción matemática (regresión simbólica nativa sobre el AST, par del Puerto de Búsqueda — ADR-0113, no PySR) -> Traducción de nodos (AST Visual) -> Explicación semántica por **reporte estructurado determinista** (ADR-0115; LLM local soberano opcional, nunca Ollama requerido).

## Narrativa de Producto

**Capacidad:** `CAP-09` — Una interfaz donde se ve el razonamiento

**Historia de usuario:** Como investigador de estrategias, quiero que el sistema traduzca la lógica interna de un modelo generado automáticamente a una explicación matemática y visual que pueda auditar, para no tener que confiar a ciegas en algo que ni el propio sistema puede explicar.

**El problema (técnico):** cuando un modelo complejo de aprendizaje encuentra una combinación rentable, su salida es un conjunto de números sin forma legible — no hay manera de saber si encontró una relación real del mercado o memorizó ruido. Ninguna persona, institucional o retail, debería arriesgar capital en algo que no puede explicarse a sí mismo.

**La solución (técnica):** un traductor de tres etapas — extrae la fórmula matemática subyacente del modelo, la convierte en un diagrama de bloques donde cada pieza es un operador o indicador reconocible, y genera un reporte en español a partir de esa estructura, determinista y por plantilla en su forma base. El usuario puede además arrastrar un bloque del diagrama, ajustar un umbral, y devolver la estrategia modificada al sistema sin romper su lógica.

**Explicado sin tecnicismos:** imagina que le pides a alguien que te explique por qué ganó una partida, y en vez de contarte su plan, te entrega un montón de números sin sentido. Eso es lo que hace un modelo de estos por defecto.

Esta pieza es el traductor entre ese montón de números y una explicación real: convierte la maraña en un diagrama de piezas conectadas que puedes seguir con la vista, y le pone palabras en español — algo como "compra cuando el pánico del mercado supera tres veces lo normal". Y si algo del diagrama no te convence, lo puedes tocar tú mismo, ajustar un número, y regresarlo al sistema ya corregido.

## 2. Comportamientos Observables

- [ ] Tras el entrenamiento de un modelo de Deep Learning, el sistema procesa el modelo y muestra una representación en diagrama de bloques (DAG) en la pantalla.
- [ ] En el "Inspector de Estrategia", el usuario ve un texto claro redactado en español neutro (ej. *"Compro cuando el pánico supera 3 desviaciones estándar"*), generado de forma autónoma.
- [ ] El humano puede arrastrar un nodo de este diagrama, modificar un umbral (ej. subir "3 desviaciones" a "4"), anclarlo y devolver la estrategia al orquestador sin romper la lógica.

## 3. Restricciones

- La explicación NUNCA debe contradecir la matemática real; solo traduce la topología del AST. El reporte base es determinista por plantilla.
- La ejecución en producción NUNCA utiliza un LLM; la ejecución opera al nivel de código máquina nativo o Rust SIMD/Rayon.
- NUNCA se admiten dependencias de APIs en la nube ni runtimes externos (Ollama). Si se habilita el realce de lenguaje natural, es vía LLM local soberano embebido (`candle`), siempre opcional (ADR-0115).

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| SYMBOLIC_COMPLEXITY_LIMIT | 15 | 5-30 | Máxima complejidad de la ecuación devuelta por la regresión simbólica nativa | CONFIG |
| LLM_ENHANCEMENT_ENABLED | false | true/false | Activa el realce opcional de lenguaje natural vía LLM local soberano (`candle`); por defecto, reporte determinista por plantilla | CONFIG |

## 5. Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de Regresión Simbólica nativa (implementación par del Puerto de Estrategia de Búsqueda sobre el AST, ADR-0113) acoplados al generador de Árbol de Sintaxis Abstracta (AST Translator), y generador de reporte determinista por plantilla.
- **Shell (Infraestructura):** Orquestación de colas; inferencia de LLM local soberano (`candle`) solo si el realce opcional está habilitado, sin runtimes externos.
- **Frontera Pública:** Generador de JSON AST y Prompts estructurados pre-cargados.

## 6. Ciclo de Vida de la Feature

### Entrada
- Matriz de recompensas o modelo DRL en bruto (entrenado vía `candle`/`burn` en fase moonshot, ADR-0112).
- Historial de acciones exitosas del agente en la simulación temporal.

### Proceso
- La regresión simbólica nativa (par del Puerto de Búsqueda sobre el AST) destila la complejidad del DRL en una ecuación fundamental que imita el comportamiento de compra/venta.
- El traductor transforma esta ecuación en componentes atómicos de un DAG (operadores, indicadores).
- El generador determinista por plantilla recorre el diagrama y produce una narrativa en lenguaje natural explicando el razonamiento de la entrada (LLM local opcional para prosa más rica).

### Salida
- Grafo visual interactivo listo para Flutter CustomPainter.
- Texto narrativo (Semantic Explainer).

### Contextos de Uso
**Contexto 1: Inspección de Estrategia Generada (Generate / Validate)**
- Entrada: Modelo DRL candidato.
- Impacto: Permite revisión humana o veto manual por "lógica sin sentido financiero" a un nivel gráfico y textual.

**Contexto 2: Hibridación Humano-IA (Generate)**
- Entrada: Grafo visual parcialmente modificado por el humano.
- Impacto: El sistema fusiona IA + intuición, anclando nodos fijos para futuras iteraciones del algoritmo evolutivo.

## 7. Tareas (TTRs)

### **TTR-001: Extractor de Tesis (DRL a Regresión Simbólica Nativa)**
* **Problema:** Un modelo de Deep Learning no es auditable.
* **Comportamiento:** Se extraen las reglas de acción imitando al agente en un espacio de ecuaciones controladas mediante el motor de regresión simbólica nativa (par del Puerto de Búsqueda sobre el AST, ADR-0113).
* **Criterio de Éxito:** La ecuación extraída alcanza al menos 90% de paridad operativa con el DRL opaco.

### **TTR-002: AST Translator (Ecuación a DAG Visual)**
* **Problema:** Una ecuación gigante `sin(x) / exp(y-1)` sigue siendo ilegible.
* **Comportamiento:** Transforma símbolos matemáticos en bloques gráficos (`Indicador X` conectado a `Operador Matemático`).
* **Criterio de Éxito:** Un JSON deserializable directamente en los nodos de Flutter CustomPainter.

### **TTR-003: Semantic Explainer (DAG a Lenguaje Natural Determinista)**
* **Problema:** El humano requiere entender el "Por Qué" de la IA en tiempo rápido.
* **Comportamiento:** El generador determinista por plantilla lee el JSON DAG y produce la premisa narrativa (ADR-0115). LLM local soberano opcional para prosa más rica.
* **Criterio de Éxito:** Resumen narrativo estricto, determinista y sin alucinaciones.

## 8. Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Reporte determinista por plantilla sin runtimes externos; LLM local soberano (`candle`) opcional y embebido (ADR-0115).
- **Fidelidad (ADR-0017):** Baja (Ruta de análisis estático, no corre en hot-path).
- **Inundación de Fundaciones (ADR-0020):**
  - **Perfil IA / R&D:** Foco en Identidad + Soberanía IP.
  - Campos a inyectar: Grupo I completo (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) + `version_node_id`, `logic_hash` (para anclar la ecuación simbólica nativa).
  - **Hooks de Auditoría:** Registro de varianza entre modelo opaco y ecuación final extraída.
- **Rastro de Evidencia:** El modelo de explicación textual se almacena en el `test_analysis` persistente para su trazabilidad en el tiempo.

## 9. Decisión Arquitectónica Asociada
ADR-0057: Glass-Box AI Translator (Semantic Explainer y AST)

## 10. Dependencias y Bloqueantes
**Depende de:** `generate` (Pipeline de minería genérica de Alpha).
**Bloquea:** Nada directamente (opera como overlay analítico).
