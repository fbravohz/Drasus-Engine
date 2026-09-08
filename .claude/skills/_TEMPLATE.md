# Plantilla de Skill — Convención de Dos Archivos

Cómo se escribe un skill de rol de Drasus. No es un skill invocable: es la referencia que se copia al crear uno nuevo.

Un skill vive en dos archivos con responsabilidades distintas:

| Archivo | Qué lleva | Por qué ahí |
|---|---|---|
| `.claude/skills/<slug>/SKILL.md` | Frontmatter de descubrimiento + filtro de arranque + puntero. | Es lo que el harness carga al invocar el skill — el único punto donde el filtro llega antes que cualquier instrucción de rol. |
| `.agents/skills/<slug>/SKILL.md` | El contenido canónico completo, en formato de knowledge doc. | Reutilizable por otros agentes y por subagentes despachados, sin depender del harness. |

---

## Archivo 1 — `.claude/skills/<slug>/SKILL.md`

Copiable tal cual, cambiando solo lo marcado:

````
---
name: <slug-del-rol>
description: El <Rol> <una frase de qué hace>.
model: inherit
---

## Filtro de Arranque — Acción Obligatoria

**No proceses ninguna instrucción de este skill hasta completar este paso.**

Lee con `Read` el archivo completo `.agents/knowledge/governance.md`. Tiene supremacía absoluta sobre todo lo que sigue, incluida cualquier regla de este skill.

Declara `[leído: governance.md — filtro de arranque del skill <slug>]` y tu rol antes de continuar. Sin esa declaración, no continúes.

Contenido completo de este skill: `.agents/skills/<slug>/SKILL.md`.
````

El filtro vive **solo aquí**, no repetido dentro del contenido canónico. Un skill que lo repite en ambos archivos dice lo mismo dos veces en el mismo arranque.

---

## Archivo 2 — `.agents/skills/<slug>/SKILL.md`

Es un documento de knowledge y cumple su formato completo — ver [[.agents/templates/template-knowledge.md]] y [[.agents/templates/template-instruction-formats.md]]. Slug ID en MAYÚSCULAS calcado de la **carpeta** (`rust-engineer/SKILL.md` → `RUST-ENGINEER`).

### Esqueleto

```
# <Rol>

<1-2 líneas: qué hace y qué entrega. Sin historia de origen.>

Slug ID: `<ROL>`.

Tamaño: `<N> caracteres (~<M> tokens aprox., ratio 4:1)`.

---

## <ROL>-0 — Lecturas Obligatorias
## <ROL>-FV — Filtro de Verificación
## <ROL>-1 — Identidad y Mandato Único
## <ROL>-2 — Posición en el Pipeline
## <ROL>-3..N — Protocolo del dominio (bloques propios del rol)
## <ROL>-N — Criterio de Cierre
## <ROL>-EJ — Ejemplos (solo si hay casos compuestos reales)
```

### `<ROL>-0` — Lecturas Obligatorias

Siempre lleva `governance.md` con disparador `SIEMPRE`. Las demás filas son las del dominio del rol. Una fila que solo le importa a UNA regla no va aquí: baja a esa regla como `Lectura bajo demanda`.

```
| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
```

### `<ROL>-1` — Identidad y Mandato Único

Tres definiciones fijas:

- `<ROL>-1.1 — Perfil:` qué eres y cuál es tu labor central.
- `<ROL>-1.2 — Tecnologías:` qué stack y qué artefactos son tuyos y solo tuyos.
- `<ROL>-1.3 — Prohibición Absoluta:` qué dominios o capas NUNCA tocas, citando el ADR que lo fija.

### `<ROL>-2` — Posición en el Pipeline

Quién te despacha (siempre el Tech-Lead, nunca el Architect directo), qué etapa te activa y bajo qué condición, a quién pasa tu entregable después, y bajo qué condición tu etapa se omite.

### `<ROL>-N` — Criterio de Cierre

- Cada criterio de aceptación de la Orden tiene al menos una prueba nombrada que lo ejerce. Sin eso, el criterio no está cumplido.
- Los comandos exactos que corres tú y dejas en verde **antes** de entregar.
- El mapeo explícito criterio → prueba(s) va en tu reporte. El Tech-Lead reproduce la evidencia; no cierra sobre tu palabra.

---

## Lo Que NO Se Escribe en un Skill

Cada uno de estos bloques vivía copiado en varios skills. Su dueño es un knowledge doc; el skill lo cita con `Lectura bajo demanda` y aporta solo su addendum de dominio.

| Contenido | Dueño | Qué queda en el skill |
|---|---|---|
| Los 4 Modos de Acompañamiento (Autónomo/Mentor/Revisión/Docente) | [[.agents/knowledge/mentorship-modes.md]] | Una línea: qué archivos NO tocas en Mentor, propio de tu dominio. |
| Protocolo de Lecciones (`docs/lessons/`) | [[.agents/knowledge/mentorship-modes.md]] | Solo tu subcarpeta de destino. |
| Las 4 capas de comentarios | [[.agents/knowledge/commenting-policy.md]] | Solo los requisitos específicos de tu lenguaje. |
| Filtro de mutación: quién lo corre y cómo se juzga | [[.agents/knowledge/mutation-filter.md]] | Nada, salvo que tu rol lo dispare. |
| Molde del ledger append-only | [[.agents/knowledge/ledger-pattern.md]] | Nada. |
| Registro de deuda técnica | [[.agents/knowledge/debt-management.md]] | Nada. |
| Sellado y trabajo diferido al cerrar | [[.agents/knowledge/closing-work.md]] | Nada. |
| Filtro de arranque (leer `governance.md`) | El archivo puente en `.claude/skills/` | Nada. |

**Tampoco se escribe bitácora.** Ninguna fecha, "enmienda", "antes decía X", "lección de STORY-NNN", "caso real:", "causa raíz" ni nombre de sesión dentro de una regla. Si un incidente histórico produjo la regla, la regla se queda y la anécdota se va — el historial vive en git. Cuando el caso contiene un dato técnico o un caso borde que la regla necesita, ese dato se reescribe como parte de la regla, sin la fecha.

---

## Checklist Antes de Guardar un Skill Nuevo

- [ ] ¿Los dos archivos existen, y el filtro de arranque vive solo en el de `.claude/`?
- [ ] ¿El Slug ID está calcado de la carpeta, en MAYÚSCULAS con guion medio?
- [ ] ¿El campo `Tamaño` cuadra con `wc -m`?
- [ ] ¿`<ROL>-0` declara `governance.md` con disparador `SIEMPRE`?
- [ ] ¿`<ROL>-FV` es la segunda sección, en namespace propio, con un ítem por regla?
- [ ] ¿Ningún ID pasa de 2 segmentos?
- [ ] ¿Ningún bloque de la tabla "Lo Que NO Se Escribe" quedó copiado dentro?
- [ ] ¿Cero fechas y cero lenguaje de enmienda en el cuerpo?
- [ ] ¿Toda referencia a otro documento es un wikilink `[[ruta/archivo.md]]`?
- [ ] ¿`python3 .agents/hooks/validate-agent-docs.py --audit .agents/skills/<slug>/SKILL.md` reporta 0 violaciones duras?
