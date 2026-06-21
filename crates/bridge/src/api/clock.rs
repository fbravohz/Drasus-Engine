//! Funciones FFI para el observable "Reloj" del Panel Operativo.
//!
//! Expone el timestamp del [`shared::SystemClock`] hacia Dart.
//! El reloj de Drasus es monótono no decreciente: nunca devuelve un valor
//! menor al anterior, aunque el reloj del sistema operativo salte hacia
//! atrás por una corrección NTP.

use flutter_rust_bridge::frb;
// Clock es el trait (puerto) que define timestamp_ns().
// SystemClock es la implementación de producción que lee el reloj del SO.
// Ambos viven en public_interface.rs — la única superficie pública de shared.
use shared::public_interface::{Clock, SystemClock};

/// Retorna el timestamp actual del reloj de producción de Drasus en
/// nanosegundos desde el Unix epoch (1970-01-01 00:00:00 UTC).
///
/// **Tipo de retorno:** `i64`. El reloj interno usa `i64` porque SQLite
/// almacena los timestamps como enteros con signo, y se mantiene el mismo
/// tipo en toda la pila para evitar conversiones con pérdida.
///
/// **Garantía de ownership:** `i64` es un primitivo de tamaño fijo copiado
/// por valor al cruzar la frontera — no hay heap involucrado, no hay memoria
/// que liberar en ninguno de los dos lados.
///
/// **`#[frb(sync)]`:** indica a flutter_rust_bridge que esta función es
/// síncrona — Dart la llama y recibe el valor en el mismo frame, sin
/// necesidad de `await`. Solo se usa para operaciones que no hacen I/O
/// (no van a disco ni a red) y retornan en microsegundos.
#[frb(sync)]
pub fn get_clock_timestamp_ns() -> i64 {
    // SystemClock::new() es barato: solo inicializa un AtomicI64.
    // La llamada a timestamp_ns() lee SystemTime::now() del SO y aplica
    // el clampeo monótono para evitar retrocesos de reloj.
    SystemClock::new().timestamp_ns()
}
