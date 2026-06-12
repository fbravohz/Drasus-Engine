# AST Compiler (Serde Zero-Trust)

**Carpeta:** `./features/ast-compiler/`
**Estado:** En Diseño
**Última actualización:** 2026-06-11
**Decisión Arquitectónica Asociada:** ADR-0002 (Patrón FCIS), ADR-0043, ADR-0108 (Genomas Modulares por Dominio)

## ¿Qué es esta feature?

El **Compilador de Árbol de Sintaxis Abstracta (AST)** es el primer filtro del protocolo **Zero-Trust Validation**. Traduce el diseño visual de la estrategia (grafo de nodos) en una estructura de datos JSON estricta. Utiliza **esquemas Serde tipados nativos en Rust** para garantizar que las conexiones sean coherentes. Soporta **Programación Evolutiva Parcial (ADR-0043)**, permitiendo nodos `wildcard_group` que el motor genético resuelve autónomamente.

A partir de ADR-0108, cada nodo `wildcard_group` queda etiquetado con su **dominio genómico de origen** (Señal, Riesgo y Gestión de Posición, Régimen y Filtro de Entorno, o Portafolio y Correlación). Esto permite que el motor genético resuelva el genoma de UN dominio activo mientras los nodos `wildcard_group` de los demás dominios permanecen congelados con su última resolución dentro del mismo Strategy Manifest — el patrón **"Wildcard Invertido"**.

## Comportamientos Observables

- [ ] Generación de un `run_id` tras la validación exitosa del esquema Serde.
- [ ] Validación de tipos de datos entre pines en tiempo real (Zero-Trust).
- [ ] Generación de un JSON unificado (Strategy Manifest) que describe todo el genoma de la estrategia.
- [ ] Detección de errores de diseño antes de la ejecución: "Ciclo infinito detectado" o "Falta conexión obligatoria".
- [ ] Etiquetado de cada nodo `wildcard_group` con su dominio genómico de origen (Señal, Riesgo y Gestión de Posición, Régimen y Filtro de Entorno, Portafolio y Correlación) dentro del Strategy Manifest.
- [ ] Congelamiento de los nodos `wildcard_group` pertenecientes a dominios genómicos no activos en la corrida de evolución actual, compilándolos como nodos fijos con su última resolución conocida.

## Restricciones

- **NUNCA** permitir la generación de un `run_id` para un AST que no pase la validación de esquema Serde.
- **OBLIGATORIO:** El AST debe incluir versionamiento (major.minor.patch) para garantizar la compatibilidad hacia atrás.
- El proceso de compilación debe ser puramente determinista (mismo grafo → mismo JSON → mismo `run_id` si los parámetros son idénticos).
- **Aislamiento de Dominio (FIJO, ADR-0108):** un nodo `wildcard_group` etiquetado para un dominio genómico solo puede ser enviado al motor genético cuando ese dominio figura en `ACTIVE_GENOME_DOMAINS`. Los nodos de dominios no activos se serializan como nodos fijos.
- **Multi-Dominio Simultáneo (FIJO, ADR-0108):** `ACTIVE_GENOME_DOMAINS` puede contener cualquier subconjunto no vacío de los 4 dominios del Registro (incluyendo los 4 a la vez). Cuando contiene más de uno, el motor genético resuelve un único genoma compuesto cuyas Reglas Genómicas combinan Genes de Condición y de Acción de cualquiera de los dominios listados. Los dominios fuera de `ACTIVE_GENOME_DOMAINS` se serializan congelados (Wildcard Invertido).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| MAX_NODES_PER_STRATEGY | 50 | 5 - 500 | Límite de complejidad del grafo | CONFIG |
| STRICT_TYPE_CHECKING | True | True/False | Si valida tipos de datos entre pines | [FIJO] |
| AST_VERSION | 4.0 | - | Versión del formato de contrato (soporta WildCards multi-dominio, ADR-0108) | [FIJO] |
| ENABLE_WILDCARDS | True | True/False | Permite la presencia de nodos de grupos aleatorios | CONFIG |
| ACTIVE_GENOME_DOMAINS | [Señal] | Señal / Riesgo y Gestión / Régimen y Filtro / Portafolio y Correlación | Dominio(s) genómico(s) cuyos nodos `wildcard_group` el motor genético puede resolver en esta corrida (cualquier subconjunto no vacío, incluidos los 4 simultáneos); los demás quedan congelados | CONFIG |
| MAX_CONDITIONS_PER_RULE | 3 | 1 - 8 | Número máximo de Genes de Condición combinables (AND/OR) en una Regla Genómica | CONFIG |
| MAX_ACTIONS_PER_RULE | 2 | 1 - 5 | Número máximo de Genes de Acción simultáneos disparados por una Regla Genómica | CONFIG |
| RULE_LOGICAL_OPERATORS | [AND] | AND / OR | Operadores lógicos disponibles para combinar Genes de Condición dentro de una Regla Genómica | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Tipos Serde (`Node`, `Edge`, `StrategySchema`) y algoritmos de ordenamiento topológico (grafo dirigido nativo en Rust). Incluye la resolución de `wildcard_group` segmentada por dominio genómico y la aplicación del congelamiento de dominios inactivos (ADR-0108).
- **Shell (Infraestructura):** API endpoint recibiendo el JSON de Flutter CustomPainter y generador de clases dinámicas para Nautilus.

## Ciclo de Vida de la Feature — AST Compiler

### Entrada
- JSON de Flutter CustomPainter (Nodos + Posiciones + Pesos).

### Proceso
1. **Sanitization:** Limpieza inicial del JSON (escape de caracteres, prevención de inyección).
2. **Structural Validation:** Serde verifica que la topología del Grafo (DAG) sea válida y los tipos coincidan bajo el protocolo Zero-Trust.
3. **Compilador AST de Lógica Procedural:** El sistema traduce la lógica en árboles sintácticos optimizados para hardware acelerado.
4. **Bloque Factory (AOT):** El compilador identifica nodos estándar y los vincula a "Bloques Pre-Compilados" de NautilusTrader (AOT - Ahead-Of-Time).
5. **Escape-Hatch (JIT):** Para lógica personalizada que no existe en los bloques base, el sistema genera un snippet Rust SIMD/Rayon que se inyecta dinámicamente en el flujo de ejecución procesando matrices de hardware.
6. **Manifest Generation:** Se emite el contrato de ejecución final como una estructura Serde inmutable con su correspondiente `run_id`, segmentando los genomas resueltos por dominio genómico (ADR-0108) cuando el Manifest contiene nodos `wildcard_group` de más de un dominio.
7. **Integrity Signature:** Registro del `manifest_id` coordinado con el `logic_hash` para auditoría forense.

### Salida
- `StrategyExecutableObject` listo para backtest o live trading.

## Tareas (TTRs)

### **TTR-001: Modelado de Esquema Serde Recursivo**
* **¿Cuál es el problema?** Representar una estrategia fractal en JSON requiere esquemas que puedan anidarse y validarse de forma recursiva sin perder el orden topológico.
* **¿Qué tiene que pasar?** El sistema debe generar tipos Serde `StrategyNode` que soporten sub-grafos (estrategias dentro de estrategias) manteniendo la integridad de tipos.
* **¿Cómo sé que está hecho?**
    - [ ] Validación exitosa de un JSON de estrategia de 3 niveles de profundidad.
    - [ ] Detección automática de tipos incompatibles en pines anidados.
* **¿Qué no puede pasar?** NUNCA permitir un esquema que no sea determinista en su serialización.

### **TTR-002: Inyector Dinámico Nautilus-Ready**
* **¿Cuál es el problema?** El motor de NautilusTrader requiere clases de Rust estáticas al arranque, mientras que el diseño visual es dinámico y fluido.
* **¿Qué tiene que pasar?** El compilador debe utilizar el patrón Factory para inyectar la lógica del AST validado en una clase `TemplateStrategy` en tiempo de ejecución.
* **¿Cómo sé que está hecho?**
    - [ ] Ejecución de una señal generada visualmente sin errores de atributo en Nautilus.
    - [ ] El `run_id` generado es rastreable en los logs de Nautilus desde el milisegundo 1.
* **¿Qué no puede pasar?** NO inyectar lógica que no haya pasado el gate anterior de Zero-Trust.

### **TTR-003: Orquestación de Grupos Aleatorios (WildCard Group Logic)**
* **¿Cuál es el problema?** El humano a veces conoce la "salida" (Take Profit) pero desconoce la "entrada" óptima (Alpha).
* **¿Qué tiene que pasar?** El compilador debe permitir la presencia de nodos de tipo `wildcard_group`. Durante la compilación, estos nodos se marcan como "pendientes de resolución" y se envían al motor genético (`nsga2-optimizer`) para ser completados.
* **¿Cómo sé que está hecho?**
    - [ ] El sistema compila exitosamente un AST con un nodo vacío marcado como WildCard.
    - [ ] El motor genético inyecta indicadores aleatorios en dicho nodo y el backtester lo evalúa sin errores de tipos.
* **¿Qué no puede pasar?** No se pueden ejecutar estrategias con WildCards en LIVE; deben ser resueltas y re-compiladas con nodos fijos antes de salir de incubación.

### **TTR-004: Resolución de Wildcard Invertido Multi-Dominio (ADR-0108)**
* **¿Cuál es el problema?** El TTR-003 resuelve `wildcard_group` únicamente para el Dominio de Señal (el humano fija la salida, el motor descubre la entrada). Los nuevos dominios genómicos del Registro (Riesgo y Gestión de Posición, Régimen y Filtro de Entorno, Portafolio y Correlación) requieren el patrón inverso: el Genoma de Señal queda fijo y el motor resuelve los nodos `wildcard_group` etiquetados para el dominio activo.
* **¿Qué tiene que pasar?** Durante la Validación Estructural, el compilador etiqueta cada nodo `wildcard_group` con su dominio genómico de origen. Al ensamblar el Manifest, expone al motor genético (`nsga2-optimizer`) los nodos cuyo dominio está incluido en `ACTIVE_GENOME_DOMAINS` (uno o varios), agrupados en Reglas Genómicas que pueden combinar Genes de Condición/Acción de distintos dominios activos; los nodos de los dominios fuera de `ACTIVE_GENOME_DOMAINS` se compilan como nodos fijos usando su última resolución conocida.
* **¿Cómo sé que está hecho?**
    - [ ] El sistema compila exitosamente un Manifest con nodos `wildcard_group` etiquetados en al menos dos dominios genómicos distintos, resolviendo solo el dominio activo.
    - [ ] Recompilar el mismo Manifest cambiando `ACTIVE_GENOME_DOMAINS` produce una resolución distinta del dominio recién activado, sin alterar los nodos de los dominios que quedaron congelados.
    - [ ] Activar simultáneamente 2+ dominios en `ACTIVE_GENOME_DOMAINS` produce Reglas Genómicas con Genes de Condición/Acción de ambos dominios en el mismo individuo evolutivo.
* **¿Qué no puede pasar?** Una Regla Genómica no puede referenciar Genes de Condición o de Acción de un dominio que no figure en `ACTIVE_GENOME_DOMAINS`, ni exceder `MAX_CONDITIONS_PER_RULE`/`MAX_ACTIONS_PER_RULE`.

## Persistencia (Filtro de Relevancia AI/R&D — ADR-0020 V2)

Cada genoma de estrategia compilado por este motor debe portar el set filtrado de metadatos para auditoría de diseño:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la compilación |
| | `created_at` | Timestamp de compilación |
| | `audit_hash` | Hash forense para inmutabilidad del genoma |
| **II. Soberanía** | `owner_id` | Autor de la estrategia |
| | `institutional_tag` | Etiqueta de la firma/experimento |
| | `manifest_id` | ID del contrato de diseño legal |
| | `access_token_id` | Permiso de compilación institucional |
| **III. Pesos/Arquitectura** | `logic_hash` | SHA-256 del contenido lógico |
| | `indicator_state_hash` | Configuración y estados de indicadores del AST |
| | `version_node_id` | Nodo en el DAG de versiones (Git-style) |
| | `parent_id` | ID de la estrategia padre (si es mutación) |
| **IV. Hardware** | `node_id` | ID del hardware de compilación |
| | `process_id` | PID del orquestador de compilación |
| | `session_id` | Sesión de usuario activa |

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2):** El JSON generado incluye `logic_hash` y `manifest_id` mandatorios.
- **IA Híbrida (ADR-0031/0113):** El esquema AST es la base de la regresión simbólica nativa (modo del motor NSGA-II); exportable a fórmulas matemáticas legibles.
- **Genomas Modulares por Dominio (ADR-0108):** El AST_VERSION 4.0 etiqueta cada `wildcard_group` con su dominio genómico de origen (Señal, Riesgo y Gestión de Posición — ADR-0109, Régimen y Filtro de Entorno — ADR-0110, Portafolio y Correlación — ADR-0111). Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
