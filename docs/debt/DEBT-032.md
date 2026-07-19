# DEBT-032 · El patrón de ledger que se copia tiene dos huecos, y se propagan

| Campo | Valor |
|---|---|
| **ID** | DEBT-032 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Gate de QA sobre STORY-051; el QA identificó correctamente que no es un defecto de esta Story |
| **Dominio** | Transversal — todos los ledgers append-only |

## Causa raíz

Cada feature nueva construye su ledger append-only copiando el patrón de una feature anterior. Es la práctica correcta y así se pidió explícitamente en STORY-051. El problema es que **el patrón de referencia tiene dos huecos**, y copiarlo fielmente los replica.

### Hueco 1 — La prueba companion del clasificador no prueba nada

El patrón exige tres pruebas acompañantes. La segunda debe verificar que el clasificador de errores distingue un conflicto de escritura **transitorio** (contención: hay que reintentar) de uno **permanente** (clave duplicada: reintentar es inútil y el bucle no termina nunca).

En `sovereign-data-fetcher` (STORY-046) esa prueba **nunca llama a la función que dice cubrir**: verifica el mensaje de un `INSERT` crudo, con un comentario auto-consciente que admite que se comprueba "indirectamente". STORY-051 copió el patrón fielmente, incluido el hueco. La mutación lo confirma: el clasificador puede mutarse a **devolver siempre "transitorio"** sin que ninguna prueba falle.

Consecuencia si el clasificador se rompiera: el ledger reintentaría en bucle una escritura que no puede tener éxito jamás, hasta agotar los intentos, y reportaría contención donde hay un error permanente de datos.

### Hueco 2 — Migraciones sin triggers anti UPDATE/DELETE

Seis de los ocho ledgers del repositorio (migraciones 0010, 0014, 0015, 0016, 0018, 0019, 0020) declaran triggers que hacen que la base de datos **rechace** cualquier modificación o borrado. Dos no: `sovereign-data-fetcher` (0006) y `hybrid-data-transformer` (0021).

STORY-051 heredó la ausencia precisamente porque se le indicó seguir el patrón de `sovereign-data-fetcher`. **La 0021 se corrige dentro de STORY-051** (disposición del Tech-Lead, 2026-07-18). La 0006 queda aquí.

Un ledger cuyo carácter inmutable depende solo de que nadie escriba el `UPDATE` equivocado no es append-only: es una convención de buena voluntad. La inmutabilidad debe estar en la base de datos, no en la disciplina del programador.

## Impacto actual

Ninguno vivo. Ningún código escribe `UPDATE` sobre esos ledgers, y ningún clasificador está roto hoy. Ambos son huecos de **detección**: si algo se rompiera, nada avisaría.

## Estado del molde (actualizado 2026-07-19)

**El molde ya está corregido y declarado canónico:** `.agents/knowledge/ledger-pattern.md`. Contiene los 7 puntos del patrón, las 4 pruebas obligatorias, y la historia de los dos huecos para que nadie los repita.

**Regla nueva, que es lo que de verdad cierra la causa raíz:** cuando una Orden diga "sigue el patrón de ledger", se refiere a ESE documento — nunca a "la feature que ya lo tiene implementado". Elegir referencia por antigüedad ("la primera que se construyó") es el peor criterio posible: fue exactamente lo que propagó estos dos huecos.

Lo que queda abierto en esta deuda es **el retrofit del código existente**, no el molde.

## Qué hay que hacer

1. **Prueba real del clasificador en `sovereign-data-fetcher`**: ejercitar la función directamente con los casos que debe distinguir, y verificar por mutación que sobrevive a la inversión de su lógica.
2. **Triggers anti UPDATE/DELETE en la migración 0006**, siguiendo el patrón de la 0019, con la prueba que demuestra que la base rechaza modificación y borrado. Fase GREENFIELD: se edita in-situ (ADR-0006).
3. ~~**Corregir el patrón de referencia en la documentación**~~ — ✅ **hecho el 2026-07-19**: `.agents/knowledge/ledger-pattern.md`, declarado canónico e indexado.
4. Barrer los seis ledgers restantes para confirmar que su prueba del clasificador sí es discriminante y no comparte el mismo defecto.

## Disparador de pago

**Antes de construir el siguiente ledger append-only.** El coste de la deuda no es el hueco actual: es que cada feature nueva lo copia. Cuanto más tarde se corrija el patrón, más sitios hay que arreglar.
