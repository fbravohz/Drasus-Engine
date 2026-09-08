# Portabilidad de Compilación en Rust

Qué filtro de compilación exige cada API específica de plataforma, y cuáles son los targets reales del proyecto.

Slug ID: `CROSS-PLATFORM-RUST`.

Tamaño: `4650 caracteres (~1162 tokens aprox., ratio 4:1)`.

---

## CROSS-PLATFORM-RUST-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |

---

## CROSS-PLATFORM-RUST-FV — Filtro de Verificación

**Aplica a:** Rust-Engineer al entregar, y QA al auditar código Rust.

* **CROSS-PLATFORM-RUST-FV-1:** ¿toda API específica de plataforma lleva su filtro exacto, incluidos los tests que la usan? (`CROSS-PLATFORM-RUST-1`)
* **CROSS-PLATFORM-RUST-FV-2:** ¿cada API exclusiva tiene su stub para las demás plataformas, con el comentario que explica por qué existe? (`CROSS-PLATFORM-RUST-2`)
* **CROSS-PLATFORM-RUST-FV-3:** ¿usé `#[cfg(unix)]` cuando la API es POSIX y `#[cfg(target_os = "linux")]` solo cuando de verdad es exclusiva de Linux? (`CROSS-PLATFORM-RUST-1.2`)

De ser satisfactoria cada validación de la lista, declara `CROSS-PLATFORM-RUST - FILTRO DE VERIFICACION APROBADO`.

---

## CROSS-PLATFORM-RUST-1 — Toda API de Plataforma Lleva Su Filtro

El despliegue es Linux ([[docs/adr/ADR-0016.md]]), pero el desarrollo también ocurre en Windows. El workspace **debe compilar en ambos**. No es opcional, y el compilador de una sola plataforma no lo detecta.

**Aplica a:** Rust-Engineer al escribir, y QA al auditar cualquier crate del workspace.

* **CROSS-PLATFORM-RUST-1.1 — Tabla de Filtros Obligatorios:**

  | Si usas... | Filtro obligatorio |
  |---|---|
  | `std::os::unix::*`, `nix::*`, señales POSIX (SIGTERM, SIGKILL) | `#[cfg(unix)]` — cubre Linux + macOS |
  | `/proc/{pid}/stat`, `prctl` (syscalls exclusivas de Linux) | `#[cfg(target_os = "linux")]` |
  | `kqueue`, `mach_*` (exclusivas de macOS) | `#[cfg(target_os = "macos")]` |
  | `std::os::windows::*`, `winapi::` | `#[cfg(windows)]` |
  | Tests que usan cualquiera de lo anterior | el mismo `#[cfg(...)]` en el test |
  | Archivos de test enteros sin sentido fuera de Unix | `#![cfg(unix)]` al inicio del archivo |

* **CROSS-PLATFORM-RUST-1.2 — `unix` No Es Sinónimo de `linux`:**

  `#[cfg(unix)]` cubre Linux **y** macOS, y es el filtro correcto para cualquier API POSIX. `#[cfg(target_os = "linux")]` queda reservado a lo que de verdad solo existe en Linux — `prctl`, `/proc`. Confundirlos produce un fallo latente que no aparece hasta compilar en la otra plataforma.
  * ❌ *Incorrecto:* `prctl` sin filtro, o bajo `#[cfg(unix)]` — compila en Linux y revienta en macOS.
  * ✅ *Correcto:* `prctl` bajo `#[cfg(target_os = "linux")]`; una señal POSIX bajo `#[cfg(unix)]`.

* **CROSS-PLATFORM-RUST-1.3 — El Filtro Alcanza a los Tests:**

  un test que ejercita una API de plataforma sin su filtro rompe la compilación del workspace en la otra plataforma igual que el código de producción.

**Filtro de verificación:** `CROSS-PLATFORM-RUST-FV-1`, `CROSS-PLATFORM-RUST-FV-3`.

---

## CROSS-PLATFORM-RUST-2 — Toda API Exclusiva Necesita Su Stub

Un filtro sin contraparte deja la función inexistente en las demás plataformas, y el error aparece en el llamador, no donde está la causa.

**Aplica a:** Rust-Engineer, cada vez que use una API exclusiva de una plataforma.

* **CROSS-PLATFORM-RUST-2.1 — Stub con Comportamiento Razonable y Su Porqué:**

  añade la implementación alternativa para las plataformas restantes, con un comentario que explique por qué existe el stub — qué hace la plataforma real y qué se degrada aquí.

**Filtro de verificación:** `CROSS-PLATFORM-RUST-FV-2`.

---

## CROSS-PLATFORM-RUST-3 — Targets de Despliegue del Proyecto

Contra qué plataformas se juzga la portabilidad, y dónde corre cada capa.

**Aplica a:** Rust-Engineer, QA y Tech-Lead, al decidir si un fallo de plataforma es bloqueante.

* **CROSS-PLATFORM-RUST-3.1 — Desktop Nativo:**

  Windows, Linux y macOS — los tres son targets de producción, ninguno es secundario.

* **CROSS-PLATFORM-RUST-3.2 — Mobile Client-Only:**

  iOS y Android reciben una cáscara delgada de Flutter que conecta al backend Rust por gRPC — VPS, red local o VPS distribuida.

* **CROSS-PLATFORM-RUST-3.3 — Web:**

  posible a futuro como cáscara delgada de Flutter Web. No es target hoy.

* **CROSS-PLATFORM-RUST-3.4 — Dónde Corre el Backend:**

  el backend Rust corre en desktop o en VPS. **Nunca** en mobile ni en navegador — un requisito que obligue a lo contrario es un error de diseño, no un problema de portabilidad.
