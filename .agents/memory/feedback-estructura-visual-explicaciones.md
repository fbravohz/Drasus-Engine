---
name: feedback-estructura-visual-explicaciones
description: Formato visual exacto que necesita el usuario para leer explicaciones estructuradas (problema/solución/por qué/detalle) sin tener que releer dos veces.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2efeeeb3-0a36-45a2-bb5d-56ac9007bf90
---

Al aplicar la estructura Contexto-Antes-de-Detalle (`base.md` §2.3.3: problema → solución → por qué → detalle) en una explicación al usuario:

- Etiqueta o encabezado corto por bloque (ej. `### 1. Problema`) en su **propia línea**, con salto de línea antes de empezar el texto del bloque — nunca la etiqueta pegada al texto en la misma línea.
- Bullets para cualquier bloque que liste 2 o más ítems (varios riesgos, varias causas, varios pasos) — nunca enumerarlos en prosa corrida dentro de una oración.
- En el bloque de Detalle Técnico sobre todo: cada función, comando, flag o herramienta que se nombre lleva entre paréntesis qué hace esa sintaxis concreta — nunca asumir que el nombre se explica solo.

**Esta regla ya está codificada como obligatoria en `base.md` §2.3 ("Formato Visual de los Bloques") y en el gate de auto-validación §7.1 guarda 5** — no depende de esta memoria para aplicarse, pero la memoria documenta el porqué.

**Why:** el usuario reportó tener que leer dos veces una explicación que sí seguía el orden correcto (problema→solución→por qué→detalle), porque el bloque de "problema" mezclaba dos riesgos en una sola oración de prosa, y la etiqueta `[1. Problema]` estaba pegada al texto sin salto de línea. El orden lógico solo no bastó — la delimitación visual es lo que permite leerlo en una pasada.

**How to apply:** en toda explicación estructurada (modo Mentor/Docente, o simplemente al explicar una decisión técnica en cualquier respuesta), delimitar cada bloque con encabezado en línea propia + salto de línea antes del cuerpo, y usar bullets dentro de cada bloque en cuanto haya 2+ ítems paralelos. Relacionado: [[comunicacion-lenguaje-llano-ceo]].
