# TASK-063 · Formato estándar de ingesta y doctrina compra/venta

| Campo | Valor |
|---|---|
| **ID** | TASK-063 |
| **Tipo** | Decisión de arquitectura + doctrina de dominio |
| **Estado** | Abierta |
| **Abierta** | 2026-07-19 |
| **Origen** | Directiva del propietario, sesión 2026-07-19 |
| **Destinatarios** | Quant-Engineer (Etapa 0.3) → Architect (Etapa 0.4) |
| **Dominio** | Ingesta — contrato de entrada de datos de mercado |
| **Relacionadas** | ADR-0154 (conectores, no datos), ADR-0155 (esquema canónico), DEBT-029 (contratos laxos), TASK-060 (muestra real, bloqueada en el propietario) |

---

## 1. Qué disparó esta Task

Al explicar el riesgo de los archivos de ticks con filas de naturaleza mixta, el propietario fijó tres posiciones. Las tres son directivas, no propuestas — lo que queda por decidir es **cómo** se implementan, no **si**.

---

## 2. Directivas del propietario (vinculantes)

### D1 — No se ingesta cualquier cosa

**No se aceptan archivos de formato arbitrario.** Existe un formato estándar al que el dato debe llegar. O el usuario transforma su archivo afuera, o Drasus lo transforma **declarándolo abiertamente y mostrando vista previa** de cómo queda el dataset tras la transformación.

Preferencia declarada del propietario: **que el usuario transforme afuera.** Razonamiento textual: *"si le paso archivos pendejos a Strategy Quant, yo no podría demandarlos por darme un backtest pendejo — me van a decir 'tú metiste ejecuciones en tus cotizaciones'"*. La responsabilidad del contenido es de quien exporta.

**Pregunta abierta para el General-Counsel:** ¿transformar el archivo transfiere responsabilidad sobre el resultado? Hipótesis de trabajo del Tech-Lead, a confirmar: **transformar con mapeo declarado y aprobado por el usuario no transfiere responsabilidad; adivinar el mapeo sí.** La diferencia legal está en si el usuario vio y aceptó la interpretación.

### D2 — Plantillas por plataforma

Los formatos de exportación difieren por plataforma (NinjaTrader, MetaTrader, Interactive Brokers y sucesivas). **Se admiten plantillas estándar por plataforma**, con el mapeo de columnas ya resuelto.

Para lo que no encaje en ninguna plantilla: **mapeo explícito donde el usuario declara qué es cada columna suya** contra las columnas de Drasus, que son innegociables. Nunca mapeo automático por adivinanza de nombres de columna.

Rechazar sin más es más seguro que transformar en silencio, pero cuesta usuarios: nadie exporta limpio a la primera. La plantilla + el mapeo declarado cubren ambos lados.

### D3 — La ejecución simulada solo lee compra o venta

Posición del propietario: *"sin importar cuál sea el activo siempre operamos en el bid o el ask, según la dirección; si bid y ask son iguales estamos ante un activo sin spread"*.

**El Tech-Lead confirma el fondo y matiza el alcance** (a validar por el Quant):

- ✅ **Invariante duro, sin excepción:** el precio de una operación simulada sale de **compra o venta según la dirección**. Nunca de un campo genérico de precio. Un backtest que ejecuta al punto medio asume un precio al que nadie opera jamás — es optimismo sistemático incorporado en cada resultado.
- ⚠️ **Pero eliminar el concepto de precio negociado rompería trabajo ya especificado.** En acciones, futuros y cripto *"se negoció a X"* es un hecho real, no un promedio inventado. Y `docs/features/algorithmic-bars.md` construye las barras Renko, de Rango, de Volumen y de Tick **contando operaciones reales**, no cotizaciones. Sin ese dato no se pueden producir.
- El volumen de Forex al contado no es volumen real: es conteo de cambios de precio. Otra razón para no mezclar magnitudes (ya cubierto por las cuatro magnitudes independientes de ADR-0155).

**Formulación propuesta, más fuerte que prohibir el nombre:**

1. El motor de ejecución **solo puede leer compra o venta**. Leer un campo de precio negociado para calcular un fill es defecto crítico.
2. Una columna de precio **sin significado declarado se rechaza**, no se adivina. El problema nunca fue la palabra: fue un precio sin dueño declarado.
3. El dato de operación **se conserva** — es materia prima legítima para tipos de barra, no para simular fills.
4. Si el dataset trae volumen y dirección junto a cotizaciones, es señal de mezcla: **se rechaza con error de formato explícito**, nunca se consume parcialmente en silencio.

### D4 — La declaración del mapeo es evidencia, y como tal es inmutable

**Directiva del propietario, 2026-07-19:** el acto por el cual el usuario declara qué columna suya corresponde a cuál de las nuestras debe ser **rastreable, auditable e inmutable**. Textual: *"así tenemos evidencia de que el usuario mapeó tales columnas con tales y presionó el botón"*.

Esto convierte el mapeo en un **registro append-only**, no en configuración. La diferencia es sustantiva: una configuración se edita y la versión anterior desaparece; una evidencia no puede desaparecer, porque el día que un backtest salga raro hay que poder reconstruir **qué declaró el usuario, cuándo, y sobre qué archivo**.

**Es lo que cierra la exposición legal de D1.** Sin registro, "el usuario aceptó el mapeo" es una afirmación nuestra sin respaldo. Con registro inmutable, es un hecho verificable. Sin esto, transformar sí nos acerca a cargar con el resultado.

**Construcción:** sigue el molde canónico de `.agents/knowledge/ledger-pattern.md` — **ese documento, no la feature que ya lo tenga implementado**. Los siete puntos y las cuatro pruebas obligatorias aplican íntegros.

**Qué debe quedar sellado en cada fila, como mínimo (a cerrar por el Architect):**
- Identidad del archivo origen — nombre y huella del contenido, para que no se pueda alegar que se declaró sobre otro archivo.
- El mapeo completo columna-por-columna, tal como se le mostró al usuario.
- La plantilla de plataforma aplicada, si hubo una, y su versión.
- El instante de la aceptación, y la vista previa exacta que el usuario tenía delante al aceptar.
- Encadenado a la fila anterior por hash, como todo ledger.

**Pregunta abierta:** ¿el mapeo declarado queda ligado al dataset resultante de forma que cualquier resultado derivado pueda rastrearse hasta la declaración que lo originó? El Tech-Lead cree que sí y que es el punto entero del registro, pero la mecánica —si viaja en el sello de auditoría de las barras o por referencia— la decide el Architect.

---

## 3. Qué hay que decidir

1. **¿Rechazo puro, o transformación con vista previa?** (D1) — el propietario prefiere rechazo; la vista previa con mapeo declarado es la alternativa que no pierde usuarios. Decide el Architect con veredicto legal del General-Counsel.
2. **Catálogo inicial de plantillas por plataforma** (D2) — cuáles se soportan al arrancar y cuál es la estructura de una plantilla. Depende de TASK-060 (muestras reales), **bloqueada en el propietario**.
3. **Redacción canónica de la doctrina compra/venta** (D3) — ¿enmienda a ADR-0155 o ADR nuevo? Es doctrina de dominio: pasa por las 7 comprobaciones de `critical-domain-reasoning.md` y por gate del Quant antes de sellarse.
4. **Relación con DEBT-029** — los contratos de entrada que hoy aceptan campos desconocidos en silencio son exactamente el mecanismo que D1 prohíbe. Verificar si esta Task absorbe esa deuda o si son capas distintas.

---

## 4. Veredicto legal (General Counsel, 2026-07-19)

**Veredicto:** APTO-CON-CONDICIONES. Documento completo: [LDR-0008](../legal/LDR-0008.md).

**Respuesta directa a la pregunta del propietario:** transformar el archivo del usuario **no** transfiere a Drasus el peso legal del resultado, siempre que el mapeo lo declare el propio usuario sobre una vista previa real de su dato ya transformado (no solo nombres de columna) y esa declaración quede registrada como evidencia inmutable (D4). Adivinar el mapeo sí lo transfiere — es exactamente lo que D2 ya prohíbe. La hipótesis del Tech-Lead **se confirma**, con una corrección de alcance: las **plantillas por plataforma** (D2, primera vía) no son el mismo caso que el mapeo manual declarado (D2, segunda vía). Una plantilla la afirma Drasus, no el usuario — si la plantilla de NinjaTrader/MetaTrader/IB está mal construida, ese es un defecto del propio producto (misma familia que un bug de software), no una responsabilidad que el usuario haya aceptado. Detalle de los tres casos, la comprobación de límite degenerado y las cuatro condiciones que sostienen la protección: [LDR-0008](../legal/LDR-0008.md).

**Criterios de aceptación técnicos que este veredicto añade al ledger de D4** (los cinco campos que D4 ya lista son necesarios, no suficientes):

1. **Bandera explícita del origen del mapeo** (`mapeo_manual_usuario` vs. `plantilla_plataforma`), campo propio y consultable — es el hecho que decide quién responde ante un resultado cuestionado.
2. **Versión del motor de transformación y del esquema canónico** (ADR-0155) contra el que se declaró el mapeo — sin esto no se puede distinguir "el usuario declaró mal" de "el transformador tenía un defecto en esa versión".
3. **La vista previa registrada debe ser de datos transformados reales, no de la tabla de correspondencias columna→columna** — un registro que solo guarda el esquema no sostiene la defensa de consentimiento informado que se le pide sostener.
4. **Trazabilidad obligatoria hasta el dataset y el backtest que un mapeo produjo** — responde la pregunta abierta de D4 ("¿el mapeo declarado queda ligado al dataset resultante?"): sí, es requisito, no preferencia. La mecánica exacta (sello de linaje vs. referencia) queda a criterio del Architect.
5. **La declaración del usuario no exime a Drasus de correr sus propios guardarraíles antes de aceptar el mapeo** — la regla D3 de rechazo de payload mixto y el validador de punto-en-el-tiempo (`pit-data-validator`) deben correr antes de sellar la fila del ledger. Si Drasus deja pasar algo que su propio validador está construido para detectar, eso es defecto de Drasus, no del mapeo declarado.

**Jurisdicciones:** no se abre análisis jurisdiccional completo aquí — la exigibilidad real de la aceptación (clickwrap) queda anotada en `docs/legal/ROADMAP-TYC.md` Tabla 3 como pendiente de abogado colegiado, y la determinación de dónde se constituye Drasus y bajo qué régimen de consumo queda en TASK-053.

**Bloqueo:** ninguno. No hay objeción legal a que el Architect avance con la mecánica de D1/D4 en Etapa 0.4, sujeta a las cinco condiciones de arriba.

---

## 5. Nota de método

El propietario llegó a D3 por razonamiento propio y pidió confirmación o corrección: *"confírmalo o tal vez yo estoy diciendo la pendejada"*. **No lo estaba** — el invariante es correcto y no estaba escrito en ninguna parte. La corrección es de alcance, no de fondo: el hecho negociado se conserva como materia prima aunque nunca se use para simular una ejecución.

---

## 6. Resolución del Architect (2026-07-19)

**Rechazo por defecto + transformación con vista previa como acción explícita** (no son alternativas excluyentes — D1 fija el default, D2 opera el mecanismo). Canonizado en **ADR-0156** (nuevo), que extiende ADR-0154/ADR-0155. **D3 (doctrina compra/venta)** canonizada como tercera regla FIJA del Contrato de Consistencia Conservadora, **ADR-0114** §Reglas (c) — no ADR nuevo, porque ese es exactamente el ADR que ya gobierna qué precio lee el motor de ejecución. **Trazabilidad de D4/LDR-0008:** por referencia estable (ledger de mapeo → `transformation_records` → `data_snapshot_id`), nunca duplicando la declaración dentro de cada fila de mercado — mismo principio que ADR-0155 ya aplica a Grupo I. **DEBT-029:** capas distintas — su punto 1 (decisión de política) queda resuelto por D3/ADR-0156 restricción 5; sus puntos 2–3 (implementación) siguen abiertos, ahora con doctrina que los gobierna. `docs/features/data-import-wizard.md` corregido: eliminaba el defecto legal de mayor exposición (mapeo automático presentado como aceptado) que contradecía D2 directamente. Detalle completo, las 7 comprobaciones de dominio y la mecánica de trazabilidad: `.agents/plans/architect-2026-07-19-escala-barras-ingesta.md`.
