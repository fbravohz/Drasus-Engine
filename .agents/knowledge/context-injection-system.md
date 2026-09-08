# Sistema de Inyección de Contexto

- Mapa completo de cómo llega cualquier lectura -- obligatoria o recomendada -- al contexto de un agente sin depender de que decida leerla por su cuenta: los 4 niveles, sus hooks, su estado compartido y sus huecos conocidos. Referencia obligatoria antes de declarar una lectura SIEMPRE o forzosa nueva.
- SLUG-ID: `CONTEXT-INJECTION-SYSTEM`
- Tamaño: `8845 caracteres (~2211 tokens aprox., ratio 4:1)`
- Lecturas:
  - [[.agents/knowledge/governance.md]] -> SIEMPRE -> Todos

## CONTEXT-INJECTION-SYSTEM-1: Los 4 Niveles

- **Nivel 1 — SIEMPRE universal:** dos documentos (`governance.md`, `hard-prohibitions.md`) que llegan completos al contexto de toda sesión, sin excepción de skill.
- **Nivel 2 — SIEMPRE propio de un skill:** lecturas que un skill concreto declara obligatorias, pero que Nivel 1 no puede anticipar porque no sabe qué skill se va a invocar.
- **Nivel 3 — bajo demanda por palabra clave:** manifiesto de wikilinks (nunca contenido completo) que aparece cuando el prompt del humano matchea contra el índice del corpus.
- **Nivel 4 — lectura forzosa mid-flujo:** gate que bloquea una acción concreta (`Write`/`Edit`/`Bash`) de un skill ya en marcha si una lectura declarada como prerrequisito de ese punto del flujo no ocurrió en la sesión.
- Ningún nivel es semántico -- los 4 son lexicales/estructurales (coincidencia de ruta, palabra clave o existencia de una marca). No existe ningún hook atado al razonamiento interno del agente, solo a `SessionStart`, al envío de un prompt (`UserPromptSubmit`) y a llamadas de herramienta (`PreToolUse`/`PostToolUse`) -- verificado contra la documentación oficial de hooks, no asumido.

## CONTEXT-INJECTION-SYSTEM-2: Nivel 1 — `session-start-siempre.py`

- **Hook:** `SessionStart`, matcher `startup|resume|clear|compact|fork` -- las 5 razones de arranque documentadas, todas cubiertas.
- **Mecanismo:** imprime por stdout el contenido completo de `governance.md` + `hard-prohibitions.md`, cableado a mano (`SIEMPRE_UNIVERSAL`, no leído de ninguna tabla) -- llega al contexto del modelo antes de procesar el primer turno.
- **Gap conocido:** no cubre lecturas SIEMPRE propias de un skill -- no sabe qué skill se va a invocar. Ese hueco lo cierra Nivel 2.

## CONTEXT-INJECTION-SYSTEM-3: Nivel 2 — `gate-skill-siempre.py` + `track-siempre-reads.py`

- **Hook de gate:** `PreToolUse` sobre el tool `Skill`. Antes de dejar invocar un skill, parsea su propio `SKILL.md` (bloque `Lecturas:`, formato ligero `- [[ruta]] -> SIEMPRE -> <alcance>` o legado de tabla) y filtra las filas con disparador `SIEMPRE` cuyo alcance NO sea "Todos" -- esas ya las cubrió Nivel 1 o no aplican a este skill.
- **Hook de marca:** `PostToolUse` sobre `Read` (`track-siempre-reads.py`) -- cada vez que se lee cualquier `.md` del repo, marca la lectura en el estado compartido (Sección 6). Sin esa marca, el gate no tiene cómo saber que la lectura ya ocurrió.
- **Denegación:** si falta alguna, deniega con la lista completa de wikilinks faltantes en un solo mensaje -- no uno a la vez -- el agente lee, reintenta, pasa.
- **El valor real del campo `alcance`:** el código nunca compara el texto contra el nombre del skill -- solo mira si contiene "todos". Cualquier otro valor dispara el gate igual. Repetir el nombre del propio skill en su propio `SKILL.md` (ej. `-> Tech Lead` dentro de [[.agents/skills/tech-lead/SKILL.md]]) es válido pero 100% redundante -- la información real es un booleano (Todos / no-Todos), nunca el texto exacto.

## CONTEXT-INJECTION-SYSTEM-4: Nivel 3 — `inject-context.py`

- **Hook:** `UserPromptSubmit`, sin `matcher` (el evento no lo soporta).
- **Índice:** SQLite FTS5 sobre `.agents/knowledge/*.md` + `.agents/skills/*/SKILL.md`, partido por Sección (`## <DOC>-N: Título` en formato ligero) -- un documento legado sin ese patrón cae completo en un solo segmento. Se reconstruye solo cuando el mtime de algún archivo fuente cambió.
- **Ranking:** `bm25()`, ponderado por el campo `Tags:` (cabecera o por Sección) repitiendo sus palabras varias veces en el texto indexado -- boost de frecuencia en una sola columna. Se descartó a propósito ponderar con un argumento de peso por columna separada para `tags`: la normalización de longitud de BM25 es una estadística compartida por todo el corpus, y ese enfoque resultó no monótono en pruebas reales (subir el peso podía encoger la ventaja del documento etiquetado en vez de crecerla).
- **Entrega:** manifiesto de wikilinks citando ruta + Sección exacta que matcheó -- nunca contenido completo por defecto. El agente decide si los lee.
- **Switch de troubleshooting:** `FORCE_FULL_CONTENT` (constante de módulo, `False` por defecto) -- en `True`, pega el texto real (sin el boost de tags) de cada Sección que matcheó, con un tope de caracteres combinado como red de seguridad. Pensado para si en la práctica el agente no abre los wikilinks lo bastante.

## CONTEXT-INJECTION-SYSTEM-5: Nivel 4 — `enforce-essential-reads.py`

- **Hook:** `PreToolUse` sobre `Write|Edit|Bash`.
- **Por qué existe:** Nivel 2 solo gatilla al invocar el skill, una vez, al inicio. No cubre un punto intermedio del propio flujo del skill ("antes de escribir la Orden, hace falta haber leído su formato") sin que el humano escriba un prompt nuevo en ese instante -- y ese punto es invisible por diseño: no hay hook atado al razonamiento interno, solo a la llamada de herramienta que ese punto produce.
- **Convención declarativa** (bullet en cualquier `SKILL.md` o knowledge doc): `` - Lectura forzosa: [[ruta]] antes de `Tool(patrón)` `` -- mismo `Tool(especificador)` que usa `.claude/settings.json`, comparado con `fnmatch` contra `file_path` (Write/Edit) o `command` (Bash).
- **Estado:** infraestructura lista, cero declaraciones reales todavía en ningún skill -- ver Sección 8.

## CONTEXT-INJECTION-SYSTEM-6: Estado Compartido — `.agents/state/.session-reads/`

- **Esquema:** `_session_reads.py` (helper, no es hook) -- una marca es un archivo vacío en `<STATE_DIR>/<session_id>/<slug de la ruta>`. La existencia del archivo es la única señal, sin contenido que parsear.
- **Consumidores:** `gate-skill-siempre.py` y `enforce-essential-reads.py` leen; `track-siempre-reads.py` escribe; `reset-session-reads.py` borra.
- **Ciclo de vida:** `reset-session-reads.py` (`SessionStart`, matcher `compact|resume|clear`) borra el directorio completo de la sesión actual en esas 3 transiciones. Decisión de diseño explícita: `session_id` sobrevive a `/compact` (verificado comparando la ruta del transcript pre-compactación contra la del hook `SessionStart:compact` disparado después) -- pero eso no basta para confiar en la marca, porque la garantía real es "está en el contexto usable ahora", no "se leyó alguna vez". Se borra igual, sobreviva o no el id.
- **Barrido de huérfanos:** directorios de sesión sin tocarse hace más de `SWEEP_DAYS` días se borran solos -- se invoca desde `session-start-siempre.py` (las 5 razones de arranque) y desde `reset-session-reads.py`, así que corre en cada inicio de sesión, no solo en compact/resume/clear.
- **No versionado:** `.gitignore` excluye este directorio -- estado efímero, nunca conocimiento curado.

## CONTEXT-INJECTION-SYSTEM-7: Protección del Estado — `permissions.deny`

- **Regla real:** deny de `Read` + `Edit` sobre `.agents/state/.session-reads/**` en `.claude/settings.json`. Verificado contra la documentación oficial de permisos: Claude Code solo consulta reglas de ruta contra `Edit`/`Read` -- una regla de `Write` sobre una ruta se acepta pero nunca se aplica, es un error silencioso si se escribe así.
- **Mitigación de mejor esfuerzo:** reglas `Bash` que buscan el texto del comando de borrado/movimiento junto con la ruta del estado -- coincidencia de texto, no infalible. Read/Edit no cubren subprocesos arbitrarios (ej. un script que abre el archivo directo) -- blindaje total requeriría el sandbox a nivel SO, fuera de alcance.
- **Efecto colateral real, confirmado en la práctica:** estas mismas reglas también bloquean un comando de limpieza legítimo si su texto combina ambos patrones -- usar una vía que no junte "rm"/"mv" con la ruta del estado en el mismo texto (ej. `find ruta -delete`).

## CONTEXT-INJECTION-SYSTEM-8: Huecos Conocidos

- **Nivel 4 sin adopción:** ningún skill real declara todavía un bullet "Lectura forzosa" -- construir el mecanismo no crea las reglas, eso es autoría de contenido por skill.
- **Hooks sin protección propia:** el `deny` no cubre los scripts de `.agents/hooks/*.py` -- editarlos directamente neutraliza cualquier gate. Señalado, sin extenderlo unilateralmente (bloquearía también el mantenimiento legítimo de los hooks).
- **Nivel 3 no sabe qué skill está activo:** el índice no filtra por rol -- dos sesiones con el mismo prompt reciben el mismo manifiesto sin importar qué skill esté corriendo cada una.
