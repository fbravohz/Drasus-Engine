//! [SHELL] Buffer de alta velocidad para telemetría
//! (`docs/features/telemetry.md` TTR-001, ADR-0015).
//!
//! ## Por qué `record_latency`/`record_heartbeat` son funciones síncronas
//!
//! El requisito "ALTA EFICIENCIA" (telemetry.md "Restricciones": registrar
//! una muestra debe tardar menos de 50µs) se cumple porque encolar NO hace
//! `await` a nada: ni al disco, ni a un lock async, ni a otra tarea. Toma
//! brevemente un `std::sync::Mutex` (la cadena de hashes en memoria) y
//! empuja a un canal `mpsc` no acotado, cuyo `send` nunca bloquea al que
//! llama — solo falla si ya no queda ningún receptor vivo. El vaciado a
//! disco ocurre en [`TelemetryBuffer::spawn_flush_task`], una tarea de
//! fondo separada que el llamador nunca espera.
//!
//! ## Por qué la cadena de hashes vive en memoria, no en SQLite
//!
//! [`crate::domain::telemetry::build_sample`] necesita la muestra anterior
//! para encadenar `audit_hash`/`event_sequence_id`. Leerla de SQLite en
//! cada llamada (como hace `AuditLogRepository::append` para el Audit Log)
//! violaría el límite de 50µs. En su lugar, [`TelemetryBuffer::bootstrap`]
//! lee la cola UNA SOLA VEZ al iniciar el proceso y la guarda en memoria;
//! cada `record_*` posterior solo lee/actualiza esa copia en memoria.

use std::sync::{Arc, Mutex as StdMutex};

use tokio::sync::{mpsc, Mutex as TokioMutex};
use uuid::Uuid;

use crate::domain::clock::Clock;
use crate::domain::telemetry::{build_sample, TelemetrySample, TelemetrySampleContent};
use crate::orchestrator::job_executor::ExecutorIdentity;
use crate::persistence::telemetry::{TelemetryError, TelemetryRepository};

/// Configuración del buffer (`docs/features/telemetry.md` "Parámetros
/// Configurables" no declara un intervalo de flush explícito — es un
/// detalle de implementación de esta cáscara, no un parámetro de la
/// Feature).
#[derive(Debug, Clone)]
pub struct TelemetryBufferConfig {
    /// Cada cuánto la tarea de fondo intenta vaciar la cola a SQLite.
    pub flush_interval_ms: u64,
}

impl Default for TelemetryBufferConfig {
    fn default() -> Self {
        Self { flush_interval_ms: 100 }
    }
}

/// Estado compartido detrás del handle barato de clonar [`TelemetryBuffer`].
struct Shared {
    pool: sqlx::SqlitePool,
    clock: Arc<dyn Clock>,
    identity: ExecutorIdentity,
    config: TelemetryBufferConfig,
    /// Última muestra encadenada conocida por este proceso. `None` antes de
    /// [`TelemetryBuffer::bootstrap`] o si la tabla está vacía — la
    /// siguiente muestra será la génesis de la cadena.
    chain_state: StdMutex<Option<TelemetrySample>>,
}

/// El buffer de alta velocidad de telemetría.
///
/// Barato de clonar (un handle `Arc`): los clones comparten la misma cola
/// en memoria y el mismo estado de cadena.
#[derive(Clone)]
pub struct TelemetryBuffer {
    shared: Arc<Shared>,
    queue_tx: mpsc::UnboundedSender<TelemetrySample>,
    queue_rx: Arc<TokioMutex<Option<mpsc::UnboundedReceiver<TelemetrySample>>>>,
}

impl TelemetryBuffer {
    /// Crea un buffer ligado a `pool` (ya migrado) y `clock`. NO siembra el
    /// estado de cadena desde disco ni arranca la tarea de fondo — llama a
    /// [`Self::bootstrap`] y luego a [`Self::spawn_flush_task`] explícitamente,
    /// en ese orden, antes de registrar la primera muestra real.
    pub fn new(pool: sqlx::SqlitePool, clock: Arc<dyn Clock>, identity: ExecutorIdentity, config: TelemetryBufferConfig) -> Self {
        let (queue_tx, queue_rx) = mpsc::unbounded_channel();

        Self {
            shared: Arc::new(Shared {
                pool,
                clock,
                identity,
                config,
                chain_state: StdMutex::new(None),
            }),
            queue_tx,
            queue_rx: Arc::new(TokioMutex::new(Some(queue_rx))),
        }
    }

    fn repo(&self) -> TelemetryRepository<'_> {
        TelemetryRepository::new(&self.shared.pool)
    }

    /// Siembra el estado de cadena en memoria leyendo la última muestra
    /// persistida (o `None` si la tabla está vacía). DEBE llamarse UNA SOLA
    /// VEZ, después de [`Self::new`] y antes de cualquier `record_*` —
    /// evita que `event_sequence_id` colisione con filas de una corrida
    /// anterior del proceso (mismo motivo que `JobExecutor::recover_at_startup`).
    pub async fn bootstrap(&self) -> Result<(), TelemetryError> {
        let tail = self.repo().load_tail().await?;
        let mut chain_state = self.shared.chain_state.lock().expect("chain_state mutex envenenado");
        *chain_state = tail;
        Ok(())
    }

    /// Registra una muestra de latencia (criterio #1). Síncrona — ver el
    /// comentario del módulo sobre por qué no es `async fn`.
    pub fn record_latency(&self, metric_name: impl Into<String>, execution_latency_ms: i64, details_json: Option<String>) {
        self.enqueue(metric_name.into(), Some(execution_latency_ms), details_json);
    }

    /// Registra una señal de vida (heartbeat, criterio #2): sin valor de
    /// latencia. Síncrona, misma razón que [`Self::record_latency`].
    pub fn record_heartbeat(&self, metric_name: impl Into<String>) {
        self.enqueue(metric_name.into(), None, None);
    }

    /// Construye la siguiente muestra de la cadena y la empuja al canal.
    /// Esta es la operación completa que el límite de 50µs mide.
    fn enqueue(&self, metric_name: String, execution_latency_ms: Option<i64>, details_json: Option<String>) {
        let content = TelemetrySampleContent {
            metric_name,
            details_json,
            institutional_tag: self.shared.identity.institutional_tag.clone(),
            logic_hash: self.shared.identity.logic_hash.clone(),
            session_id: self.shared.identity.session_id.clone(),
            node_id: self.shared.identity.node_id.clone(),
            process_id: self.shared.identity.process_id.clone(),
            execution_latency_ms,
        };

        let id = Uuid::new_v4().to_string();
        let created_at_ns = self.shared.clock.timestamp_ns();

        let mut chain_state = self.shared.chain_state.lock().expect("chain_state mutex envenenado");
        let sample = build_sample(id, created_at_ns, content, chain_state.as_ref());
        *chain_state = Some(sample.clone());
        drop(chain_state); // libera el lock antes de tocar el canal — no es necesario, pero evita tenerlo abierto un instante de más.

        // Un envío a un canal `unbounded` nunca bloquea; solo falla si no
        // queda ningún receptor vivo (la tarea de fondo se cayó o nunca se
        // arrancó) — en ese caso no hay nada útil que hacer con el error.
        let _ = self.queue_tx.send(sample);
    }

    /// **Vaciado por lotes (TTR-001)**: arranca una tarea de fondo que, cada
    /// `flush_interval_ms`, drena lo acumulado en el canal y lo inserta en
    /// SQLite en una sola transacción ([`TelemetryRepository::insert_batch`]).
    /// Devuelve de inmediato; la tarea corre hasta que el último
    /// [`TelemetryBuffer`] (y por tanto el emisor del canal) se destruye.
    ///
    /// Debe llamarse sobre un runtime de Tokio. Llamarla dos veces sobre el
    /// mismo buffer entra en pánico (el receptor del canal solo se puede
    /// tomar una vez).
    pub fn spawn_flush_task(&self) -> tokio::task::JoinHandle<()> {
        let buffer = self.clone();

        tokio::spawn(async move {
            let mut receiver = {
                let mut guard = buffer.queue_rx.lock().await;
                guard
                    .take()
                    .expect("spawn_flush_task: ya se llamó antes sobre este buffer")
            };

            let mut ticker = tokio::time::interval(std::time::Duration::from_millis(buffer.shared.config.flush_interval_ms));

            loop {
                ticker.tick().await;

                let mut batch = Vec::new();
                let mut disconnected = false;
                loop {
                    match receiver.try_recv() {
                        Ok(sample) => batch.push(sample),
                        Err(mpsc::error::TryRecvError::Empty) => break,
                        Err(mpsc::error::TryRecvError::Disconnected) => {
                            disconnected = true;
                            break;
                        }
                    }
                }

                if !batch.is_empty() {
                    let _ = buffer.repo().insert_batch(&batch).await;
                }

                if disconnected {
                    break;
                }
            }
        })
    }

    /// Purga las muestras más viejas que `retention_days` (telemetry.md
    /// "PODA AUTOMÁTICA"). El corte se calcula a partir del [`Clock`]
    /// inyectado — nunca `SystemTime::now()`.
    pub async fn purge(&self, retention_days: u32) -> Result<u64, TelemetryError> {
        const NANOS_PER_DAY: i64 = 86_400 * 1_000_000_000;

        let now_ns = self.shared.clock.timestamp_ns();
        let cutoff_ns = now_ns - i64::from(retention_days) * NANOS_PER_DAY;

        self.repo().purge_older_than(cutoff_ns).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    fn test_identity() -> ExecutorIdentity {
        ExecutorIdentity {
            process_id: "test-process".to_string(),
            session_id: Some("test-session".to_string()),
            node_id: Some("test-node".to_string()),
            logic_hash: Some("telemetry-v1".to_string()),
            institutional_tag: "DRASUS_TEST".to_string(),
        }
    }

    /// CRITERIO #3: registrar una muestra (encolar, no el flush a disco)
    /// tarda menos de 50µs — medido de verdad con `Instant`, no asumido.
    /// Promedio sobre 1000 llamadas para no depender del ruido de una sola
    /// medición.
    #[tokio::test]
    async fn record_heartbeat_enqueues_in_under_50_microseconds() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");

        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(1_000, 100));
        let buffer = TelemetryBuffer::new(pool, clock, test_identity(), TelemetryBufferConfig::default());

        // Precalentamiento: evita medir el costo de "primera vez" del alocador.
        for _ in 0..10 {
            buffer.record_heartbeat("warmup");
        }

        const ITERATIONS: u32 = 1_000;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            buffer.record_heartbeat("bench.heartbeat");
        }
        let elapsed = start.elapsed();
        let per_call_nanos = elapsed.as_nanos() / u128::from(ITERATIONS);

        assert!(
            per_call_nanos < 50_000,
            "cada record_heartbeat debe tardar menos de 50µs; tardó {per_call_nanos}ns en promedio sobre {ITERATIONS} llamadas"
        );
    }

    /// CRITERIO #4: el buffer no bloquea al llamador mientras el flush a
    /// disco está en curso. Se simula un disco lento sosteniendo el lock de
    /// escritura de SQLite (`BEGIN IMMEDIATE`) en una conexión separada
    /// durante 150ms; mientras esa conexión retiene el lock, se llama
    /// `record_heartbeat` 100 veces y se mide que el total sea muchísimo
    /// menor que esos 150ms. Al final se libera el lock y se confirma que
    /// las muestras encoladas sí terminan persistidas.
    #[tokio::test]
    async fn record_does_not_block_while_a_slow_flush_is_in_progress() {
        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("telemetry_nonblocking.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(1_000, 100));
        let buffer = TelemetryBuffer::new(
            pool.clone(),
            clock,
            test_identity(),
            TelemetryBufferConfig { flush_interval_ms: 10 },
        );
        buffer.bootstrap().await.expect("bootstrap sobre tabla vacía");
        buffer.spawn_flush_task();

        // "Disco lento": una conexión separada toma el lock de escritura de
        // SQLite y lo retiene 150ms.
        let mut locking_conn = pool.acquire().await.expect("adquirir conexión para simular el lock");
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *locking_conn)
            .await
            .expect("tomar el lock de escritura");

        let lock_held_for = std::time::Duration::from_millis(150);
        let release_at = tokio::time::Instant::now() + lock_held_for;

        const CALLS_WHILE_LOCKED: u32 = 100;
        let start = std::time::Instant::now();
        for i in 0..CALLS_WHILE_LOCKED {
            buffer.record_heartbeat(format!("metric.{i}"));
        }
        let elapsed_while_locked = start.elapsed();

        assert!(
            elapsed_while_locked < lock_held_for / 4,
            "100 llamadas a record_heartbeat tardaron {elapsed_while_locked:?} mientras el \"disco\" estaba \
             bloqueado por {lock_held_for:?} — debieron ser muchísimo más rápidas, el buffer no debe esperar al disco"
        );

        tokio::time::sleep_until(release_at).await;
        sqlx::query("COMMIT")
            .execute(&mut *locking_conn)
            .await
            .expect("liberar el lock de escritura");
        drop(locking_conn);

        // Le da tiempo a la tarea de fondo de reintentar el flush ahora que
        // el lock ya se liberó.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let repo = TelemetryRepository::new(&pool);
        let tail = repo.load_tail().await.expect("cargar la cola");
        assert!(
            tail.is_some(),
            "las muestras encoladas mientras el disco estaba lento deben terminar persistidas"
        );

        pool.close().await;
    }

    /// `bootstrap` siembra el estado de cadena desde la última muestra
    /// persistida — la siguiente muestra registrada en este proceso debe
    /// encadenar con la de la corrida anterior, no reiniciar en génesis.
    #[tokio::test]
    async fn bootstrap_seeds_chain_state_from_a_previous_run() {
        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("telemetry_bootstrap.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let previous_sample_id = {
            let pool = connect(&database_url).await.expect("conectar (corrida anterior)");
            migrate(&pool).await.expect("migrar (corrida anterior)");
            let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(1_000, 100));
            let buffer = TelemetryBuffer::new(pool.clone(), clock, test_identity(), TelemetryBufferConfig::default());
            buffer.bootstrap().await.expect("bootstrap inicial (tabla vacía)");
            buffer.record_heartbeat("previous_run.heartbeat");

            // Vacía manualmente la cola (sin tarea de fondo) para no
            // depender de timing en este test.
            let sample = {
                let chain_state = buffer.shared.chain_state.lock().expect("lock");
                chain_state.clone().expect("ya se registró una muestra")
            };
            TelemetryRepository::new(&pool)
                .insert_batch(std::slice::from_ref(&sample))
                .await
                .expect("insertar la muestra de la corrida anterior");

            pool.close().await;
            sample.id
        };

        // Corrida nueva: bootstrap debe ver la muestra anterior.
        let pool = connect(&database_url).await.expect("conectar (corrida nueva)");
        migrate(&pool).await.expect("migrar (corrida nueva)");
        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(2_000, 100));
        let buffer = TelemetryBuffer::new(pool.clone(), clock, test_identity(), TelemetryBufferConfig::default());
        buffer.bootstrap().await.expect("bootstrap desde la corrida anterior");

        let chained_id = {
            let chained = buffer.shared.chain_state.lock().expect("lock");
            chained.as_ref().expect("debe haber estado sembrado").id.clone()
        };
        assert_eq!(chained_id, previous_sample_id);

        pool.close().await;
    }
}
