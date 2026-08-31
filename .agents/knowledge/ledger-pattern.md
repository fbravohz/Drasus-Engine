# HOW TO: Construir un Ledger Append-Only — Molde Canónico

Molde canónico para crear o tocar una tabla append-only (registro inmutable con `event_sequence_id` y cadena de hashes).

Slug ID: `LEDGER-PATTERN`.

Tamaño: `7883 caracteres (~1971 tokens aprox., ratio 4:1)`.

---

## LEDGER-PATTERN-FV — Filtro de Verificación

**Aplica a:** todos los roles, como último paso antes de cerrar un cambio que crea o toca un ledger.

* **LEDGER-PATTERN-FV-1:** ¿usaste este documento como referencia del patrón de ledger, no una feature ya implementada? (`LEDGER-PATTERN-1`)
* **LEDGER-PATTERN-FV-2:** ¿la prueba del clasificador de errores llama directamente a la función, con sus tres casos? (`LEDGER-PATTERN-2`)
* **LEDGER-PATTERN-FV-3:** ¿la migración lleva candados anti `UPDATE`/`DELETE` en la base de datos, no solo un acuerdo de convención en el código? (`LEDGER-PATTERN-3`)
* **LEDGER-PATTERN-FV-4:** ¿la tabla cumple los seis puntos del molde canónico? (`LEDGER-PATTERN-4`)
* **LEDGER-PATTERN-FV-5:** ¿las cuatro pruebas obligatorias están escritas, son discriminantes, y demuestran que fallan sin la implementación? (`LEDGER-PATTERN-5`)
* **LEDGER-PATTERN-FV-6:** si mejoraste el patrón, ¿lo actualizaste primero en este documento? (`LEDGER-PATTERN-6`)

De ser satisfactoria cada validación de la lista, declara `LEDGER-PATTERN - FILTRO DE VERIFICACION APROBADO`.

---

## LEDGER-PATTERN-1 — Este Documento Es la Referencia Canónica

Cuando una Orden de Trabajo diga "sigue el patrón de ledger", se refiere a este documento — nunca a "la feature que ya lo tiene implementado".

**Aplica a:** todos los roles, al construir o tocar una tabla append-only.

* **LEDGER-PATTERN-1.1 — Nunca una Feature Como Referencia:**

  ninguna implementación del repo es "el ejemplo a seguir" completo. El molde es siempre este documento, nunca el código de la feature que lo implementó primero.

* **LEDGER-PATTERN-1.2 — Por Qué:**

  la primera implementación de un patrón es el peor criterio posible para elegirla como referencia canónica — hereda huecos silenciosos que cada copia fiel propaga sin darse cuenta. Una referencia se declara explícitamente, se audita, y vive en un documento, no en el código de quien llegó primero. Los huecos concretos que motivan esta regla están documentados en `LEDGER-PATTERN-2` y `LEDGER-PATTERN-3`, para que no se repitan.

**Filtro de verificación:** `LEDGER-PATTERN-FV-1`.

---

## LEDGER-PATTERN-2 — La Prueba del Clasificador Llama a la Función, No al Síntoma

El patrón exige distinguir dos clases de error de escritura.

**Aplica a:** todos los roles, al escribir la prueba del clasificador de errores de un ledger.

* **LEDGER-PATTERN-2.1 — Las Dos Clases de Error:**

  transitorio (contención: otro escritor tiene la base tomada) → hay que reintentar. Permanente (clave duplicada) → reintentar es inútil, el bucle no terminaría nunca.

* **LEDGER-PATTERN-2.2 — El Hueco: Probar el Síntoma, No la Función:**

  una prueba que nunca llama a la función clasificadora, y en su lugar verifica el mensaje de un `INSERT` crudo, no prueba el clasificador — solo prueba indirectamente un síntoma. El gate de mutación demuestra que ese clasificador puede mutarse a devolver siempre "transitorio" sin que ninguna prueba falle.
  * Consecuencia si se rompiera: el ledger reintentaría en bucle una escritura que jamás puede tener éxito, hasta agotar los intentos, y reportaría contención donde hay un error permanente de datos.

* **LEDGER-PATTERN-2.3 — La Forma Correcta:**

  ejercitar la función directamente, con sus tres casos — violación de clave primaria (permanente), colisión de secuencia (transitorio), y un error que no sea de base de datos.

**Filtro de verificación:** `LEDGER-PATTERN-FV-2`.

---

## LEDGER-PATTERN-3 — Los Candados Viven en la Migración, No en la Convención

Un ledger cuya inmutabilidad depende de que nadie escriba el `UPDATE` equivocado no es append-only: es un acuerdo de caballeros. La inmutabilidad tiene que estar en la base de datos.

**Aplica a:** todos los roles, al escribir la migración de una tabla append-only.

* **LEDGER-PATTERN-3.1 — Sin Candados en Base de Datos No Es Append-Only:**

  la inmutabilidad se garantiza con constraints/triggers en la propia base de datos, nunca solo con la disciplina de quien escribe el código de aplicación. Las migraciones `0006` (`sovereign-data-fetcher`) y `0021` (`hybrid-data-transformer`) nacieron sin candados: la `0021` ya los tiene corregidos; la `0006` sigue pendiente (`DEBT-032`). Los ledgers posteriores — migraciones `0010`, `0014`, `0015`, `0016`, `0018`, `0019` y `0020` — ya corrigieron este hueco por su cuenta antes de que existiera este molde canónico.

**Filtro de verificación:** `LEDGER-PATTERN-FV-3`.

---

## LEDGER-PATTERN-4 — El Molde Canónico, en Seis Puntos

**Aplica a:** todos los roles, al diseñar la tabla y la escritura de un ledger nuevo.

* **LEDGER-PATTERN-4.1 — Tabla y Tipos:**

  tabla `STRICT`, clave primaria `TEXT` con UUIDv7, todos los tipos según [[docs/adr/ADR-0141.md]] (precio y volumen `INTEGER` escalado ×10⁸, timestamps `INTEGER` en nanosegundos UTC).

* **LEDGER-PATTERN-4.2 — Secuencia Monótona:**

  `event_sequence_id INTEGER UNIQUE` — la secuencia monótona sin huecos que ordena el ledger.

* **LEDGER-PATTERN-4.3 — Cadena de Hashes:**

  `audit_hash` de la fila y `audit_chain_hash` que la encadena con la anterior. `NULL` en la fila génesis, nunca un valor centinela.

* **LEDGER-PATTERN-4.4 — Escritura Atómica:**

  escritura envuelta en `BEGIN IMMEDIATE` con reintento acotado y error tipado de contención. Nunca sentencias sueltas: la lectura de la secuencia y la escritura de la fila son una sola operación atómica, o hay pérdida de eventos bajo concurrencia.

* **LEDGER-PATTERN-4.5 — Candados en la Migración:**

  candados anti `UPDATE`/`DELETE` en la migración, con `RAISE(ABORT, ...)` y un mensaje que diga por qué. Ejemplo correcto de este punto concreto: migración `0019` (`data_portability`) o `0006` (`sovereign_data_fetcher`, ya corregida).

  ⚠️ Copia el punto, no el archivo. La migración `0019`, por ejemplo, cumple los candados de este punto pero no cumple `LEDGER-PATTERN-5` en sus tres casos — se la señaló como referencia limpia sin auditarla entera, exactamente el error que este documento existe para impedir, un nivel más arriba. Ningún archivo del repo es "el ejemplo a seguir" completo. El molde es este documento.

* **LEDGER-PATTERN-4.6 — Ficha de Datos:**

  ficha en `docs/data-model/<tabla>.md` creada en el mismo cambio, coincidiendo columna por columna con la migración, más su fila en el índice [[docs/DATA-MODEL.md]].

**Filtro de verificación:** `LEDGER-PATTERN-FV-4`.

---

## LEDGER-PATTERN-5 — Las Cuatro Pruebas Obligatorias

Todas discriminantes y todas con demostración de que fallan sin la implementación.

**Aplica a:** todos los roles, al escribir las pruebas de un ledger nuevo.

* **LEDGER-PATTERN-5.1 — Concurrencia Real:**

  16 escritores en paralelo, runtime multihilo, sin perder ni duplicar secuencia.

* **LEDGER-PATTERN-5.2 — Contención Sostenida:**

  un segundo escritor reteniendo la transacción agota los intentos y devuelve el error tipado con el conteo exacto.

* **LEDGER-PATTERN-5.3 — Clasificador Directo:**

  la función de clasificación, llamada por su nombre, con sus tres casos. Este es el hueco histórico de `LEDGER-PATTERN-2` — no lo repitas.

* **LEDGER-PATTERN-5.4 — Fidelidad de la Fila Devuelta:**

  hash, eslabón de cadena y marca de tiempo frescos, no los de la fila anterior.

**Filtro de verificación:** `LEDGER-PATTERN-FV-5`.

---

## LEDGER-PATTERN-6 — Regla de Mantenimiento

Si mejoras el patrón, lo actualizas aquí primero. Una mejora que solo vive en la última feature que la aplicó no es una mejora del patrón: es otra referencia informal esperando a que alguien la copie a medias.

**Filtro de verificación:** `LEDGER-PATTERN-FV-6`.
