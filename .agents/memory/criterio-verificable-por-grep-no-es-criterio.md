---
name: criterio-verificable-por-grep-no-es-criterio
description: "Un criterio de aceptación verificable por grep no es, por eso solo, un criterio correcto; y regenerar un golden sin mirarlo es bendecir lo que salga"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 98146cd8-fa24-4881-a025-10d85296e2d3
  modified: 2026-08-01T17:40:29.951Z
---

Un criterio de aceptación **verificable por `grep` no es, por eso solo, un criterio correcto**. Tres ocurrencias en la misma cadena de trabajo (STORY-073, 2026-07-31):

- Criterio 12 original, escrito por el propio Tech-Lead: "`grep` sobre el vocabulario retirado → 0 resultados". Imposible de cumplir sin mutilar el documento — `Gx.textGlow` y `HoverGlow` son código real y vigente.
- Criterio de pago de `DEBT-037`: `grep -c "Glow" docs/DESIGN.md → 0`. Hay 51 apariciones legítimas, y el criterio **premiaba borrar** las notas históricas que sirven a quien recuerda el nombre viejo.
- Corolario del mismo turno: `flutter test --update-goldens` deja el suite en verde sin que nadie haya mirado el PNG. Media pantalla salía en cajas de relleno porque la escala tipográfica nace sin familia hasta que `load()` corre. **Regenerar un golden sin inspeccionarlo es bendecir lo que sea que haya salido.**

**Why:** cuando la verificación mecánica sale más barata que pensar el criterio, se acaba midiendo lo que la herramienta sabe medir en vez de lo que el trabajo debe cumplir. Un `grep` no distingue entre nombrar como usable una clase inexistente y describir correctamente una que existe.

Cuarta ocurrencia, misma raíz, otro disfraz (2026-08-01): el criterio 13 de STORY-073 se asignó al propietario con un "solo lo puede ejecutar él" **y ninguna instrucción de cómo**. Abrió la feature vecina del lienzo, no encontró el selector de archivos y perdió la sesión. **Un criterio que el Tech-Lead no puede ejecutar es un criterio que debe poder dictar.**

**How to apply:** al redactar un criterio de aceptación o un criterio de pago de deuda, pregúntate qué pasaría si alguien lo cumpliera de la peor manera posible. Si la forma más barata de ponerlo en verde es destruir contenido correcto, el criterio está mal escrito. El criterio correcto describe el **estado del mundo** ("cada fila apunta a una clase real con firma verificada, o dice 'Sin construir'"), no el resultado de un comando. Todo artefacto visual regenerado se inspecciona antes de versionarlo. Y **todo criterio cuyo ejecutor sea el propietario se entrega con el recorrido literal**: dónde pulsar, qué debe aparecer, cómo saber que salió bien.

Relacionadas: [[razonamiento-critico-dominio]], [[banco-de-pruebas-dod-y-promocion-deuda]], [[unidad-declarada-junto-al-dato]].
