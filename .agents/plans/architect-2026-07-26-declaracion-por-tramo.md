# Informe del Architect — TASK-069, reloj de bróker y declaración por tramo

**Encargo:** `docs/execution/TASK-069-architect-escalation-reloj-de-broker.md`. Dos hallazgos de la misma semana — el reloj de servidor de IC Markets y la mezcla de procedencia dentro de un solo archivo declarado — comparten causa raíz: el sistema toma la declaración por archivo completo, y un archivo real no es homogéneo en todo su rango.

---

## 1. Qué decidí y dónde quedó escrito

Todo canonizado en `docs/adr/ADR-0162.md` (banner de enmienda 2026-07-26, Restricciones 7/9/10 nuevas o corregidas, esquema de `broker_server_clock`, sección completa "Enmienda 2026-07-26" con las 7 comprobaciones de razonamiento crítico). Propagado a:

- `docs/features/data-import-wizard.md` — selector de reloj por ancla+desplazamiento, declaración por tramo, dependencia nueva de `broker_server_clock`.
- `docs/features/session-catalog.md` — aclaración de que `market_timezone_ref` no es el reloj de un archivo.
- `docs/features/instrument-catalog.md` — tabla `broker_server_clock`, TTR-004, puertos, restricciones.
- `docs/features/pit-data-validator.md` — TTR-005 (Gate de Homogeneidad de Procedencia Declarada), parámetros `CONFIG` nuevos, puertos, dependencia de `instrument-catalog`.

**Decisión 1 — el reloj de un archivo se declara como zona ancla del catálogo IANA + desplazamiento fijo en minutos, nunca como una zona plana.**

Un identificador IANA basta cuando el reloj de un bróker coincide con una zona real. No basta para IC Markets: su servidor corre en GMT+2/+3, pero el cambio de horario sigue las fechas de Estados Unidos, no las europeas — no hay zona IANA con ese par de offsets y ese calendario de cambio a la vez. La forma nueva es: elige una zona ancla por su **calendario** de cambios de horario de verano (no por su offset), y declara un desplazamiento fijo constante frente a ella. Desplazamiento `0` reproduce la declaración de hoy sin caso especial — no son dos mecanismos, uno es el caso particular del otro.

Este campo vive en `data-import-wizard`/`import_mapping_declarations` (el reloj de un archivo concreto), **no** en `session_template.market_timezone_ref` (`session-catalog`) — ver §2.

Tabla nueva `broker_server_clock` (una fila por `venue`, propiedad de `instrument-catalog`): guarda la zona ancla, el desplazamiento, y opcionalmente `venue_operational_since` (fecha de operación del bróker, hecho externo verificado, nunca inferido). Se declara una sola vez por bróker y alimenta como borrador el selector de cada importación — el usuario confirma o corrige, igual que ya hace con el mapeo de columnas.

**Decisión 2 — la declaración de mapeo pasa de grano "archivo" a grano "tramo declarado dentro de un archivo".**

Un archivo de 25 años puede no ser homogéneo. El Quant midió que `<VOL>` cambia de régimen sin transición (fractura de seis órdenes de magnitud en 2017) y que el propio archivo arranca en 2000 pese a que IC Markets se fundó en 2007 — un tramo entero no puede ser tráfico real de ese bróker. Distinguí dos casos que no se resuelven igual:

- **Misma identidad real, característica que cambia.** El bróker de ejecución sigue siendo el mismo; lo que cambia es la convención de un campo (unidad de volumen, reloj). Se resuelve con **más de una fila del ledger para el mismo `instrument_id`/`venue`**, cada una con su propio rango de fechas explícito.
- **Identidad real distinta, fusionada bajo una etiqueta.** Un tramo (ej. el histórico pre-2007, casi con certeza rellenado por un proveedor al configurar el terminal) no es tráfico genuino del bróker declarado. Ese tramo, si se importa, se declara con su **propio `venue`/`source_provider`** — nunca bajo la etiqueta cuya identidad no sostiene.

Mecánicamente, esto exige en `import_mapping_declarations`: columnas nuevas `declared_range_start_ns`/`declared_range_end_ns` (el rango que cada fila declara cubrir, explícito), y relajar `UNIQUE(source_file_hash)` a `UNIQUE(source_file_hash, declared_range_start_ns, declared_range_end_ns)` — hoy esa restricción rechazaría por error la segunda declaración de un archivo genuinamente heterogéneo.

**Decisión 3 — Gate de Homogeneidad de Procedencia Declarada, nuevo en `pit-data-validator` (TTR-005), hermano del Gate de Coherencia de Calendario.**

Corre antes de sellar el mapeo. Dos chequeos, ambos sobre hechos medibles, ninguno infiere un valor:

- Fractura de magnitud/disponibilidad de un campo con convención estática declarada, por encima de un umbral `CONFIG` (default 1000×; la fractura real medida es de 10⁶) — si se dispara, el usuario decide: segmenta la importación o confirma explícitamente que declara homogeneidad pese a la fractura mostrada.
- Fecha del tramo anterior a `venue_operational_since` (si está declarado) — rechazo automático con motivo explícito.

**Decisión 4 (explícita, para que no quede implícita) — NO se versiona temporalmente `broker_instrument_terms.real_volume_unit_declared`.** Es la pregunta que el Quant dejó abierta para mí. Añadir vigencia temporal a un campo de catálogo resuelve el síntoma en la capa equivocada: la heterogeneidad es de un archivo/tramo concreto, no del bróker en general. La declaración por tramo (Decisión 2) ya resuelve esto en el grano correcto, sin inventar versionado de catálogo.

---

## 2. Qué reté del encuadre recibido, y con qué conclusión

**La hipótesis del Tech-Lead ("ancla de Nueva York + 7 horas") se valida, pero no literal — con dos correcciones (`GOVERNANCE-4.7`):**

1. **El desplazamiento se declara en minutos, no en horas.** Cubre además brókers con offset fraccionario frente a su ancla, a costo cero.
2. **La hipótesis, tal como llegó, apuntaba al mecanismo equivocado.** El paquete señalaba "el mecanismo de declaración de ADR-0162" en general, y ADR-0162 hoy tiene un solo campo de zona horaria visible en su esquema: `session_template.market_timezone_ref`. Al examinar el sistema completo encontré que ese campo **no es** el reloj de un archivo — es la zona de referencia de los límites de ventana de una plantilla de sesión (convención de mercado, ej. "Forex abre domingo hora de Nueva York"), un hecho geográfico real que una zona IANA plana ya captura sin defecto. El campo que sí tiene el problema de IC Markets es otro, ya existente pero silencioso en el paquete de escalamiento: `source_timezone_declared`, en `data-import-wizard`/`import_mapping_declarations`. Corregir el campo equivocado habría dejado el problema real intacto y roto algo que no estaba roto.

**Sobre si los dos casos comparten causa raíz (pregunta explícita del encargo): sí, y agruparlos fue correcto.** Ambos son "una declaración se sella para un archivo completo, pero el archivo no es homogéneo en ese eje" — el reloj en un caso, la procedencia/unidad de un campo en el otro. La solución que cierra el segundo caso (declaración por tramo) es exactamente la que hace expresable, sin trabajo adicional, que el reloj también varíe por tramo (pregunta 3 del Quant en la Task) — separarlos habría producido dos mecanismos donde uno basta.

**No encontré ninguna contradicción entre documentos vigentes** que no fuera resoluble por mí — el hueco era de expresividad (ninguna zona IANA cubre IC Markets) y de grano (una declaración por archivo no cubre un archivo heterogéneo), no una regla que se contradijera con otra.

---

## 3. Bloqueo de producto para el propietario

**Ninguno duro.** Las dos decisiones se resuelven con datos que el sistema ya puede pedir al usuario (zona ancla, desplazamiento, fecha de operación de un bróker, rango de un tramo) — no dependen de una decisión de negocio que solo él pueda tomar.

**Una decisión de umbral que el propietario puede querer revisar, no bloqueante:** el default de `PROVENANCE_HOMOGENEITY_MAGNITUDE_RATIO_THRESHOLD` (1000×) lo fijé yo con margen amplio bajo la fractura real medida (10⁶) — es `CONFIG`, ajustable sin rediseño si en la práctica resulta demasiado sensible o demasiado laxo con más archivos reales.

---

## 4. Para el Tech-Lead — impacto en lo ya construido (no ejecutado en esta sesión, por instrucción explícita)

**`crates/features/data/data-import-wizard/` (STORY-068) necesita retrofit.** Verificado en el código: `domain.rs`/`schemas.rs`/`persistence.rs` usan hoy `source_timezone_declared: String` como zona IANA plana, pasada directa a `resolve_event_timestamp_ns`. Cambios de esquema pendientes (definidos en ADR-0162, no implementados):

- `import_mapping_declarations` (`migrations/0023`, edición in-situ — fase GREENFIELD, filas existentes son solo de prueba):
  - Nuevas: `declared_range_start_ns`/`declared_range_end_ns` (`INTEGER NOT NULL`, ns UTC, `CHECK (end > start)`).
  - `UNIQUE(source_file_hash)` → `UNIQUE(source_file_hash, declared_range_start_ns, declared_range_end_ns)`.
  - `source_timezone_declared TEXT NOT NULL` → `source_clock_anchor_timezone_ref TEXT NOT NULL` + `source_clock_fixed_offset_minutes INTEGER NOT NULL DEFAULT 0`.
- `instrument_master`/`broker_instrument_terms`/... (`migrations/0022`): tabla nueva `broker_server_clock` (esquema completo en ADR-0162).
- `pit-data-validator`: TTR-005 nuevo (Gate de Homogeneidad) — feature aún no tiene crate propio verificado por mí en esta sesión; el Tech-Lead confirma su estado real antes de despachar.
- Lógica de resolución de timestamp en `data-import-wizard::domain` debe aplicar ancla+desplazamiento, no una zona plana.

**No toqué migraciones, código ni fichas de `docs/data-model/`** — son espejo de lo realmente migrado (`ledger-pattern.md` punto 6) y hoy siguen describiendo con exactitud el esquema actual; se actualizan en la Story que ejecute este retrofit, junto con la migración.

---

**Trazabilidad:** ADR-0162 (enmienda 2026-07-26), `docs/execution/TASK-069-architect-escalation-reloj-de-broker.md`, `.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md`, `.agents/plans/quant-2026-07-26-volumen-forex-vs-futuros.md`.
