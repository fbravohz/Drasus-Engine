> Story: [STORY-014 · Smoke test: NautilusTrader v2 crates compilan en el workspace](../../execution/STORY-014-nautilus-smoke-test.md)

# Lecciones — STORY-014: Smoke test NautilusTrader v2

Esta Story integró los crates Rust del motor NautilusTrader v2 al workspace del proyecto, creando la capa anticorrupción stub `nautilus_compat`. Los conceptos enseñados cubren la organización de proyectos Cargo, cómo Rust resuelve dependencias, y el patrón de aislamiento de dependencias externas.

---

## Concepto

### 1. Qué es crates.io y cómo Cargo lo usa

**Qué es el problema.** Cuando escribes código Rust y necesitas usar una librería externa (por ejemplo, para serialización, manejo de fechas o, en este caso, un motor de trading), necesitas una forma de declararla y descargarla automáticamente.

**La solución.** `crates.io` es el registro público oficial de librerías Rust. Funciona igual que npm para JavaScript o PyPI para Python: es el lugar donde los autores publican sus librerías y donde Cargo las busca cuando las necesitas.

Cargo es la herramienta de construcción y gestión de dependencias de Rust. Cuando agregas una dependencia en `Cargo.toml` y corres `cargo build`, Cargo va a crates.io, descarga el código de esa librería, la compila y la enlaza con tu proyecto.

**El código real de esta Story.** En `crates/nautilus_compat/Cargo.toml`:

```toml
[dependencies]
nautilus-model = { version = "=0.58.0" }
```

Esta línea le dice a Cargo: "busca en crates.io el crate llamado `nautilus-model` y descarga exactamente la versión `0.58.0`". El prefijo `=` antes del número de versión es la clave del siguiente concepto.

---

### 2. Versión exacta vs. versión flexible: por qué usamos `=0.58.0`

**El problema.** Por defecto, cuando escribes `version = "0.58.0"` en Cargo (sin el `=`), Cargo interpreta eso como "esta versión o cualquier compatible hacia arriba". Podría descargar `0.58.1`, `0.58.5` o incluso `0.59.0` en el futuro si están disponibles. Esto es cómodo para librerías estables, pero peligroso para una librería como NautilusTrader que todavía declara inestabilidad entre releases (serie `0.x`): cualquier actualización automática podría romper la compilación o cambiar el comportamiento del motor.

**La solución: versión exacta con `=`.** Al escribir `"=0.58.0"`, le decimos a Cargo: "quiero exactamente esta versión, sin excepción". Cargo jamás actualizará esta dependencia de forma automática. Si queremos cambiar de versión, tenemos que editar el `Cargo.toml` a mano — eso convierte la actualización en una decisión deliberada, no un accidente.

**El código real:**

```toml
# Versión exacta fijada (=) porque la serie 0.x del upstream no garantiza estabilidad entre releases.
nautilus-model = { version = "=0.58.0" }
```

---

### 3. Qué es un workspace Cargo y cómo funciona

**El problema.** Un proyecto real no es un solo archivo Rust — es decenas de componentes que necesitan coexistir, compartir dependencias y compilarse en conjunto sin repetir configuración.

**La solución: workspace.** Un workspace Cargo es un conjunto de crates (librerías o binarios) que comparten el mismo `Cargo.lock` y el mismo directorio de salida (`target/`). Cada crate tiene su propio `Cargo.toml` con su nombre y sus dependencias específicas, pero todos están declarados en el `Cargo.toml` raíz bajo la sección `[workspace]`.

**El código real.** En `/Cargo.toml` (raíz del proyecto):

```toml
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/ingest",
    ...
    "crates/nautilus_compat",  ← lo que añadimos en esta Story
]
```

Cada entrada en `members` es la ruta a un directorio que contiene su propio `Cargo.toml`. Cargo los trata como un conjunto unificado: `cargo build --workspace` compila todos juntos.

**Cómo funciona la resolución de dependencias entre crates del workspace.** Cuando todos los crates del workspace usan la misma versión de una dependencia externa (por ejemplo, `serde`), Cargo solo descarga y compila esa librería una vez. Si dos crates del workspace piden versiones incompatibles de la misma librería, Cargo puede compilar ambas versiones en paralelo — pero eso aumenta el tamaño del binario y puede causar errores de tipos incompatibles si esas versiones se cruzan en la API pública. La clave es mantener versiones consistentes a través del workspace.

---

### 4. Qué es una capa anticorrupción y cómo se implementa en código

**El problema.** Este proyecto depende de NautilusTrader para el motor de ejecución. Pero si cada módulo del proyecto importa tipos de NT directamente (`use nautilus_model::OrderSide` en el crate `ingest`, en el crate `execute`, en el crate `generate`...), entonces toda la base de código queda acoplada a NT. Si mañana cambia la API de NT, o decidimos reemplazarlo por otro motor, habría que modificar decenas de archivos en toda la base de código.

**La solución: capa anticorrupción.** Un solo crate conoce los tipos externos. Todos los demás crates del proyecto importan tipos propios del dominio (que también son propios del proyecto). El crate anticorrupción es el único punto de contacto: si NT cambia, solo cambia ese crate, no el resto.

**El código real.** El crate `nautilus_compat` es la capa anticorrupción stub. Su `lib.rs`:

```rust
//! Solo este crate puede importar tipos NT — ningún otro crate del workspace
//! debe tener `use nautilus_*` directamente.

pub mod stub {
    // Re-exporta el enum AccountType del modelo de dominio de NT v2.
    pub use nautilus_model::enums::AccountType;
}
```

Y la regla se verifica con este comando (debe devolver cero resultados):

```bash
grep -rn "use nautilus" crates/ --include="*.rs" | grep -v nautilus_compat
```

En esta Story el stub solo re-exporta un tipo para confirmar que todo compila. La implementación real (mapeo de tipos NT a tipos propios del dominio) es trabajo de fases futuras.

---

### 5. Tests de integración en `tests/` vs. tests unitarios dentro de `src/`

**El problema.** En Rust hay dos lugares donde se pueden escribir tests. ¿Cuál usar? ¿Por qué importa la diferencia?

**Tests unitarios (`#[cfg(test)]` dentro de `src/`).** Son tests que viven dentro del mismo archivo que el código que prueban. Tienen acceso a los elementos privados del módulo (los que no están marcados como `pub`). Se usan para probar la lógica interna de una función, incluyendo los casos de error que los usuarios externos nunca verían directamente.

**Tests de integración (directorio `tests/`).** Son archivos en el directorio `tests/` al mismo nivel que `src/`. Cada archivo en `tests/` es un programa Rust independiente que solo puede acceder a la API pública del crate (lo que está marcado como `pub`). Esto los convierte en tests que verifican exactamente lo que un usuario externo del crate puede hacer — sin ver los detalles internos.

**Por qué el smoke test de esta Story va en `tests/`.** El smoke test `nautilus_crates_compile_and_basic_type_is_accessible` verifica que la API pública de `nautilus_compat` es accesible desde afuera, exactamente como la usaría cualquier otro crate del workspace. Si lo pusiéramos dentro de `src/lib.rs` con `#[cfg(test)]`, estaríamos dentro del crate y tendríamos acceso interno — no verificaríamos lo mismo.

**El código real.** En `crates/nautilus_compat/tests/smoke_test.rs`:

```rust
#[test]
fn nautilus_crates_compile_and_basic_type_is_accessible() {
    use nautilus_compat::stub::AccountType;  // acceso a través de la API pública
    let _ = std::any::TypeId::of::<AccountType>();
}
```

---

### 6. Qué es `std::any::TypeId` y para qué sirve aquí

**El problema.** Queremos confirmar en un test que el tipo `AccountType` existe y es accesible, pero no queremos instanciarlo (porque es un enum y tendríamos que elegir una variante, lo que no aporta nada al test). Necesitamos una forma de "mencionar" el tipo sin crear un valor.

**La solución: `TypeId::of::<T>()`.** `std::any::TypeId` es una estructura de la librería estándar de Rust que representa la identidad única de un tipo. Cada tipo en un programa Rust tiene un `TypeId` diferente. La función `TypeId::of::<T>()` devuelve el identificador del tipo `T` en tiempo de ejecución.

Lo importante: para llamar a `TypeId::of::<T>()`, el compilador necesita conocer `T` en tiempo de compilación. Si `T` no existe o no está disponible donde lo usamos, el programa no compila — exactamente el error que queremos detectar.

**El código real:**

```rust
use nautilus_compat::stub::AccountType;
let _ = std::any::TypeId::of::<AccountType>();
```

`let _ = ...` descarta el resultado (TypeId en sí no nos importa; solo nos importa que el compilador lo pueda calcular). El test pasa si compila y ejecuta — lo que prueba que el tipo existe, está enlazado correctamente y es accesible desde la API pública.

---

### 7. Qué es `Cargo.lock` y por qué es la fuente de verdad de versiones

**El problema.** `Cargo.toml` declara rangos de versiones (o versiones exactas como en este caso). Pero cuando tienes un equipo de varias personas, o cuando corres el build en CI, quieres garantizar que todos compilan exactamente con las mismas versiones de cada dependencia — hasta las dependencias de tus dependencias (dependencias transitivas).

**La solución: `Cargo.lock`.** Cuando Cargo resuelve las dependencias por primera vez (o cuando corres `cargo update`), escribe en `Cargo.lock` la versión exacta de cada crate que decidió usar, incluyendo todas las dependencias transitivas. La próxima vez que alguien corre `cargo build`, Cargo lee el `Cargo.lock` y usa exactamente esas versiones — sin consultar crates.io de nuevo.

**La diferencia clave:**

| Archivo | Qué declara | Cuándo se modifica |
|---|---|---|
| `Cargo.toml` | Rangos o restricciones de versión | A mano, cuando quieres cambiar una dependencia |
| `Cargo.lock` | Versiones exactas elegidas para este build | Automáticamente por Cargo al resolver |

En esta Story, después de agregar `nautilus-model = "=0.58.0"`, el `Cargo.lock` quedó con estas entradas:

```
name = "nautilus-core"
name = "nautilus-model"
```

Eso confirma que Cargo resolvió la dependencia y fijó las versiones exactas. Cualquier otra persona que clone el repositorio y corra `cargo build` obtendrá exactamente las mismas versiones.

---

### 8. Qué significa `-D warnings` en clippy y por qué lo usamos

**Qué es clippy.** Clippy es el analizador estático oficial de Rust. Detecta patrones de código que, aunque compilan, son potencialmente problemáticos: código muerto, clones innecesarios, condiciones que siempre son verdaderas, uso de APIs deprecadas, etc. Clippy emite sus hallazgos como "warnings" (advertencias) — por defecto no detienen la compilación.

**Qué hace `-D warnings`.** La bandera `-D` significa "tratar como error". `-D warnings` convierte cualquier warning de clippy en un error que detiene la compilación. El comando completo:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

- `--workspace`: analiza todos los crates del workspace, no solo el actual.
- `--all-targets`: analiza tanto el código de producción como los tests.
- `-- -D warnings`: lo que va después de `--` son argumentos que se pasan directamente al compilador; `-D warnings` le dice al compilador que eleve todos los warnings a errores.

**Por qué es útil.** Si un crate externo (como `nautilus-model`) introdujera un tipo deprecado que importamos, clippy nos avisaría con un warning. Con `-D warnings`, ese warning se convierte en un error que impide que el CI pase — forzándonos a decidir conscientemente si usar `#[allow(deprecated)]` con un comentario que explique por qué.

En esta Story, `cargo clippy --workspace --all-targets -- -D warnings` terminó sin ningún error ni warning — confirmando que la integración de `nautilus-model 0.58.0` no introduce problemas de calidad de código en el proyecto.

---

## Trucos de Senior

**Versión exacta con `=` para dependencias de ecosistemas inestables.** La sintaxis `{ version = "=X.Y.Z" }` en Cargo te da reproducibilidad total para dependencias `0.x` que no garantizan estabilidad de API entre versiones menores. Es el equivalente de fijar un commit hash en npm.

**El grep de aislamiento como test estructural.** El comando:
```bash
grep -rn "use nautilus" crates/ --include="*.rs" | grep -v nautilus_compat
```
no es un test de Rust — es una verificación estructural que se corre en CI como parte de los criterios de aceptación. Si devuelve cualquier resultado, significa que la regla de la capa anticorrupción fue violada. Es barato, directo y funciona aunque el compilador no lo detecte.

**`TypeId::of::<T>()` como test de compilación.** Cuando quieres confirmar que un tipo es accesible sin instanciarlo, `std::any::TypeId::of::<T>()` es la herramienta idiomática. Si el tipo no existe o no está en scope, el programa no compila — sin necesidad de constructores ni fixtures.
