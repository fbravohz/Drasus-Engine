# Strategy Self-Explanation

**Carpeta:** `./features/strategy-self-explanation/`
**Estado:** En Diseño
**Última actualización:** 2026-04-28
**Decisión Arquitectónica Asociada:** ADR-0051

## 1. ¿Qué es esta feature?
Módulo de auditoría que traduce un Árbol de Sintaxis Abstracta (AST) críptico (típicamente vomitado por el motor evolutivo genético) a un párrafo de texto legible por humanos. 
Resuelve el problema de la "Caja Negra Evolutiva", permitiendo al gestor auditar lógicamente qué ha descubierto el algoritmo.

## Narrativa de Producto

**Capacidad:** `CAP-09` — Una interfaz donde se ve el razonamiento

**Historia de usuario:** Como gestor de una estrategia, quiero abrir cualquier estrategia generada automáticamente y leer en un párrafo qué hace y por qué, para auditarla sin tener que descifrar su estructura interna nodo por nodo.

**El problema (técnico):** un motor evolutivo puede producir un árbol de decisión de decenas de condiciones encadenadas. Nadie lo entiende de un vistazo, ni siquiera el propio sistema que lo generó — y sin entenderlo, no hay manera real de confiar en él ni de explicárselo a un tercero que audite el capital.

**La solución (técnica):** el sistema recorre la estructura inmutable de la estrategia y redacta un párrafo en español que describe su lógica ("entra en compra cuando hay una expansión de volatilidad y el sesgo semanal es alcista"). Cada frase del reporte queda enlazada a los nodos exactos del árbol que la producen —un clic en la frase ilumina esos nodos—, y cualquier nodo se puede bloquear para que una optimización posterior lo respete sin tocarlo.

**Explicado sin tecnicismos:** es como pedirle a un traductor que convierta un contrato lleno de cláusulas cruzadas en un resumen de una página que cualquiera entiende, y que además, si señalas una frase del resumen, te marque exactamente qué cláusulas del contrato original la sostienen.

Aquí pasa lo mismo con una estrategia de cuarenta reglas: el sistema te da el resumen en español, y si tocas una frase de ese resumen, ves iluminadas las reglas exactas de las que salió. Si hay una regla que ya revisaste y te gusta como está, la puedes "sellar" para que nadie —ni el propio sistema al mejorar la estrategia después— la vuelva a tocar.

## 2. Comportamientos Observables
- [ ] En la inspección del Genoma, el usuario presiona "Auditar Lógica".
- [ ] El sistema descompone el grafo matemático y redacta un reporte: "La estrategia entra en compras cuando existe una expansión de volatilidad (ATR) y el sesgo semanal es alcista".
- [ ] **Trazabilidad Semántica Bidireccional:** El usuario hace clic en una frase del reporte (ej. "compra los martes de baja volatilidad") y la UI ilumina en rojo/azul los nodos matemáticos exactos del árbol de decisión que provocan ese comportamiento.
- [ ] **Candado de Nodo:** Tras auditar un nodo, el usuario puede "bloquearlo" con un candado para que la optimización o la IA no lo toquen, mientras el resto sí puede reoptimizarse.

## 3. Restricciones
- La salida del LLM es meramente informativa (read-only). No altera ni influencia la compilación Rust SIMD/Rayon o las decisiones de mercado.

## 4. Parámetros Configurables
| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| NARRATIVE_LANGUAGE | es-MX | en, es, fr | Idioma de la redacción humana | CONFIG |
| VERBOSITY | normal | normal / dev | Nivel de detalle técnico en el texto | CONFIG |

## 5. Estructura Interna (FCIS)
- **Core (Lógica Pura):** Serializador AST -> Representación Textual Intermedia (LISP-like).
- **Shell (Infraestructura):** Conexión al motor de LLM.
- **Frontera Pública:** Interfaz de solo lectura adyacente a la base de datos inmutable.

## 6. Ciclo de Vida de la Feature
### Entrada
- Genoma validado (AST).
### Proceso
- Convierte el árbol en un pseudo-código estandarizado.
- Alimenta el pseudo-código y los metadatos estadísticos al LLM.
### Salida
- Documento Markdown con la explicación de causales y filtros.
### Contextos de Uso
- **Contexto 1: Inspector de Estrategia (manage / withdraw)**
  - Documentar la vida y obra del algoritmo de forma auditable por riesgo y compliance institucional.

## 7. Tareas (TTRs)
### **TTR-001: Generación de Documentación Evolutiva**
* **¿Cuál es el problema?** El motor de IA encuentra combinaciones lógicas matemáticas que ningún humano entiende de un vistazo.
* **¿Qué tiene que pasar?** El módulo deserializa el AST y fuerza al modelo a traducirlo al español.
* **¿Cómo sé que está hecho?**
  - [ ] Abro una estrategia generada hace 5 días.
  - [ ] Leo un párrafo claro que me explica que utiliza "una divergencia de momento y un filtro de sesión europea".
* **¿Qué no puede pasar?**
  - El sistema no puede mentir: debe basar su explicación estrictamente en la topología inmutable provista.

### **TTR-002: Trazabilidad Bidireccional Frase↔Nodo y Candado**
* **¿Cuál es el problema?** Si la máquina generó una estrategia de 50 condiciones, auditar el XML o el código es tedioso. El humano quiere auditar la semántica y, si no le gusta una regla, bloquearla sin tocar el resto.
* **¿Qué tiene que pasar?** Al hacer clic en una frase del reporte, la UI resalta los nodos exactos del AST responsables de esa frase. El usuario puede poner un candado a un nodo; la optimización posterior respeta el candado y solo reoptimiza lo no bloqueado.
* **¿Cómo sé que está hecho?**
  - [ ] Clic en una frase → se iluminan los nodos correspondientes en el árbol.
  - [ ] Bloqueo un nodo y la siguiente optimización no lo modifica.
* **¿Qué no puede pasar?**
  - El mapeo frase↔nodo NUNCA puede apuntar a un nodo que no contribuye a esa frase (la trazabilidad debe ser fiel a la topología inmutable).
  - La optimización NUNCA altera un nodo con candado.
* **Slice Visual (Flutter/Impeller/FFI):** Panel de lectura humana con frases clicables; resaltado del árbol DAG vía FFI; el mapeo se computa en el Core sobre el AST.

## 8. Gobernanza y Estándares (Fijos)
- **Local-First / Cloud-Hybrid:** Se recomienda ejecución en API o local Llama 3 por privacidad de Alpha.
- **Inundación de Fundaciones (ADR-0020):**
  - **Perfil B (IA / R&D):** la explicación deriva de la generación de la estrategia; la etiqueta "Documentación" no existe en la tabla canónica.
  - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
  - **II. Soberanía & Propiedad:** `owner_id`, `manifest_id`.
  - **III. Pesos/Arquitectura (subset):** `logic_hash`, `version_node_id`.
  - **Contrato de Persistencia:** Grupo I completo + Perfil B (II + III subset) arriba.
  - **Hooks de Auditoría:** Hash criptográfico de la explicación ligada al `logic_hash` de la estrategia, evidenciando el contrato de auditoría.
