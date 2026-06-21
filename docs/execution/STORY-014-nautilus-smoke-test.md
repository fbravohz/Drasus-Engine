# STORY-014 · Smoke test: NautilusTrader v2 crates compilan en el workspace

| Campo | Valor |
|---|---|
| **ID** | STORY-014 |
| **Título** | Smoke test: NautilusTrader v2 crates compilan en el workspace |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | En curso |
| **Responsable** | Rust-Engineer (Sonnet) · auditará Tech-Lead + QA-Engineer |
| **Creada** | 2026-06-21 |
| **Completada** | — |

## 0. Resumen ejecutivo

La investigación sobre si NautilusTrader v2 es viable como motor ya está resuelta (ADR-0107): se usarán los crates Rust nativos del núcleo v2, vendorizados con versión exacta. Lo que nunca se ejecutó es la confirmación empírica de que esos crates **existen en crates.io, compilan sin conflictos con nuestras dependencias actuales**, y que el patrón de capa anticorrupción es viable en el workspace real.

**Qué se construye:**
- Un nuevo crate `crates/nautilus_compat` en el workspace: la capa anticorrupción stub.
- Dependencias de NT v2 añadidas con versión exacta fijada (`=x.y.z`).
- Un test de smoke que crea/referencia al menos un tipo NT a través del stub.

**Por qué ahora:** si NT no compila, hay que activar el Plan B del ADR-0107 antes de diseñar EPIC-1 (que depende de los adaptadores NT para la ingesta). Este cierra el gate de SPIKE-001.

---

## 1. Especificación de origen
- **Feature:** [`nautilus-integration.md`](../features/nautilus-integration.md)
- **TTR(s):** ninguno (smoke test pre-implementación; los TTRs reales son EPIC-2/5)
- **ADR(s):** ADR-0107 (integración NT v2), ADR-0003 (una Feature → un crate dueño)

## 2. Objetivo
Confirmar que los crates Rust v2 de NautilusTrader compilan en el workspace con el patrón de capa anticorrupción, o documentar el fallo y activar el Plan B del ADR-0107.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Rust-Engineer | Etapa 2 — implementación | ninguno | Docente |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | **Rust-Engineer** | **Autónomo** |

## 4. Instrucciones de despacho por agente

### 4.1 Rust-Engineer (Modo Docente)

```
Eres el Rust-Engineer de Drasus Engine. Lee primero:
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/base/SKILL.md
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/rust-engineer/SKILL.md

Tu Modo para esta Orden es DOCENTE (ADR-0122): implementas tú cada bloque con Edit/Write, y antes de avanzar al siguiente te detienes, explicas el concepto de Rust con profundidad cero-conocimiento (nunca asumas que el usuario ya sabe) e invitas preguntas. Un bloque por vez.

Lee la Orden completa:
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/docs/execution/STORY-014-nautilus-smoke-test.md

Contexto obligatorio:
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/docs/adr/ADR-0107.md
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/docs/features/nautilus-integration.md
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/Cargo.toml

--- TAREA ---

BLOQUE 1 — Investigación de crates disponibles:
  Busca en crates.io los crates Rust del núcleo v2 de NautilusTrader. El proyecto está en
  nautechsystems/nautilus_trader (GitHub). Busca nombres como nautilus-model, nautilus-core,
  nautilus-common o similares. Usa `cargo search` o WebSearch para encontrar los crates exactos.
  Documenta en §8 qué encontraste: nombres, versiones disponibles, si son crates.io o git.
  Si no existen en crates.io, busca la fuente git oficial y planifica la dependencia con rev fijo.

BLOQUE 2 — Crear el crate stub:
  - Añade `"crates/nautilus_compat"` al array `members` del Cargo.toml raíz.
  - Crea `crates/nautilus_compat/Cargo.toml` declarando los crates NT encontrados con versión
    EXACTA fijada (ej. `nautilus-model = { version = "=0.x.y" }` o git con `rev`).
  - Crea `crates/nautilus_compat/src/lib.rs` con un módulo `stub` que importa al menos un
    tipo público de NT (enum o struct básico). Sin lógica de negocio — solo la importación.
  - RESTRICCIÓN: ningún crate del workspace fuera de `nautilus_compat` puede importar tipos NT.

BLOQUE 3 — Smoke test:
  Escribe un test en `crates/nautilus_compat/tests/smoke_test.rs` (o `src/lib.rs` #[cfg(test)]):
    `nautilus_crates_compile_and_basic_type_is_accessible`
  Crea o referencia al menos un tipo NT. Si la API no permite instanciación sin contexto,
  basta con `use nautilus_xxx::SomeType; let _ = std::any::TypeId::of::<SomeType>();`

BLOQUE 4 — Verificación y limpieza:
  Corre `cargo build --workspace` y `cargo clippy --workspace --all-targets -- -D warnings`.
  Si NT introduce warnings que no son del proyecto, supríme con `#[allow(...)]` documentando.
  Corre `cargo test --workspace` y `cargo llvm-cov --workspace --summary-only`.
  Documenta resultados en §7 de la Orden.

CASO FALLO: si los crates no existen en ninguna fuente confiable, deja `lib.rs` con
  `// TODO: Plan B ADR-0107 — crates NT v2 no encontrados`, documenta en §8, y reporta al
  Tech-Lead para escalar al Architect.

Documenta tu Plan de Implementación en §4.1 de la Orden ANTES de empezar a editar código.
Lección consolidada en: docs/lessons/rust/STORY-014-nautilus-smoke-test.md
```

**Plan de Implementación** (Rust-Engineer · Modo Docente · 2026-06-21):

**Premisa:** Los crates NT v2 pueden o no estar en crates.io. Hay dos caminos:
- **Camino A (crates.io):** dependencia con versión exacta `=x.y.z`.
- **Camino B (git):** dependencia git con `rev` fijo al commit más reciente de la rama `develop` o `main` de `nautechsystems/nautilus_trader`.
- **Camino C (Plan B ADR-0107):** si ninguno funciona, `lib.rs` queda con comentario TODO y se escala al Architect.

**Orden de ejecución:**

1. **BLOQUE 1 — Investigación:** buscar con `cargo search nautilus` y WebSearch si los crates existen en crates.io con sus versiones exactas. Documentar en §8. Enseñar: qué es crates.io, diferencia crates.io vs git en Cargo, por qué ADR-0107 exige versión exacta.

2. **BLOQUE 2 — Stub:** (a) añadir `"crates/nautilus_compat"` a `members` del `Cargo.toml` raíz; (b) crear `crates/nautilus_compat/Cargo.toml` con la dependencia NT de versión exacta; (c) crear `crates/nautilus_compat/src/lib.rs` con módulo `stub` que importa al menos un tipo NT. Enseñar: workspace Cargo, resolución de dependencias, capa anticorrupción.

3. **BLOQUE 3 — Smoke test:** crear `crates/nautilus_compat/tests/smoke_test.rs` con el test `nautilus_crates_compile_and_basic_type_is_accessible`. Enseñar: `std::any::TypeId`, diferencia unitario vs integration test en Rust.

4. **BLOQUE 4 — Verificación:** correr los 6 comandos de §6, documentar resultados en §7, suprimir warnings ajenos con `#[allow(...)]` comentado. Enseñar: `Cargo.lock` como fuente de verdad, `-D warnings` en clippy.

5. **CIERRE:** crear `docs/lessons/rust/STORY-014-nautilus-smoke-test.md` con todos los conceptos enseñados.

### 4.2 QA-Engineer (Modo Autónomo)

```
Eres el QA-Engineer de Drasus Engine. Lee:
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/base/SKILL.md
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/qa-engineer/SKILL.md

Lee la Orden completa:
  /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/docs/execution/STORY-014-nautilus-smoke-test.md

Audita el entregable del Rust-Engineer contra los 5 criterios de §5. Verifica:
1. `grep "nautilus" Cargo.lock` muestra al menos una entrada con versión exacta.
2. `cargo build --workspace` limpio.
3. `grep -rn "use nautilus" crates/ --include="*.rs" | grep -v nautilus_compat` → 0 resultados
   (ningún tipo NT fuera del crate stub).
4. El test smoke existe y corre verde.
5. `cargo clippy --workspace --all-targets -- -D warnings` limpio.
Emite veredicto APTO o NO APTO con evidencia concreta.
```

---

## 5. Criterio de aceptación

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | Crates NT v2 en `Cargo.lock` con versión exacta | `grep "nautilus" Cargo.lock` muestra la entrada |
| 2 | `cargo build --workspace` sin errores | salida limpia |
| 3 | Ningún tipo NT importado fuera de `nautilus_compat` | grep → 0 resultados fuera del crate stub |
| 4 | Smoke test verde | `nautilus_crates_compile_and_basic_type_is_accessible` pasa |
| 5 | `cargo clippy` limpio | 0 warnings propios del proyecto |

## 6. Comandos de validación
```bash
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
grep "nautilus" Cargo.lock
grep -rn "use nautilus" crates/ --include="*.rs" | grep -v nautilus_compat
cargo llvm-cov --workspace --summary-only
```

## 7. Registro de ejecución

**Ejecutado:** 2026-06-21 · Rust-Engineer · Modo Docente · Lección: [`docs/lessons/rust/STORY-014-nautilus-smoke-test.md`](../lessons/rust/STORY-014-nautilus-smoke-test.md)

**QA Gate:** 2026-06-21 · QA-Engineer · Modo Autónomo · **Veredicto: APTO**

### Criterios de aceptación — resultado

| # | Criterio | Resultado |
|---|---|---|
| 1 | Crates NT v2 en `Cargo.lock` con versión exacta | ✅ `nautilus-model 0.58.0` y `nautilus-core 0.58.0` aparecen en `Cargo.lock` |
| 2 | `cargo build --workspace` sin errores | ✅ Limpio en 14.87s |
| 3 | Ningún tipo NT importado fuera de `nautilus_compat` | ✅ grep → 0 resultados fuera del crate stub |
| 4 | Smoke test verde | ✅ `nautilus_crates_compile_and_basic_type_is_accessible` pasa |
| 5 | `cargo clippy` limpio | ✅ 0 warnings propios del proyecto |

### Salidas de los comandos de §6

```
cargo build --workspace           → Finished `dev` profile en 14.87s (0 errores)
cargo clippy --workspace ...      → Finished (0 warnings, 0 errores)
cargo test --workspace            → 110 tests: 110 passed, 0 failed
grep "nautilus" Cargo.lock        → nautilus-core 0.58.0 · nautilus-model 0.58.0
grep -rn "use nautilus" crates/   → (sin resultados fuera de nautilus_compat)
cargo llvm-cov --workspace        → Líneas: 85.42% · Funciones: 78.92%
```

### Mapeo criterio → prueba

| Criterio | Prueba |
|---|---|
| CA-4: smoke test accesible | `tests/smoke_test.rs::nautilus_crates_compile_and_basic_type_is_accessible` |
| CA-3: aislamiento del stub | grep de §6 → 0 resultados (evidencia estructural, no test) |

### Advertencias suprimidas
Ninguna. `nautilus-model 0.58.0` no introduce warnings en el código del proyecto.

### Veredicto QA (2026-06-21)

**APTO — 5/5 criterios verificados con evidencia real. Gate de SPIKE-001 cerrado.**

| Criterio | Evidencia | Resultado |
|---|---|---|
| 1 · Crates NT en `Cargo.lock` | `grep "nautilus" Cargo.lock` → `nautilus-core` y `nautilus-model`; `Cargo.toml` fija `=0.58.0` | ✅ |
| 2 · Build limpio | `Finished dev profile in 0.13s` — 0 errores | ✅ |
| 3 · Aislamiento del stub | `grep -rn "use nautilus" crates/ \| grep -v nautilus_compat` → 0 resultados | ✅ |
| 4 · Smoke test verde | `nautilus_crates_compile_and_basic_type_is_accessible ... ok`; 110 tests, 0 fallos | ✅ |
| 5 · Clippy limpio | `Finished dev profile in 0.17s` — 0 warnings, 0 errores | ✅ |

Revisión de código fuente: ningún `unwrap()` ni `unsafe` sin justificación; cero lógica de negocio en el stub; comentarios en español, identificadores en inglés (ADR-0121); test importa el tipo vía `nautilus_compat::stub`, no directamente de `nautilus_model` — contrato de encapsulación verificado.

## 8. Pendientes derivados / decisiones

### Hallazgos BLOQUE 1 (2026-06-21) — Investigación de crates

**Fuente:** crates.io (Camino A — crates.io con versión exacta).

Los crates Rust nativos del núcleo v2 de NautilusTrader **existen en crates.io** con la versión `0.58.0`:

| Crate | Versión | Descripción |
|---|---|---|
| `nautilus-model` | `0.58.0` | Modelo de dominio (tipos de instrumentos, órdenes, barras, events) |
| `nautilus-core` | `0.58.0` | Funcionalidad central del motor |
| `nautilus-common` | `0.58.0` | Maquinaria compartida (relojes, caché, sistema de mensajes) |
| `nautilus-data` | `0.58.0` | Manejo de datos de mercado |
| `nautilus-serialization` | `0.58.0` | Serialización (Parquet/Arrow) |

- **Licencia:** LGPL-3.0-or-later (conforme a ADR-0107).
- **Rust mínimo requerido:** 1.96.0 — coincide con el toolchain instalado (`rustc 1.96.0 · 2026-05-25`).
- **Repositorio upstream:** `https://github.com/nautechsystems/nautilus_trader`.
- **Decisión:** se usa `nautilus-model = "=0.58.0"` (versión exacta, Camino A). Solo este crate en el stub — es el más pequeño y contiene los tipos de dominio que la capa anticorrupción necesita mapear.
- Vendoring completo (`vendor/`) se realiza al arrancar EPIC-2 (ADR-0107 §Versionado Congelado).

### Pendientes restantes
- La implementación real de `nautilus-integration` (TTR-001 a TTR-004) es EPIC-2/5.
- Vendoring completo (código fuente local bajo `vendor/`) se hace al arrancar EPIC-2.
