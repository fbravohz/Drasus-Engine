//! [SHELL] Interfaz pública (puerto) de `shared`.
//!
//! Esta es la única superficie de la que pueden depender los módulos del
//! pipeline (`ingest`, `generate`, `validate`, `incubate`, `manage`,
//! `execute`, `feedback`, `withdraw`) al reusar componentes comunes
//! (ADR-0003).
//!
//! ## Clock (W3, `docs/features/clock.md`)
//!
//! Cada módulo que necesita la hora actual depende del puerto [`Clock`]
//! en vez de llamar directo al reloj del sistema:
//!
//! - [`SystemClock`]: implementación de producción (TTR-001,
//!   `request_type = REAL`), con precisión de nanosegundos y monótona no
//!   decreciente.
//! - [`DeterministicClock`]: implementación de backtest/test (TTR-002,
//!   `request_type = FAKE`), que solo avanza vía llamadas explícitas a
//!   `advance(ns)` / `tick()` — la misma semilla
//!   (`initial_timestamp_ns`, `step_ns`) y la misma secuencia de llamadas
//!   producen una secuencia de timestamps idéntica, bit a bit.
//!
//! ## Audit Log (`docs/features/audit-log.md` TTR-001)
//!
//! Cada módulo dispara eventos de auditoría a través de
//! [`AuditLogRepository::append`] en vez de escribir logs directamente
//! (audit-log.md: "El Core nunca escribe logs. En su lugar, dispara
//! eventos al puerto de auditoría injected.").
//!
//! - [`AuditEventContent`]: el payload del evento (`action_type`,
//!   `entity_type`, `entity_id`, `details_json`, más los campos del
//!   perfil "Ops / Auditoría" de ADR-0020 — `process_id` e
//!   `institutional_tag` son obligatorios).
//! - [`AuditEvent`]: un evento persistido y encadenado por hash
//!   (`audit_hash`, `audit_chain_hash`, `event_sequence_id`).
//! - [`AuditLogRepository`]: repositorio de solo-apéndice (`append`,
//!   `load_chain`, `events_for_entity`) — no existe superficie de
//!   update/delete.
//! - [`verify_chain`] / [`ChainVerificationResult`]: verificación pura de
//!   la cadena de hashes, detecta manipulación de eventos históricos.
//! - [`AuditLogError`]: tipo de error para operaciones del repositorio.
//!
//! ## Rastro de Auditoría del Clock (`docs/features/clock.md` "Gobernanza y Estándares")
//!
//! El Clock no tiene persistencia propia — sus tres eventos auditables se
//! emiten vía [`AuditLogRepository::append`] a través de
//! [`ClockAuditContext`] y las tres funciones `emit_*` de abajo. La
//! granularidad está fija en exactamente estos tres eventos;
//! `timestamp_ns()`, `advance(ns)` y `tick()` nunca emiten eventos de
//! auditoría.
//!
//! - [`ClockAuditContext`]: identidad provista por quien llama
//!   (`session_id`, `institutional_tag`, `process_id`) compartida por
//!   los tres eventos.
//! - [`ClockMode`]: `REAL` / `SIMULATION`, usado por
//!   [`emit_mode_transition`].
//! - [`emit_ntp_sync`]: `CLOCK_NTP_SYNC` (TTR-001, una vez al iniciar).
//! - [`emit_mode_transition`]: `CLOCK_MODE_TRANSITION` (en transiciones
//!   `REAL` <-> `SIMULATION`).
//! - [`emit_session_close`]: `CLOCK_SESSION_CLOSE` (TTR-002, una vez
//!   cuando cierra una sesión de simulación).
//!
//! ## Async Job Executor (`docs/features/async-job-executor.md`)
//!
//! Patrón de job asíncrono de tres fases (ADR-0011): enviar un job,
//! sondear su estado y progreso, recuperar su resultado inmutable una
//! vez terminal.
//!
//! - [`JobState`]: los cinco estados de la máquina de estados del job +
//!   [`validate_transition`] puro (TTR-002/004/006).
//! - [`Progress`] / [`estimate_remaining_seconds`]: progreso 0-100 y
//!   estimación de tiempo restante (TTR-005).
//! - [`Job`] / [`JobResult`] / [`NewJob`] / [`NewJobResult`] /
//!   [`RecoveredJob`]: tipos de la capa de persistencia
//!   (`jobs`/`job_results`, migración `0003_jobs.sql`).
//! - [`JobRepository`] / [`JobRepositoryError`]: el repositorio de
//!   `jobs`/`job_results` (TTR-001/003/004).
//! - [`JobExecutor`] / [`JobExecutorConfig`] / [`ExecutorIdentity`] /
//!   [`JobExecutorError`]: la cáscara del executor — enviar, recuperar
//!   en startup, levantar el pool de workers, sondear estado/resultado,
//!   cancelar (TTR-001/002/004/006).
//! - [`JobHandler`] / [`JobOutcome`] / [`ProgressReporter`] /
//!   [`CancellationToken`]: el contrato de callback enchufable por
//!   `job_type` (TTR-002/005/006). TTR-ASYNC-EXECUTOR-007 (conectar
//!   handlers reales desde `generate`/`validate`/`manage`/`incubate`/
//!   `feedback`) está fuera de alcance para esta historia.
//!
//! ## Telemetría (`docs/features/telemetry.md` TTR-001)
//!
//! Buffer de alta velocidad: cualquier módulo registra una muestra de
//! latencia o un heartbeat sin esperar al disco; una tarea de fondo vacía
//! la cola a SQLite por lotes.
//!
//! - [`TelemetrySample`] / [`TelemetrySampleContent`] / [`build_sample`] /
//!   [`expired_sample_ids`]: núcleo puro — construcción de una muestra
//!   encadenada y la decisión de poda por ventana de retención.
//! - [`TelemetryRepository`] / [`TelemetryError`]: repositorio de
//!   `telemetry_samples` (insertar por lote, purgar, consultar por
//!   `metric_name` + rango, migración `0004_telemetry.sql`).
//! - [`TelemetryBuffer`] / [`TelemetryBufferConfig`]: la cáscara — cola en
//!   memoria no bloqueante (`record_latency`/`record_heartbeat`), siembra
//!   de la cadena al iniciar (`bootstrap`), vaciado por lotes en segundo
//!   plano (`spawn_flush_task`) y poda (`purge`). Reusa [`ExecutorIdentity`]
//!   del Async Job Executor — mismo perfil de campos ADR-0020, no se
//!   duplica el tipo.
//!
//! ## Central Identity (`docs/features/central-identity.md`, ADR-0143,
//! ## ADR-0144, STORY-027)
//!
//! Cimiento #1 del substrato de monetización: la cuenta LOCAL de usuario.
//! `licensing-system`, `usage-metering` y `consent-registry` dependen de su
//! `owner_id`.
//!
//! - [`AccountIdentity`]: el tipo de puerto `identity_out` (ADR-0137,
//!   catálogo) — identidad de cuenta + estado de verificación, SIN
//!   secretos (ADR-0093).
//! - [`compute_hardware_fingerprint`] / [`validate_email_format`] /
//!   [`verify_oauth_signature`]: el núcleo puro (sin I/O, ADR-0002/0004).
//! - [`Account`] / [`NewAccount`] / [`AccountRepository`]: la tabla
//!   `accounts` (migración `0007_central_identity.sql`), MUTABLE con
//!   `row_version` (ADR-0141), no append-only.
//! - [`IdentityCache`] / [`IdentityCacheConfig`]: caché local con TTL
//!   (`IDENTITY_CACHE_TTL`, default 24h) para operación offline.
//! - [`CentralIdentityVerifier`] / [`LocalStubCentralIdentityVerifier`]: el
//!   puerto de verificación contra la Cabina de Mando Central, con su
//!   implementación stub local (ADR-0144: "puerto ahora, adaptador
//!   después" — la Cabina de Mando todavía no existe).
//! - [`verify_central_identity`]: harness CLI (Canal #2, ADR-0142) —
//!   `cargo run -p app -- verify central-identity --input '{"email":"a@b.com"}'`.
//!
//! ## Plan / Tier / Quota (`docs/features/plan-tier-quota.md`, ADR-0143,
//! ## ADR-0144, STORY-029)
//!
//! Cimiento #3 del substrato de monetización: el catálogo configurable de
//! planes. Produce el tipo de puerto `PlanLimits` que `licensing-system`
//! (#2) hoy consume por stub y que `usage-metering` (#4, futuro) necesitará.
//!
//! Vive bajo su propio submódulo público
//! ([`plan_tier_quota`]) en vez de aplanarse a este nivel superior: el
//! doc-comment de `plan_tier_quota` explica por qué (colisión de nombre
//! `PlanLimits` con el stub aún vigente en `licensing_system`).
//!
//! ## Usage Metering / Libro de Nocional (`docs/features/usage-metering.md`,
//! ## ADR-0143, ADR-0144, STORY-030)
//!
//! Cimiento #4 del substrato de monetización: el libro append-only de
//! nocional en USD por ciclo de facturación. Primer cimiento que consume
//! un puerto REAL de otro cimiento -- [`orchestrator::usage_metering::record_metered_operation`]
//! resuelve el `PlanLimits` REAL de `plan_tier_quota` (#3), no un stub.
//!
//! - [`domain::usage_metering::compute_notional`]: nocional de una
//!   operación, reescalado ×10¹⁶→×10⁸ con `i128` y redondeo explícito --
//!   EL punto de correctitud crítico de esta Story.
//! - [`domain::usage_metering::accumulate`] /
//!   [`domain::usage_metering::detect_quota_crossing`] /
//!   [`domain::usage_metering::derive_billing_cycle_id`]: acumulación por
//!   ciclo, veredicto de cuota y derivación del ciclo mensual.
//! - [`domain::usage_metering::MeteredOperation`]: entrada mínima de
//!   metering (placeholder hasta que el `Order` real de `execute`/EPIC-5
//!   exista).
//! - [`domain::usage_metering::UsageRecord`]: el tipo de puerto
//!   `usage_out` (acumulado + veredicto, sin secretos ADR-0093).
//! - [`persistence::usage_metering::UsageRepository`][]: repositorio
//!   APPEND-ONLY (`event_sequence_id`, ADR-0141) para `usage_records`
//!   (migración `0010_usage_metering.sql`).
//! - [`orchestrator::usage_metering::record_metered_operation`][]: la
//!   composición completa -- resuelve `PlanLimits` REAL + deriva el ciclo
//!   + persiste append-only.
//! - [`verify_usage_metering`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify usage-metering --input '{"tier":"FREE","operations":[...]}'`.
//!
//! ## Consent Registry / Registro de Consentimiento ToS (`docs/features/consent-registry.md`,
//! ## ADR-0143, ADR-0144, ADR-0141, STORY-031)
//!
//! Cimiento #5 del substrato de monetización: el registro append-only y
//! versionado de aceptación de ToS, con granularidad opt-in/opt-out por
//! tipo de dato -- la columna vertebral legal (GDPR) del firehose gratuito
//! (ADR-0143) y de `data-aggregation` (#9).
//!
//! - [`domain::consent_registry::needs_reacceptance`]: compara la versión
//!   aceptada contra la vigente (`REACCEPT_ON_VERSION_CHANGE`, FIJO).
//! - [`domain::consent_registry::resolve_coverage`]: EL punto de
//!   correctitud legal -- decide `Covered`/`NotCovered{reason}` para un
//!   tipo de dato; el default es SIEMPRE negar.
//! - [`domain::consent_registry::apply_consent_action`]: EL punto de
//!   modelado crítico -- fusiona el estado vigente con una acción nueva
//!   (aceptar versión / cambiar opt-outs) produciendo el snapshot
//!   COMPLETO que se persiste como fila-evento nueva (event-sourcing).
//! - [`domain::consent_registry::ConsentVerdict`]: el tipo de puerto
//!   `consent_out` (acumulado + veredicto, sin secretos ADR-0093).
//! - [`persistence::consent_registry::ConsentRepository`][]: repositorio
//!   APPEND-ONLY (`event_sequence_id`, ADR-0141) para `consent_records`
//!   (migración `0011_consent_registry.sql`).
//! - [`orchestrator::consent_registry::record_consent_action`] /
//!   [`orchestrator::consent_registry::resolve_consent_verdict`][]: la
//!   composición completa -- registrar un evento y resolver el veredicto
//!   de cobertura.
//! - [`verify_consent_registry`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify consent-registry --input
//!   '{"current_version":"v2","actions":[...],"query":{"data_type":"aggregation"}}'`
//!
//! ## Third-Party API Gateway (`docs/features/third-party-api-gateway.md`,
//! ## ADR-0143, ADR-0144, ADR-0142, ADR-0093, STORY-035)
//!
//! Cimiento #8 del substrato de monetización: la puerta de entrada
//! autenticada para terceros. Convierte cada capacidad interna
//! (certificación, feeds, ruteo) en un producto vendible por API sin
//! reabrir el core -- valida la credencial (hash SHA-256, NUNCA en claro,
//! ADR-0093), limita la tasa por ventana, consulta el `consent_out` REAL
//! de `consent-registry` (#5) y decide delegar (o no), sin cablear la
//! delegación real a los puertos internos (diferida, STORY-035 §8). El
//! servidor gRPC/tonic + mTLS + protos por dominio también quedan
//! diferidos -- el Core y el esquema son el contrato.
//!
//! - [`third_party_api_gateway::authenticate`] / [`third_party_api_gateway::
//!   hash_api_credential`]: autenticación con revocación prioritaria sobre
//!   un hash correcto.
//! - [`third_party_api_gateway::compute_rate_limit`]: la ventana de
//!   rate-limit determinista, borde exacto.
//! - [`third_party_api_gateway::decide_gateway_outcome`]: EL punto de
//!   modelado crítico -- compone las cuatro puertas (autenticación,
//!   endpoint habilitado, rate-limit, consentimiento) en la
//!   `ThirdPartyResponse` final.
//! - [`third_party_api_gateway::handle_gateway_request`]: la composición
//!   completa -- resuelve el `consent_out` REAL de #5 (no un stub) y
//!   persiste el registro de uso.
//! - [`verify_third_party_api_gateway`]: harness CLI (Canal #2, ADR-0142)
//!   -- `cargo run -p app -- verify third-party-api-gateway --input
//!   '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}'`.
//!
//! ## Data Anonymization & Aggregation (`docs/features/data-aggregation.md`,
//! ## ADR-0144, ADR-0102, ADR-0143, ADR-0141, ADR-0093, STORY-036)
//!
//! Cimiento #9 del substrato de monetización: convierte eventos de
//! ejecución individuales en índices agregados vendibles (sentimiento,
//! régimen, fricción de bróker, correlación) donde ningún usuario es
//! reconocible -- ruido gaussiano de privacidad diferencial con RNG
//! SEMBRADO e inyectado (nunca entropía del sistema), hash unidireccional
//! de topología de estrategia (SHA-256, ADR-0102) y k-anonimato con
//! supresión FIJA por debajo de `MIN_COHORT_SIZE`. El pipeline de venta
//! externa y la exposición por la API de terceros (#8) quedan diferidos.
//!
//! - [`data_aggregation::apply_differential_privacy`]: ruido gaussiano
//!   determinista (Box-Muller) -- misma semilla, mismo resultado siempre.
//! - [`data_aggregation::hash_strategy_topology`]: topología cruda ->
//!   SHA-256 hex, irreversible.
//! - [`data_aggregation::aggregate_index`]: EL punto de modelado crítico
//!   -- suma los valores cubiertos, aplica el ruido y verifica
//!   k-anonimato; `None` si suprime.
//! - [`data_aggregation::run_aggregation`]: la composición completa --
//!   resuelve el `consent_out` REAL de `consent-registry` (#5) evento por
//!   evento, respeta la separación de canales interno/externo
//!   (`EXTERNAL_SALE_ENABLED`) y persiste el snapshot append-only atómico.
//! - [`verify_data_aggregation`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify data-aggregation --input
//!   '{"seed":42,"min_cohort":5,"external_sale_enabled":false,"events":[{"metric_e8":150000000,"consent":"COVERED"}]}'`.
//!
//! ## Verified Account Registry (`docs/features/verified-account-registry.md`,
//! ## ADR-0145 cimiento #10 -- rector, ADR-0093, ADR-0143, ADR-0141, ADR-0020, STORY-037)
//!
//! Cimiento #10 y último del substrato de monetización: el pilar de
//! "Cuentas Verificadas" (análogo a myFXbook/MT5 Signals), con el
//! diferenciador soberano de que Drasus atestigua criptográficamente lo que
//! su propio motor ejecutó, no solo lo que el bróker reporta. Registra
//! cuentas multi-bróker bajo un `owner_id`, calcula su track record por
//! ámbito de atestación (`SOVEREIGN` vs `BROKER_READONLY`, distinción
//! INVIOLABLE) con un gain% que EXCLUYE el flujo de capital (depósitos/
//! retiros nunca cuentan como ganancia), firma el contenido de forma
//! reproducible, y solo publica con el `consent_out` REAL de
//! `consent-registry` (#5) -- default privado, nunca un stub.
//!
//! - [`verified_account_registry::compute_track_record`]: EL punto de
//!   modelado crítico -- suma `realized_pnl` de `OrderExecuted`
//!   (`EnrichedDomainEvent`, #6) para el PnL/gain%, y acumula
//!   `CapitalFlow` en columnas de transparencia SEPARADAS que nunca tocan
//!   el PnL ni el numerador del gain%.
//! - [`verified_account_registry::compute_track_record_signature`] /
//!   [`verified_account_registry::compute_track_record_audit_hash`]: firma
//!   reproducible del CONTENIDO (subset V) vs. hash de la FILA del ledger
//!   (Grupo I) -- mismo patrón que `institutional_report_engine` (#7).
//! - [`verified_account_registry::decide_publication`]: el gate de
//!   publicación puro -- sin consentimiento vigente, NUNCA avanza a
//!   `PUBLIC`.
//! - [`verified_account_registry::AttestationScope::is_sovereign_attestation`]:
//!   la ÚNICA fuente de verdad sobre si un track puede reclamar "Ejecución
//!   Verificada por Drasus".
//! - [`orchestrator::verified_account_registry::register_account`] /
//!   [`orchestrator::verified_account_registry::attest_track_record`] /
//!   [`orchestrator::verified_account_registry::request_publication`]: la
//!   composición completa -- registra (default PRIVATE), calcula y firma
//!   el track por ámbito, y resuelve el `consent_out` REAL de #5 antes de
//!   publicar.
//! - [`persistence::verified_account_registry::VerifiedAccountRepository`]:
//!   repositorio MUTABLE (`row_version`, ADR-0141) para `verified_accounts`.
//! - [`persistence::verified_account_registry::AttestedTrackRecordRepository`]:
//!   repositorio APPEND-ONLY ATÓMICO (`event_sequence_id`, `BEGIN IMMEDIATE`
//!   + reintento) para `attested_track_records`.
//! - [`verify_verified_account_registry`]: harness CLI (Canal #2, ADR-0142)
//!   -- `cargo run -p app -- verify verified-account-registry --input
//!   '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN"},"consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}'`.
//!
//! ## Instance Continuity (`docs/features/instance-continuity.md`,
//! ## ADR-0146 cimiento #11 -- rector, ADR-0093, ADR-0143, ADR-0141, ADR-0020, STORY-039)
//!
//! Cimiento #11 del substrato de monetización: respaldo cifrado
//! client-side de la DB local (el proveedor jamás lee el contenido) +
//! relevo de custodia "maestro itinerante" (exactamente una máquina
//! titular escritora de la cadena de auditoría por cuenta, en cada
//! instante). Motivo de urgencia (ADR-0146): en el tier de pago la
//! telemetría de trabajo se suprime en origen (ADR-0143) -- el historial
//! soberano de #10 NO está en el proveedor, así que un disco muerto lo
//! borraría de forma irreversible sin este cimiento.
//!
//! - [`instance_continuity::derive_encryption_key`]: KDF Argon2id --
//!   deriva la clave AES-256 desde el secreto maestro del usuario. La
//!   clave y el secreto NUNCA se persisten ni salen de esta función.
//! - [`instance_continuity::generate_nonce`]: nonce de AES-GCM con RNG
//!   SEMBRADO e inyectado (mismo patrón que el ruido de #9) -- nunca
//!   `rand::thread_rng()` en el Core.
//! - [`instance_continuity::encrypt_backup_blob`] /
//!   [`instance_continuity::decrypt_backup_blob`]: cifrado/descifrado
//!   autenticado AES-256-GCM -- el tag GCM detecta CUALQUIER manipulación,
//!   nunca devuelve basura silenciosa.
//! - [`instance_continuity::compute_backup_delta`]: filtra del snapshot
//!   crudo las credenciales de bróker y las IPs de servidor live -- las
//!   MISMAS clases de secreto que se excluyen de la telemetría (ADR-0093).
//! - [`instance_continuity::decide_custody_claim`] /
//!   [`instance_continuity::is_current_titular`]: EL gate de titularidad
//!   exclusiva -- concurrencia optimista a nivel de INSTANCIA COMPLETA
//!   (`custody_epoch`), no de una fila de negocio cualquiera.
//! - [`instance_continuity::take_encrypted_snapshot`]: la composición
//!   completa del `backup_blob_out` -- filtra, cifra y persiste
//!   append-only atómico.
//! - [`instance_continuity::claim_custody`] / [`instance_continuity::is_titular`]:
//!   la composición completa del `custody_status_out` -- reclama/consulta
//!   la titularidad persistida.
//! - [`verify_instance_continuity`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify instance-continuity --input
//!   '{"master_secret":"correct horse battery staple","plaintext":"snapshot-bytes","nonce_seed":42,"custody":{"titular_node_id":"node-A","custody_epoch":3},"my_node_id":"node-A"}'`.
//!
//! ## Master Account Hierarchy (`docs/features/master-account-hierarchy.md`,
//! ## ADR-0147 cimiento #12 -- rector y ÚLTIMO del substrato, ADR-0143, ADR-0141, ADR-0093, ADR-0020, STORY-040)
//!
//! Cimiento #12, cierra el substrato de monetización (12/12): una cuenta
//! maestra raíz (fondo) agrupa N cuentas maestras hijas con autoridad de
//! auditoría y override sobre cada una -- pero el mando NUNCA escribe
//! directo en la base de datos de la hija (el adaptador de red del relé
//! genérico, ADR-0143, queda diferido) y todo override exige consentimiento
//! vigente y queda doblemente atestado (fondo + hija, nunca una mutación
//! silenciosa).
//!
//! - [`master_account_hierarchy::decide_override_authorization`]: EL gate
//!   de autorización -- `Executed` solo si el `ConsentVerdict` REAL de
//!   `consent-registry` (#5) es `Covered`; sin opt-in vigente, `Denied`
//!   SIEMPRE.
//! - [`master_account_hierarchy::apply_local_command_effect`]: "eliminar" =
//!   archivar (ADR-0141) -- catálogo cerrado de dos valores, ninguno es un
//!   borrado físico.
//! - [`master_account_hierarchy::link_child_to_parent`]: registra el
//!   puntero de jerarquía de una hija hacia su fondo (regla fija #1: el
//!   puntero, nunca el árbol completo -- anti-`tenant_id`).
//! - [`master_account_hierarchy::issue_override`] /
//!   [`master_account_hierarchy::receive_override`]: la composición
//!   completa de la doble atestación -- el fondo emite (fila ISSUER), la
//!   hija re-valida localmente y ejecuta o rechaza (fila EXECUTOR).
//! - [`verify_master_account_hierarchy`]: harness CLI (Canal #2,
//!   ADR-0142) -- `cargo run -p app -- verify master-account-hierarchy
//!   --input '{"parent_owner_id":"fund-X","child_owner_id":"trader-7",
//!   "node_id":"node-A","consent":"COVERED","command_kind":"ARCHIVE",
//!   "target_ref":"strategy-42","justification":"riesgo excedido"}'`.
//!
//! ## Data Portability (`docs/features/data-portability.md`, ADR-0148
//! ## cimiento #13, ADR-0141, ADR-0093, ADR-0020, STORY-043)
//!
//! Cimiento #13: infraestructura de cumplimiento transversal (mismo nivel
//! que `consent-registry`, #5), NO dominio de trading -- da a un `owner_id`
//! autenticado el derecho a exportar sus datos (Art. 15/20 GDPR) y a pedir
//! el olvido (Art. 17), con excepciones de retención legal. Se cimenta
//! AHORA un catálogo declarativo de qué tablas portan `owner_id` y un
//! registro append-only de solicitudes con su estado; el generador de
//! archivo real (recorrer el esquema y volcar el dato) y la UI quedan
//! diferidos (adaptador posterior sobre este mismo puerto).
//!
//! - [`data_portability::decide_forget_disposition`]: la ÚNICA puerta que
//!   decide qué le pasa a una tabla al pedirse el olvido -- SIEMPRE
//!   pseudonimización, NUNCA un DELETE físico (ADR-0141); el catálogo de
//!   salida es estructuralmente incapaz de expresar un borrado.
//! - [`data_portability::is_excluded_from_export`]: filtro de exclusión de
//!   secretos (ADR-0093) -- ninguna credencial de bróker, clave de cifrado
//!   ni IP de servidor live llega jamás al manifiesto de exportación.
//! - [`data_portability::seed_known_catalog`]: siembra idempotente del
//!   catálogo con las tablas ya conocidas del substrato que portan
//!   `owner_id`.
//! - [`data_portability::request_export`] / [`data_portability::request_forget`]:
//!   la composición completa -- arma el manifiesto/detalle de disposición
//!   vía el Core y registra el evento `RECEIVED` append-only atómico.
//! - [`verify_data_portability`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify data-portability --input
//!   '{"owner_id":"user-42","institutional_tag":"LIVE","node_id":"node-A","request_type":"FORGET"}'`.
//!
//! ## Operator Roles (`docs/features/operator-roles.md`, ADR-0149
//! ## cimiento #14 y ÚLTIMO del substrato, ADR-0123, ADR-0141, ADR-0020,
//! ## STORY-044)
//!
//! Cimiento #14: dentro de UNA cuenta maestra, el dueño crea roles de
//! operador a la carta (matriz de capacidades por puerto de Feature) y los
//! asigna a operadores (`HUMAN` o `AGENT`). Una llamada se concede solo si
//! el rol la permite Y pasa el evaluador de riesgo de pipeline ya
//! existente (`mcp_gateway::evaluate_permission`, ADR-0123) -- el gate de
//! rol es ADICIONAL, nunca sustituto. El invariante "último admin en pie"
//! corre DENTRO de la misma transacción que escribe cada mutación
//! admin-afectante.
//!
//! - [`operator_roles::CapabilityMatrix`]: matriz de capacidades
//!   `BTreeMap<String, bool>` (ordenada, hash determinista) -- denegada por
//!   defecto si la clave no está declarada.
//! - [`operator_roles::evaluate_operator_call`]: compone el gate de rol con
//!   `mcp_gateway::evaluate_permission` -- concede solo si AMBOS conceden.
//! - [`operator_roles::check_last_admin_standing`]: el invariante puro
//!   "último admin en pie", cuyo guardarraíl en la Shell corre dentro de
//!   `BEGIN IMMEDIATE`.
//! - [`operator_roles::seed_admin_bootstrap`]: siembra el rol ADMIN inicial
//!   y lo asigna al primer operador `HUMAN` de la cuenta -- idempotente.
//! - [`verify_operator_roles`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify operator-roles --input
//!   '{"owner_id":"acc-1","institutional_tag":"LIVE","node_id":"node-A","access_token_id":"tok-owner","capability_key":"generate.run_search","pipeline":"GENERATE"}'`.

pub use crate::clock_audit::{
    emit_mode_transition, emit_ntp_sync, emit_session_close, ClockAuditContext, ClockMode,
};
pub use crate::domain::audit_log::{
    AuditEvent, AuditEventContent, ChainVerificationResult, verify_chain,
};
pub use crate::domain::central_identity::{
    compute_account_audit_hash, compute_hardware_fingerprint, normalize_email,
    validate_email_format, verify_oauth_signature, AccountIdentity, EmailFormatError,
    EmailVerificationStatus, HardwareFingerprintError, OAuthTokenMaterial,
};
pub use crate::orchestrator::central_identity::{
    CentralIdentityError, CentralIdentityVerifier, IdentityCache, IdentityCacheConfig,
    IdentityVerificationRequest, LocalStubCentralIdentityVerifier,
};
pub use crate::persistence::central_identity::{
    Account, AccountRepository, AccountRepositoryError, NewAccount,
};
pub use crate::domain::licensing_system::{
    canonical_license_bytes, derive_execution_gate, evaluate_heartbeat_status, hardware_matches,
    heartbeat_status_to_compliance_status_id, verify_license_signature, ExecutionGate,
    GateEvaluationInput, GateVerdict, HeartbeatConfig, HeartbeatStatus, LicensePayload,
    LicenseSignatureError, LicenseTier, PlanLimits, DEFAULT_HEARTBEAT_INTERVAL_NS,
};
pub use crate::orchestrator::licensing_system::{
    build_execution_gate, sync_compliance_status, BuildExecutionGateError, ExecutionGateCache,
    ExecutionGateCacheConfig, IssueLicenseRequest, LocalStubLicenseIssuer,
    LocalStubPlanLimitsProvider, PlanLimitsProvider, SignedLicenseFile,
};
pub use crate::persistence::licensing_system::{
    LicenseRecord, LicenseRepository, LicenseRepositoryError, NewLicenseActivation,
};
pub use crate::domain::clock::{Clock, DeterministicClock};
pub use crate::domain::job::{estimate_remaining_seconds, validate_transition, InvalidTransition, JobState, Progress};
pub use crate::orchestrator::job_executor::{
    CancellationToken, ExecutorIdentity, JobExecutor, JobExecutorConfig, JobExecutorError, JobHandler, JobOutcome,
    ProgressReporter, JOB_RECOVERED_AT_STARTUP,
};
pub use crate::orchestrator::telemetry::{TelemetryBuffer, TelemetryBufferConfig};
pub use crate::orchestrator::SystemClock;
pub use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};
pub use crate::persistence::pool::{connect as create_pool, migrate as run_migrations};
pub use crate::persistence::job::{Job, JobRepository, JobRepositoryError, JobResult, NewJob, NewJobResult, RecoveredJob};
pub use crate::persistence::telemetry::{TelemetryError, TelemetryRepository};
pub use crate::domain::telemetry::{build_sample, expired_sample_ids, TelemetrySample, TelemetrySampleContent};
pub use crate::domain::worker_orchestrator::{WorkerBackend, WorkerBackendError, WorkerConfig, WorkerOrchestrator};
pub use crate::orchestrator::worker_runner::{graceful_shutdown, is_process_alive, open_readonly, OsWorkerBackend, SharedMemorySegment, ShmError};
pub use crate::domain::mcp_gateway::{
    evaluate_permission, compute_audit_hash, outcome_to_string, institutional_tag_to_string,
    InstitutionalTag, PermissionDecision, PermissionOutcome, PermissionRequest, Pipeline,
};
pub use crate::orchestrator::mcp_server::run_mcp_server;
pub use crate::persistence::mcp_gateway::{McpGatewayError, McpGatewayRepository};

// ── Harness de verificación CLI de Central Identity (ADR-0142 Fase 1) ───────

/// Input para la verificación de Central Identity vía CLI (`docs/features/central-identity.md`,
/// STORY-027). Se deserializa desde el JSON que pasa el usuario con
/// `--input '...'`.
///
/// `email` es el único campo obligatorio: `cargo run -p app -- verify
/// central-identity --input '{"email":"a@b.com"}'` ya es una invocación
/// válida. El resto tiene valores por defecto razonables para una
/// verificación de humo rápida.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CentralIdentityVerifyInput {
    /// Correo con el que se registra/vincula la cuenta.
    pub email: String,
    /// Proveedor de identidad federada, si el login fue vía OAuth.
    #[serde(default)]
    pub oauth_provider: Option<String>,
    /// Identificadores de máquina sin procesar para calcular la huella de
    /// hardware. Si se omite, usa el hostname del proceso como único
    /// identificador (suficiente para una verificación de humo local).
    #[serde(default)]
    pub machine_identifiers: Option<Vec<String>>,
    /// Entorno/etiqueta institucional de la cuenta.
    #[serde(default = "default_institutional_tag")]
    pub institutional_tag: String,
}

/// Valor por defecto de `institutional_tag` cuando el usuario no lo pasa en
/// `--input` -- una verificación de humo local no pertenece a ningún
/// entorno de producción real.
fn default_institutional_tag() -> String {
    "DRASUS_LOCAL_VERIFY".to_string()
}

/// Output de la verificación de Central Identity. Siempre serializa a JSON
/// válido (ADR-0142: "JSON estructurado en el CLI, FIJO").
///
/// Si `ok` es `true`, los campos de identidad están rellenos y coinciden
/// EXACTAMENTE con lo que expondría el puerto `identity_out` -- ningún
/// campo adicional, ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CentralIdentityVerifyOutput {
    /// `true` si la verificación completó sin errores.
    pub ok: bool,
    pub owner_id: Option<String>,
    pub email: Option<String>,
    pub email_verification_status: Option<String>,
    pub node_id: Option<String>,
    pub institutional_tag: Option<String>,
    /// `true` si el valor devuelto salió de la caché con TTL en vez de una
    /// verificación fresca contra el verificador (en esta llamada de CLI,
    /// siempre pasa por ambos pasos: verifica y luego cachea, así que
    /// `cached` confirma que el cableado caché -> puerto quedó correcto).
    pub cached: bool,
    pub error: Option<String>,
}

impl CentralIdentityVerifyOutput {
    /// Construye un output de error con todos los campos de identidad en
    /// `None`.
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            owner_id: None,
            email: None,
            email_verification_status: None,
            node_id: None,
            institutional_tag: None,
            cached: false,
            error: Some(msg),
        }
    }

    /// Construye un output exitoso a partir de la identidad ya cacheada.
    fn from_identity(identity: AccountIdentity) -> Self {
        Self {
            ok: true,
            owner_id: Some(identity.owner_id),
            email: Some(identity.email),
            email_verification_status: Some(identity.email_verification_status.as_str().to_string()),
            node_id: Some(identity.node_id),
            institutional_tag: Some(identity.institutional_tag),
            cached: true,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Central Identity con adaptadores reales
/// (BD SQLite temporal + reloj de sistema real + verificador stub local).
///
/// Crea una BD SQLite temporal exclusiva para esta verificación (mismo
/// patrón que `sovereign-data-fetcher::public_interface::verify`), aplica
/// las migraciones embebidas, verifica/vincula la identidad vía
/// [`LocalStubCentralIdentityVerifier`], la guarda en una [`IdentityCache`]
/// recién creada y devuelve lo que la caché reporta -- ejercitando el
/// camino completo Core -> Shell -> puerto que un usuario real recorrería.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify central-identity --input '{"email":"a@b.com"}'`
pub async fn verify_central_identity(input: CentralIdentityVerifyInput) -> CentralIdentityVerifyOutput {
    // BD SQLite temporal exclusiva para esta verificación -- no contamina
    // datos de producción (mismo patrón que sovereign-data-fetcher::verify).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-central-identity-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return CentralIdentityVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return CentralIdentityVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return CentralIdentityVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    // Reloj de producción: la caché mide el TTL contra la hora real.
    // `Arc<dyn Clock>` porque tanto el verificador como la caché necesitan
    // su propia referencia compartida al mismo reloj.
    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Sin identificadores de máquina explícitos: usa el hostname del
    // proceso como único identificador -- suficiente para una verificación
    // de humo local (no se espera acceso a hardware real en CI).
    let machine_identifiers = input.machine_identifiers.unwrap_or_else(|| {
        vec![hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown-host".to_string())]
    });

    let verifier = crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
    let request = crate::orchestrator::central_identity::IdentityVerificationRequest {
        email: input.email,
        oauth_provider: input.oauth_provider,
        machine_identifiers,
        institutional_tag: input.institutional_tag,
        access_token_id: None,
    };

    let identity = match verifier.verify_identity(request).await {
        Ok(identity) => identity,
        Err(e) => return CentralIdentityVerifyOutput::from_error(e.to_string()),
    };

    // Pasa por la caché con TTL antes de reportar -- ejercita el cableado
    // completo que el observable de la Story describe ("identidad cacheada
    // + estado"), no solo la verificación cruda.
    let cache = crate::orchestrator::central_identity::IdentityCache::new(
        clock,
        crate::orchestrator::central_identity::IdentityCacheConfig::default(),
    );
    cache.set(identity);

    match cache.get() {
        Some(cached_identity) => CentralIdentityVerifyOutput::from_identity(cached_identity),
        // Inalcanzable en la práctica: acabamos de guardar con TTL de 24h;
        // solo fallaría si el reloj del sistema saltara 24h entre `set` y
        // `get`, lo cual no ocurre en una sola invocación síncrona del CLI.
        None => CentralIdentityVerifyOutput::from_error(
            "la identidad recién guardada ya no está vigente en la caché (inesperado)".to_string(),
        ),
    }
}

// ── Harness de verificación CLI de Licensing System (ADR-0142 Fase 1) ───────

/// Input para la verificación de Licensing System vía CLI (`docs/features/licensing-system.md`,
/// STORY-028). Se deserializa desde el JSON que pasa el usuario con
/// `--input '...'`.
///
/// `tier` es el único campo que un uso típico necesita:
/// `cargo run -p app -- verify licensing-system --input '{"tier":"SOVEREIGN"}'`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicensingSystemVerifyInput {
    /// `"SOVEREIGN"` o `"EXPLORER"` (`docs/features/licensing-system.md`
    /// "Niveles de Licencia").
    #[serde(default = "default_license_tier")]
    pub tier: String,
    /// Correo de la cuenta local a vincular (vía `central-identity`, puerto
    /// `identity_in`). Si se omite, usa un correo fijo de verificación.
    #[serde(default = "default_owner_email")]
    pub owner_email: String,
}

fn default_license_tier() -> String {
    "SOVEREIGN".to_string()
}

fn default_owner_email() -> String {
    "verify-licensing@drasus.local".to_string()
}

/// Output de la verificación de Licensing System. Siempre serializa a JSON
/// válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que expone
/// el puerto `execution_gate_out` -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicensingSystemVerifyOutput {
    pub ok: bool,
    pub verdict: Option<String>,
    pub tier: Option<String>,
    pub suppress_work_telemetry: Option<bool>,
    pub activations: Option<i64>,
    pub reason: Option<String>,
    pub error: Option<String>,
}

impl LicensingSystemVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            verdict: None,
            tier: None,
            suppress_work_telemetry: None,
            activations: None,
            reason: None,
            error: Some(msg),
        }
    }

    fn from_gate(gate: ExecutionGate) -> Self {
        let verdict = match gate.verdict {
            GateVerdict::Allow => "Allow",
            GateVerdict::Deny => "Deny",
            GateVerdict::UpgradeRequired => "UpgradeRequired",
        };
        Self {
            ok: true,
            verdict: Some(verdict.to_string()),
            tier: Some(gate.tier.as_str().to_string()),
            suppress_work_telemetry: Some(gate.suppress_work_telemetry),
            activations: Some(gate.activations),
            reason: Some(gate.reason),
            error: None,
        }
    }
}

/// Ejecuta la verificación de Licensing System con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real + emisor de licencia stub +
/// proveedor de límites stub), recorriendo el camino completo del cimiento
/// #2: vincula una `AccountIdentity` local (reutiliza `central-identity`,
/// puerto `identity_in` -- NO recalcula la huella de hardware), emite y
/// activa una licencia de desarrollo firmada para el `tier` pedido, obtiene
/// `PlanLimits` del stub (puerto `plan_limits_in`), construye el
/// `ExecutionGate` y lo pasa por su caché con TTL antes de reportar.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify licensing-system --input '{"tier":"SOVEREIGN"}'`
pub async fn verify_licensing_system(input: LicensingSystemVerifyInput) -> LicensingSystemVerifyOutput {
    let tier = match LicenseTier::from_str_value(&input.tier) {
        Some(tier) => tier,
        None => {
            return LicensingSystemVerifyOutput::from_error(format!(
                "tier desconocido: '{}' -- se esperaba SOVEREIGN o EXPLORER",
                input.tier
            ))
        }
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón que
    // verify_central_identity).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-licensing-system-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return LicensingSystemVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return LicensingSystemVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return LicensingSystemVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- identity_in: vincula/crea la AccountIdentity local vía
    // central-identity (REUTILIZA su huella de hardware, no la recalcula).
    let machine_identifiers = vec![hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string())];
    let identity_verifier =
        crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
    let identity = match identity_verifier
        .verify_identity(crate::orchestrator::central_identity::IdentityVerificationRequest {
            email: input.owner_email,
            oauth_provider: None,
            machine_identifiers,
            institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
            access_token_id: None,
        })
        .await
    {
        Ok(identity) => identity,
        Err(e) => return LicensingSystemVerifyOutput::from_error(format!("fallo al vincular identidad: {e}")),
    };

    // Paso 2 -- emisor stub: firma una licencia de desarrollo para esta
    // cuenta + esta máquina (la clave privada nunca sale de `issuer`).
    let issuer = LocalStubLicenseIssuer::new();
    let now_ns = clock.timestamp_ns();
    let signed = issuer.issue_license(IssueLicenseRequest {
        owner_id: identity.owner_id.clone(),
        node_id: identity.node_id.clone(),
        tier,
        issued_at_ns: now_ns,
        heartbeat_expires_at_ns: now_ns + DEFAULT_HEARTBEAT_INTERVAL_NS,
    });

    // Paso 3 -- activa (persiste) la licencia firmada para esta máquina.
    let license_repo = LicenseRepository::new(&pool, clock.as_ref());
    let license = match license_repo
        .activate(NewLicenseActivation {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            access_token_id: None,
            node_id: identity.node_id.clone(),
            license_id: signed.license_id.clone(),
            process_id: Some(format!("drasus-pid-{}", std::process::id())),
            signature_hash: signed.signature_hex.clone(),
            tier,
            issued_at_ns: signed.issued_at_ns,
            heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
            compliance_status_id: "ACTIVE".to_string(),
        })
        .await
    {
        Ok(license) => license,
        Err(e) => return LicensingSystemVerifyOutput::from_error(format!("fallo al activar licencia: {e}")),
    };

    // Paso 4 -- plan_limits_in: límites del stub (plan-tier-quota real, diferido).
    let plan_limits_provider = LocalStubPlanLimitsProvider::default();
    let plan_limits = plan_limits_provider.plan_limits_for(&identity.owner_id, tier).await;

    // Paso 5 -- construye el veredicto (fuera del hot-path: esta función SÍ
    // hace lecturas de BD local; el hot-path real solo leería la caché).
    let heartbeat_config = HeartbeatConfig::default();
    let gate = match build_execution_gate(
        &pool,
        clock.as_ref(),
        &identity.node_id,
        &license,
        &signed.signature_hex,
        &signed.public_key_hex,
        &heartbeat_config,
        &plan_limits,
    )
    .await
    {
        Ok(gate) => gate,
        Err(e) => return LicensingSystemVerifyOutput::from_error(format!("fallo al construir el gate: {e}")),
    };

    // Paso 6 -- pasa por la caché con TTL antes de reportar, ejercitando el
    // cableado completo que el hot-path real consultaría.
    let cache = ExecutionGateCache::new(clock, ExecutionGateCacheConfig::default());
    cache.set(gate);

    match cache.get() {
        Some(cached_gate) => LicensingSystemVerifyOutput::from_gate(cached_gate),
        // Inalcanzable en la práctica: acabamos de guardar con el TTL por
        // defecto (5 minutos); solo fallaría si el reloj saltara ese tiempo
        // entre `set` y `get`, lo cual no ocurre en una invocación síncrona.
        None => LicensingSystemVerifyOutput::from_error(
            "el veredicto recién guardado ya no está vigente en la caché (inesperado)".to_string(),
        ),
    }
}

// ── Plan / Tier / Quota (STORY-029, vive en `shared` -- ver ADR-0137) ───────

/// Submódulo público del cimiento #3 (`docs/features/plan-tier-quota.md`,
/// ADR-0143, ADR-0144, STORY-029).
///
/// **Por qué un submódulo y no un `pub use` plano como el resto de este
/// archivo:** el puerto `plan_limits_out` de esta Feature produce un tipo
/// llamado `PlanLimits` (ADR-0137, catálogo, enmienda 2026-07-03). Pero
/// `licensing-system` (cimiento #2, STORY-028, YA SELLADO) declaró antes
/// su PROPIO struct `PlanLimits` como stub temporal
/// (`domain::licensing_system::PlanLimits`, sin `notional_limit`), y ese
/// nombre ya está aplanado en este mismo archivo unas líneas arriba.
/// Aplanar aquí el `PlanLimits` real de este cimiento colisionaría
/// (`error[E0255]: the name 'PlanLimits' is defined multiple times`). La
/// Orden de esta Story prohíbe expresamente tocar el código sellado de
/// `licensing-system` para unificarlos ("Re-cableado de licensing-system
/// (#2)... NO parte de esta Orden", STORY-029 §8) -- por eso, mientras ese
/// follow-up de integración no se ejecute, ambos tipos conviven bajo rutas
/// distintas: `public_interface::PlanLimits` (el stub de #2) y
/// `public_interface::plan_tier_quota::PlanLimits` (el real de #3).
pub mod plan_tier_quota {
    pub use crate::domain::plan_tier_quota::{
        canonical_features_json, compute_plan_audit_hash, decode_features_json, resolve_limits,
        validate_plan, PlanCandidate, PlanLimits, PlanSnapshot, PlanTier, PlanValidationError,
        PricingModel,
    };
    pub use crate::orchestrator::plan_tier_quota::{
        build_plan_limits_for_tier, seed_default_catalog, BuildPlanLimitsError,
        LocalStubPlanCatalogConfig, PlanLimitsCache, PlanLimitsCacheConfig,
    };
    pub use crate::persistence::plan_tier_quota::{
        NewPlan, Plan, PlanRepository, PlanRepositoryError,
    };
}

// ── Usage Metering (STORY-030, vive en `shared` -- ver ADR-0137) ───────────

pub use crate::domain::usage_metering::{
    accumulate, compute_notional, compute_usage_audit_hash, derive_billing_cycle_id,
    detect_quota_crossing, MeteredOperation, NotionalError, QuotaVerdict, UsageRecord,
    AMOUNT_SCALE,
};
pub use crate::orchestrator::usage_metering::{record_metered_operation, RecordMeteredOperationError};
pub use crate::persistence::usage_metering::{
    RecordOperationInput, UsageRecordRow, UsageRepository, UsageRepositoryError,
};

/// Una operación de entrada para la verificación de Usage Metering vía CLI
/// -- espejo mínimo de [`MeteredOperation`] pero con campos `String`/`i64`
/// deserializables directamente desde JSON (`MeteredOperation` toma
/// `&str`, no apto para deserializar con ownership propio).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeteredOperationVerifyInput {
    /// Tamaño operado, `INTEGER` escalado ×10⁸.
    pub size: i64,
    /// Precio de ejecución, `INTEGER` escalado ×10⁸.
    pub price: i64,
    #[serde(default = "default_verify_instrument_id")]
    pub instrument_id: String,
}

fn default_verify_instrument_id() -> String {
    "BTCUSDT".to_string()
}

/// Input para la verificación de Usage Metering vía CLI
/// (`docs/features/usage-metering.md`, STORY-030). Se deserializa desde
/// el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify usage-metering --input
/// '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageMeteringVerifyInput {
    /// `"FREE"` o `"PAID"` (mismo vocabulario que `plan_tier_quota::PlanTier`).
    #[serde(default = "default_plan_tier")]
    pub tier: String,
    /// Las operaciones a registrar, EN ORDEN, contra el mismo dueño y el
    /// mismo ciclo -- cada una se acumula sobre la anterior.
    pub operations: Vec<MeteredOperationVerifyInput>,
}

/// Output de la verificación de Usage Metering. Siempre serializa a JSON
/// válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que expone
/// el puerto `usage_out` tras la ÚLTIMA operación registrada -- ningún
/// secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageMeteringVerifyOutput {
    pub ok: bool,
    pub tier: Option<String>,
    pub billing_cycle_id: Option<String>,
    pub cycle_accumulated: Option<i64>,
    pub quota_verdict: Option<String>,
    /// Cuántas operaciones se registraron con éxito antes de reportar (o
    /// antes de que una fallara).
    pub operations_recorded: usize,
    pub error: Option<String>,
}

impl UsageMeteringVerifyOutput {
    fn from_error(operations_recorded: usize, msg: String) -> Self {
        Self {
            ok: false,
            tier: None,
            billing_cycle_id: None,
            cycle_accumulated: None,
            quota_verdict: None,
            operations_recorded,
            error: Some(msg),
        }
    }

    fn from_record(tier: plan_tier_quota::PlanTier, record: UsageRecord, operations_recorded: usize) -> Self {
        Self {
            ok: true,
            tier: Some(tier.as_str().to_string()),
            billing_cycle_id: Some(record.billing_cycle_id),
            cycle_accumulated: Some(record.cycle_accumulated),
            quota_verdict: Some(record.quota_verdict.as_str().to_string()),
            operations_recorded,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Usage Metering con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real + catálogo REAL de
/// plan-tier-quota), recorriendo el camino completo del cimiento #4:
/// siembra el catálogo Free/Paid real (#3), registra CADA operación de
/// `input.operations` EN ORDEN (acumulando sobre la misma cuenta y el
/// mismo ciclo vigente) vía [`record_metered_operation`], y reporta el
/// `UsageRecord` resultante de la ÚLTIMA operación -- ejercitando Core ->
/// Shell -> puerto tal como lo recorrería un usuario real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify usage-metering --input
/// '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'`
pub async fn verify_usage_metering(input: UsageMeteringVerifyInput) -> UsageMeteringVerifyOutput {
    let tier = match plan_tier_quota::PlanTier::from_str_value(&input.tier) {
        Some(tier) => tier,
        None => {
            return UsageMeteringVerifyOutput::from_error(
                0,
                format!("tier desconocido: '{}' -- se esperaba FREE o PAID", input.tier),
            )
        }
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_central_identity / verify_licensing_system / verify_plan_tier_quota).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-usage-metering-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return UsageMeteringVerifyOutput::from_error(
            0,
            format!("no se pudo crear el directorio temporal de verificación: {e}"),
        );
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return UsageMeteringVerifyOutput::from_error(
                0,
                format!("no se pudo crear la BD temporal de verificación: {e}"),
            )
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return UsageMeteringVerifyOutput::from_error(
            0,
            format!("error al aplicar migraciones en la BD temporal: {e}"),
        );
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- siembra el catálogo REAL de plan-tier-quota (#3) si esta
    // BD temporal todavía no lo tiene. Sin esto, record_metered_operation
    // fallaría con PlanNotFound -- el cableado real exige que el catálogo
    // exista, no hay fallback silencioso a un stub.
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    if let Err(e) = plan_tier_quota::seed_default_catalog(
        &pool,
        clock.as_ref(),
        "drasus-system",
        &node_id,
        "DRASUS_LOCAL_VERIFY",
        &plan_tier_quota::LocalStubPlanCatalogConfig::default(),
    )
    .await
    {
        return UsageMeteringVerifyOutput::from_error(0, format!("fallo al sembrar el catálogo: {e}"));
    }

    // Paso 2 -- registra cada operación EN ORDEN, acumulando sobre la
    // misma cuenta y el mismo ciclo vigente (fuera del hot-path: esta
    // función SÍ hace lecturas/escrituras de BD local).
    let owner_id = "verify-usage-metering-owner";
    let mut last_record: Option<UsageRecord> = None;
    for (index, operation) in input.operations.iter().enumerate() {
        let result = record_metered_operation(
            &pool,
            clock.as_ref(),
            owner_id,
            "DRASUS_LOCAL_VERIFY",
            &node_id,
            tier,
            MeteredOperation {
                size: operation.size,
                price: operation.price,
                instrument_id: &operation.instrument_id,
            },
        )
        .await;

        match result {
            Ok(record) => last_record = Some(record),
            Err(e) => {
                return UsageMeteringVerifyOutput::from_error(
                    index,
                    format!("fallo al registrar la operación #{index}: {e}"),
                )
            }
        }
    }

    match last_record {
        Some(record) => UsageMeteringVerifyOutput::from_record(tier, record, input.operations.len()),
        // Sin operaciones en el input: no hay nada que reportar como
        // UsageRecord, pero tampoco es un error -- el catálogo se sembró
        // y el tier es válido, simplemente no se registró ninguna operación.
        None => UsageMeteringVerifyOutput::from_error(
            0,
            "no se proveyó ninguna operación en 'operations' -- nada que registrar".to_string(),
        ),
    }
}

/// Input para la verificación de Plan / Tier / Quota vía CLI
/// (`docs/features/plan-tier-quota.md`, STORY-029). Se deserializa desde
/// el JSON que pasa el usuario con `--input '...'`.
///
/// `tier` es el único campo que un uso típico necesita:
/// `cargo run -p app -- verify plan-tier-quota --input '{"tier":"FREE"}'`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanTierQuotaVerifyInput {
    /// `"FREE"` o `"PAID"` (`docs/features/plan-tier-quota.md` "Parámetros
    /// Configurables": `TIER_SET`).
    #[serde(default = "default_plan_tier")]
    pub tier: String,
}

fn default_plan_tier() -> String {
    "FREE".to_string()
}

/// Output de la verificación de Plan / Tier / Quota. Siempre serializa a
/// JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `plan_limits_out` -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanTierQuotaVerifyOutput {
    pub ok: bool,
    pub tier: Option<String>,
    pub notional_limit: Option<i64>,
    pub max_activations: Option<i64>,
    /// Cuota de cuentas maestras hijas del plan (STORY-042, #12/#14).
    pub max_child_accounts: Option<i64>,
    pub features_enabled: Option<Vec<String>>,
    /// `true` si el valor devuelto salió de la caché con TTL en vez de una
    /// resolución fresca contra el catálogo (esta llamada de CLI siempre
    /// pasa por ambos pasos: resuelve y luego cachea).
    pub cached: bool,
    pub error: Option<String>,
}

impl PlanTierQuotaVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            tier: None,
            notional_limit: None,
            max_activations: None,
            max_child_accounts: None,
            features_enabled: None,
            cached: false,
            error: Some(msg),
        }
    }

    fn from_limits(tier: plan_tier_quota::PlanTier, limits: plan_tier_quota::PlanLimits) -> Self {
        Self {
            ok: true,
            tier: Some(tier.as_str().to_string()),
            notional_limit: Some(limits.notional_limit),
            max_activations: Some(limits.max_activations),
            max_child_accounts: Some(limits.max_child_accounts),
            features_enabled: Some(limits.features_enabled),
            cached: true,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Plan / Tier / Quota con adaptadores reales
/// (BD SQLite temporal + reloj de sistema real + catálogo de desarrollo
/// stub), recorriendo el camino completo del cimiento #3: siembra el
/// catálogo Free/Paid (si aún no existe en esta BD temporal), resuelve
/// `PlanLimits` para el `tier` pedido, y lo pasa por su caché con TTL antes
/// de reportar -- ejercitando el camino completo Core -> Shell -> puerto
/// que un usuario real recorrería.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify plan-tier-quota --input '{"tier":"FREE"}'`
pub async fn verify_plan_tier_quota(input: PlanTierQuotaVerifyInput) -> PlanTierQuotaVerifyOutput {
    let tier = match plan_tier_quota::PlanTier::from_str_value(&input.tier) {
        Some(tier) => tier,
        None => {
            return PlanTierQuotaVerifyOutput::from_error(format!(
                "tier desconocido: '{}' -- se esperaba FREE o PAID",
                input.tier
            ))
        }
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón que
    // verify_central_identity / verify_licensing_system).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-plan-tier-quota-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return PlanTierQuotaVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return PlanTierQuotaVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return PlanTierQuotaVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- siembra el catálogo de desarrollo (Free + Paid) si esta BD
    // temporal todavía no lo tiene.
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    if let Err(e) = plan_tier_quota::seed_default_catalog(
        &pool,
        clock.as_ref(),
        "drasus-system",
        &node_id,
        "DRASUS_LOCAL_VERIFY",
        &plan_tier_quota::LocalStubPlanCatalogConfig::default(),
    )
    .await
    {
        return PlanTierQuotaVerifyOutput::from_error(format!("fallo al sembrar el catálogo: {e}"));
    }

    // Paso 2 -- resuelve PlanLimits para el tier pedido (fuera del
    // hot-path: esta función SÍ hace lecturas de BD local).
    let limits = match plan_tier_quota::build_plan_limits_for_tier(&pool, clock.as_ref(), tier).await {
        Ok(limits) => limits,
        Err(e) => return PlanTierQuotaVerifyOutput::from_error(format!("fallo al resolver límites: {e}")),
    };

    // Paso 3 -- pasa por la caché con TTL antes de reportar, ejercitando el
    // cableado completo que el hot-path real consultaría.
    let cache = plan_tier_quota::PlanLimitsCache::new(clock, plan_tier_quota::PlanLimitsCacheConfig::default());
    cache.set(tier, limits);

    match cache.get(tier) {
        Some(cached_limits) => PlanTierQuotaVerifyOutput::from_limits(tier, cached_limits),
        // Inalcanzable en la práctica: acabamos de guardar con el TTL por
        // defecto (15 minutos); solo fallaría si el reloj saltara ese
        // tiempo entre `set` y `get`, lo cual no ocurre en una invocación
        // síncrona.
        None => PlanTierQuotaVerifyOutput::from_error(
            "los límites recién guardados ya no están vigentes en la caché (inesperado)".to_string(),
        ),
    }
}

// ── Consent Registry (STORY-031, vive en `shared` -- ver ADR-0137) ─────────

pub use crate::domain::consent_registry::{
    apply_consent_action, compute_consent_audit_hash, needs_reacceptance, parse_optout_map,
    resolve_coverage, ConsentAction, ConsentActionInput, ConsentState, ConsentVerdict,
    NotCoveredReason, OptoutMapError,
};
pub use crate::orchestrator::consent_registry::{record_consent_action, resolve_consent_verdict};
pub use crate::persistence::consent_registry::{
    ConsentRecordRow, ConsentRepository, ConsentRepositoryError, RecordConsentActionInput,
};

/// Una acción de consentimiento de entrada para la verificación vía CLI --
/// espejo de [`ConsentActionInput`] pero con `optout_map` (no
/// `optout_changes`) para que el JSON del usuario sea legible: cada acción
/// trae el mapa de cambios de opt-out que quiere aplicar sobre el estado
/// vigente (`docs/features/consent-registry.md`, STORY-031).
///
/// Uso típico de cada elemento de `actions`:
/// `{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentActionVerifyInput {
    /// `"ACCEPT"`, `"REACCEPT"` u `"OPTOUT_CHANGE"` (mismo vocabulario que
    /// `ConsentAction::as_str`).
    pub action: String,
    /// Versión de ToS que se acepta -- solo relevante para
    /// `ACCEPT`/`REACCEPT`; se omite (o se manda `null`) en
    /// `OPTOUT_CHANGE`.
    #[serde(default)]
    pub tos_version: Option<String>,
    /// Cambios de opt-out a fusionar sobre el estado vigente -- solo las
    /// claves que cambian, el resto del mapa previo se conserva
    /// ([`apply_consent_action`]).
    #[serde(default)]
    pub optout_map: std::collections::BTreeMap<String, bool>,
}

/// La consulta de cobertura a resolver DESPUÉS de aplicar todas las
/// `actions` -- `(data_type, current_version)` (`current_version` viaja a
/// nivel de [`ConsentRegistryVerifyInput`], no aquí, porque es la MISMA
/// versión vigente contra la que se registraron las acciones).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentQueryVerifyInput {
    pub data_type: String,
}

/// Input para la verificación de Consent Registry vía CLI
/// (`docs/features/consent-registry.md`, STORY-031). Se deserializa desde
/// el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify consent-registry --input
/// '{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentRegistryVerifyInput {
    /// La versión de ToS vigente contra la que se evalúan tanto las
    /// acciones registradas como la consulta final.
    pub current_version: String,
    /// Las acciones a registrar, EN ORDEN, contra el mismo dueño -- cada
    /// una se fusiona sobre el snapshot que dejó la anterior.
    pub actions: Vec<ConsentActionVerifyInput>,
    /// La consulta de cobertura a resolver tras registrar todas las
    /// acciones.
    pub query: ConsentQueryVerifyInput,
}

/// Output de la verificación de Consent Registry. Siempre serializa a
/// JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `consent_out` -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentRegistryVerifyOutput {
    pub ok: bool,
    /// `"COVERED"` o `"NOT_COVERED"`.
    pub verdict: Option<String>,
    /// Presente solo si `verdict` es `"NOT_COVERED"`:
    /// `"STALE_VERSION"` | `"OPTED_OUT"` | `"NO_CONSENT"`.
    pub reason: Option<String>,
    /// Cuántas acciones se registraron con éxito antes de resolver la
    /// consulta (o antes de que una fallara).
    pub actions_recorded: usize,
    pub error: Option<String>,
}

impl ConsentRegistryVerifyOutput {
    fn from_error(actions_recorded: usize, msg: String) -> Self {
        Self {
            ok: false,
            verdict: None,
            reason: None,
            actions_recorded,
            error: Some(msg),
        }
    }

    fn from_verdict(verdict: ConsentVerdict, actions_recorded: usize) -> Self {
        let (verdict_str, reason_str) = match &verdict {
            ConsentVerdict::Covered => ("COVERED".to_string(), None),
            ConsentVerdict::NotCovered(reason) => {
                let reason_str = match reason {
                    NotCoveredReason::StaleVersion => "STALE_VERSION",
                    NotCoveredReason::OptedOut => "OPTED_OUT",
                    NotCoveredReason::NoConsent => "NO_CONSENT",
                };
                ("NOT_COVERED".to_string(), Some(reason_str.to_string()))
            }
        };
        Self {
            ok: true,
            verdict: Some(verdict_str),
            reason: reason_str,
            actions_recorded,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Consent Registry con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real), recorriendo el camino
/// completo del cimiento #5: registra CADA acción de `input.actions` EN
/// ORDEN (fusionando sobre el mismo dueño vía [`record_consent_action`]),
/// y resuelve la consulta final vía [`resolve_consent_verdict`] --
/// ejercitando Core -> Shell -> puerto tal como lo recorrería un usuario
/// real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify consent-registry --input
/// '{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}'`
pub async fn verify_consent_registry(input: ConsentRegistryVerifyInput) -> ConsentRegistryVerifyOutput {
    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_usage_metering / verify_plan_tier_quota).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-consent-registry-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return ConsentRegistryVerifyOutput::from_error(
            0,
            format!("no se pudo crear el directorio temporal de verificación: {e}"),
        );
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return ConsentRegistryVerifyOutput::from_error(
                0,
                format!("no se pudo crear la BD temporal de verificación: {e}"),
            )
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return ConsentRegistryVerifyOutput::from_error(
            0,
            format!("error al aplicar migraciones en la BD temporal: {e}"),
        );
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    let owner_id = "verify-consent-registry-owner";

    // Paso 1 -- registra cada acción EN ORDEN, fusionando sobre el mismo
    // dueño (fuera del hot-path: esta función SÍ hace lecturas/escrituras
    // de BD local).
    for (index, action) in input.actions.iter().enumerate() {
        let parsed_action = match ConsentAction::from_str_value(&action.action) {
            Some(a) => a,
            None => {
                return ConsentRegistryVerifyOutput::from_error(
                    index,
                    format!(
                        "acción #{index} desconocida: '{}' -- se esperaba ACCEPT, REACCEPT u OPTOUT_CHANGE",
                        action.action
                    ),
                )
            }
        };

        let result = record_consent_action(
            &pool,
            clock.as_ref(),
            RecordConsentActionInput {
                owner_id: owner_id.to_string(),
                institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
                node_id: node_id.clone(),
                compliance_status_id: None,
                action: parsed_action,
                tos_version: action.tos_version.clone(),
                optout_changes: action.optout_map.clone(),
            },
        )
        .await;

        if let Err(e) = result {
            return ConsentRegistryVerifyOutput::from_error(
                index,
                format!("fallo al registrar la acción #{index}: {e}"),
            );
        }
    }

    // Paso 2 -- resuelve la consulta final tras aplicar todas las acciones.
    let verdict = match resolve_consent_verdict(
        &pool,
        clock.as_ref(),
        owner_id,
        &input.query.data_type,
        &input.current_version,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return ConsentRegistryVerifyOutput::from_error(
                input.actions.len(),
                format!("fallo al resolver el veredicto de consentimiento: {e}"),
            )
        }
    };

    ConsentRegistryVerifyOutput::from_verdict(verdict, input.actions.len())
}

// ── Enriched Domain Events (STORY-033, vive en `shared` -- ver ADR-0137) ────

/// Submódulo público del cimiento #6 (`docs/features/enriched-domain-events.md`,
/// ADR-0144, ADR-0145, STORY-033) -- la raíz del substrato de monetización.
///
/// Expone los dos puertos de la Feature (ADR-0137): `event_out`
/// (`EnrichedDomainEvent` persistido, Output 1..N) y `gate_in`
/// (`ExecutionGate` consumido, Input 1). Vive bajo su propio submódulo en
/// vez de aplanarse a este nivel superior, por simetría con
/// `plan_tier_quota` y para agrupar el catálogo de tipos de evento (que es
/// grande) bajo un espacio de nombres claro.
///
/// **Guardarraíl ADR-0093:** ningún tipo re-exportado aquí modela un
/// secreto -- ni el evento ni su payload pueden portar credenciales de
/// bróker, IPs live o claves de firma (verificado por el test
/// `no_payload_variant_leaks_secret_looking_fields` del Core).
pub mod enriched_domain_events {
    // event_out: el catálogo de eventos del Core + su serialización canónica
    // + el hash encadenado + la decisión de replicación.
    pub use crate::domain::enriched_domain_events::{
        compute_event_audit_hash, decide_replication, AccountSnapshotPayload,
        BacktestCompletedPayload, CapitalFlowPayload, CapitalFlowSign, CorrelationChangePayload,
        DrawdownDetectedPayload, EnrichedDomainEvent, LiquidityStressPayload, OrderExecutedPayload,
        OrderSide, RegimeDetectedPayload,
    };
    // La composición completa (recibe evento + gate real, deriva replicate,
    // persiste append-only atómico).
    pub use crate::orchestrator::enriched_domain_events::{
        record_domain_event, EventEmissionIdentity,
    };
    pub use crate::persistence::enriched_domain_events::{
        DomainEventRepository, DomainEventRepositoryError, DomainEventRow, RecordDomainEventInput,
    };
    // gate_in: el tipo de puerto de entrada -- se consume el ExecutionGate
    // REAL de licensing-system (#2), no un stub.
    pub use crate::domain::licensing_system::ExecutionGate;
}

/// El evento de entrada para la verificación de Enriched Domain Events vía
/// CLI -- un enum etiquetado por `type` que refleja el catálogo del Core
/// (`docs/features/enriched-domain-events.md`, STORY-033). Los enums del
/// Core (`OrderSide`, `CapitalFlowSign`) llegan aquí como `String` para que
/// el JSON del usuario sea legible; se convierten en
/// [`DomainEventVerifyEvent::into_domain_event`].
///
/// Los montos son `i64` escalados ×10⁸ (ADR-0141) -- exactamente los mismos
/// enteros que porta el Core, sin `f64` en ningún punto del camino.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum DomainEventVerifyEvent {
    OrderExecuted {
        instrument_id: String,
        /// `"BUY"` o `"SELL"`.
        side: String,
        quantity: i64,
        price: i64,
        #[serde(default)]
        slippage: i64,
        #[serde(default)]
        fill_time_ns: i64,
        broker: String,
        notional: i64,
        account_id: String,
        #[serde(default)]
        realized_pnl: i64,
        #[serde(default)]
        mae: i64,
        #[serde(default)]
        mfe: i64,
        #[serde(default)]
        duration_ns: i64,
    },
    CapitalFlow {
        account_id: String,
        /// `"DEPOSIT"`, `"WITHDRAWAL"` o `"TRANSFER"`.
        sign: String,
        amount: i64,
        currency: String,
        #[serde(default)]
        timestamp_ns: i64,
    },
    AccountSnapshot {
        account_id: String,
        equity: i64,
        balance: i64,
        margin_available: i64,
        margin_required: i64,
        #[serde(default)]
        timestamp_ns: i64,
    },
    BacktestCompleted {
        sharpe: i64,
        drawdown: i64,
        pbo: i64,
        regime: String,
    },
    RegimeDetected {
        instrument_id: String,
        regime_label: String,
        #[serde(default)]
        timestamp_ns: i64,
    },
    DrawdownDetected {
        account_id: String,
        drawdown_pct: i64,
        #[serde(default)]
        timestamp_ns: i64,
    },
    LiquidityStress {
        instrument_id: String,
        severity: String,
        #[serde(default)]
        timestamp_ns: i64,
    },
    CorrelationChange {
        instrument_a: String,
        instrument_b: String,
        correlation: i64,
        #[serde(default)]
        timestamp_ns: i64,
    },
}

impl DomainEventVerifyEvent {
    /// Convierte la entrada de CLI en el `EnrichedDomainEvent` del Core,
    /// validando los strings de enum (`side`, `sign`). Devuelve `Err(String)`
    /// con un mensaje legible si un enum no es reconocido -- nunca hace
    /// panic sobre input del usuario.
    fn into_domain_event(self) -> Result<enriched_domain_events::EnrichedDomainEvent, String> {
        use enriched_domain_events::{
            AccountSnapshotPayload, BacktestCompletedPayload, CapitalFlowPayload, CapitalFlowSign,
            CorrelationChangePayload, DrawdownDetectedPayload, EnrichedDomainEvent,
            LiquidityStressPayload, OrderExecutedPayload, OrderSide, RegimeDetectedPayload,
        };

        match self {
            DomainEventVerifyEvent::OrderExecuted {
                instrument_id, side, quantity, price, slippage, fill_time_ns, broker, notional,
                account_id, realized_pnl, mae, mfe, duration_ns,
            } => {
                let side = OrderSide::from_str_value(&side)
                    .ok_or_else(|| format!("side desconocido: '{side}' -- se esperaba BUY o SELL"))?;
                Ok(EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
                    instrument_id, side, quantity, price, slippage, fill_time_ns, broker, notional,
                    account_id, realized_pnl, mae, mfe, duration_ns,
                }))
            }
            DomainEventVerifyEvent::CapitalFlow { account_id, sign, amount, currency, timestamp_ns } => {
                let sign = CapitalFlowSign::from_str_value(&sign).ok_or_else(|| {
                    format!("sign desconocido: '{sign}' -- se esperaba DEPOSIT, WITHDRAWAL o TRANSFER")
                })?;
                Ok(EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
                    account_id, sign, amount, currency, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::AccountSnapshot {
                account_id, equity, balance, margin_available, margin_required, timestamp_ns,
            } => Ok(EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
                account_id, equity, balance, margin_available, margin_required, timestamp_ns,
            })),
            DomainEventVerifyEvent::BacktestCompleted { sharpe, drawdown, pbo, regime } => {
                Ok(EnrichedDomainEvent::BacktestCompleted(BacktestCompletedPayload {
                    sharpe, drawdown, pbo, regime,
                }))
            }
            DomainEventVerifyEvent::RegimeDetected { instrument_id, regime_label, timestamp_ns } => {
                Ok(EnrichedDomainEvent::RegimeDetected(RegimeDetectedPayload {
                    instrument_id, regime_label, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::DrawdownDetected { account_id, drawdown_pct, timestamp_ns } => {
                Ok(EnrichedDomainEvent::DrawdownDetected(DrawdownDetectedPayload {
                    account_id, drawdown_pct, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::LiquidityStress { instrument_id, severity, timestamp_ns } => {
                Ok(EnrichedDomainEvent::LiquidityStress(LiquidityStressPayload {
                    instrument_id, severity, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::CorrelationChange { instrument_a, instrument_b, correlation, timestamp_ns } => {
                Ok(EnrichedDomainEvent::CorrelationChange(CorrelationChangePayload {
                    instrument_a, instrument_b, correlation, timestamp_ns,
                }))
            }
        }
    }
}

/// Input para la verificación de Enriched Domain Events vía CLI
/// (`docs/features/enriched-domain-events.md`, STORY-033). Se deserializa
/// desde el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify enriched-domain-events --input
/// '{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichedDomainEventsVerifyInput {
    /// El tier que gobierna la supresión de telemetría (ADR-0143). `"FREE"`
    /// (gratuito -> Explorer, no suprime -> replica) o `"PAID"` (pago al
    /// corriente -> Sovereign, suprime -> no replica). También se aceptan
    /// los nombres de licencia crudos `"EXPLORER"`/`"SOVEREIGN"`.
    #[serde(default = "default_domain_event_tier")]
    pub tier: String,
    /// El evento a construir, persistir y observar.
    pub event: DomainEventVerifyEvent,
}

fn default_domain_event_tier() -> String {
    "FREE".to_string()
}

/// Output de la verificación de Enriched Domain Events. Siempre serializa a
/// JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `event_out` tras persistir el evento -- ningún secreto
/// (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichedDomainEventsVerifyOutput {
    pub ok: bool,
    /// El tier resuelto (`"EXPLORER"` o `"SOVEREIGN"`).
    pub tier: Option<String>,
    /// `true` si el gate real suprime telemetría de trabajo -- espejo del
    /// campo del `ExecutionGate` que gobernó la decisión.
    pub suppress_work_telemetry: Option<bool>,
    /// La decisión derivada: `true` = el evento se replica al proveedor;
    /// `false` = solo local. Es el inverso de `suppress_work_telemetry`.
    pub replicate: Option<bool>,
    pub event_type: Option<String>,
    /// El payload JSON canónico persistido (string, tal cual quedó en la BD).
    pub payload: Option<String>,
    pub event_sequence_id: Option<i64>,
    /// `true` si la fila persistida es la génesis (`audit_chain_hash` NULL).
    pub is_genesis: Option<bool>,
    pub audit_hash: Option<String>,
    pub error: Option<String>,
}

impl EnrichedDomainEventsVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            tier: None,
            suppress_work_telemetry: None,
            replicate: None,
            event_type: None,
            payload: None,
            event_sequence_id: None,
            is_genesis: None,
            audit_hash: None,
            error: Some(msg),
        }
    }
}

/// Traduce el `tier` del input (`FREE`/`PAID`, o los crudos
/// `EXPLORER`/`SOVEREIGN`) al [`LicenseTier`] de `licensing-system`.
/// `FREE` -> `Explorer` (gratuito, nunca suprime), `PAID` -> `Sovereign`
/// (pago al corriente, suprime). Devuelve `None` si no reconoce el valor.
fn resolve_domain_event_tier(value: &str) -> Option<LicenseTier> {
    match value.to_uppercase().as_str() {
        "FREE" | "EXPLORER" => Some(LicenseTier::Explorer),
        "PAID" | "SOVEREIGN" => Some(LicenseTier::Sovereign),
        _ => None,
    }
}

/// Ejecuta la verificación de Enriched Domain Events con adaptadores reales
/// (BD SQLite temporal + reloj de sistema real + el `ExecutionGate` REAL de
/// `licensing-system` #2), recorriendo el camino completo del cimiento #6:
/// construye una licencia de desarrollo firmada para el tier pedido, deriva
/// el `ExecutionGate` real (no un stub), compone el evento del catálogo,
/// deriva `replicate` y lo persiste append-only atómico vía
/// [`enriched_domain_events::record_domain_event`], y reporta la fila
/// persistida -- ejercitando Core -> Shell -> puerto tal como lo recorrería
/// el motor real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify enriched-domain-events --input
/// '{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}'`
pub async fn verify_enriched_domain_events(
    input: EnrichedDomainEventsVerifyInput,
) -> EnrichedDomainEventsVerifyOutput {
    let tier = match resolve_domain_event_tier(&input.tier) {
        Some(tier) => tier,
        None => {
            return EnrichedDomainEventsVerifyOutput::from_error(format!(
                "tier desconocido: '{}' -- se esperaba FREE o PAID (o EXPLORER/SOVEREIGN)",
                input.tier
            ))
        }
    };

    // Construye el evento del Core ANTES de tocar la BD -- así un input mal
    // formado (side/sign inválido) falla rápido y barato.
    let event = match input.event.into_domain_event() {
        Ok(event) => event,
        Err(msg) => return EnrichedDomainEventsVerifyOutput::from_error(msg),
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón que
    // verify_licensing_system / verify_consent_registry).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-enriched-domain-events-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return EnrichedDomainEventsVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return EnrichedDomainEventsVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return EnrichedDomainEventsVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- gate_in: construye el ExecutionGate REAL de #2 (no un stub),
    // recorriendo el mismo camino que verify_licensing_system: vincula una
    // identidad local, emite+activa una licencia de desarrollo firmada para
    // el tier, obtiene PlanLimits del stub y deriva el gate.
    let machine_identifiers = vec![hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string())];
    let identity_verifier =
        crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
    let identity = match identity_verifier
        .verify_identity(crate::orchestrator::central_identity::IdentityVerificationRequest {
            email: "verify-enriched-domain-events@drasus.local".to_string(),
            oauth_provider: None,
            machine_identifiers,
            institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
            access_token_id: None,
        })
        .await
    {
        Ok(identity) => identity,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al vincular identidad: {e}")),
    };

    let issuer = LocalStubLicenseIssuer::new();
    let now_ns = clock.timestamp_ns();
    let signed = issuer.issue_license(IssueLicenseRequest {
        owner_id: identity.owner_id.clone(),
        node_id: identity.node_id.clone(),
        tier,
        issued_at_ns: now_ns,
        heartbeat_expires_at_ns: now_ns + DEFAULT_HEARTBEAT_INTERVAL_NS,
    });

    let license_repo = LicenseRepository::new(&pool, clock.as_ref());
    let license = match license_repo
        .activate(NewLicenseActivation {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            access_token_id: None,
            node_id: identity.node_id.clone(),
            license_id: signed.license_id.clone(),
            process_id: Some(format!("drasus-pid-{}", std::process::id())),
            signature_hash: signed.signature_hex.clone(),
            tier,
            issued_at_ns: signed.issued_at_ns,
            heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
            compliance_status_id: "ACTIVE".to_string(),
        })
        .await
    {
        Ok(license) => license,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al activar licencia: {e}")),
    };

    let plan_limits_provider = LocalStubPlanLimitsProvider::default();
    let plan_limits = plan_limits_provider.plan_limits_for(&identity.owner_id, tier).await;

    let heartbeat_config = HeartbeatConfig::default();
    let gate = match build_execution_gate(
        &pool,
        clock.as_ref(),
        &identity.node_id,
        &license,
        &signed.signature_hex,
        &signed.public_key_hex,
        &heartbeat_config,
        &plan_limits,
    )
    .await
    {
        Ok(gate) => gate,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al construir el gate: {e}")),
    };

    let suppress = gate.suppress_work_telemetry;

    // Paso 2 -- event_out: compone el evento + el gate real, deriva replicate
    // y persiste append-only atómico.
    let identity_for_event = enriched_domain_events::EventEmissionIdentity {
        owner_id: identity.owner_id.clone(),
        institutional_tag: identity.institutional_tag.clone(),
        node_id: identity.node_id.clone(),
        process_id: format!("drasus-pid-{}", std::process::id()),
        session_id: None,
    };

    let row = match enriched_domain_events::record_domain_event(
        &pool,
        clock.as_ref(),
        identity_for_event,
        &gate,
        event,
    )
    .await
    {
        Ok(row) => row,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al persistir el evento: {e}")),
    };

    EnrichedDomainEventsVerifyOutput {
        ok: true,
        tier: Some(tier.as_str().to_string()),
        suppress_work_telemetry: Some(suppress),
        replicate: Some(row.replicate),
        event_type: Some(row.event_type),
        payload: Some(row.payload),
        event_sequence_id: Some(row.event_sequence_id),
        is_genesis: Some(row.audit_chain_hash.is_none()),
        audit_hash: Some(row.audit_hash),
        error: None,
    }
}

// ── Institutional Report Engine (STORY-034, vive en `shared` -- ver ADR-0137) ─

/// Submódulo público del cimiento #7 (`docs/features/institutional-report-engine.md`,
/// ADR-0144, ADR-0101, ADR-0027, STORY-034).
///
/// Expone los dos puertos de la Feature (ADR-0137): `report_out`
/// (`InstitutionalReport` ensamblado + firmado + persistido, Output 1) y
/// `result_in` (entrada mínima de reporte -- placeholder hasta que
/// `BacktestResult`/`RobustnessScore` reales existan, Input 1..N). Vive
/// bajo su propio submódulo en vez de aplanarse a este nivel superior, por
/// simetría con `enriched_domain_events` y `plan_tier_quota`.
///
/// **Guardarraíl ADR-0093:** ningún tipo re-exportado aquí modela un
/// secreto -- ni el reporte ni sus métricas pueden portar credenciales de
/// bróker, IPs live o claves de firma (verificado por el test
/// `assembled_report_json_does_not_leak_secret_looking_fields` del Core).
///
/// **Render Tera diferido (ADR-0101):** este submódulo NO depende de Tera
/// -- Tera no está en el workspace. El "reporte" es la serialización
/// canónica JSON (`report_body`); el render a PDF/HTML es un adaptador
/// posterior sobre este mismo puerto.
pub mod institutional_report_engine {
    // report_out: el ensamblado puro del Core + su serialización canónica +
    // su firma reproducible + el hash de auditoría de la fila del ledger.
    pub use crate::domain::institutional_report_engine::{
        assemble_report, compute_report_audit_hash, compute_report_signature, AssembleReportInput,
        InstitutionalReport, ReportType,
    };
    // La composición completa (lee el reloj, ensambla, firma, persiste
    // append-only atómico).
    pub use crate::orchestrator::institutional_report_engine::{
        generate_report, GenerateReportError, ReportGenerationIdentity,
    };
    pub use crate::persistence::institutional_report_engine::{
        GeneratedReportRepository, GeneratedReportRepositoryError, GeneratedReportRow,
        RecordGeneratedReportInput,
    };
}

/// Input para la verificación de Institutional Report Engine vía CLI
/// (`docs/features/institutional-report-engine.md`, STORY-034). Se
/// deserializa desde el JSON que pasa el usuario con `--input '...'`.
///
/// `metrics` son ENTEROS escalados ×10⁸ (ADR-0141) -- exactamente lo que
/// persiste el Core, sin `f64` en ningún punto del camino.
///
/// Uso típico:
/// `cargo run -p app -- verify institutional-report-engine --input
/// '{"report_type":"VALIDATION","metrics":{"sharpe_e8":150000000,"max_drawdown_e8":-8000000},"source_event_refs":["evt-1","evt-2"]}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstitutionalReportEngineVerifyInput {
    /// `"VALIDATION"`, `"BACKTEST"`, `"EXECUTION"`, `"STRESS_TEST"`,
    /// `"MODEL_VALIDATION"`, `"BACKTEST_CERTIFICATION"` o
    /// `"DRAWDOWN_FORENSICS"` (mismo vocabulario que
    /// `institutional_report_engine::ReportType::as_str`).
    pub report_type: String,
    /// Métricas nombradas del resultado a reportar, `i64` escalados ×10⁸.
    pub metrics: std::collections::BTreeMap<String, i64>,
    /// Referencia de texto libre al resultado fuente (opcional).
    #[serde(default)]
    pub source_result_ref: Option<String>,
    /// Ids de eventos del event-store (#6) / audit-log que este reporte
    /// cita (trazabilidad, ADR-0027).
    #[serde(default)]
    pub source_event_refs: Vec<String>,
}

/// Output de la verificación de Institutional Report Engine. Siempre
/// serializa a JSON válido (ADR-0142). Si `ok` es `true`, refleja
/// EXACTAMENTE lo que expone el puerto `report_out` tras persistir el
/// reporte -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstitutionalReportEngineVerifyOutput {
    pub ok: bool,
    pub report_type: Option<String>,
    /// La firma REPRODUCIBLE del contenido del reporte -- distinta de
    /// `audit_hash` (integridad de la fila del ledger).
    pub signature_hash: Option<String>,
    pub audit_hash: Option<String>,
    pub event_sequence_id: Option<i64>,
    /// `true` si la fila persistida es la génesis (`audit_chain_hash` NULL).
    pub is_genesis: Option<bool>,
    /// El contenido JSON canónico completo del reporte persistido -- el
    /// mismo string que `signature_hash` hashea.
    pub report_body: Option<String>,
    /// Los `source_event_refs` tal cual quedaron persistidos (JSON string).
    pub source_event_refs: Option<String>,
    pub error: Option<String>,
}

impl InstitutionalReportEngineVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            report_type: None,
            signature_hash: None,
            audit_hash: None,
            event_sequence_id: None,
            is_genesis: None,
            report_body: None,
            source_event_refs: None,
            error: Some(msg),
        }
    }

    fn from_row(row: institutional_report_engine::GeneratedReportRow) -> Self {
        Self {
            ok: true,
            report_type: Some(row.report_type),
            signature_hash: Some(row.signature_hash),
            audit_hash: Some(row.audit_hash.clone()),
            event_sequence_id: Some(row.event_sequence_id),
            is_genesis: Some(row.audit_chain_hash.is_none()),
            report_body: Some(row.report_body),
            source_event_refs: Some(row.source_event_refs),
            error: None,
        }
    }
}

/// Ejecuta la verificación de Institutional Report Engine con adaptadores
/// reales (BD SQLite temporal + reloj de sistema real), recorriendo el
/// camino completo del cimiento #7: construye la entrada mínima de reporte
/// desde el JSON del usuario, ensambla el reporte (Core), calcula su firma
/// reproducible y lo persiste append-only atómico vía
/// [`institutional_report_engine::generate_report`] -- ejercitando Core ->
/// Shell -> puerto tal como lo recorrería un adaptador de producto real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify institutional-report-engine --input
/// '{"report_type":"VALIDATION","metrics":{"sharpe_e8":150000000,"max_drawdown_e8":-8000000},"source_event_refs":["evt-1","evt-2"]}'`
pub async fn verify_institutional_report_engine(
    input: InstitutionalReportEngineVerifyInput,
) -> InstitutionalReportEngineVerifyOutput {
    let report_type = match institutional_report_engine::ReportType::from_str_value(&input.report_type) {
        Some(report_type) => report_type,
        None => {
            return InstitutionalReportEngineVerifyOutput::from_error(format!(
                "report_type desconocido: '{}' -- se esperaba VALIDATION, BACKTEST, EXECUTION, \
                 STRESS_TEST, MODEL_VALIDATION, BACKTEST_CERTIFICATION o DRAWDOWN_FORENSICS",
                input.report_type
            ))
        }
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_enriched_domain_events / verify_consent_registry).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-institutional-report-engine-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return InstitutionalReportEngineVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return InstitutionalReportEngineVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return InstitutionalReportEngineVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());

    let identity = institutional_report_engine::ReportGenerationIdentity {
        owner_id: "verify-institutional-report-engine-owner".to_string(),
        institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
        node_id,
        compliance_status_id: None,
    };

    let assemble_input = institutional_report_engine::AssembleReportInput {
        report_type,
        metrics: input.metrics,
        source_result_ref: input.source_result_ref,
        source_event_refs: input.source_event_refs,
        // Sobrescrito dentro de generate_report con el reloj real -- el
        // valor aquí es un placeholder sin efecto.
        generated_at_ns: 0,
    };

    match institutional_report_engine::generate_report(&pool, clock.as_ref(), identity, assemble_input).await {
        Ok(row) => InstitutionalReportEngineVerifyOutput::from_row(row),
        Err(e) => InstitutionalReportEngineVerifyOutput::from_error(format!("fallo al generar el reporte: {e}")),
    }
}

// ── Third-Party API Gateway (STORY-035, vive en `shared` -- ver ADR-0137) ──

/// Submódulo público del cimiento #8 (`docs/features/third-party-api-gateway.md`,
/// ADR-0144, ADR-0142, ADR-0093, STORY-035) -- la puerta de entrada
/// autenticada para terceros.
///
/// Expone los dos puertos de la Feature (ADR-0137): `api_request_in`
/// (`ThirdPartyRequest`, Input `0..N`) y `api_response_out`
/// (`ThirdPartyResponse`, Output `0..N`). Vive bajo su propio submódulo en
/// vez de aplanarse a este nivel superior, por simetría con
/// `enriched_domain_events` y `institutional_report_engine`.
///
/// **Guardarraíl ADR-0093:** ningún tipo re-exportado aquí modela el
/// secreto de la credencial en claro -- el test
/// `third_party_response_json_never_leaks_the_presented_secret` del Core
/// lo verifica sobre un caso concreto.
///
/// **Servidor gRPC/tonic + mTLS + protos diferidos (STORY-035 §8):** este
/// submódulo NO depende de tonic -- tonic no está en el workspace. El
/// Core + el esquema son el contrato; el servidor público es un adaptador
/// posterior sobre estos mismos puertos.
pub mod third_party_api_gateway {
    // api_request_in / api_response_out: el catálogo del Core + las cuatro
    // puertas de decisión + el hash de auditoría encadenado de ambas tablas.
    pub use crate::domain::third_party_api_gateway::{
        authenticate, compute_api_credential_audit_hash, compute_api_usage_audit_hash,
        compute_rate_limit, decide_gateway_outcome, hash_api_credential, is_endpoint_enabled,
        AuthDenialReason, AuthVerdict, CredentialStatus, GatewayOutcome, RateLimitVerdict,
        ThirdPartyRequest, ThirdPartyResponse,
    };
    // La composición completa (autentica, cuenta la ventana, resuelve
    // consent_out REAL de #5, decide y persiste).
    pub use crate::orchestrator::third_party_api_gateway::{
        handle_gateway_request, HandleGatewayRequestError, API_GATEWAY_CONSENT_DATA_TYPE,
    };
    pub use crate::persistence::third_party_api_gateway::{
        ApiCredentialRepository, ApiCredentialRepositoryError, ApiCredentialRow,
        ApiUsageRepository, ApiUsageRepositoryError, ApiUsageRow, NewApiCredential,
        RecordApiUsageInput,
    };
}

/// Input para la verificación de Third-Party API Gateway vía CLI
/// (`docs/features/third-party-api-gateway.md`, STORY-035). Se deserializa
/// desde el JSON que pasa el usuario con `--input '...'`.
///
/// El harness crea una credencial fresca en la BD temporal con el secreto
/// y los límites dados, SIEMBRA `requests_in_window` solicitudes `ALLOWED`
/// previas (para poder demostrar el borde del rate-limit desde la CLI sin
/// tener que invocar el comando repetidas veces) y registra por adelantado
/// un consentimiento `ACCEPT` que cubre el gateway para `consent_version` --
/// así el único factor que decide el desenlace observable es lo que el
/// usuario pasó en `--input`.
///
/// Uso típico:
/// `cargo run -p app -- verify third-party-api-gateway --input
/// '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThirdPartyApiGatewayVerifyInput {
    /// El secreto crudo de la credencial de API -- solo viaja en memoria
    /// para hashearse (ADR-0093); el harness lo usa también para hashear
    /// la credencial que siembra, así que el mismo valor autentica.
    pub credential: String,
    /// El endpoint a invocar -- el harness lo habilita de antemano en la
    /// credencial sembrada.
    pub endpoint: String,
    #[serde(default = "default_rate_limit_per_window")]
    pub rate_limit_per_window: i64,
    /// Cuántas solicitudes `ALLOWED` previas sembrar en la ventana vigente
    /// ANTES de ejercitar la solicitud real -- el vehículo para demostrar
    /// el borde exacto del rate-limit desde la CLI.
    #[serde(default)]
    pub requests_in_window: i64,
    #[serde(default = "default_window_seconds")]
    pub window_seconds: i64,
    #[serde(default = "default_consent_version")]
    pub consent_version: String,
}

fn default_rate_limit_per_window() -> i64 {
    100
}

fn default_window_seconds() -> i64 {
    60
}

fn default_consent_version() -> String {
    "v1".to_string()
}

/// Output de la verificación de Third-Party API Gateway. Siempre serializa
/// a JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `api_response_out` -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThirdPartyApiGatewayVerifyOutput {
    pub ok: bool,
    /// `"ALLOWED"`, `"RATE_LIMITED"` o `"DENIED"`.
    pub outcome: Option<String>,
    /// El endpoint interno al que se delegaría -- `Some(...)` solo cuando
    /// `outcome == "ALLOWED"`.
    pub delegate_to: Option<String>,
    pub denial_reason: Option<String>,
    pub error: Option<String>,
}

impl ThirdPartyApiGatewayVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            outcome: None,
            delegate_to: None,
            denial_reason: None,
            error: Some(msg),
        }
    }

    fn from_response(response: third_party_api_gateway::ThirdPartyResponse) -> Self {
        Self {
            ok: true,
            outcome: Some(response.outcome.as_str().to_string()),
            delegate_to: response.delegate_to,
            denial_reason: response.denial_reason,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Third-Party API Gateway con adaptadores
/// reales (BD SQLite temporal + reloj de sistema real + el `consent_out`
/// REAL de `consent-registry` #5), recorriendo el camino completo del
/// cimiento #8: crea una credencial fresca con el secreto e endpoint
/// pedidos, siembra `requests_in_window` usos previos `ALLOWED`, registra
/// por adelantado el consentimiento que cubre el gateway, y ejercita
/// [`third_party_api_gateway::handle_gateway_request`] -- ejercitando Core
/// -> Shell -> puerto tal como lo recorrería un tercero real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify third-party-api-gateway --input
/// '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}'`
pub async fn verify_third_party_api_gateway(
    input: ThirdPartyApiGatewayVerifyInput,
) -> ThirdPartyApiGatewayVerifyOutput {
    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_consent_registry / verify_enriched_domain_events).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-third-party-api-gateway-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return ThirdPartyApiGatewayVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return ThirdPartyApiGatewayVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return ThirdPartyApiGatewayVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    let owner_id = "verify-third-party-api-gateway-owner";

    // Paso 1 -- siembra una credencial ACTIVA con el secreto e endpoint
    // pedidos (fuera del hot-path: esta función SÍ hace lecturas/escrituras
    // de BD local).
    let credential_repo = third_party_api_gateway::ApiCredentialRepository::new(&pool, clock.as_ref());
    let credential = match credential_repo
        .create(third_party_api_gateway::NewApiCredential {
            owner_id: owner_id.to_string(),
            access_token_id: None,
            node_id: node_id.clone(),
            credential_hash: third_party_api_gateway::hash_api_credential(&input.credential),
            rate_limit_per_window: input.rate_limit_per_window,
            window_seconds: input.window_seconds,
            endpoints_enabled: vec![input.endpoint.clone()],
        })
        .await
    {
        Ok(credential) => credential,
        Err(e) => return ThirdPartyApiGatewayVerifyOutput::from_error(format!("fallo al crear la credencial: {e}")),
    };

    // Paso 2 -- siembra `requests_in_window` usos ALLOWED previos, para que
    // el borde del rate-limit sea observable desde un solo comando de CLI.
    let usage_repo = third_party_api_gateway::ApiUsageRepository::new(&pool, clock.as_ref());
    for _ in 0..input.requests_in_window {
        if let Err(e) = usage_repo
            .record_usage(third_party_api_gateway::RecordApiUsageInput {
                owner_id: owner_id.to_string(),
                access_token_id: None,
                node_id: node_id.clone(),
                credential_id: credential.id.clone(),
                endpoint: input.endpoint.clone(),
                outcome: third_party_api_gateway::GatewayOutcome::Allowed,
            })
            .await
        {
            return ThirdPartyApiGatewayVerifyOutput::from_error(format!("fallo al sembrar uso previo: {e}"));
        }
    }

    // Paso 3 -- registra por adelantado el consentimiento que cubre el
    // gateway para `consent_version` -- consent_out REAL de #5, no un stub.
    let mut optout_changes = std::collections::BTreeMap::new();
    optout_changes.insert(
        third_party_api_gateway::API_GATEWAY_CONSENT_DATA_TYPE.to_string(),
        false,
    );
    if let Err(e) = record_consent_action(
        &pool,
        clock.as_ref(),
        RecordConsentActionInput {
            owner_id: owner_id.to_string(),
            institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
            node_id: node_id.clone(),
            compliance_status_id: None,
            action: ConsentAction::Accept,
            tos_version: Some(input.consent_version.clone()),
            optout_changes,
        },
    )
    .await
    {
        return ThirdPartyApiGatewayVerifyOutput::from_error(format!("fallo al registrar el consentimiento: {e}"));
    }

    // Paso 4 -- ejercita el flujo completo del gateway con el MISMO
    // secreto que se hasheó al crear la credencial.
    match third_party_api_gateway::handle_gateway_request(
        &pool,
        clock.as_ref(),
        &input.credential,
        &input.endpoint,
        &input.consent_version,
    )
    .await
    {
        Ok(response) => ThirdPartyApiGatewayVerifyOutput::from_response(response),
        Err(e) => ThirdPartyApiGatewayVerifyOutput::from_error(format!("fallo al procesar la solicitud: {e}")),
    }
}

// ── Data Anonymization & Aggregation (STORY-036, vive en `shared` -- ver ADR-0137) ──

pub mod data_aggregation {
    // apply_differential_privacy / hash_strategy_topology / meets_k_anonymity
    // / aggregate_index: el Core -- ruido DP con RNG sembrado, hash
    // unidireccional de topología, k-anonimato y el hash de auditoría
    // encadenado.
    pub use crate::domain::data_aggregation::{
        aggregate_index, apply_differential_privacy, compute_aggregate_audit_hash,
        hash_strategy_topology, meets_k_anonymity, AggregatedIndex, Channel, IndexType,
    };
    // La composición completa: gate de consentimiento REAL de #5 evento por
    // evento, separación de canales y persistencia append-only atómica.
    pub use crate::orchestrator::data_aggregation::{
        run_aggregation, AggregationEventInput, AggregationOutcome, AggregationRunConfig,
        DataAggregationError, DATA_AGGREGATION_CONSENT_DATA_TYPE,
    };
    pub use crate::persistence::data_aggregation::{
        AggregatedIndexRepository, AggregatedIndexRepositoryError, AggregatedIndexRow,
        RecordAggregatedIndexInput,
    };
}

/// Un evento candidato de entrada para la verificación vía CLI -- espejo
/// de [`data_aggregation::AggregationEventInput`] pero con el estado de
/// consentimiento a SEMBRAR (`consent`, texto) en vez de un `owner_id` que
/// el usuario tendría que inventar coherente con una BD que no ve
/// (`docs/features/data-aggregation.md`, STORY-036).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataAggregationEventVerifyInput {
    pub metric_e8: i64,
    /// `"COVERED"` (se siembra un ACCEPT sin opt-out) | `"OPTED_OUT"` (se
    /// siembra un ACCEPT con opt-out explícito del tipo de dato de
    /// agregación) | cualquier otro valor / ausente -> `"NO_CONSENT"` (no
    /// se siembra ningún evento de consentimiento para este owner --
    /// resuelve al default-deny real de `consent-registry`).
    #[serde(default = "default_event_consent_state")]
    pub consent: String,
}

fn default_event_consent_state() -> String {
    "COVERED".to_string()
}

/// Input para la verificación de Data Aggregation vía CLI (`docs/features/
/// data-aggregation.md`, STORY-036). Se deserializa desde el JSON que pasa
/// el usuario con `--input '...'`.
///
/// El harness siembra un `owner_id` sintético distinto por cada evento
/// (`verify-data-aggregation-owner-{índice}`) y, según `consent`, registra
/// por adelantado el evento de consentimiento REAL correspondiente
/// (`consent-registry`, #5) -- así el único factor que decide el
/// desenlace observable es lo que el usuario pasó en `--input`, igual que
/// `verify_third_party_api_gateway`.
///
/// Uso típico:
/// `cargo run -p app -- verify data-aggregation --input
/// '{"seed":42,"min_cohort":5,"external_sale_enabled":false,"events":[{"metric_e8":150000000,"consent":"COVERED"}]}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataAggregationVerifyInput {
    #[serde(default = "default_index_type")]
    pub index_type: String,
    #[serde(default = "default_time_window")]
    pub time_window: String,
    #[serde(default = "default_channel")]
    pub channel: String,
    pub min_cohort: i64,
    #[serde(default = "default_noise_level_e8")]
    pub noise_level_e8: i64,
    pub seed: u64,
    #[serde(default = "default_consent_version")]
    pub consent_version: String,
    #[serde(default)]
    pub external_sale_enabled: bool,
    pub events: Vec<DataAggregationEventVerifyInput>,
}

fn default_index_type() -> String {
    "SENTIMENT".to_string()
}

fn default_time_window() -> String {
    "2026-W27".to_string()
}

fn default_channel() -> String {
    "INTERNAL".to_string()
}

fn default_noise_level_e8() -> i64 {
    1_000_000
}

/// Output de la verificación de Data Aggregation. Siempre serializa a
/// JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `aggregate_out` cuando se publica -- ningún dato
/// crudo (ADR-0093/0102).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataAggregationVerifyOutput {
    pub ok: bool,
    /// `"PUBLISHED"`, `"SUPPRESSED_BY_COHORT_SIZE"` o
    /// `"EXTERNAL_CHANNEL_DISABLED"`.
    pub outcome: Option<String>,
    pub index_type: Option<String>,
    pub time_window: Option<String>,
    pub channel: Option<String>,
    pub cohort_size: Option<i64>,
    pub noise_level_e8: Option<i64>,
    pub metric_value_e8: Option<i64>,
    pub error: Option<String>,
}

impl DataAggregationVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            outcome: None,
            index_type: None,
            time_window: None,
            channel: None,
            cohort_size: None,
            noise_level_e8: None,
            metric_value_e8: None,
            error: Some(msg),
        }
    }

    fn from_outcome(outcome: data_aggregation::AggregationOutcome) -> Self {
        match outcome {
            data_aggregation::AggregationOutcome::Published(row) => Self {
                ok: true,
                outcome: Some("PUBLISHED".to_string()),
                index_type: Some(row.index_type.as_str().to_string()),
                time_window: Some(row.time_window.clone()),
                channel: Some(row.channel.as_str().to_string()),
                cohort_size: Some(row.cohort_size),
                noise_level_e8: Some(row.noise_level_e8),
                metric_value_e8: Some(row.metric_value_e8),
                error: None,
            },
            data_aggregation::AggregationOutcome::SuppressedByCohortSize => Self {
                ok: true,
                outcome: Some("SUPPRESSED_BY_COHORT_SIZE".to_string()),
                index_type: None,
                time_window: None,
                channel: None,
                cohort_size: None,
                noise_level_e8: None,
                metric_value_e8: None,
                error: None,
            },
            data_aggregation::AggregationOutcome::ExternalChannelDisabled => Self {
                ok: true,
                outcome: Some("EXTERNAL_CHANNEL_DISABLED".to_string()),
                index_type: None,
                time_window: None,
                channel: None,
                cohort_size: None,
                noise_level_e8: None,
                metric_value_e8: None,
                error: None,
            },
        }
    }
}

/// Ejecuta la verificación de Data Aggregation con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real + el `consent_out` REAL de
/// `consent-registry` #5): siembra, por cada evento de `--input`, un
/// `owner_id` sintético distinto y su consentimiento real correspondiente,
/// y ejercita [`data_aggregation::run_aggregation`] -- Core -> Shell ->
/// puerto, tal como lo recorrería el orquestador de producción.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify data-aggregation --input
/// '{"seed":42,"min_cohort":5,"external_sale_enabled":false,"events":[{"metric_e8":150000000,"consent":"COVERED"}]}'`
pub async fn verify_data_aggregation(input: DataAggregationVerifyInput) -> DataAggregationVerifyOutput {
    let Some(index_type) = data_aggregation::IndexType::from_str_value(&input.index_type) else {
        return DataAggregationVerifyOutput::from_error(format!(
            "index_type no reconocido: '{}'. Valores válidos: SENTIMENT, REGIME, BROKER_FRICTION, CORRELATION",
            input.index_type
        ));
    };
    let Some(channel) = data_aggregation::Channel::from_str_value(&input.channel) else {
        return DataAggregationVerifyOutput::from_error(format!(
            "channel no reconocido: '{}'. Valores válidos: INTERNAL, EXTERNAL",
            input.channel
        ));
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_third_party_api_gateway / verify_consent_registry).
    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-data-aggregation-{}", uuid::Uuid::new_v4()));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return DataAggregationVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return DataAggregationVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return DataAggregationVerifyOutput::from_error(format!("error al aplicar migraciones en la BD temporal: {e}"));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());

    // Paso 1 -- siembra un owner_id sintético distinto por cada evento y,
    // según `consent`, registra por adelantado el consentimiento REAL
    // correspondiente (consent_out de #5, nunca un stub).
    let mut events = Vec::with_capacity(input.events.len());
    for (index, event) in input.events.iter().enumerate() {
        let owner_id = format!("verify-data-aggregation-owner-{index}");

        match event.consent.as_str() {
            "COVERED" => {
                let mut optout_changes = std::collections::BTreeMap::new();
                optout_changes.insert(data_aggregation::DATA_AGGREGATION_CONSENT_DATA_TYPE.to_string(), false);
                if let Err(e) = record_consent_action(
                    &pool,
                    clock.as_ref(),
                    RecordConsentActionInput {
                        owner_id: owner_id.clone(),
                        institutional_tag: default_institutional_tag(),
                        node_id: node_id.clone(),
                        compliance_status_id: None,
                        action: ConsentAction::Accept,
                        tos_version: Some(input.consent_version.clone()),
                        optout_changes,
                    },
                )
                .await
                {
                    return DataAggregationVerifyOutput::from_error(format!(
                        "fallo al registrar el consentimiento COVERED del evento {index}: {e}"
                    ));
                }
            }
            "OPTED_OUT" => {
                let mut optout_changes = std::collections::BTreeMap::new();
                optout_changes.insert(data_aggregation::DATA_AGGREGATION_CONSENT_DATA_TYPE.to_string(), true);
                if let Err(e) = record_consent_action(
                    &pool,
                    clock.as_ref(),
                    RecordConsentActionInput {
                        owner_id: owner_id.clone(),
                        institutional_tag: default_institutional_tag(),
                        node_id: node_id.clone(),
                        compliance_status_id: None,
                        action: ConsentAction::Accept,
                        tos_version: Some(input.consent_version.clone()),
                        optout_changes,
                    },
                )
                .await
                {
                    return DataAggregationVerifyOutput::from_error(format!(
                        "fallo al registrar el consentimiento OPTED_OUT del evento {index}: {e}"
                    ));
                }
            }
            _ => {
                // "NO_CONSENT" o cualquier otro valor: NO se siembra
                // ningún evento -- resuelve a NotCovered(NoConsent), el
                // default-deny real de consent-registry.
            }
        }

        events.push(data_aggregation::AggregationEventInput {
            owner_id,
            metric_e8: event.metric_e8,
            raw_topology: None,
        });
    }

    let config = data_aggregation::AggregationRunConfig {
        index_type,
        time_window: input.time_window.clone(),
        channel,
        min_cohort: input.min_cohort,
        noise_level_e8: input.noise_level_e8,
        seed: input.seed,
        consent_version: input.consent_version.clone(),
        external_sale_enabled: input.external_sale_enabled,
        owner_id: "verify-data-aggregation-aggregator".to_string(),
        institutional_tag: default_institutional_tag(),
        node_id,
        data_snapshot_id: Some(format!("verify-snapshot-{}", uuid::Uuid::new_v4())),
    };

    match data_aggregation::run_aggregation(&pool, clock.as_ref(), &events, &config).await {
        Ok(outcome) => DataAggregationVerifyOutput::from_outcome(outcome),
        Err(e) => DataAggregationVerifyOutput::from_error(format!("fallo al ejecutar la agregación: {e}")),
    }
}

// ── Verified Account Registry (STORY-037, vive en `shared` -- ver ADR-0137) ─

/// Submódulo público del cimiento #10 (`docs/features/verified-account-registry.md`,
/// ADR-0145, STORY-037).
pub mod verified_account_registry {
    // El Core -- enums de cuenta/publicación/ámbito, cálculo puro del
    // track (gain% que EXCLUYE el flujo de capital), firma reproducible,
    // hash de auditoría, gate de publicación puro y los tipos de puerto.
    pub use crate::domain::verified_account_registry::{
        canonical_attestation_scopes_json, compute_track_record, compute_track_record_audit_hash,
        compute_track_record_signature, compute_verified_account_audit_hash,
        decide_publication, decode_attestation_scopes_json, AccountType, AttestationScope,
        AttestationScopeDecodeError, AttestedTrackRecord, CapitalReality, PublicationStatus,
        TrackRecordMetrics, VerifiedAccountRecord, NS_PER_DAY,
    };
    // La composición completa -- registrar (default PRIVATE), calcular y
    // firmar el track por ámbito, y el gate de publicación con el
    // consent_out REAL de #5.
    pub use crate::orchestrator::verified_account_registry::{
        attest_track_record, register_account, request_publication, VerifiedAccountRegistryError,
        VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE,
    };
    pub use crate::persistence::verified_account_registry::{
        AttestedTrackRecordRepository, AttestedTrackRecordRepositoryError, AttestedTrackRecordRow,
        NewVerifiedAccount, VerifiedAccountRepository, VerifiedAccountRepositoryError,
        VerifiedAccountRow,
    };
}

/// La cuenta a registrar, entrada de la verificación vía CLI -- espejo
/// simplificado de [`verified_account_registry::NewVerifiedAccount`] con
/// defaults razonables (`leverage`, `attestation_scopes`) para que un uso
/// típico solo necesite `broker`/`currency`/`account_type`
/// (`docs/features/verified-account-registry.md`, STORY-037).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifiedAccountRegistryAccountVerifyInput {
    pub broker: String,
    #[serde(default = "default_verified_account_leverage")]
    pub leverage: i64,
    pub currency: String,
    /// `"FUNDED"`, `"PROP"` o `"OWN"`.
    pub account_type: String,
    /// Ámbitos de atestación habilitados para la cuenta -- por defecto,
    /// ambos (soberano y read-only) coexisten, tal como describe
    /// ADR-0145.
    #[serde(default = "default_verified_account_scopes")]
    pub attestation_scopes: Vec<String>,
    /// Grupo II obligatorio -- en ESTA feature `institutional_tag` ES el Eje
    /// B (`docs/adr/ADR-0145.md` corregido 2026-07-07, STORY-041/DEBT-016):
    /// `"LIVE"` (default), `"PAPER"`, `"DEMO"` o `"CHALLENGE"` -- ORTOGONAL a
    /// `attestation_scopes` (Eje A). Reemplaza el antiguo campo
    /// `capital_reality` -- las dos tablas de este cimiento reutilizan el
    /// campo Grupo II en vez de una columna nueva (ADR-0144 FIJO). El
    /// default LIVE evita romper invocaciones previas a este retrabajo.
    #[serde(default = "default_verified_account_institutional_tag")]
    pub institutional_tag: String,
    /// Referencia NO SECRETA a una conexión de bróker ya vinculada
    /// (nullable, ADR-0093).
    #[serde(default)]
    pub broker_connection_ref: Option<String>,
}

fn default_verified_account_leverage() -> i64 {
    100
}

fn default_verified_account_scopes() -> Vec<String> {
    vec!["SOVEREIGN".to_string(), "BROKER_READONLY".to_string()]
}

fn default_verified_account_institutional_tag() -> String {
    "LIVE".to_string()
}

/// Un evento de entrada SIMPLIFICADO para la verificación vía CLI -- a
/// diferencia de [`DomainEventVerifyEvent`] (que exige TODOS los campos
/// del catálogo real de #6), este tipo solo pide lo que efectivamente
/// afecta el cálculo del track (`compute_track_record`); el resto de
/// campos obligatorios de `EnrichedDomainEvent` se completan con valores
/// de relleno documentados en [`Self::into_domain_event`]. `day_offset`
/// distribuye los eventos en días de trading distintos (multiplicado por
/// [`verified_account_registry::NS_PER_DAY`]).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum VerifiedAccountRegistryEventVerifyInput {
    CapitalFlow {
        /// `"DEPOSIT"`, `"WITHDRAWAL"` o `"TRANSFER"`.
        sign: String,
        amount_e8: i64,
        #[serde(default)]
        day_offset: i64,
    },
    OrderExecuted {
        pnl_e8: i64,
        #[serde(default)]
        day_offset: i64,
        #[serde(default = "default_verify_order_duration_ns")]
        duration_ns: i64,
    },
    AccountSnapshot {
        equity_e8: i64,
        balance_e8: i64,
        #[serde(default)]
        day_offset: i64,
    },
}

fn default_verify_order_duration_ns() -> i64 {
    3_600_000_000_000 // 1 hora
}

impl VerifiedAccountRegistryEventVerifyInput {
    /// Convierte la entrada simplificada de CLI en el
    /// `EnrichedDomainEvent` real del Core de #6, etiquetando `account_id`
    /// con el id de la cuenta ya registrada -- los campos que el catálogo
    /// real exige pero que esta feature no usa para calcular el track
    /// (instrumento, lado, precio, slippage, notional, MAE, MFE) quedan
    /// con valores de relleno fijos, documentados aquí en vez de dejarlos
    /// implícitos.
    fn into_domain_event(
        self,
        account_id: &str,
        broker: &str,
        currency: &str,
    ) -> Result<enriched_domain_events::EnrichedDomainEvent, String> {
        use enriched_domain_events::{
            AccountSnapshotPayload, CapitalFlowPayload, CapitalFlowSign, EnrichedDomainEvent,
            OrderExecutedPayload, OrderSide,
        };
        use verified_account_registry::NS_PER_DAY;

        match self {
            VerifiedAccountRegistryEventVerifyInput::CapitalFlow { sign, amount_e8, day_offset } => {
                let sign = CapitalFlowSign::from_str_value(&sign).ok_or_else(|| {
                    format!("sign desconocido: '{sign}' -- se esperaba DEPOSIT, WITHDRAWAL o TRANSFER")
                })?;
                Ok(EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
                    account_id: account_id.to_string(),
                    sign,
                    amount: amount_e8,
                    currency: currency.to_string(),
                    timestamp_ns: day_offset * NS_PER_DAY,
                }))
            }
            VerifiedAccountRegistryEventVerifyInput::OrderExecuted { pnl_e8, day_offset, duration_ns } => {
                // Nocional de relleno: cantidad y precio fijos (no entran
                // al cálculo del track -- solo `realized_pnl`,
                // `fill_time_ns` y `duration_ns` importan).
                Ok(EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
                    instrument_id: "VERIFY".to_string(),
                    side: OrderSide::Buy,
                    quantity: 100_000_000,
                    price: 100_000_000_000,
                    slippage: 0,
                    fill_time_ns: day_offset * NS_PER_DAY,
                    broker: broker.to_string(),
                    notional: 100_000_000_000,
                    account_id: account_id.to_string(),
                    realized_pnl: pnl_e8,
                    mae: 0,
                    mfe: 0,
                    duration_ns,
                }))
            }
            VerifiedAccountRegistryEventVerifyInput::AccountSnapshot { equity_e8, balance_e8, day_offset } => {
                Ok(EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
                    account_id: account_id.to_string(),
                    equity: equity_e8,
                    balance: balance_e8,
                    margin_available: balance_e8,
                    margin_required: 0,
                    timestamp_ns: day_offset * NS_PER_DAY,
                }))
            }
        }
    }
}

/// Input para la verificación de Verified Account Registry vía CLI
/// (`docs/features/verified-account-registry.md`, STORY-037). Se
/// deserializa desde el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify verified-account-registry --input
/// '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN"},"consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifiedAccountRegistryVerifyInput {
    pub account: VerifiedAccountRegistryAccountVerifyInput,
    /// El ámbito de atestación bajo el que se calcula y firma el track --
    /// `"SOVEREIGN"` (default) o `"BROKER_READONLY"`.
    #[serde(default = "default_verify_scope")]
    pub scope: String,
    #[serde(default = "default_verify_time_window")]
    pub time_window: String,
    /// `"COVERED"` (siembra un opt-in real de publicación) | `"OPTED_OUT"`
    /// (siembra un opt-out explícito) | cualquier otro valor / ausente ->
    /// `"NO_CONSENT"` (no siembra nada -- resuelve al default-deny real).
    #[serde(default = "default_verify_consent_state")]
    pub consent: String,
    #[serde(default = "default_verify_consent_version")]
    pub consent_version: String,
    /// Si `true`, tras calcular el track intenta publicar la cuenta
    /// (`request_publication`) -- gobernado por `consent` de arriba.
    #[serde(default)]
    pub publish: bool,
    pub events: Vec<VerifiedAccountRegistryEventVerifyInput>,
}

fn default_verify_scope() -> String {
    "SOVEREIGN".to_string()
}

fn default_verify_time_window() -> String {
    "2026-W27".to_string()
}

fn default_verify_consent_state() -> String {
    "COVERED".to_string()
}

fn default_verify_consent_version() -> String {
    "v1".to_string()
}

/// Output de la verificación de Verified Account Registry. Siempre
/// serializa a JSON válido (ADR-0142). Si `ok` es `true`, refleja
/// EXACTAMENTE lo que exponen los puertos `registry_out`/`track_record_out`
/// -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifiedAccountRegistryVerifyOutput {
    pub ok: bool,
    pub verified_account_id: Option<String>,
    /// `"PRIVATE"` o `"PUBLIC"` -- el estado de publicación DESPUÉS de
    /// resolver el gate (si `publish=true`), o el default `PRIVATE` si no
    /// se pidió publicar.
    pub publication_status: Option<String>,
    /// `"SOVEREIGN"` o `"BROKER_READONLY"` -- el ámbito bajo el que se
    /// calculó este track.
    pub scope: Option<String>,
    /// `true` SOLO si `scope == SOVEREIGN` -- la etiqueta "Ejecución
    /// Verificada por Drasus" vs. "Reportado por el Bróker".
    pub is_attested_by_drasus: Option<bool>,
    /// Eje B -- `"LIVE"`, `"PAPER"`, `"DEMO"` o `"CHALLENGE"`. SIEMPRE
    /// presente junto a `scope`/`is_attested_by_drasus` (Eje A), nunca
    /// omitido.
    pub capital_reality: Option<String>,
    /// `true` SOLO si `capital_reality == LIVE` -- la etiqueta "Cuenta LIVE
    /// (capital real)" vs. "Cuenta PAPER/DEMO/CHALLENGE (capital virtual)".
    /// Derivado ÚNICAMENTE del Eje B, independiente de `is_attested_by_drasus`.
    pub is_real_capital: Option<bool>,
    pub gain_pct_e8: Option<i64>,
    pub max_drawdown_e8: Option<i64>,
    pub win_rate_e8: Option<i64>,
    pub trading_days: Option<i64>,
    pub total_realized_pnl_e8: Option<i64>,
    pub total_deposits_e8: Option<i64>,
    pub total_withdrawals_e8: Option<i64>,
    pub signature_hash: Option<String>,
    pub error: Option<String>,
}

impl VerifiedAccountRegistryVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            verified_account_id: None,
            publication_status: None,
            scope: None,
            is_attested_by_drasus: None,
            capital_reality: None,
            is_real_capital: None,
            gain_pct_e8: None,
            max_drawdown_e8: None,
            win_rate_e8: None,
            trading_days: None,
            total_realized_pnl_e8: None,
            total_deposits_e8: None,
            total_withdrawals_e8: None,
            signature_hash: None,
            error: Some(msg),
        }
    }

    fn from_result(
        account: &verified_account_registry::VerifiedAccountRow,
        track: &verified_account_registry::AttestedTrackRecordRow,
    ) -> Self {
        let projected = verified_account_registry::AttestedTrackRecord::from(track);
        Self {
            ok: true,
            verified_account_id: Some(account.id.clone()),
            publication_status: Some(account.publication_status.as_str().to_string()),
            scope: Some(projected.scope.clone()),
            is_attested_by_drasus: Some(projected.is_attested_by_drasus),
            capital_reality: Some(projected.capital_reality.clone()),
            is_real_capital: Some(projected.is_real_capital),
            gain_pct_e8: Some(projected.gain_pct_e8),
            max_drawdown_e8: Some(projected.max_drawdown_e8),
            win_rate_e8: Some(projected.win_rate_e8),
            trading_days: Some(projected.trading_days),
            total_realized_pnl_e8: Some(projected.total_realized_pnl_e8),
            total_deposits_e8: Some(projected.total_deposits_e8),
            total_withdrawals_e8: Some(projected.total_withdrawals_e8),
            signature_hash: Some(projected.signature_hash),
            error: None,
        }
    }
}

/// Ejecuta la verificación de Verified Account Registry con adaptadores
/// reales (BD SQLite temporal + reloj de sistema real + el `consent_out`
/// REAL de `consent-registry` #5), recorriendo el camino completo del
/// cimiento #10: registra la cuenta (default PRIVATE), construye los
/// eventos de #6 a partir de `input.events`, calcula y firma el track para
/// el ámbito pedido, siembra (según `input.consent`) el consentimiento
/// real de publicación, y si `input.publish` lo pide, resuelve el gate de
/// publicación -- ejercitando Core -> Shell -> puerto tal como lo
/// recorrería el motor real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify verified-account-registry --input
/// '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN"},"consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}'`
pub async fn verify_verified_account_registry(
    input: VerifiedAccountRegistryVerifyInput,
) -> VerifiedAccountRegistryVerifyOutput {
    let Some(account_type) = verified_account_registry::AccountType::from_str_value(&input.account.account_type)
    else {
        return VerifiedAccountRegistryVerifyOutput::from_error(format!(
            "account_type no reconocido: '{}'. Valores válidos: FUNDED, PROP, OWN",
            input.account.account_type
        ));
    };
    let Some(scope) = verified_account_registry::AttestationScope::from_str_value(&input.scope) else {
        return VerifiedAccountRegistryVerifyOutput::from_error(format!(
            "scope no reconocido: '{}'. Valores válidos: SOVEREIGN, BROKER_READONLY",
            input.scope
        ));
    };
    // Valida el Eje B ANTES de tocar la BD -- en esta feature
    // `institutional_tag` ES el Eje B (STORY-041), no un campo separado.
    if verified_account_registry::CapitalReality::from_str_value(&input.account.institutional_tag).is_none() {
        return VerifiedAccountRegistryVerifyOutput::from_error(format!(
            "institutional_tag no reconocido: '{}'. Valores válidos: LIVE, PAPER, DEMO, CHALLENGE",
            input.account.institutional_tag
        ));
    }
    let mut attestation_scopes = Vec::with_capacity(input.account.attestation_scopes.len());
    for raw_scope in &input.account.attestation_scopes {
        match verified_account_registry::AttestationScope::from_str_value(raw_scope) {
            Some(parsed) => attestation_scopes.push(parsed),
            None => {
                return VerifiedAccountRegistryVerifyOutput::from_error(format!(
                    "attestation_scopes con valor no reconocido: '{raw_scope}'"
                ))
            }
        }
    }

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_data_aggregation / verify_third_party_api_gateway).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-verified-account-registry-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return VerifiedAccountRegistryVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return VerifiedAccountRegistryVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return VerifiedAccountRegistryVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    let owner_id = "verify-verified-account-registry-owner".to_string();

    // Paso 1 -- registra la cuenta (default PRIVATE, estructural).
    let account = match verified_account_registry::register_account(
        &pool,
        clock.as_ref(),
        verified_account_registry::NewVerifiedAccount {
            owner_id: owner_id.clone(),
            // Eje B, STORY-041: viene del input del CLI (`account.institutional_tag`,
            // default LIVE, ya validado arriba), no del placeholder genérico
            // `default_institutional_tag()` que usan las demás tablas.
            institutional_tag: input.account.institutional_tag.clone(),
            node_id: node_id.clone(),
            broker: input.account.broker.clone(),
            leverage: input.account.leverage,
            currency: input.account.currency.clone(),
            account_type,
            attestation_scopes,
            broker_connection_ref: input.account.broker_connection_ref.clone(),
        },
    )
    .await
    {
        Ok(account) => account,
        Err(e) => return VerifiedAccountRegistryVerifyOutput::from_error(format!("fallo al registrar la cuenta: {e}")),
    };

    // Paso 2 -- construye los eventos de #6 a partir del input simplificado.
    let mut events = Vec::with_capacity(input.events.len());
    for event_input in input.events {
        match event_input.into_domain_event(&account.id, &account.broker, &account.currency) {
            Ok(event) => events.push(event),
            Err(msg) => return VerifiedAccountRegistryVerifyOutput::from_error(msg),
        }
    }

    // Paso 3 -- calcula y firma el track para el ámbito pedido, y lo
    // persiste append-only atómico.
    let track = match verified_account_registry::attest_track_record(
        &pool,
        clock.as_ref(),
        &account,
        scope,
        &input.time_window,
        &events,
    )
    .await
    {
        Ok(track) => track,
        Err(e) => return VerifiedAccountRegistryVerifyOutput::from_error(format!("fallo al calcular el track: {e}")),
    };

    // Paso 4 -- siembra (según input.consent) el consentimiento REAL de
    // publicación de #5 -- nunca un stub (mismo patrón que
    // verify_data_aggregation).
    let mut optout_changes = std::collections::BTreeMap::new();
    match input.consent.as_str() {
        "COVERED" => {
            optout_changes.insert(
                verified_account_registry::VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE.to_string(),
                false,
            );
            if let Err(e) = record_consent_action(
                &pool,
                clock.as_ref(),
                RecordConsentActionInput {
                    owner_id: owner_id.clone(),
                    institutional_tag: default_institutional_tag(),
                    node_id: node_id.clone(),
                    compliance_status_id: None,
                    action: ConsentAction::Accept,
                    tos_version: Some(input.consent_version.clone()),
                    optout_changes,
                },
            )
            .await
            {
                return VerifiedAccountRegistryVerifyOutput::from_error(format!(
                    "fallo al registrar el consentimiento COVERED: {e}"
                ));
            }
        }
        "OPTED_OUT" => {
            optout_changes.insert(
                verified_account_registry::VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE.to_string(),
                true,
            );
            if let Err(e) = record_consent_action(
                &pool,
                clock.as_ref(),
                RecordConsentActionInput {
                    owner_id: owner_id.clone(),
                    institutional_tag: default_institutional_tag(),
                    node_id: node_id.clone(),
                    compliance_status_id: None,
                    action: ConsentAction::Accept,
                    tos_version: Some(input.consent_version.clone()),
                    optout_changes,
                },
            )
            .await
            {
                return VerifiedAccountRegistryVerifyOutput::from_error(format!(
                    "fallo al registrar el consentimiento OPTED_OUT: {e}"
                ));
            }
        }
        _ => {
            // "NO_CONSENT" o cualquier otro valor: no siembra nada --
            // resuelve al default-deny real de consent-registry.
        }
    }

    // Paso 5 -- si se pidió publicar, resuelve el gate real de #5.
    let account = if input.publish {
        match verified_account_registry::request_publication(
            &pool,
            clock.as_ref(),
            &account,
            verified_account_registry::PublicationStatus::Public,
            &input.consent_version,
        )
        .await
        {
            Ok(account) => account,
            Err(e) => {
                return VerifiedAccountRegistryVerifyOutput::from_error(format!(
                    "fallo al resolver la publicación: {e}"
                ))
            }
        }
    } else {
        account
    };

    VerifiedAccountRegistryVerifyOutput::from_result(&account, &track)
}

// ── Instance Continuity (STORY-039, vive en `shared` -- ver ADR-0137) ──────

/// Submódulo público del cimiento #11 (`docs/features/instance-continuity.md`,
/// ADR-0146, ADR-0093, ADR-0143, STORY-039).
///
/// Expone los tres puertos de la Feature (ADR-0137): `identity_in`
/// (`AccountIdentity`, el tipo REAL de #1 -- ya aplanado en este mismo
/// archivo, no se duplica aquí), `backup_blob_out` (`EncryptedBackupBlob`,
/// Output `0..1`) y `custody_status_out` (`CustodyStatusOut`, Output `1`).
///
/// **Guardarraíl ADR-0093:** ningún tipo re-exportado aquí modela la clave
/// de cifrado, el secreto maestro, credenciales de bróker ni IPs live --
/// verificado por los tests `encrypted_backup_blob_json_never_leaks_key_or_secret`
/// y `custody_status_out_json_never_leaks_secrets` del Core.
///
/// **NO acopla `licensing-system` (ADR-0137):** `custody_status_out` lo
/// CONSUMIRÁ `licensing-system` (#2) más adelante -- es un consumidor
/// downstream, no una dependencia; este submódulo no importa
/// `licensing-system`.
///
/// **Adaptador de almacén de objetos + liberación forzada central + UI
/// diferidos (STORY-039 §8):** este submódulo NO sube el blob a ningún
/// servidor -- el Core + el esquema son el contrato; el transporte real es
/// un adaptador posterior sobre este mismo puerto.
pub mod instance_continuity {
    // Core: KDF, cifrado/descifrado autenticado, filtro del delta,
    // decisión pura de titularidad y hash de auditoría encadenado de
    // ambas tablas.
    pub use crate::domain::instance_continuity::{
        canonical_delta_bytes, compute_backup_audit_hash, compute_backup_delta,
        compute_custody_audit_hash, decide_custody_claim, decrypt_backup_blob,
        derive_encryption_key, encrypt_backup_blob, generate_nonce, is_current_titular,
        BackupField, CustodyClaimError, CustodyState, CustodyStatusOut, EncryptedBackupBlob,
        EncryptionError, HexDecodeError,
    };
    // La composición completa (filtra secretos, cifra, persiste append-only
    // atómico; reclama/consulta el gate de titularidad).
    pub use crate::orchestrator::instance_continuity::{
        claim_custody, is_titular, round_trip_decrypts_to, take_encrypted_snapshot,
        BackupSnapshotError, BackupSnapshotResult, ClaimCustodyError, InstanceContinuityIdentity,
    };
    pub use crate::persistence::instance_continuity::{
        BackupRegistryRepository, BackupRegistryRepositoryError, ClaimTitularInput,
        CustodyRepository, CustodyRepositoryError, CustodyRow, InstanceBackupRow,
        RecordBackupInput,
    };
}

/// El submódulo `master_account_hierarchy` -- cimiento #12, ÚLTIMO del
/// substrato de monetización (`docs/features/master-account-hierarchy.md`,
/// ADR-0147, STORY-040). Re-exporta Core (gate de autorización, "eliminar =
/// archivar", hashes de auditoría de ambas tablas), la composición completa
/// (vincular jerarquía, emitir/recibir override) y los repositorios.
///
/// **Adaptador de red del relé genérico + UI diferidos (STORY-040 §11):**
/// este submódulo NO transmite el comando cifrado a ninguna máquina remota
/// -- el Core + el esquema son el contrato; el transporte real es un
/// adaptador posterior sobre este mismo puerto.
pub mod master_account_hierarchy {
    // Core: catálogo de comandos/lados/etiquetas, gate de autorización,
    // efecto local "eliminar = archivar" y hash de auditoría encadenado de
    // ambas tablas.
    pub use crate::domain::master_account_hierarchy::{
        apply_local_command_effect, compute_hierarchy_audit_hash, compute_override_audit_hash,
        decide_override_authorization, AttestationSide, LocalEffect, OverrideCommandKind,
        OverrideOutcome, OverrideOutcomeLabel,
    };
    // La composición completa (vincular jerarquía; emitir/recibir override,
    // con el consent_out REAL de #5 resuelto por cada lado).
    pub use crate::orchestrator::master_account_hierarchy::{
        execute_override, issue_override, link_child_to_parent, receive_override,
        MasterAccountHierarchyError, OverrideExecutionResult, MASTER_ACCOUNT_OVERRIDE_CONSENT_DATA_TYPE,
    };
    pub use crate::persistence::master_account_hierarchy::{
        AccountHierarchyRepository, AccountHierarchyRepositoryError, AccountHierarchyRow,
        NewAccountHierarchy, OverrideAttestationRepository, OverrideAttestationRepositoryError,
        OverrideAttestationRow, RecordOverrideAttestationInput,
    };
}

/// El submódulo `data_portability` -- cimiento #13
/// (`docs/features/data-portability.md`, ADR-0148, STORY-043). Re-exporta
/// Core (vocabulario de tipo/estado, decisión de disposición del olvido,
/// filtro de secretos, manifiesto de exportación, hashes de auditoría de
/// ambas tablas), la composición completa (declarar/sembrar el catálogo,
/// pedir export/olvido) y los repositorios.
///
/// **Generador de archivo real + UI diferidos (STORY-043 §1/§11):** este
/// submódulo NO recorre el esquema real ni vuelca ningún dato de otra
/// tabla -- el Core + el esquema son el contrato; el recorrido real es un
/// adaptador posterior sobre este mismo puerto.
pub mod data_portability {
    // Core: vocabulario cerrado (tipo/estado de solicitud), decisión de
    // disposición del olvido, filtro de secretos, manifiesto de
    // exportación y hashes de auditoría encadenados de ambas tablas.
    pub use crate::domain::data_portability::{
        build_export_manifest, build_forget_disposition_detail, compute_catalog_audit_hash,
        compute_request_audit_hash, decide_forget_disposition, disposition_detail_to_json,
        is_excluded_from_export, CatalogEntry, ExportManifest, ForgetDisposition,
        ManifestTableEntry, RequestStatus, RequestType, TableDispositionEntry,
    };
    // La composición completa (declarar/sembrar el catálogo; pedir
    // export/olvido, armando el manifiesto/detalle vía el Core y
    // persistiendo el evento append-only atómico).
    pub use crate::orchestrator::data_portability::{
        declare_exportable_table, request_export, request_forget, seed_known_catalog,
        DataPortabilityError, DataPortabilityIdentity, ExportRequestResult, ForgetRequestResult,
    };
    pub use crate::persistence::data_portability::{
        DataPortabilityRequestRepository, DataPortabilityRequestRepositoryError,
        DataPortabilityRequestRow, ExportableDataCatalogRepository,
        ExportableDataCatalogRepositoryError, ExportableDataCatalogRow, NewCatalogEntry,
        RecordDataPortabilityRequestInput,
    };
}

/// El submódulo `operator_roles` -- cimiento #14 y ÚLTIMO del substrato de
/// monetización (`docs/features/operator-roles.md`, ADR-0149, STORY-044).
/// Re-exporta Core (matriz de capacidades, gate compuesto rol+pipeline,
/// invariante "último admin en pie", gate de cuota de cuentas hijas, hashes
/// de auditoría de las tres tablas), la composición completa (definir/
/// reclasificar/revocar roles, asignar/revocar operadores, evaluar
/// llamadas, sembrar el primer admin, cascada de autoridad) y los
/// repositorios.
///
/// **Transporte de red de la cascada + UI diferidos (STORY-044 §1/§6):**
/// este submódulo modela SOLO la decisión/registro LOCAL de un override de
/// asignación -- el relé cifrado (ADR-0143) y la doble atestación
/// cross-máquina completa de `master-account-hierarchy` (#12) son un
/// adaptador posterior sobre este mismo puerto.
pub mod operator_roles {
    // Core: tipo de operador, baja lógica, catálogo de cambios del ledger,
    // matriz de capacidades (BTreeMap ordenado), gate de rol puntual, gate
    // compuesto (rol AND mcp_gateway::evaluate_permission), invariante
    // "último admin en pie" y gate de cuota de cuentas hijas, hashes de
    // auditoría de las tres tablas.
    pub use crate::domain::operator_roles::{
        admins_remaining_after, can_create_child_account, check_last_admin_standing,
        compute_assignment_audit_hash, compute_event_audit_hash, compute_role_audit_hash,
        evaluate_operator_call, evaluate_role_capability, AssignmentView, CapabilityMatrix,
        ChildAccountVerdict, CombinedVerdict, LastAdminViolation, LifecycleStatus,
        OperatorRoleChangeType, OperatorType, ProposedChange, RoleVerdict, RoleView,
        CAPABILITY_CREATE_CHILD_ACCOUNT, CAPABILITY_MANAGE_ROLES,
    };
    // La composición completa (definir/reclasificar/revocar roles, asignar/
    // revocar operadores, evaluar llamadas, cuota de cuentas hijas, primer
    // admin por defecto, cascada de autoridad).
    pub use crate::orchestrator::operator_roles::{
        apply_authority_override, assign_operator, define_role, evaluate_call, request_child_account,
        revoke_assignment, revoke_role, seed_admin_bootstrap, update_role_matrix, EvaluateCallResult,
        OperatorRolesIdentity,
    };
    pub use crate::persistence::operator_roles::{
        NewOperatorRole, OperatorAssignmentRepository, OperatorAssignmentRow, OperatorRoleError,
        OperatorRoleEventRepository, OperatorRoleEventRow, OperatorRoleRepository, OperatorRoleRow,
        RecordOperatorRoleEventInput, SetAssignmentInput,
    };
}

/// El estado de custodia PREVIO al reclamo, tal como lo asume el harness de
/// verificación CLI -- simula "esta cuenta ya tenía un historial de
/// custodia" sin tener que recorrer todos los reclamos intermedios (ver
/// [`instance_continuity::CustodyRepository::seed_initial_state`]).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustodyVerifyInput {
    pub titular_node_id: String,
    pub custody_epoch: i64,
}

/// Input para la verificación de Instance Continuity vía CLI
/// (`docs/features/instance-continuity.md`, STORY-039). Se deserializa
/// desde el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify instance-continuity --input
/// '{"master_secret":"correct horse battery staple","plaintext":"snapshot-bytes","nonce_seed":42,"custody":{"titular_node_id":"node-A","custody_epoch":3},"my_node_id":"node-A"}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstanceContinuityVerifyInput {
    /// El secreto maestro del usuario -- SOLO viaja en memoria para
    /// derivar la clave de cifrado (ADR-0093); NUNCA aparece en el output.
    pub master_secret: String,
    /// El contenido a cifrar/descifrar, en texto plano.
    pub plaintext: String,
    /// La semilla del nonce de AES-GCM -- determinista en esta
    /// verificación (mismo patrón sembrado que el cimiento #9).
    pub nonce_seed: u64,
    /// El estado de custodia PREVIO al reclamo -- se siembra en la BD
    /// temporal antes de ejercitar el reclamo real de esta verificación.
    pub custody: CustodyVerifyInput,
    /// La máquina que ejecuta esta verificación -- la que cifra el
    /// snapshot y la que intenta reclamar la titularidad. Este mismo valor
    /// se usa como identificador de hardware para resolver `identity_in`
    /// (ver `Paso 0` de [`verify_instance_continuity`]) -- NO se acepta un
    /// `owner_id` suelto por separado: el `owner_id` real sale de
    /// `central-identity` (#1), nunca se inventa en este harness.
    pub my_node_id: String,
}

/// Output de la verificación de Instance Continuity. Siempre serializa a
/// JSON válido (ADR-0142). **Ningún campo porta la clave de cifrado, el
/// secreto maestro, credenciales de bróker ni IPs live** (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstanceContinuityVerifyOutput {
    pub ok: bool,
    pub owner_id: Option<String>,
    pub node_id: Option<String>,
    /// `true` si `decrypt(encrypt(plaintext)) == plaintext` -- el
    /// round-trip de cifrado autenticado AES-256-GCM.
    pub round_trip_ok: Option<bool>,
    /// El nonce usado, en hex -- NO es secreto (ADR-0002).
    pub nonce_hex: Option<String>,
    pub blob_hash: Option<String>,
    pub blob_size_bytes: Option<i64>,
    pub event_sequence_id: Option<i64>,
    /// `true` si `my_node_id` YA era la titular vigente ANTES de intentar
    /// el reclamo de esta verificación.
    pub is_titular_before_claim: Option<bool>,
    /// `"CLAIMED"` (el reclamo ganó) o `"CONFLICT"` (otra máquina ya había
    /// avanzado el epoch -- el gate de titularidad exclusiva bloqueó el
    /// reclamo, regla obligatoria #4 de ADR-0146).
    pub custody_claim_outcome: Option<String>,
    pub custody_epoch_after: Option<i64>,
    pub titular_node_id_after: Option<String>,
    pub error: Option<String>,
}

impl InstanceContinuityVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            owner_id: None,
            node_id: None,
            round_trip_ok: None,
            nonce_hex: None,
            blob_hash: None,
            blob_size_bytes: None,
            event_sequence_id: None,
            is_titular_before_claim: None,
            custody_claim_outcome: None,
            custody_epoch_after: None,
            titular_node_id_after: None,
            error: Some(msg),
        }
    }
}

/// Ejecuta la verificación de Instance Continuity con adaptadores reales
/// (BD SQLite temporal + reloj de sistema real + el `AccountIdentity` REAL
/// de `central-identity`, #1 -- NO un placeholder), recorriendo el camino
/// completo del cimiento #11: resuelve `identity_in` vinculando una cuenta
/// local para `my_node_id`, cifra el `plaintext` con la clave derivada del
/// `master_secret` (KDF Argon2id) y el nonce sembrado, registra el
/// respaldo append-only atómico, descifra de vuelta para demostrar el
/// round-trip autenticado, siembra el estado de custodia PREVIO dado por
/// `input.custody`, y ejercita el reclamo REAL de titularidad para
/// `my_node_id` -- ejercitando Core -> Shell -> puerto tal como lo
/// recorrería la app real al cerrar/abrir sesión en una máquina.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify instance-continuity --input
/// '{"master_secret":"correct horse battery staple","plaintext":"snapshot-bytes","nonce_seed":42,"custody":{"titular_node_id":"node-A","custody_epoch":3},"my_node_id":"node-A"}'`
pub async fn verify_instance_continuity(input: InstanceContinuityVerifyInput) -> InstanceContinuityVerifyOutput {
    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_verified_account_registry / verify_enriched_domain_events).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-instance-continuity-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return InstanceContinuityVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return InstanceContinuityVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return InstanceContinuityVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 0 -- identity_in: vincula/crea la AccountIdentity local REAL vía
    // central-identity (#1) para esta máquina -- mismo camino que
    // verify_licensing_system/verify_enriched_domain_events. `owner_id` e
    // `institutional_tag` SIEMPRE salen de aquí, nunca se inventan sueltos.
    let identity_verifier =
        crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
    let account_identity = match identity_verifier
        .verify_identity(crate::orchestrator::central_identity::IdentityVerificationRequest {
            email: format!("verify-instance-continuity-{}@drasus.local", input.my_node_id),
            oauth_provider: None,
            machine_identifiers: vec![input.my_node_id.clone()],
            institutional_tag: default_institutional_tag(),
            access_token_id: None,
        })
        .await
    {
        Ok(identity) => identity,
        Err(e) => {
            return InstanceContinuityVerifyOutput::from_error(format!("fallo al vincular identidad: {e}"))
        }
    };

    let identity = instance_continuity::InstanceContinuityIdentity {
        owner_id: account_identity.owner_id.clone(),
        institutional_tag: account_identity.institutional_tag.clone(),
        node_id: input.my_node_id.clone(),
    };

    // Paso 1 -- backup_blob_out: cifra el plaintext (envuelto en un único
    // campo de delta -- el harness no ejercita el filtro de secretos por
    // separado, eso lo cubren los tests unitarios del Core) y registra el
    // respaldo append-only atómico.
    let raw_fields =
        vec![instance_continuity::BackupField { key: "snapshot".to_string(), value: input.plaintext.clone() }];
    let snapshot = match instance_continuity::take_encrypted_snapshot(
        &pool,
        clock.as_ref(),
        &identity,
        &input.master_secret,
        &raw_fields,
        input.nonce_seed,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            return InstanceContinuityVerifyOutput::from_error(format!("fallo al tomar el snapshot cifrado: {e}"))
        }
    };

    // Round-trip: descifra de vuelta con la MISMA clave (re-derivada, la
    // Shell nunca guarda la clave entre llamadas) y compara contra el
    // delta filtrado que se cifró.
    let expected_plaintext =
        instance_continuity::canonical_delta_bytes(&instance_continuity::compute_backup_delta(&raw_fields));
    let round_trip_ok = match instance_continuity::round_trip_decrypts_to(
        &snapshot.blob,
        &input.master_secret,
        &identity.owner_id,
        &expected_plaintext,
    ) {
        Ok(ok) => ok,
        Err(e) => return InstanceContinuityVerifyOutput::from_error(format!("fallo al descifrar el round-trip: {e}")),
    };

    // Paso 2 -- custody_status_out: siembra el estado PREVIO dado por
    // input.custody (simula "esta cuenta ya existía") y ejercita el
    // reclamo real de titularidad para my_node_id.
    let custody_repo =
        crate::persistence::instance_continuity::CustodyRepository::new(&pool, clock.as_ref());
    if let Err(e) = custody_repo
        .seed_initial_state(
            &identity.owner_id,
            &identity.institutional_tag,
            &input.custody.titular_node_id,
            input.custody.custody_epoch,
        )
        .await
    {
        return InstanceContinuityVerifyOutput::from_error(format!("fallo al sembrar el estado de custodia: {e}"));
    }

    let is_titular_before_claim =
        match instance_continuity::is_titular(&pool, clock.as_ref(), &identity.owner_id, &input.my_node_id).await {
            Ok(v) => v,
            Err(e) => {
                return InstanceContinuityVerifyOutput::from_error(format!("fallo al consultar la titularidad: {e}"))
            }
        };

    let (custody_claim_outcome, custody_epoch_after, titular_node_id_after) = match instance_continuity::claim_custody(
        &pool,
        clock.as_ref(),
        &identity,
        input.custody.custody_epoch,
    )
    .await
    {
        Ok(row) => ("CLAIMED".to_string(), Some(row.custody_epoch), Some(row.titular_node_id)),
        Err(instance_continuity::ClaimCustodyError::Repository(
            instance_continuity::CustodyRepositoryError::CustodyConflict { .. },
        )) => ("CONFLICT".to_string(), None, None),
        Err(e) => {
            return InstanceContinuityVerifyOutput::from_error(format!("fallo al reclamar la titularidad: {e}"))
        }
    };

    InstanceContinuityVerifyOutput {
        ok: true,
        owner_id: Some(identity.owner_id),
        node_id: Some(input.my_node_id),
        round_trip_ok: Some(round_trip_ok),
        nonce_hex: Some(snapshot.blob.nonce_hex),
        blob_hash: Some(snapshot.row.blob_hash),
        blob_size_bytes: Some(snapshot.row.blob_size_bytes),
        event_sequence_id: Some(snapshot.row.event_sequence_id),
        is_titular_before_claim: Some(is_titular_before_claim),
        custody_claim_outcome: Some(custody_claim_outcome),
        custody_epoch_after,
        titular_node_id_after,
        error: None,
    }
}

// ────────────────────────────────────────────────────────────────────────
// Master Account Hierarchy (STORY-040, cimiento #12 -- vive en `shared`,
// ver ADR-0137)
// ────────────────────────────────────────────────────────────────────────

/// Input para la verificación de Master Account Hierarchy vía CLI
/// (`docs/features/master-account-hierarchy.md`, STORY-040). Se
/// deserializa desde el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify master-account-hierarchy --input
/// '{"parent_owner_id":"fund-X","child_owner_id":"trader-7","node_id":"node-A","consent":"COVERED","command_kind":"ARCHIVE","target_ref":"strategy-42","justification":"riesgo excedido"}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MasterAccountHierarchyVerifyInput {
    /// El fondo -- la cuenta maestra raíz que emite el override.
    pub parent_owner_id: String,
    /// La hija -- la cuenta maestra sobre la que el override actúa.
    pub child_owner_id: String,
    /// La máquina que ejecuta esta verificación -- usada como `node_id` de
    /// AMBOS lados (emisor y ejecutor) en este harness de un solo proceso;
    /// en producción real cada lado corre en su propia máquina, unidas por
    /// el adaptador de red del relé genérico (diferido, ADR-0143).
    pub node_id: String,
    /// `"COVERED"` (siembra un opt-in real de la hija) | `"OPTED_OUT"`
    /// (siembra un opt-out explícito) | cualquier otro valor / ausente ->
    /// `"NO_CONSENT"` (no siembra nada -- resuelve al default-deny real de
    /// `consent-registry`, #5). Mismo vocabulario que
    /// `VerifiedAccountRegistryVerifyInput::consent`.
    #[serde(default = "default_verify_consent_state")]
    pub consent: String,
    #[serde(default = "default_master_account_hierarchy_consent_version")]
    pub consent_version: String,
    /// `"ARCHIVE"` | `"MODIFY"` | `"REQUEST_AUDIT_REPORT"`.
    pub command_kind: String,
    pub target_ref: String,
    pub justification: Option<String>,
}

fn default_master_account_hierarchy_consent_version() -> String {
    "v1".to_string()
}

/// Output de la verificación de Master Account Hierarchy. Siempre
/// serializa a JSON válido (ADR-0142). **Ningún campo porta el comando
/// cifrado en claro, credenciales de bróker ni IPs live** (ADR-0093) -- solo
/// el HECHO auditado del intento (desenlace + hashes de ambas atestaciones).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MasterAccountHierarchyVerifyOutput {
    pub ok: bool,
    /// `"EXECUTED"` o `"DENIED"` -- el desenlace del gate de autorización,
    /// idéntico en AMBAS atestaciones (regla fija #4: nunca se descarta un
    /// intento denegado en silencio).
    pub outcome: Option<String>,
    /// Presente SOLO si `outcome == "DENIED"` -- la razón real que
    /// `ConsentVerdict::NotCovered` trajo, nunca un booleano ciego.
    pub denial_reason: Option<String>,
    /// `"ARCHIVED"` o `"NO_EFFECT"` -- el efecto local que la hija aplicó
    /// (regla fija #5: "eliminar" nunca es un DELETE físico).
    pub local_effect: Option<String>,
    pub issuer_event_sequence_id: Option<i64>,
    pub issuer_audit_hash: Option<String>,
    pub executor_event_sequence_id: Option<i64>,
    pub executor_audit_hash: Option<String>,
    pub error: Option<String>,
}

impl MasterAccountHierarchyVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            outcome: None,
            denial_reason: None,
            local_effect: None,
            issuer_event_sequence_id: None,
            issuer_audit_hash: None,
            executor_event_sequence_id: None,
            executor_audit_hash: None,
            error: Some(msg),
        }
    }

    fn from_result(result: &master_account_hierarchy::OverrideExecutionResult) -> Self {
        let (outcome_str, denial_reason) = match &result.outcome {
            master_account_hierarchy::OverrideOutcome::Executed => ("EXECUTED".to_string(), None),
            master_account_hierarchy::OverrideOutcome::Denied(reason) => ("DENIED".to_string(), Some(reason.clone())),
        };
        let local_effect_str = match result.local_effect {
            master_account_hierarchy::LocalEffect::Archived => "ARCHIVED",
            master_account_hierarchy::LocalEffect::NoEffect => "NO_EFFECT",
        };

        Self {
            ok: true,
            outcome: Some(outcome_str),
            denial_reason,
            local_effect: Some(local_effect_str.to_string()),
            issuer_event_sequence_id: Some(result.issuer.event_sequence_id),
            issuer_audit_hash: Some(result.issuer.audit_hash.clone()),
            executor_event_sequence_id: Some(result.executor.event_sequence_id),
            executor_audit_hash: Some(result.executor.audit_hash.clone()),
            error: None,
        }
    }
}

/// Ejecuta la verificación de Master Account Hierarchy con adaptadores
/// reales (BD SQLite temporal + reloj de sistema real + el `consent_out`
/// REAL de `consent-registry`, #5), recorriendo el camino completo del
/// cimiento #12: siembra (según `input.consent`) el consentimiento real de
/// la hija, registra la jerarquía fondo->hija, emite el override desde el
/// fondo y lo recibe/ejecuta en la hija -- ejercitando Core -> Shell ->
/// puerto tal como lo recorrería el motor real (el adaptador de red del
/// relé genérico queda diferido, ver `docs/features/master-account-hierarchy.md`).
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify master-account-hierarchy --input
/// '{"parent_owner_id":"fund-X","child_owner_id":"trader-7","node_id":"node-A","consent":"COVERED","command_kind":"ARCHIVE","target_ref":"strategy-42","justification":"riesgo excedido"}'`
pub async fn verify_master_account_hierarchy(
    input: MasterAccountHierarchyVerifyInput,
) -> MasterAccountHierarchyVerifyOutput {
    let Some(command_kind) = master_account_hierarchy::OverrideCommandKind::from_str_value(&input.command_kind)
    else {
        return MasterAccountHierarchyVerifyOutput::from_error(format!(
            "command_kind no reconocido: '{}'. Valores válidos: ARCHIVE, MODIFY, REQUEST_AUDIT_REPORT",
            input.command_kind
        ));
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_instance_continuity / verify_verified_account_registry).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-master-account-hierarchy-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return MasterAccountHierarchyVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return MasterAccountHierarchyVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return MasterAccountHierarchyVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Siembra el consentimiento REAL de la hija según input.consent -- igual
    // que verify_verified_account_registry / verify_data_aggregation. Sin
    // esto (o con cualquier valor distinto de COVERED/OPTED_OUT) no se
    // registra ningún evento, y el consent_out real resuelve NotCovered
    // (NoConsent) por default-deny.
    if input.consent == "COVERED" || input.consent == "OPTED_OUT" {
        let mut optout_changes = std::collections::BTreeMap::new();
        optout_changes.insert(
            master_account_hierarchy::MASTER_ACCOUNT_OVERRIDE_CONSENT_DATA_TYPE.to_string(),
            input.consent == "OPTED_OUT",
        );
        if let Err(e) = crate::orchestrator::consent_registry::record_consent_action(
            &pool,
            clock.as_ref(),
            crate::persistence::consent_registry::RecordConsentActionInput {
                owner_id: input.child_owner_id.clone(),
                institutional_tag: default_institutional_tag(),
                node_id: input.node_id.clone(),
                compliance_status_id: None,
                action: crate::domain::consent_registry::ConsentAction::Accept,
                tos_version: Some(input.consent_version.clone()),
                optout_changes,
            },
        )
        .await
        {
            return MasterAccountHierarchyVerifyOutput::from_error(format!(
                "fallo al sembrar el consentimiento: {e}"
            ));
        }
    }

    // Registra la jerarquía (hija -> fondo) ANTES de ejercer el override --
    // mismo camino que el orquestador real recorrería.
    if let Err(e) = master_account_hierarchy::link_child_to_parent(
        &pool,
        clock.as_ref(),
        &input.child_owner_id,
        Some(&input.parent_owner_id),
        &input.consent_version,
        &input.node_id,
    )
    .await
    {
        return MasterAccountHierarchyVerifyOutput::from_error(format!("fallo al registrar la jerarquía: {e}"));
    }

    let result = match master_account_hierarchy::execute_override(
        &pool,
        clock.as_ref(),
        &input.parent_owner_id,
        &input.child_owner_id,
        &input.node_id,
        &input.node_id,
        command_kind,
        &input.target_ref,
        input.justification.as_deref(),
        &input.consent_version,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return MasterAccountHierarchyVerifyOutput::from_error(format!("fallo al ejecutar el override: {e}")),
    };

    MasterAccountHierarchyVerifyOutput::from_result(&result)
}

#[cfg(test)]
mod master_account_hierarchy_verify_tests {
    use super::*;

    /// CRITERIO (Orden §8): JSON no filtra secretos (ADR-0093) -- fija la
    /// lista exacta de claves permitidas en el output serializado y
    /// confirma que ningún patrón de secreto/credencial aparece, mismo
    /// patrón que `verified_account_record_json_never_leaks_secret_looking_fields`.
    #[test]
    fn master_account_hierarchy_verify_output_json_never_leaks_secret_fields() {
        let output = MasterAccountHierarchyVerifyOutput {
            ok: true,
            outcome: Some("EXECUTED".to_string()),
            denial_reason: None,
            local_effect: Some("ARCHIVED".to_string()),
            issuer_event_sequence_id: Some(1),
            issuer_audit_hash: Some("hash-issuer".to_string()),
            executor_event_sequence_id: Some(2),
            executor_audit_hash: Some("hash-executor".to_string()),
            error: None,
        };

        let json = serde_json::to_value(&output).expect("serializar");
        let object = json.as_object().expect("el JSON debe ser un objeto");

        let allowed_keys = [
            "ok",
            "outcome",
            "denial_reason",
            "local_effect",
            "issuer_event_sequence_id",
            "issuer_audit_hash",
            "executor_event_sequence_id",
            "executor_audit_hash",
            "error",
        ];
        for key in object.keys() {
            assert!(allowed_keys.contains(&key.as_str()), "clave no permitida en el output: '{key}'");
        }

        let json_lowercase = serde_json::to_string(&output).expect("serializar").to_lowercase();
        for forbidden in [
            "password", "api_key", "api-key", "broker_secret", "private_key",
            "signing_key", "investor_password", "master_secret", "192.168.", "10.0.0.",
        ] {
            assert!(!json_lowercase.contains(forbidden), "el output no debe contener '{forbidden}'");
        }
    }
}

// ────────────────────────────────────────────────────────────────────────
// Data Portability (STORY-043, cimiento #13 -- vive en `shared`, ver
// ADR-0137)
// ────────────────────────────────────────────────────────────────────────

/// Input para la verificación de Data Portability vía CLI
/// (`docs/features/data-portability.md`, STORY-043). Se deserializa desde
/// el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify data-portability --input
/// '{"owner_id":"user-42","institutional_tag":"LIVE","node_id":"node-A","request_type":"FORGET"}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataPortabilityVerifyInput {
    /// El titular autenticado que pide su acceso/portabilidad/olvido --
    /// SIEMPRE sale de `central-identity` (#1) en producción; este harness
    /// lo acepta directo para poder ejercitar el camino sin depender de
    /// otro cimiento.
    pub owner_id: String,
    #[serde(default = "default_institutional_tag")]
    pub institutional_tag: String,
    /// La máquina que registra esta solicitud.
    pub node_id: String,
    /// `"EXPORT"` (Art. 15/20 GDPR) | `"FORGET"` (Art. 17 GDPR).
    pub request_type: String,
}

/// Output de la verificación de Data Portability. Siempre serializa a JSON
/// válido (ADR-0142). **Ningún campo porta el dato real de ninguna tabla,
/// credenciales de bróker ni IPs live** (ADR-0093) -- el manifiesto solo
/// lista NOMBRES de tabla (la estructura), nunca contenido.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataPortabilityVerifyOutput {
    pub ok: bool,
    pub request_type: Option<String>,
    /// El estado del evento recién registrado -- siempre `"RECEIVED"` para
    /// una solicitud nueva (el avance PROCESSING/COMPLETED lo emite el
    /// adaptador diferido como eventos posteriores del mismo grupo).
    pub status: Option<String>,
    pub request_group_id: Option<String>,
    pub event_sequence_id: Option<i64>,
    pub audit_hash: Option<String>,
    /// Presente SOLO si `request_type == "EXPORT"` -- los NOMBRES de tabla
    /// del manifiesto (ya filtrados de secretos), nunca su contenido.
    pub manifest_tables: Option<Vec<String>>,
    /// Presente SOLO si `request_type == "FORGET"` -- el JSON de
    /// disposición por tabla (`PSEUDONYMIZE_AND_RETAIN`/`PSEUDONYMIZE_AND_PURGE`),
    /// el mismo que quedó persistido en `disposition_detail`.
    pub disposition_detail: Option<String>,
    pub error: Option<String>,
}

impl DataPortabilityVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            request_type: None,
            status: None,
            request_group_id: None,
            event_sequence_id: None,
            audit_hash: None,
            manifest_tables: None,
            disposition_detail: None,
            error: Some(msg),
        }
    }

    /// Construye el output a partir de un [`data_portability::ExportRequestResult`]
    /// -- `manifest_tables` es la lista de NOMBRES de tabla del manifiesto,
    /// nunca su dato real.
    fn from_export_result(result: &data_portability::ExportRequestResult) -> Self {
        Self {
            ok: true,
            request_type: Some(result.request.request_type.as_str().to_string()),
            status: Some(result.request.status.as_str().to_string()),
            request_group_id: Some(result.request.request_group_id.clone()),
            event_sequence_id: Some(result.request.event_sequence_id),
            audit_hash: Some(result.request.audit_hash.clone()),
            manifest_tables: Some(result.manifest.tables.iter().map(|t| t.table_name.clone()).collect()),
            disposition_detail: None,
            error: None,
        }
    }

    /// Construye el output a partir de un [`data_portability::ForgetRequestResult`]
    /// -- `disposition_detail` es el JSON ya persistido, sin ningún dato
    /// real de ninguna tabla.
    fn from_forget_result(result: &data_portability::ForgetRequestResult) -> Self {
        Self {
            ok: true,
            request_type: Some(result.request.request_type.as_str().to_string()),
            status: Some(result.request.status.as_str().to_string()),
            request_group_id: Some(result.request.request_group_id.clone()),
            event_sequence_id: Some(result.request.event_sequence_id),
            audit_hash: Some(result.request.audit_hash.clone()),
            manifest_tables: None,
            disposition_detail: result.request.disposition_detail.clone(),
            error: None,
        }
    }
}

/// Ejecuta la verificación de Data Portability con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real), recorriendo el camino
/// completo del cimiento #13: siembra el catálogo declarativo conocido
/// ([`data_portability::seed_known_catalog`], idempotente) y registra la
/// solicitud pedida (`EXPORT` arma el manifiesto vía el Core; `FORGET`
/// arma el detalle de disposición vía el Core) -- ejercitando Core -> Shell
/// -> puerto tal como lo recorrería la app real. El generador de archivo
/// real (recorrer el esquema y volcar el dato) queda diferido, ver
/// `docs/features/data-portability.md`.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify data-portability --input
/// '{"owner_id":"user-42","institutional_tag":"LIVE","node_id":"node-A","request_type":"FORGET"}'`
pub async fn verify_data_portability(input: DataPortabilityVerifyInput) -> DataPortabilityVerifyOutput {
    let Some(request_type) = data_portability::RequestType::from_str_value(&input.request_type) else {
        return DataPortabilityVerifyOutput::from_error(format!(
            "request_type no reconocido: '{}'. Valores válidos: EXPORT, FORGET",
            input.request_type
        ));
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_master_account_hierarchy / verify_instance_continuity).
    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-data-portability-{}", uuid::Uuid::new_v4()));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return DataPortabilityVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return DataPortabilityVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return DataPortabilityVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Siembra el catálogo declarativo conocido -- idempotente, mismo camino
    // que recorrería el arranque real de la app (STORY-043 §6).
    if let Err(e) = data_portability::seed_known_catalog(&pool, clock.as_ref()).await {
        return DataPortabilityVerifyOutput::from_error(format!("fallo al sembrar el catálogo: {e}"));
    }

    let identity = data_portability::DataPortabilityIdentity {
        owner_id: input.owner_id.clone(),
        institutional_tag: input.institutional_tag.clone(),
        node_id: input.node_id.clone(),
    };

    match request_type {
        data_portability::RequestType::Export => {
            match data_portability::request_export(&pool, clock.as_ref(), &identity).await {
                Ok(result) => DataPortabilityVerifyOutput::from_export_result(&result),
                Err(e) => DataPortabilityVerifyOutput::from_error(format!("fallo al solicitar el export: {e}")),
            }
        }
        data_portability::RequestType::Forget => {
            match data_portability::request_forget(&pool, clock.as_ref(), &identity).await {
                Ok(result) => DataPortabilityVerifyOutput::from_forget_result(&result),
                Err(e) => DataPortabilityVerifyOutput::from_error(format!("fallo al solicitar el olvido: {e}")),
            }
        }
    }
}

#[cfg(test)]
mod data_portability_verify_tests {
    use super::*;

    /// CRITERIO (Orden §8): JSON no filtra secretos (ADR-0093) -- fija la
    /// lista exacta de claves permitidas en el output serializado y
    /// confirma que ningún patrón de secreto/credencial aparece, mismo
    /// patrón que `master_account_hierarchy_verify_output_json_never_leaks_secret_fields`.
    #[test]
    fn data_portability_verify_output_json_never_leaks_secret_fields() {
        let output = DataPortabilityVerifyOutput {
            ok: true,
            request_type: Some("FORGET".to_string()),
            status: Some("RECEIVED".to_string()),
            request_group_id: Some("grp-1".to_string()),
            event_sequence_id: Some(1),
            audit_hash: Some("hash-1".to_string()),
            manifest_tables: None,
            disposition_detail: Some(
                "[{\"table_name\":\"usage_records\",\"feature_name\":\"usage-metering\",\"disposition\":\"PSEUDONYMIZE_AND_RETAIN\"}]"
                    .to_string(),
            ),
            error: None,
        };

        let json = serde_json::to_value(&output).expect("serializar");
        let object = json.as_object().expect("el JSON debe ser un objeto");

        let allowed_keys = [
            "ok",
            "request_type",
            "status",
            "request_group_id",
            "event_sequence_id",
            "audit_hash",
            "manifest_tables",
            "disposition_detail",
            "error",
        ];
        for key in object.keys() {
            assert!(allowed_keys.contains(&key.as_str()), "clave no permitida en el output: '{key}'");
        }

        let json_lowercase = serde_json::to_string(&output).expect("serializar").to_lowercase();
        for forbidden in [
            "password", "api_key", "api-key", "broker_secret", "private_key",
            "signing_key", "investor_password", "master_secret", "encryption_key",
            "192.168.", "10.0.0.",
        ] {
            assert!(!json_lowercase.contains(forbidden), "el output no debe contener '{forbidden}'");
        }
    }

    /// CRITERIO DE CIERRE: `manifest_tables` de un EXPORT real NUNCA
    /// incluye `api_credentials` -- el filtro de secretos del Core corre
    /// dentro de `request_export` antes de que este output se arme.
    #[tokio::test]
    async fn verify_export_manifest_never_includes_the_api_credentials_table() {
        let output = verify_data_portability(DataPortabilityVerifyInput {
            owner_id: "user-42".to_string(),
            institutional_tag: "LIVE".to_string(),
            node_id: "node-A".to_string(),
            request_type: "EXPORT".to_string(),
        })
        .await;

        assert!(output.ok, "la verificación debe tener éxito: {:?}", output.error);
        let tables = output.manifest_tables.expect("un EXPORT debe traer manifest_tables");
        assert!(!tables.contains(&"api_credentials".to_string()), "api_credentials porta secretos -- nunca debe exportarse");
        assert!(tables.contains(&"verified_accounts".to_string()));
    }

    /// CRITERIO DE CIERRE: un FORGET real produce `disposition_detail` con
    /// las tres tablas de retención legal marcadas
    /// `PSEUDONYMIZE_AND_RETAIN`, nunca una variante de borrado.
    #[tokio::test]
    async fn verify_forget_disposition_never_contains_a_delete_variant() {
        let output = verify_data_portability(DataPortabilityVerifyInput {
            owner_id: "user-42".to_string(),
            institutional_tag: "LIVE".to_string(),
            node_id: "node-A".to_string(),
            request_type: "FORGET".to_string(),
        })
        .await;

        assert!(output.ok, "la verificación debe tener éxito: {:?}", output.error);
        let detail = output.disposition_detail.expect("un FORGET debe traer disposition_detail");
        assert!(!detail.to_lowercase().contains("delete"), "el detalle de disposición nunca debe mencionar un borrado");
        assert!(detail.contains("PSEUDONYMIZE_AND_RETAIN"));
        assert!(detail.contains("PSEUDONYMIZE_AND_PURGE"));
    }

    /// Un `request_type` fuera de EXPORT/FORGET falla con un error claro,
    /// nunca con un panic.
    #[tokio::test]
    async fn verify_rejects_unknown_request_type() {
        let output = verify_data_portability(DataPortabilityVerifyInput {
            owner_id: "user-42".to_string(),
            institutional_tag: "LIVE".to_string(),
            node_id: "node-A".to_string(),
            request_type: "RECTIFY".to_string(),
        })
        .await;

        assert!(!output.ok);
        assert!(output.error.expect("debe traer error").contains("request_type"));
    }
}

// ── Operator Roles -- cimiento #14 (Canal #2, ADR-0142) ─────────────────────

/// Token de acceso raíz que este harness siembra como primer admin por
/// defecto cuando `input.root_access_token_id` no se especifica -- mismo
/// valor que el ejemplo canónico de la Orden (STORY-044 §9), para que el
/// comando de ejemplo funcione copy/paste sin campos adicionales.
const DEFAULT_ROOT_ACCESS_TOKEN_ID: &str = "tok-owner";

/// Traduce el nombre de pipeline en texto (`"GENERATE"`, `"EXECUTE"`, …) al
/// enum [`crate::domain::mcp_gateway::Pipeline`] -- este harness NO
/// modifica `mcp_gateway.rs` (código sellado de #8); esta es plomería de
/// CLI local a `public_interface`.
fn parse_pipeline(value: &str) -> Option<crate::domain::mcp_gateway::Pipeline> {
    use crate::domain::mcp_gateway::Pipeline;
    match value.to_uppercase().as_str() {
        "INGEST" => Some(Pipeline::Ingest),
        "GENERATE" => Some(Pipeline::Generate),
        "VALIDATE" => Some(Pipeline::Validate),
        "INCUBATE" => Some(Pipeline::Incubate),
        "MANAGE" => Some(Pipeline::Manage),
        "EXECUTE" => Some(Pipeline::Execute),
        "FEEDBACK" => Some(Pipeline::Feedback),
        "WITHDRAW" => Some(Pipeline::Withdraw),
        _ => None,
    }
}

/// Traduce `"LIVE"`/`"DEMO"` al enum
/// [`crate::domain::mcp_gateway::InstitutionalTag`] -- solo relevante
/// cuando `pipeline == "MANAGE"`.
fn parse_manage_institutional_tag(value: &str) -> Option<crate::domain::mcp_gateway::InstitutionalTag> {
    use crate::domain::mcp_gateway::InstitutionalTag;
    match value.to_uppercase().as_str() {
        "LIVE" => Some(InstitutionalTag::Live),
        "DEMO" => Some(InstitutionalTag::Demo),
        _ => None,
    }
}

/// Input para la verificación de Operator Roles vía CLI
/// (`docs/features/operator-roles.md`, ADR-0149, STORY-044). Se deserializa
/// desde el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico (golden path -- admin invocando una capacidad que él mismo se
/// declaró, en pipeline abierto):
/// `cargo run -p app -- verify operator-roles --input
/// '{"owner_id":"acc-1","institutional_tag":"LIVE","node_id":"node-A","access_token_id":"tok-owner","capability_key":"generate.run_search","pipeline":"GENERATE"}'`
///
/// Uso típico (operador sin rol -- mismo `owner_id`, un `access_token_id`
/// que NUNCA se asignó):
/// `cargo run -p app -- verify operator-roles --input
/// '{"owner_id":"acc-1","institutional_tag":"LIVE","node_id":"node-A","access_token_id":"tok-nunca-asignado","capability_key":"generate.run_search","pipeline":"GENERATE"}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperatorRolesVerifyInput {
    /// La cuenta maestra dueña del catálogo de roles -- SIEMPRE sale de
    /// `central-identity` (#1) en producción; este harness lo acepta
    /// directo para poder ejercitar el camino sin depender de otro
    /// cimiento.
    pub owner_id: String,
    #[serde(default = "default_institutional_tag")]
    pub institutional_tag: String,
    /// La máquina que registra los eventos de esta verificación.
    pub node_id: String,
    /// El operador (login humano o conexión MCP) cuya llamada se evalúa.
    pub access_token_id: String,
    /// El token de acceso que este harness siembra como primer ADMIN de la
    /// cuenta (`seed_admin_bootstrap`) -- por defecto
    /// [`DEFAULT_ROOT_ACCESS_TOKEN_ID`]. Pásalo distinto de
    /// `access_token_id` para demostrar el camino "operador sin rol".
    #[serde(default)]
    pub root_access_token_id: Option<String>,
    /// El puerto de Feature invocado -- clave de capacidad evaluada contra
    /// la matriz del rol.
    pub capability_key: String,
    /// `"INGEST"|"GENERATE"|"VALIDATE"|"INCUBATE"|"MANAGE"|"EXECUTE"|"FEEDBACK"|"WITHDRAW"`
    /// -- el pipeline de destino para el evaluador de ADR-0123.
    pub pipeline: String,
    /// Solo relevante si `pipeline == "MANAGE"`: `"LIVE"` o `"DEMO"`.
    #[serde(default)]
    pub manage_institutional_tag: Option<String>,
    /// Estado del interruptor de producción en el momento de la evaluación
    /// -- por defecto apagado (`false`).
    #[serde(default)]
    pub production_override_active: bool,
}

/// Output de la verificación de Operator Roles. Siempre serializa a JSON
/// válido (ADR-0142). **Ningún campo porta credenciales, secretos de
/// bróker ni IPs live** (ADR-0093) -- solo el veredicto y metadatos de
/// auditoría no sensibles.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperatorRolesVerifyOutput {
    pub ok: bool,
    /// `"GRANTED"` | `"DENIED_BY_ROLE"` | `"DENIED_BY_PIPELINE"`.
    pub verdict: Option<String>,
    /// Motivo de la denegación, si aplica.
    pub reason: Option<String>,
    /// El rol resuelto para `access_token_id`, o `None` si no tenía
    /// asignación ACTIVA.
    pub resolved_role_id: Option<String>,
    pub resolved_role_name: Option<String>,
    /// `audit_hash` del último evento del ledger registrado durante esta
    /// verificación (la reclasificación de la matriz del admin que habilita
    /// el golden path).
    pub audit_hash: Option<String>,
    pub event_sequence_id: Option<i64>,
    pub error: Option<String>,
}

impl OperatorRolesVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            verdict: None,
            reason: None,
            resolved_role_id: None,
            resolved_role_name: None,
            audit_hash: None,
            event_sequence_id: None,
            error: Some(msg),
        }
    }
}

/// Ejecuta la verificación de Operator Roles con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real), recorriendo el camino
/// completo del cimiento #14:
///
/// 1. Siembra el primer admin por defecto (`seed_admin_bootstrap`,
///    idempotente) para `root_access_token_id`.
/// 2. El propio admin -- que YA tiene `CAPABILITY_MANAGE_ROLES` -- declara
///    la capacidad `input.capability_key` en su propio rol
///    (`update_role_matrix`), demostrando que un admin puede conceder
///    capacidades nuevas sin reinventar el bootstrap con un comodín.
/// 3. Evalúa la llamada de `access_token_id` contra `capability_key` y el
///    pipeline pedido -- gate compuesto (#14 AND ADR-0123).
///
/// Si `access_token_id` es distinto de `root_access_token_id` y nunca se
/// le asignó nada, el paso 2 no lo afecta -- el paso 3 devuelve
/// `DENIED_BY_ROLE` (ADR-0149: sin rol explícito, denegado).
///
/// Uso típico desde el CLI: ver [`OperatorRolesVerifyInput`].
pub async fn verify_operator_roles(input: OperatorRolesVerifyInput) -> OperatorRolesVerifyOutput {
    let Some(pipeline) = parse_pipeline(&input.pipeline) else {
        return OperatorRolesVerifyOutput::from_error(format!(
            "pipeline no reconocido: '{}'. Valores válidos: INGEST, GENERATE, VALIDATE, INCUBATE, MANAGE, EXECUTE, FEEDBACK, WITHDRAW",
            input.pipeline
        ));
    };

    let manage_institutional_tag = match input.manage_institutional_tag.as_deref() {
        Some(raw) => match parse_manage_institutional_tag(raw) {
            Some(tag) => Some(tag),
            None => {
                return OperatorRolesVerifyOutput::from_error(format!(
                    "manage_institutional_tag no reconocido: '{raw}'. Valores válidos: LIVE, DEMO"
                ))
            }
        },
        None => None,
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_master_account_hierarchy / verify_data_portability).
    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-operator-roles-{}", uuid::Uuid::new_v4()));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return OperatorRolesVerifyOutput::from_error(format!("no se pudo crear el directorio temporal de verificación: {e}"));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => return OperatorRolesVerifyOutput::from_error(format!("no se pudo crear la BD temporal de verificación: {e}")),
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return OperatorRolesVerifyOutput::from_error(format!("error al aplicar migraciones en la BD temporal: {e}"));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    let identity = operator_roles::OperatorRolesIdentity {
        owner_id: input.owner_id.clone(),
        institutional_tag: input.institutional_tag.clone(),
        node_id: input.node_id.clone(),
    };

    let root_access_token_id = input.root_access_token_id.clone().unwrap_or_else(|| DEFAULT_ROOT_ACCESS_TOKEN_ID.to_string());

    // Paso 1 -- siembra el primer admin por defecto (idempotente).
    let (admin_role, _admin_assignment) =
        match operator_roles::seed_admin_bootstrap(&pool, clock.as_ref(), &identity, &root_access_token_id).await {
            Ok(seeded) => seeded,
            Err(e) => return OperatorRolesVerifyOutput::from_error(format!("fallo al sembrar el admin por defecto: {e}")),
        };

    // Paso 2 -- el admin, que ya tiene CAPABILITY_MANAGE_ROLES, declara la
    // capacidad pedida en SU PROPIO rol -- último evento del ledger que
    // este harness expone en el output.
    let mut expanded_matrix = admin_role.capability_matrix.clone();
    expanded_matrix.set(input.capability_key.clone(), true);
    let (_updated_admin_role, last_event) =
        match operator_roles::update_role_matrix(&pool, clock.as_ref(), &identity, &admin_role.id, expanded_matrix).await {
            Ok(result) => result,
            Err(e) => return OperatorRolesVerifyOutput::from_error(format!("fallo al declarar la capacidad en el rol admin: {e}")),
        };

    // Paso 3 -- evalúa la llamada REAL del operador solicitado.
    let permission_request = crate::domain::mcp_gateway::PermissionRequest {
        pipeline,
        institutional_tag: manage_institutional_tag,
        production_override_active: input.production_override_active,
        agent_session_id: input.access_token_id.clone(),
        requested_scope: input.capability_key.clone(),
    };

    let evaluation = match operator_roles::evaluate_call(
        &pool,
        clock.as_ref(),
        &identity,
        &input.access_token_id,
        &input.capability_key,
        &permission_request,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => return OperatorRolesVerifyOutput::from_error(format!("fallo al evaluar la llamada del operador: {e}")),
    };

    let (verdict_str, reason) = match &evaluation.verdict {
        operator_roles::CombinedVerdict::Granted => ("GRANTED".to_string(), None),
        operator_roles::CombinedVerdict::DeniedByRole { reason } => ("DENIED_BY_ROLE".to_string(), Some(reason.clone())),
        operator_roles::CombinedVerdict::DeniedByPipeline { reason } => ("DENIED_BY_PIPELINE".to_string(), Some(reason.clone())),
    };

    let resolved_role_name = match &evaluation.resolved_role_id {
        Some(role_id) => match operator_roles::OperatorRoleRepository::new(&pool, clock.as_ref()).get_role(role_id).await {
            Ok(Some(role)) => Some(role.role_name),
            _ => None,
        },
        None => None,
    };

    OperatorRolesVerifyOutput {
        ok: true,
        verdict: Some(verdict_str),
        reason,
        resolved_role_id: evaluation.resolved_role_id,
        resolved_role_name,
        audit_hash: Some(last_event.audit_hash),
        event_sequence_id: Some(last_event.event_sequence_id),
        error: None,
    }
}

#[cfg(test)]
mod operator_roles_verify_tests {
    use super::*;

    fn sample_input(access_token_id: &str) -> OperatorRolesVerifyInput {
        OperatorRolesVerifyInput {
            owner_id: "acc-1".to_string(),
            institutional_tag: "LIVE".to_string(),
            node_id: "node-A".to_string(),
            access_token_id: access_token_id.to_string(),
            root_access_token_id: None,
            capability_key: "generate.run_search".to_string(),
            pipeline: "GENERATE".to_string(),
            manage_institutional_tag: None,
            production_override_active: false,
        }
    }

    /// CRITERIO (Orden §9): el comando de ejemplo -- un ADMIN invocando una
    /// capacidad permitida en pipeline abierto -- da `GRANTED`.
    #[tokio::test]
    async fn verify_grants_admin_operator_invoking_a_declared_capability_on_an_open_pipeline() {
        let output = verify_operator_roles(sample_input(DEFAULT_ROOT_ACCESS_TOKEN_ID)).await;

        assert!(output.ok, "la verificación debe tener éxito: {:?}", output.error);
        assert_eq!(output.verdict, Some("GRANTED".to_string()));
        assert!(output.resolved_role_id.is_some());
        assert_eq!(output.resolved_role_name, Some("Admin".to_string()));
        assert!(output.audit_hash.is_some());
        assert!(output.event_sequence_id.is_some());
    }

    /// CRITERIO (Orden §9): un operador SIN rol asignado se deniega.
    #[tokio::test]
    async fn verify_denies_operator_without_any_assignment() {
        let output = verify_operator_roles(sample_input("tok-nunca-asignado")).await;

        assert!(output.ok, "la verificación en sí debe tener éxito: {:?}", output.error);
        assert_eq!(output.verdict, Some("DENIED_BY_ROLE".to_string()));
        assert!(output.resolved_role_id.is_none());
        assert!(output.reason.is_some());
    }

    /// CRITERIO (Orden §8): JSON no filtra secretos (ADR-0093).
    #[test]
    fn operator_roles_verify_output_json_never_leaks_secret_fields() {
        let output = OperatorRolesVerifyOutput {
            ok: true,
            verdict: Some("GRANTED".to_string()),
            reason: None,
            resolved_role_id: Some("role-1".to_string()),
            resolved_role_name: Some("Admin".to_string()),
            audit_hash: Some("hash-1".to_string()),
            event_sequence_id: Some(1),
            error: None,
        };

        let json = serde_json::to_value(&output).expect("serializar");
        let object = json.as_object().expect("el JSON debe ser un objeto");

        let allowed_keys =
            ["ok", "verdict", "reason", "resolved_role_id", "resolved_role_name", "audit_hash", "event_sequence_id", "error"];
        for key in object.keys() {
            assert!(allowed_keys.contains(&key.as_str()), "clave no permitida en el output: '{key}'");
        }

        let json_lowercase = serde_json::to_string(&output).expect("serializar").to_lowercase();
        for forbidden in [
            "password", "api_key", "api-key", "broker_secret", "private_key",
            "signing_key", "investor_password", "master_secret", "encryption_key",
            "192.168.", "10.0.0.",
        ] {
            assert!(!json_lowercase.contains(forbidden), "el output no debe contener '{forbidden}'");
        }
    }

    /// Un `pipeline` fuera del catálogo falla con un error claro, nunca con
    /// un panic.
    #[tokio::test]
    async fn verify_rejects_unknown_pipeline() {
        let mut input = sample_input(DEFAULT_ROOT_ACCESS_TOKEN_ID);
        input.pipeline = "UNKNOWN".to_string();
        let output = verify_operator_roles(input).await;

        assert!(!output.ok);
        assert!(output.error.expect("debe traer error").contains("pipeline"));
    }
}
