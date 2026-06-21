//! [SHELL] Crate `bridge` — capa de comunicación FFI entre el Core Rust de
//! Drasus Engine y la interfaz Flutter/Dart.
//!
//! Este crate actúa como la "frontera": expone funciones Rust que Dart puede
//! llamar directamente, usando `flutter_rust_bridge` para generar los bindings
//! Dart de forma automática. No contiene lógica de negocio — solo traduce
//! entre el mundo Rust y el mundo Dart, delegando toda operación real a
//! `shared` (ADR-0003, ADR-0097).
//!
//! ## Tipos de retorno en la frontera FFI
//!
//! `flutter_rust_bridge` solo puede transportar tipos que ambos lados
//! entiendan: primitivos (`i64`, `u64`, `f64`, `bool`, `String`) y structs
//! cuyos campos sean todos primitivos o `String`. Cualquier enum o tipo
//! complejo de Rust debe convertirse a String antes de cruzar.
//!
//! ## Funciones síncronas vs asíncronas
//!
//! - `#[frb(sync)]` → la función se llama directamente desde el hilo de
//!   Flutter y devuelve el valor de forma inmediata.
//! - Sin atributo (async fn) → flutter_rust_bridge la expone como `Future<T>`
//!   en Dart; Flutter espera el resultado sin bloquear su hilo principal.

// Bindings FFI generados por flutter_rust_bridge_codegen — no editar a mano.
mod frb_generated;
// Módulo con las funciones públicas expuestas a Flutter (reloj, trabajos, auditoría).
pub mod api;
