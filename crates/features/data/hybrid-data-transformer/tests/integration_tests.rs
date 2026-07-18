//! Tests de integración del Hybrid Data Transformer.
//!
//! Cubren los criterios 4 (determinismo, a nivel de orquestación completa),
//! 9 (ledger atómico), 10 (verificación end-to-end) y el uso de la muestra
//! real congelada de cripto (`tests/fixtures/binance_btcusdt_sample.csv`).
//! Los criterios 1-3, 5-8 se prueban a nivel unitario puro en `src/domain.rs`
//! (no requieren I/O).
//!
//! Ningún test toca la red: la BD usa archivos temporales reales en disco
//! (nunca `:memory:`, para probar durabilidad) y los ticks vienen de la
//! muestra congelada o de datos fabricados en el propio test.

use std::path::Path;

use polars::prelude::*;
use tempfile::TempDir;

use shared::public_interface::{create_pool, run_migrations, DeterministicClock};
use hybrid_data_transformer::public_interface::{
    execute, load_ticks_from_parquet, transform, verify, AggressorSideInput, ExecuteRequest, NewTransformationRecord,
    TradeTickInput, TransformError, TransformRequest, TransformerConfig, TransformationRepository,
    TransformationRepositoryError, VerifyInput,
};

// ── Helpers de test ──────────────────────────────────────────────────────────

/// Crea una base de datos SQLite en un archivo temporal y aplica todas las
/// migraciones. Devuelve el pool y el directorio temporal (que se borra al
/// salir del scope).
async fn setup_db() -> (sqlx::SqlitePool, TempDir) {
    let dir = tempfile::tempdir().expect("crear directorio temporal");
    let db_path = dir.path().join("test.sqlite");
    let url = format!("sqlite://{}", db_path.display());
    let pool = create_pool(&url).await.expect("abrir base de datos temporal");
    run_migrations(&pool).await.expect("aplicar migraciones");
    (pool, dir)
}

/// Parsea la muestra congelada de cripto (`binance_btcusdt_sample.csv`) --
/// formato simple `event_timestamp_ns,price_scaled,trade_volume_scaled,origin_seq`,
/// líneas que empiezan con `#` son comentarios. Sin dependencia de un crate
/// CSV: el formato es trivial y esto evita una dependencia nueva solo para
/// un fixture de 20 filas.
fn load_frozen_binance_sample() -> Vec<TradeTickInput> {
    let raw = include_str!("fixtures/binance_btcusdt_sample.csv");
    let mut ticks = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("event_timestamp_ns") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(fields.len(), 4, "fila mal formada en el fixture congelado: {line}");
        ticks.push(TradeTickInput {
            event_timestamp_ns: fields[0].parse().expect("event_timestamp_ns debe ser i64"),
            price_scaled: fields[1].parse().expect("price_scaled debe ser i64"),
            trade_volume_scaled: fields[2].parse().expect("trade_volume_scaled debe ser i64"),
            // El fixture congelado (4 columnas, sin lado comprador/vendedor)
            // no distingue aggressor_side -- se propaga honestamente como
            // None, no se inventa (ADR-0155 restricción 4).
            aggressor_side: None,
            origin_seq: fields[3].parse().expect("origin_seq debe ser u64"),
        });
    }
    ticks
}

fn sample_request(ticks: Vec<TradeTickInput>, target_interval_ns: i64) -> TransformRequest {
    TransformRequest {
        data_snapshot_id: Some("binance_BTCUSDT_1m_202401".to_string()),
        ticks,
        target_interval_ns,
        parent_id: None,
        node_id: Some("node-test".to_string()),
        process_id: Some("process-test".to_string()),
        session_id: Some("session-test".to_string()),
    }
}

// ── Muestra real congelada de cripto ─────────────────────────────────────────

/// La muestra congelada tiene 20 trades reales de Binance en una ventana de
/// menos de 1 segundo, con varios timestamps de milisegundo repetidos
/// (Binance no reporta sub-milisegundo). Con un intervalo de agregación de
/// 1 hora, TODOS deben caer en la misma barra -- y el conteo de entrada
/// (20 ticks) debe reflejarse exactamente en el volumen agregado, sin que
/// ningún trade se pierda por los timestamps repetidos (criterio 8, ahora
/// contra datos reales en vez de fabricados).
#[tokio::test(flavor = "multi_thread")]
async fn frozen_binance_sample_aggregates_without_losing_any_real_trade() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);
    let ticks = load_frozen_binance_sample();
    assert_eq!(ticks.len(), 20, "precondición: el fixture congelado tiene 20 trades reales");

    let config = TransformerConfig::default();
    let request = sample_request(ticks.clone(), 3_600_000_000_000); // 1h

    let result = transform(&config, request, &pool, &clock)
        .await
        .expect("la transformación sobre datos reales debe tener éxito");

    assert_eq!(result.rows_in, 20);
    assert_eq!(result.rows_out, 1, "toda la ventana de <1s cae en una sola barra de 1h");

    // El volumen agregado debe ser la suma EXACTA de los 20 volúmenes reales
    // -- verifica que ningún trade se descartó por compartir timestamp.
    let expected_volume: i64 = ticks.iter().map(|t| t.trade_volume_scaled).sum();
    assert_eq!(result.bars[0].volume_traded, expected_volume, "ningún trade real debe perderse por timestamp repetido");
    assert_eq!(result.bars[0].trade_count, 20, "trade_count debe reflejar las 20 ejecuciones reales, no colapsarse con volume_traded");

    // El primer/último precio de la barra deben coincidir con el primer/
    // último trade por orden canónico (timestamp, luego aggTradeId real).
    let mut sorted = ticks.clone();
    sorted.sort_by_key(|t| (t.event_timestamp_ns, t.origin_seq));
    assert_eq!(result.bars[0].open, sorted.first().unwrap().price_scaled);
    assert_eq!(result.bars[0].close, sorted.last().unwrap().price_scaled);
}

// ── Criterio 4 (a nivel de orquestación): determinismo bajo distinto polars_threads ──

/// Dos corridas del mismo plan sobre el mismo dataset real, con distinto
/// número de hilos de Polars, deben producir el mismo primer y último precio
/// por barra, el mismo `logic_hash` y el mismo `audit_hash`.
#[tokio::test(flavor = "multi_thread")]
async fn same_dataset_different_thread_count_produces_identical_bars_and_hashes() {
    let ticks = load_frozen_binance_sample();

    let (pool_a, _dir_a) = setup_db().await;
    let clock_a = DeterministicClock::new(1_000_000, 1_000);
    let config_a = TransformerConfig { interop_threshold: 100_000, polars_threads: Some(1) };
    let result_a = transform(&config_a, sample_request(ticks.clone(), 3_600_000_000_000), &pool_a, &clock_a)
        .await
        .expect("corrida A debe tener éxito");

    let (pool_b, _dir_b) = setup_db().await;
    let clock_b = DeterministicClock::new(1_000_000, 1_000);
    let config_b = TransformerConfig { interop_threshold: 100_000, polars_threads: Some(4) };
    let result_b = transform(&config_b, sample_request(ticks, 3_600_000_000_000), &pool_b, &clock_b)
        .await
        .expect("corrida B debe tener éxito");

    assert_eq!(result_a.logic_hash, result_b.logic_hash, "el logic_hash no depende del número de hilos");
    assert_eq!(result_a.audit_hash, result_b.audit_hash, "el audit_hash no depende del número de hilos");
    assert_eq!(result_a.bars[0].open, result_b.bars[0].open, "el primer precio debe coincidir sin importar el N de hilos");
    assert_eq!(result_a.bars[0].close, result_b.bars[0].close, "el último precio debe coincidir sin importar el N de hilos");
}

// ── Criterio 7 (a nivel de orquestación): matrix_out honesto ────────────────

/// Con un `INTEROP_THRESHOLD` alto, el bloque de barras se materializa como
/// `ndarray::Array2<f64>` real vía el puente Zero-Copy de Polars.
#[tokio::test(flavor = "multi_thread")]
async fn matrix_out_materializes_when_within_threshold() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);
    let ticks = load_frozen_binance_sample();

    let config = TransformerConfig { interop_threshold: 100_000, polars_threads: None };
    let result = transform(&config, sample_request(ticks, 3_600_000_000_000), &pool, &clock)
        .await
        .expect("debe tener éxito");

    let matrix = result.matrix_out.expect("con umbral alto, matrix_out debe materializarse");
    assert_eq!(matrix.shape(), &[1, 5], "1 barra x 5 columnas (open,high,low,close,volume)");
}

/// Con un `INTEROP_THRESHOLD` fijado deliberadamente en 0, la conversión se
/// rechaza -- `matrix_out` debe ser `None`, sin importar que los valores
/// individuales sean exactos.
#[tokio::test(flavor = "multi_thread")]
async fn matrix_out_is_none_when_threshold_is_exceeded() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);
    let ticks = load_frozen_binance_sample();

    let config = TransformerConfig { interop_threshold: 0, polars_threads: None };
    let result = transform(&config, sample_request(ticks, 3_600_000_000_000), &pool, &clock)
        .await
        .expect("la transformación en sí debe tener éxito; solo se rechaza la matriz");

    assert!(result.matrix_out.is_none(), "1 fila > umbral de 0 -> matrix_out debe rechazarse");
}

// ── Lectura desde Parquet (ADR-0105: scan_parquet) ───────────────────────────

/// Round-trip: escribe un Parquet real con el esquema canónico de
/// ejecuciones (`TradeTick`, usando el propio motor Polars) y verifica que
/// `load_ticks_from_parquet` lo lee de vuelta sin pérdida -- ejercita
/// `LazyFrame::scan_parquet` (restricción FIJA de la feature: "Entrada:
/// Archivos Parquet") contra un archivo real en disco, no contra la red.
/// Este archivo NO declara la columna `aggressor_side` -- ejercita también
/// el camino "columna ausente a nivel de archivo -> None para todas las
/// filas" (ADR-0155 restricción 4).
#[tokio::test(flavor = "multi_thread")]
async fn load_ticks_from_parquet_round_trips_real_file() {
    let dir = tempfile::tempdir().expect("crear directorio temporal");
    let path = dir.path().join("ticks.parquet");

    let mut df = df![
        "event_timestamp_ns" => [1_i64, 2, 3],
        "price_scaled" => [100_i64, 200, 300],
        "trade_volume_scaled" => [10_i64, 20, 30],
        "origin_seq" => [0_u64, 1, 2],
    ]
    .expect("construir DataFrame de prueba");

    let file = std::fs::File::create(&path).expect("crear archivo Parquet de prueba");
    ParquetWriter::new(file).finish(&mut df).expect("escribir Parquet de prueba");

    let ticks = load_ticks_from_parquet(Path::new(&path)).await.expect("leer Parquet de prueba");

    assert_eq!(ticks.len(), 3);
    assert_eq!(ticks[0].event_timestamp_ns, 1);
    assert_eq!(ticks[0].price_scaled, 100);
    assert_eq!(ticks[2].origin_seq, 2);
    assert!(ticks[0].aggressor_side.is_none(), "columna ausente en el archivo -> aggressor_side None, nunca inventado");
}

/// `execute()` con `source_parquet_path` (sin `ticks` explícitos) debe
/// resolver los ticks desde el archivo Parquet real.
#[tokio::test(flavor = "multi_thread")]
async fn execute_resolves_ticks_from_parquet_path_when_ticks_empty() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);

    let parquet_dir = tempfile::tempdir().expect("crear directorio temporal");
    let path = parquet_dir.path().join("ticks.parquet");
    let mut df = df![
        "event_timestamp_ns" => [0_i64, 10],
        "price_scaled" => [100_i64, 110],
        "trade_volume_scaled" => [1_i64, 1],
        "origin_seq" => [0_u64, 1],
    ]
    .unwrap();
    let file = std::fs::File::create(&path).unwrap();
    ParquetWriter::new(file).finish(&mut df).unwrap();

    let request = ExecuteRequest {
        data_snapshot_id: None,
        ticks: vec![],
        source_parquet_path: Some(path),
        target_interval_ns: 1_000,
        interop_threshold: 100_000,
        polars_threads: None,
        parent_id: None,
        node_id: None,
        process_id: None,
        session_id: None,
    };

    let result = execute(request, &pool, &clock).await.expect("execute debe resolver desde Parquet");
    assert_eq!(result.rows_in, 2);
}

// ── Criterio 10: verificación end-to-end (ADR-0142) ─────────────────────────

/// `verify()` con adaptadores reales (BD temporal + reloj real + Polars
/// real) debe devolver un `VerifyOutput` con `ok: true` y todas las piezas
/// del observable declarado en la feature: esquema, filas in/out, hash de
/// lógica, duración.
#[tokio::test(flavor = "multi_thread")]
async fn verify_returns_ok_with_full_observable() {
    let ticks = load_frozen_binance_sample();
    let input = VerifyInput { target_aggregation: "1h".to_string(), ticks: Some(ticks), ..Default::default() };

    let output = verify(input).await;

    assert!(output.ok, "verify() debe completarse sin error: {:?}", output.error);
    assert_eq!(output.rows_in, Some(20));
    assert_eq!(output.rows_out, Some(1));
    assert!(output.logic_hash.is_some());
    assert!(output.audit_hash.is_some());
    assert!(output.transform_duration_ns.is_some());
    let schema = output.schema.as_ref().expect("el esquema resultante debe estar presente");
    assert!(schema.contains(&"open:Int64".to_string()));
    assert!(schema.contains(&"volume_traded:Int64".to_string()), "el esquema retrofit ADR-0155 expone volume_traded, no 'volume'");
    assert!(schema.contains(&"trade_count:Int64".to_string()));
    assert!(schema.contains(&"volume_buy:Int64".to_string()));
    assert!(schema.contains(&"volume_ticks:Int64".to_string()));

    // Forma del JSON -- serializable, tal como lo consumiría el CLI/SVF.
    let json_str = serde_json::to_string(&output).expect("VerifyOutput debe serializar a JSON");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("el JSON producido debe ser válido");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["rows_out"].is_number());
}

/// `verify()` sin `--input` (valores por defecto) también debe completarse
/// -- usa la muestra sintética embebida, no la congelada de cripto.
#[tokio::test(flavor = "multi_thread")]
async fn verify_without_explicit_input_uses_smoke_sample_and_succeeds() {
    let output = verify(VerifyInput::default()).await;
    assert!(output.ok, "verify() sin --input debe usar la muestra sintética por defecto: {:?}", output.error);
    assert!(output.rows_in.unwrap_or(0) > 0);
}

// ── Criterio 9: ledger atómico (16 escritores concurrentes) ─────────────────

fn sample_new_transformation_record(schema_version: &str) -> NewTransformationRecord {
    NewTransformationRecord {
        data_snapshot_id: None,
        transformation_id: "OHLCV_AGGREGATION".to_string(),
        logic_hash: "test-logic-hash".to_string(),
        parent_id: None,
        node_id: Some("node-test".to_string()),
        process_id: Some("process-test".to_string()),
        session_id: None,
        transform_duration_ns: 1_000,
        rows_in: 10,
        rows_out: 1,
        schema_version: schema_version.to_string(),
        content_audit_hash: format!("content-hash-{schema_version}"),
    }
}

/// El registro de transformaciones sobrevive a 16 escritores concurrentes
/// reales (no forzados) sin perder ni duplicar `event_sequence_id`.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn transformation_ledger_survives_16_concurrent_writers_without_loss_or_duplication() {
    let dir = tempfile::tempdir().expect("crear directorio temporal");
    let db_path = dir.path().join("concurrent.sqlite");
    let url = format!("sqlite://{}", db_path.display());
    let pool = create_pool(&url).await.expect("crear BD");
    run_migrations(&pool).await.expect("migrar");

    const WRITERS: usize = 16;
    let mut handles = Vec::with_capacity(WRITERS);
    for i in 0..WRITERS {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            let clock = DeterministicClock::new(1_000 + i as i64, 10);
            let repo = TransformationRepository::new(&pool, &clock);
            repo.record(sample_new_transformation_record(&format!("writer-{i}"))).await
        }));
    }

    let mut sequences = Vec::with_capacity(WRITERS);
    for handle in handles {
        let record = handle.await.expect("la tarea no debe entrar en pánico").expect("el registro no debe perderse");
        sequences.push(record.event_sequence_id);
    }

    sequences.sort_unstable();
    let expected: Vec<i64> = (1..=WRITERS as i64).collect();
    assert_eq!(sequences, expected, "los 16 escritores deben producir la secuencia 1..=16 sin huecos ni duplicados");

    // Verifica también contra la tabla real: exactamente 16 filas.
    let verify_clock = DeterministicClock::new(0, 1);
    let repo = TransformationRepository::new(&pool, &verify_clock);
    let all = repo.list_all().await.expect("listar registros");
    assert_eq!(all.len(), WRITERS, "deben persistir exactamente 16 filas, ninguna perdida ni duplicada");
}

/// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
/// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
/// suelta), el bucle de reintento de `TransformationRepository::record` debe
/// agotar EXACTAMENTE el máximo de intentos y rendirse con
/// `WriteContention { attempts: MAX }` -- nunca descartar el registro de
/// transformación en silencio. Mismo patrón que
/// `sovereign_data_fetcher::persistence::DownloadRepository` (STORY-046 M1).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transformation_record_exhausts_exactly_max_attempts_when_write_lock_is_held() {
    use std::str::FromStr;
    use std::time::Duration;

    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

    const MAX_RECORD_ATTEMPTS: u32 = 5; // mismo valor que persistence.rs (privado al crate)

    let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
    let db_path = temp_dir.path().join("transformation_forced_contention.sqlite");
    let database_url = format!("sqlite://{}", db_path.display());

    let setup_pool = create_pool(&database_url).await.expect("conectar (setup)");
    run_migrations(&setup_pool).await.expect("migrar");
    setup_pool.close().await;

    let immediate_opts = || {
        SqliteConnectOptions::from_str(&database_url)
            .expect("parsear opciones")
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_millis(0))
    };

    let lock_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(immediate_opts())
        .await
        .expect("pool que retiene el lock");
    let lock_tx = lock_pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .expect("tomar el lock de escritura reservado");

    let repo_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(immediate_opts())
        .await
        .expect("pool del repositorio");
    let clock = DeterministicClock::new(1_000, 100);
    let repo = TransformationRepository::new(&repo_pool, &clock);

    let result = repo.record(sample_new_transformation_record("contention-test")).await;

    drop(lock_tx);

    match result {
        Err(TransformationRepositoryError::WriteContention { attempts }) => {
            assert_eq!(attempts, MAX_RECORD_ATTEMPTS, "bajo contención sostenida debe agotar EXACTAMENTE el máximo de intentos");
        }
        other => panic!("se esperaba WriteContention {{ attempts: {MAX_RECORD_ATTEMPTS} }} bajo contención sostenida, se obtuvo: {other:?}"),
    }
}

/// CRITERIO DE CIERRE (QA por mutación): una violación UNIQUE PERMANENTE (la
/// PK `id`, que NO se debe reintentar) NO debe clasificarse como contención
/// transitoria.
#[tokio::test(flavor = "multi_thread")]
async fn duplicate_primary_key_insert_is_a_permanent_error_not_a_sequence_collision() {
    let (pool, _dir) = setup_db().await;

    let insert_with_id_dup = |event_sequence_id: i64| {
        sqlx::query(
            "INSERT INTO transformation_records \
             (id, created_at, updated_at, audit_hash, event_sequence_id, transformation_id, logic_hash, transform_duration_ns, rows_in, rows_out, schema_version) \
             VALUES ('dup-transformation-id', 1, 1, 'h', ?, 'OHLCV_AGGREGATION', 'lh', 0, 0, 0, '1.0')",
        )
        .bind(event_sequence_id)
        .execute(&pool)
    };

    insert_with_id_dup(1).await.expect("primera fila válida");
    let err = insert_with_id_dup(2).await.expect_err("la segunda fila debe violar la PRIMARY KEY id");

    let message = err.to_string().to_lowercase();
    assert!(
        !message.contains("database is locked") && !message.contains("event_sequence_id"),
        "una violación de PK debe distinguirse textualmente de una colisión de secuencia o un lock transitorio; mensaje: {message}"
    );
}

/// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE
/// `TransformationRepository::record` refleja SIEMPRE valores frescos --
/// `audit_hash` (el `content_audit_hash` recién pasado), `audit_chain_hash`
/// encadenado a la fila anterior y `created_at`/`updated_at` leídos del
/// reloj EN ESE INSTANTE -- nunca datos por defecto ni reutilizados de una
/// llamada previa.
#[tokio::test(flavor = "multi_thread")]
async fn transformation_record_returned_row_reflects_fresh_hash_chain_and_timestamp() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000, 100);
    let repo = TransformationRepository::new(&pool, &clock);

    let first = repo.record(sample_new_transformation_record("v1")).await.expect("registrar primer registro");
    assert_eq!(first.created_at, 1_000, "precondición: el primer registro nace en el now inicial");
    assert!(first.audit_chain_hash.is_none(), "precondición: el primer registro no encadena");
    assert_eq!(first.event_sequence_id, 1);
    assert_eq!(first.audit_hash, "content-hash-v1");

    clock.tick(); // 1_000 -> 1_100
    let second = repo.record(sample_new_transformation_record("v2")).await.expect("registrar segundo registro");

    assert_eq!(second.audit_hash, "content-hash-v2", "el audit_hash devuelto debe ser el content_audit_hash recién pasado, no reutilizado");
    assert_eq!(
        second.audit_chain_hash,
        Some(first.audit_hash.clone()),
        "el audit_chain_hash devuelto debe encadenar al audit_hash del registro anterior"
    );
    assert_eq!(second.created_at, 1_100, "el created_at devuelto debe ser el now del reloj tras el tick, no el viejo");
    assert_eq!(second.event_sequence_id, 2);
}

/// El registro persistido en `transformation_records` trae los campos del
/// Perfil A (ADR-0020) completos -- verificación de humo del cableado
/// completo (`transform()` -> `TransformationRepository`).
#[tokio::test(flavor = "multi_thread")]
async fn transformation_record_persisted_with_profile_a_fields() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);
    let ticks = vec![TradeTickInput { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 1, aggressor_side: None, origin_seq: 0 }];

    let config = TransformerConfig::default();
    let result = transform(&config, sample_request(ticks, 1_000), &pool, &clock)
        .await
        .expect("la transformación debe completarse");

    let repo = TransformationRepository::new(&pool, &clock);
    let record = repo.find(&result.record_id).await.expect("consulta de la base de datos").expect("el registro debe existir");

    assert!(!record.id.is_empty());
    assert!(record.created_at > 0);
    assert_eq!(record.transformation_id, "OHLCV_AGGREGATION");
    assert_eq!(record.logic_hash, result.logic_hash);
    assert_eq!(record.rows_in, 1);
    assert_eq!(record.rows_out, 1);
    assert_eq!(record.schema_version, "2.1", "corrección DEBT-028: el retiro de vwap cambió la forma de Bar de forma incompatible");
    assert_eq!(record.data_snapshot_id.as_deref(), Some("binance_BTCUSDT_1m_202401"));
}

// ── Invariante 5 (frontera de coma flotante) a nivel de orquestación ────────

/// Un dataset cuyo volumen agregado supera 2^53 debe seguir completando la
/// transformación en sí (las barras se persisten igual, en enteros exactos)
/// pero el `matrix_out` debe rechazarse -- el error de dominio NUNCA debe
/// abortar toda la transformación por un problema exclusivo de la frontera
/// f64.
#[tokio::test(flavor = "multi_thread")]
async fn transformation_succeeds_even_when_matrix_conversion_is_rejected_for_precision() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);

    // Un volumen por encima de 2^53 (9_007_199_254_740_992).
    let ticks = vec![TradeTickInput {
        event_timestamp_ns: 0,
        price_scaled: 100,
        trade_volume_scaled: 9_007_199_254_740_993,
        aggressor_side: None,
        origin_seq: 0,
    }];

    let config = TransformerConfig::default();
    let result = transform(&config, sample_request(ticks, 1_000), &pool, &clock)
        .await
        .expect("la transformación debe completarse aunque la matriz se rechace");

    assert_eq!(result.bars[0].volume_traded, 9_007_199_254_740_993, "el valor entero exacto se persiste igual, sin pasar por f64");
    assert!(result.matrix_out.is_none(), "matrix_out debe rechazarse por exceder 2^53");
}

/// Un producto de precio x volumen que desborda incluso el intervalo
/// razonable produce `TransformError::Domain`, nunca un pánico ni un valor
/// silenciosamente incorrecto -- verificación a nivel de orquestación de
/// que el error del Núcleo se propaga intacto a través de la Cáscara.
#[tokio::test(flavor = "multi_thread")]
async fn domain_error_propagates_through_orchestrator_without_panicking() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);

    let ticks = vec![TradeTickInput { event_timestamp_ns: 0, price_scaled: 1, trade_volume_scaled: 1, aggressor_side: None, origin_seq: 0 }];
    let config = TransformerConfig::default();
    // Intervalo no positivo -- el Núcleo debe rechazarlo explícitamente.
    let result = transform(&config, sample_request(ticks, 0), &pool, &clock).await;

    assert!(matches!(result, Err(TransformError::Domain(_))), "un intervalo no positivo debe producir TransformError::Domain");
}

// ── Retrofit ADR-0155: pruebas discriminantes exigidas por la Orden ─────────

/// PRUEBA DISCRIMINANTE (exigida por el retrofit): un JSON con forma de
/// cotización (`bid`/`ask`, sin `price_scaled`/`trade_volume_scaled`) NUNCA
/// logra convertirse en un `TradeTickInput` -- la deserialización falla por
/// campos obligatorios ausentes. Como `transform()`/`verify()` solo aceptan
/// `Vec<TradeTickInput>` ya deserializado, esto demuestra en el borde real
/// del sistema (el mismo que usa el CLI/SVF) que una `QuoteTick` no tiene
/// ningún camino para llegar a `aggregate_trades_to_bars` y contribuir a una
/// barra agregada (ADR-0155 restricción 2).
#[test]
fn quote_shaped_json_never_deserializes_into_a_trade_tick_and_cannot_reach_aggregation() {
    // Forma real de una QuoteTick (ADR-0155): bid/ask obligatorios, sin
    // price/trade_volume -- exactamente los campos que TradeTickInput exige.
    let quote_shaped_json = r#"{
        "event_timestamp_ns": 1700000000000000000,
        "bid": 4300000000000,
        "ask": 4300100000000,
        "origin_seq": 0
    }"#;

    let result: Result<TradeTickInput, _> = serde_json::from_str(quote_shaped_json);
    assert!(
        result.is_err(),
        "un JSON con forma de QuoteTick (bid/ask) debe FALLAR al deserializar como TradeTickInput -- \
         si esto pasara, una cotización podría colarse en la agregación de barras"
    );
}

/// PRUEBA DISCRIMINANTE (exigida por el retrofit), a nivel de orquestación
/// completa: cuando la fuente de ejecuciones distingue el lado comprador
/// para TODAS las operaciones del intervalo, `volume_buy` debe reflejar
/// exactamente la porción comprada -- una magnitud numéricamente DISTINTA de
/// `volume_traded` (que suma todo el lado). Demuestra que las cuatro
/// magnitudes de volumen no se colapsan en una sola columna al atravesar el
/// pipeline completo (Polars -> Núcleo -> persistencia -> TransformResult),
/// no solo en el Núcleo puro.
#[tokio::test(flavor = "multi_thread")]
async fn four_volume_magnitudes_remain_independent_through_the_full_pipeline() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);

    let ticks = vec![
        TradeTickInput { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 7, aggressor_side: Some(AggressorSideInput::Buy), origin_seq: 0 },
        TradeTickInput { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 3, aggressor_side: Some(AggressorSideInput::Sell), origin_seq: 1 },
    ];

    let config = TransformerConfig::default();
    let result = transform(&config, sample_request(ticks, 1_000), &pool, &clock)
        .await
        .expect("la transformación debe completarse");

    let bar = &result.bars[0];
    assert_eq!(bar.volume_traded, 10, "volume_traded suma TODO (7+3)");
    assert_eq!(bar.trade_count, 2, "trade_count es un conteo de ejecuciones, no un volumen");
    assert_eq!(bar.volume_buy, Some(7), "volume_buy refleja SOLO el lado comprador");
    assert_eq!(bar.volume_ticks, None, "volume_ticks nunca se fabrica desde TradeTick agregado en esta feature");
    assert_ne!(bar.volume_buy.unwrap(), bar.volume_traded, "las magnitudes no deben colapsar entre sí");

    // Con una fuente que NO distingue el lado en absoluto, volume_buy debe
    // quedar None -- nunca 0, nunca igual a volume_traded.
    let (pool2, _dir2) = setup_db().await;
    let clock2 = DeterministicClock::new(1_000_000, 1_000);
    let ticks_no_side = vec![
        TradeTickInput { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 7, aggressor_side: None, origin_seq: 0 },
        TradeTickInput { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 3, aggressor_side: None, origin_seq: 1 },
    ];
    let result_no_side = transform(&config, sample_request(ticks_no_side, 1_000), &pool2, &clock2)
        .await
        .expect("la transformación debe completarse sin lado conocido");
    assert_eq!(result_no_side.bars[0].volume_buy, None, "sin fuente que distinga el lado, volume_buy NUNCA se rellena por defecto");
    assert_eq!(result_no_side.bars[0].volume_traded, 10, "volume_traded se sigue calculando igual -- es independiente de volume_buy");
}
