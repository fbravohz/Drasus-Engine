# STORY-050 · Banco de Pruebas genérico + conexión transversal de todo lo ya desarrollado

> **Plantilla de Orden de Trabajo (Spec-Driven).** La Orden de Trabajo es la **especificación ejecutable**: contiene la instrucción EXACTA que recibió cada agente, los comandos para que el usuario valide por su cuenta, y el registro de lo que pasó. Vive en git, NO en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-050 |
| **Título** | Banco de Pruebas genérico (harness SVF) + enchufe de las 14 features del substrato |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación (cierre de Definición de Terminado del substrato) |
| **Sprint** | Sprint "Banco de Pruebas" (tanda de UI del substrato — paga DEBT-005) |
| **Estado** | ✅ Implementado |
| **Responsable** | Architect (Sonnet) · UI-Designer (Sonnet) · Bridge-Engineer (Sonnet) · Flutter-Engineer (Sonnet) · auditó QA-Engineer + Tech-Lead |
| **Creada** | 2026-07-12 |
| **Completada** | 2026-07-12 |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** los 14 cimientos del substrato de monetización solo se pueden probar por terminal (`drasus verify`, Canal #2); el propietario (perfil frontend) no tiene forma de probarlos dentro de la app. Se rastreaba como DEBT-005.
- **Qué se va a construir:** (1) el **Banco de Pruebas genérico** (harness SVF que se construye una vez: selector de feature + input JSON precargado + botón Enviar + respuesta real por FFI, con indicador **visual de validez del input** — bien/mal formado); (2) el **contrato FFI genérico** `verify_feature` que despacha a cualquier `verify_*` del backend; (3) el **enchufe de las 14 features** ya desarrolladas en el registro extensible.
- **Por qué ahora:** el substrato cerró 14/14 y la auditoría EPIC-0 terminó → el disparador de DEBT-005 ("al cerrar los backends del substrato") ya se cumplió. Además esta Story instituye el **gate permanente de Definición de Terminado** (ADR nuevo): ninguna feature se considera Terminada sin su conexión al Banco de Pruebas.

---

## 1. Especificación de origen (qué specs implementa)
- **Feature(s):** [`verification-bench`](../features/verification-bench.md) — **creada por el Architect en esta misma Story** (spec viva).
- **ADR(s):** ADR nuevo (siguiente libre = **ADR-0152**; el propietario lo mencionó como "0170", pero el más alto real es 0151) — eleva el Banco a gate de DoD y enmienda **ADR-0117** (SVF). Reconcilia con memorias `verification-surface-svf` y `feedback-svf-galeria-transversal`.
- **Deuda que salda:** DEBT-005.

## 2. Objetivo (una frase llana)
Que el propietario pueda probar cualquier feature ya construida dentro de la app —metiendo un input, viendo si está bien o mal formado, y viendo la respuesta real del backend— sin leer una línea de código.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Architect | Etapa 0 — spec (feature doc + ADR-0152 + impacto SAD/README/ROADMAP) | ninguno | Autónomo |
| UI-Designer | Etapa 0.5 — Cáscara Visual (incl. indicador de validez de input) | Architect (feature doc) | Autónomo |
| Bridge-Engineer | Etapa 2 — contrato FFI genérico `verify_feature` + codegen | ninguno (contrato fijado en §4) | Autónomo |
| Flutter-Engineer | Etapa 3 — harness genérico UI + enchufe de las 14 features | Bridge (binding, contrato fijado) | Autónomo |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | **Bridge + Flutter** | **Autónomo** |

> El General-Counsel NO participa: no hay exposición legal en herramienta de verificación interna (decisión del propietario 2026-07-12).

## 4. Instrucciones de despacho por agente (la spec ejecutable)

> Los cuatro primeros agentes fueron despachados en paralelo el 2026-07-12 (background, Sonnet). El QA se despacha tras la entrega de Bridge + Flutter. Los prompts exactos:

### 4.1 Architect
```
Formalizar el Banco de Pruebas (Verification Bench) como feature de primera clase + gate de Definición
de Terminado. Entregables: (1) docs/features/verification-bench.md siguiendo la plantilla FEATURE.md
(modelo de 3 zonas: input JSON precargado izq / botón Enviar centro / respuesta FFI read-only der;
registro extensible; requisito NUEVO de validez-de-input visual; contrato FFI genérico
verify_feature(feature_id, input_json) -> {input_status: Valid|Invalid(reason), ok, output_json, error};
Cáscara Visual dejada como marcador para el UI-Designer). (2) ADR nuevo con el siguiente número libre
(ADR-0152, NO 0170) que eleva el Banco a gate permanente de DoD ("ninguna feature Terminada sin conexión
al Banco de Pruebas") y enmienda ADR-0117; registrarlo en docs/ADR.md. (3) Impacto documental: enmendar
ADR-0117, impactar el SAD pertinente, añadir la feature al índice docs/README.md, reflejar el sprint en
ROADMAP. Contraste bidireccional. Solo docs, no código. Edición quirúrgica.
```

### 4.2 UI-Designer
```
Diseñar la Cáscara Visual del Banco de Pruebas. Leer el chasis existente (verification_bank_tab.dart,
verification_registry.dart) y los tokens/componentes (gx_tokens.dart, surfaces.dart, components/ alias
custom_ui). Modelo de 3 zonas + selector de feature. REQUISITO CLAVE: diseñar visualmente el indicador de
validez del input (input válido vs input inválido: <razón>) con estados idle/enviando/éxito/input-inválido/
error de backend. Entregar la sección "Cáscara Visual" (plantilla Paso 4) como mensaje final; NO editar la
feature doc (la crea el Architect en paralelo). Solo componentes/tokens que existen; gaps declarados, no
inventados.
```

### 4.3 Bridge-Engineer
```
Construir el contrato FFI GENÉRICO del Banco. Nuevo crates/bridge/src/api/verification.rs con
verify_feature(feature_id: String, input_json: String) -> VerificationOutcome { input_status:
Valid|Invalid{reason}, ok: bool, output_json: String, error: Option<String> }. Semántica: valida primero
que input_json esté bien formado y con la estructura que la feature espera (si no, Invalid{reason} sin
ejecutar backend); si es válido, despacha a la verify_* correspondiente de shared::public_interface y
devuelve el round-trip real. Mecanismo de despacho feature_id -> verify_* (enum/tabla en el lado Rust,
reutilizando lo existente). list_verifiable_features() -> Vec<FeatureDescriptor> (id + nombre + JSON de
ejemplo). Registrar en mod.rs, correr codegen FRB + post-codegen; binding en ui/lib/src/rust/api/
verification.dart. cargo build verde + pruebas discriminantes de dispatcher y validación de input. Dueño
del codegen; NO tocar ui/lib/tabs/.
```

### 4.4 Flutter-Engineer
```
Convertir el chasis del Banco en el harness GENÉRICO y enchufar todas las features desarrolladas. Sección
genérica reutilizable de 3 zonas (editor JSON input izq / botón Enviar centro / respuesta FFI read-only der)
que consume verifyFeature + listVerifiableFeatures (binding del Bridge; contrato fijo; NO correr codegen).
Indicador VISUAL de validez de input (válido / inválido: <razón>) + estados idle/enviando/éxito/input-
inválido/error. Enchufar en kVerificationRegistry las 14 features del substrato + plomería/fetcher que falte
(cada una ~una línea al usar la sección genérica). Solo componentes/tokens existentes (custom_ui, Gx). Thin
Shell. flutter build linux verde. NO tocar crates/.
```

### 4.5 QA-Engineer (a despachar tras Bridge + Flutter)
```
Gate obligatorio. Revisar la lógica antes de correr tests (qa-engineer §1c). Verificar: (a) la validación de
input rechaza JSON malformado y estructura incorrecta con razón, SIN ejecutar el backend; (b) el despacho
llega a la verify_* correcta de cada feature; (c) el harness genérico muestra los 5 estados; (d) las 14
features están enchufadas y responden el round-trip real por FFI; (e) flutter build linux + cargo build
verdes. Prueba de 2 escritores NO aplica (no hay ledger nuevo). Veredicto APTO/NO APTO.
```

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | `verify_feature` con JSON malformado o estructura incorrecta devuelve `Invalid{reason}` sin ejecutar backend | test Rust `verify_feature_rejects_malformed_input` |
| 2 | `verify_feature` con input válido despacha a la `verify_*` correcta y devuelve el round-trip real | test Rust `verify_feature_dispatches_and_runs` |
| 3 | `list_verifiable_features` enumera las 14 features + plomería con su JSON de ejemplo | test Rust `list_verifiable_features_covers_substrate` |
| 4 | El harness genérico muestra los 5 estados (idle/enviando/éxito/input-inválido/error) | widget test / verificación manual del TL |
| 5 | Las 14 features del substrato quedan enchufadas en `kVerificationRegistry` | grep del registro + `flutter build linux` verde |
| 6 | El indicador visual de validez del input distingue claro válido vs inválido | verificación del TL contra la Cáscara Visual del UI-Designer |

## 6. Comandos de validación (para el usuario — copy/paste)
```bash
# Backend + contrato FFI
cargo test -p bridge
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings

# UI
cd ui && flutter build linux && flutter analyze

# Probar a mano en la app: abrir el tab "Verificación", elegir una feature,
# meter un input malo (ver que marca "input inválido") y uno bueno (ver la respuesta real).
```

## 7. Registro de ejecución (bitácora cronológica)
- 2026-07-12 · Tech-Lead · Orden creada; despachados en paralelo (background, Sonnet): Architect, UI-Designer, Bridge-Engineer, Flutter-Engineer.
- 2026-07-12 · Architect (Sonnet) · APROBADO · feature `verification-bench.md` + ADR-0152 (gate permanente de DoD) + enmienda ADR-0117 + SAD-06 + README + ROADMAP. Contrato alineado con el del Bridge sin fricción.
- 2026-07-12 · UI-Designer (Sonnet) · APROBADO · Cáscara Visual con distinción rojo (input inválido) / ámbar (error de backend); 2 gaps de catálogo anotados (§8).
- 2026-07-12 · Bridge-Engineer (Sonnet) · APROBADO · `verify_feature` + `list_verifiable_features`, despacho a 15 features, validación de input tipada previa al backend. **Verificación independiente del TL:** `cargo test -p bridge` 6/6 verde, `cargo build --workspace` verde.
- 2026-07-12 · Flutter-Engineer (Sonnet) · APROBADO (con 1 corrección) · harness genérico de 3 zonas + 15 features enchufadas + `flutter build linux` verde.
- 2026-07-12 · QA-Engineer (Sonnet) · **NO APTO → APTO** · cazó bug bloqueante que ningún test cubría: `invalidInput` y `backendError` se pintaban idénticos en rojo, violando la regla FIJO de ADR-0152. **Fix del TL** en `generic_verification_section.dart` (caso `backendError`: `ChipStatus.alert` + `BannerType.warning` = ámbar; `invalidInput` se mantiene rojo); `flutter analyze` limpio. QA reverificó → **APTO**.

## 8. Pendientes derivados / decisiones
- El adaptador de red a la Cabina de Mando sigue diferido (solo el cable final; toda la fontanería local se prueba ya por FFI).
- **Gaps de catálogo detectados por el UI-Designer (2026-07-12, no bloquean):** (1) `custom_ui.Textarea` sin variante monoespaciada → el editor de JSON se ve en fuente sans; recomendado extender con `monospace: bool`. (2) No existe `custom_ui.CodeBlock` funcional (`GlowCode` sigue siendo showcase-only en galería, no migrado) → se usa `SelectableText` + `Gx.dataMono` como solución inmediata válida. Ambos → posible mini-Story de librería.
- **Decisión de diseño del UI-Designer (adoptada):** "input inválido" (rojo `criticalCrimson`) y "error de backend" (ámbar `alertAmber`) son estados visualmente distintos — uno se corrige editando el JSON, el otro no; usar el mismo color engañaría.
- La sección a-mano del `sovereign-data-fetcher` se migra al patrón genérico salvo que aporte visualización extra justificable.

## 9. Cierre ejecutivo (para el usuario — CEO)

**ESTADO:** ✅ Cerrada (QA APTO, 2026-07-12). El Banco de Pruebas existe y funciona: puedes abrir el tab "Verificación", elegir cualquiera de las 15 features (los 14 cimientos + el descargador), meter un input y ver la respuesta real del backend por FFI. Si tu JSON está mal, lo marca en **rojo** con el motivo (lo corriges tú); si el sistema falla con un input válido, lo marca en **ámbar** (no es tu dato) — esa distinción es la regla que el QA hizo respetar.

**PROGRESO MACRO:** se instituyó la regla permanente (ADR-0152): ninguna feature está Terminada sin su conexión al Banco. Con esto, las 14 features del substrato que antes solo se probaban por terminal ya son verificables por ti en la app. Se saldó DEBT-005.

**FRICCIONES Y DEUDA:** el QA cazó un bug de color que ningún test automático cubría (se arregló). Queda una observación menor: no hay test de widget que fije la distinción rojo/ámbar, así que una regresión futura de ese color no la atraparía una prueba → registrado como DEBT-022 (baja, cobertura). Dos gaps menores del catálogo de componentes (editor sin fuente monoespaciada; sin `CodeBlock` funcional) anotados en §8.

**INPUT REQUERIDO DEL CEO:** autorizar los commits agrupados por tipo (feature+ADR, Banco de Pruebas, split de deuda, correcciones). El push a `origin` sigue pendiente de tu OK aparte.
