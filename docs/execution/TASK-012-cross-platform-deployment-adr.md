# TASK-012 · ADR de despliegue multiplataforma (Windows + Linux + macOS + mobile + web)

| Campo | Valor |
|---|---|
| **ID** | TASK-012 |
| **Título** | Formalizar la matriz de plataformas de despliegue y la topología mobile/web |
| **Tipo** | Task (escalamiento al Architect — sin código) |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | Completada |
| **Responsable** | Architect (Opus) |
| **Creada** | 2026-06-20 |
| **Completada** | 2026-06-20 |

## 0. Resumen ejecutivo

El proyecto confirmó que el backend Rust y el frontend Flutter deben ser nativamente multiplataforma (Windows, Linux, macOS como targets de producción para el desktop; iOS y Android como cliente thin-shell; Web como posible futuro). Esta decisión no estaba documentada en ningún ADR — los documentos existentes asumían Linux como plataforma principal sin declararlo explícitamente. El bug de `prctl` en macOS (detectado 2026-06-20) reveló que hay gaps técnicos sin diseño. El Architect debe formalizar la decisión y definir el mecanismo de reemplazo de `prctl` para macOS/Windows.

---

## 1. Especificación de origen

- **ADR-0033** — Arquitectura de Despliegue Trimodal: ya menciona Windows y Linux pero no los declara como targets nativos.
- **ADR-0030** — Zero-Docker / Persistencia Soberana: menciona "portabilidad absoluta" sin especificar plataformas.
- **ADR-0016** — Local-First: principio agnóstico al SO.
- **Bug STORY-008/009 (2026-06-20):** `worker_runner.rs` usaba `prctl(PR_SET_PDEATHSIG)` dentro de `#[cfg(unix)]`, que es Linux-only — no compila en macOS.

## 2. Decisiones del usuario (confirmadas — el Architect las documenta, no las debate)

1. **Desktop nativo:** Windows, Linux, macOS son los tres targets de producción. El usuario los desarrolla y usa en los tres.
2. **Mobile client-only:** iOS y Android — Flutter thin shell que se conecta al backend Rust vía gRPC. El backend Rust NO corre en mobile.
3. **Topología mobile:** el cliente mobile conecta a: (a) VPS remoto del usuario, (b) red de VPS distribuida, o (c) desktop del usuario vía red local. Las tres variantes son válidas.
4. **Web:** posible en el futuro (incierto). Si ocurre: Flutter Web thin shell → backend Rust remoto. No hay decisión firme aún.
5. **Backend Rust:** corre en desktop (Windows/Linux/macOS) o VPS (Linux). No corre en mobile ni en browser.

## 3. Trabajo del Architect

### 3.1 Crear nuevo ADR (o enmendar ADR-0033)

Documentar canónicamente:
- La matriz de plataformas: qué corre en cada plataforma (backend Rust, frontend Flutter, o ambos).
- La topología mobile: Flutter thin shell → Rust backend remoto/local vía gRPC (extensión natural del modo SaaSCloudEngine de ADR-0033).
- La posición sobre Web: "diseño preparado, implementación futura incierta".
- La relación con ADR-0033: el modo `LocalPowerUser` aplica a Windows/Linux/macOS; `SaaSCloudEngine` es el backend para mobile y web.

### 3.2 Decidir el reemplazo de `prctl` en macOS/Windows

**El gap técnico:** en `worker_runner.rs`, se usa `prctl(PR_SET_PDEATHSIG, SIGTERM)` para que un worker process muera automáticamente si su proceso padre muere. Esta syscall solo existe en Linux. El código ya tiene:
- Implementación Linux: `prctl` + SIGKILL/SIGTERM.
- Stub Windows: retorna error (sin soporte de señales).
- **Hueco:** macOS entra en `#[cfg(unix)]` donde se usa `prctl` → no compila en macOS.

**Opciones a considerar (el Architect elige):**

| Opción | Descripción | Pros | Contras |
|---|---|---|---|
| **A. Keepalive file** | Confiar 100% en el archivo keepalive que ya existe. El worker lo sondea y muere cuando desaparece. Macros ya implementado. | Cero código nuevo; ya funciona | Latencia de detección = intervalo de poll (~50ms) |
| **B. `kqueue` (macOS)** | Monitorear el PID del padre con `kqueue(2)` + `EVFILT_PROC` | Inmediato, igual que `prctl` | Código adicional; solo macOS |
| **C. Thread watchdog** | Thread en el worker que muere si el padre desaparece (verificando `/proc` en Linux, `kill(ppid, 0)` en Unix) | Portátil | Complejidad añadida |

**Recomendación del Tech-Lead (el Architect puede aceptarla o descartar):** Opción A — el keepalive file ya existe y ya fue probado. `prctl` es una optimización de Linux que acelera la detección; en macOS y Windows, el keepalive file con 50ms de latencia es suficiente para EPIC-0. Cuando el proyecto madure en macOS, se puede añadir `kqueue` como mejora en EPIC-8+.

### 3.3 Buscar referencias "Linux-only" en la documentación

Hacer grep en `docs/` para términos como `Linux`, `ubuntu`, `unix`, `ADR-0016` en contexto de plataforma de despliegue y verificar si necesitan actualización. El Architect tiene autoridad para editar `docs/adr/`, `docs/features/`, `docs/modules/`, `docs/sad/`, `docs/ROADMAP.md`.

## 4. Lo que NO hace el Architect en esta TASK

- No toca código (`crates/`) — eso es del Rust-Engineer en el ticket siguiente.
- No modifica los SKILLs — el Tech-Lead ya los actualizó.
- No decide la fecha de implementación de Web o Mobile — solo documenta la posición arquitectónica.

## 5. Criterio de cierre

- Existe un ADR (nuevo o enmienda) que declara la matriz de plataformas canónicamente.
- El ADR documenta la topología mobile (thin shell → Rust backend remoto/local).
- El ADR documenta la decisión sobre `prctl` y su reemplazo en macOS.
- El Tech-Lead puede citar ese ADR en la Orden del Rust-Engineer para el fix de `prctl`.

## 6. Comandos de validación

```bash
# No aplica — es documental.
# Verificación post-Architect:
grep -rn "prctl\|linux.*only\|ubuntu.*CLI" docs/adr/ | head -20
```

## 7. Registro de ejecución

**2026-06-20 — Architect (Opus).** Resolución completa de la TASK-012.

### ADR creado: ADR-0134
Se creó **[ADR-0134](../adr/ADR-0134.md) — Matriz de Plataformas de Despliegue (Desktop Nativo Windows/Linux/macOS + Mobile/Web Cliente Delgado) y Detección de Muerte del Padre Portátil**. Se eligió ADR nuevo sobre enmendar ADR-0033 porque la decisión es de naturaleza distinta: ADR-0033 fija los *modos* de despliegue (cómo se acoplan UI y Core); ADR-0134 fija la *matriz de sistemas operativos* y el *cliente delgado móvil/Web*, una capa que ADR-0033 no cubría. ADR-0134 documenta:
- Matriz canónica: Core Rust nativo en Windows/Linux/macOS y VPS Linux; nunca en móvil ni navegador. Flutter en los tres escritorios (FFI) y como cliente delgado puro en móvil/Web (gRPC).
- Topología móvil: cliente puro a (a) VPS, (b) red de VPS distribuida (ADR-0119), o (c) escritorio del usuario por red local — las tres válidas.
- Posición sobre Web: diseño preparado, implementación futura incierta.

### Decisión sobre `prctl`: Opción A (keepalive file)
Se acepta la recomendación del Tech-Lead. Mecanismo canónico **para todas las plataformas** = archivo keepalive sondeado (~50 ms de latencia, aceptable para EPIC-0). En Linux se permite `prctl(PR_SET_PDEATHSIG)` como optimización **opcional**, acotada estrictamente a `target_os = "linux"` (NUNCA `#[cfg(unix)]`, que incluye macOS y era la causa del bug). macOS y Windows usan solo keepalive. `kqueue(2)`/`EVFILT_PROC` queda como mejora diferida para macOS (EPIC-8+). Justificación: portátil, ya probado, cero código nuevo, sin romper la compilación en Mac.

### Documentos actualizados
- **ADR-0033:** banner extendido para citar ADR-0134; el ejemplo "Ubuntu Server CLI" se generalizó ("típicamente Linux server… la matriz la fija ADR-0134") para no implicar Linux-only.
- **ADR-0016 y ADR-0030:** referencia cruzada añadida a ADR-0134 en su línea "Ver también" (la matriz de SO concreta su principio Local-First / portabilidad).
- **docs/ADR.md:** fila de ADR-0134 añadida al índice.

### Referencias "Linux-only" en la doc
El grep no halló suposiciones Linux-only problemáticas en documentos de diseño. La única referencia a Linux en un ADR era el ejemplo "Ubuntu Server CLI" de ADR-0033 (ya generalizado). ADR-0078 "Multiplatform Infrastructure" se refiere a terminales de bróker (MT/NT/cTrader), no a SO — sin cambio. La mención a `prctl` en `docs/lessons/rust/STORY-008-*.md` es un registro histórico correcto de lo que hace esa syscall — no se altera.

### Pendiente derivado para el Tech-Lead
BUG-013 (fix de `prctl` en `worker_runner.rs`) puede citar ADR-0134 §"Detección de muerte del padre" como la decisión canónica a implementar.

## 8. Pendientes derivados

- **BUG-013** (a crear después del Architect): fix de `prctl` sin `#[cfg(target_os = "linux")]` en `worker_runner.rs` — el Rust-Engineer implementa la decisión del Architect.
- Auditoría futura de moonshots: algunos pueden tener referencias a "Linux-only" que también necesiten actualización.
