# Lecciones — Aprendizaje Acumulado por Story/Task

Esta carpeta no documenta el proyecto: documenta lo que el usuario aprende mientras lo construye (Protocolo de Lecciones, ADR-0122, corregido por [ADR-0124](../adr/ADR-0124.md)). Cada archivo es una Story/Task concreta — el mismo ID que su Orden en `docs/execution/` — y consolida TODO lo enseñado durante esa Story, anclado a su código real. Cualquier skill puede leer y escribir aquí bajo Modo Mentor, Revisión o Docente (ADR-0120/ADR-0122).

## Reglas (detalle completo en `.claude/skills/base/SKILL.md`)

- Un archivo por Story/Task, nombrado igual que su Orden (`STORY-007-telemetry.md`), nunca por tema de lenguaje suelto.
- Cada concepto explicado dentro del archivo cita el código real de esa Story (ruta + fragmento), no un ejemplo de manual.
- El archivo enlaza a su Orden en `docs/execution/<ID>.md` al inicio — trazabilidad en ambos sentidos.
- Si la misma Story se retoma en una sesión posterior, no se crea un segundo archivo: se añade al ya existente, debajo de lo escrito.
- Cada archivo tiene como mínimo `## Concepto` (una subsección por cada concepto enseñado, cero-conocimiento, anclada a código real) y `## Trucos de Senior` (atajos/azúcar sintáctica reales de esa Story — solo si hay algo real que destacar).

## Carpetas por dominio

| Carpeta | Ingeniero(s) que escriben aquí | Dominio |
|---|---|---|
| [`rust/`](./rust/) | `rust-engineer`, `refactoring-engineer` | Lenguaje Rust, Cargo, build/release |
| [`dart-flutter/`](./dart-flutter/) | `flutter-engineer` | Dart, Flutter, Impeller, `CustomPainter` |
| [`ffi-grpc/`](./ffi-grpc/) | `bridge-engineer` | FFI (`flutter_rust_bridge`), Arrow, Protobuf, gRPC |
| [`quant/`](./quant/) | `quant-engineer` | Estadística, matemática financiera, sesgos |
| [`testing/`](./testing/) | `qa-engineer` | Patrones de prueba (unitaria, propiedad, oráculo, SLA) |
