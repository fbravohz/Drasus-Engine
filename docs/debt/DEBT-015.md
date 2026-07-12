# DEBT-015 · #11 `instance-continuity` — `canonical_delta_bytes` sin test de valor-dorado
- **Severidad:** 🟡 Media (completitud de datos, no seguridad; impacto nulo hoy por greenfield + adaptador de subida diferido, pero un respaldo vacío silencioso sería pérdida de datos al restaurar).
- **Origen:** QA por mutación de STORY-039 (2026-07-06): 3 sobrevivientes, todos `canonical_delta_bytes -> Vec<u8>` reemplazado por `vec![]`/`vec![0]`/`vec![1]` — los tests no fijan la salida EXACTA de la serialización canónica del delta.
- **Descripción:** `compute_backup_delta` (el filtro que EXCLUYE secretos) sí está cazado; lo que falta es un test de valor-dorado sobre `canonical_delta_bytes` que ancle los bytes exactos producidos, de modo que un defecto que devolviera bytes triviales/vacíos (respaldo sin contenido) se detecte. Análogo a DEBT-012 (Box-Muller sin valor-dorado).
- **Impacto actual:** nulo (fase greenfield; el adaptador de subida S3/R2 está diferido, no hay respaldo real aún — STORY-039 §8).
- **Disparador de pago:** añadir el test de valor-dorado **antes** de construir el adaptador de almacén de objetos (antes de que exista un respaldo real que pudiera salir vacío).
- **Estado:** Abierta.
