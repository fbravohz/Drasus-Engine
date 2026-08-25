// verification_bridge.dart — Punto de cableado FFI del Banco de Verificación.
//
// El binding real ya aterrizó en `ui/lib/src/rust/api/verification.dart`
// (generado por flutter_rust_bridge, Bridge-Engineer + `dart run
// build_runner build` para materializar la unión freezed `InputStatus`).
// Este archivo es un re-export delgado: el resto del Banco (registry,
// sección genérica, tab maestro-detalle) importa SOLO este punto — así una
// futura regeneración del binding (nueva feature, firma ajustada) solo toca
// este archivo si el nombre del export cambia.
export '../../src/rust/api/verification.dart';
