# TASK-065 · Catálogo de Bróker × Instrumento × Sesión — Normalización, Validación de Coherencia y Frontera Backend/UI

| Campo | Valor |
|---|---|
| **ID** | TASK-065 |
| **Tipo** | Decisión de arquitectura + diseño de esquema |
| **Estado** | Abierta |
| **Abierta** | 2026-07-22 |
| **Origen** | El propietario evaluó exportaciones de StrategyQuant X (SQX) y detectó tres problemas de diseño a contrastar contra nuestro modelo |
| **Corrección** | 2026-07-23 (propietario) — esto se productiza como **FEATURES hexagonales con puertos, no como tablas sueltas**. Ver §2-bis. El esquema de §2 es la persistencia que esas features poseen, no el entregable en sí. |

> 🔴 **CORRECCIÓN DE FONDO (2026-07-23).** La primera redacción entregó un esquema de tablas. Eso incumple el invariante de arquitectura de `CLAUDE.md` §1: **toda lógica vive en una Feature** (FCIS + hexágono con puertos tipados, ADR-0137); las tablas son solo el detalle de persistencia que una feature posee, nunca el artefacto de diseño. La causa fue el encuadre del despacho (el Tech-Lead planteó el encargo como "modelo de entidades / normalización de tablas", no como "productiza esto en features"). Se corrige en la nueva **§2-bis (Descomposición en features)**: el esquema de §2 se conserva como la persistencia interna de esas features. Además: el gate de coherencia (§3) se generaliza explícitamente a **todas** las combinaciones de clase de activo (§3-bis), y la zona horaria se declara vía selector con bandera DST (§5, patrón SQX confirmado por el propietario).
| **Destinatarios** | Tech-Lead (despacho) → Quant-Engineer (contrato de timezone/DST, validez de sesión para señal) → Rust-Engineer (implementación cuando se apruebe) |
| **Dominio** | Ingesta — identidad de instrumento, calendario de sesión, parámetros de costo de operación |
| **Relacionadas** | ADR-0154 (tabla de capacidades por bróker), ADR-0155 (esquema canónico, tripleta de identidad), ADR-0156 (formato de ingesta, ledger de mapeo), ADR-0140 (tipo `Instrument`, diferido), ADR-0141 (sesiones derivadas en runtime, timestamps UTC), ADR-0114 regla (c) (bid/ask exclusivo), DEBT-031 (catálogo de brókers inexistente — este Task lo resuelve), DEBT-029 (contratos que aceptan campos desconocidos en silencio) |

---

## 1. Qué disparó esta Task

El propietario comparó la organización de datos de SQX (perfiles de bróker, instrumentos por bróker×símbolo, sesiones por bróker×símbolo, plantilla de dato importado) contra nuestro modelo real y planteó cuatro preguntas. No son retóricas — cada una toca una decisión de esquema o de frontera de responsabilidad que hoy no está resuelta en ningún ADR.

**Verificación previa (obligatoria antes de proponer):** confirmé contra `docs/DATA-MODEL.md` (30 tablas reales, migraciones `0001`–`0021`) que **no existe hoy ninguna tabla `instruments`, `brokers`/`venues` ni `sessions`**. El tipo `Instrument` que ADR-0140 declara solo existe como enum de clase de activo (`Forex/Future/Cfd/Crypto/Equity`) en `shared/`, sin campos de costo de operación ni de calendario — su creación como registro con columnas sigue siendo "prerrequisito de ADR-0140, sin activarse" (ADR-0155, línea de Atributos de Opciones). `data-normalization-layer.md` sí declara un objeto `Symbol { base, quote, broker_symbol, tick_size }`, pero es una identidad de traducción de nombre, no un registro de parámetros de trading. `slippage-models.md` declara `SWAP_TRIPLE_DAY` como parámetro **global** (`CONFIG`), no por instrumento — contradice la realidad de que el día de triple swap varía por bróker y por activo. Ninguna feature documenta pip size, point value, spread/slippage por defecto, tamaño mínimo de distancia, ni modelo de comisión como dato estructurado. Esto confirma que el hueco que el propietario intuyó **es real**, no percepción.

---

## 2. Pregunta 1 — Duplicación de entidades: ¿se puede normalizar sin perder información?

**Lo que SQX hace mal, con precisión:** SQX repite el símbolo (ej. `AUDCAD`) como fila completa de *Instrumento* y como fila completa de *Sesión* para cada bróker, aunque la mayor parte de esos campos sean idénticos entre brókers (el calendario semanal de Forex es el mismo en Dukascopy, ICMarkets, Pepperstone…; solo cambian spread/comisión/swap y, rara vez, el calendario). SQX no tiene mecanismo de herencia — cada fila es una copia completa.

**Lo que nuestro modelo ya evita, sin que nadie lo haya decidido explícitamente como tal:** la tripleta de identidad `SIMBOLO-TEMPORALIDAD-BROKER` (ADR-0155, enmienda 2026-07-18) ya separa símbolo, temporalidad y bróker en columnas propias — nunca concatenadas en una clave opaca. Eso resuelve la identidad de la *serie de precios*. No resuelve la identidad de los *parámetros de trading* (spread, comisión, swap) ni la del *calendario de sesión* — que es exactamente lo que SQX duplica y lo que hoy no tenemos modelado en absoluto.

**Recomendación — tres niveles, ninguno duplicado innecesariamente:**

1. **`instrument_master` (Referencia/Mutable, una fila por símbolo, sin bróker).** Activa el registro `Instrument` que ADR-0140 dejó diferido y lo puebla con los campos generales de trading que ni ADR-0140 ni ADR-0155 fijaron (esos dos ADR solo fijaron los campos específicos de Opciones/Futuros). Clave: `instrument_id` (el mismo que ya es obligatorio en toda fila de `QuoteTick`/`TradeTick`/`Bars`, ADR-0155 Restricción 8). Campos propuestos: `asset_class` (reusa el enum de ADR-0140), `base_currency`/`quote_currency` (Forex/CFD), `canonical_pip_size`, `canonical_tick_size` (`INTEGER` ×10⁸ — convención de mercado, no de un bróker concreto), `default_session_template_id` (FK, ver punto 3). Los campos de Opciones/Futuros que ADR-0155 ya fijó (`strike_price`, `expiration_date`, `contract_multiplier`, `occ_symbol`…) viven en esta misma tabla — no se reinventan aquí.
2. **`broker_instrument_terms` (Referencia/Mutable, sparse — una fila solo cuando un bróker concreto ofrece este instrumento).** Clave `(instrument_id, venue)`. Todos los campos son **opcionales por diseño**: `NULL` significa "hereda el valor canónico de `instrument_master`", no "dato faltante por error" — mismo principio de `NULL` explícito que ADR-0155 ya aplica al resto del esquema. Campos: `venue_symbol` (el `broker_symbol` que `data-normalization-layer` ya declara — este campo no se duplica, se reubica aquí como su hogar estructural), `default_spread`, `default_slippage`, `pip_size_override`, `tick_size_override`, `point_value`, `order_size_multiplier`, `order_size_step`, `min_distance`, `commission_model` (`TEXT CHECK IN ('NONE','PER_TRADE','PERCENTAGE','SIZE_BASED','STOCKPICKER')` — vocabulario de dominio, no anglicismo evitable), `commission_value`, `swap_type`, `swap_long_points`, `swap_short_points`, `triple_swap_day`, `swap_rollout_hour`, `session_template_id_override` (nullable — solo cuando el calendario de ESTE bróker difiere del calendario del instrumento).
3. **`session_template` + `session_template_window` (Referencia, compartida, nunca copiada).** Un template por *forma de mercado* (`FOREX_245`, `CRYPTO_247`, `NYSE_REGULAR`, `CME_CON_MANTENIMIENTO`…), no por bróker×símbolo. `session_template_window` guarda las filas día→día que SQX ya modela (Start day/time → End day/time, bandera SEOC), pero **una sola vez por template**, referenciadas por FK desde cualquier instrumento o desde un override de bróker. La propia nota de SQX ("sessions no se usan ni se soportan en forex/MetaTrader, usa Forex_247") es la prueba de que el 95% de sus filas bróker×símbolo son copias idénticas del mismo template.

**Distinción que no se puede promediar (comprobación 7 de `critical-domain-reasoning.md`):** la tabla de capacidades de ADR-0154 (bróker × vía × clase de activo × granularidad) **no es este catálogo**. Esa tabla declara *qué datos puede traer* un bróker (disponibilidad); este catálogo declara *en qué condiciones se simula una operación* (costo) y *cuándo el mercado está abierto* (calendario). Son dominios de dato distintos, dueños distintos, y no deben fusionarse en una sola tabla — hacerlo repetiría el error de SQX de tratar "instrumento" y "sesión" como el mismo tipo de fila.

**Dueña natural del dato:** `data-normalization-layer` — ya es la dueña declarada de la tabla de capacidades (ADR-0154) y del objeto `Symbol`. Pero la *lógica* no se cuelga de una tabla: se productiza en features (§2-bis).

---

## 2-bis. Descomposición en FEATURES (corrección del propietario, 2026-07-23)

Las tres tablas de §2 **no son el entregable** — son la persistencia interna de features hexagonales con puertos tipados (ADR-0137). Toda lógica (herencia canónico↔override, resolución `COALESCE`, validación de coherencia, conversión de zona horaria en lectura) vive en una feature, nunca en la migración. Descomposición propuesta:

| Feature (nueva o extendida) | Responsabilidad (Núcleo Funcional puro) | Puertos tipados | Persistencia que posee |
|---|---|---|---|
| **`instrument-catalog`** (nueva) | Identidad canónica del instrumento + resolución de los parámetros de trading/costo por bróker (herencia: `override` no-NULL gana, NULL hereda del canónico). Cálculo puro, sin estado. | `InputPort: ResolveInstrumentTerms(instrument_id, venue) → ResolvedTerms` · `OutputPort: InstrumentCatalogStore` (lee/escribe canónico + override) | `instrument_master`, `broker_instrument_terms` |
| **`session-catalog`** (nueva) | Catálogo de plantillas de sesión compartidas (una por forma de mercado, nunca por bróker×símbolo) + la huella de calendario esperada que cada plantilla implica. | `InputPort: ResolveSession(instrument_id, venue) → SessionTemplate` · `InputPort: ExpectedCalendarFingerprint(template_id) → Fingerprint` · `OutputPort: SessionTemplateStore` | `session_template`, `session_template_window` |
| **`pit-data-validator`** (extendida, ADR-0037) | Gana una capa nueva: el gate de coherencia declarado-vs-observado (§3, §3-bis). Compara la huella declarada contra la observada en el archivo. Regla FIJA de validación, cálculo puro. | `InputPort: ValidateCalendarCoherence(declared: Fingerprint, file: BarSeries) → Verdict` (consume `session-catalog`) | ninguna propia — consume las anteriores |

**Consumidores** (ya existentes, migran de defaults globales a términos resueltos por `instrument-catalog`): `slippage-models`, `backtest-engine`. **Presentación:** el Canvas proyecta un único nodo "Instrumento" que resuelve los tres niveles al leer (§4) — nunca materializa la fusión.

Cada feature entrega su documento `docs/features/<nombre>.md` (plantilla FEATURE), su tab SVF (ADR-0117) y su cobertura del gate de mutación (ADR-0133) como cualquier otra. Las tablas de §2 se crean dentro de la Story de su feature dueña, no como una migración huérfana.

---

## 3. Pregunta 2 — Validación de coherencia sesión/activo al importar

**Lo que el propietario pide:** si el usuario importa EURUSD y selecciona (o el sistema infiere) una sesión de cripto 24/7, la importación debe **truene**, no advertir en silencio.

**Falso dilema en la pregunta tal como se planteó.** No es "reglas hardcodeadas por tipo de activo" **contra** "panel CRUD de sesiones" — son dos capas distintas y ambas son necesarias:

- **El catálogo (`session_template`, punto 3 arriba) es el dato**, editable — CRUD, preconfigurado por defecto (`instrument_master.default_session_template_id`) para que el usuario no arranque de cero, exactamente como pide la pregunta. Un `if asset_class == crypto` hardcodeado en Rust violaría la Regla de Oro de `CLAUDE.md` §1 ("Umbral/Max/Min = configurable") y no sobreviviría al primer activo exótico (un índice sintético, un CFD de energía con horario de mantenimiento) — la prueba de límite degenerado lo mata de inmediato.
- **El gate de coherencia es una regla FIJA de validación**, no un dato — vive en `pit-data-validator` (ADR-0037), como una nueva capa junto a las que ya existen (sanidad OHLC, orden cronológico). Reutiliza el patrón **declarado vs. observado** que ADR-0154 ya fija como doctrina: el template elegido/asignado es la hipótesis declarada; el patrón real de huecos en el archivo importado es lo observado. Si el patrón observado es estructuralmente incompatible con el template declarado más allá de una tolerancia configurable (ej. el archivo no tiene huecos de fin de semana pero el template dice `FOREX_245`; o tiene huecos diarios de horas mientras el template dice `CRYPTO_247`), la importación se **rechaza con motivo explícito** — mismo default de rechazo que ADR-0156 ya fija para el mapeo de columnas, aplicado aquí a la coherencia de calendario.
- Esto encaja exactamente en la Restricción 6 de ADR-0156 ("la validación propia de Drasus corre siempre antes de sellar la fila del ledger, sin excepción por el origen del mapeo") — el chequeo de coherencia de sesión es una instancia más de ese guardarraíl, no un mecanismo nuevo de doctrina.

**Qué NO resuelve esta Task:** si el gap de fin de semana observado es genuino (mercado cerrado) o un defecto de datos es una pregunta que `pit-data-validator` ya intenta responder de forma genérica (`MAX_DAILY_MOVE`, detección de gaps) — este catálogo le da al validador el dato que le faltaba para dejar de adivinar: el calendario declarado contra el cual comparar el gap observado, en vez de comparar el archivo solo contra sí mismo.

---

## 3-bis. El gate es N×N por construcción — todas las combinaciones, no un caso (corrección del propietario, 2026-07-23)

El propietario exigió que el gate esté preparado para **toda** combinación, no solo "EURUSD con calendario cripto". Lo está, y la razón es estructural: **el gate no consulta una lista de pares prohibidos — compara dos huellas de calendario.** Enumerar pares (comprobación 6, cierre transitivo) sería el diseño que se rompe con el primer caso no listado.

Cada forma de mercado deja una **huella observable** en los datos (dónde hay velas y dónde faltan):

| Clase | Huella de calendario esperada |
|---|---|
| Cripto | Continua, sin huecos (24/7/365) |
| Forex/CFD | 24/5, un hueco semanal (viernes→domingo), sin huecos diarios |
| Futuros | Hueco diario de mantenimiento + hueco semanal |
| Acciones | Sesión diaria (ej. 9:30–16:00) + noches + fin de semana + feriados |

`session-catalog` deriva la huella **declarada** de la plantilla elegida; `pit-data-validator` deriva la huella **observada** del archivo, y truena ante cualquier contradicción — **en ambas direcciones y en toda combinación**:

- cripto-declarado + archivo-forex → sobra el hueco semanal → **rechazo**
- forex-declarado + archivo-cripto → falta el hueco semanal → **rechazo**
- forex-declarado + archivo-futuro → sobra el hueco diario de mantenimiento → **rechazo**
- acciones-declarado + archivo-cripto → falta la sesión diaria → **rechazo**
- futuro-declarado + archivo-acciones, etc. → cualquier discrepancia estructural → **rechazo**

No hay caso privilegiado: es simétrico y completo porque es huella-contra-huella.

**Reductio del propio gate (comprobación 1):** si el archivo es demasiado corto para exhibir su huella (ej. cubre solo un lunes → no puede mostrar el hueco de fin de semana), el gate devuelve **`INDETERMINADO — datos insuficientes para verificar`**, nunca un `APTO` silencioso. La tolerancia (cuántos huecos atípicos se admiten antes de rechazar, ej. futuros con pausa de mantenimiento corta) es `CONFIG`, no `FIJO`.

---

## 4. Pregunta 3 — "¿Para backend y bases de datos todo será igual?"

**Se confirma, con matiz preciso — no se mata la premisa, se acota.**

**Invariante de backend/BD (fijo, no reorganizable en el Canvas):**
- El esquema de tres niveles del punto 2 (`instrument_master`, `broker_instrument_terms` sparse, `session_template` compartido) y la regla de que `NULL` en el override significa herencia, no ausencia.
- La tripleta de identidad símbolo × temporalidad × bróker (ADR-0155) y la regla de no-fusión.
- El gate de coherencia sesión/activo en `pit-data-validator` y su patrón declarado/observado.
- El ledger append-only de declaración de mapeo (ADR-0156) y la tabla de capacidades por bróker (ADR-0154) — ninguno de los dos se fusiona con este catálogo (ver comprobación de la Pregunta 1).

**Puramente presentación, libre de reorganizar en el Canvas (ADR-0136/0137):**
- Cómo el usuario asocia bróker→instrumento→sesión: pestañas SQX-style, nodos de Canvas con puertos tipados, un buscador con autocompletado, una vista de grid — es piel sobre el mismo backend.
- Que la UI **muestre** una tabla resuelta "bróker × símbolo" completa y legible (como SQX) no exige almacenarla así: es una proyección de lectura sobre `instrument_master` + `broker_instrument_terms` resuelto (`COALESCE` del override sobre el canónico), calculada al leer, nunca materializada como copia. El usuario puede seguir *viendo* la claridad de SQX sin que Drasus pague su costo de duplicación y mantenimiento.
- El grado de "fricción" del asistente (cuántos clics, agrupado por bróker o por clase de activo, con o sin buscador) — presentación pura.

**Consecuencia directa para el diseño visual:** la oportunidad de disrupción que el propietario intuye no está en "menos entidades en la BD" (el dominio real — costo de operación por bróker y calendario de mercado — exige las tres tablas, no menos) sino en que el usuario **nunca necesite saber que son tres tablas**. El Canvas puede presentar un único nodo "Instrumento" cuyo panel resuelve y muestra los tres niveles fusionados, mientras el backend los mantiene separados y sin duplicar.

---

## 5. Pregunta 4 — Zona horaria como dependencia externa (no se re-deriva aquí)

El Quant está fijando en paralelo la política de DST/timezone — esta Task **no** la toca ni la anticipa (violaría el Gate de especialista de `critical-domain-reasoning.md`, comprobación 2). Se define solo el **contrato de consumo**:

- `session_template_window` expresa sus límites Start/End en la **zona horaria oficial del mercado de ese instrumento** (no en UTC, no en la zona del usuario) — igual que SQX declara Timezone por perfil de bróker (`EETUS`, `EET`, `Asia/Jerusalem`).
- **La zona horaria se DECLARA, con su regla de DST propia (patrón SQX confirmado por el propietario, 2026-07-23).** El campo `market_timezone_ref` en `session_template` se puebla desde un **selector de zona mundial donde cada zona trae su bandera de DST** (ítem literal de SQX: *"(EST+07) New York Trading Hours, US DST, DST: Yes"*). No se infiere el offset ni el DST — se adopta la regla estándar de la zona declarada (base de datos tz). Es un **puerto tipado de entrada** cuyo valor viene de la declaración del usuario aprobada (TASK-064 §3), no de un cálculo. La política de conversión en runtime (cómo se aplica esa regla al comparar contra `event_timestamp_ns`) sigue siendo del Quant (TASK-064), pero **el mecanismo de captura ya no es una incógnita**: es un desplegable declarado.
- En runtime, comparar un `event_timestamp_ns` (siempre UTC, ADR-0141) contra una ventana de sesión exige convertir ese timestamp a la zona del mercado usando el mecanismo que el Quant canonice — la misma frontera que ADR-0141 ya fija: "la conversión a zona horaria local ocurre solo en Flutter, nunca en persistencia" se extiende aquí a "ocurre en el validador en tiempo de lectura, nunca se persiste una sesión ya resuelta" (mismo principio que ADR-0141 ya aplica a las etiquetas NY-AM/London-PM).
- Hasta que el Quant cierre esa política, el campo se canoniza ahora (costo de diseño cero, Inundación de Fundaciones ADR-0020 §7.3) pero su población y el gate de coherencia de la Pregunta 2 quedan **bloqueados** — sin zona horaria resuelta no se puede comparar un gap observado contra un calendario declarado con confianza.

---

## 6. Comprobaciones de razonamiento crítico de dominio (`critical-domain-reasoning.md`)

1. **Límite degenerado.** Empujar `broker_instrument_terms` a "todos los campos NULL para todo bróker" no rompe nada — el sistema simplemente opera con los valores canónicos de `instrument_master`, degradación correcta. Empujar el gate de coherencia de sesión a "tolerancia cero" rechazaría instrumentos legítimos con calendarios atípicos (ej. futuros con pausa de mantenimiento corta) — por eso la tolerancia es `CONFIG`, no `FIJO`.
2. **Gate de especialista — declarado y no salteado.** El contrato de timezone/DST es del Quant-Engineer (Pregunta 4, sin resolver aquí). Si el chequeo de coherencia de sesión termina alimentando features de generación de señal (no solo validación de ingesta), su validez estadística también requiere revisión del Quant antes de sellarse como canon — se deja como pregunta abierta en la sección 7, no se decide aquí.
3. **Cierre transitivo.** Este catálogo toca: `data-normalization-layer` (dueña), `data-import-wizard` (UI de selección, resuelve DEBT-031), `pit-data-validator` (gate de coherencia), `slippage-models`/`backtest-engine` (consumidores de costo de operación — hoy usan defaults globales que deberían migrar a `broker_instrument_terms` resuelto), y el Canvas (presentación).
4. **Distinguir tipos, no promediar.** Aplicado explícitamente en la sección 2: tabla de capacidades (disponibilidad) ≠ catálogo de costo/calendario (este Task) ≠ ledger de mapeo (evidencia). Tres dominios de dato, tres dueñas, cero fusión.

---

## 7. Huecos detectados que no estaban cerrados

- **`Instrument` (ADR-0140) nunca se activó como registro.** Se declaró como invariante de diseño ("no sellar el enum") pero nadie construyó la tabla. Este Task es la primera propuesta concreta de qué campos lleva más allá de Opciones/Futuros.
- **Los parámetros de costo de operación (spread, slippage, comisión, swap) viven hoy como `CONFIG` global** en `slippage-models.md`/`backtest-engine.md`, no por instrumento — `SWAP_TRIPLE_DAY` global es incorrecto frente a la realidad de que varía por bróker/activo. No se corrige en este Task (es implementación), se deja registrado como consecuencia directa a resolver cuando se implemente `broker_instrument_terms`.
- **DEBT-031 ya había detectado que "el catálogo de brókers soportados no existe todavía como artefacto"** y pidió explícitamente no inventar un catálogo aparte sino componerlo con la Task que lo resolviera. Este Task es esa composición: los selectores cerrados de bróker/símbolo/temporalidad que DEBT-031 exige se alimentan de `instrument_master`/`broker_instrument_terms`, no de una lista nueva.
- **`pit-data-validator` no tiene hoy ningún concepto de calendario de mercado** — todas sus reglas (`MAX_DAILY_MOVE`, detección de gaps) tratan el archivo importado como si fuera comparable solo contra sí mismo. El gate de coherencia de la Pregunta 2 es una capa nueva, no una extensión trivial de una regla existente.

---

## 8. Qué queda por decidir (no se sella aquí)

1. **Confirmación del propietario sobre el alcance de día 1 del catálogo de `session_template`** — cuántas plantillas preconfiguradas salen al arrancar (Forex 24/5, Cripto 24/7, ¿bolsas concretas?) depende de qué brókers soporta el primer conector real (mismo bloqueo que TASK-060, muestra real pendiente).
2. **Contrato exacto de timezone/DST** — bloqueado en el Quant-Engineer (sección 5). El gate de coherencia de sesión (sección 3) no puede implementarse con confianza hasta que ese contrato exista.
3. **Si el chequeo de coherencia de sesión debe además alimentar la generación de señal** (no solo el gate de ingesta) — pregunta para el Quant, no respondida aquí (comprobación 2 de la sección 6).
4. **Sellado formal como ADR.** Esta Task documenta el análisis y la recomendación del Architect; una vez el Tech-Lead/propietario confirme el enfoque de las secciones 2–4, el siguiente paso es un ADR nuevo (activa ADR-0140, extiende ADR-0154/ADR-0155/ADR-0141) — no se crea aquí para no sellar sin la confirmación de la sección 8.1–8.3.

---

## 9. Trazabilidad

**ADRs:** ADR-0140 (activa el registro `Instrument` diferido), ADR-0154 (distingue tabla de capacidades de este catálogo; reutiliza patrón declarado/observado), ADR-0155 (tripleta de identidad, campos de Opciones/Futuros ya fijados, Restricción 8), ADR-0156 (patrón de rechazo por defecto, aplicado al gate de coherencia), ADR-0141 (timestamps UTC, sesiones derivadas en runtime — extendido aquí a calendario de mercado), ADR-0114 regla (c) (bid/ask exclusivo, mismo dominio de costo de operación), ADR-0020 (Inundación de Fundaciones — campos canonizados con población diferida).
**Features:** [`data-normalization-layer`](../features/data-normalization-layer.md) (dueña propuesta), [`data-import-wizard`](../features/data-import-wizard.md) (UI de selección), [`pit-data-validator`](../features/pit-data-validator.md) (gate de coherencia), [`slippage-models`](../features/slippage-models.md) y [`backtest-engine`](../features/backtest-engine.md) (consumidores de costo de operación, hoy con defaults globales a migrar).
**Deuda relacionada:** [DEBT-031](../debt/DEBT-031.md) (catálogo de brókers inexistente — este Task es su composición, no una deuda nueva), [DEBT-029](../debt/DEBT-029.md) (contratos que aceptan campos desconocidos — mismo dominio de guardarraíles de ingesta).
**Origen:** sesión del propietario 2026-07-22, evaluación de exportaciones de StrategyQuant X.

---
