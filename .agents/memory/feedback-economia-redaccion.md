---
name: feedback-economia-redaccion
description: El usuario decidió el estilo de escritura final (Gemini Relajado + Listas con Contexto) tras medir tokens reales entre estilo Claude y Gemini; ahora vive codificado en WRITING-EFFICIENCY.MD.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2efeeeb3-0a36-45a2-bb5d-56ac9007bf90
---

Estilo de escritura por defecto: **oración corta de contexto (termina en `:`) + lista con bullets/numerales**, con prioridad sobre texto lineal cuando hay 2+ ítems paralelos. Texto lineal relajado solo si el contenido es una sola idea continua sin ítems paralelos.

**Why:** el usuario venía señalando que las respuestas y documentos de Claude "balbucean" — usan demasiados tokens narrando conectores ("es fundamental que", "hay una serie de pasos que debes seguir de manera ordenada para") sin aportar información nueva. Se armó una comparación medida (no estimada) en `.agents/documents/tmp/gemini_vs_claude.md`: el mismo contenido en estilo Claude natural vs. varias variantes de compresión (Gemini puro ~83% menos, sweet spot ~73%, relajado+listas ~61-67%). El usuario probó una versión propia que agregaba una oración de contexto delante de la lista (en vez de listas sueltas sin gancho) y confirmó que esa es la versión ganadora — mantiene "Contexto Antes de Detalle" (`BASE-2.3.3`) sin perder la economía de tokens.

**How to apply:** la regla vive codificada entera en `.agents/knowledge/WRITING-EFFICIENCY.MD` (`WRITING_EFFICIENCY-1` a `-6`, formato `RULE.md`); `base.md` §2 solo apunta ahí, no duplica contenido. Aplica a documentos de knowledge/skills Y a respuestas en chat — no es solo un estilo de documentación. Relacionado: [[feedback-estructura-visual-explicaciones]] (el bloque problema/solución/detalle sigue vigente; esta regla define CÓMO se escribe el texto dentro de cada bloque, no reemplaza la estructura de 4 bloques).
