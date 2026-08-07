# DEBT-037 · El mapa de componentes del documento de diseño apunta a clases que ya no existen

| Campo | Valor |
|---|---|
| **ID** | DEBT-037 |
| **Severidad** | 🟠 Media |
| **Estado** | ✅ Pagada (STORY-073, 2026-07-31) |
| **Abierta** | 2026-07-26 (hallazgo colateral del UI-Designer en la Etapa 0.5 del asistente de importación) |
| **Pagada** | 2026-07-31 — barrido completo de las 127 filas del mapa contra el código real (criterio 12 de [STORY-073](../execution/STORY-073-asistente-importacion-pantalla.md)) |
| **Documento** | `docs/DESIGN.md` — secciones del "Component Map" |
| **Origen** | STORY-025/026 renombraron y movieron la librería de componentes; el mapa del documento de diseño no se actualizó |

## Qué queda debiendo

El "Component Map" de `docs/DESIGN.md` sigue nombrando las clases con el prefijo `Glow*` que vivían en la galería. **Ninguna de esas clases existe hoy:** STORY-025 las reescribió como clases desnudas en `ui/lib/components/` y STORY-026 movió sus cimientos a `theme/`. El mapa quedó apuntando a un vocabulario retirado.

El UI-Designer corrigió **solo las filas que su propio diseño necesitaba** (unas veinticinco, cada una con la nota de qué corregía). Quedan **56 menciones** del prefijo viejo en el resto del documento — filas de componentes que ningún diseño ha necesitado todavía.

## Impacto actual

**Bajo pero real, y creciente.** El documento de diseño es la fuente que un diseñador consulta para elegir componentes. Cada fila obsoleta es una invitación a especificar un componente por un nombre que no existe — y el diseñador siguiente lo descubre al verificar, si verifica, o lo pasa al ingeniero, que lo descubre al compilar.

Ya ocurrió una vez con consecuencias caras: la galería fue durante meses un catálogo de adornos que se documentaba como si fuera una librería usable, y eso costó una Story entera de rescate (STORY-025). **Este es el mismo patrón en el documento en vez de en el código:** el mapa promete un vocabulario que la librería no habla.

## Por qué se dejó a medias, deliberadamente

Corregir las 56 restantes dentro de la Etapa 0.5 del asistente habría sido trabajo no pedido, sobre filas que nadie consulta hoy, mezclado en la entrega de otra cosa. El diseñador lo señaló en vez de arreglarlo en silencio — comportamiento correcto.

Pero **el estado parcial es peor que ninguno de los dos extremos**: hoy el documento tiene filas corregidas junto a filas obsoletas, sin que nada indique cuáles son cuáles salvo la nota de corrección. Un lector no puede saber si la fila que está leyendo es de las revisadas.

## Disparador de pago

**El próximo trabajo de interfaz que consulte el mapa de componentes** — sea un diseño nuevo o la construcción de la pantalla del asistente. En ese momento se barre el documento completo contra `ui/lib/components/` real, en un solo paso, en vez de fila por fila según quien pase.

No antes: es documentación, no hay defecto vivo en el producto. No después: cada diseño que se apoye en una fila obsoleta arrastra el error hasta que un compilador lo detenga.

## Cómo se verifica que quedó pagada

**El criterio original de esta ficha era inservible y se sustituyó.** Decía: `grep -c "Glow" docs/DESIGN.md` devuelve `0`. Es imposible de cumplir sin destruir contenido correcto — "Glow" es un concepto vivo del sistema de diseño (`Gx.glow`, `Gx.textGlow`, `HoverGlow`, la sección "Elevation y Glow"), y hay 51 apariciones legítimas. Peor: el criterio premiaba **borrar** las notas históricas ("corrige la fila anterior: `GlowButton`, clase inexistente"), que son justo lo que necesita un diseñador que aún recuerda el nombre viejo. Un criterio verificable por `grep` no es, por eso solo, un criterio correcto.

**Criterio real aplicado:** ninguna fila del mapa presenta como usable una clase que no exista. Cada fila o nombra una clase existente **con su firma verificada contra el código**, o dice explícitamente **"Sin construir"** y no ofrece nada que instanciar. Los nombres retirados solo pueden aparecer negados ("clase inexistente", "fue retirado", "no existe").

**Cómo se comprobó (2026-07-31):** barrido de las 127 filas una a una contra `ui/lib/`, más dos verificaciones automáticas — (1) todo identificador entrecomillado del mapa existe en el código, y (2) todo identificador que NO existe aparece únicamente en contexto de negación explícita. Ambas en verde.

## Qué destapó el barrido, más allá de lo registrado

La ficha estimaba "56 menciones del prefijo viejo". El defecto real era más grande y de otra naturaleza:

- **10 símbolos presentados como usables que no existen en ninguna parte** — no eran nombres viejos de clases vivas, eran clases que nunca sobrevivieron: `GlassSurface`, `Gx.vDivider`, `_regimeChip`, `_kpi`, `_vitalityBar`, `_spinner`, `_scanRing`, `_organismCell`, `_vitalityLegend`, y `_crystalOrb` con un argumento inventado.
- **~30 filas con ruta obsoleta**: todo el bloque de tokens apuntaba a `gallery/gallery_tokens.dart` (el archivo no existe; es `theme/gx_tokens.dart`), y los 14 painters de dominio a `gallery_painters.dart` (viven en `sections/section_dataviz_quant.dart`).
- **5 firmas inventadas** en piezas que sí existen: `ZuiNavPill({segments})`, `ZuiZoomFrame({zoom, content})`, `ExpectationEnvelopeBadge({isInside})`, `FleetCommandPanel({source, items})` y `MultiSelect({value})` — todas con parámetros que el código nunca tuvo. Las cuatro primeras son **vitrinas con constructor vacío**: el mapa las anunciaba como piezas que aceptan datos propios, y no lo son.
- **3 componentes reales ausentes del mapa**: `custom_ui.Card`, `custom_ui.BentoCard` y `custom_ui.Sheet` existen y no estaban registrados — la puerta abierta a que alguien los reconstruya.
- **`DrasusThemeState`**, la clase nombrada en la nota de superficies, no existe: se llama `ThemeState`.

La lección de fondo: el mapa mezclaba sin distinguir **librería reutilizable** (`components/`) y **vitrina de exhibición** (`gallery/`, datos incrustados, constructor vacío). Esa confusión es la misma que costó STORY-025. El mapa ahora lo marca fila por fila.
