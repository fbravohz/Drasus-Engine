# STORY-030 — Usage Metering / Libro de Nocional: lecciones de Rust

> **Story:** [STORY-030 — Usage Metering / Libro de Nocional (cimiento #4 del substrato de monetización)](../../execution/STORY-030-usage-metering.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0010_usage_metering.sql`, `crates/shared/src/domain/usage_metering.rs`, `crates/shared/src/persistence/usage_metering.rs`, `crates/shared/src/orchestrator/usage_metering.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Qué es un "libro append-only" y por qué NUNCA se edita

Un libro contable de verdad (el de un contador humano, en papel) no se corrige tachando: si un asiento estaba mal, se escribe un asiento NUEVO que lo corrige, y el asiento viejo se queda ahí, visible, para siempre. Esa es la idea de "append-only" ("solo agregar"): cada fila de `usage_records` es un HECHO que ya ocurrió ("esta operación se ejecutó y sumó tanto nocional") — no un estado que cambia con el tiempo. Una vez que el hecho se grabó, no existe ninguna operación de "corregirlo" ni "borrarlo": si algo estuvo mal, se agrega OTRA fila que lo refleja.

Esto no es una convención de estilo, es una garantía forzada en DOS capas independientes (defensa en profundidad):

1. **La capa Rust:** [`UsageRepository`](../../../crates/shared/src/persistence/usage_metering.rs) solo expone `record_operation` (un INSERT) y `load_chain`/`load_tail` (lecturas). No existe ningún método `update_limits` ni `delete` en ese `impl` — quien quiera mutar una fila desde Rust literalmente no tiene con qué.
2. **La capa SQLite:** aunque alguien se saltara la capa Rust (ej. abriera la BD con una herramienta externa), la migración pone un guardián a nivel de motor de base de datos:

```sql
-- migrations/0010_usage_metering.sql
CREATE TRIGGER IF NOT EXISTS trg_usage_records_no_update
BEFORE UPDATE ON usage_records
BEGIN
    SELECT RAISE(ABORT, 'usage_records is append-only: UPDATE is forbidden');
END;

CREATE TRIGGER IF NOT EXISTS trg_usage_records_no_delete
BEFORE DELETE ON usage_records
BEGIN
    SELECT RAISE(ABORT, 'usage_records is append-only: DELETE is forbidden');
END;
```

`BEFORE UPDATE`/`BEFORE DELETE` significa "antes de que SQLite ejecute el UPDATE/DELETE real, corre esto primero". `RAISE(ABORT, mensaje)` cancela toda la sentencia y devuelve un error al llamador — SQLite nunca llega a tocar la fila. La prueba discriminante ejercita EXACTAMENTE este camino, atacando la BD con SQL crudo (sin pasar por `UsageRepository`, que ni siquiera lo permitiría):

```rust
// crates/shared/src/persistence/usage_metering.rs
#[tokio::test]
async fn update_is_rejected_by_trigger() {
    let row = repo.record_operation(sample_input(...)).await.expect("registrar operación");

    let result = sqlx::query("UPDATE usage_records SET cycle_accumulated = 0 WHERE id = ?")
        .bind(&row.id)
        .execute(&pool)
        .await;

    assert!(result.is_err(), "UPDATE sobre usage_records debe ser rechazado por el trigger");
}
```

Si alguien borrara el trigger de la migración, esta prueba fallaría inmediatamente (el `UPDATE` tendría éxito) — es una prueba que SÍ puede caerse, no un `assert!(true)` decorativo.

### `event_sequence_id` vs `row_version` — por qué esta tabla usa el primero (contraste con #1/#2/#3)

Este es el mismo eje de decisión que ya apareció en los tres cimientos anteriores del substrato, y vale la pena verlos juntos:

| Cimiento | Tabla | ¿Qué representa cada fila? | Columna de versión |
|---|---|---|---|
| #1 `central-identity` | `accounts` | El ESTADO VIGENTE de una cuenta (puede cambiar: verificar el correo, re-vincular OAuth) | `row_version` |
| #2 `licensing-system` | `licenses` | El ESTADO VIGENTE de una licencia (el heartbeat se refresca EN SITIO) | `row_version` |
| #3 `plan-tier-quota` | `plans` | El ESTADO VIGENTE de un plan (el precio/límite se revisa EN SITIO) | `row_version` |
| #4 `usage-metering` | `usage_records` | Un HECHO que ya ocurrió (esta operación se ejecutó, con este nocional, en este momento) | `event_sequence_id` |

La regla general (ADR-0141): si la fila representa "el estado actual de X" y ese estado legítimamente cambia con el tiempo sobre la MISMA fila, usa `row_version` (un contador por fila, empieza en 1, sube con cada `UPDATE`). Si la fila representa "algo que pasó" y nunca vuelve a cambiar, usa `event_sequence_id` (una posición monótona GLOBAL sobre TODA la tabla, sin que ninguna fila individual tenga versiones). El ADR es explícito y tajante sobre esto: **"PROHIBIDO usar el mismo nombre `event_sequence_id` para lo que es `row_version` y viceversa"** — no son intercambiables ni una cuestión de preferencia.

`usage_records` cae claramente del lado de "hecho que ya ocurrió": una operación ejecutada con su nocional no es algo que se "revisa" después — es historia. Por eso la migración declara:

```sql
-- migrations/0010_usage_metering.sql
event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena global (1, 2, 3, ...)
```

y el repositorio asigna esa posición leyendo la fila con el `event_sequence_id` más alto de TODA la tabla (no solo del dueño ni del ciclo — la cadena es global, igual que en `audit_events`, STORY-002):

```rust
// crates/shared/src/persistence/usage_metering.rs
async fn load_tail(&self) -> Result<Option<UsageRecordRow>, UsageRepositoryError> {
    let row = sqlx::query(
        "SELECT ... FROM usage_records ORDER BY event_sequence_id DESC LIMIT 1",
    )
    .fetch_optional(self.pool)
    .await?;
    row.map(row_to_usage_record).transpose()
}
```

La prueba discriminante de esta Story confirma que la asignación es estrictamente secuencial (1, 2, 3, sin saltos ni repeticiones) y que duplicar una posición ya usada es rechazado por el `UNIQUE` de la columna:

```rust
#[tokio::test]
async fn event_sequence_id_is_monotonic_across_inserts() {
    let first = repo.record_operation(...).await.expect("primera operación");
    let second = repo.record_operation(...).await.expect("segunda operación");
    let third = repo.record_operation(...).await.expect("tercera operación");

    assert_eq!(first.event_sequence_id, 1);
    assert_eq!(second.event_sequence_id, 2);
    assert_eq!(third.event_sequence_id, 3);
}
```

### Por qué reescalar al multiplicar dos enteros ×10⁸ (EL punto de correctitud crítico de esta Story)

Todo monto de dinero en el sistema (ADR-0141) se guarda como un entero MULTIPLICADO por 100 000 000 (10⁸) — así, en vez de guardar `$40,000.00`, se guarda `4_000_000_000_000`. La razón de fondo se explica más abajo ("qué pasa con `f64`"), pero primero hay que entender un efecto colateral no obvio: **multiplicar dos cantidades que YA están en esa escala no da un resultado en esa misma escala.**

Pensemos en un ejemplo con una escala mucho más chica para que se vea claro: si en vez de ×10⁸ usáramos ×10 (una cifra decimal), "2.5" se guardaría como `25` y "4.0" como `40`. Multiplicar los enteros guardados da `25 × 40 = 1000` — pero el producto MATEMÁTICO real es `2.5 × 4.0 = 10.0`, que en la escala ×10 sería `100`, no `1000`. El producto crudo de dos números en escala ×10 queda en escala ×10×10 = ×100 (los exponentes se SUMAN). Para volver a la escala ×10 original hay que DIVIDIR el producto crudo entre 10 (`1000 / 10 = 100`, que es "10.0" en la escala ×10 — correcto).

Exactamente lo mismo pasa con ×10⁸: multiplicar `tamaño` (×10⁸) por `precio` (×10⁸) da un producto en ×10¹⁶ (10⁸ × 10⁸ = 10¹⁶), no en ×10⁸. Hay que dividir entre 10⁸ para volver a la escala correcta:

```rust
// crates/shared/src/domain/usage_metering.rs
pub fn compute_notional(size: i64, price: i64) -> Result<i64, NotionalError> {
    // ...
    let raw_product: i128 = (size as i128) * (price as i128);   // producto crudo en x10^16
    let scale: i128 = AMOUNT_SCALE as i128;                      // 10^8
    let half = scale / 2;
    let rescaled: i128 = (raw_product + half) / scale;           // vuelve a x10^8
    i64::try_from(rescaled).map_err(|_| NotionalError::Overflow)
}
```

Con los valores del ejemplo de la Orden: tamaño `2.5` (`250_000_000` en ×10⁸) por precio `$40,000.00` (`4_000_000_000_000` en ×10⁸) da un producto crudo de `1_000_000_000_000_000_000_000` (mil trillones, ×10¹⁶) — dividido entre `100_000_000` (10⁸) da `10_000_000_000_000`, que es exactamente `$100,000.00` en ×10⁸. La prueba discriminante fija este valor conocido:

```rust
#[test]
fn compute_notional_rescales_known_values_exactly() {
    let size = 250_000_000; // 2.5 * 1e8
    let price = 4_000_000_000_000; // $40,000.00 * 1e8
    let notional = compute_notional(size, price).expect("debe calcular el nocional");
    assert_eq!(notional, 10_000_000_000_000, "$100,000.00 * 1e8");
}
```

### Por qué `i128` para el producto intermedio (y no `i64`)

`size` y `price` son cada uno `i64` (rango hasta ~9.22×10¹⁸). Multiplicarlos DIRECTAMENTE en `i64` puede desbordar mucho antes de llegar siquiera al reescalado: el producto de dos `i64` grandes fácilmente supera lo que un `i64` puede representar, aunque el resultado FINAL (ya dividido entre 10⁸) sea perfectamente válido. La prueba de valores grandes de esta Story lo demuestra con números concretos:

```rust
#[test]
fn compute_notional_handles_large_values_without_overflow() {
    let size = 5_000_000_000_000;  // $50,000.00 * 1e8
    let price = 5_000_000_000_000; // $50,000.00 * 1e8
    let notional = compute_notional(size, price).expect("no debe desbordar con i128 intermedio");
    assert_eq!(notional, 250_000_000_000_000_000);
}
```

`5_000_000_000_000 × 5_000_000_000_000 = 25_000_000_000_000_000_000_000_000` (2.5×10²⁵) — muy por encima de `i64::MAX` (~9.22×10¹⁸). Multiplicar esto directamente en `i64` haría *overflow* (en modo debug, Rust hace panic; en modo release, "envuelve" el número silenciosamente a un valor sin sentido — ninguna de las dos opciones es aceptable en un cálculo de facturación). `i128` tiene rango hasta ~1.7×10³⁸, así que sostiene ese producto crudo sin problema; una vez dividido entre 10⁸, el resultado (`2.5×10¹⁷`) SÍ cabe de nuevo en `i64`, y `i64::try_from(rescaled)` lo confirma explícitamente (devolviendo `Err(Overflow)` si algún día no cupiera, en vez de truncar en silencio).

### Qué pasa si se usara `f64` en vez de enteros (por qué NUNCA)

Un `f64` (número de punto flotante de doble precisión) representa números usando una fracción binaria, igual que `1/3` no tiene una representación decimal exacta y finita (`0.333...`), la mayoría de las fracciones decimales normales (`0.1`, `0.2`, `40000.00`) tampoco tienen una representación BINARIA exacta y finita. El resultado es que operaciones aparentemente triviales acumulan un error minúsculo: en casi cualquier lenguaje con `f64`/`double`, `0.1 + 0.2` da `0.30000000000000004`, no `0.3` exacto.

Para un nocional de facturación, ese error minúsculo es dinero real mal calculado — multiplicado por miles de operaciones en un ciclo, el error se acumula. Además, un `f64` solo representa ENTEROS de forma exacta hasta 2⁵³ (~9.007×10¹⁵); por encima de eso, hasta los enteros empiezan a redondearse. Un nocional acumulado de una cuenta institucional grande puede superar esa cota tranquilamente. Por eso este módulo (y todo el sistema, ADR-0141) opera EXCLUSIVAMENTE en enteros (`i64` para los montos que persisten, `i128` solo como zona de tránsito para el producto intermedio) — el tipo `f64` no aparece en ningún cálculo de `domain::usage_metering`, ni debería aparecer nunca en ningún cálculo de dinero de este proyecto.

### La política de redondeo es una DECISIÓN explícita, no un accidente

Cuando el producto crudo no es múltiplo exacto de 10⁸, hay que decidir qué hacer con el resto. Truncar (dividir con `/` normal en enteros, que en Rust siempre trunca hacia cero) sesga sistemáticamente hacia abajo — un nocional de "$1.999999...” se convertiría siempre en "$1", nunca en "$2", ni siquiera cuando el valor real está a un pelo de redondear hacia arriba. Este módulo usa la convención "half up" (el punto medio exacto sube): sumar la mitad de la escala ANTES de dividir.

```rust
let half = scale / 2;
let rescaled: i128 = (raw_product + half) / scale;
```

La prueba discriminante construye un caso donde el producto crudo es EXACTAMENTE 1.5 veces la escala (`150_000_000`, con `AMOUNT_SCALE = 100_000_000`):

```rust
#[test]
fn compute_notional_rounds_up_at_exact_half() {
    let notional = compute_notional(150_000_000, 1).expect("debe calcular");
    assert_eq!(notional, 2, "el punto medio exacto (.5) debe redondear hacia arriba");
}
```

Sin el `+ half`, `150_000_000 / 100_000_000` (división entera trunca) daría `1`, no `2` — la prueba fallaría. La prueba hermana (`compute_notional_stays_down_just_below_half`) confirma que un valor apenas por debajo del punto medio SIGUE redondeando hacia abajo, para que quede claro que no hay un sesgo sistemático hacia arriba en todos los casos, solo en el empate exacto.

### Qué es "acumular por ciclo" y cómo el "reinicio" ocurre SIN borrar nada

"Acumular por ciclo" significa: el nocional de una operación no vive aislado — se SUMA al nocional de todas las operaciones anteriores del MISMO dueño en el MISMO ciclo de facturación (`docs/features/usage-metering.md`: "al ejecutar una orden, su nocional se acumula"). La forma más simple de implementar esto sería guardar un contador mutable ("el acumulado actual") en alguna parte y sumarle cada vez — pero eso convertiría la tabla en mutable, contradiciendo todo lo dicho arriba sobre append-only.

La solución de esta Story es que el "acumulado" no sea un contador aparte: es una SUMA calculada, cada vez, de las filas ya persistidas de ese `(owner_id, billing_cycle_id)`:

```rust
// crates/shared/src/persistence/usage_metering.rs
async fn cycle_accumulated_so_far(&self, owner_id: &str, billing_cycle_id: &str) -> Result<i64, UsageRepositoryError> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(notional_per_op), 0) AS total \
         FROM usage_records WHERE owner_id = ? AND billing_cycle_id = ?",
    )
    .bind(owner_id)
    .bind(billing_cycle_id)
    .fetch_one(self.pool)
    .await?;
    Ok(row.get::<i64, _>("total"))
}
```

`COALESCE(SUM(...), 0)` es "si no hay ninguna fila que sumar, usa 0 en vez de NULL" — sin esto, un ciclo que todavía no tiene ninguna operación devolvería `NULL` de SQLite, no `0`, y el cálculo posterior fallaría al intentar tratarlo como número.

Esto responde también a la pregunta de "reinicio de ciclo": como el acumulado se calcula filtrando por `billing_cycle_id`, un ciclo NUEVO (ej. pasar de "2026-07" a "2026-08") automáticamente empieza a sumar desde cero — no porque algo se borre, sino porque la condición `WHERE billing_cycle_id = '2026-08'` simplemente no encuentra ninguna fila vieja de julio. El histórico de julio queda completo e intacto en la tabla, consultable para siempre. La prueba discriminante ejercita justo este comportamiento:

```rust
#[tokio::test]
async fn changing_billing_cycle_resets_accumulation_but_keeps_history() {
    repo.record_operation(sample_input("2026-07", ...)).await.expect("operación de julio");
    repo.record_operation(sample_input("2026-07", ...)).await.expect("segunda operación de julio");

    let august_first = repo.record_operation(sample_input("2026-08", ...)).await.expect("primera de agosto");
    assert_eq!(august_first.cycle_accumulated, 100_000_000_000, "el ciclo nuevo arranca en cero + esta operación");

    let chain = repo.load_chain().await.expect("cargar cadena completa");
    assert_eq!(chain.len(), 3, "las 2 filas de julio + la de agosto deben seguir todas presentes");
}
```

El identificador de ciclo mismo (`"2026-07"`, `"2026-08"`) se deriva del reloj INYECTADO, nunca de `SystemTime::now()` directo (el mismo principio de determinismo que rige todo el Core, `docs/features/clock.md`):

```rust
// crates/shared/src/domain/usage_metering.rs
pub fn derive_billing_cycle_id(timestamp_ns: i64) -> String {
    const NANOS_PER_DAY: i64 = 86_400_000_000_000;
    let days_since_epoch = timestamp_ns.div_euclid(NANOS_PER_DAY);
    let (year, month, _day) = civil_from_days(days_since_epoch);
    format!("{year:04}-{month:02}")
}
```

`civil_from_days` es el algoritmo público de Howard Hinnant que convierte "días desde 1970-01-01" a (año, mes, día) con SOLO aritmética entera — se eligió implementarlo directamente (unas 10 líneas) en vez de añadir una dependencia externa de calendario (`chrono`), porque el cálculo es pequeño, no cambia nunca, y evita sumarle al crate `shared` una dependencia nueva para resolver algo que la aritmética entera resuelve sola.

### Consumir el puerto REAL de otro cimiento (el primer cableado real del substrato)

Esta es la pieza que hace especial a esta Story dentro de la secuencia de cimientos: hasta ahora, cada cimiento nuevo consumía STUBS de los cimientos que todavía no existían (`licensing-system`, STORY-028, consume un `PlanLimits` inventado porque `plan-tier-quota` no existía cuando se construyó). `usage-metering` nace DESPUÉS de que `plan-tier-quota` (#3) ya está completo — así que aquí no hay excusa para inventar nada: se consume la función REAL que #3 expone.

```rust
// crates/shared/src/orchestrator/usage_metering.rs
use crate::orchestrator::plan_tier_quota::{build_plan_limits_for_tier, BuildPlanLimitsError};

pub async fn record_metered_operation(
    pool: &SqlitePool,
    clock: &dyn Clock,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    tier: PlanTier,
    operation: MeteredOperation<'_>,
) -> Result<UsageRecord, RecordMeteredOperationError> {
    // Paso 1 -- consumo REAL del puerto plan_limits_out de plan-tier-quota (#3).
    let plan_limits = build_plan_limits_for_tier(pool, clock, tier).await?;
    // ...
    let row = repo.record_operation(RecordOperationInput {
        // ...
        notional_limit: plan_limits.notional_limit,   // el límite REAL, no un número inventado
    }).await?;
    // ...
}
```

No hay ningún struct `PlanLimits` propio de `usage-metering`, ni ninguna función `stub_plan_limits()` — se importa directamente `build_plan_limits_for_tier` de `orchestrator::plan_tier_quota` (ambos viven en el mismo crate `shared`, así que esto es una llamada de función normal, no un cableado de red ni un mock). La prueba discriminante de esta pieza siembra el catálogo REAL de planes y confirma que el veredicto de cuota depende EXACTAMENTE del límite sembrado, no de un valor arbitrario:

```rust
// crates/shared/src/orchestrator/usage_metering.rs
#[tokio::test]
async fn record_metered_operation_uses_real_plan_limits_to_detect_crossing() {
    seed_default_catalog(&pool, &clock, "drasus-system", "seed-node", "DRASUS_LOCAL", &LocalStubPlanCatalogConfig::default())
        .await.expect("sembrar catálogo real de plan-tier-quota");

    // Operación pequeña ($1,000.00) -- muy por debajo del límite FREE real ($10,000.00).
    let within = record_metered_operation(..., PlanTier::Free, MeteredOperation { size: 100_000_000, price: 100_000_000_000, instrument_id: "BTCUSDT" }).await.expect("...");
    assert_eq!(within.quota_verdict, QuotaVerdict::Within);

    // Operación grande ($100,000.00) -- el acumulado cruza el límite FREE real ($10,000.00).
    let crossed = record_metered_operation(..., PlanTier::Free, MeteredOperation { size: 250_000_000, price: 4_000_000_000_000, instrument_id: "BTCUSDT" }).await.expect("...");
    assert_eq!(crossed.quota_verdict, QuotaVerdict::Crossed, "debe cruzar el límite FREE real de $10,000.00");
}
```

Nótese que el ÚNICO dato "sembrado" aquí es el CATÁLOGO de planes (`seed_default_catalog`, que ya existía y pertenece a `plan-tier-quota`) — nada dentro de `usage-metering` inventa un límite. El humo de la CLI confirma el mismo camino con datos reales de punta a punta:

```bash
cargo run -p app -- verify usage-metering --input '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'
```

```json
{
  "ok": true,
  "tier": "FREE",
  "billing_cycle_id": "2026-07",
  "cycle_accumulated": 10000000000000,
  "quota_verdict": "CROSSED",
  "operations_recorded": 1,
  "error": null
}
```

Una sola operación de $100,000.00 de nocional cruza, de inmediato, el límite FREE real de $10,000.00/mes que `plan-tier-quota` sembró en la BD temporal de esta verificación.

### `Order` es un placeholder — por qué no se modeló un `Order` completo

El catálogo de tipos de puerto (ADR-0137) declara que el puerto `order_in` de esta Feature recibe el tipo `Order` — pero hoy `Order` es literalmente `pub struct Order;` (un struct vacío, sin ni un solo campo) en `crates/shared/src/types/mod.rs`. Ese marcador existe solo para reservar el NOMBRE en el catálogo del canvas; el tipo real con campos (símbolo, cantidad, dirección, tipo de orden, etc.) es responsabilidad del módulo `execute` (EPIC-5), que todavía no se ha construido.

Modelar aquí un `Order` completo sería alucinar un contrato que ninguna feature real ha definido todavía — en vez de eso, esta Story modela la ENTRADA MÍNIMA que el cálculo de nocional necesita, con su propio nombre, sin pretender ser el `Order` real:

```rust
// crates/shared/src/domain/usage_metering.rs
pub struct MeteredOperation<'a> {
    pub size: i64,
    pub price: i64,
    pub instrument_id: &'a str,
}
```

Cuando `execute` exista y su `Order` real tenga campos concretos, el mapeo `Order → MeteredOperation` es una función de dos líneas en la Shell (leer tamaño/precio/instrumento del `Order` real y construir este struct) — no un rediseño. Documentar la deuda explícitamente (en vez de fingir que no existe) es lo que permite que ese día sea trabajo trivial.

## Trucos de Senior

- Cuando una tabla nueva podría parecer "casi append-only" pero en realidad representa un ESTADO vigente (aunque nunca se declare explícitamente un `UPDATE`), preguntarse "¿esta fila describe algo que YA PASÓ, o algo que ES cierto ahora mismo y podría cambiar mañana?" es la pregunta correcta para decidir `event_sequence_id` vs `row_version` — no hay ambigüedad real una vez que se hace esa pregunta.
- Calcular un "acumulado" como una consulta (`SUM(...) WHERE ...`) en vez de mantener un contador mutable aparte es un patrón general útil cuando la fuente de verdad ya es append-only: evita la necesidad de una segunda tabla/columna que podría desincronizarse del histórico real, a costa de un `SUM` en cada escritura (aceptable aquí porque el volumen por ciclo de un solo dueño es pequeño, no es un hot-path de microsegundos).
- Implementar un algoritmo de calendario pequeño y estable (como `civil_from_days`) a mano, en vez de sumar una dependencia externa completa (`chrono`) para una sola conversión, es una opción legítima cuando el algoritmo es corto, bien documentado (dominio público) y no va a cambiar — sopesa siempre "¿cuánto código realmente necesito?" contra "¿cuánto peso de dependencia estoy dispuesto a cargar?".
- Cuando un cimiento nuevo consume la función pública de OTRO cimiento del mismo crate, no hace falta ningún patrón especial de "adaptador" o "puerto" en tiempo de compilación — es una llamada de función normal (`build_plan_limits_for_tier(...)`), porque ambos viven bajo el mismo crate `shared` (ADR-0137: cada feature-crate depende solo de `shared`, y aquí ambas features SON `shared`). El "puerto" es el contrato de tipos (`PlanLimits`), no un mecanismo de invocación indirecta.
