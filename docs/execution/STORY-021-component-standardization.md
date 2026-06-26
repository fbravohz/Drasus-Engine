# STORY-021 · Estandarización total de la biblioteca de componentes (4 lotes)

| Campo | Valor |
|---|---|
| **ID** | STORY-021 |
| **Título** | Estandarización total de la biblioteca de componentes + arreglo de bugs de interacción |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | UI / Galería — Estandarización |
| **Estado** | ✅ Implementado |
| **Responsable** | Flutter-Engineer (Sonnet, varios lotes) · auditó Tech-Lead + QA |
| **Creada** | 2026-06-25 |
| **Completada** | 2026-06-25 |

## 0. Resumen ejecutivo
- **Problema:** ~160 componentes de la galería están desigualmente estandarizados (colores/radios/padding hardcodeados, bordes con glow fijo en vez del énfasis, texto que no respeta el color base, bugs de interacción al hacer clic). Es una biblioteca de producción que debe quedar uniforme.
- **Qué se construye:** se normaliza ABSOLUTAMENTE TODO componente contra el contrato de STORY-020 (tokens dinámicos, N modos, énfasis en bordes/títulos, color de fuente base, espaciado), se añaden comentarios de bloque en español, y se arreglan los bugs de interacción a alcance de cáscara visual.
- **Por qué ahora:** completa la migración que ADR-0138/STORY-019 dejó a medias; deja la biblioteca lista para enchufarle lógica después.

## 1. Especificación de origen
- **Spec de origen:** plan `.claude/plans/estamos-teniendo-problemas-importantes-hazy-cloud.md` §Fase B + §Fase C.
- **ADR(s):** ADR-0138 + enmienda 2026-06-25, ADR-0139, ADR-0121.

## 2. Objetivo (una frase llana)
Que cada componente de la biblioteca tome su estilo del provider (con override interno cuando haga falta), reaccione a todos los modos sobre fondo claro y oscuro, esté comentado en español y no tenga comportamientos raros al interactuar.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)
| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Flutter-Engineer — Lote 1 | Etapa 4 | STORY-020 congelada | **Autónomo** |
| Flutter-Engineer — Lote 2 | Etapa 4 | STORY-020 congelada | **Autónomo** |
| Flutter-Engineer — Lote 3 | Etapa 4 | STORY-020 congelada | **Autónomo** |
| Flutter-Engineer — Lote 4 | Etapa 4 | STORY-020 congelada | **Autónomo** |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | los 4 lotes | **Autónomo** |

## 4. Instrucciones de despacho por agente

### Prompt común a los 4 lotes (cambia solo la lista de archivos del lote)
```
Eres el Flutter-Engineer de Drasus Engine. Antes de tocar nada:
1. Lee .claude/skills/base/SKILL.md COMPLETO y declara que lo aplicas.
2. Lee .claude/skills/flutter-engineer/SKILL.md COMPLETO.

CONTRATO DE TOKENS CONGELADO (STORY-020) — úsalo, no lo cambies:
- Superficie: usa wrappers frosted()/panelSurface()/cardSurface()/glassEnhanced()/PanelFromDecoration; NUNCA Color sólido suelto; NUNCA const en superficies. Reaccionan a los N modos (glass/tint/solid/enhancedGlass + futuros).
- Color de fondo: getters Gx.surfaceFill/surfacePanel/surfaceCard. Prohibido raws (Gx.glassFill/panelSolid/cardInner) en widgets.
- Texto base: Gx.textBase / textBaseSecondary / textBaseLabel / textBaseMuted. Prohibido Colors.white/black o hex para texto normal.
- Énfasis: bordes estructurales globales y títulos/subtítulos usan Gx.borderBase / Gx.accentDynamic (el énfasis). Los colores semánticos (óptimo/alerta/crítico) SOLO para señalizar estado dentro del componente, vía parámetro.
- Radios: Gx.rPanel/rButton/rInput/rChip. Único literal permitido: 999 (pills).
- Espaciado: escala Gx.space4..space64 para padding y margen.
- Glow/gradientes: Gx.glow()/glowStrong()/linear() con el color correcto.

ALCANCE DE TU LOTE (archivos): <LISTA DEL LOTE>

TAREA por CADA componente de tus archivos (cobertura 100%, prohibido muestrear):
1. Empieza generando en tu reporte la CHECKLIST NOMINAL de TODOS los componentes/clases de tus archivos. Ninguno se cierra sin marca.
2. Reemplaza todo color/ radio/ padding/ margen hardcodeado por el token correspondiente.
3. Toda superficie visible pasa por un wrapper y reacciona a los modos.
4. Bordes globales y títulos/subtítulos → énfasis. Estados → solo señalización interna.
5. Texto normal → Gx.textBase (verifica legibilidad sobre fondo claro 'paper' y oscuro 'bunker').
6. Comentario de bloque en español antes de cada widget/clase: qué hace, qué parámetros recibe, qué tokens consume.
7. Deja cada componente parametrizable (props con defaults), listo para reutilizar.
8. ARREGLA BUGS DE INTERACCIÓN: prueba clic/hover/foco/gestos. Corrige estados que no se resetean, gestos que disparan de más o no responden, hover/foco pegados, animaciones cortadas, áreas de tap mal puestas. NO añadas lógica de negocio; si el bug depende de la capa de lógica futura, anótalo como pendiente, no lo inventes.

RESTRICCIONES:
- Identificadores en inglés; comentarios en español (ADR-0121).
- No cambies el contrato de STORY-020; consúmelo.
- Entrega con `flutter analyze` limpio. NO toques archivos fuera de tu lote.

ENTREGA: la checklist nominal con cada componente marcado + resumen de hardcodes eliminados + bugs de interacción corregidos (y los anotados como pendientes) + confirmación de analyze verde.
```

### 4.1 Lote 1
Archivos: `section_inputs_extended.dart`, `section_buttons_extended.dart`, `section_std_missing.dart`.

### 4.2 Lote 2
Archivos: `section_nav.dart`, `section_feedback_extended.dart`, `section_data_display_extended.dart`.

### 4.3 Lote 3
Archivos: `section_dataviz_new.dart`, `section_dataviz_quant.dart`, `section_dataviz_extended.dart`.

### 4.4 Lote 4
Archivos: `section_dag_nodes.dart`, `section_animations.dart`, `section_trade_tape.dart`, `section_drasus_core_extended.dart`, + widgets de `gallery_fx.dart`/`gallery_painters.dart` no cubiertos por STORY-020.

## 5. Criterio de aceptación (cada criterio ↔ su prueba)
| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | Cobertura 100%: todo componente en la checklist nominal, tocado | confronto checklist de los 4 lotes vs código |
| 2 | Cero hardcodes de color/radio en `ui/lib/gallery/` (salvo 999 y raws en gallery_tokens.dart) | `grep` de `Colors.white\|Colors.black\|Color(0x` y `BorderRadius.circular(` literal |
| 3 | Todo componente reacciona a los N modos sobre fondo claro y oscuro | verificación visual matricial (modos × paletas bunker/paper) |
| 4 | Texto legible en `paper`; bordes/títulos siguen el énfasis | `flutter run`, cambiar paleta y énfasis |
| 5 | Comentario de bloque en español por widget | inspección + grep |
| 6 | Sin bugs de interacción residuales (o anotados como pendiente justificado) | ejercer clic/hover/foco/gestos en la app |
| 7 | `flutter build linux` verde | salida del comando |

## 6. Comandos de validación (para el usuario — copy/paste)
```bash
cd ui
flutter analyze
flutter build linux
flutter run -t lib/gallery/gallery_preview_main.dart
# Recorrer TODA la galería en modo glass mejorado y en paleta paper; verificar legibilidad,
# bordes en énfasis, y que ningún componente haga cosas raras al hacer clic.
grep -rnE "Colors\.(white|black)|Color\(0x" ui/lib/gallery/sections/   # esperado: ~0
```

## 7. Registro de ejecución
- 2026-06-25 · 4 Flutter-Engineers (Sonnet, Autónomo, paralelo) · 1ª tanda · cortada por límite de sesión a media tarea; el árbol quedó compilable. Diagnóstico en disco mostró cobertura desigual.
- 2026-06-25 · 3 Flutter-Engineers (Sonnet, paralelo) · re-tanda sobre lo incompleto (re-lotes A/B/C) · todas las 13 secciones estandarizadas; checklists nominales entregadas.
- 2026-06-25 · QA-Engineer (Sonnet) · **NO APTO** · hallazgo clave que el grep de literales no detecta: ~19 `Gx.borderPanel` (estático) en bordes de chrome (debían ser `Gx.borderBase`), tokens de texto estáticos en chrome de navegación, 3 radios reales sin tokenizar, comentarios de excepción faltantes.
- 2026-06-25 · Flutter-Engineer (Sonnet) · remediación B1-B5 aplicada (borderPanel→borderBase / divider; textos chrome→textBase*; radios→token; comentarios de excepción).
- 2026-06-25 · Tech-Lead · **APROBADO** · verificación independiente reproducida: `flutter build linux --debug` verde; `grep` en `lib/gallery/sections/` → 0 `Gx.borderPanel`, 0 tokens de texto estáticos de chrome (el `textMuted` restante son colores de DATO en painters), `Colors.white/black`+`Color(0x)` solo en máscara `BlendMode.dstIn` y defaults parametrizados de demo (justificados). 3 residuos `Gx.textLabel` en etiquetas off de botones corregidos por el Tech-Lead en el cierre. Cobertura 100% confirmada contra checklists nominales de los lotes.

## 8. Pendientes derivados / decisiones
- Bugs que requieran la capa de lógica futura → se anotan aquí por componente cuando aparezcan.
- Actualización de skills (flutter-engineer / qa-engineer) con la disciplina de estandarización + vigilancia de bugs → tarea de cierre de esta fase.
