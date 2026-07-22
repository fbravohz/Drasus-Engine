# HOW TO: correr y juzgar el gate de mutación

**Disparador:** vas a cerrar una Story que añade o cambia lógica de correctitud, o tienes que juzgar supervivientes de mutación.

**Quién lo corre:** **solo el Tech-Lead.** Ingenieros y QA lo tienen prohibido (ver §3).

---

## 1. Qué es y por qué es el gate más valioso que tenemos

El gate de mutación rompe el código a propósito —una rotura por cada cambio posible— y para cada rotura corre las pruebas. Si las pruebas siguen en verde con el código roto, **esa rotura "sobrevivió"**: significa que ninguna prueba vigila esa línea.

Es la única metaprueba que responde a la pregunta que ninguna otra hace: *"¿mis pruebas se enterarían si esto dejara de funcionar?"*

**Evidencia de su valor (STORY-051, 2026-07-18):** una feature con 70 pruebas verdes, análisis estático limpio y 90% de cobertura pasó por auditoría humana sin que nadie notara nada. El gate destapó **46 huecos**, entre ellos una prueba que comparaba una función consigo misma y habría pasado igual si la función devolviera siempre una cadena vacía. **Ninguno era un defecto de comportamiento** — el código hacía lo correcto. Todos eran huecos de detección: nada avisaría si dejara de hacerlo.

---

## 2. Cómo se corre

```bash
scripts/mutation-gate.sh <crate> [jobs]
```

**Ese script es el único punto de entrada.** No escribas el comando a mano.

### Por qué existe el script — el fallo que evita

`cargo-mutants` copia el árbol de fuentes completo al directorio temporal del sistema y compila ahí **una copia por cada trabajo en paralelo**. En estas máquinas `/tmp` es `tmpfs`: RAM disfrazada de disco. El árbol arrastra Polars, cuyos binarios de depuración pesan cientos de megas y cuyo enlazado necesita varios gigas de memoria de trabajo **por proceso**. Con el paralelismo por defecto (= número de núcleos) arrancan tantos enlazadores simultáneos como núcleos haya, se agota la RAM, y el enlazador muere con:

```
collect2: fatal error: ld terminated with signal 7 [Bus error], core dumped
error: could not compile ... (test "integration_tests")
ERROR cargo build failed in an unmutated tree, so no mutants were tested
```

**Ese mensaje aparenta un fallo de compilación grave** y cuesta un buen rato descubrir que era falta de memoria. Ocurrió el 2026-07-18 y costó una corrida entera.

El script aplica las tres medidas: temporales en disco real, paralelismo acotado a 2, y compilación sin símbolos de depuración (los binarios de mutación no se depuran nunca, y quitar esa información reduce drásticamente tamaño y memoria de enlazado). Además **aborta con un mensaje claro** si detecta que su base cayó en `tmpfs`.

---

## 2-bis. El veredicto se lee en el TEXTO, nunca en el código de salida

**Fallo real del Tech-Lead, 2026-07-19.** Corrió el gate canalizando la salida a `tail` para ver el final:

```bash
./scripts/mutation-gate.sh sovereign-data-fetcher 2>&1 | tail -30   # ❌
```

El script devolvió `1` (**NO APTO**, 49 supervivientes) pero **la tubería devuelve el código del último comando**, que fue `tail` — o sea `0`. El sistema reportó "completado con éxito". Estuvo a un paso de reportarle al propietario "gate superado" sobre una crate con 49 huecos.

**Reglas:**
- El veredicto es la línea `GATE APTO` / `GATE NO APTO` **del texto**. Léela.
- Si canalizas, el código de salida que vale es el de la tubería (`PIPESTATUS[0]` en bash), no el del filtro.
- Mejor: **no canalices**. Deja que el script imprima su resumen, que para eso lo tiene.
- Regla general, más allá de este gate: **un "completado con éxito" del sistema no es un veredicto de calidad.** Solo dice que un proceso terminó.

## 3. Por qué NO lo corren los subagentes

No es preferencia de estilo, es economía medida. El 2026-07-18 **tres agentes consecutivos agotaron su presupuesto completo esperando** una corrida —consultando el archivo de salida, viendo que no terminaba, volviendo a consultar— y **ninguno entregó nada**. El trabajo estaba hecho; el presupuesto se fue en la sala de espera.

Una corrida son ~18 minutos de CPU y **cero tokens** si la lanza quien puede hacer otra cosa mientras tanto.

**La división correcta:**
- El **ingeniero** escribe las pruebas y entrega el código.
- El **Tech-Lead** corre el gate y certifica.
- Si hay supervivientes, se los devuelve **con el diagnóstico hecho** —qué mutante, en qué función, por qué esa prueba no lo detecta— para que el ingeniero gaste su presupuesto escribiendo, no diagnosticando.
- El **QA** juzga el resultado: qué es hueco real y qué es equivalente genuino.

---

## 4. Cómo se juzga un superviviente

**Estándar: 0 supervivientes.** `timeout` y `unviable` no cuentan como supervivientes.

- **`timeout`** = el mutante colgó la prueba. Cuenta como **captura**: la prueba sí detecta el cambio, solo que colgándose en vez de fallando. Ejemplo real: mutar `attempt += 1` a `attempt *= 1` deja el contador en cero para siempre y cuelga la prueba de contención sostenida.
- **`unviable`** = el mutante ni compila. No es un cambio posible.

### El único superviviente aceptable: el equivalente genuino

Un mutante es **equivalente** cuando ninguna prueba puede matarlo porque el cambio que introduce **no puede alcanzar el valor de retorno**. Se acepta SOLO con la demostración aritmética escrita en un comentario junto al código.

**"Es trivial" NO es justificación.** El Tech-Lead traza la demostración él mismo antes de aceptarla.

**Ejemplo real y correcto** (`round_half_to_even_div_i128`, STORY-051): mutar `<` a `<=` en la detección de signo `(numerator < 0) != (denominator < 0)`.
- Rama del divisor: la función retorna antes si el divisor es cero → el cambio es inalcanzable.
- Rama del dividendo: si el dividendo es cero, el signo calculado **sí** difiere, pero el valor absoluto redondeado es cero y `signo × 0 = 0` en ambos casos → el bit alterado nunca llega al retorno.
- Con ambos distintos de cero, `<` y `<=` coinciden.

Exigir una prueba ahí obligaría a escribir una aserción que no puede fallar nunca — lo contrario de lo que queremos.

---

## 5. Los dos olores que el gate destapa una y otra vez

**La aserción autorreferencial.** Una prueba que compara la salida de una función consigo misma. Expresa "es determinista", pero para una función pura eso es gratis por construcción del lenguaje: la prueba pasa con cualquier implementación, incluida una que devuelva siempre vacío. La forma correcta es: entradas **distintas** → salidas **distintas**; entradas equivalentes **construidas por separado** → misma salida.

**La prueba escrita para matar el mutante.** Tras un NO APTO, entra una tanda de pruebas bajo presión de cerrar el gate. Una prueba que **fija un valor sin explicar por qué ese valor es el correcto** es deuda disfrazada de cobertura — y peor que no tenerla, porque el verde hace que nadie vuelva a mirar. El QA debe buscarlas explícitamente y reportarlas **aunque el gate pase**.
