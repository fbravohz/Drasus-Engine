# Strategy Self-Explanation

**Carpeta:** `./features/strategy-self-explanation/`
**Estado:** En Diseño
**Última actualización:** 2026-04-28
**Decisión Arquitectónica Asociada:** ADR-0051

## 1. ¿Qué es esta feature?
Módulo de auditoría que traduce un Árbol de Sintaxis Abstracta (AST) críptico (típicamente vomitado por el motor evolutivo genético) a un párrafo de texto legible por humanos. 
Resuelve el problema de la "Caja Negra Evolutiva", permitiendo al gestor auditar lógicamente qué ha descubierto el algoritmo.

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
- **Inundación de Fundaciones (ADR-0020 V2):**
  - **Perfil Documentación:** Identidad + Linaje Genético.
  - **Contrato de Persistencia:** (id, created_at, logic_hash, version_node_id, owner_id).
  - **Hooks Forenses:** Hash criptográfico de la explicación ligada al `logic_hash` de la estrategia, evidenciando el contrato de auditoría.
