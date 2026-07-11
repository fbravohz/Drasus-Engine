# Índice de la Base de Conocimiento

Los documentos de `knowledge/` son **HOW-TOs de lectura bajo demanda**, no contexto que se carga en masa. Este índice dice de qué va cada uno y **cuándo** abrirlo, para aplicarlo con criterio. Igual que `docs/ADR.md` indexa los ADR, este archivo indexa el conocimiento operativo.

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [`base.md`](./base.md) | Gobernanza meta + mapa de referencias. **Supremacía absoluta** sobre cualquier skill. | **SIEMPRE** al iniciar cualquier skill (gate de arranque obligatorio). No es bajo demanda. | Todos los agentes |
| [`commenting-policy.md`](./commenting-policy.md) | Cómo escribir comentarios (4 capas jerárquicas). | Al escribir/revisar comentarios en código. | Ingenieros de código |
| [`debt-management.md`](./debt-management.md) | Cómo registrar deuda técnica (`ponytail:` inline vs. `DEBT-XXX` en `docs/DEBT.md`). | Al detectar/registrar deuda o un atajo deliberado. | TL + ingenieros |
| [`ponytail.md`](./ponytail.md) | Simplificación deliberada (Capa 3 de comentarios): cuándo NO sobre-ingenierizar. | Al decidir el alcance de una implementación (evitar sobre-diseño). | TL + ingenieros |
| [`memory-policy.md`](./memory-policy.md) | Qué y cómo recordar entre sesiones (`.agents/memory/` vs. `.agents/state/`). | Al cerrar trabajo, aprender algo durable, o dudar de qué persistir. | TL + Architect |
| [`critical-domain-reasoning.md`](./critical-domain-reasoning.md) | Ojo crítico experto antes de **sellar decisiones de dominio** (7 comprobaciones + gate de especialista). Nació del casi-desastre del DSR. | Al canonizar/sellar cualquier regla cuantitativa, estadística, financiera, criptográfica, de microestructura, fiscal o legal — o cuando el concepto en curso toca ese terreno. | Architect, TL, ingenieros especialistas |

## Gate de Reportaje de Conocimiento (OBLIGATORIO)

La lectura bajo demanda solo es verificable si se **declara**. Por eso:

- **Cuando un agente detecte que el concepto en el que trabaja toca el área de un documento de este índice**, DEBE (1) leerlo bajo demanda y (2) **declarar explícitamente al invocador** —el usuario, o el Architect/Tech-Lead que lo despachó— **qué documento(s) de knowledge consultó y por qué** (qué conflicto o duda real lo disparó).
- La declaración va en el mensaje al invocador y, si aplica, en §8 de la Orden de Trabajo. Formato: `[knowledge consultado: <doc> — <motivo>]`.
- Esto le permite al invocador saber que el agente usó el conocimiento correcto para resolver un conflicto real, en vez de resolverlo por su cuenta cuando las instrucciones vigentes no bastaban.

`base.md` se declara siempre (gate de arranque); los demás se declaran **cuando se consultan**.
