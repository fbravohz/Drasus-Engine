# <ID> · <Título llano>

> **Plantilla de Orden de Trabajo (Spec-Driven).** Copia este archivo a `docs/execution/<ID>-<slug>.md`.
> La Orden de Trabajo es la **especificación ejecutable**: contiene la instrucción EXACTA que recibió el agente,
> los comandos para que el usuario valide por su cuenta, y el registro de lo que pasó. Vive en git, NO en el chat.
> Si la especificación cambia, se EDITA aquí y se re-despacha — así el cambio queda reflejado y versionado.

| Campo | Valor |
|---|---|
| **ID** | STORY-000 |
| **Título** | <llano, sin códigos> |
| **Tipo** | Story \| Spike \| Bug \| Task |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | <n> |
| **Estado** | Pendiente \| En curso \| Bloqueado \| 🟡 Parcial \| ✅ Implementado |
| **Responsable** | <rol> (<modelo>) · auditó <Tech-Lead / QA> |
| **Creada** | YYYY-MM-DD |
| **Completada** | YYYY-MM-DD |

> Para tickets con un solo Agente, basta con **Responsable** arriba. Para tickets multi-agente (ej. Quant + Rust + Flutter), la tabla de §3 es la fuente de verdad: un Agente, una fila, su propio Modo.

## 1. Especificación de origen (qué specs implementa)
Enlaces a las unidades de especificación que gobiernan este trabajo:
- **Feature(s):** [`nombre`](../features/nombre.md)
- **TTR(s):** TTR-001, TTR-002…
- **Módulo(s):** [`nombre`](../modules/nombre.md)
- **ADR(s):** ADR-XXXX

## 2. Objetivo (una frase llana)
<qué logra este trabajo, en lenguaje de negocio>

## 3. Agentes y Modo de Acompañamiento (ADR-0120)
> Declara aquí TODOS los Agentes que participan en este ticket (uno o varios — ej. Quant-Engineer + Rust-Engineer + Flutter-Engineer) y, por cada uno, su Modo. El Modo se lee SOLO de esta tabla: cuando el usuario invoque el skill del Agente (`/rust-engineer`, `/flutter-engineer`, etc.) pasándole esta Orden, el Agente busca su fila aquí — nunca se le indica el Modo por chat. Si una fila no declara Modo, el Agente asume **Autónomo** (ADR-0120).

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| <Rust-Engineer / Flutter-Engineer / Bridge-Engineer / QA-Engineer / Quant-Engineer / Refactoring-Engineer> | Etapa <n> | <agente del que depende, o "ninguno"> | Autónomo \| Mentor \| Revisión |

**Significado de cada Modo** (contrato completo en `<agente>/SKILL.md` §"Modos de Acompañamiento"):
- **Autónomo:** el Agente implementa y entrega código + pruebas terminadas. El usuario solo audita el resultado.
- **Mentor:** el usuario teclea. El Agente explica el concepto y dicta el fragmento EXACTO a escribir, bloque por bloque, verificando cada bloque antes de avanzar al siguiente.
- **Revisión:** el usuario entrega código ya escrito por su cuenta. El Agente audita y explica el porqué de cada corrección, sin dictar la solución de antemano.

## 4. Instrucciones de despacho por agente (la spec ejecutable)
> El prompt EXACTO que recibe cada Agente. Si cambia la spec, se edita ESTO y se re-despacha. Bajo Modo Mentor o Revisión, el Agente llena aquí mismo (no en el chat) su Plan de Implementación Guiada o Checklist de Revisión, antes/mientras lo ejecuta en el chat — esta Orden es la fuente de verdad, no el chat.

### 4.1 <Nombre del Agente>
```
<prompt completo dado al agente>
```

**Plan de Implementación / Revisión** (lo llena el Agente al ser invocado, según su Modo declarado en §3):
<Modo Mentor → secuencia concepto → fragmento exacto a teclear → punto de verificación, bloque por bloque. Modo Revisión → checklist de auditoría aplicado y veredicto por bloque. Modo Autónomo → resumen de lo implementado.>

<Repetir el bloque "4.n" por cada Agente declarado en §3.>

## 5. Criterio de aceptación (cada criterio ↔ su prueba)
> Regla de proceso: ningún criterio se da por cumplido sin una prueba nombrada que lo ejerza. El ingeniero entrega ya en verde; el Tech-Lead reproduce y verifica cobertura del criterio (no solo "tests verdes").

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | <condición verificable 1> | `<nombre_del_test>` |
| 2 | <condición verificable 2> | `<nombre_del_test>` |

## 6. Comandos de validación (para el usuario — copy/paste)
```bash
# comandos exactos para reproducir y validar por tu cuenta
cargo test -p <crate>
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --summary-only          # % de cobertura de líneas
```

## 7. Registro de ejecución (bitácora cronológica)
- YYYY-MM-DD · <agente/modelo> · <APROBADO / RECHAZADO> · <evidencia de la auditoría del Tech-Lead>

## 8. Pendientes derivados / decisiones
- <pendientes que salieron de este trabajo, con su destino (otra Orden, escalamiento, etc.)>
