# DEBT-037 · El mapa de componentes del documento de diseño apunta a clases que ya no existen

| Campo | Valor |
|---|---|
| **ID** | DEBT-037 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-26 (hallazgo colateral del UI-Designer en la Etapa 0.5 del asistente de importación) |
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

`grep -c "Glow" docs/DESIGN.md` devuelve `0`, y cada fila del mapa nombra una clase que existe en `ui/lib/components/`.
