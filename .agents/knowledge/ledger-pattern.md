# HOW TO: construir un ledger append-only — MOLDE CANÓNICO

**Disparador:** vas a crear o tocar una tabla append-only (registro inmutable con `event_sequence_id` y cadena de hashes).

> ## 🔒 ESTE DOCUMENTO ES LA REFERENCIA CANÓNICA
>
> **Cuando una Orden de Trabajo diga "sigue el patrón de ledger", se refiere a ESTE documento — nunca a "la feature que ya lo tiene implementado".**
>
> **Por qué esta regla existe, con el caso real.** La primera feature del proyecto que necesitó un registro inmutable fue `sovereign-data-fetcher`. Se construyó, funcionó, y **se convirtió en "el patrón establecido" por el simple hecho de haber sido la primera**. A partir de ahí, cada Orden nueva decía literalmente *"siguiendo el patrón ya establecido por `sovereign-data-fetcher` — léelo antes de escribir"*.
>
> El problema: esa primera implementación tenía **dos huecos**, y cada feature que copió fielmente los heredó. STORY-051 los heredó en 2026-07-18 **por hacer exactamente lo que se le pidió**. Lo destapó el gate de mutación, no la revisión humana.
>
> Lo irónico: los ledgers posteriores (migraciones 0010, 0014, 0015, 0016, 0018, 0019, 0020) **sí** corrigieron uno de los huecos por su cuenta. O sea que la práctica mejoró, pero **la referencia oficial se quedó siendo la vieja** — y a las features nuevas se las seguía apuntando ahí.
>
> **La lección general, aplicable más allá de los ledgers:** "la primera implementación" es el peor criterio posible para elegir una referencia canónica. Una referencia se **declara** explícitamente, se **audita**, y vive en un documento — no en el código de quien llegó primero.

---

## Los dos huecos históricos, para que nadie los repita

### Hueco 1 — La prueba del clasificador que no prueba nada

El patrón exige distinguir dos clases de error de escritura:
- **Transitorio** (contención: otro escritor tiene la base tomada) → hay que **reintentar**.
- **Permanente** (clave duplicada) → reintentar es **inútil**, el bucle no terminaría nunca.

La prueba original **nunca llamaba a la función clasificadora**: verificaba el mensaje de un `INSERT` crudo, con un comentario auto-consciente admitiendo que lo comprobaba "indirectamente". El gate de mutación demostró que el clasificador puede mutarse a **devolver siempre "transitorio"** sin que ninguna prueba falle.

**Consecuencia si se rompiera:** el ledger reintentaría en bucle una escritura que jamás puede tener éxito, hasta agotar los intentos, y reportaría contención donde hay un error permanente de datos.

**La forma correcta:** ejercitar la función **directamente**, con sus tres casos — violación de clave primaria (permanente), colisión de secuencia (transitorio), y un error que no sea de base de datos.

### Hueco 2 — Migración sin candados

Un ledger cuya inmutabilidad depende de que nadie escriba el `UPDATE` equivocado **no es append-only: es un acuerdo de caballeros.** La inmutabilidad tiene que estar en la base de datos.

Las migraciones `0006` (`sovereign-data-fetcher`) y `0021` (`hybrid-data-transformer`) nacieron sin ellos. La `0021` se corrigió en STORY-051; la `0006` sigue pendiente (**DEBT-032**).

---

## El molde, en siete puntos

1. **Tabla `STRICT`**, clave primaria `TEXT` con UUIDv7, todos los tipos según ADR-0141 (precio y volumen `INTEGER` escalado ×10⁸, timestamps `INTEGER` en nanosegundos UTC).

2. **`event_sequence_id INTEGER UNIQUE`** — la secuencia monótona sin huecos que ordena el ledger.

3. **Cadena de hashes:** `audit_hash` de la fila y `audit_chain_hash` que la encadena con la anterior. **`NULL` en la fila génesis**, nunca un valor centinela.

4. **Escritura envuelta en `BEGIN IMMEDIATE`** con reintento acotado y **error tipado de contención**. Nunca sentencias sueltas: la lectura de la secuencia y la escritura de la fila son una sola operación atómica o hay pérdida de eventos bajo concurrencia.

5. **Candados anti UPDATE/DELETE en la migración**, con `RAISE(ABORT, ...)` y un mensaje que diga por qué. Ejemplo correcto **de este punto concreto**: migración `0019` (`data_portability`) o `0006` (`sovereign_data_fetcher`, corregida 2026-07-19).

   ⚠️ **Copia el punto, no el archivo.** La `0019` se citó aquí como "referencia limpia" el 2026-07-19 y el barrido de ese mismo día descubrió que **cumple los candados pero no cumple el punto 7 en sus tres casos**. Se eligió por verse limpia, sin auditarla entera — exactamente el error que este documento existe para impedir, un nivel más arriba. **Ningún archivo del repo es "el ejemplo a seguir" completo. El molde es este documento.**

6. **Ficha en `docs/data-model/<tabla>.md`** creada en el mismo cambio, coincidiendo columna por columna con la migración, más su fila en el índice `docs/DATA-MODEL.md`.

7. **Las cuatro pruebas obligatorias**, todas discriminantes y todas con demostración de que fallan sin la implementación:
   - **Concurrencia real:** 16 escritores en paralelo, runtime multihilo, sin perder ni duplicar secuencia.
   - **Contención sostenida:** un segundo escritor reteniendo la transacción agota los intentos y devuelve el error tipado con el conteo exacto.
   - **Clasificador directo:** la función de clasificación, llamada por su nombre, con sus tres casos. **Este es el hueco histórico — no lo repitas.**
   - **Fidelidad de la fila devuelta:** hash, eslabón de cadena y marca de tiempo frescos, no los de la fila anterior.

---

## Regla de mantenimiento

Si mejoras el patrón, **lo actualizas aquí primero**. Una mejora que solo vive en la última feature que la aplicó no es una mejora del patrón: es otra referencia informal esperando a que alguien la copie a medias.
