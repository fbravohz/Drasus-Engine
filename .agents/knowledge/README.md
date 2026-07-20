# Índice de la Base de Conocimiento

Los documentos de `knowledge/` son **HOW-TOs de lectura bajo demanda**, no contexto que se carga en masa. Este índice dice de qué va cada uno y **cuándo** abrirlo, para aplicarlo con criterio. Igual que `docs/ADR.md` indexa los ADR, este archivo indexa el conocimiento operativo.

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [`governance.md`](./governance.md) | Identidad + supremacía + gate de arranque + auto-validación + idioma + checkpoint. **Núcleo siempre activo** — es lo único que reemplaza al antiguo `base.md` monolítico. | **SIEMPRE** al iniciar cualquier skill (gate de arranque obligatorio). No es bajo demanda. | Todos los agentes |
| [`commenting-policy.md`](./commenting-policy.md) | Cómo escribir comentarios (4 capas jerárquicas). | Al escribir/revisar comentarios en código. | Ingenieros de código |
| [`debt-management.md`](./debt-management.md) | Cómo registrar deuda técnica (`ponytail:` inline vs. `DEBT-XXX` en `docs/DEBT.md`); referencia por documento canónico, no por ejemplo (DEBT-032). | Al detectar/registrar deuda, un atajo deliberado, o interpretar "sigue el patrón de X". | TL + ingenieros |
| [`mutation-gate.md`](./mutation-gate.md) | Cómo correr y juzgar el gate de mutación: por qué solo lo corre el TL, el script determinista, y cómo se juzga un superviviente equivalente. | Al cerrar una Story con lógica de correctitud, o al juzgar supervivientes. | TL + QA |
| [`ledger-pattern.md`](./ledger-pattern.md) | **Molde canónico** del ledger append-only: los 7 puntos y las 4 pruebas obligatorias. Sustituye a "copia la feature que ya lo tiene". | Al crear o tocar una tabla append-only. | TL + ingenieros |
| [`ponytail.md`](./ponytail.md) | Simplificación deliberada (Capa 3 de comentarios): cuándo NO sobre-ingenierizar. | Al decidir el alcance de una implementación (evitar sobre-diseño). | TL + ingenieros |
| [`memory-policy.md`](./memory-policy.md) | Qué y cómo recordar entre sesiones (`.agents/memory/` vs. `.agents/state/`); auto-edición de skill ante corrección. | Al cerrar trabajo, aprender algo durable, dudar de qué persistir, o recibir una corrección de comportamiento. | TL + Architect |
| [`critical-domain-reasoning.md`](./critical-domain-reasoning.md) | Ojo crítico experto antes de **sellar decisiones de dominio** (7 comprobaciones + gate de especialista). Nació del casi-desastre del DSR. | Al canonizar/sellar cualquier regla cuantitativa, estadística, financiera, criptográfica, de microestructura, fiscal o legal — o cuando el concepto en curso toca ese terreno. | Architect, TL, ingenieros especialistas |
| [`writing-efficiency.md`](./writing-efficiency.md) | Fuente única de escritura/comunicación/claridad: economía de tokens (60-75% menos, medido), traducción de términos al usuario, Contexto Antes de Detalle, formato visual de bloques. | Al escribir cualquier documento o respuesta — sobre todo con 2+ ítems paralelos, jerga interna, o una explicación estructurada. | Todos los agentes |
| [`context-window-budget.md`](./context-window-budget.md) | Rangos de contexto confiable por modelo (vs. ventana nominal) para dimensionar el arranque de un skill y cuándo compactar. | Al escribir Lecturas Obligatorias de un skill, dimensionar `CLAUDE.md`/knowledge docs, o decidir si compactar una conversación larga. | Context Engineer, Architect, TL |
| [`tool-usage.md`](./tool-usage.md) | `glob` no lee dot-directories; protocolo de lectura progresiva para archivos extensos. | Al buscar dentro de una carpeta con punto, o leer un archivo >2000 líneas/50KB. | Todos los agentes |
| [`closing-work.md`](./closing-work.md) | Sellado con fecha, comandos de validación, y registro obligatorio de trabajo diferido. | Al cerrar cualquier TTR/Feature/Módulo/Story. | Todos los roles |
| [`mentorship-modes.md`](./mentorship-modes.md) | Modos Mentor/Revisión/Docente (ADR-0120/0122), profundidad cero-conocimiento, Protocolo de Lecciones (ADR-0124). | Al invocar Modo Mentor/Revisión/Docente en un Ingeniero. | Los seis Ingenieros |
| [`persistence-claims.md`](./persistence-claims.md) | Prohibido declarar persistencia sin citar archivo y sección exactos. | Antes de afirmar que algo "quedó registrado/guardado/como regla". | Todos los agentes |
| [`doc-first.md`](./doc-first.md) | Buscar con `grep` sobre `docs/` antes de proponer una regla, contrato, fórmula o cambio de alcance. | Antes de proponer cualquier cambio de dominio o arquitectura. | Todos los agentes |
| [`provider-bypass.md`](./provider-bypass.md) | Detección de valores hardcoded que evitan el theme/provider del sistema. | Al escribir/revisar código de UI (Flutter principalmente). | Ingenieros de UI, TL |

## Gate de Reportaje de Conocimiento (OBLIGATORIO)

La lectura bajo demanda solo es verificable si se **declara**. Por eso:

- **Cuando un agente detecte que el concepto en el que trabaja toca el área de un documento de este índice**, DEBE (1) leerlo bajo demanda y (2) **declarar explícitamente al invocador** —el usuario, o el Architect/Tech-Lead que lo despachó— **qué documento(s) de knowledge consultó y por qué** (qué conflicto o duda real lo disparó).
- La declaración va en el mensaje al invocador y, si aplica, en §8 de la Orden de Trabajo. Formato: `[knowledge consultado: <doc> — <motivo>]`.
- Esto le permite al invocador saber que el agente usó el conocimiento correcto para resolver un conflicto real, en vez de resolverlo por su cuenta cuando las instrucciones vigentes no bastaban.

`governance.md` se declara siempre (gate de arranque); los demás se declaran **cuando se consultan**.
