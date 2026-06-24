//! Imperative Shell — adaptadores de infraestructura.
//!
//! Implementa los puertos declarados en `public_interface` usando los
//! servicios de `shared` (pool SQLite, clock, job executor).
//! Aquí vive el código que toca I/O: consultas SQL, FFI, sistema de archivos.
