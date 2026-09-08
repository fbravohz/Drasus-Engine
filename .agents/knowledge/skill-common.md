# Bloques Comunes de Skill

- 4 bloques hoy repetidos casi literalmente en 12 de los 15 `SKILL.md` (partner, general-counsel, qa-engineer, quant-engineer, bridge-engineer, rust-engineer, refactoring-engineer, flutter-engineer, ui-designer, tech-lead, auditor, adversary) — cada skill afectado reemplaza su copia local por una línea de referencia al bloque de aquí. Sin bloque de Protocolo de Lecciones: ya vive centralizado en [[.agents/knowledge/mentorship-modes.md]] (`MENTORSHIP-MODES-3`) — cada skill referencia eso directo, no hace falta duplicarlo aquí también. Sin bloque de Cierre del Filtro de Verificación: el aparato `<DOC>-FV` completo ya se retiró del catálogo (`TEMPLATE-INSTRUCTION-FORMATS-8`) — reintroducir su línea de cierre aquí lo habría revivido por la puerta de atrás.
- SLUG-ID: `SKILL-COMMON`
- Tamaño: `5341 caracteres (~1335 tokens aprox., ratio 4:1)`

## SKILL-COMMON-1: Filtro de Arranque

- Paso 1 del Pipeline de todo skill que lo tenga: lee `governance.md` completo, declara `[leído: governance.md]` + rol (`GOVERNANCE-1.1`, `-1.4`).
- Skill sin bloque `<SLUG>-P` (Pipeline): el mismo requisito va como primer bullet de su Sección de arranque, misma redacción.

## SKILL-COMMON-2: Modos de Acompañamiento

- **Alcance:** aplica a todo skill que ejecute una Story/Task bajo el Tech-Lead con acompañamiento del usuario. No aplica a un skill de invocación directa sin pipeline (ej. auditoría bajo demanda) ni a una etapa de traducción de una sola pasada — esos casos se documentan como excepción explícita en el propio skill, no se fuerza el bloque donde no corresponde.
- **Localización:** ubica la fila del rol activo en la tabla de Modos de Acompañamiento — nunca se asume del chat. Default: Autónomo.
- **Mentor:** no usa `Edit`/`Write` sobre los archivos de lógica de negocio. En su lugar: (1) explica el concepto del bloque que sigue, con profundidad cero-conocimiento — nunca asume que el usuario ya domina el tema ([[.agents/knowledge/mentorship-modes.md#MENTORSHIP-MODES-1]]); (2) dicta el fragmento exacto a teclear, con archivo y ubicación; (3) espera confirmación del usuario de que ya lo escribió; (4) relee el archivo con `Read`, compara contra lo dictado y corrige/explica cualquier desviación antes de pasar al siguiente bloque. Granularidad pequeña: un bloque lógico por vez, nunca un archivo completo de un golpe. Puede usar `Bash` para correr verificación (build/lint/test), nunca para arreglar el código por el usuario salvo que se lo pida explícitamente.
- **Revisión:** no reescribe — evalúa el trabajo ajeno contra el criterio de aceptación y reporta veredicto explícito, con la lista de fallas concretas si aplica. Cada skill puede agregar su propio checklist de dominio encima de este principio.
- **Docente:** mecánica genérica (implementa, pausa, enseña) en [[.agents/knowledge/mentorship-modes.md#MENTORSHIP-MODES-2]] — no se repite aquí.
- **Cierre transversal:** en los cuatro Modos, el criterio de aceptación y los comandos de validación de la Orden se cumplen igual, sin excepción. Documenta el Plan de Implementación (Mentor/Docente) o el Checklist de Revisión (Revisión) dentro del bloque de la Orden — no solo en el chat (Spec-Driven, ADR-0120).

## SKILL-COMMON-3: Demostración Roja/Verde

- No basta con afirmar que una prueba discrimina. Para cada criterio de aceptación, y para toda prueba que cubra una garantía de comportamiento, se demuestra con hechos: (1) rompe a propósito el comportamiento que la prueba dice cubrir (comenta la línea, invierte la condición, devuelve una constante); (2) corre la prueba y pega la salida en rojo en el reporte; (3) revierte la rotura; (4) corre la prueba y pega la salida en verde. Cuatro pasos, imposible de fingir. Si el reporte no trae el rojo, la prueba se considera no verificada.
- Extensión de dominio (ej. ciclo de estados visuales en UI) vive local al skill que la necesita, sobre esta misma base.

## SKILL-COMMON-4: Formato de Reporte al CEO

- **Uso:** único formato de comunicación con el usuario fuera de Modo Mentor/Revisión — se aplica cada vez que se reporta avance, se pausa para pedir una decisión, o se cierra una Story, no solo al final. Sin saludos, sin "recap:", sin relleno de apertura o cierre: directo al bloque.
- **Plantilla:**

```
ESTADO: 🟢 COMPLETADO | 🟡 REQUIERE DECISIÓN | 🔴 BLOQUEADO

PROGRESO MACRO:
- <1-2 oraciones. Avance funcional en términos de negocio. Traduce todo identificador con [[.agents/knowledge/writing-efficiency.md]] (WRITING-EFFICIENCY-3).>

FRICCIONES Y DEUDA:
- <Opcional — omite la sección completa si no hay nada que reportar. Traducida a impacto real, nunca como ID crudo de DEBT.md.>

INPUT REQUERIDO DEL CEO:
- <Si ESTADO es 🟡 o 🔴: presenta el bloqueo de forma conceptual, con opciones concretas (A/B). Nunca pidas al CEO que audite o depure código.>
- <Si ESTADO es 🟢: escribe literalmente "Ninguna." — no omitas el campo ni lo dejes en blanco.>
```

- **Extensión por rol:** Auditor y Adversary comparten este mismo principio (traducción de jerga, Modo Gemini Relajado + Listas sin esperar activación, cero relleno) pero con su propio esquema de veredicto (`APROBADO`/`APROBADO CON OBSERVACIONES`/`RECHAZADO`/`RECHAZADO — SIN PLAN QUE AUDITAR`/`PÉRDIDA`) citando hallazgos por archivo y línea — ese esquema de veredicto vive en cada uno de esos skills, no se duplica aquí.
