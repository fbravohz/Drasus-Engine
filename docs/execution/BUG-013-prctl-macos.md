# BUG-013 · `prctl` sin gate Linux en `worker_runner.rs` — fallo de compilación en macOS

| Campo | Valor |
|---|---|
| **ID** | BUG-013 |
| **Título** | `nix::sys::prctl::set_pdeathsig` dentro de `#[cfg(unix)]` rompe compilación en macOS |
| **Tipo** | Bug |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | ✅ Completada |
| **Responsable** | Rust-Engineer (Sonnet) · QA-Engineer (Sonnet) · auditó Tech-Lead |
| **Creada** | 2026-06-20 |
| **Completada** | 2026-06-20 |

## 0. Resumen ejecutivo

`worker_runner.rs` compila en Linux y Windows (con stub), pero falla en macOS. La causa: `nix::sys::prctl::set_pdeathsig` vive dentro de un bloque `#[cfg(unix)]` que incluye macOS, pero `nix::sys::prctl` es un módulo Linux-only — en macOS ese símbolo no existe. La decisión arquitectónica (ADR-0134) confirmada: `prctl` es una optimización opcional Linux-only; macOS y Windows usan el mecanismo de keepalive file ya existente. Fix: envolver el bloque `unsafe { cmd.pre_exec(...) }` con `#[cfg(target_os = "linux")]`.

---

## 1. Especificación de origen

- **ADR-0134** — Matriz de Plataformas: prctl es optimización Linux-only; macOS usa solo keepalive (~50ms de latencia, aceptable). `#[cfg(unix)]` NUNCA puede contener código Linux-only.
- **Archivo afectado:** `crates/shared/src/orchestrator/worker_runner.rs`

## 2. Causa raíz exacta

```
Línea 299: #[cfg(unix)]          ← incluye Linux + macOS
impl WorkerBackend for OsWorkerBackend {
    fn launch(...) {
        ...
        unsafe {
            cmd.pre_exec(|| {
                nix::sys::prctl::set_pdeathsig(...)  ← Linux-only: módulo no existe en macOS
            });
        }
    }
    ...
}
```

En macOS, `nix::sys::prctl` no está compilado — el compilador falla con "unresolved module".

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Rust-Engineer | Etapa 2 — fix de código | ninguno | Autónomo |
| QA-Engineer | Etapa 5 — gate obligatorio | Rust-Engineer | Autónomo |

## 4. Instrucciones de despacho por agente

### 4.1 Rust-Engineer

```
Eres el Rust-Engineer de Drasus Engine.

PASO OBLIGATORIO ANTES DE ACTUAR:
1. Lee `/var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/base/SKILL.md`.
2. Lee `/var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/rust-engineer/SKILL.md`.
Declara "[base/SKILL.md y rust-engineer/SKILL.md leídos y activos]".

DIRECTORIO: /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine

## Contexto

ADR-0134 confirma: Windows, Linux y macOS son los tres targets de producción. `prctl(PR_SET_PDEATHSIG)` es una syscall Linux-only — se permite como optimización opcional en Linux, pero macOS y Windows solo usan el mecanismo de keepalive file.

## Archivos a leer antes de tocar nada

1. `crates/shared/src/orchestrator/worker_runner.rs` — completo.
2. `crates/shared/Cargo.toml` — ver si `nix` tiene features que limiten prctl a Linux.

## El fix (quirúrgico — mínimo cambio necesario)

En `worker_runner.rs`, dentro del `impl WorkerBackend for OsWorkerBackend` bajo `#[cfg(unix)]`, el método `fn launch` tiene:

```rust
unsafe {
    cmd.pre_exec(|| {
        nix::sys::prctl::set_pdeathsig(Signal::SIGTERM)
            .map_err(|e: nix::errno::Errno| {
                std::io::Error::from_raw_os_error(e as i32)
            })
    });
}
```

**Fix requerido:** envolver ese bloque `unsafe` en `#[cfg(target_os = "linux")]`:

```rust
// Optimización Linux: el kernel envía SIGTERM al hijo si el padre muere.
// En macOS y Windows el keepalive file cumple la misma función (ADR-0134).
#[cfg(target_os = "linux")]
unsafe {
    cmd.pre_exec(|| {
        nix::sys::prctl::set_pdeathsig(Signal::SIGTERM)
            .map_err(|e: nix::errno::Errno| {
                std::io::Error::from_raw_os_error(e as i32)
            })
    });
}
```

El resto del método `launch` (creación del `Command`, `spawn`, inserción en `children`) funciona igual en Linux y macOS — NO lo toques.

## También actualiza los comentarios afectados

Hay comentarios en el archivo que dicen "el despliegue real es Linux (ADR-0016)" — deben actualizarse a "despliegue nativo en Windows/Linux/macOS (ADR-0134)". Busca con grep:

```bash
grep -n "ADR-0016\|despliegue real es Linux\|Implementación Unix.*prctl" crates/shared/src/orchestrator/worker_runner.rs
```

Actualiza cada comentario afectado con la descripción correcta y sin referencias a ADR por número — sigue la política de comentarios de base/SKILL.md.

## Qué NO debes cambiar

- El stub `#[cfg(not(unix))]` para Windows — ya está correcto.
- Los métodos `send_sigterm`, `send_sigkill`, `is_alive` — funcionan en macOS vía POSIX.
- `is_process_alive` — ya tiene el split `#[cfg(target_os = "linux")]` / `#[cfg(not(target_os = "linux"))]` correcto.
- El test de kill-9 (`tests/kill9_recovery.rs`) — ya tiene `#![cfg(unix)]`, correcto.

## Criterios de cierre

1. `cargo build --workspace` limpio en Linux.
2. `cargo clippy --workspace --all-targets -- -D warnings` sin warnings.
3. `cargo test --workspace` todos en verde.
4. `grep -n "prctl" crates/shared/src/orchestrator/worker_runner.rs` — el único `prctl` que aparece está dentro de `#[cfg(target_os = "linux")]`.
5. Comentarios del archivo actualizados: sin referencias a "Linux-only deployment".

Reporta al Tech-Lead: archivos modificados, líneas cambiadas, resultado de cada criterio.
```

### 4.2 QA-Engineer

```
Eres el QA-Engineer de Drasus Engine.

PASO OBLIGATORIO ANTES DE ACTUAR:
1. Lee `/var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/base/SKILL.md`.
2. Lee `/var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/qa-engineer/SKILL.md`.
Declara "[base/SKILL.md y qa-engineer/SKILL.md leídos y activos]".

DIRECTORIO: /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine

## Tu tarea: auditar el fix del BUG-013

Lee `crates/shared/src/orchestrator/worker_runner.rs` completo.

Verifica:
1. **El único `prctl`** en el archivo está dentro de `#[cfg(target_os = "linux")]` — no de `#[cfg(unix)]`. Si hay algún `prctl` fuera de ese gate, es BLOQUEANTE.
2. **`nix::sys::prctl`** no aparece en ningún scope `#[cfg(unix)]` sin el subnivel `target_os = "linux"`. BLOQUEANTE si lo hay.
3. **Los métodos restantes del impl unix** (`send_sigterm`, `send_sigkill`, `is_alive`) siguen intactos y usan POSIX signals que sí existen en macOS.
4. **Los comentarios** del archivo no tienen referencias a ADR por número (`ADR-0016`, `ADR-0134`) — solo descripciones en español. OBSERVACIÓN si los hay.
5. **Lógica del keepalive:** el worker en macOS y Windows sobrevive sin `prctl` porque usa el archivo keepalive. ¿Existe ese mecanismo de keepalive? Búscalo con grep: `grep -n "keepalive\|KEEPALIVE" crates/shared/src/orchestrator/worker_runner.rs`.
6. **`cargo build --workspace`** y **`cargo test --workspace`** en verde.

Reporta al Tech-Lead: APTO o NO APTO con hallazgos clasificados como BLOQUEANTE / OBSERVACIÓN / SUGERENCIA.
```

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | `cargo build --workspace` limpio | — (build) |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings | — (clippy) |
| 3 | El único `prctl` en worker_runner.rs está bajo `#[cfg(target_os = "linux")]` | `grep -n "prctl" crates/shared/src/orchestrator/worker_runner.rs` |
| 4 | `nix::sys::prctl` no aparece en scope `#[cfg(unix)]` sin subnivel linux | grep |
| 5 | Tests previos siguen verdes | `cargo test --workspace` |
| 6 | QA veredicto APTO | Reporte QA-Engineer |

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
grep -n "prctl" crates/shared/src/orchestrator/worker_runner.rs
grep -n "#\[cfg(unix)\]" crates/shared/src/orchestrator/worker_runner.rs
```

## 7. Registro de ejecución

### Rust-Engineer — 2026-06-20

**Archivo modificado:** `crates/shared/src/orchestrator/worker_runner.rs`

**Líneas cambiadas:**

| Línea(s) | Cambio |
|---|---|
| 1–3 (doc módulo) | Eliminadas referencias `ADR-0013, ADR-0016`; añadida frase de soporte multi-plataforma |
| 25–28 (doc TTR-002) | Aclarado que `prctl` es Linux-only; keepalive cubre macOS y Windows |
| 187–190 (is_process_alive doc) | Reemplazado "SO de despliegue, ADR-0016" por descripción correcta de los tres sistemas |
| 264–272 (OsWorkerBackend doc) | Reescrito para reflejar que `prctl` solo aplica en Linux |
| 301–315 (impl unix doc) | Cabecera e introducción actualizadas: "Linux y macOS" explícito; `prctl` marcado como Linux |
| 323–332 (bloque unsafe) | Añadido `#[cfg(target_os = "linux")]` antes del `unsafe`; comentario actualizado |
| 362–364 (stub Windows doc) | Eliminada referencia "despliegue real es Linux (ADR-0016)" |

**Criterios de cierre:**

| # | Resultado |
|---|---|
| 1 | `cargo build --workspace` — `Finished` sin errores |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings |
| 3 | `grep -n "prctl" worker_runner.rs` — único `nix::sys::prctl::set_pdeathsig` en línea 333, bajo `#[cfg(target_os = "linux")]` en línea 330 |
| 4 | `nix::sys::prctl` no aparece en ningún scope `#[cfg(unix)]` sin subnivel linux |
| 5 | `cargo test --workspace` — 103 tests passed, 0 failed |

### QA-Engineer — 2026-06-20

Revisión completa de `worker_runner.rs`. Sin hallazgos BLOQUEANTES. Confirmado: único `prctl` bajo `#[cfg(target_os = "linux")]`; imports unix solo usan POSIX (válidos en macOS); keepalive presente en dos capas; comentarios sin referencias a ADR por número. Veredicto: **APTO**.

### Tech-Lead — 2026-06-20 (auditoría independiente)

Build limpio, clippy 0 warnings, 103/103 tests verdes. `prctl` en línea 333 confirmado bajo `#[cfg(target_os = "linux")]` en línea 330 (lectura directa del archivo). BUG-013 → **CERRADO**.

## 8. Pendientes derivados

- Cuando el proyecto madure en macOS (EPIC-8+), evaluar añadir `kqueue(2)` + `EVFILT_PROC` como mejora opcional de detección inmediata (ya documentado en ADR-0134).
- Auditar moonshots para referencias "Linux-only" si procede (TASK futura).
