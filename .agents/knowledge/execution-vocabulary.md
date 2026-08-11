# Vocabulario de Ejecución e Identificadores

Qué significa cada identificador de trabajo y cómo se asigna su número, para que dos documentos nunca reclamen el mismo.

Slug ID: `EXECUTION-VOCABULARY`.

Tamaño: `4933 caracteres (~1233 tokens aprox., ratio 4:1)`.

---

## EXECUTION-VOCABULARY-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/knowledge/writing-efficiency.md]] | Cómo se traduce un identificador al hablar con el usuario (`WRITING-EFFICIENCY-3`). | Al mencionar cualquiera de estos identificadores en un mensaje al propietario. | Todos los agentes |

---

## EXECUTION-VOCABULARY-FV — Filtro de Verificación

**Aplica a:** Tech-Lead y Architect, al crear cualquier documento de ejecución.

* **EXECUTION-VOCABULARY-FV-1:** ¿el tipo de identificador corresponde a la naturaleza del trabajo — con código o sin código? (`EXECUTION-VOCABULARY-1`)
* **EXECUTION-VOCABULARY-FV-2:** ¿el número salió del máximo de toda la carpeta `docs/execution/` más uno, no del siguiente de su propio tipo? (`EXECUTION-VOCABULARY-2.1`)
* **EXECUTION-VOCABULARY-FV-3:** ¿evité pre-reservar números para épicas que no son la activa? (`EXECUTION-VOCABULARY-2.3`)

De ser satisfactoria cada validación de la lista, declara `EXECUTION-VOCABULARY - FILTRO DE VERIFICACION APROBADO`.

---

## EXECUTION-VOCABULARY-1 — Los Identificadores

Estilo Jira: palabra completa más número, estables. Prohibidos los códigos crípticos de una letra.

**Aplica a:** Tech-Lead y Architect, al crear una Orden de Trabajo o al listar trabajo en el ROADMAP.

* **EXECUTION-VOCABULARY-1.1 — Tabla de Tipos:**

  | ID | Tipo | Qué es |
  |---|---|---|
  | `EPIC-0`…`EPIC-9` | Épica | una fase del producto (EPIC-0 = Fundación). Gran entregable. |
  | `SPRINT-n` | Sprint | grupo de trabajos despachados juntos. |
  | `STORY-###` | Story | trabajo que implica escribir código (implementa TTRs/feature/módulo). |
  | `SPIKE-###` | Spike | investigación de un riesgo técnico bloqueante. |
  | `BUG-###` | Bug | corrección de un defecto. |
  | `TASK-###` | Task | trabajo SIN código: investigación, escalar al Architect, registrar un ADR, seleccionar el siguiente trabajo. |

* **EXECUTION-VOCABULARY-1.2 — Unidades de Especificación, No de Trabajo:**

  **TTR**, **ADR**, **Feature** y **Módulo** conservan su nombre propio: no son IDs de trabajo, son unidades de especificación. Una Story *implementa* uno o más TTRs — relación "implementa", nunca "es padre de".

* **EXECUTION-VOCABULARY-1.3 — La Épica y el Sprint No Van en el ID:**

  viven en los metadatos de la Orden de Trabajo, igual que en Jira. Un `STORY-045` no cambia de número al moverse de sprint.

**Filtro de verificación:** `EXECUTION-VOCABULARY-FV-1`.

---

## EXECUTION-VOCABULARY-2 — Protocolo de Numeración

Cómo se elige el número siguiente, y por qué el Spike es la única excepción.

**Aplica a:** Tech-Lead y Architect, al crear cualquier `STORY-`, `TASK-` o `BUG-`.

* **EXECUTION-VOCABULARY-2.1 — Un Solo Contador Global para Story, Task y Bug:**

  los tres comparten una única secuencia. El siguiente número libre es el máximo usado por cualquiera de los tres más uno — **nunca "el siguiente de mi propio tipo"**. En la práctica: el máximo de toda la carpeta `docs/execution/`, más uno.
  * ❌ *Incorrecto:* crear `STORY-004` porque la última Story fue la 003, existiendo ya un `TASK-004`.
  * ✅ *Correcto:* barrer `docs/execution/` entera, tomar el máximo sin importar el tipo, y sumar uno.

* **EXECUTION-VOCABULARY-2.2 — El Spike Tiene Contador Propio, y Es Fijo por Diseño:**

  los Spikes de Viabilidad son una lista cerrada definida de antemano en [[docs/ROADMAP.md]] §6 — los riesgos técnicos bloqueantes de EPIC-0 —, no trabajo que se despacha incrementalmente. Por eso reutilizan los números 1-6 sin que eso sea una colisión. No es una excepción pendiente de corregir.

* **EXECUTION-VOCABULARY-2.3 — Solo la Épica Activa Lleva Número Asignado:**

  las épicas futuras se listan en el ROADMAP por nombre de feature o módulo, **sin número pre-reservado**. El número se asigna recién al crear su Orden de Trabajo, tomando el siguiente del contador global en ese momento. Es intencional: deja espacio para insertar un Task, Bug o Spike entre épicas sin renumerar nada retroactivamente.
  * ❌ *Incorrecto:* pre-asignar `STORY-090` a una feature de EPIC-5 mientras la fase activa es EPIC-1.
  * ✅ *Correcto:* listar esa feature por nombre en el ROADMAP y numerarla el día que se despache.

**Precedencia:** `EXECUTION-VOCABULARY-2.1` gana sobre cualquier conveniencia de lectura — un número duplicado obliga a renumerar y a reparar referencias cruzadas, algunas ya incrustadas en código.

**Filtro de verificación:** `EXECUTION-VOCABULARY-FV-2`, `EXECUTION-VOCABULARY-FV-3`.
