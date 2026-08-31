# Session Catalog

**Carpeta:** `./features/session-catalog/`
**Estado:** ✅ Implementado (STORY-066, 2026-07-25 — QA APTO; backend completo, UI de captura delegada a `data-import-wizard`)
**Última actualización:** 2026-07-25 (STORY-066)
**Decisión Arquitectónica Asociada:** ADR-0141 (sesiones derivadas en runtime) · ADR-0162 (esta feature)

## ¿Qué es?

Es el catálogo de plantillas de calendario de mercado. Una plantilla de sesión (`session_template`) representa una *forma de mercado* (`FOREX_245`, `CRYPTO_247`, `NYSE_REGULAR`…), no un bróker ni un símbolo — se declara **una sola vez** y se referencia desde cuantos instrumentos u overrides de bróker la compartan. Es la respuesta directa al defecto de StrategyQuant X que el propietario detectó: repetir la misma fila de sesión por cada combinación bróker×símbolo cuando el 95% del tiempo el calendario es idéntico.

**Problema:** hoy no existe ninguna noción de calendario de mercado en el sistema. `pit-data-validator` trata cualquier archivo importado como comparable solo contra sí mismo — no tiene contra qué calendario declarado comparar un hueco observado. Además, la zona horaria de un mercado no se puede inferir de forma confiable: EE. UU. y Europa cambian de horario de verano (DST) en fechas distintas del calendario, y un archivo mal etiquetado desplaza silenciosamente todas sus barras una hora completa durante 1–3 semanas cada primavera y cada otoño.

**Solución:** plantillas de sesión compartidas, cada una con sus ventanas semanales (día/hora de inicio y fin) y su zona horaria de mercado declarada mediante un selector de zona mundial con bandera de horario de verano (DST) por zona — patrón que StrategyQuant X ya usa. La zona horaria **nunca se infiere**; el usuario la declara, el sistema adopta la regla estándar de esa zona (base de datos de zonas horarias IANA) sin calcular nada por su cuenta.

---

## Narrativa de Producto

**Capacidad:** `CAP-01` — Datos en los que se puede confiar

**Historia de usuario:** Como investigador de estrategias, quiero declarar una sola vez cuándo abre y cierra cada mercado y en qué huso horario, para que el sistema sepa distinguir un hueco legítimo de un dato faltante y no me desplace las barras una hora sin avisar.

**El problema (técnico):** sin calendario declarado no hay forma de juzgar un hueco: el validador solo puede comparar el archivo consigo mismo, y no sabe si la ausencia de datos un martes significa "el mercado estaba cerrado" o "falta un pedazo". Y hay un defecto peor: el huso horario de un mercado **no se puede inferir de forma confiable**. Estados Unidos y Europa cambian al horario de verano en fechas distintas, así que un archivo mal etiquetado desplaza todas sus barras una hora completa durante una a tres semanas, dos veces al año. El desplazamiento es silencioso y sobrevive a cualquier revisión superficial.

**La solución (técnica):** plantillas de calendario declaradas una sola vez por *forma de mercado* —no por combinación de bróker y símbolo— y referenciadas desde cuantos instrumentos las compartan. Cada una lleva sus ventanas semanales y su huso horario **declarado explícitamente** contra el estándar internacional de zonas horarias. El sistema adopta la regla de esa zona; **nunca la calcula ni la deduce**.

**Explicado sin tecnicismos:** los mercados tienen horarios. Unos abren de lunes a viernes, otros no cierran nunca. Sin saber ese horario, el sistema no puede distinguir entre "aquí no hay datos porque estaba cerrado" y "aquí faltan datos".

Pero el problema serio es otro, y es de los que arruinan meses de trabajo sin hacer ruido: **el cambio de horario de verano**. Estados Unidos y Europa lo hacen en fechas distintas. Si tu archivo trae mal etiquetada la zona horaria, durante dos o tres semanas al año **todas tus barras quedan corridas una hora completa**. Nada se rompe, nada avisa. Simplemente tus resultados de esas semanas describen un mercado que no existió.

Por eso Drasus **nunca adivina la zona horaria**: te la pregunta. Tú la declaras una vez, y el sistema aplica la regla oficial de esa zona del mundo sin inventarse nada.

## Comportamientos Observables

- [ ] El usuario crea una plantilla de sesión nueva; declara su código (`FOREX_245`), sus ventanas semanales (ej. domingo 22:00 → viernes 22:00) y elige la zona horaria de mercado de un selector mundial que muestra la bandera de DST de cada zona.
- [ ] El usuario asocia un instrumento (o el override de un bróker concreto) a una plantilla de sesión existente — la plantilla no se copia, se referencia.
- [ ] Cualquier consumidor pide la huella de calendario esperada de una plantilla (`ExpectedCalendarFingerprint`) y recibe la descripción estructural de dónde debería haber huecos (semanal, diario de mantenimiento, ninguno) según sus ventanas.
- [ ] El sistema NO ofrece ningún campo de texto libre para "offset horario" ni ningún botón de "detectar automáticamente por volatilidad" — solo el selector de zona con su bandera de DST.

## Restricciones

- **NUNCA una plantilla de sesión se copia por bróker×símbolo.** Un template por forma de mercado, referenciado por FK desde cuantos instrumentos/overrides lo compartan.
- **NUNCA la zona horaria se infiere de volatilidad, de una noticia programada, ni de ningún otro proxy de mercado.** Método descartado explícitamente por el propietario (2026-07-23, ver ADR-0162 restricción 7): es ruidoso y frágil. El hueco de fin de semana queda como verificación estructural **opcional**, nunca como mecanismo primario de determinación del offset.
- **NUNCA se persiste una convención de desplazamiento horario inventada** (tipo `"EST+07"`). Se persiste un identificador estándar de la base de datos de zonas horarias IANA — la fuente que ya resuelve correctamente las reglas de DST de cada región sin que Drasus las codifique a mano.
- **NUNCA la conversión de un timestamp UTC a la zona del mercado se persiste.** Ocurre en tiempo de lectura, en el validador que compara — mismo principio que ADR-0141 ya fija para las etiquetas de sesión (NY-AM, London-PM).
- **Aclaración (ADR-0162 enmienda 2026-07-26) — `market_timezone_ref` NO es el reloj de un archivo importado.** Es la zona en la que se expresan los **límites de ventana de una plantilla** (convención de mercado, ej. "Forex abre domingo 22:00 hora de Nueva York") — un hecho geográfico real que una zona IANA plana ya captura sin defecto. El reloj de un bróker/servidor concreto (que sí puede anclar su horario de verano al calendario de otro país, ej. IC Markets) se declara aparte, en `data-import-wizard`/`broker_server_clock` (`instrument-catalog`), con su propia forma (ancla + desplazamiento fijo). Comparten el mismo widget de selector, nunca la misma columna ni el mismo dueño.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| SESSION_TEMPLATE_SEED_SET | Forex 24/5, Cripto 24/7, Futuros (CME Globex) | — | Plantillas preconfiguradas que el sistema ofrece al arrancar, para que el usuario no empiece de cero. **Confirmado por el propietario 2026-07-25:** Forex, Cripto y Futuros. La plantilla de Futuros es un default **representativo** estilo CME Globex (zona `America/Chicago`, hueco diario de mantenimiento + hueco semanal), **editable** por el usuario — los calendarios de futuros son específicos del mercado/producto y esta semilla es solo un punto de partida, no una verdad universal | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** el cálculo de la huella de calendario esperada a partir de las ventanas semanales de una plantilla es una función pura: entra el conjunto de ventanas, sale la descripción estructural de huecos esperados.
- **Shell (Infraestructura):** persistencia SQLite de `session_template`/`session_template_window`, resolución del identificador IANA contra la base de datos de zonas horarias del runtime (`chrono-tz` u equivalente) en tiempo de lectura.
- **Frontera Pública:** `ResolveSession(instrument_id, venue) → SessionCalendar`, `ExpectedCalendarFingerprint(template_id) → CalendarFingerprint`.

## Ciclo de Vida de la Feature — Session Catalog

### Entrada
- Declaración del usuario: código de plantilla, ventanas semanales, zona horaria elegida del selector.
- Consulta de un consumidor: `template_id` (para la huella esperada) o `(instrument_id, venue)` (para la plantilla resuelta de ese instrumento).

### Proceso
- Alta: valida los días/horas de las ventanas, persiste el identificador IANA elegido.
- Resolución: dado `(instrument_id, venue)`, sigue la cadena `broker_instrument_terms.session_template_id_override` (si existe) → `instrument_master.default_session_template_id` → plantilla resuelta.
- Cálculo de huella: a partir de las ventanas semanales de la plantilla, deriva la forma estructural esperada (continua / hueco semanal / hueco semanal + diario / sesión diaria).

### Salida
- `SessionCalendar`: plantilla resuelta con sus ventanas y zona horaria.
- `CalendarFingerprint`: huella de calendario esperada, consumida por el Gate de Coherencia de `pit-data-validator`.

### Contextos de Uso

**Contexto 1: Importación de datos (Ingest)**
- `pit-data-validator` consulta la huella esperada de la plantilla declarada para un archivo antes de compararla contra la huella observada del propio archivo (Gate de Coherencia, ADR-0162).

**Contexto 2: Catálogo de instrumento**
- `instrument-catalog` referencia una plantilla como `default_session_template_id` al dar de alta un instrumento nuevo.

## Tareas (TTRs)

### TTR-001: Alta de Plantilla de Sesión y Ventanas Semanales
- Implementa la persistencia de `session_template`/`session_template_window`, con el selector de zona horaria mundial (bandera DST por zona) como único mecanismo de captura del `market_timezone_ref`.

### TTR-002: Cálculo de Huella de Calendario Esperada
- Implementa la función pura que deriva, a partir de las ventanas semanales de una plantilla, su huella estructural esperada (continua / hueco semanal / hueco semanal+diario / sesión diaria).

### TTR-003: Resolución de Sesión por Instrumento/Bróker
- Implementa `ResolveSession`, siguiendo la cadena de override de bróker → default de instrumento.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `resolve_session_in` | `(interno)` | Input | 1 | Solicitud `(instrument_id, venue)` — payload específico de esta feature |
| `session_calendar_out` | `SessionCalendar` | Output | 1 | Plantilla de sesión resuelta (ventanas + zona horaria declarada) |
| `expected_fingerprint_out` | `(interno)` | Output | 1 | Huella de calendario esperada — payload de comparación consumido directamente por `pit-data-validator`, no cableable de forma independiente en el Canvas |

## Cáscara Visual (Thin Shell)

> **Pendiente — Etapa 0.5 (skill `ui-designer`).** No corresponde al Architect rellenarla. Contrato funcional mínimo: el selector de zona horaria mundial debe mostrar la bandera de DST por zona de forma tan explícita como el desplegable de referencia de StrategyQuant X citado en TASK-064 §3.2 (ej. "(EST+07) New York Trading Hours, US DST: Yes") — nunca un campo de texto libre para el offset.

### Ventana de Verificación Visual
Observable de plomería: el `market_timezone_ref` (identificador IANA) y las ventanas semanales de una plantilla, visibles en el punto donde el usuario asocia un instrumento a su calendario (dentro de `instrument-catalog`/`data-import-wizard`), y el veredicto del Gate de Coherencia consumidor en `pit-data-validator`.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** No aplica.
- **Inundación de Fundaciones (ADR-0020):** Perfil **A. Datos/Ingest** — Identidad (I) + Linaje (III) + Hardware (IV). **Corrección de secuencia (ADR-0141 §"Semántica de secuencias"):** `session_template`/`session_template_window` son tablas Referencia, mutables por diseño — usan `row_version`, no `event_sequence_id`.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Tablas propias, esquema completo en ADR-0162.

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Clave técnica UUIDv7 |
| | `template_code` | Clave de negocio (`FOREX_245`, `CRYPTO_247`…) |
| | `created_at` / `updated_at` | Timestamps de alta/edición |
| | `row_version` | Contador de versión (mutable) |
| | `audit_hash` / `audit_chain_hash` | Hash de la fila / `NULL` en génesis |
| **III. Linaje** | — | No aplica: declaración del usuario, no dato con procedencia externa |
| **IV. Hardware** | `node_id` / `process_id` | Hardware y proceso que registraron el alta/edición |

**Rastro de Evidencia:** el historial de `row_version` permite reconstruir si una plantilla de sesión cambió entre dos importaciones, insumo para auditar por qué un archivo pasó el Gate de Coherencia en una fecha y no en otra.

## Dependencias y Bloqueantes

- Bloqueado por: ninguno para el mecanismo (puede construirse en cuanto se apruebe ADR-0162). El **alcance de día 1 del `SESSION_TEMPLATE_SEED_SET`** quedó **resuelto por el propietario (2026-07-25): Forex 24/5, Cripto 24/7 y Futuros (CME Globex representativo, editable)**. Los calendarios de futuros específicos por producto/mercado se refinan por el usuario sobre la plantilla semilla.
- **"Consumido por" = accede al puerto:** [`pit-data-validator`](./pit-data-validator.md) (huella esperada del Gate de Coherencia), [`instrument-catalog`](./instrument-catalog.md) (default de sesión por instrumento).
- **Contrato de Integración UI:** **Ventana de Verificación** — consumidora principal: `pit-data-validator`, veredicto del Gate de Coherencia. El observable de esta feature es la huella de calendario esperada visible junto al resultado del gate tras una importación.

## Dependencias
**Depende de:**
- Ninguna.

**Consumido por:**
- [`pit-data-validator`](./pit-data-validator.md), [`instrument-catalog`](./instrument-catalog.md).
