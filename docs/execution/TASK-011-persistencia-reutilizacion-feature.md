# TASK-011 · Documentar la regla de persistencia por módulo dueño (enmienda ADR-0003)

| Campo | Valor |
|---|---|
| **ID** | TASK-011 |
| **Título** | Enmienda ADR-0003 — regla de tabla única por feature y TTRs de integración vs construcción |
| **Tipo** | Task (escalamiento al Architect — sin código) |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | Completado |
| **Responsable** | Architect (Opus) |
| **Creada** | 2026-06-20 |
| **Completada** | 2026-06-20 |

## 1. Origen del escalamiento

**Quién detectó el vacío:** Tech-Lead, durante la revisión de la spec de `worker-isolation-orchestrator` y el análisis de las 4 features nuevas de datos fundamentales (`fundamental-event-store`, `event-impact-scorer`, `asset-exposure-map`, `fundamental-indicator-projector`).

**Cuándo:** sesión 2026-06-20, al responder la pregunta del usuario: *"¿se supone que también están definidas sus tablas/columnas que usarán según el módulo al que pertenecen? ¿cómo vamos a manejar que una feature se use en diferentes módulos?"*

## 2. El vacío de diseño

La regla de persistencia en la reutilización de features existe **implícitamente** en la intersección de dos ADRs:

- **ADR-0003** (cada módulo dueño de sus tablas, prohibido acceso cross-module): establece que los módulos no se tocan entre sí a nivel de esquema.
- **ADR-0118** (una feature se construye una sola vez, en el primer módulo que la usa): establece que no se reconstruye la feature en cada módulo consumidor.

Pero **ninguno de los dos dice explícitamente** qué ocurre con la migración de persistencia cuando una feature es consumida por varios módulos. Sin esa regla canónica escrita:

- Un Ingeniero que llegue a EPIC-1 viendo que `fundamental-event-store` tiene "Consumido por: generate, validate, execute, manage" podría asumir que `validate` necesita correr su propia migración `CREATE TABLE fundamental_events`.
- No existe ningún documento que le diga que ese "Consumido por" significa "llama al puerto", no "duplica el esquema".

## 3. La regla a documentar

Lo que el Architect debe fijar como canónico (en sus palabras — yo no redacto el ADR):

1. **Una Feature → Una Tabla → Un Módulo dueño.** La migración de persistencia de una feature se ejecuta UNA SOLA VEZ, en el módulo donde la feature se construye (el primer consumidor en el pipeline `ingest → … → withdraw`). No existe una "copia" de esa tabla en los módulos consumidores.

2. **TTR de Integración ≠ TTR de Construcción.** Cuando un módulo consumidor tiene un TTR que dice "integrar `fundamental-event-store`", ese TTR consiste en: enchufar el puerto `public_interface` del módulo dueño. NUNCA en correr una migración nueva para esa feature.

3. **"Consumido por" en el campo de una feature = accede al puerto, no al esquema.**

4. **Si un módulo consumidor necesita persistir datos propios relacionados con la feature**, los guarda en sus PROPIAS tablas (con una referencia o snapshot del dato de la feature) — no duplica la tabla de la feature.

5. **Dónde amend:** La regla más natural es una **enmienda a ADR-0003**, ya que ese ADR es la fuente de verdad de la propiedad de tablas. Puede incluir también una actualización de `docs/templates/FEATURE.md` para que el campo "Consumido por" tenga una nota de qué significa para la persistencia.

## 4. Qué NO decide el Tech-Lead

- Si la regla va como enmienda de ADR-0003, como enmienda de ADR-0118, o como ADR nuevo: eso lo decide el Architect.
- La redacción exacta del ADR.
- Si la plantilla `FEATURE.md` necesita cambios y cuáles.

## 5. Criterio de cierre de esta TASK

- Existe un ADR (nuevo o enmienda) que contiene la regla de forma canónica.
- El campo "Consumido por" de las features (o su plantilla) tiene una nota que aclara el significado para la persistencia.
- No quedan features multi-consumidor cuyo "Consumido por" pueda interpretarse como "duplicar tabla".
- El Tech-Lead puede citar un número de ADR al Rust-Engineer en una Orden de Trabajo para explicar por qué los TTRs de integración no corren migraciones.

## 6. Comandos de validación

```bash
# No aplica — es documental. El Architect edita docs/adr/ y docs/templates/.
# Verificación: grep de la regla en el ADR resultante.
grep -i "tabla única\|integration TTR\|consumido por\|puerto" docs/adr/ADR-0003.md
```

## 7. Registro de ejecución

**2026-06-20 — Architect (Opus):**
- **`docs/adr/ADR-0003.md`:** añadida sección "Persistencia en Features Multi-Consumidor (Regla de Tabla Única — enmienda 2026-06-20, escalamiento TASK-011)" con 4 reglas FIJO: una Feature → una tabla → un módulo dueño; TTR de Integración ≠ TTR de Construcción; "Consumido por" = accede al puerto; datos propios del consumidor van en sus propias tablas.
- **`docs/adr/ADR-0118.md`:** referencia cruzada bidireccional añadida (cita ADR-0003 Regla de Tabla Única como "cara de persistencia" de Construcción vs Integración).
- **`docs/templates/FEATURE.md`:** nota en sección "Dependencias y Bloqueantes" aclarando que "Consumido por" = accede al puerto, con cita a ADR-0003 y ADR-0118.
- **Decisión del Architect:** enmienda a ADR-0003 (no ADR nuevo) porque la regla es la explicitación de una consecuencia ya implícita en el ADR de propiedad de tablas. ADR-0006 (Migraciones Centralizadas) refuerza la regla estructuralmente.

**Evidencia de verificación (Tech-Lead):**
```
grep -in "tabla única\|integración\|consumido por\|puerto" docs/adr/ADR-0003.md
→ 4 líneas con las reglas canónicas confirmadas.

grep -n "Consumido por\|puerto\|ADR-0003" docs/templates/FEATURE.md
→ Nota en línea 83 confirmada.
```

## 8. Pendientes derivados

- Tras la enmienda, el Tech-Lead actualiza la Orden de STORY-009 y STORY-010 (si tiene TTRs de integración) para citar el ADR.
- El Gate de Coherencia Pre-Despacho del `tech-lead/SKILL.md` ya incorpora la regla de tabla única (añadida 2026-06-20); solo falta el número de ADR para poder citarlo.
