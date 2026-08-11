# Discriminación de Pruebas

Cómo se demuestra que una prueba discrimina de verdad, y los cuatro patrones de prueba que dan verde sin probar nada.

Slug ID: `TEST-DISCRIMINATION`.

Tamaño: `7368 caracteres (~1842 tokens aprox., ratio 4:1)`.

---

## TEST-DISCRIMINATION-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/knowledge/mutation-gate.md]] | Quién corre el gate de mutación y cómo se juzga un superviviente. | Al cerrar una Story con lógica de correctitud, o al juzgar supervivientes. | TL + QA |
| [[.agents/knowledge/ledger-pattern.md]] | Las cuatro pruebas que el patrón de ledger append-only exige. | Al crear o tocar una tabla append-only. | TL + ingenieros |

---

## TEST-DISCRIMINATION-FV — Filtro de Verificación

**Aplica a:** ingenieros al entregar pruebas, y al QA al auditarlas.

* **TEST-DISCRIMINATION-FV-1:** ¿demostré la discriminación con el rojo pegado en el reporte, en vez de afirmarla? (`TEST-DISCRIMINATION-1`)
* **TEST-DISCRIMINATION-FV-2:** ¿ninguna prueba compara una función consigo misma ni afirma un valor que ella misma acaba de producir? (`TEST-DISCRIMINATION-2`)
* **TEST-DISCRIMINATION-FV-3:** ¿cada garantía de comportamiento se mide, en vez de asumirse? (`TEST-DISCRIMINATION-3`)
* **TEST-DISCRIMINATION-FV-4:** ¿ningún valor fijado quedó sin la razón por la que ese valor es el correcto? (`TEST-DISCRIMINATION-4`)
* **TEST-DISCRIMINATION-FV-5:** si hubo un pase mecánico en N sitios, ¿confirmé 0 ocurrencias residuales con `grep` antes de reportar? (`TEST-DISCRIMINATION-5`)

De ser satisfactoria cada validación de la lista, declara `TEST-DISCRIMINATION - FILTRO DE VERIFICACION APROBADO`.

---

## TEST-DISCRIMINATION-1 — Demostración Obligatoria de Discriminación

Afirmar que una prueba discrimina no basta. Se demuestra con hechos, en cuatro pasos que cuestan dos minutos y son imposibles de fingir.

**Aplica a:** todos los ingenieros, por cada criterio de aceptación y por toda prueba que cubra una garantía de comportamiento.

* **TEST-DISCRIMINATION-1.1 — Los Cuatro Pasos:**

  1. Rompe a propósito el comportamiento que la prueba dice cubrir — comenta la línea, invierte la condición, devuelve una constante.
  2. Corre la prueba y pega la salida **en rojo** en tu reporte.
  3. Revierte la rotura.
  4. Corre la prueba y pega la salida **en verde**.

* **TEST-DISCRIMINATION-1.2 — Sin Rojo, No Está Verificada:**

  si el reporte no trae el rojo, la prueba se considera no verificada, sin importar cuán razonable se vea el código.

**Filtro de verificación:** `TEST-DISCRIMINATION-FV-1`.

---

## TEST-DISCRIMINATION-2 — Prohibida la Aserción Autorreferencial

Una prueba que compara la salida de una función consigo misma, o que afirma un valor que la propia función acaba de producir, no es una prueba: es una tautología que pasa con cualquier implementación.

**Aplica a:** todos los ingenieros y al QA, en hashes, serializadores, formateadores, claves derivadas y cualquier función de transformación.

* **TEST-DISCRIMINATION-2.1 — Por Qué Engaña:**

  `assert_eq!(f(&x), f(&x))` sí expresa la idea "el resultado es estable entre corridas". El problema: si la función es pura, esa estabilidad es gratis por construcción del lenguaje. La prueba pasa igual si la función devuelve siempre vacío. Se probó la propiedad en la que el autor estaba pensando, no la que importa: **que la salida dependa realmente de la entrada**.
  * ❌ *Incorrecto:* `assert_eq!(compute_logic_hash(&plan), compute_logic_hash(&plan))` — pasa aunque la función devuelva `String::new()`.
  * ✅ *Correcto:* entradas **distintas** producen salidas **distintas**, y entradas **equivalentes construidas por separado** producen la misma salida.

* **TEST-DISCRIMINATION-2.2 — Nunca la Misma Variable a Ambos Lados:**

  la regla mecánica que evita el caso entero: si el mismo identificador aparece en los dos lados de la comparación, la prueba está mal construida.

**Filtro de verificación:** `TEST-DISCRIMINATION-FV-2`.

---

## TEST-DISCRIMINATION-3 — Las Garantías de Comportamiento se Miden, No se Asumen

Concurrencia, recuperación, atomicidad y límites de recursos no se prueban afirmando que ocurrieron: se instrumentan y se mide el valor real.

**Aplica a:** todos los ingenieros y al QA, en toda prueba de garantía de comportamiento.

* **TEST-DISCRIMINATION-3.1 — Ciclo Rojo→Verde:**

  escribe la prueba de cada criterio antes o junto con el código, y verifica que **falla** sin la implementación. Una prueba que pasa aunque el comportamiento esté ausente es inútil.

* **TEST-DISCRIMINATION-3.2 — Concurrencia: el Umbral Importa:**

  la concurrencia se prueba con un contador atómico de tareas activas que registra el pico, bajo runtime `#[tokio::test(flavor = "multi_thread")]`.
  * ❌ *Incorrecto:* afirmar `pico > 0` — un bucle secuencial con un `Semaphore` decorativo da verde con `pico = 1` y no prueba nada.
  * ✅ *Correcto:* afirmar `pico >= 2`, que solo se cumple si dos tareas corrieron de verdad en paralelo.

* **TEST-DISCRIMINATION-3.3 — Toda Aserción Debe Poder Caerse:**

  para cada criterio crítico, razona o demuestra que la aserción se cae con la implementación incorrecta. Una prueba verde que pasaría incluso con la implementación rota no prueba el criterio.

**Filtro de verificación:** `TEST-DISCRIMINATION-FV-3`.

**Lectura bajo demanda:** [[.agents/knowledge/ledger-pattern.md]] — si la garantía en juego es la atomicidad de un ledger append-only; ahí viven las cuatro pruebas concretas que ese patrón exige.

---

## TEST-DISCRIMINATION-4 — Valor Fijado Sin Justificar Es Deuda Disfrazada de Cobertura

Una prueba escrita para cerrar un gate bajo presión, que fija un valor sin explicar por qué ese valor es el correcto, es peor que no tenerla: viene con el número verde que hace que nadie vuelva a mirar.

**Aplica a:** todos los ingenieros al escribir pruebas, y al QA al auditarlas.

* **TEST-DISCRIMINATION-4.1 — Todo Valor Esperado Trae Su Razón:**

  el valor fijado se deriva de la especificación, de un cálculo mostrado, o de una fuente citada — nunca de "lo que salió al correrlo".

* **TEST-DISCRIMINATION-4.2 — El QA lo Reporta Aunque el Gate Pase:**

  este olor no lo detecta ninguna métrica de cobertura ni el gate de mutación. Se reporta igual.

**Filtro de verificación:** `TEST-DISCRIMINATION-FV-4`.

---

## TEST-DISCRIMINATION-5 — Pase Mecánico en N Sitios: Confirma Cero Residual

Cuando el trabajo pide sustituir un patrón en varios sitios, un compilado verde no detecta un sitio olvidado si el patrón viejo sigue siendo válido.

**Aplica a:** todos los ingenieros, ante cualquier renombrado, reemplazo de API o migración de patrón que toque más de un archivo.

* **TEST-DISCRIMINATION-5.1 — Grep Antes de Reportar:**

  no te fíes de haber recorrido "los que recuerdas". Corre `grep -rn '<patrón viejo>'` sobre el alcance declarado y confirma **0 ocurrencias** antes de reportar.
  * ❌ *Incorrecto:* dar por completo un cambio de `Uuid::new_v4()` a `Uuid::now_v7()` porque el proyecto compila — ambas formas compilan igual.
  * ✅ *Correcto:* `grep -rn 'new_v4' <alcance>` devolviendo cero, pegado en el reporte.

**Filtro de verificación:** `TEST-DISCRIMINATION-FV-5`.
