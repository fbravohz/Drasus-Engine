# Lecciones — Aprendizaje Acumulado por Tema

Esta carpeta no documenta el proyecto: documenta lo que el usuario aprende mientras lo construye (Protocolo de Lecciones, ADR-0122). Cada archivo es un tema del lenguaje, framework o disciplina — nunca una tarea, Historia u Orden de Trabajo. Cualquier skill puede leer y escribir aquí bajo Modo Mentor, Revisión o Docente (ADR-0120/ADR-0122).

## Reglas (detalle completo en `.claude/skills/base/SKILL.md`)

- Un archivo por tema, nombrado por el concepto (`ownership.md`, `async-await.md`), nunca por la tarea donde se enseñó.
- Si el archivo ya existe, no se reescribe: se añaden las líneas/secciones nuevas debajo de lo ya escrito.
- Cada archivo tiene como mínimo `## Concepto` (explicación cero-conocimiento) y `## Trucos de Senior` (atajos/azúcar sintáctica — solo si hay algo real que destacar).

## Carpetas por dominio

| Carpeta | Ingeniero(s) que escriben aquí | Dominio |
|---|---|---|
| [`rust/`](./rust/) | `rust-engineer`, `refactoring-engineer` | Lenguaje Rust, Cargo, build/release |
| [`dart-flutter/`](./dart-flutter/) | `flutter-engineer` | Dart, Flutter, Impeller, `CustomPainter` |
| [`ffi-grpc/`](./ffi-grpc/) | `bridge-engineer` | FFI (`flutter_rust_bridge`), Arrow, Protobuf, gRPC |
| [`quant/`](./quant/) | `quant-engineer` | Estadística, matemática financiera, sesgos |
| [`testing/`](./testing/) | `qa-engineer` | Patrones de prueba (unitaria, propiedad, oráculo, SLA) |
