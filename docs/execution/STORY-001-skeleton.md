# STORY-001 · Esqueleto del proyecto

| Campo | Valor |
|---|---|
| **ID** | STORY-001 |
| **Título** | Esqueleto del proyecto (workspace Cargo, 8 módulos + shared) |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 0 |
| **Estado** | ✅ Implementado |
| **Responsable** | Rust-Engineer (Sonnet) · auditó Tech-Lead |
| **Creada** | 2026-06-12 |
| **Completada** | 2026-06-12 |

## 1. Especificación de origen
- **ADR(s):** ADR-0003 (organización FCIS de módulos + features reutilizables)
- **Fuente:** ROADMAP §EPIC-0, Sprint 0, fila STORY-001
- **SAD:** §4.2 (estructura de crate)

## 2. Objetivo (una frase llana)
Montar el armazón del programa: un workspace Cargo con las 8 piezas del sistema como crates internos + `shared`, cajas vacías que compilan, sin lógica.

## 3. Instrucciones de despacho (la spec ejecutable)
```
Eres el Rust-Engineer de Drasus Engine. El Tech-Lead te despacha la tarea STORY-001 de la Épica 0 (Fundación).

PASOS DE ARRANQUE: 1) Lee base/SKILL.md y declara que lo aplicarás. 2) Lee rust-engineer/SKILL.md y
adopta el rol. 3) Lee ADR-0003 en docs/ADR.md (patrón FCIS + organización de módulos). 4) Lee ROADMAP
§EPIC-0 Sprint 0 fila STORY-001 para el criterio de cierre.

ORDEN: Crear el workspace Cargo en la raíz del repo (hoy solo hay docs/ y .claude/).
- 8 crates internos: ingest, generate, validate, incubate, execute, manage, feedback, withdraw.
- Más una carpeta/crate común shared.
- Cada crate: CAJA VACÍA con public_interface declarada pero SIN lógica, patrón FCIS de ADR-0003
  (núcleo puro + cáscara delgada; CERO lógica en orquestadores).
- Debe COMPILAR: cargo build y cargo test en verde.

LÍMITES: Solo el esqueleto. NO base de datos ni migraciones (es STORY-002). NO lógica de negocio. NO inventes
campos/contratos/dependencias no justificadas por ADR-0003 o el ROADMAP. Si algo es ambiguo, repórtalo
como bloqueo. Código en inglés.

ENTREGABLE: 1) árbol de archivos; 2) salida de cargo build/test; 3) cómo respetaste FCIS; 4) ambigüedades.
```

## 4. Criterio de aceptación
- `cargo build` y `cargo test` verdes en el esqueleto.
- Estructura FCIS: cero lógica de negocio en cáscaras/orquestadores.
- Los 8 módulos + `shared` presentes como crates internos.

## 5. Comandos de validación (para el usuario — copy/paste)
```bash
cd /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine
cargo build --workspace          # debe terminar sin errores ni warnings
cargo test --workspace           # 9 crates, 1 test "crate_compiles_and_links" cada uno
find crates -name Cargo.toml | sort   # confirma 9 crates (8 módulos + shared)
```

## 6. Registro de ejecución (bitácora)
- 2026-06-12 · Rust-Engineer (Sonnet) · **APROBADO** · Auditoría Tech-Lead: `cargo build` 0 warnings, `cargo test` 9/9 verdes, FCIS verificado por inspección (`orchestrator.rs` y `domain/logic.rs` vacíos con solo documentación de su rol). Estructura según SAD §4.2.

## 7. Pendientes derivados / decisiones
- **Crate binario raíz `app`** (archivo principal de orquestación, SAD §4.2): NO se creó (criterio literal de STORY-001 no lo pedía). Decisión Tech-Lead: se crea en **STORY-009** (CLI), su hogar natural. No es deuda, es secuenciación.
