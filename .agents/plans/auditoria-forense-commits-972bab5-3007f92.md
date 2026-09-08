# Plan — Auditoría forense de los 5 commits desde `972bab5` hasta `HEAD`

Auditoría de detective sobre el rango `972bab5^..HEAD` (5 commits, 2026-08-11 → 2026-08-24), con foco en `972bab5` y `69d0fff` porque nadie los auditó al momento de comitear. Objetivo: verificar que lo que se dijo que se hizo, se hizo; que no se perdió información en las reescrituras; y que se respetaron formatos y plantillas.

---

## Método

- **Verificación mecánica propia:** el hook `validate-agent-docs.py --audit` sobre los 44 archivos normativos, censo estructural de los 14 skills, y los criterios de cierre que el propio plan `refactor-masiva-agents.md` se autoimpuso.
- **Barridos delegados:** 5 subagentes en paralelo, cada uno comparando `git show <commit>^:<ruta>` contra `git show <commit>:<ruta>` con lectura del 100% de ambas versiones. Ninguno editó nada.
- **Contraste contra lo declarado:** el plan `.agents/plans/refactor-masiva-agents.md` y el reporte `.agents/sessions/2026-08-07-refactor-masiva-agents.md` son la lista de promesas; se auditó realidad contra promesa.

---

## Veredicto global

**El trabajo de fondo es bueno y las reescrituras son mayormente honestas.** En volumen, la inmensa mayoría del contenido normativo sobrevivió: ninguna regla, umbral operativo ni ID desapareció sin equivalente en los 22 knowledge docs. El archivado del sistema de diseño viejo fue **fiel y superset** (más completo que el original), y el `DESIGN.md` nuevo es un reset legítimo y autosuficiente.

**Pero sí hay tres cosas que exigen acción antes de seguir construyendo:**

- **Una decisión de producto que se perdió sin tomarse** (N-1): al borrar el volcado de conversación se fue el único lugar donde se argumentaba que el motor no debe morir con la UI. Hoy el modo por defecto hace exactamente eso, y ningún ADR lo reconoce como trade-off aceptado.
- **Una pérdida de contenido con impacto funcional** (N-2): `general-counsel` se quedó sin definición operable de sus Modos de Acompañamiento, remitido a un documento que lo excluye de su alcance.
- **Un runbook roto** (N-3): `docs/TEST.md` manda un gate obligatorio de QA que hoy falla.

**Fuera de eso, el patrón dominante no es la calidad — es el cierre.** Cuatro síntomas del mismo hábito:

1. **Trabajo incompleto comiteado como completo.** El mensaje de `972bab5` declara "armonización de 13 skills"; el propio plan del commit declaraba en su cuerpo "Pendiente — 9 skills". Hoy siguen **5 skills fuera del canon**, bloqueados por el hook que ese mismo commit introdujo.
2. **Una deuda con disparador "pagar antes de comitear" que se comiteó igual** (`CONTEXT-DEBT-003`). Su impacto dejó de ser hipotético: la pérdida ya está en `HEAD`.
3. **Un error auto-confesado por escrito y no corregido en tres semanas** (N-4): los punteros temáticos mal redirigidos en 4 skills, anotados en la propia bitácora de sesión.
4. **Un rename masivo de carpeta sin barrido de referencias.** 44 referencias vivas a `ui/` quedaron rotas.

**Buena noticia que conviene decir explícita:** una auditoría previa (2026-08-20) ya detectó gran parte de esto y lo registró como `CONTEXT-DEBT-001..008`. Este plan **no la repite** — la verifica, corrige sus cifras desfasadas y añade lo que no cubrió.

---

## ✅ Decisiones del propietario — TOMADAS (2026-08-31)

Las cinco están resueltas. Se conservan aquí con su respuesta para que nadie las vuelva a preguntar.

### D-1 · Motor y UI en el mismo proceso (hallazgo N-1) → **adelantar la separación de procesos**

Se adelanta lo que ADR-0119 difería a EPIC-9+, **pero se ejecuta con el Architect**: se deja todo preparado en el `ROADMAP` de forma que, cuando el Tech-Lead lo lea, **sepa que esto bloquea** y se resuelva en la **siguiente Story**.

Qué falta hacer:
1. **Architect:** ADR que declara la separación de procesos motor/UI como decisión adelantada, enmendando ADR-0033 (`LocalPowerUser`) y ADR-0107 (`LiveNode` in-process), y referenciando ADR-0119.
2. **Architect:** entrada en `docs/ROADMAP.md` marcada como **bloqueante**, con el alcance de la Story que la resuelve.

### D-2 · Modos de `general-counsel` (hallazgo N-2) → **restaurar en el skill** ✅ HECHO

Se repuso su definición propia como bloque `GENERAL-COUNSEL-6` (Autónomo, Mentor, Revisión, Docente) más su ítem de Filtro de Verificación. No se tocó el alcance de `mentorship-modes.md` ni ADR-0120.

### D-3 · Referencias rotas a `ui/` (hallazgo N-3) → **marcar como suspendido**

En cada punto se declara explícitamente que la ruta espera al `ui/` nuevo, aún por construir. No se redirige a `ui-old/` ni se promete un comando que no corre.

Alcance: 44 referencias en 16 archivos, incluido el gate obligatorio de QA de `docs/TEST.md`.

### D-4 · `base.md` → **sí se borra**

Se ratifica la Fase 0 del plan original. Secuencia obligatoria:
1. Remapear las **4 referencias vivas** que lo citan (`.agents/state/tech-lead/PROGRESS.md:411` cita además una sección muerta, `base.md §7.4` → `doc-first.md`).
2. Quitar `base.md` de la lista de exenciones del hook (`NO_SLUG_REQUIRED` y `STUB_FILES` en `.agents/hooks/validate-agent-docs.py`).
3. Borrar el archivo.
4. Verificar: `grep -rn "base\.md" .` sin resultados vivos.

### D-5 · Canonización de `tech-lead` → **formato + partirlo en piezas**

Además de la cabecera canónica, el bloque de verificación y la purga de las 36 marcas de bitácora, se extraen a knowledge docs propios:
- El **Pipeline de Ejecución** (las 8 etapas, hoy en negrita y por tanto invisible a cualquier índice).
- El **Gate de Coherencia Pre-Despacho**.

Queda en el skill la identidad, el mandato y las referencias.

---

## 📍 Estado de ejecución (2026-08-31)

**Hecho y verificado** — hook `validate-agent-docs.py`: de 10 violaciones duras a **2** (solo `rust-engineer`).

- ✅ **N-2** `general-counsel` recupera sus Modos (`GENERAL-COUNSEL-6` + `-FV-7`).
- ✅ **N-4** punteros temáticos corregidos en `bridge-engineer` y `quant-engineer` (→ `mentorship-modes.md` / `commenting-policy.md`).
- ✅ **N-5** evidencia restaurada: caso `DEBT-005`, cifras de `DEBT-032` ("dos defectos durante meses en catorce implementaciones"), lista de migraciones `0010–0020` en `ledger-pattern.md`, puntero a `governance.md` en `commenting-policy.md`.
- ✅ **N-9** wikilinks: 6 en `ui-verification-model.md`, 1 en `cross-platform-rust.md`.
- ✅ **N-10** ruta `.claude/skills/tech-lead.md` corregida en los skills ya canonizados.
- ✅ **N-11** `brief.md` indexado en `.agents/knowledge/README.md`.
- ✅ **N-12** `STORY-009-cli-app.md` y la promesa de `.claude/rules/comments.md` (reformulada, ya no promete un archivo inexistente).
- ✅ **N-13** `PARTNER-FV-7` añadido, `PARTNER-4.1` renumerado, checklist de Modo Revisión repuesto en `refactoring-engineer` (con su umbral de 400 líneas).
- ✅ **CONTEXT-DEBT-002** contradicción de memoria resuelta (puntero marcado como histórico suspendido).
- ✅ **CONTEXT-DEBT-003** pagada: verificación FFI repuesta, `UI-DESIGNER-7.3` de vuelta a prohibición absoluta, ADR-0116/0038/0131 repuestos.
- ✅ **CONTEXT-DEBT-006** resuelta en sus dos mitades (`commenting-policy` y `partner`).
- ✅ **Canonizados:** `social-strategist`, `bridge-engineer`, `quant-engineer` (más `architect` en un commit previo).
- ✅ **D-5** `tech-lead` canonizado y partido en `.agents/knowledge/order-compliance-gate.md` + `.agents/knowledge/execution-protocol.md` (ambos nuevos, canónicos); salda `CONTEXT-DEBT-007` y la parte de `CONTEXT-DEBT-004` correspondiente a `tech-lead`.

**Pendiente.**

| Qué | Bloqueado por | Dueño |
|---|---|---|
| Canonizar `rust-engineer` (2 violaciones duras) | — listo para ejecutar | `context-engineer` |
| ~~Canonizar `tech-lead` + partirlo en piezas~~ | ✅ Hecho — 0 violaciones duras, 0 avisos. Partido en `order-compliance-gate.md` + `execution-protocol.md` (nuevos), ambos canónicos | `context-engineer` |
| Canonizar `qa-engineer` (`CONTEXT-DEBT-008`, avisos blandos) | — listo para ejecutar | `context-engineer` |
| ADR de separación de procesos + entrada bloqueante en `ROADMAP` | — listo, con alcance de D-1 | `architect` |
| Marcar como suspendidas las 44 referencias a `ui/` | — listo, con alcance de D-3 | `architect` (docs) + `context-engineer` (.agents) + `tech-lead` (`docs/TEST.md`) |
| Borrar `base.md` en 4 pasos | — listo, con alcance de D-4 | `context-engineer` |
| `CONTEXT-DEBT-004` wikilinks a los 5 knowledge nuevos | 🟠 Parcial — `tech-lead` saldó su parte (4 de 5 docs); siguen `architect`, `qa-engineer`, `rust-engineer`, `ui-designer`, `flutter-engineer` | `context-engineer` |
| ~~`CONTEXT-DEBT-007` numeración de etapas de `tech-lead`~~ | ✅ Pagada — resuelta dentro de D-5 | `context-auditor` audita → `context-engineer` corrige |
| `N-7` reformatear `DEBT-046` | — listo para ejecutar | `architect` |
| Actualizar fichas `CONTEXT-DEBT-001/003/008` y abrir `009` | Tras cerrar lo anterior | `context-engineer` |

---

## Parte 0 — Estado real de la auditoría previa (`CONTEXT-DEBT-001..008`)

Las 8 fichas se verificaron una por una contra el árbol actual. **Las 8 siguen abiertas y son legítimas**, pero dos necesitan corrección:

| Ficha | Estado verificado hoy | Acción |
|---|---|---|
| `CONTEXT-DEBT-001` | **Cifras desfasadas.** Dice "6 skills / 12 violaciones duras". Real hoy: **5 skills / 10 violaciones** — `architect` ya fue corregido (tiene `Slug ID` y `Tamaño`, cambio *staged* sin comitear). Los 5 vigentes: `bridge-engineer`, `quant-engineer`, `rust-engineer`, `social-strategist`, `tech-lead`. | Corregir cifras en la ficha |
| `CONTEXT-DEBT-002` | **Vigente.** `debt-management.md:146` sigue mandando leer `[[.agents/memory/comentarios-ponytail-reconciliacion.md]]`, que `MEMORY-POLICY-14.3` prohíbe. La línea 216 del mismo archivo sí está alineada — la contradicción es interna. | Sin cambio, pagar |
| `CONTEXT-DEBT-003` | **ESCALAR a 🔴 Alta.** Su disparador era "antes de comitear este archivo" y **se comiteó igual** el 2026-08-24. Verificado en `HEAD`: `UI-DESIGNER-7.3` quedó como condicional ("sin visto bueno del Architect" en vez de prohibición absoluta); `ADR-0131` y `ADR-0038` tienen **0 menciones**; la verificación FFI (`rust/api`) tiene **0 menciones**. El impacto ya no es nulo. | Reescribir impacto + subir severidad |
| `CONTEXT-DEBT-004` | **Vigente y confirmada.** Los 5 knowledge nuevos siguen citados por **0 skills**. Están correctamente indexados en `README.md`, pero nadie los lee: el contenido duplicado inline sigue siendo la copia efectiva. | Sin cambio, pagar |
| `CONTEXT-DEBT-005` | **Vigente.** 6 referencias a `ui/COMPONENTS.md` en `flutter-engineer` y `ui-designer`. | Absorber en la deuda nueva de rutas (abajo) |
| `CONTEXT-DEBT-006` | **Vigente.** `commenting-policy.md:31` (`COMMENTING-POLICY-1` como ítem) y `partner/SKILL.md:114` (`PARTNER-4` como ítem). | Sin cambio, pagar |
| `CONTEXT-DEBT-007` | **Vigente.** `tech-lead/SKILL.md:310` — "Barrido de Cierre Documental (OBLIGATORIO...)" sigue sin número de etapa. | Sin cambio, pagar |
| `CONTEXT-DEBT-008` | **Vigente.** `qa-engineer` tiene `Slug`/`Tamaño` pero sin bloque `-FV` y con 3 marcas de bitácora. | Sin cambio, pagar |

---

## Parte 1 — Hallazgos nuevos, no cubiertos por la auditoría previa

### 🔴 N-1 · Un riesgo de arquitectura quedó sin reconciliar al borrar el volcado de conversación

`3007f92` borró `conversacion-flutter-tauri-react-etc.md` como "volcado temporal". De sus tres bloques de decisión, dos **sí** estaban ya canonizados y no se perdió nada:

- FFI vs. Tauri IPC → ya sellado en ADR-0029, ADR-0019, ADR-0104, ADR-0106, ADR-0116 (de hecho, anterior a la conversación).
- Arquitectura distribuida multi-nodo → canonizada en ADR-0033, ADR-0119 y el modelo de Tres Planos.

**El tercero no.** La conversación argumentaba que el motor no debe compartir proceso con la UI ni siquiera en local: *"si está en el mismo proceso que tu React (vía FFI), se lleva la UI entera [un crash]… el motor no puede morir con la UI"*, calificándolo de descalificante para un producto 24/7.

El canon actual hace exactamente lo contrario, y nadie lo declaró como trade-off aceptado:
- **ADR-0033** fija `LocalPowerUser` — el modo **por defecto** — como "Flutter y Rust operan en el mismo proceso (OS) mediante FFI".
- **ADR-0107** confirma que el `LiveNode` de Execute corre "dentro del mismo proceso Rust del Core… sin puentes inter-proceso".
- **ADR-0119**, la única mitigación (multi-nodo), está **explícitamente diferida**: *"pertenece a la fase SaaS/distribuida (EPIC-9+)… No se construye hasta que el Core Local genere Alpha real"*.

Consecuencia hoy, en el modo por defecto: **cerrar la ventana de Flutter mata el motor de ejecución en vivo**. No hay ADR que reconozca ese riesgo como aceptado a propósito ni deuda que lo programe. Al borrar el archivo sin extraer esta pieza, se perdió el razonamiento — y con él, la oportunidad de decidirlo conscientemente.

**Esto no es un hallazgo documental: es una decisión de producto pendiente.** Requiere del propietario, no del `context-engineer`.

### 🔴 N-2 · `general-counsel` quedó sin definición operable de sus Modos de Acompañamiento

`972bab5` borró la sección propia "Modos de Acompañamiento" de `general-counsel/SKILL.md` — que definía comportamiento específico del rol legal (*"Mentor: no redactas el documento final; explicas… la cláusula exacta recomendada"*, *"Docente: sí redactas, y antes de avanzar enseñas qué principio legal aplicaste"*) — y la reemplazó por una fila que remite a `[[.agents/knowledge/mentorship-modes.md]]`.

Pero ese documento **declara su alcance como "los seis Ingenieros (Rust, Flutter, Bridge, QA, Quant, Refactoring)"** en su tabla `FV` y en cada regla `MENTORSHIP-MODES-1/2/3`, coherente con ADR-0120. La propia fila copiada en `general-counsel` reproduce ese Alcance — es decir, **remite a un documento cuyo alcance lo excluye explícitamente**.

Resultado: si una Orden de Trabajo declara Modo Mentor, Revisión o Docente para General Counsel, **no existe ninguna definición operativa de qué debe hacer**. Es pérdida de contenido con impacto funcional, no cosmética.

### 🔴 N-3 · 44 referencias vivas a `ui/` rotas tras el rename, incluido un runbook de QA que falla

`69d0fff` renombró `ui/` → `ui-old/` completo y **no barrió las referencias**. `ui/` ya no existe en el árbol. Conteo por archivo (excluyendo historial, que es correcto que diga `ui/`):

| Archivo | Refs | Gravedad |
|---|---|---|
| `.agents/skills/tech-lead/SKILL.md` | 5 | Instrucción operativa activa |
| `docs/features/verification-bench.md` | 4 | |
| `docs/features/hybrid-data-transformer.md` | 4 | |
| `.agents/skills/ui-designer/SKILL.md` | 4 | `CONTEXT-DEBT-005` |
| `.agents/knowledge/ui-verification-model.md` | 4 | Doc canónico nuevo, nace roto |
| `docs/ROADMAP.md` | 3 | |
| `docs/features/sovereign-data-fetcher.md` | 3 | |
| `docs/adr/ADR-0138.md` | 3 | ADR vigente describe estructura inexistente |
| `.agents/skills/qa-engineer/SKILL.md` | 3 | |
| `docs/TEST.md` | 2 | **Runbook roto** |
| `docs/adr/ADR-0117.md` | 2 | |
| `.agents/skills/flutter-engineer/SKILL.md` | 2 | `CONTEXT-DEBT-005` |
| `.agents/skills/bridge-engineer/SKILL.md` | 2 | Instrucción operativa activa |
| `docs/features/data-import-wizard.md`, `docs/DEBT.md`, `docs/adr/ADR-0152.md` | 1 c/u | |

**Lo más grave:** `docs/TEST.md:35` da `cd ui && flutter build linux` como **gate obligatorio de QA**, y `:37`/`:43` dan `cd ui && flutter run -d linux` como el comando para abrir la app. Hoy los tres fallan.

`DEBT-046` **no cubre esto** — solo cubre las 7 secciones `## Cáscara Visual` de features. Este hallazgo no está registrado en ninguna deuda.

### 🟠 N-4 · Punteros temáticos mal redirigidos por un find-replace masivo — auto-confesado y sin corregir

Al reparar el gate roto, las 29 referencias a `base.md` se redirigieron **todas** a `governance.md` con un reemplazo masivo, sin distinguir el destino por tema. El propio autor lo dejó escrito en su bitácora (`.agents/sessions/2026-08-07-refactor-masiva-agents.md:323`):

> *"el destino correcto depende del tema — la Política de Comentarios pertenece a `commenting-policy.md` y el Protocolo de Lecciones a `mentorship-modes.md`. Lo corregí en `refactoring-engineer` y anoté los cinco skills donde sobrevive."*

Estado verificado en `HEAD`, tres semanas después: **4 de los 5 siguen rotos.**

| Skill | Puntero incorrecto | ¿Sigue roto? |
|---|---|---|
| `bridge-engineer` | Protocolo de Lecciones + Política de Comentarios → `governance.md` | **Sí** |
| `qa-engineer` | Igual patrón | **Sí** |
| `quant-engineer` | Igual patrón | **Sí** |
| `rust-engineer` | Protocolo de Lecciones → `governance.md` | **Sí** |
| `flutter-engineer` | Igual patrón | No — reescrito en `69d0fff` |

`governance.md` no contiene **nada** sobre Protocolo de Lecciones ni Política de Comentarios (verificado por grep: cero coincidencias de "Autónomo/Mentor/Revisión/Docente"). El agente que siga el puntero llega a un documento que no responde su pregunta.

Ironía verificada: `_TEMPLATE.md`, reescrito en ese mismo commit, advierte explícitamente en su tabla "Lo Que NO Se Escribe en un Skill" que esos dos bloques van a `mentorship-modes.md`. La plantilla nueva previene exactamente el error que el commit dejó sin corregir.

### 🟠 N-5 · Pérdida de evidencia de respaldo en 3 knowledge docs

No se perdió ninguna regla, pero sí el dato duro que la justificaba — exactamente lo que `CONTEXT-ENGINEER-3.2` prohíbe recortar (dato técnico y caso borde, no conector):

| Documento | Qué se perdió |
|---|---|
| `debt-management.md` | El caso real `DEBT-005` (disparador cumplido que nadie promovió) quedó como afirmación abstracta sin ID ni evidencia. Y en `DEBT-MANAGEMENT-10`, las cifras de `DEBT-032` — "**dos** defectos propagados durante **meses** a través de **catorce** implementaciones" — se degradaron a "defectos por copia a través de las implementaciones siguientes". |
| `ledger-pattern.md` | La lista de 7 migraciones (`0010, 0014, 0015, 0016, 0018, 0019, 0020`) que ya habían corregido el hueco de candados por su cuenta. Verificado: `grep` de esos números devuelve **cero** en el archivo actual. |
| `commenting-policy.md` | El puntero a `governance.md` desapareció por completo del documento. Es el único de sus tres archivos hermanos que lo perdió — `ponytail.md` y `memory-policy.md` sí lo conservan. |

### 🟠 N-6 · `972bab5` es el commit masivo que `CLAUDE.md` prohíbe

`CLAUDE.md` §1 fija: *"agrupa los cambios pendientes en commits separados por tipo. Nunca un commit masivo de todo."* El plan del propio trabajo declaraba en su paso 19 "Commits agrupados por tipo" y afirma haber hecho "13 commits sobre `2df8665`".

**Realidad verificada:** entre `2df8665` y `972bab5` solo existen 3 commits, y `972bab5` mezcla en un solo objeto de 58 archivos: un hook nuevo (`feat`), la configuración que lo cablea (`chore`), 22 knowledge docs reescritos (`refactor`), 13 skills (`refactor`), y 4 documentos de `docs/` (`docs`). No es rastreable ni reversible por partes.

### 🟠 N-7 · `DEBT-046` no sigue la plantilla de `docs/debt/`

Sus 7 vecinos (`DEBT-037`, `040`–`045`) usan uniformemente: título `# DEBT-NNN · …`, tabla de campos, y secciones `## El hecho`, `## Cómo se verifica que quedó pagada`, `## Relacionadas`. `DEBT-046` usa una lista plana de viñetas sin tabla ni esas secciones. El contenido es correcto; la estructura no.

### 🟡 N-8 · El plan `refactor-masiva-agents.md` quedó desincronizado de su propia ejecución

- Su **Fase 0, paso 4** ordena *"Borrar `base.md` una vez sin referencias entrantes"*. `base.md` **sigue existiendo** (819 bytes) y el hook lo **whitelistea deliberadamente** (`NO_SLUG_REQUIRED` + `STUB_FILES`, con comentario explicando que sobrevive para no romper enlaces de planes cerrados). La decisión cambió durante la ejecución y **el plan nunca se actualizó**: quien lo retome intentará borrarlo y romperá 4 referencias históricas.
- Su criterio de verificación `grep -rn "base\.md" . → cero` **no se cumple** y ya no puede cumplirse por diseño.
- Su **Fase 4, paso 18** (*"Actualizar `CLAUDE.md` §2 con el mapa nuevo"*) no se ejecutó: los 5 knowledge docs nuevos no aparecen en `CLAUDE.md`. Impacto bajo — sí están en el índice canónico `.agents/knowledge/README.md`, que es a donde `CLAUDE.md` remite.
- `.agents/state/tech-lead/PROGRESS.md:411` sigue citando `base.md §7.4`, una sección que no existe en ningún archivo del repo.

### 🟡 N-9 · Wikilinks incumplidos en documentos nuevos que sí saben el formato

`ui-verification-model.md` cita ADR en texto plano en 7 puntos (L3, L58, L72, L80 ×2, L90, L104) mientras usa `[[docs/adr/ADR-0117.md]]` correctamente en su propia tabla de Lecturas Obligatorias. `cross-platform-rust.md` tiene 1 caso (L33, ADR-0016). Viola la regla explícita de `.claude/rules/agents-docs-format.md`.

### 🟠 N-10 · `CONTEXT-DEBT-008` subestima su propio alcance

La ficha registra la ruta rota `.claude/skills/tech-lead.md` (el archivo real es `.claude/skills/tech-lead/SKILL.md`) solo para `qa-engineer`. Verificado por grep: el mismo path roto está también en `rust-engineer/SKILL.md`, `bridge-engineer/SKILL.md` y `quant-engineer/SKILL.md`. **Son 4 archivos, no 1** — y en todos es doble violación: ruta inexistente y sin wikilink.

### 🟠 N-11 · `brief.md` es huérfano del índice de knowledge

`brief.md` existe, está bien formateado (`Slug ID: BRIEF`, `Tamaño` correcto) y es citado activamente por wikilink desde `tech-lead/SKILL.md` y `context-auditor/SKILL.md` — pero **no aparece en `.agents/knowledge/README.md`**, el índice que el propio sistema declara como la única puerta de entrada al conocimiento. El chequeo inverso está limpio: ningún documento indexado es inexistente.

(`base.md` tampoco está indexado, pero es un stub autodeclarado de compatibilidad — omisión defendible.)

### 🟠 N-12 · Dos referencias rotas más en documentos vivos

- `.agents/knowledge/commenting-policy.md:140` → `[[docs/execution/STORY-009.md]]`. El archivo real es `docs/execution/STORY-009-cli-app.md`.
- `.agents/knowledge/claude-code-mechanisms.md:252` cita `.claude/rules/comments.md` como conclusión de un análisis. Ese archivo no existe y no hay evidencia de que se haya creado — o es una propuesta nunca ejecutada, o quedó obsoleta.

### 🟡 N-13 · Dos imprecisiones menores heredadas del refactor

- **`partner/SKILL.md`:** `PARTNER-3.1` (la guarda de arquitectura hexagonal, que corrigió bien el residuo `contracts.py` de la era Python) cita como filtro `PARTNER-FV-1`, que en realidad cubre otra regla (`PARTNER-1.3`, prohibición de escribir código). Ningún ítem del Filtro verifica realmente esa guarda.
- **`refactoring-engineer/SKILL.md`:** el checklist propio del Modo Revisión (*"¿preserva el comportamiento funcional?, ¿reduce realmente la deuda (archivo <400 líneas, sin ciclos)?, ¿la suite sigue verde?"*) y el comportamiento de Modo Docente se eliminaron delegando a `mentorship-modes.md`, que tampoco los contiene. La sustancia sobrevive disuelta en `REFACTORING-ENGINEER-3.1/3.3`, pero el vínculo explícito desapareció. Es consecuencia del diseño nuevo de `_TEMPLATE.md`, no un accidente — pero deja hueco real, mismo patrón que N-2 aunque de menor gravedad.

---

## Parte 2 — Lo que se verificó y está BIEN

Vale registrarlo para que nadie lo vuelva a auditar:

- **Archivado del sistema de diseño: fiel y superset.** `ui-old/_docs-archive/DESIGN.md` (1107 líneas) contiene todo el viejo (1074) más dos secciones añadidas antes de archivar. Cero líneas borradas. Igual para los dos skills archivados. **No hubo pérdida en el traslado.**
- **`DESIGN.md` nuevo: reset legítimo.** 11 secciones numeradas sin huecos, autosuficiente, con su cadena de autoridad ADR declarada. Todas las referencias `§N` que le hacen `ui-designer` y `flutter-engineer` existen y corresponden temáticamente.
- **Los 5 knowledge docs nuevos: formato impecable.** `Slug ID` correcto, campo `Tamaño` verificado con `wc -m` real y coincidente **exacto** en los 5, orden de bloques correcto, cero IDs de 3 segmentos, los 5 indexados en `README.md` con disparador.
- **11 de los 13 knowledge modificados: sin pérdida alguna.** Los recortes de fecha y atribución son cumplimiento de la regla "sin lenguaje de bitácora", no pérdida.
- **`ponytail.md` y `memory-policy.md`: quedaron mejor, sin pérdida normativa.**
- **Los 14 skills declaran `governance.md`** — el gate de arranque roto que motivó todo el refactor está efectivamente reparado.
- **El "misterio ponytail" no es pérdida.** El diffstat `SKILL.md | 0` alarmaba. Rastreada la cadena completa: el skill real (206 líneas) se tradujo íntegro a `.agents/knowledge/ponytail.md` el 2026-07-09 (commit `9a8312e`), que además creó `.agents/skills/ponytail/SKILL.md` **ya vacío**. Estuvo huérfano y en 0 bytes un mes. `972bab5` solo borró el cadáver. Cero referencias huérfanas hoy. **El mensaje del commit es impreciso** ("integrado en ponytail.md" sugiere que la integración ocurrió ahí), pero no hubo pérdida.
- **Los 14 stubs `.claude/skills/`: correspondencia 1:1 perfecta** con `.agents/skills/`, en ambas direcciones, cada uno apuntando al destino correcto. Sin desajustes.
- **`d6f6b8b` y `6d1055e`: limpios.** Las 8 fichas `CONTEXT-DEBT` son consistentes entre índice y contenido, numeración sin huecos, formato uniforme. El skill `context-auditor` cumple el formato canónico completo (`Tamaño` verificado exacto). La línea añadida a `docs/DEBT.md` es un campo de Alcance que delimita los dos registros — no contaminación. Los 3 micro-cambios de `6d1055e` son correctos y sin contradicción interna.

---

## Parte 3 — Censo estructural de skills (la foto dura de hoy)

| Skill | Slug ID | Tamaño | Bloque `-FV` | Bitácora | Chars | Estado |
|---|---|---|---|---|---|---|
| `context-auditor` | sí | sí | sí | 0 | 15 240 | ✅ canon |
| `context-engineer` | sí | sí | sí | 0 | 15 832 | ✅ canon |
| `flutter-engineer` | sí | sí | sí | 0 | 12 479 | ✅ canon |
| `general-counsel` | sí | sí | sí | 0 | 9 566 | ✅ canon |
| `partner` | sí | sí | sí | 0 | 6 154 | ✅ canon |
| `refactoring-engineer` | sí | sí | sí | 0 | 7 548 | ✅ canon |
| `ui-designer` | sí | sí | sí | 0 | 11 586 | ✅ canon |
| `architect` | sí | sí | **NO** | 5 | 25 833 | ⚠️ a medias (*staged*) |
| `qa-engineer` | sí | sí | **NO** | 3 | 20 067 | ⚠️ a medias |
| `bridge-engineer` | **NO** | **NO** | **NO** | 2 | 11 929 | ❌ legacy |
| `quant-engineer` | **NO** | **NO** | **NO** | 3 | 11 934 | ❌ legacy |
| `rust-engineer` | **NO** | **NO** | **NO** | 13 | 22 470 | ❌ legacy |
| `social-strategist` | **NO** | **NO** | **NO** | 0 | 15 934 | ❌ legacy |
| `tech-lead` | **NO** | **NO** | **NO** | **36** | **85 489** | ❌ legacy, el peor |

`flutter-engineer` y `ui-designer` **no** se canonizaron en `972bab5` sino en `69d0fff` — el reset de UI cerró de rebote 2 de los 9 skills que el refactor masivo dejó pendientes.

---

## Parte 3-bis — Quién arregla qué

Regla de dominio, verificada en cada skill:

- **`context-engineer`** edita `.agents/skills/` y `.agents/knowledge/` — formato, redacción y restauración de contenido que ya existía.
- **`architect`** edita `docs/` — ADR, SAD, features, módulos, `ROADMAP`, `docs/debt/`.
- **`tech-lead`** mantiene `docs/TEST.md` (paso 4 de su Barrido de Cierre Documental).
- **`context-auditor`** nunca corrige: solo audita y reporta. Se usa para verificar, no para arreglar.
- **`qa-engineer`** valida que el runbook reparado corra de verdad.

| Ítem | Dueño | Nota |
|---|---|---|
| D-1 → ADR o ficha de deuda | `architect` | Bloqueado por D-1 |
| N-2 · Modos de `general-counsel` | `context-engineer` | Si eliges D-2 (b), entra también `architect` |
| N-3 · refs `ui/` en `docs/` (ROADMAP, ADR-0117/0138/0152, features, `DEBT.md`) | `architect` | |
| N-3 · refs `ui/` en `.agents/skills/` y `.agents/knowledge/` | `context-engineer` | |
| N-3 · `docs/TEST.md` | `tech-lead` | Bloqueado por D-3; `qa-engineer` valida después |
| N-4 · punteros temáticos en `bridge`/`qa`/`quant`/`rust` | `context-engineer` | |
| N-5 · evidencia perdida en 3 knowledge docs | `context-engineer` | |
| N-7 · reformatear `DEBT-046` | `architect` | Vive en `docs/debt/` |
| N-8 · sincronizar el plan viejo con la realidad | `context-engineer` | Bloqueado por D-4 |
| N-9 · wikilinks faltantes | `context-engineer` | |
| N-10 · ruta rota en 4 skills | `context-engineer` | |
| N-11 · indexar `brief.md` | `context-engineer` | |
| N-12 · dos referencias rotas | `context-engineer` | |
| N-13 · `PARTNER-FV` y hueco de `refactoring-engineer` | `context-engineer` | |
| Canonizar los 5 skills legacy | `context-engineer` | Bloqueado por D-5 solo para `tech-lead` |
| `CONTEXT-DEBT-001` · corregir cifras | `context-engineer` | |
| `CONTEXT-DEBT-002` · contradicción de memoria | `context-engineer` | |
| `CONTEXT-DEBT-003` · restaurar FFI, revertir `7.3`, reponer ADRs | `context-engineer` | La ficha ya especifica qué restaurar |
| `CONTEXT-DEBT-004` · wikilinks a los 5 knowledge nuevos | `context-engineer` | |
| `CONTEXT-DEBT-006` · IDs colisionados | `context-engineer` | |
| `CONTEXT-DEBT-007` · numeración de `tech-lead` | `context-auditor` audita → `context-engineer` corrige | La ficha nombra al auditor para el diagnóstico |
| `CONTEXT-DEBT-008` · corregir alcance + canonizar `qa-engineer` | `context-engineer` | |
| Abrir `CONTEXT-DEBT-009` (refs `ui/`) | quien lo pague primero | El registro admite que cualquier rol abra ficha |
| N-6 · commit masivo | — | No se arregla retroactivamente; es regla para el futuro |

**Lectura práctica:** el 80 % de la lista es un solo despacho a `context-engineer`, que puede hacerse en lotes. `architect` tiene 3 ítems. `tech-lead` uno. Nada exige coordinar a los tres a la vez.

---

## Parte 4 — Remediación priorizada

### Prioridad 0 — Decisiones del propietario

Las cinco decisiones (D-1 a D-5) están arriba, en su propia sección. **D-1 bloquea al Architect**; el resto tiene ruta por defecto si no respondes.

### Prioridad 1 — Rompe operación hoy

2. **Reparar N-2 (`general-counsel` sin Modos operables).** Dos salidas: restaurar en el skill la definición propia del rol legal, o ampliar el Alcance declarado de `mentorship-modes.md` para incluir roles no-ingeniero. La primera es más fiel a lo que se perdió.
3. **Reparar el runbook de QA (N-3).** `docs/TEST.md` (L35, L37, L43, L295): decidir si el comando apunta a `ui-old/` mientras se reconstruye `ui/`, o marcar el bloque como suspendido. No dejarlo dando una orden que falla.
4. **Actualizar `CONTEXT-DEBT-003`:** subir a 🔴 Alta, reescribir "Impacto actual" (ya no es hipotético, está en `HEAD`) y ejecutar su plan: restaurar la verificación FFI en `UI-DESIGNER-5.1`, revertir `UI-DESIGNER-7.3` a prohibición absoluta, reponer las citas de `ADR-0131`, `ADR-0116` y `ADR-0038`.
5. **Corregir las cifras de `CONTEXT-DEBT-001`** (5 skills / 10 violaciones, no 6 / 12) **y el alcance de `CONTEXT-DEBT-008`** (N-10: 4 archivos con la ruta rota, no 1).

### Prioridad 2 — Deuda que crece si se ignora

6. **Corregir los punteros temáticos (N-4)** en `bridge`, `qa`, `quant` y `rust`: Protocolo de Lecciones → `mentorship-modes.md`, Política de Comentarios → `commenting-policy.md`. Está identificado por escrito desde hace tres semanas.
7. **Abrir `CONTEXT-DEBT-009`** para las 44 referencias rotas a `ui/` (N-3), absorbiendo `CONTEXT-DEBT-005` (subconjunto). Barrido único: separar referencias vivas (se corrigen) de historial (se deja).
8. **Canonizar los 5 skills legacy**, en el orden que el plan original ya fijó: `social-strategist` → `bridge` → `quant` → `rust` → `tech-lead` al final. Cada uno desbloquea su archivo del hook.
9. **Restaurar la evidencia perdida (N-5):** el caso `DEBT-005` y las cifras de `DEBT-032` en `debt-management.md`, la lista de migraciones en `ledger-pattern.md`, el puntero a `governance.md` en `commenting-policy.md`.
10. **Pagar `CONTEXT-DEBT-004`:** reemplazar los bloques duplicados inline por wikilinks a los 5 docs canónicos. Hoy nadie los lee.

### Prioridad 3 — Higiene

11. **Indexar `brief.md`** en `.agents/knowledge/README.md` (N-11).
12. **Arreglar las 2 referencias rotas de N-12** (`STORY-009` y `.claude/rules/comments.md`).
13. **Sincronizar el plan `refactor-masiva-agents.md`** (N-8) con lo que realmente pasó: `base.md` se conserva por decisión, y su criterio de verificación cambia.
14. **Remapear `PROGRESS.md:411`** (`base.md §7.4` → `doc-first.md`).
15. **Reformatear `DEBT-046`** a la plantilla de `docs/debt/` (N-7).
16. **Wikilinks faltantes** en `ui-verification-model.md` (7) y `cross-platform-rust.md` (1) — N-9.
17. **Corregir N-13:** el `PARTNER-FV` que no cubre `PARTNER-3.1`, y el hueco de Modo Revisión/Docente en `refactoring-engineer`.
18. **`CONTEXT-DEBT-006`, `-007`** según su propio plan.

### Nota sobre agrupación de commits

Al pagar esto, **no repetir el error de N-6**: `feat` (fichas nuevas), `docs` (correcciones de contenido) y `refactor` (canonización de skills) van en commits separados, como manda `CLAUDE.md` §1.

---

## Verificación de cierre

- `python3 .agents/hooks/validate-agent-docs.py --audit` → **cero violaciones duras** (hoy: 10).
- `grep -rnoE '\bui/[a-zA-Z0-9_./-]+' <docs vivos> | grep -v ui-old` → cero, o cada superviviente justificado.
- Cada uno de los 5 knowledge nuevos citado por wikilink desde al menos un skill.
- `docs/TEST.md`: todo comando del runbook ejecutable, o marcado como suspendido.
- Las 8 fichas `CONTEXT-DEBT` reflejan el estado real del árbol.
