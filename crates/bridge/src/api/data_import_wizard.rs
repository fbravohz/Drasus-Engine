//! Funciones FFI para la vía de mapeo manual del asistente de importación
//! (`data-import-wizard`) hacia Flutter/Dart -- la superficie mínima que el
//! Inspector Panel de seis pasos necesita (STORY-073 Etapa 3, §4.2).
//!
//! ## La frontera lectura/escritura, en la FORMA del API (LDR-0008 condición 1)
//!
//! De las cinco operaciones expuestas, **una sola recibe `db_path`**:
//!
//! | Operación                              | Toma `db_path` | Puede escribir el ledger |
//! |------------------------------------------|:---------------:|:--------------------------:|
//! | [`select_manual_import_file`]           | No              | No -- imposible, no hay parámetro |
//! | [`get_manual_import_columns`]           | No              | No -- imposible, no hay parámetro |
//! | [`preview_manual_bar_import`]           | No              | No -- imposible, no hay parámetro |
//! | [`analyze_manual_bar_file_content_range`] | No            | No -- imposible, no hay parámetro |
//! | [`confirm_manual_bar_import`]           | **Sí**          | Sí -- la única que sella   |
//!
//! Esto no es una convención de nombres ni un comentario que alguien podría
//! ignorar: es una garantía del compilador. Las primeras cuatro funciones no
//! reciben un `SqlitePool` ni una ruta de base de datos en ningún punto de su
//! firma -- no existe forma de que su cuerpo llegue a escribir en SQLite,
//! porque no tienen cómo abrir una conexión. Es el MISMO principio que ya
//! aplica en el Núcleo: `data_import_wizard::public_interface::
//! parse_and_map_manual_bar_file` y `preview_manual_bar_file` son funciones
//! PURAS sin `SqlitePool` (ver su doc en `domain.rs`) precisamente para que
//! "pedir la vista previa no escribe nada" sea una garantía estructural, no
//! observada. Este módulo preserva esa garantía al cruzar la frontera FFI: si
//! en algún momento alguien intentara añadirle `db_path` a cualquiera de las
//! cuatro primeras, el cambio sería visible en la firma misma de la función,
//! no un defecto escondido en el cuerpo.
//!
//! ## Por qué `select` y `get_columns` son dos operaciones y no una
//!
//! `select_manual_import_file` y `get_manual_import_columns` podrían
//! colapsarse en una sola llamada (ambas leen los mismos bytes). Se
//! mantienen separadas porque cada una resuelve una pregunta distinta y
//! barata de responder por separado: "¿el archivo que arrastré es legible?"
//! (tamaño + huella, sin interpretar una sola columna) vs. "¿qué columnas
//! trae, para poblar el paso de mapeo?" (sí interpreta la cabecera). Fusionar
//! ambas no ahorra una lectura real (el archivo ya está en memoria del lado
//! Dart) y sí oscurece cuál pregunta falló cuando algo sale mal.
//!
//! ## Vista previa: dos operaciones de costo distinto, nunca fusionadas (LDR-0008 condición 1, DEBT-039)
//!
//! Medido contra el archivo real del propietario (2026-07-31, 9.231.696
//! filas, 535 MB): parsear el archivo COMPLETO cuesta ~5,3 s y ~700 MB de
//! asignaciones. El asistente pide una vista previa en cada ajuste de mapeo
//! del usuario -- pagar ese costo en cada vuelta convertiría el paso de
//! Vista Previa en inutilizable sobre un archivo real. Por eso esta frontera
//! separa DOS operaciones que antes vivían fusionadas en una sola llamada:
//!
//! - [`preview_manual_bar_import`] -- BARATA, para el ciclo interactivo
//!   "ajusto el mapeo y vuelvo a mirar". Envuelve
//!   `data_import_wizard::public_interface::preview_manual_bar_file`, que
//!   detiene el parseo tras [`PREVIEW_ROW_LIMIT`] filas de dato -- nunca lee
//!   el archivo completo. Devuelve solo `rows`: las primeras
//!   [`PREVIEW_ROW_LIMIT`] filas transformadas, EXACTAMENTE las que
//!   `confirm_manual_bar_import` sellará si el usuario acepta (comparten el
//!   mismo parseo por fila -- LDR-0008 condición técnica 1). Se puede llamar
//!   cientos de veces sin pagar el costo del archivo completo ni una sola.
//! - [`analyze_manual_bar_file_content_range`] -- CARA, deliberadamente
//!   separada. Envuelve `parse_and_map_manual_bar_file` (sin límite) y
//!   reduce sobre TODAS las filas para dar el mínimo de apertura y el máximo
//!   de cierre reales del archivo (nunca asume orden: lee y compara cada
//!   fila, igual que `check_declared_range_against_parsed_content` en el
//!   Núcleo) -- el dato que DEBT-039 exige mostrar junto al rango declarado
//!   antes de confirmar (criterio 5b de STORY-073). Se invoca UNA vez, cuando
//!   el usuario llega al paso de confirmación con su mapeo ya estable --
//!   nunca en cada tecleo del paso de mapeo.
//!
//! Fusionar ambas en una sola llamada -- como hacía la versión anterior de
//! este módulo -- hace que la operación barata pague, en cada vuelta, el
//! costo de la cara: exactamente el defecto que esta separación corrige.
//! Ninguna de las dos recibe `db_path` (ver tabla arriba): la vista previa se
//! pide y se descarta sin efecto, y el análisis de rango es de la misma
//! naturaleza -- solo lee bytes que Dart ya tiene en memoria.
//!
//! ## Patrón de DTO local
//!
//! Mismo patrón que `hybrid_data_transformer.rs` y `data_fetcher.rs`: DTOs
//! propios definidos dentro de `crate::api`, mapeados campo a campo desde los
//! tipos reales de la feature -- `flutter_rust_bridge` solo genera bindings
//! Dart para tipos dentro del árbol declarado en `rust_input`
//! (`flutter_rust_bridge.yaml`), nunca para tipos reexportados de otro crate.

use data_import_wizard::public_interface::{
    compute_source_file_hash, detect_csv_columns, import_manual_bar_file, parse_and_map_manual_bar_file,
    preview_manual_bar_file, CanonicalBarField, CanonicalBarRow, ManualColumnMapping, ManualImportBarFileError,
    ManualImportBarFileInput, ManualRunContext, PREVIEW_ROW_LIMIT,
};
use shared::public_interface::{create_pool, run_migrations, SystemClock};

// ── Tipos FFI-safe ──────────────────────────────────────────────────────

/// Una correspondencia columna origen -> campo canónico, tal como Dart la
/// construye en el Paso 2 del asistente (Mapeo). Espejo FFI-safe de
/// [`ManualColumnMapping`] -- `canonical_field` viaja como texto porque
/// `CanonicalBarField` no deriva `Serialize`/`Deserialize` en el Núcleo (es
/// un enum de dominio puro, no un DTO de frontera).
///
/// Vocabulario FIJO reconocido en `canonical_field` (ver
/// [`parse_canonical_bar_field`]): `TIMESTAMP`, `TIMESTAMP_DATE`,
/// `TIMESTAMP_TIME`, `OPEN`, `HIGH`, `LOW`, `CLOSE`, `VOLUME_TICKS`,
/// `VOLUME_TRADED`, `SPREAD_REPORTED`. `TIMESTAMP` (columna combinada) y
/// `TIMESTAMP_DATE`+`TIMESTAMP_TIME` (fecha y hora en columnas separadas,
/// ej. el archivo real de MetaTrader del propietario: `<DATE>` TAB `<TIME>`)
/// son dos estrategias mutuamente excluyentes -- el Núcleo
/// (`resolve_manual_mapping`) rechaza declarar solo una de las dos columnas
/// separadas, o declarar ambas estrategias a la vez.
#[derive(Debug, Clone)]
pub struct ManualColumnMappingDto {
    pub source_column: String,
    pub canonical_field: String,
}

/// Resultado de "seleccionar archivo" (Paso 1, Archivo) -- confirma que el
/// buffer llegó completo y da al usuario una huella verificable, sin
/// interpretar ni una sola columna todavía.
///
/// Frontera lectura/escritura: esta función NO recibe `db_path` -- no hay
/// forma, en su firma, de que toque una base de datos.
#[derive(Debug, Clone)]
pub struct SelectManualImportFileResultDto {
    pub file_size_bytes: u64,
    /// Huella SHA-256 del contenido -- la MISMA función que el Núcleo usa
    /// para su clave de idempotencia (`compute_source_file_hash`), mostrada
    /// aquí de forma temprana solo para que la pantalla pueda desplegarla;
    /// `confirm_manual_bar_import` la recalcula por su cuenta y es la que
    /// cuenta legalmente -- este valor es informativo, no autoritativo.
    pub source_file_hash: String,
}

/// Resultado de "obtener las columnas detectadas" (Paso 2, Mapeo) -- las
/// columnas reales de la cabecera, para poblar los `select` de mapeo.
///
/// Frontera lectura/escritura: esta función NO recibe `db_path`.
#[derive(Debug, Clone)]
pub struct ManualImportColumnsResultDto {
    pub columns: Vec<String>,
    /// `null` en Dart si `columns` se pobló con éxito. Poblado (y `columns`
    /// vacío) si el archivo no es UTF-8 válido o no tiene ni una línea de
    /// cabecera -- nunca hace panic ante un archivo malformado.
    pub error: Option<String>,
}

/// Una fila ya transformada al esquema canónico -- espejo FFI-safe 1:1 de
/// [`CanonicalBarRow`]. Es DATO, no correspondencia de nombres: escala
/// entera (ADR-0141, precio ×10⁸ vía `parse_price_to_scaled`) y nanosegundos
/// UTC, exactamente lo que LDR-0008 condición 1 exige que el usuario vea
/// antes de aceptar.
#[derive(Debug, Clone)]
pub struct PreviewCanonicalBarDto {
    pub instrument_id: String,
    pub venue: String,
    pub source_provider: String,
    pub bar_open_timestamp_ns: i64,
    pub bar_close_timestamp_ns: i64,
    /// Siempre `"TIME"` en esta vía (ver doc de [`CanonicalBarRow::bar_type`]).
    pub bar_type: String,
    pub timeframe_value: String,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    pub volume_ticks: i64,
    pub volume_traded_raw: i128,
    pub spread_reported_raw: i64,
    pub spread_reported_unit: String,
    pub is_activity_anomaly: bool,
    pub volume_unit_unverified: bool,
}

impl From<&CanonicalBarRow> for PreviewCanonicalBarDto {
    fn from(bar: &CanonicalBarRow) -> Self {
        Self {
            instrument_id: bar.instrument_id.clone(),
            venue: bar.venue.clone(),
            source_provider: bar.source_provider.clone(),
            bar_open_timestamp_ns: bar.bar_open_timestamp_ns,
            bar_close_timestamp_ns: bar.bar_close_timestamp_ns,
            bar_type: bar.bar_type.to_string(),
            timeframe_value: bar.timeframe_value.clone(),
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume_ticks: bar.volume_ticks,
            volume_traded_raw: bar.volume_traded_raw,
            spread_reported_raw: bar.spread_reported_raw,
            spread_reported_unit: bar.spread_reported_unit.clone(),
            is_activity_anomaly: bar.is_activity_anomaly,
            volume_unit_unverified: bar.volume_unit_unverified,
        }
    }
}

/// Resultado de "pedir la vista previa transformada" (ciclo interactivo de
/// ajuste de mapeo, dentro del Paso 6, Vista Previa y Confirmación) -- dato
/// real, nunca esquema. Deliberadamente BARATO: no trae el total de filas ni
/// el rango de contenido -- ambos exigen el archivo completo y viven en
/// [`analyze_manual_bar_file_content_range`] (ver "Vista previa: dos
/// operaciones de costo distinto" en el comentario de módulo).
///
/// Frontera lectura/escritura: esta función NO recibe `db_path` -- se puede
/// pedir y descartar cuantas veces haga falta sin ningún efecto (criterio de
/// aceptación #3 de STORY-073).
#[derive(Debug, Clone)]
pub struct ManualImportPreviewResultDto {
    /// Las primeras [`PREVIEW_ROW_LIMIT`] filas transformadas, en el mismo
    /// orden en que el archivo las trae -- EXACTAMENTE lo que
    /// `confirm_manual_bar_import` sellará si el usuario acepta (mismo
    /// parseo por fila, LDR-0008 condición técnica 1). Nunca más de
    /// [`PREVIEW_ROW_LIMIT`] filas -- el parseo se detiene ahí, no se trunca
    /// después de leer el resto del archivo.
    pub rows: Vec<PreviewCanonicalBarDto>,
    /// `null` en Dart si la vista previa se produjo con éxito. Poblado (y
    /// `rows` vacío) si el mapeo declarado es inválido (columna inexistente,
    /// campo obligatorio ausente, duplicado) o si alguna de las primeras
    /// [`PREVIEW_ROW_LIMIT`] filas no parsea -- nunca una vista previa
    /// parcial.
    pub error: Option<String>,
}

/// Resultado de "analizar el rango de contenido real del archivo completo" --
/// operación CARA, deliberadamente separada de la vista previa (ver
/// comentario de módulo). Da a la pantalla lo que el criterio 5b de
/// STORY-073 / DEBT-039 exige: la primera y la última fecha que el archivo
/// REALMENTE contiene, para contrastarlas contra el rango que el usuario
/// declaró en el Paso 5 antes de confirmar.
///
/// Frontera lectura/escritura: esta función NO recibe `db_path` -- analiza
/// bytes que Dart ya tiene en memoria, sin tocar ninguna base de datos.
#[derive(Debug, Clone)]
pub struct ManualImportContentRangeResultDto {
    /// Total de filas de dato que el archivo completo produjo -- para que la
    /// pantalla pueda mostrar "9.245.183 filas encontradas".
    pub rows_parsed_total: u64,
    /// Apertura de la primera barra real del archivo completo (nanosegundos
    /// UTC), calculada por reducción sobre TODAS las filas -- nunca leyendo
    /// solo la primera línea física (el archivo puede no venir ordenado).
    /// `null` si el archivo no tiene ninguna fila de dato.
    pub content_min_open_timestamp_ns: Option<i64>,
    /// Cierre de la última barra real del archivo completo, misma reducción
    /// que el campo anterior. `null` en el mismo caso que arriba.
    pub content_max_close_timestamp_ns: Option<i64>,
    /// `null` en Dart si el análisis se produjo con éxito. Poblado (y el
    /// resto de los campos en cero/`null`) si el mapeo declarado es inválido
    /// o si alguna fila del archivo completo no parsea.
    pub error: Option<String>,
}

/// Resultado de "confirmar la declaración" (cierre del Paso 6) -- el ÚNICO
/// de los cinco resultados con identidad de ledger (`declaration_id`,
/// `event_sequence_id`, `audit_hash`): las otras cuatro operaciones no tienen
/// nada de esto porque no sellaron nada.
///
/// Frontera lectura/escritura: esta es la ÚNICA de las cinco funciones que
/// recibe `db_path` -- la única que puede escribir.
#[derive(Debug, Clone)]
pub struct ManualImportConfirmResultDto {
    pub ok: bool,
    /// UUIDv7 de la declaración sellada. Cadena vacía si `ok` es `false`.
    pub declaration_id: String,
    /// Siempre `"MANUAL_USER"` cuando `ok` es `true` -- distinguible de
    /// `"PLATFORM_TEMPLATE"` (criterio de aceptación #4 de STORY-073).
    pub mapping_origin: String,
    pub event_sequence_id: i64,
    pub audit_hash: String,
    pub rows_imported: u64,
    /// `true` si este tramo (mismo archivo, mismo rango declarado) ya
    /// estaba sellado -- la importación fue un no-op, no una segunda fila
    /// de ledger (ADR-0163).
    pub was_noop_reimport: bool,
    /// `null` en Dart si `ok` es `true`. Poblado si el mapeo es inválido, si
    /// el rango declarado no cubre el contenido real (DEBT-039), si el Gate
    /// de Homogeneidad de Procedencia rechaza el tramo, o si el ledger no
    /// pudo sellarse -- nunca una confirmación parcial (criterio de
    /// aceptación #11).
    pub error: Option<String>,
}

impl ManualImportConfirmResultDto {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            declaration_id: String::new(),
            mapping_origin: String::new(),
            event_sequence_id: 0,
            audit_hash: String::new(),
            rows_imported: 0,
            was_noop_reimport: false,
            error: Some(msg),
        }
    }
}

// ── Conversión del vocabulario fijo de campos canónicos ─────────────────

/// Convierte el nombre de campo canónico recibido desde Dart al enum tipado
/// del Núcleo. `CanonicalBarField` no deriva `serde` (es dominio puro), así
/// que a diferencia de `parse_bar_type` en `hybrid_data_transformer.rs` esto
/// no puede delegar en serde -- es un `match` explícito contra el vocabulario
/// FIJO de diez campos que el enum `CanonicalBarField` del Núcleo declara
/// (DEBT-041: `TIMESTAMP_DATE`/`TIMESTAMP_TIME` se añadieron el 2026-07-31
/// junto a `TimestampDate`/`TimestampTime` en `domain.rs` -- la vía de fecha
/// y hora separadas del archivo real de MetaTrader del propietario, `<DATE>`
/// TAB `<TIME>`). No existe una constante `ALL` en el Núcleo que enumere las
/// diez variantes juntas -- `ALL_NON_TIMESTAMP` cubre solo las siete que no
/// son de tiempo, a propósito (la marca de tiempo se resuelve aparte, con su
/// propia lógica de dos estrategias excluyentes, en `resolve_manual_mapping`).
fn parse_canonical_bar_field(value: &str) -> Result<CanonicalBarField, String> {
    match value {
        "TIMESTAMP" => Ok(CanonicalBarField::Timestamp),
        "TIMESTAMP_DATE" => Ok(CanonicalBarField::TimestampDate),
        "TIMESTAMP_TIME" => Ok(CanonicalBarField::TimestampTime),
        "OPEN" => Ok(CanonicalBarField::Open),
        "HIGH" => Ok(CanonicalBarField::High),
        "LOW" => Ok(CanonicalBarField::Low),
        "CLOSE" => Ok(CanonicalBarField::Close),
        "VOLUME_TICKS" => Ok(CanonicalBarField::VolumeTicks),
        "VOLUME_TRADED" => Ok(CanonicalBarField::VolumeTraded),
        "SPREAD_REPORTED" => Ok(CanonicalBarField::SpreadReported),
        other => Err(format!(
            "canonical_field '{other}' no reconocido (se esperaba uno de los diez campos canónicos fijos: \
             TIMESTAMP, TIMESTAMP_DATE, TIMESTAMP_TIME, OPEN, HIGH, LOW, CLOSE, VOLUME_TICKS, VOLUME_TRADED, \
             SPREAD_REPORTED)"
        )),
    }
}

/// Traduce el `Vec<ManualColumnMappingDto>` recibido desde Dart al
/// `Vec<ManualColumnMapping>` que el Núcleo espera -- comparten
/// `preview_manual_bar_import` y `confirm_manual_bar_import`, así que vive
/// una sola vez.
fn parse_column_mappings(dtos: Vec<ManualColumnMappingDto>) -> Result<Vec<ManualColumnMapping>, String> {
    dtos.into_iter()
        .map(|dto| {
            let canonical_field = parse_canonical_bar_field(&dto.canonical_field)?;
            Ok(ManualColumnMapping { source_column: dto.source_column, canonical_field })
        })
        .collect()
}

// ── Funciones FFI ─────────────────────────────────────────────────────────

/// Operación 1/5 -- "seleccionar archivo" (Paso 1, Archivo). Puramente
/// mecánica: calcula tamaño y huella del buffer que Dart ya leyó del disco,
/// sin interpretar ni una sola columna.
///
/// ## Ownership al cruzar la frontera FFI
/// `file_bytes` se mueve hacia Rust; esta función no retiene ninguna copia
/// más allá de su propio cuerpo.
pub async fn select_manual_import_file(file_bytes: Vec<u8>) -> SelectManualImportFileResultDto {
    let file_size_bytes = file_bytes.len() as u64;
    let source_file_hash = compute_source_file_hash(&file_bytes);
    SelectManualImportFileResultDto { file_size_bytes, source_file_hash }
}

/// Operación 2/5 -- "obtener las columnas detectadas" (Paso 2, Mapeo).
/// Envoltura directa de `data_import_wizard::public_interface::
/// detect_csv_columns` -- delimitador detectado internamente, nunca
/// declarado por el usuario (es inferencia de FORMATO, no de significado).
pub async fn get_manual_import_columns(file_bytes: Vec<u8>) -> ManualImportColumnsResultDto {
    match detect_csv_columns(&file_bytes) {
        Ok(columns) => ManualImportColumnsResultDto { columns, error: None },
        Err(e) => ManualImportColumnsResultDto { columns: Vec::new(), error: Some(e.to_string()) },
    }
}

/// Operación 3/5 -- "pedir la vista previa transformada bajo un mapeo
/// propuesto" (ciclo interactivo dentro del Paso 6, Vista Previa y
/// Confirmación). Envoltura directa de `preview_manual_bar_file` -- la
/// función ACOTADA del Núcleo que comparte, fila a fila, el mismo parseo que
/// `confirm_manual_bar_import` sellará (LDR-0008 condición técnica 1), pero
/// SIN pagar el costo del archivo completo: se detiene tras
/// [`PREVIEW_ROW_LIMIT`] filas. Ver "Vista previa: dos operaciones de costo
/// distinto" en el comentario de módulo -- si necesitas el total de filas o
/// el rango de contenido real, esa es [`analyze_manual_bar_file_content_range`],
/// deliberadamente aparte.
///
/// ## Por qué NO recibe `db_path`
/// Ver "La frontera lectura/escritura" en el comentario de módulo. Esta
/// función se puede llamar cientos de veces mientras el usuario ajusta su
/// mapeo -- cada llamada es independiente, barata, y no deja rastro.
///
/// ## Error handling
/// Nunca hace panic ante un mapeo o un archivo malformado: devuelve
/// `ManualImportPreviewResultDto` con `error` poblado y `rows` vacío.
#[allow(clippy::too_many_arguments)]
pub async fn preview_manual_bar_import(
    file_bytes: Vec<u8>,
    column_mappings: Vec<ManualColumnMappingDto>,
    instrument_id: String,
    venue: String,
    source_provider: String,
    source_clock_anchor_timezone_ref: Option<String>,
    source_clock_fixed_offset_minutes: i64,
    real_volume_unit_declared: Option<String>,
    spread_reported_unit_declared: Option<String>,
    timeframe_value: String,
    bar_interval_ns: i64,
) -> ManualImportPreviewResultDto {
    let mappings = match parse_column_mappings(column_mappings) {
        Ok(m) => m,
        Err(e) => return ManualImportPreviewResultDto { rows: Vec::new(), error: Some(e) },
    };

    let ctx = ManualRunContext {
        instrument_id: &instrument_id,
        venue: &venue,
        source_provider: &source_provider,
        source_clock_anchor_timezone_ref: source_clock_anchor_timezone_ref.as_deref(),
        source_clock_fixed_offset_minutes,
        real_volume_unit_declared: real_volume_unit_declared.as_deref(),
        spread_reported_unit_declared: spread_reported_unit_declared.as_deref(),
        timeframe_value: &timeframe_value,
        bar_interval_ns,
    };

    // `preview_manual_bar_file` ya se detiene sola en `PREVIEW_ROW_LIMIT`
    // filas -- no hace falta un `.take()` aquí encima: sería una segunda
    // fuente de verdad para el mismo número.
    match preview_manual_bar_file(&file_bytes, &mappings, &ctx) {
        Ok(bars) => {
            // Verificación defensiva de la garantía que sostiene todo este
            // diseño: si `preview_manual_bar_file` dejara de acotarse a sí
            // misma, esta línea lo hace explícito en vez de dejar que la
            // vista previa crezca en silencio hasta el tamaño del archivo.
            debug_assert!(
                bars.len() <= PREVIEW_ROW_LIMIT,
                "preview_manual_bar_file debe detenerse en PREVIEW_ROW_LIMIT filas, devolvió {}",
                bars.len()
            );
            let rows = bars.iter().map(PreviewCanonicalBarDto::from).collect();
            ManualImportPreviewResultDto { rows, error: None }
        }
        Err(e) => ManualImportPreviewResultDto { rows: Vec::new(), error: Some(e.to_string()) },
    }
}

/// Operación 4/5 -- "analizar el rango de contenido real del archivo
/// completo" (llamada explícita, una sola vez, al llegar al Paso 6 con el
/// mapeo ya estable -- nunca en el ciclo de ajuste de mapeo). Envoltura
/// directa de `parse_and_map_manual_bar_file` -- la función SIN LÍMITE del
/// Núcleo, la misma que `confirm_manual_bar_import` usa internamente para
/// sellar. Es la operación CARA que DEBT-039 exige para el criterio 5b: sin
/// ella, no hay forma honesta de contrastar "lo que el usuario declaró"
/// contra "lo que el archivo realmente contiene" -- acotar el parseo (como
/// hace la operación 3/5) descarta exactamente el dato que este criterio
/// necesita.
///
/// ## Por qué NO recibe `db_path`
/// Ver "La frontera lectura/escritura" en el comentario de módulo -- analiza
/// bytes en memoria, nunca toca la base de datos.
///
/// ## Error handling
/// Nunca hace panic ante un mapeo o un archivo malformado: devuelve
/// `ManualImportContentRangeResultDto` con `error` poblado y el resto de los
/// campos en cero/`null`.
#[allow(clippy::too_many_arguments)]
pub async fn analyze_manual_bar_file_content_range(
    file_bytes: Vec<u8>,
    column_mappings: Vec<ManualColumnMappingDto>,
    instrument_id: String,
    venue: String,
    source_provider: String,
    source_clock_anchor_timezone_ref: Option<String>,
    source_clock_fixed_offset_minutes: i64,
    real_volume_unit_declared: Option<String>,
    spread_reported_unit_declared: Option<String>,
    timeframe_value: String,
    bar_interval_ns: i64,
) -> ManualImportContentRangeResultDto {
    let mappings = match parse_column_mappings(column_mappings) {
        Ok(m) => m,
        Err(e) => {
            return ManualImportContentRangeResultDto {
                rows_parsed_total: 0,
                content_min_open_timestamp_ns: None,
                content_max_close_timestamp_ns: None,
                error: Some(e),
            }
        }
    };

    let ctx = ManualRunContext {
        instrument_id: &instrument_id,
        venue: &venue,
        source_provider: &source_provider,
        source_clock_anchor_timezone_ref: source_clock_anchor_timezone_ref.as_deref(),
        source_clock_fixed_offset_minutes,
        real_volume_unit_declared: real_volume_unit_declared.as_deref(),
        spread_reported_unit_declared: spread_reported_unit_declared.as_deref(),
        timeframe_value: &timeframe_value,
        bar_interval_ns,
    };

    match parse_and_map_manual_bar_file(&file_bytes, &mappings, &ctx) {
        Ok(bars) => {
            // Reducción sobre TODAS las filas -- nunca se asume que el
            // archivo viene ordenado, así que nunca se lee solo la primera y
            // la última línea física. Misma definición que
            // `check_declared_range_against_parsed_content` usa en el
            // Núcleo para su propio cotejo (DEBT-039); aquí es informativo
            // para la pantalla, no una compuerta de rechazo.
            let content_min_open_timestamp_ns = bars.iter().map(|b| b.bar_open_timestamp_ns).min();
            let content_max_close_timestamp_ns = bars.iter().map(|b| b.bar_close_timestamp_ns).max();
            let rows_parsed_total = bars.len() as u64;
            ManualImportContentRangeResultDto {
                rows_parsed_total,
                content_min_open_timestamp_ns,
                content_max_close_timestamp_ns,
                error: None,
            }
        }
        Err(e) => ManualImportContentRangeResultDto {
            rows_parsed_total: 0,
            content_min_open_timestamp_ns: None,
            content_max_close_timestamp_ns: None,
            error: Some(e.to_string()),
        },
    }
}

/// Operación 5/5 -- "confirmar la declaración" (cierre del Paso 6). Envoltura
/// directa de `import_manual_bar_file` -- sella el ledger con
/// `mapping_origin = "MANUAL_USER"` ANTES de invocar el puerto de salida.
///
/// ## Por qué es la ÚNICA función con `db_path`
/// Ver "La frontera lectura/escritura" en el comentario de módulo -- esta es
/// la única de las cinco operaciones que puede escribir, y es la única cuya
/// firma admite la ruta de base de datos que hace falta para hacerlo.
///
/// ## El puerto de salida en esta Story (DEBT-036)
/// El puerto `bar_sink` acepta siempre (consumidor en memoria) -- la
/// persistencia real de las barras queda fuera de esta Story por decisión
/// explícita (§4.1 de la Orden, "Lo que NO entra"); se conecta en la entrega
/// siguiente. `rows_imported` refleja cuántas filas produjo el parseo, no
/// cuántas quedaron escritas en disco todavía.
///
/// ## Error handling
/// Nunca hace panic. Cubre, sin distinguir con un tipo propio (mismo patrón
/// que `TransformOutcomeDto`/`VerifyOutput` en este mismo crate): mapeo
/// inválido, rango declarado que no cubre el contenido real (DEBT-039),
/// rechazo del Gate de Homogeneidad de Procedencia, y fallo del repositorio.
#[allow(clippy::too_many_arguments)]
pub async fn confirm_manual_bar_import(
    db_path: String,
    source_file_name: String,
    file_bytes: Vec<u8>,
    instrument_id: String,
    venue: String,
    source_provider: String,
    declared_range_start_ns: i64,
    declared_range_end_ns: i64,
    source_clock_anchor_timezone_ref: Option<String>,
    source_clock_fixed_offset_minutes: i64,
    real_volume_unit_declared: Option<String>,
    spread_reported_unit_declared: Option<String>,
    venue_operational_since_ns: Option<i64>,
    timeframe_value: String,
    bar_interval_ns: i64,
    column_mappings: Vec<ManualColumnMappingDto>,
    node_id: Option<String>,
    process_id: Option<String>,
) -> ManualImportConfirmResultDto {
    let mappings = match parse_column_mappings(column_mappings) {
        Ok(m) => m,
        Err(e) => return ManualImportConfirmResultDto::from_error(e),
    };

    let url = format!("sqlite://{db_path}");
    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(e) => return ManualImportConfirmResultDto::from_error(format!("no se pudo abrir la base de datos '{db_path}': {e}")),
    };
    if let Err(e) = run_migrations(&pool).await {
        return ManualImportConfirmResultDto::from_error(format!("error al aplicar migraciones: {e}"));
    }

    let clock = SystemClock::new();
    let input = ManualImportBarFileInput {
        source_file_name,
        file_bytes,
        instrument_id,
        venue,
        source_provider,
        declared_range_start_ns,
        declared_range_end_ns,
        source_clock_anchor_timezone_ref,
        source_clock_fixed_offset_minutes,
        real_volume_unit_declared,
        spread_reported_unit_declared,
        venue_operational_since_ns,
        timeframe_value,
        bar_interval_ns,
        column_mappings: mappings,
        node_id,
        process_id,
    };

    // Puerto de salida en memoria -- ver "El puerto de salida en esta
    // Story (DEBT-036)" arriba: acepta siempre, no persiste nada todavía.
    match import_manual_bar_file(&pool, &clock, input, |_rows| Ok(())).await {
        Ok(outcome) => ManualImportConfirmResultDto {
            ok: true,
            declaration_id: outcome.declaration.id,
            mapping_origin: outcome.declaration.mapping_origin,
            event_sequence_id: outcome.declaration.event_sequence_id,
            audit_hash: outcome.declaration.audit_hash,
            rows_imported: outcome.bars.len() as u64,
            was_noop_reimport: outcome.was_noop_reimport,
            error: None,
        },
        Err(e) => ManualImportConfirmResultDto::from_error(describe_confirm_error(&e)),
    }
}

/// Mensaje de error legible -- todas las variantes de
/// `ManualImportBarFileError` ya implementan `Display` vía `thiserror`
/// (`#[error(transparent)]`/mensajes propios), así que `.to_string()` en un
/// solo punto cubre las cuatro (mapeo inválido, Gate de Homogeneidad,
/// cotejo de rango DEBT-039, repositorio) sin un `match` de cuatro brazos
/// que mantener sincronizado con el Núcleo.
fn describe_confirm_error(e: &ManualImportBarFileError) -> String {
    e.to_string()
}

// ── Pruebas de integración -- criterio de aceptación #7 de STORY-073 ──────
//
// Capa 2 (ADR-0133): contra el binario real de este crate (`cargo test -p
// bridge`), con SQLite en archivo temporal -- nunca `:memory:` -- para que
// la prueba de "cero escrituras" sea sobre el MISMO archivo que
// `confirm_manual_bar_import` habría usado, no sobre una conexión efímera
// que no prueba nada sobre persistencia real.

#[cfg(test)]
mod tests {
    use super::*;

    /// Ruta de BD SQLite temporal única por prueba -- mismo patrón que
    /// `hybrid_data_transformer.rs` (evita colisión entre pruebas paralelas).
    fn temp_db_path(label: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("el reloj del sistema es posterior a la época Unix")
            .as_nanos();
        std::env::temp_dir().join(format!("drasus-bridge-test-diw-{label}-{nanos}.db")).to_string_lossy().to_string()
    }

    /// Cuenta las filas de `import_mapping_declarations` directamente contra
    /// el archivo SQLite -- verificación independiente de cualquier función
    /// de este módulo, para que la prueba de "cero escrituras" no dependa de
    /// la misma capa que se está poniendo a prueba.
    async fn count_sealed_declarations(db_path: &str) -> i64 {
        let url = format!("sqlite://{db_path}");
        let pool = create_pool(&url).await.expect("pool de verificación");
        run_migrations(&pool).await.expect("migraciones de verificación");
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM import_mapping_declarations")
            .fetch_one(&pool)
            .await
            .expect("el conteo de declaraciones selladas debe poder leerse")
    }

    /// Da de alta el instrumento EURUSD en `instrument_master` -- la tabla
    /// de declaraciones tiene una `FOREIGN KEY` sobre `instrument_id`
    /// (migración 0023), así que toda prueba que llame a
    /// `confirm_manual_bar_import` necesita este seed primero, exactamente
    /// como ya hace `orchestrator.rs::temp_pool_with_eurusd` dentro del
    /// propio crate de la feature.
    async fn seed_eurusd_instrument(db_path: &str) {
        let url = format!("sqlite://{db_path}");
        let pool = create_pool(&url).await.expect("pool de seed");
        run_migrations(&pool).await.expect("migraciones de seed");
        sqlx::query(
            "INSERT INTO instrument_master (id, created_at, updated_at, audit_hash, row_version, \
             instrument_id, asset_class, canonical_pip_size, canonical_tick_size) \
             VALUES ('bridge-test-instrument-eurusd', 0, 0, 'seed-hash', 1, 'EURUSD', 'FOREX', 10000, 1000)",
        )
        .execute(&pool)
        .await
        .expect("la fila semilla de instrument_master debe insertarse");
    }

    /// CSV de columnas arbitrarias -- mismo espíritu que `ARBITRARY_CSV` de
    /// `orchestrator.rs` (nombres en español, orden barajado, delimitador
    /// coma), construido aquí para no depender de un fixture del crate de la
    /// feature (fuera de alcance: "no toques data-import-wizard").
    const ARBITRARY_CSV: &[u8] = b"SpreadPts,Cierre,Fecha,VolumenReal,Maximo,Minimo,VolumenTicks,Apertura\n\
         5,2.50500000,2024-03-01 10:00:00,1000,2.51000000,2.49000000,10,2.50000000\n\
         6,2.50900000,2024-03-01 10:05:00,1200,2.51200000,2.50400000,12,2.50500000\n";

    fn arbitrary_column_mappings() -> Vec<ManualColumnMappingDto> {
        vec![
            ManualColumnMappingDto { source_column: "Fecha".to_string(), canonical_field: "TIMESTAMP".to_string() },
            ManualColumnMappingDto { source_column: "Apertura".to_string(), canonical_field: "OPEN".to_string() },
            ManualColumnMappingDto { source_column: "Maximo".to_string(), canonical_field: "HIGH".to_string() },
            ManualColumnMappingDto { source_column: "Minimo".to_string(), canonical_field: "LOW".to_string() },
            ManualColumnMappingDto { source_column: "Cierre".to_string(), canonical_field: "CLOSE".to_string() },
            ManualColumnMappingDto {
                source_column: "VolumenTicks".to_string(),
                canonical_field: "VOLUME_TICKS".to_string(),
            },
            ManualColumnMappingDto {
                source_column: "VolumenReal".to_string(),
                canonical_field: "VOLUME_TRADED".to_string(),
            },
            ManualColumnMappingDto {
                source_column: "SpreadPts".to_string(),
                canonical_field: "SPREAD_REPORTED".to_string(),
            },
        ]
    }

    /// CSV con fecha y hora en columnas SEPARADAS, delimitador TAB -- la
    /// FORMA del archivo real de MetaTrader del propietario (`<DATE>` TAB
    /// `<TIME>`, DEBT-041). Mismos valores numéricos que `ARBITRARY_CSV` fila
    /// por fila (mismo Open/High/Low/Close/volumen/spread), para que la
    /// prueba que cruza ambos CSV pueda afirmar el mismo timestamp resultante
    /// por dos caminos de parseo distintos -- combinado vs. separado.
    const ARBITRARY_CSV_SPLIT_DATE_TIME: &[u8] = b"<DATE>\t<TIME>\t<OPEN>\t<HIGH>\t<LOW>\t<CLOSE>\t<TICKVOL>\t<VOL>\t<SPREAD>\n\
         2024.03.01\t10:00:00\t2.50000000\t2.51000000\t2.49000000\t2.50500000\t10\t1000\t5\n\
         2024.03.01\t10:05:00\t2.50500000\t2.51200000\t2.50400000\t2.50900000\t12\t1200\t6\n";

    /// Mapeo del CSV separado -- declara `TIMESTAMP_DATE` + `TIMESTAMP_TIME`
    /// en vez de `TIMESTAMP` (la vía que DEBT-041 dejaba sin forma sintáctica
    /// de declararse desde Dart).
    fn split_date_time_column_mappings() -> Vec<ManualColumnMappingDto> {
        vec![
            ManualColumnMappingDto { source_column: "<DATE>".to_string(), canonical_field: "TIMESTAMP_DATE".to_string() },
            ManualColumnMappingDto { source_column: "<TIME>".to_string(), canonical_field: "TIMESTAMP_TIME".to_string() },
            ManualColumnMappingDto { source_column: "<OPEN>".to_string(), canonical_field: "OPEN".to_string() },
            ManualColumnMappingDto { source_column: "<HIGH>".to_string(), canonical_field: "HIGH".to_string() },
            ManualColumnMappingDto { source_column: "<LOW>".to_string(), canonical_field: "LOW".to_string() },
            ManualColumnMappingDto { source_column: "<CLOSE>".to_string(), canonical_field: "CLOSE".to_string() },
            ManualColumnMappingDto {
                source_column: "<TICKVOL>".to_string(),
                canonical_field: "VOLUME_TICKS".to_string(),
            },
            ManualColumnMappingDto { source_column: "<VOL>".to_string(), canonical_field: "VOLUME_TRADED".to_string() },
            ManualColumnMappingDto {
                source_column: "<SPREAD>".to_string(),
                canonical_field: "SPREAD_REPORTED".to_string(),
            },
        ]
    }

    /// Rango declarado por defecto de las pruebas: el día completo
    /// 2024-03-01 UTC.
    fn arbitrary_declared_range() -> (i64, i64) {
        use chrono::{TimeZone, Utc};
        let start = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();
        let end = Utc.with_ymd_and_hms(2024, 3, 2, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();
        (start, end)
    }

    #[allow(clippy::too_many_arguments)]
    async fn preview_arbitrary(file_bytes: Vec<u8>) -> ManualImportPreviewResultDto {
        preview_manual_bar_import(
            file_bytes,
            arbitrary_column_mappings(),
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            Some("UTC".to_string()),
            0,
            Some("CONTRACTS".to_string()),
            Some("POINTS_GENERIC".to_string()),
            "5M".to_string(),
            300_000_000_000,
        )
        .await
    }

    /// Mismos argumentos que `preview_arbitrary`, pero contra la operación
    /// CARA (`analyze_manual_bar_file_content_range`) -- las pruebas del
    /// rango de contenido (DEBT-039) usan esta, nunca la vista previa barata,
    /// porque la vista previa acotada no tiene forma de conocer el rango
    /// real del archivo completo.
    #[allow(clippy::too_many_arguments)]
    async fn analyze_arbitrary(file_bytes: Vec<u8>) -> ManualImportContentRangeResultDto {
        analyze_manual_bar_file_content_range(
            file_bytes,
            arbitrary_column_mappings(),
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            Some("UTC".to_string()),
            0,
            Some("CONTRACTS".to_string()),
            Some("POINTS_GENERIC".to_string()),
            "5M".to_string(),
            300_000_000_000,
        )
        .await
    }

    /// Construye un CSV con `valid_rows` filas de dato válidas (mismo
    /// esquema que `ARBITRARY_CSV`, hora incrementando 5 minutos por fila) y,
    /// si `append_malformed_row` es `true`, UNA fila adicional al final con
    /// un campo de precio no numérico -- garantiza que cualquier parseo que
    /// SÍ llegue a leerla falle.
    fn csv_with_rows(valid_rows: usize, append_malformed_row: bool) -> Vec<u8> {
        let mut csv = String::from("SpreadPts,Cierre,Fecha,VolumenReal,Maximo,Minimo,VolumenTicks,Apertura\n");
        for i in 0..valid_rows {
            let minute = i * 5;
            csv.push_str(&format!(
                "5,2.505{i:05},2024-03-01 10:{minute:02}:00,1000,2.510{i:05},2.490{i:05},10,2.500{i:05}\n",
            ));
        }
        if append_malformed_row {
            let minute = valid_rows * 5;
            csv.push_str(&format!("5,NO_ES_UN_NUMERO,2024-03-01 10:{minute:02}:00,1000,2.51000000,2.49000000,10,2.50000000\n"));
        }
        csv.into_bytes()
    }

    #[allow(clippy::too_many_arguments)]
    async fn confirm_arbitrary(db_path: String, file_bytes: Vec<u8>) -> ManualImportConfirmResultDto {
        let (start, end) = arbitrary_declared_range();
        confirm_manual_bar_import(
            db_path,
            "broker_generico_export.csv".to_string(),
            file_bytes,
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            start,
            end,
            Some("UTC".to_string()),
            0,
            Some("CONTRACTS".to_string()),
            Some("POINTS_GENERIC".to_string()),
            None,
            "5M".to_string(),
            300_000_000_000,
            arbitrary_column_mappings(),
            Some("test-node".to_string()),
            Some("test-process".to_string()),
        )
        .await
    }

    /// Mismo espíritu que `confirm_arbitrary`, pero sobre el CSV de fecha y
    /// hora SEPARADAS (`ARBITRARY_CSV_SPLIT_DATE_TIME` + `split_date_time_
    /// column_mappings`) -- la vía que DEBT-041 bloqueaba de punta a punta.
    #[allow(clippy::too_many_arguments)]
    async fn confirm_split_date_time(db_path: String, file_bytes: Vec<u8>) -> ManualImportConfirmResultDto {
        let (start, end) = arbitrary_declared_range();
        confirm_manual_bar_import(
            db_path,
            "mt5_export.csv".to_string(),
            file_bytes,
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            start,
            end,
            Some("UTC".to_string()),
            0,
            Some("CONTRACTS".to_string()),
            Some("POINTS_GENERIC".to_string()),
            None,
            "5M".to_string(),
            300_000_000_000,
            split_date_time_column_mappings(),
            Some("test-node".to_string()),
            Some("test-process".to_string()),
        )
        .await
    }

    /// Operación 1: "seleccionar archivo" -- tamaño y huella correctos, sin
    /// interpretar ninguna columna.
    #[tokio::test(flavor = "multi_thread")]
    async fn select_reports_size_and_hash_of_the_real_bytes() {
        let result = select_manual_import_file(ARBITRARY_CSV.to_vec()).await;
        assert_eq!(result.file_size_bytes, ARBITRARY_CSV.len() as u64);
        assert!(!result.source_file_hash.is_empty());

        // Discriminación: contenido DISTINTO produce huella DISTINTA (nunca
        // una aserción autorreferencial contra la propia función).
        let other = select_manual_import_file(b"contenido completamente distinto".to_vec()).await;
        assert_ne!(result.source_file_hash, other.source_file_hash);
    }

    /// Operación 2: "obtener las columnas detectadas" -- las ocho columnas
    /// reales de la cabecera, en el orden real del archivo.
    #[tokio::test(flavor = "multi_thread")]
    async fn get_columns_returns_the_real_header_in_order() {
        let result = get_manual_import_columns(ARBITRARY_CSV.to_vec()).await;
        assert!(result.error.is_none(), "no debe fallar: {:?}", result.error);
        assert_eq!(
            result.columns,
            vec!["SpreadPts", "Cierre", "Fecha", "VolumenReal", "Maximo", "Minimo", "VolumenTicks", "Apertura"]
        );
    }

    /// Un archivo vacío se rechaza con error explícito, nunca con un panic
    /// ni con una lista de columnas inventada.
    #[tokio::test(flavor = "multi_thread")]
    async fn get_columns_rejects_empty_file_with_explicit_error() {
        let result = get_manual_import_columns(Vec::new()).await;
        assert!(result.error.is_some());
        assert!(result.columns.is_empty());
    }

    /// Operación 3, criterio de aceptación #2: la vista previa muestra DATO
    /// transformado (escala entera, ns UTC) -- no nombres de columna.
    #[tokio::test(flavor = "multi_thread")]
    async fn preview_shows_transformed_data_not_column_names() {
        let preview = preview_arbitrary(ARBITRARY_CSV.to_vec()).await;
        assert!(preview.error.is_none(), "no debe fallar: {:?}", preview.error);
        assert_eq!(preview.rows.len(), 2);

        let first = &preview.rows[0];
        // 2.50500000 declarado como Close, escalado ×10⁸ (ADR-0141) -- un
        // valor NUMÉRICO derivado, nunca el texto "Cierre" ni "2.50500000".
        assert_eq!(first.close, 250_500_000);
        assert_eq!(first.open, 250_000_000);
        assert_eq!(first.volume_traded_raw, 1000);
        assert_eq!(first.spread_reported_raw, 5);
        // 2024-03-01 10:00:00 UTC en nanosegundos -- ns UTC real, no una
        // fecha de texto.
        assert_eq!(first.bar_open_timestamp_ns, 1_709_287_200_000_000_000);
    }

    /// DEBT-041 -- la vía de fecha y hora en columnas SEPARADAS (el formato
    /// del archivo real de MetaTrader del propietario, `<DATE>` TAB `<TIME>`)
    /// llega hasta el Núcleo a través del puente FFI. Antes del arreglo,
    /// `parse_canonical_bar_field` rechazaba `"TIMESTAMP_DATE"`/
    /// `"TIMESTAMP_TIME"` con "canonical_field '...' no reconocido" y esta
    /// prueba fallaba en el primer `assert!` con ese mensaje.
    ///
    /// La aserción NO es autorreferencial: `bar_open_timestamp_ns` se compara
    /// contra el MISMO valor literal (`1_709_287_200_000_000_000`) que
    /// `preview_shows_transformed_data_not_column_names` obtiene por el
    /// camino de la columna COMBINADA -- dos parseos distintos (uno combina
    /// `<DATE>`+`<TIME>` con `NaiveDate::and_time`, el otro parsea una sola
    /// columna) deben producir el mismo instante real si la combinación es
    /// correcta.
    #[tokio::test(flavor = "multi_thread")]
    async fn preview_via_split_date_and_time_columns_matches_the_combined_timestamp_path() {
        let preview = preview_manual_bar_import(
            ARBITRARY_CSV_SPLIT_DATE_TIME.to_vec(),
            split_date_time_column_mappings(),
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            Some("UTC".to_string()),
            0,
            Some("CONTRACTS".to_string()),
            Some("POINTS_GENERIC".to_string()),
            "5M".to_string(),
            300_000_000_000,
        )
        .await;

        assert!(preview.error.is_none(), "TIMESTAMP_DATE/TIMESTAMP_TIME deben reconocerse: {:?}", preview.error);
        assert_eq!(preview.rows.len(), 2);

        let first = &preview.rows[0];
        assert_eq!(first.open, 250_000_000);
        assert_eq!(first.close, 250_500_000);
        // Mismo instante que la fila 1 de `ARBITRARY_CSV` (2024-03-01
        // 10:00:00 UTC), ahora derivado de `<DATE>`="2024.03.01" +
        // `<TIME>`="10:00:00" combinadas por el Núcleo, no de una sola
        // columna.
        assert_eq!(first.bar_open_timestamp_ns, 1_709_287_200_000_000_000);
    }

    /// EL CORAZÓN DE LA SEPARACIÓN DE COSTOS: `preview_manual_bar_import`
    /// jamás debe pagar el costo del archivo completo. Se prueba con un CSV
    /// de [`PREVIEW_ROW_LIMIT`] filas VÁLIDAS seguidas de UNA fila 11
    /// deliberadamente malformada (precio no numérico). Si la vista previa
    /// llamara, aunque fuera internamente, a la función SIN LÍMITE
    /// (`parse_and_map_manual_bar_file`), el parseo completo tropezaría con
    /// la fila 11 y devolvería `error: Some(..)` -- exactamente el defecto
    /// que motivó separar esta operación. Con la función ACOTADA
    /// (`preview_manual_bar_file`), el parseo se detiene en la fila 10 y la
    /// fila 11 malformada NUNCA se lee: la vista previa tiene éxito. Esto
    /// discrimina el camino barato del camino caro por COMPORTAMIENTO
    /// observable, no solo por el conteo final de filas (un `.take()`
    /// posterior a un parseo completo habría dado el mismo conteo pero
    /// habría fallado igual ante la fila 11).
    #[tokio::test(flavor = "multi_thread")]
    async fn preview_never_reads_past_the_row_limit_even_when_later_rows_are_malformed() {
        let csv = csv_with_rows(PREVIEW_ROW_LIMIT, true);
        let preview = preview_arbitrary(csv).await;

        assert!(
            preview.error.is_none(),
            "la vista previa acotada NO debe tropezar con la fila malformada más allá de PREVIEW_ROW_LIMIT: {:?}",
            preview.error
        );
        assert_eq!(preview.rows.len(), PREVIEW_ROW_LIMIT, "la vista previa nunca debe exceder PREVIEW_ROW_LIMIT filas");

        // Confirmación cruzada: la operación CARA, sobre el MISMO archivo,
        // SÍ debe tropezar con la fila malformada -- prueba que la
        // discriminación de arriba no es casualidad del fixture, sino la
        // consecuencia real de parsear el archivo completo.
        let analysis = analyze_arbitrary(csv_with_rows(PREVIEW_ROW_LIMIT, true)).await;
        assert!(
            analysis.error.is_some(),
            "el análisis SIN LÍMITE, sobre el mismo archivo, debe fallar en la fila malformada"
        );
    }

    /// Rango de contenido real expuesto para el criterio 5b (DEBT-039): la
    /// primera apertura y la última cierre del archivo COMPLETO, calculadas
    /// sobre las dos filas reales. Ejercita la operación CARA
    /// (`analyze_manual_bar_file_content_range`), nunca la vista previa --
    /// la vista previa acotada no tiene forma de conocer este dato.
    #[tokio::test(flavor = "multi_thread")]
    async fn analyze_reports_the_real_content_range_for_debt_039() {
        let analysis = analyze_arbitrary(ARBITRARY_CSV.to_vec()).await;
        assert!(analysis.error.is_none(), "no debe fallar: {:?}", analysis.error);
        assert_eq!(analysis.rows_parsed_total, 2);
        let expected_min_open = 1_709_287_200_000_000_000; // fila 1, 10:00:00
        let expected_max_close = 1_709_287_500_000_000_000 + 300_000_000_000; // fila 2, 10:05:00 + 5m
        assert_eq!(analysis.content_min_open_timestamp_ns, Some(expected_min_open));
        assert_eq!(analysis.content_max_close_timestamp_ns, Some(expected_max_close));
    }

    /// Criterio de aceptación #6: un campo canónico obligatorio sin declarar
    /// se rechaza -- nunca se asume.
    #[tokio::test(flavor = "multi_thread")]
    async fn preview_rejects_mapping_missing_a_required_field() {
        let mut mappings = arbitrary_column_mappings();
        mappings.retain(|m| m.canonical_field != "SPREAD_REPORTED"); // quita un campo obligatorio

        let preview = preview_manual_bar_import(
            ARBITRARY_CSV.to_vec(),
            mappings,
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            Some("UTC".to_string()),
            0,
            Some("CONTRACTS".to_string()),
            Some("POINTS_GENERIC".to_string()),
            "5M".to_string(),
            300_000_000_000,
        )
        .await;

        assert!(preview.error.is_some(), "sin SpreadReported declarado, debe rechazarse");
        assert!(preview.rows.is_empty());
    }

    /// Un `canonical_field` fuera del vocabulario fijo se rechaza antes de
    /// tocar el Núcleo.
    #[tokio::test(flavor = "multi_thread")]
    async fn preview_rejects_unknown_canonical_field_before_touching_the_core() {
        let mut mappings = arbitrary_column_mappings();
        mappings[0].canonical_field = "NO_ES_UN_CAMPO_REAL".to_string();

        let preview = preview_manual_bar_import(
            ARBITRARY_CSV.to_vec(),
            mappings,
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            Some("UTC".to_string()),
            0,
            None,
            Some("POINTS_GENERIC".to_string()),
            "5M".to_string(),
            300_000_000_000,
        )
        .await;

        assert!(preview.error.is_some());
    }

    /// EL CORAZÓN DEL CRITERIO 7 -- LDR-0008 condición 1: pedir la vista
    /// previa DIEZ veces contra una base de datos real y verificar, por
    /// consulta SQL directa (no por el valor de retorno de ninguna función
    /// de este módulo), que el ledger sigue en CERO filas. `preview_manual_
    /// bar_import` ni siquiera recibe el `db_path` -- esta prueba confirma
    /// que la ausencia del parámetro se traduce, en la práctica, en cero
    /// escrituras reales.
    #[tokio::test(flavor = "multi_thread")]
    async fn requesting_preview_ten_times_never_writes_to_the_real_ledger() {
        let db_path = temp_db_path("preview-noop");
        // La BD existe y tiene el esquema migrado ANTES de pedir ninguna
        // vista previa -- si `preview_manual_bar_import` pudiera escribir,
        // esta es la BD donde se vería.
        let _ = count_sealed_declarations(&db_path).await;

        for _ in 0..10 {
            let preview = preview_arbitrary(ARBITRARY_CSV.to_vec()).await;
            assert!(preview.error.is_none(), "cada vista previa debe tener éxito: {:?}", preview.error);
        }

        assert_eq!(count_sealed_declarations(&db_path).await, 0, "diez vistas previas no deben haber sellado ninguna fila");
    }

    /// La operación CARA (`analyze_manual_bar_file_content_range`) tampoco
    /// tiene `db_path` en su firma -- misma garantía estructural que la
    /// vista previa, verificada aquí de la misma forma: consulta SQL
    /// directa, no el valor de retorno de la propia función.
    #[tokio::test(flavor = "multi_thread")]
    async fn requesting_content_range_analysis_never_writes_to_the_real_ledger() {
        let db_path = temp_db_path("analyze-noop");
        let _ = count_sealed_declarations(&db_path).await;

        let analysis = analyze_arbitrary(ARBITRARY_CSV.to_vec()).await;
        assert!(analysis.error.is_none(), "el análisis debe tener éxito: {:?}", analysis.error);

        assert_eq!(count_sealed_declarations(&db_path).await, 0, "analizar el rango no debe haber sellado ninguna fila");
    }

    /// Operación 5, criterio de aceptación #4: confirmar sella
    /// `mapping_origin = "MANUAL_USER"`, con identidad de ledger real
    /// (`event_sequence_id`/`audit_hash` no vacíos).
    #[tokio::test(flavor = "multi_thread")]
    async fn confirm_seals_manual_user_origin_with_real_ledger_identity() {
        let db_path = temp_db_path("confirm-origin");
        seed_eurusd_instrument(&db_path).await;
        let result = confirm_arbitrary(db_path.clone(), ARBITRARY_CSV.to_vec()).await;

        assert!(result.ok, "no debe fallar: {:?}", result.error);
        assert_eq!(result.mapping_origin, "MANUAL_USER");
        assert!(!result.declaration_id.is_empty());
        assert!(!result.audit_hash.is_empty());
        assert_eq!(result.event_sequence_id, 1, "primera declaración de una BD nueva");
        assert_eq!(result.rows_imported, 2);
        assert!(!result.was_noop_reimport);

        assert_eq!(count_sealed_declarations(&db_path).await, 1, "confirmar SÍ debe sellar exactamente una fila");
    }

    /// DEBT-041, criterio de aceptación #13 de STORY-073 -- la vía de fecha y
    /// hora SEPARADAS se sella de PUNTA A PUNTA por el puente: desde los DTO
    /// que Dart construye (`"TIMESTAMP_DATE"`/`"TIMESTAMP_TIME"`), pasando
    /// por `confirm_manual_bar_import` con `db_path` real, hasta una fila
    /// contada por consulta SQL directa contra el archivo SQLite -- no el
    /// valor de retorno de la propia función. Antes del arreglo, esta
    /// aserción de `ok` fallaba con "canonical_field 'TIMESTAMP_DATE' no
    /// reconocido" y nada llegaba a sellarse.
    #[tokio::test(flavor = "multi_thread")]
    async fn confirm_seals_via_split_date_and_time_columns_end_to_end() {
        let db_path = temp_db_path("confirm-split-date-time");
        seed_eurusd_instrument(&db_path).await;
        let result = confirm_split_date_time(db_path.clone(), ARBITRARY_CSV_SPLIT_DATE_TIME.to_vec()).await;

        assert!(result.ok, "el archivo con <DATE>/<TIME> separadas debe importarse: {:?}", result.error);
        assert_eq!(result.mapping_origin, "MANUAL_USER");
        assert!(!result.declaration_id.is_empty());
        assert_eq!(result.rows_imported, 2);

        assert_eq!(
            count_sealed_declarations(&db_path).await,
            1,
            "la vía de fecha+hora separadas debe sellar exactamente una fila del ledger"
        );
    }

    /// Reimportar el MISMO tramo es un no-op -- no duplica el ledger.
    #[tokio::test(flavor = "multi_thread")]
    async fn confirm_twice_with_identical_tramo_is_a_noop() {
        let db_path = temp_db_path("confirm-noop");
        seed_eurusd_instrument(&db_path).await;
        let first = confirm_arbitrary(db_path.clone(), ARBITRARY_CSV.to_vec()).await;
        assert!(first.ok);
        assert!(!first.was_noop_reimport);

        let second = confirm_arbitrary(db_path.clone(), ARBITRARY_CSV.to_vec()).await;
        assert!(second.ok);
        assert!(second.was_noop_reimport, "el segundo confirm del mismo tramo debe ser no-op");
        assert_eq!(second.declaration_id, first.declaration_id);

        assert_eq!(count_sealed_declarations(&db_path).await, 1, "no debe existir una segunda fila");
    }

    /// Criterio de aceptación #5 (DEBT-039) sobre la vía manual, ejercitado
    /// por el binario real del Bridge: un rango declarado sin relación con
    /// el contenido real se rechaza, y NADA queda sellado.
    #[tokio::test(flavor = "multi_thread")]
    async fn confirm_rejects_declared_range_that_does_not_cover_content() {
        let db_path = temp_db_path("confirm-range-mismatch");
        use chrono::{TimeZone, Utc};
        let bogus_start = Utc.with_ymd_and_hms(2019, 1, 1, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();
        let bogus_end = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();

        let result = confirm_manual_bar_import(
            db_path.clone(),
            "broker_generico_export.csv".to_string(),
            ARBITRARY_CSV.to_vec(),
            "EURUSD".to_string(),
            "GENERIC_BROKER".to_string(),
            "MANUAL_UPLOAD".to_string(),
            bogus_start,
            bogus_end,
            Some("UTC".to_string()),
            0,
            Some("CONTRACTS".to_string()),
            Some("POINTS_GENERIC".to_string()),
            None,
            "5M".to_string(),
            300_000_000_000,
            arbitrary_column_mappings(),
            None,
            None,
        )
        .await;

        assert!(!result.ok, "un rango sin relación con el contenido real debe rechazarse");
        assert!(result.error.is_some());
        assert_eq!(count_sealed_declarations(&db_path).await, 0, "nada debe sellarse cuando el rango se rechaza");
    }

    /// Las CINCO operaciones, de punta a punta, contra la MISMA base de
    /// datos real -- exactamente el flujo que el Inspector Panel recorre:
    /// seleccionar, ver columnas, previsualizar (repetidamente, barato),
    /// analizar el rango real (una vez, caro), confirmar. Prueba de
    /// integración por el binario real del criterio de aceptación #7.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_five_operations_compose_end_to_end_against_a_real_database() {
        let db_path = temp_db_path("five-ops-e2e");
        seed_eurusd_instrument(&db_path).await;

        // 1. Seleccionar archivo.
        let selected = select_manual_import_file(ARBITRARY_CSV.to_vec()).await;
        assert!(!selected.source_file_hash.is_empty());

        // 2. Obtener columnas detectadas.
        let columns = get_manual_import_columns(ARBITRARY_CSV.to_vec()).await;
        assert!(columns.error.is_none());
        assert_eq!(columns.columns.len(), 8);

        // 3. Pedir vista previa -- dos veces, para reforzar que repetir no
        // tiene efecto -- y confirmar que el ledger sigue vacío.
        let preview_1 = preview_arbitrary(ARBITRARY_CSV.to_vec()).await;
        let preview_2 = preview_arbitrary(ARBITRARY_CSV.to_vec()).await;
        assert!(preview_1.error.is_none() && preview_2.error.is_none());
        assert_eq!(preview_1.rows.len(), preview_2.rows.len());
        assert_eq!(count_sealed_declarations(&db_path).await, 0, "antes de confirmar, cero filas selladas");

        // 4. Analizar el rango real del archivo completo -- una sola vez, al
        // llegar al paso de confirmación -- y confirmar que tampoco escribe.
        let analysis = analyze_arbitrary(ARBITRARY_CSV.to_vec()).await;
        assert!(analysis.error.is_none(), "no debe fallar: {:?}", analysis.error);
        assert_eq!(analysis.rows_parsed_total, 2);
        assert_eq!(count_sealed_declarations(&db_path).await, 0, "analizar el rango tampoco debe sellar nada");

        // 5. Confirmar -- ahora SÍ escribe.
        let confirmed = confirm_arbitrary(db_path.clone(), ARBITRARY_CSV.to_vec()).await;
        assert!(confirmed.ok, "no debe fallar: {:?}", confirmed.error);
        assert_eq!(confirmed.mapping_origin, "MANUAL_USER");
        assert_eq!(count_sealed_declarations(&db_path).await, 1, "después de confirmar, exactamente una fila sellada");
    }

    /// Cobertura directa de los dos brazos nuevos del `match` de
    /// `parse_canonical_bar_field` (DEBT-041) -- complementa las pruebas de
    /// punta a punta de arriba con la verificación más acotada posible: el
    /// `match` mismo resuelve al enum correcto, no solo "no falla".
    #[test]
    fn parse_canonical_bar_field_accepts_the_two_split_timestamp_variants() {
        assert_eq!(parse_canonical_bar_field("TIMESTAMP_DATE"), Ok(CanonicalBarField::TimestampDate));
        assert_eq!(parse_canonical_bar_field("TIMESTAMP_TIME"), Ok(CanonicalBarField::TimestampTime));
    }

    /// Mata los dos supervivientes de mutación de `describe_confirm_error`
    /// (`String::new()`, `"xyzzy".into()`): afirma el mensaje EXACTO (no solo
    /// "no vacío") para dos variantes DISTINTAS de `ManualImportBarFileError`,
    /// construidas por separado -- nunca `f(x)` comparado consigo mismo. El
    /// mensaje que el usuario lee cuando su importación se rechaza debe
    /// describir la causa real, no ser basura ni estar vacío: misma familia
    /// que el superviviente de `label()` ya cerrado en el Núcleo.
    #[test]
    fn describe_confirm_error_returns_the_real_display_text_and_varies_by_cause() {
        use data_import_wizard::public_interface::{ManualImportCoreError, ManualMappingError};

        let sink_error = ManualImportBarFileError::Sink("disco lleno (simulado)".to_string());
        let sink_message = describe_confirm_error(&sink_error);
        assert_eq!(
            sink_message,
            "la persistencia de datos aguas abajo falló DESPUÉS de sellar el ledger: disco lleno (simulado)"
        );

        let mapping_error =
            ManualImportBarFileError::Core(ManualImportCoreError::Mapping(ManualMappingError::MissingTimestampDeclaration));
        let mapping_message = describe_confirm_error(&mapping_error);
        assert!(
            mapping_message.contains("no se declaró ninguna marca de tiempo"),
            "mensaje real esperado, obtenido: {mapping_message}"
        );

        // Discriminación: DOS variantes DISTINTAS producen DOS mensajes
        // DISTINTOS -- ni `String::new()` ni `"xyzzy"` podrían satisfacer las
        // dos aserciones anteriores a la vez.
        assert_ne!(sink_message, mapping_message);
    }
}
