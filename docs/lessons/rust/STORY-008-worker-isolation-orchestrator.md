> Story: [STORY-008 — Orquestador de Aislamiento de Workers](../../execution/STORY-008-worker-isolation-orchestrator.md)

# Lecciones de STORY-008 — Worker Isolation Orchestrator

Conceptos enseñados durante esta historia, con ejemplos del código real producido.

---

## Concepto 1 — `mmap`: memoria compartida sin copias entre procesos

### ¿Qué es?

`mmap` (memory-mapped file) es una llamada del sistema operativo que mapea el contenido de un archivo directamente al espacio de direcciones virtuales de un proceso. En vez de copiar los datos con `read()`, el proceso accede a las páginas de memoria directamente. El OS mantiene un único conjunto de marcos físicos de RAM para ese archivo, y cualquier proceso que mapee el mismo archivo reutiliza esos mismos marcos.

### ¿Por qué existe?

El problema que resuelve: si el orquestador tiene un buffer de barras de mercado de 2 GB y lanza 8 workers, sin `mmap` cada worker recibiría una copia de 2 GB (16 GB en total). Con `mmap(MAP_SHARED)`, todos los procesos comparten las mismas páginas físicas — el consumo de RAM no crece con el número de workers.

### ¿Cómo se usa en Rust? La crate `memmap2`

```rust
// En crates/shared/src/orchestrator/worker_runner.rs

// El padre crea el segmento con acceso de lectura-escritura:
let mut file = OpenOptions::new()
    .read(true).write(true).create(true).truncate(true)
    .open(&data_path)?;
file.write_all(data)?;
// SAFETY: el archivo acaba de ser creado; solo este hilo tiene acceso.
let mmap: MmapMut = unsafe { MmapOptions::new().map_mut(&file) }?;

// Un worker abre el mismo archivo solo en lectura:
let file = File::open(path)?;        // O_RDONLY — sin permiso de escritura
let mmap: Mmap = unsafe { MmapOptions::new().map(&file) }?;
```

El tipo `MmapMut` (del padre) y `Mmap` (del worker) se diferencian en los permisos del mapping:
- `map_mut()` → `mmap(PROT_READ | PROT_WRITE, MAP_SHARED, fd, ...)`
- `map()` → `mmap(PROT_READ, MAP_SHARED, fd, ...)`

Si el `fd` fue abierto con `O_RDONLY` y se intenta `map_mut()`, el OS devuelve `EACCES`. Rust lo convierte a `Err(...)`. Eso es lo que prueba el criterio 3.

### ¿Qué pasa con el bloque `unsafe`?

`MmapOptions::map_mut()` es `unsafe` porque Rust no puede verificar que el archivo subyacente no cambie mientras el `MmapMut` está vivo. Si otro proceso escribe en el mismo archivo de forma concurrente, el contenido del slice puede cambiar — lo que en Rust viola el invariante de `&mut`. El contrato de seguridad lo establece el programador: en esta implementación, el padre escribe primero y solo luego los workers leen.

---

## Concepto 2 — FCIS: el dominio no importa `std::process`, `nix` ni `memmap2`

### ¿Qué es FCIS?

FCIS (Functional Core / Imperative Shell) divide el código en dos capas:
- **Core (dominio):** lógica pura, sin efectos de I/O. Misma entrada → misma salida, bit a bit.
- **Shell (cáscara):** coordina el mundo real (archivos, red, procesos, tiempo).

### ¿Por qué es importante?

Un módulo que importa `std::process::Command` no puede ser probado sin lanzar procesos reales. Un módulo que importa `nix::signal::kill` no puede probarse sin tener PIDs reales. Al separar las decisiones de los efectos, las primeras se prueban con tests unitarios instantáneos (sin I/O); las segundas, en tests de integración.

### ¿Cómo se expresa en el código?

```rust
// domain/worker_orchestrator.rs — núcleo puro
// Sin ningún import de sistema:
use std::collections::HashMap;

pub trait WorkerBackend: Send + Sync {
    fn launch(&self, job_id: &str, shm_path: &str, keepalive_path: &str)
        -> Result<u32, WorkerBackendError>;
    fn send_sigterm(&self, pid: u32) -> Result<(), WorkerBackendError>;
    fn send_sigkill(&self, pid: u32) -> Result<(), WorkerBackendError>;
    fn is_alive(&self, pid: u32) -> bool;
}
```

```rust
// orchestrator/worker_runner.rs — cáscara con efectos reales
use std::process::Command;
use memmap2::{Mmap, MmapMut, MmapOptions};
use nix::sys::signal::{self, Signal};

pub struct OsWorkerBackend { binary: PathBuf, ... }
impl WorkerBackend for OsWorkerBackend { ... } // efectos reales aquí
```

El dominio define **qué** hacer; la cáscara lo **hace**. Las pruebas del dominio son síncronas y no necesitan procesos reales.

---

## Concepto 3 — `pre_exec`: código que corre en el hijo antes del `exec`

### ¿Qué problema resuelve?

Cuando un proceso lanza un hijo con `Command::spawn()`, internamente ocurre un `fork()` + `exec()`. Hay un breve instante entre el fork y el exec donde el proceso hijo es una copia del padre pero todavía no ejecuta el binario destino. `CommandExt::pre_exec` permite registrar una función que corre en ese instante, en el contexto del hijo.

Para este feature, lo usamos para llamar a `prctl(PR_SET_PDEATHSIG, SIGTERM)`: si el padre muere, el kernel envía SIGTERM al hijo automáticamente.

```rust
// orchestrator/worker_runner.rs

unsafe {
    cmd.pre_exec(|| {
        // Esta closure corre en el proceso hijo, tras fork, antes del exec.
        // Solo hay un hilo (el que hizo fork) — es seguro llamar a prctl aquí.
        nix::sys::prctl::set_pdeathsig(Signal::SIGTERM)
            .map_err(|e: nix::errno::Errno| {
                std::io::Error::from_raw_os_error(e as i32)
            })
    });
}
```

### ¿Por qué es `unsafe`?

En el hijo después del fork, todos los mutexes del padre siguen en el estado que tenían. Si otro hilo del padre tenía un mutex bloqueado en el momento del fork, el hijo hereda ese mutex en estado "bloqueado para siempre" — no hay nadie que lo libere. Por eso Rust marca `pre_exec` como `unsafe`: el programador debe garantizar que la closure no toca mutexes ni estructuras compartidas.

La solución: la closure solo llama a `prctl`, una syscall que no toca nada de Rust.

---

## Concepto 4 — Keepalive file: cómo un proceso detecta que su padre murió

### ¿El problema?

`prctl(PR_SET_PDEATHSIG)` cubre la muerte real del proceso padre. Pero en tests no podemos matar el proceso padre (es `cargo test`). Necesitamos un mecanismo simulable.

### La solución: archivo centinela

El padre crea un archivo vacío (`drasus-shm-<uuid>.keepalive`). Los workers sondean su existencia cada 50ms. Cuando el padre "muere" (simulado mediante `drop(segment)`), el archivo se elimina y los workers terminan:

```rust
// orchestrator/worker_runner.rs — dentro del Drop del padre
impl Drop for SharedMemorySegment {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.keepalive_path); // señal de cierre
        let _ = std::fs::remove_file(&self.data_path);
    }
}
```

El worker (proceso `sh` en los tests):
```sh
while [ -f "$DRASUS_WORKER_KEEPALIVE" ]; do sleep 0.05; done
```

### ¿Por qué no solo `prctl`?

`prctl(PR_SET_PDEATHSIG)` no funciona cuando el proceso muere con `SIGKILL` y el sistema operativo no tiene tiempo de notificar. El archivo keepalive actúa de capa adicional: si el padre murió violentamente y no ejecutó `Drop`, en producción puede haber un watchdog externo que lo elimine.

---

## Concepto 5 — Zombies y `/proc/{pid}/stat`

### ¿Qué es un proceso zombie?

Cuando un proceso hijo termina, el OS no lo elimina de la tabla de procesos de inmediato. Guarda su entrada (incluyendo el código de salida) hasta que el padre llame a `wait()`. Mientras tanto, el proceso está en estado **Z (zombie)**: ya no corre, pero su entrada en `/proc/{pid}` existe.

### El bug que tuvo la implementación

La primera versión de `is_process_alive` solo verificaba si `/proc/{pid}` existía:
```rust
// VERSIÓN INCORRECTA
fn is_process_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}
```

En los tests, después de enviar SIGTERM, los procesos `sleep` terminaban y se convertían en zombies. La función los reportaba como "vivos", el test esperaba 2s, y el timeout expiraba forzando SIGKILL.

### La corrección: leer el campo `state` de `/proc/{pid}/stat`

```rust
// orchestrator/worker_runner.rs — versión correcta
pub fn is_process_alive(pid: u32) -> bool {
    let stat = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if let Some(pos) = stat.rfind(')') {
        // Formato: "pid (nombre) estado ..."
        // rfind(')') porque el nombre puede contener paréntesis.
        let state = stat[pos + 1..].trim_start().chars().next().unwrap_or('Z');
        state != 'Z'
    } else {
        true
    }
}
```

La razón de `rfind` (no `find`): algunos nombres de proceso contienen paréntesis (ej. `(kworker/0:0)`). El nombre siempre es el primer campo entre `(...)`, así que el último `)` marca el fin del nombre.

---

## Concepto 6 — `graceful_shutdown`: SIGTERM → espera → SIGKILL

### El protocolo de apagado de dos fases

```rust
// orchestrator/worker_runner.rs
pub async fn graceful_shutdown(pids: &[u32], timeout: Duration) -> Vec<u32> {
    // Fase 1: dar la oportunidad de limpieza
    for &pid in pids {
        let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }

    // Fase 2: esperar hasta el deadline
    let deadline = Instant::now() + timeout;
    let mut still_alive: Vec<u32> = pids.to_vec();
    while !still_alive.is_empty() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
        still_alive.retain(|&pid| is_process_alive(pid));
    }

    // Fase 3: forzar a los supervivientes
    for &pid in &still_alive {
        let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
    }
    still_alive // PIDs que necesitaron SIGKILL
}
```

### ¿Por qué `async`?

El `sleep` de espera usa `tokio::time::sleep` (en vez de `std::thread::sleep`) para no bloquear el hilo del runtime de Tokio. Si el orquestador maneja otras tareas concurrentes (reportar estado, escuchar cancelaciones), un `thread::sleep` lo bloquearía durante toda la espera de 2 segundos.

### SIGTERM vs SIGKILL

| Señal | El proceso puede ignorarla | Limpieza | Uso |
|---|---|---|---|
| SIGTERM | Sí | Puede cerrar archivos, liberar recursos | Primer intento |
| SIGKILL | No — el kernel lo mata directamente | Ninguna | Último recurso |

---

## Trucos de Senior

### 1. `Vec::retain` para filtrar en-lugar

```rust
still_alive.retain(|&pid| is_process_alive(pid));
```

`retain` es más eficiente que `filter` + `collect` porque modifica el vector en su lugar sin alocar uno nuevo. Útil en loops de polling donde el vector cambia en cada iteración.

### 2. `unwrap_or_else` en Mutex para recuperar posesión

```rust
let mut children = self.children
    .lock()
    .unwrap_or_else(|e| e.into_inner());
```

Si el mutex está "envenenado" (un hilo entró en pánico mientras lo tenía bloqueado), `lock()` devuelve `Err(PoisonError)`. `into_inner()` recupera la guardia de todas formas — útil cuando queremos seguir operando aunque un hilo anterior fallara.

### 3. `std::hint::black_box` para evitar que el compilador optimice el test

```rust
let _ = std::hint::black_box(segment.as_slice()[0]);
```

Sin `black_box`, el compilador puede detectar que el resultado del acceso nunca se usa y eliminar la lectura por completo en modo release. `black_box` le impide hacer esa optimización, garantizando que la medición de latencia sea real.
