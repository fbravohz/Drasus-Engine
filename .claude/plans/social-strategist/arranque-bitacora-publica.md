# Plan de Acción — Social Strategist · Arranque de la Bitácora Pública

> **Skill:** `social-strategist` · **Fecha de captura:** 2026-06-21 · **Sesión:** primer escaneo completo
> **Ruta destino preferida del usuario:** `./.claude/plans/social-strategist/arranque-bitacora-publica.md` (reubicar este archivo ahí al ejecutar; el modo plan obligó a usar el nombre autogenerado).
> **Estado:** aprobado el plan de acción por el usuario; pendiente de ejecutar. El usuario cambia de máquina y necesita este plan para continuar sin el chat.

---

## Context (por qué existe este plan)

El proyecto tiene **70 commits** y nunca se ha generado **nada** de contenido social. No existía el archivo de seguimiento del skill (`.claude/state/social-strategist/PROGRESS.md`) — se creó en esta sesión. Hay **12 cierres publicables sin tocar** y **0 Pulsos** emitidos.

Además, `CONTENT-STRATEGY.md` (la fuente de verdad del skill) se redactó cuando había 14 commits y hasta STORY-007. El desarrollo ya está **muy por delante**: STORY-008, 009, 010, un bug multiplataforma real (BUG-013/ADR-0134) y dos historias en curso (STORY-014 Nautilus, STORY-015 panel Flutter). Hay más historia de la que el documento contempla.

El usuario aprobó el plan de 5 acciones que sigue. Este archivo lo congela para retomarlo en otra máquina.

---

## Estado verificado del proyecto (escaneo 2026-06-21)

### Inventario de contenido publicable, por Pilar

**🏗️ Pilar G — Building in Public (insignia, LISTO para Episodio):** 10 historias cerradas. Es el Caso de Estudio #0 / video de lanzamiento (`CONTENT-STRATEGY.md` §7), sobre-maduro.

| Historia | Commit | Gancho | Capa visual fuerte |
|---|---|---|---|
| STORY-001 esqueleto workspace | 6f5ad59 | "Esqueleto completo de un motor de dinero real, antes de la lógica." | `cargo test` verde / 8 crates |
| STORY-002 SQLite WAL | 5cc4b29 | "La primera decisión fue una base de datos. Soberanía día 1." | esquema 25 campos |
| STORY-003 reloj determinista | bd7dba1 | "Reloj propio. Sin esto ningún backtest es confiable." | mismo input→mismo output |
| STORY-004 audit-log hash chain | d4919fd | "Libro contable que ni nosotros alteramos." | **animación cadena de hash (oro)** |
| STORY-005 recuperación kill -9 | c74fff6 | "Si el motor se cae a mitad de cálculo: no pasa nada." | terminal `kill -9` + recuperación |
| STORY-007 telemetría | c03ec68 | "El sistema nervioso del motor en tiempo real." | buffer / heartbeat |
| STORY-009 CLI `drasus` | 5274ee7 | "El motor arranca como programa real; cierra el examen final de la Fundación." | CLI Clap |
| STORY-010 MCP Gateway agéntico | 9bc8412 | "Copiloto de IA que nunca depende de un servidor externo." | servidor stdio |

**🔄 Pilar E — Devlog: decisiones que cambiamos (LISTO, el más honesto):**
- 3 erradicaciones documentadas: no a PyTorch (ADR-0112), no a PySR (ADR-0113), no a Ollama (ADR-0115).
- STORY-008 (65ecf23) purga de residuos Python del aislamiento de workers.
- TASK-011 (f13c70a) cambio de regla de arquitectura (tabla única por feature).
- **🆕 BUG-013 + ADR-0134 multiplataforma (2e343a5):** bug real que solo aparecía en Windows/macOS (API de Linux sin protección), cazado y corregido en público. Short excelente — "building in public" honesto no esconde bugs.

**🛡️ Pilar D — Infraestructura soberana (PARCIAL):** STORY-002 (SQLite local) + STORY-010 (LLM local). 2 entradas → Pulsos sí, Episodio aún no. Madura en EPIC-1.

**📣 Pilar F — Mitos del trading algorítmico (siempre disponible):** no depende del código. Combustible para Shorts.

### En el horno (NO publicar como cerrado todavía)
- **STORY-014** smoke test NautilusTrader (en curso) → al cerrar **paga la apuesta #1** (ADR-0107: motor institucional como Rust nativo, sin Python). Gancho de los más fuertes del catálogo.
- **STORY-015** primer panel operativo Flutter (en curso) → desbloquea **B-roll de UI real (Capa 1)**, que hoy no existe.

### Entorno de producción (`command -v`)
| Herramienta | Estado |
|---|---|
| node / npm / python3 | ✅ OK |
| manim, ffmpeg, whisper | ❌ FALTA |
| silicon, carbon-now-cli (captura de código) | ❌ FALTA |

---

## Hallazgo: capturas de pantalla (pregunta del usuario)

**De código (lo que el usuario pidió): SÍ, automatizable vía CLI que yo ejecuto por terminal.**

| Herramienta | Qué es | Instalación |
|---|---|---|
| `silicon` | CLI Rust: `.rs` → PNG con resaltado. "Carbon desde terminal". | `cargo install silicon` (puede requerir libs de fuentes vía `dnf`) |
| `carbon-now-cli` | "Carbon" oficial vía Node, mismo resultado. | `npm i -g carbon-now-cli` |

Ya está contemplado en `CONTENT-STRATEGY.md` §6.2 ("Snippets de código bonitos: Silicon/Carbon"). Es el equivalente CLI de `easy-codesnap`/`CodeSnap`/`screendown` — apunto a un archivo real, genero el PNG, el usuario lo sube. Sin trabajo manual.

**Límites honestos:** no "veo" la pantalla en vivo; no capturo la UI Flutter ni `cargo test` en movimiento (eso es grabación OBS, la hace el usuario). Si el usuario genera un PNG, sí puedo leerlo para verificarlo.

**Gap del skill:** la tabla del Pipeline D en `.claude/skills/social-strategist/SKILL.md` **no lista** `silicon`/`carbon`, aunque la estrategia sí. Proponer agregarlos como capacidad de primera clase.

---

## Plan de acción aprobado (5 opciones, ejecutables por bloques)

> Procesar por bloques, no todo de golpe. Recomendación de orden: **4 → 1 → 3 → 2 → 5** (instalar captura de código primero, así los Pulsos salen ilustrados desde el primer post).

1. **Pulso (ES+EN) de las 10 historias del Pilar G.** Arranca la bitácora pública. Genera el par `-es.md` / `-en.md` por cierre en `.claude/documents/social-strategist/`. Bajo costo, alto valor de track record. Actualizar `PROGRESS.md` marcando cada cierre como procesado + `último_commit_procesado`.

2. **Guion + assets del Episodio "Construimos un motor de trading. Cero estrategia todavía"** (Caso de Estudio #0, Pilar G, Nivel 0 sin cara). Seguir plantilla §7.1: guion minuto a minuto + assets Manim (árbol de decisiones descartadas, animación hash chain) + diagrama C4 animado + checklist de grabación OBS. Guardar en `.claude/documents/social-strategist/episodios/<slug>/`. Marcar STORYs/ADRs cubiertos en `PROGRESS.md` (regla de no duplicación §10).

3. **Short del bug multiplataforma** (Pilar E): "Encontramos un bug que solo existía en Windows. Así lo cazamos en público." (BUG-013/ADR-0134). Formato 30-60s, ES+EN. Rápido y honesto.

4. **Configurar entorno de captura de código:** instalar `silicon` (`cargo install silicon`) y/o `carbon-now-cli` (`npm i -g carbon-now-cli`). **Pedir confirmación antes de instalar** (regla del skill: cero acciones de sistema sin confirmación). Re-chequear con `command -v` tras instalar. Luego **agregar silicon/carbon a la tabla del Pipeline D** en `SKILL.md` (edición quirúrgica con `Edit`, bloque pequeño).

5. **Contenido de postura (Pilar F):** Shorts confrontacionales contra mitos del trading algorítmico hispano, con argumentos técnicos anclados a lo que Drasus resuelve. ES+EN. Independiente del calendario de desarrollo.

---

## Archivos relevantes (para retomar en la nueva máquina)

- `CONTENT-STRATEGY.md` (raíz) — fuente de verdad: Pilares, Capas, mapa Épica→Pilar (§1.4), idioma (§1.5), guion Caso de Estudio #0 (§7), formatos por red (§9).
- `.claude/skills/social-strategist/SKILL.md` — motor de ejecución: pipelines A/B/C/D, plantillas de archivos, Pipeline D (tabla de herramientas a ampliar).
- `.claude/state/social-strategist/PROGRESS.md` — memoria del skill (ya creada esta sesión; refleja el escaneo y los 12 cierres pendientes).
- `.claude/skills/base/SKILL.md` — governance con supremacía; leer al iniciar cualquier sesión del skill.
- `docs/execution/` — Órdenes de Trabajo (estado/sellos de cada STORY/TASK/BUG).

---

## Verificación / cómo retomar

1. Nueva sesión: `/social-strategist` → el skill lee `base/SKILL.md`, `CONTENT-STRATEGY.md` y este `PROGRESS.md` (ya poblado).
2. Confirmar entorno: `for t in manim ffmpeg whisper node npm python3 silicon carbon-now; do command -v $t || echo "FALTA $t"; done`.
3. Elegir opción(es) del menú de 5. Orden sugerido arriba.
4. Cada Pulso queda en `.claude/documents/social-strategist/social-strategy-YYYY-MM-DD-{es,en}.md`; cada Episodio en `episodios/<slug>/`.
5. Tras cada bloque, actualizar `PROGRESS.md` (commit procesado + cierres marcados) para no duplicar.
