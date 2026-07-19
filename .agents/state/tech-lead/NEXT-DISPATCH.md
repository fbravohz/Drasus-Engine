# Cola de despacho — sesión fresca

> ⚠️ **DESPACHADOS al cierre de la sesión del 2026-07-18 (el propietario dijo "procede"):** el **punto 1** (ADR del esquema de campos, Architect Sonnet) y el **punto 2** (corpus legal, General Counsel Sonnet). Corren en segundo plano; **el Tech-Lead se quedó sin contexto para auditarlos**.
> **El punto 2 YA ENTREGÓ (2026-07-18):** corpus legal en `docs/legal/` con índice `docs/LEGAL.md`, prefijo **LDR** (*Legal Decision Record*), siete decisiones `LDR-0001..0007` + `ROADMAP-TYC.md`. Existencia verificada por el TL; **falta auditar el contenido**. Dos cosas que la sesión fresca debe llevar al propietario: (a) el abogado **acotó** su cláusula de responsabilidad — "ninguna responsabilidad de nada" es más amplia de lo que un tribunal sostendría, y una cláusula descartable en bloque protege menos que una acotada; (b) la cláusula de **mayor riesgo de todo el corpus** no es ninguna de datos, es la de ADR-0143 (el trabajo del usuario del tier gratuito puede terminar operado con capital del propietario) — exige abogado de consumo antes de publicarse donde haya protección al consumidor fuerte.
> **Lo primero de la sesión fresca:** auditar esas dos entregas antes de encadenar nada. Comprobar que el ADR del esquema existe con su índice actualizado y sin reescrituras completas, y que el corpus legal quedó donde el abogado dijo.
> **NO despachados** por riesgo de colisión sobre el índice de ADR: puntos 0 (tripleta), 1-bis (costura de futuros) y 1-ter (habilitación por capacidad de dato). Van en la sesión fresca, uno a uno.

> Escrito por el Tech-Lead el 2026-07-18 al agotarse la sesión. El propietario ya autorizó todo lo que está aquí; la sesión nueva **solo despacha**, no vuelve a preguntar. Orden de arriba abajo: hay dependencias reales.
> Contexto completo en `PROGRESS.md` (bloque del 2026-07-18). Fuentes de insumo: `.agents/plans/esquema-canonico-datos-mercado.md`, `.agents/plans/veredicto-legal-datos-mercado.md`, `docs/adr/ADR-0154.md`.

---

## 0. Architect (Sonnet) — tripleta de linaje símbolo × marco temporal × bróker

**Estado: DETENIDO por el propietario antes de terminar (2026-07-18). Hay que redespacharlo.** Verificar primero si alcanzó a dejar ediciones a medias en ADR-0154 / ADR-0141 / features de ingesta, y sanearlas antes de relanzar.

Encargo: canonizar la tripleta como **identidad mínima** de toda serie de mercado — el EURUSD de un bróker NO es el de otro, y cada uno con su marco temporal (patrón `EURUSD-1M-PEPPERSTONE`, tomado de StrategyQuant). Dos series que difieran en cualquiera de los tres componentes son distintas y no se funden jamás.

Debe cubrir: forma canónica del identificador y su contraste con el formato que ADR-0141 ya fija · por qué el mismo símbolo en dos brókers es dato distinto, con la razón verificable (cotizaciones, spread, sesión, redondeo, swaps, filtrado de ticks) · el invariante de no-fusión expresado de forma que el sistema lo haga cumplir, no que lo desaconseje · series derivadas (barras derivadas de ticks heredan el bróker y se marcan como derivadas, no observadas) · consecuencias en particionado, validez de backtests entre brókers y registro de procedencia · y si hace falta un cuarto componente (tipo de cuenta, servidor del bróker), con el caso que lo demuestre.

Propagar a `sovereign-data-fetcher`, `data-normalization-layer`, `hive-partition-manager`, `data-import-wizard` y la sección del SAD que toque. Edición quirúrgica.

---

## 1. ✅ ENTREGADO (2026-07-18) — ADR-0155, esquema canónico de datos de mercado

**Ya no bloquea. El punto 3 (retrofit del transformador) queda DESBLOQUEADO.** Pendiente de auditoría de contenido por el TL.

Canonizado: `QuoteTick` y `TradeTick` como tipos distintos · spread declarado por la fuente como observación independiente · volumen en cuatro magnitudes · barras con campos de futuros y opciones esquema-listo · profundidad de libro de nivel 2 y 3 · y el principio general **"guarda el hecho, calcula la interpretación"**.

**Dos correcciones que el Architect hizo al insumo del Quant, y que hay que verificar:** (a) los campos de auditoría del Grupo I gobiernan los ledgers SQLite de las features, **no** cada fila del dataset Parquet de mercado — habría roto el paralelismo de escritura y duplicado procedencia que `data_snapshot_id` ya cubre; (b) asignó feature productora a los tipos de profundidad de libro, que el insumo dejaba huérfanos.

También enmendó ADR-0137 (el tipo `Tick` del catálogo se parte en dos) y tocó ADR-0154, ADR-0140, cinco features, `ingest.md` y `SAD-08`. El código sigue intacto: poblar los tipos en Rust es Story de ingeniería.

<details><summary>Encargo original (histórico)</summary>

## 1-histórico. Architect — ADR del esquema de campos (BLOQUEA el transformador)

**Modelo:** Sonnet. **Prioridad: la más alta** — es lo único que bloquea código.

Convertir `.agents/plans/esquema-canonico-datos-mercado.md` (entrega del Quant, ya aprobada de facto por el propietario) en ADR formal, y con él llenar el tipo vacío `crates/shared/src/types/mod.rs:51`.

Debe canonizar: cotización y ejecución como dos tipos distintos · compra y venta obligatorias más el spread declarado por la fuente como observación independiente · las cuatro magnitudes de volumen · opciones y los tres niveles de profundidad de libro · y la regla de que el precio ajustado de acciones y el contrato continuo de futuros **no se persisten como columna**, son vista derivada (un split o un rolo reescribirían el pasado, prohibido por ADR-0141).

Restricción: el ingeniero NO debe poblar el tipo por inferencia antes de que exista este ADR.

</details>

## 1-bis. Architect — métodos de ajuste histórico y de costura de futuros

**Modelo:** Sonnet. Va junto con el punto 1 (misma familia de decisión) o inmediatamente después.

El propietario aprobó el principio **"guarda el hecho, calcula la interpretación"** y pidió que quede canonizado sin excepción, con **todos los métodos posibles** definidos — no uno solo por defecto.

Debe cubrir:
- **Ajuste por eventos corporativos** (splits, splits inversos, dividendos): el crudo se persiste intacto; la tabla de eventos es aparte; el ajuste se aplica al leer. Definir los métodos (proporcional sobre precio, con y sin dividendos, y el crudo sin ajustar) y cuál es el defecto de cada uso.
- **Costura de contratos de futuros:** definir **todos** los métodos de empalme y los criterios de rolo, no solo el habitual. Al menos: ajuste retroactivo aditivo (tipo Panamá), ajuste proporcional o por ratio, empalme crudo sin ajustar, y contrato individual sin coser. Y los disparadores de rolo: cruce de volumen, cruce de interés abierto, días fijos antes del vencimiento, y primer día de aviso. Cada método cambia la historia entera, así que debe quedar **explícito, elegible por el usuario y registrado en el linaje del resultado** — dos backtests con distinto método no son comparables.
- **Anclaje empírico:** el propietario recuerda que en NinjaTrader nunca vio una mala costura pero no recuerda el método. Investigar qué hacen NinjaTrader, TradingView y los proveedores de datos de futuros, y **verificar** en vez de afirmar de memoria.
- Consecuencia obligatoria: el método usado forma parte de la identidad de la serie derivada (encaja con la tripleta del punto 0).

## 1-ter. Architect — habilitación de funciones por capacidad de dato (escalera honesta)

**Modelo:** Sonnet. **Aprobado por el propietario (2026-07-18).** Depende de que exista la tabla de capacidades (puntos 0 y 1).

Canoniza el principio: **una función se habilita o no según el dato que el usuario realmente tiene, y el sistema lo sabe antes de que lo intente.** No es una lista arbitraria de funciones por plan: es la consecuencia de qué entrega su fuente.

Debe definir:
- **Clasificación de cada feature por el dato mínimo que exige** (barras de N minutos / ticks de ejecución / ticks de cotización con compra y venta / profundidad de libro). Recorrer el catálogo de features y etiquetarlas. Las que dependen de tick —order flow, microestructura, modelado de spread real, simulación de llenado a nivel tick— quedan marcadas como tales.
- **Resolución en el momento de conectar:** al dar de alta una fuente, el sistema contrasta lo que esa fuente entrega contra lo que cada feature exige, y **le dice al usuario qué desbloqueó y qué no, con el motivo y qué le faltaría**. Ejemplo del propietario: *"tu bróker da barras desde 2016 y ticks desde 2023; con esto tienes X, para Y necesitas un feed de ticks profundo"*.
- **Presentación:** la función no disponible se muestra **visible y apagada, con el motivo**, nunca oculta. Convierte la limitación en camino de mejora en vez de en una pantalla vacía que el usuario descubre después de pagar.
- **Relación con el substrato de monetización:** cómo se compone esta habilitación por dato con la habilitación por plan (`plan-tier-quota`, cimiento #3). Son dos puertas distintas y ambas deben poder negar; definir la precedencia y el mensaje resultante.
- **Regla anti-promesa:** ninguna función puede anunciarse como disponible si el dato del usuario no la sostiene. Es verificable por el sistema, no por el material comercial.

Salida: ADR (nuevo o enmienda, decide) + propagación a las features afectadas y a `plan-tier-quota`. Nota para el Tech-Lead: la parte de interfaz es Story posterior, no de esta iteración.

## 2. General Counsel — corpus legal con estructura de decisiones

**Modelo:** Sonnet. Independiente, puede ir en paralelo con lo anterior.

Crear el corpus legal del proyecto **con la misma estructura que los ADR**: carpeta propia, índice, y un documento por decisión con prefijo y numeración propios (el abogado elige nombre y prefijo, justificándolo).

Debe volcar todo lo dictaminado en esta sesión y dejar la base para redactar los términos y condiciones del servicio:
- Los tres veredictos de `.agents/plans/veredicto-legal-datos-mercado.md`.
- **Decisión del propietario, vinculante:** el usuario es responsable del proveedor de datos que elija y de la calidad de lo que importe. Drasus no asume responsabilidad alguna sobre eso. Debe traducirse a cláusula redactada, con su límite real (hasta dónde esa exención es oponible y dónde no lo es).
- **Decisión del propietario:** sí se graba el flujo de datos en vivo. Falta dictaminar su legalidad y alcances — es un encargo abierto para el abogado, no un hecho consumado. **Postura APROBADA por el propietario (2026-07-18); el abogado la dictamina y afina, no la reabre:** usar el dato en vivo para operar es uso normal y permitido (es para lo que el bróker vende el feed); el riesgo aparece al **almacenarlo** y convertirlo en conjunto de datos, y se dispara si ese conjunto alimenta la agregación monetizable. Accionables derivados, ninguno de los cuales exige maquinaria nueva: (a) lo grabado en vivo nace marcado **no agregable por defecto**, salvo que los términos de esa fuente lo permitan expresamente — es la misma marca de DEBT-026; (b) no sale de la máquina del usuario, que ya es la arquitectura vigente; (c) el permiso de grabación en vivo se verifica **por conector** y se declara como columna de la tabla de capacidades. **Sobre la portabilidad bancaria: no hay nada que ajustar** — era un argumento defensivo disponible, no un cimiento de nada construido; estar suspendido solo significa que no podemos apoyarnos en él.
- La frontera de agregación: se agrega lo que la licencia de la fuente permite, se omite lo que no. Requiere la marca por origen (DEBT-026).
- Lo ya cubierto por el substrato: consentimiento, portabilidad y olvido, roles de operador, cuentas verificadas.

## 3. ✅ ENTREGADO Y VERIFICADO EN VERDE (2026-07-18) — retrofit del transformador a ADR-0155

**El TL reprodujo: 45 pruebas verdes** (28 unitarias + 17 de integración, subieron desde 36) **y el comando de verificación devuelve el esquema nuevo con las cuatro magnitudes de volumen como columnas independientes.** Falta auditar el detalle del código y el mapeo criterio→prueba.

**⚖️ TRES DECISIONES ABIERTAS — el ingeniero PARÓ en vez de resolverlas solo (comportamiento correcto, exigido en su encargo). Fallo pendiente del Tech-Lead:**

1. **`Bar.vwap` frente al principio "guarda el hecho, calcula la interpretación".** El precio medio ponderado por volumen lo **calcula Drasus**; ADR-0155 prohíbe persistir interpretaciones. Hoy no hay violación viva porque la barra aún no se escribe a Parquet, pero **hay que fallarlo antes de que exista ese escritor**. Opciones: sacarlo del tipo persistido y calcularlo al leer (coherente con el principio), o justificarlo como excepción documentada. **Recomendación del TL: sacarlo** — es exactamente la forma de campo que el principio describe.
2. **`crates/shared/src/types/mod.rs` quedó desactualizado** frente a la enmienda de ADR-0137: sigue declarando `Tick` en vez de `QuoteTick`/`TradeTick`/`DepthLevel`/`OrderEvent`. El ingeniero no lo tocó porque rompería `sovereign-data-fetcher`. **Necesita Story corta dedicada** que actualice el catálogo y repunte a sus consumidores.
3. **Campos de identidad ausentes en `Bar`** (instrumento, mercado, tipo de barra). Señalados, no añadidos por estar fuera del alcance. **Encajan con la tripleta del punto 0** — resolver junto con ella, no por separado.

<details><summary>Encargo original (histórico)</summary>

**Corriendo en segundo plano; el TL se quedó sin contexto para auditarlo.** Al arrancar: comprobar si terminó, reproducir su evidencia (`cargo test -p hybrid-data-transformer`, `cargo clippy`, `cargo run -p app -- verify hybrid-data-transformer`), y confirmar dos pruebas concretas que exigí — que las cuatro magnitudes de volumen no se colapsan ni se rellenan por defecto, y que una cotización nunca contribuye a una barra agregada. Si el ingeniero paró y reportó una contradicción de ADR-0155, resolverla antes de seguir.

**Solo entonces** despachar el punto 4 (puente → interfaz → calidad → validación cuantitativa).

<details><summary>Encargo original (histórico)</summary>

## 3-histórico. Rust-Engineer — rehacer la agregación del transformador

**Modelo:** Sonnet. **Depende de que el punto 1 esté cerrado.** Modo Docente (continúa STORY-051).

El motor entregado agrega asumiendo un solo precio. Con el esquema canonizado debe producir barras con compra, venta y spread reales. Orden: `docs/execution/STORY-051-hybrid-data-transformer.md` — actualizar §4.1 y §6 antes de despachar.

</details>

## 4. Bridge → Flutter → QA → Quant (cierre de STORY-051)

Cadena ya especificada en §4.2–§4.4 de la Orden. **No despachar el puente hasta que el punto 3 esté cerrado**: congelaría el contrato equivocado.

## 5. Task nueva — secuenciación de conectores

Sin código. Reemplaza a TASK-052, que quedó replanteada. Pregunta: **qué conectores se construyen primero y con qué tabla de capacidades**, bajo ADR-0154.

Insumo ya verificado y disponible: vía de exportación de terminal (cualquier bróker) · interfaz abierta de cTrader (verificar por bróker) · Darwinex · Swissquote · para futuros, los proveedores que el usuario pague. **Preferencia del propietario:** cripto le interesa **con apalancamiento** (futuros o CFDs sobre cripto), lo que se obtiene de brókers, no de exchanges — eso deja a Binance fuera del camino crítico.

## 6. Deuda — repaso conjunto (instrucción del propietario)

DEBT-025, 026 y 027 se acumulan y **se repasan todas juntas al cerrar los pendientes en curso**, antes de arrancar cualquier feature nueva. No pagarlas sueltas.

---

## Decisiones del propietario tomadas en esta sesión (vinculantes, no volver a preguntar)

| # | Decisión |
|---|---|
| 1 | **Drasus no provee datos.** Cada usuario trae los suyos. Canonizado en ADR-0154. |
| 2 | **Sí se graba el flujo en vivo.** Pendiente el dictamen legal de alcances, no la decisión. |
| 3 | **Conector de Binance:** se convierte en conector con credenciales del usuario, como los demás → DEBT-027. Es la prioridad más baja del propietario. **Prohibido pedir permiso a Binance.** Descartarlo del producto es alternativa aceptable. |
| 4 | **Responsabilidad del dato:** del usuario, íntegra. Drasus no asume ninguna. |
| 5 | **Linaje = tripleta** símbolo × marco temporal × bróker. Aprobado y en canonización. |
| 6 | **Agregación:** se agrega lo permitido, se omite lo bloqueado. Requiere la marca por origen. |
| 7 | Todo lo entregado por el Architect en ADR-0154 queda **aprobado**. |
