# HOW TO: Correr y Juzgar el Gate de Mutación

Cómo correr `scripts/mutation-gate.sh`, leer su veredicto y juzgar supervivientes — dominio exclusivo del Tech-Lead.

Slug ID: `MUTATION-GATE`.

Tamaño: `10173 caracteres (~2543 tokens aprox., ratio 4:1)`.

---

## MUTATION-GATE-FV — Filtro de Verificación

**Aplica a:** el Tech-Lead, como último paso antes de certificar una corrida del gate.

* **MUTATION-GATE-FV-1:** ¿solo el Tech-Lead corrió el gate, y lo hizo en el disparador correcto? (`MUTATION-GATE-1`)
* **MUTATION-GATE-FV-2:** ¿entendiste que el gate mide huecos de detección, no defectos de comportamiento? (`MUTATION-GATE-2`)
* **MUTATION-GATE-FV-3:** ¿usaste `scripts/mutation-gate.sh` como único punto de entrada, sin escribir el comando `cargo-mutants` a mano? (`MUTATION-GATE-3`)
* **MUTATION-GATE-FV-4:** ¿leíste el veredicto `GATE APTO`/`GATE NO APTO` en el texto, sin canalizar la salida a otro comando? (`MUTATION-GATE-4`)
* **MUTATION-GATE-FV-5:** ¿delegaste la corrida al Tech-Lead en vez de esperarla desde un subagente? (`MUTATION-GATE-5`)
* **MUTATION-GATE-FV-6:** ¿cada superviviente aceptado es `timeout`, `unviable`, o equivalente genuino con demostración aritmética escrita junto al código? (`MUTATION-GATE-6`)
* **MUTATION-GATE-FV-7:** ¿revisaste el resultado buscando aserciones autorreferenciales o pruebas escritas solo para matar el mutante, incluso si el gate pasó? (`MUTATION-GATE-7`)

De ser satisfactoria cada validación de la lista, declara `MUTATION-GATE - FILTRO DE VERIFICACION APROBADO`.

---

## MUTATION-GATE-1 — Disparador y Rol Exclusivo del Tech-Lead

**Aplica a:** Tech-Lead (autorizado a correrlo); Ingenieros y QA (prohibido correrlo).

* **MUTATION-GATE-1.1 — Disparador:**

  vas a cerrar una Story que añade o cambia lógica de correctitud, o tienes que juzgar supervivientes de mutación.

* **MUTATION-GATE-1.2 — Quién lo Corre:**

  solo el Tech-Lead. Ingenieros y QA lo tienen prohibido — ver la división de trabajo en `MUTATION-GATE-5`.

**Filtro de verificación:** `MUTATION-GATE-FV-1`.

---

## MUTATION-GATE-2 — Qué Es y Por Qué Es el Gate Más Valioso

El gate de mutación rompe el código a propósito — una rotura por cada cambio posible — y para cada rotura corre las pruebas.

**Aplica a:** todos los roles, al interpretar qué mide el gate.

* **MUTATION-GATE-2.1 — Definición de Superviviente:**

  si las pruebas siguen en verde con el código roto, esa rotura "sobrevivió": significa que ninguna prueba vigila esa línea. Es la única metaprueba que responde a "¿mis pruebas se enterarían si esto dejara de funcionar?".

* **MUTATION-GATE-2.2 — Mide Huecos de Detección, No Defectos:**

  el gate puede destapar decenas de huecos en una feature con todas las pruebas verdes, análisis estático limpio y alta cobertura, sin que ninguno sea un defecto de comportamiento — el código hace lo correcto, pero nada avisaría si dejara de hacerlo. Ver caso compuesto en `MUTATION-GATE-EJ-1`.

**Filtro de verificación:** `MUTATION-GATE-FV-2`.

---

## MUTATION-GATE-3 — Cómo Se Corre: el Script Como Único Punto de Entrada

```bash
scripts/mutation-gate.sh <crate> [jobs]
```

**Aplica a:** el Tech-Lead, al ejecutar el gate.

* **MUTATION-GATE-3.1 — Nunca a Mano:**

  ese script es el único punto de entrada. No escribas el comando `cargo-mutants` a mano.

* **MUTATION-GATE-3.2 — Por Qué Existe el Script:**

  `cargo-mutants` copia el árbol de fuentes completo al directorio temporal del sistema y compila ahí una copia por cada trabajo en paralelo. En estas máquinas `/tmp` es `tmpfs` — RAM disfrazada de disco. El árbol arrastra Polars, cuyos binarios de depuración pesan cientos de megas y cuyo enlazado necesita varios gigas de memoria de trabajo por proceso. Con el paralelismo por defecto (= número de núcleos) arrancan tantos enlazadores simultáneos como núcleos haya, se agota la RAM, y el enlazador muere con:
  ```
  collect2: fatal error: ld terminated with signal 7 [Bus error], core dumped
  error: could not compile ... (test "integration_tests")
  ERROR cargo build failed in an unmutated tree, so no mutants were tested
  ```
  Ese mensaje aparenta un fallo de compilación grave y cuesta descubrir que es falta de memoria.

* **MUTATION-GATE-3.3 — Las Tres Medidas del Script:**

  temporales en disco real, paralelismo acotado a 2, y compilación sin símbolos de depuración (los binarios de mutación no se depuran nunca, y quitar esa información reduce drásticamente tamaño y memoria de enlazado). Además aborta con un mensaje claro si detecta que su base cayó en `tmpfs`.

**Filtro de verificación:** `MUTATION-GATE-FV-3`.

---

## MUTATION-GATE-4 — El Veredicto Se Lee en el Texto, Nunca en el Código de Salida

**Aplica a:** todos los roles, al leer el resultado de una corrida del gate.

* **MUTATION-GATE-4.1 — El Veredicto Es la Línea de Texto:**

  el veredicto es la línea `GATE APTO` / `GATE NO APTO` del texto. Léela.

* **MUTATION-GATE-4.2 — Riesgo de Canalizar la Salida:**

  si canalizas la salida a otro comando, el código de salida que vale es el de la tubería (`PIPESTATUS[0]` en bash), no el del filtro. Un gate `NO APTO` con supervivientes puede canalizarse a `tail` y el sistema reportar "completado con éxito" porque ese es el código de salida de `tail`, no del script.
  * ❌ *Incorrecto:* `./scripts/mutation-gate.sh sovereign-data-fetcher 2>&1 | tail -30`
  * ✅ *Correcto:* dejar que el script imprima su resumen completo, sin canalizar.

* **MUTATION-GATE-4.3 — Regla General Más Allá de Este Gate:**

  un "completado con éxito" del sistema no es un veredicto de calidad. Solo dice que un proceso terminó.

**Filtro de verificación:** `MUTATION-GATE-FV-4`.

---

## MUTATION-GATE-5 — Por Qué No Lo Corren los Subagentes

No es preferencia de estilo, es economía medida.

**Aplica a:** todos los roles, al decidir quién ejecuta la corrida.

* **MUTATION-GATE-5.1 — El Riesgo de Esperar Desde un Subagente:**

  un subagente que lanza la corrida y se queda esperando —consultando el archivo de salida, viendo que no termina, volviendo a consultar— agota su presupuesto completo sin entregar nada, aunque el trabajo de fondo sí termine. Una corrida son ~18 minutos de CPU y cero tokens si la lanza quien puede hacer otra cosa mientras tanto.

* **MUTATION-GATE-5.2 — La División Correcta:**

  el ingeniero escribe las pruebas y entrega el código. El Tech-Lead corre el gate y certifica. Si hay supervivientes, se los devuelve con el diagnóstico hecho —qué mutante, en qué función, por qué esa prueba no lo detecta— para que el ingeniero gaste su presupuesto escribiendo, no diagnosticando. El QA juzga el resultado: qué es hueco real y qué es equivalente genuino.

**Filtro de verificación:** `MUTATION-GATE-FV-5`.

---

## MUTATION-GATE-6 — Cómo Se Juzga un Superviviente

**Estándar: 0 supervivientes.** `timeout` y `unviable` no cuentan como supervivientes.

**Aplica a:** el Tech-Lead, al clasificar cada superviviente reportado por el gate.

* **MUTATION-GATE-6.1 — `timeout` Cuenta Como Captura:**

  el mutante colgó la prueba — la prueba sí detecta el cambio, solo que colgándose en vez de fallando. Ejemplo: mutar `attempt += 1` a `attempt *= 1` deja el contador en cero para siempre y cuelga la prueba de contención sostenida.

* **MUTATION-GATE-6.2 — `unviable` No Es un Cambio Posible:**

  el mutante ni compila.

* **MUTATION-GATE-6.3 — El Único Superviviente Aceptable, el Equivalente Genuino:**

  un mutante es equivalente cuando ninguna prueba puede matarlo porque el cambio que introduce no puede alcanzar el valor de retorno. Se acepta solo con la demostración aritmética escrita en un comentario junto al código. "Es trivial" no es justificación — el Tech-Lead traza la demostración él mismo antes de aceptarla.

  Ejemplo correcto (`round_half_to_even_div_i128`): mutar `<` a `<=` en la detección de signo `(numerator < 0) != (denominator < 0)`.
  * Rama del divisor: la función retorna antes si el divisor es cero → el cambio es inalcanzable.
  * Rama del dividendo: si el dividendo es cero, el signo calculado sí difiere, pero el valor absoluto redondeado es cero y `signo × 0 = 0` en ambos casos → el bit alterado nunca llega al retorno.
  * Con ambos distintos de cero, `<` y `<=` coinciden.
  * Exigir una prueba ahí obligaría a escribir una aserción que no puede fallar nunca — lo contrario de lo que queremos.

**Filtro de verificación:** `MUTATION-GATE-FV-6`.

---

## MUTATION-GATE-7 — Los Dos Olores que el Gate Destapa una y Otra Vez

**Aplica a:** el QA, al revisar el resultado de una corrida — incluso si el gate pasó.

* **MUTATION-GATE-7.1 — La Aserción Autorreferencial:**

  una prueba que compara la salida de una función consigo misma expresa "es determinista", pero para una función pura eso es gratis por construcción del lenguaje: la prueba pasa con cualquier implementación, incluida una que devuelva siempre vacío. La forma correcta: entradas distintas → salidas distintas; entradas equivalentes construidas por separado → misma salida.

* **MUTATION-GATE-7.2 — La Prueba Escrita para Matar el Mutante:**

  tras un `NO APTO`, entra una tanda de pruebas bajo presión de cerrar el gate. Una prueba que fija un valor sin explicar por qué ese valor es el correcto es deuda disfrazada de cobertura — y peor que no tenerla, porque el verde hace que nadie vuelva a mirar. El QA debe buscarlas explícitamente y reportarlas aunque el gate pase.

**Filtro de verificación:** `MUTATION-GATE-FV-7`.

---

## MUTATION-GATE-EJ — Ejemplos

### MUTATION-GATE-EJ-1 — Una Feature Verde que Escondía 46 Huecos

Caso compuesto: una feature con 70 pruebas verdes, análisis estático limpio y 90% de cobertura pasó por auditoría humana sin que nadie notara nada — demuestra `MUTATION-GATE-2.2` y `MUTATION-GATE-7.1`.

* **Sin el gate:** 70 pruebas verdes, 90% de cobertura, auditoría humana aprobada. Ninguna señal de alarma.
* **Con el gate:** 46 huecos destapados, entre ellos una prueba que comparaba una función consigo misma — habría pasado igual si la función devolviera siempre una cadena vacía. Ninguno de los 46 era un defecto de comportamiento: el código hacía lo correcto. Todos eran huecos de detección — nada avisaría si dejara de hacerlo.
