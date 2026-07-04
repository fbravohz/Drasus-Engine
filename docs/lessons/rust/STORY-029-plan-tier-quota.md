# STORY-029 — Plan / Tier / Quota: lecciones de Rust

> **Story:** [STORY-029 — Plan / Tier / Quota (cimiento #3 del substrato de monetización)](../../execution/STORY-029-plan-tier-quota.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0009_plan_tier_quota.sql`, `crates/shared/src/domain/plan_tier_quota.rs`, `crates/shared/src/persistence/plan_tier_quota.rs`, `crates/shared/src/orchestrator/plan_tier_quota.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Un "catálogo configurable" es dato, no código

La feature entera existe para resolver un problema muy concreto: si los precios y límites de cada tier estuvieran escritos como constantes en el código Rust (`const FREE_LIMIT: i64 = ...`), cambiar el precio de un plan exigiría recompilar y volver a desplegar el binario. Eso es lo opuesto a lo que un negocio necesita — el área comercial quiere poder decir "el plan Paid ahora permite el doble de nocional" sin depender de un ingeniero ni de un release.

La solución es que un "plan" sea una FILA en una tabla (`plans`, migración `0009_plan_tier_quota.sql`), no un valor incrustado en el binario. El código Rust no sabe, en tiempo de compilación, cuánto vale el límite nocional del plan Free — lo LEE de la base de datos en tiempo de ejecución:

```rust
// crates/shared/src/persistence/plan_tier_quota.rs
pub async fn find_latest_by_tier(&self, tier: PlanTier) -> Result<Option<Plan>, PlanRepositoryError> {
    let row = sqlx::query(
        "SELECT ... FROM plans WHERE tier = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(tier.as_str())
    .fetch_optional(self.pool)
    .await?;
    row.map(row_to_plan).transpose()
}
```

Esto es exactamente lo que ADR-0008 ("Configurabilidad Universal") pide: el código fija la MECÁNICA (cómo se valida un plan, cómo se resuelven sus límites), pero el VALOR de cada cuota vive en un dato editable. Hoy ese dato lo siembra un stub local (`seed_default_catalog`, ver más abajo); cuando exista la Cabina de Mando Central (ADR-0143), el mismo mecanismo leerá planes que un administrador definió remotamente, sin tocar una sola línea de Rust.

### Por qué los montos son enteros escalados y nunca `float`

`notional_limit` y `price` representan dinero (10 000 dólares, 49 dólares). La tentación natural es guardarlos como `f64` (`10000.00`) porque así se ven "naturales". El problema es que la aritmética de punto flotante NO representa la mayoría de los decimales exactamente en binario — `0.1 + 0.2` en casi cualquier lenguaje con floats da `0.30000000000000004`, no `0.3`. Sumar, comparar o persistir así montos de dinero acumula error silencioso: dos lecturas del "mismo" valor pueden dar números ligeramente distintos, y una diferencia de un centavo en un sistema de facturación es un bug real, no un detalle cosmético.

La solución de ADR-0141 es representar el dinero como un ENTERO, escalado por 10⁸ (cien millones): en vez de guardar `10000.00`, se guarda `1_000_000_000_000` (10 000 × 10⁸). Los enteros SÍ tienen aritmética exacta — sumar, comparar o guardar `1_000_000_000_000` nunca "se corre" ni un bit. La prueba discriminante de este criterio lo comprueba directamente:

```rust
// crates/shared/src/persistence/plan_tier_quota.rs
#[tokio::test]
async fn amounts_persist_and_reload_as_exact_integers_never_real() {
    let plan = repo.create(sample_free_plan()).await.expect("crear plan");
    assert_eq!(plan.notional_limit, 1_000_000_000_000);

    let reloaded = repo.find_by_id(&plan.id).await.expect("releer").expect("debe existir");
    assert_eq!(reloaded.notional_limit, 1_000_000_000_000, "round-trip debe ser exacto, sin deriva de float");
    // ...
    for row in columns {
        if name == "notional_limit" || name == "price" || name == "max_activations" {
            assert_eq!(column_type, "INTEGER", "la columna '{name}' nunca debe ser REAL");
        }
    }
}
```

La prueba no solo compara el valor releído contra el original — también inspecciona el TIPO declarado de la columna en SQLite (`pragma_table_info`) para confirmar que es `INTEGER`, no `REAL`. Esto cierra el hueco de "el valor de hoy coincide por casualidad, pero mañana alguien cambia la columna a `REAL` y nadie se entera": la prueba fallaría inmediatamente si eso ocurriera.

La migración refuerza esto con `CHECK`:

```sql
-- migrations/0009_plan_tier_quota.sql
notional_limit        INTEGER NOT NULL CHECK (notional_limit >= 0),
price                  INTEGER NOT NULL CHECK (price >= 0),
```

`STRICT` (SQLite ≥ 3.37, ya activo en toda tabla del proyecto) es lo que hace que declarar una columna `INTEGER` sea una promesa REAL, no una sugerencia — sin `STRICT`, SQLite acepta guardar cualquier tipo en cualquier columna ("type affinity" flexible); con `STRICT`, intentar insertar un `REAL` en una columna `INTEGER` es un error en el momento del INSERT, no un bug silencioso descubierto meses después.

### Qué es "resolver límites": separar el dato crudo del veredicto usable

"Resolver límites" significa: dado un plan ya cargado desde la base de datos (con sus columnas crudas — `notional_limit`, `max_activations`, `features_enabled` como texto JSON), producir el struct `PlanLimits` que el resto del sistema (`licensing-system`, y en el futuro `usage-metering`) puede consumir directamente, sin tener que saber nada sobre cómo se persiste un plan.

Esto se modela con DOS tipos y UNA función pura, deliberadamente separados:

```rust
// crates/shared/src/domain/plan_tier_quota.rs

// Lo mínimo que el Core necesita para resolver -- NO es la fila completa
// de la Shell (esa vive en persistence::plan_tier_quota::Plan).
pub struct PlanSnapshot<'a> {
    pub tier: PlanTier,
    pub notional_limit: i64,
    pub max_activations: i64,
    pub features_enabled: &'a [String],
}

// El veredicto que el puerto plan_limits_out expone.
pub struct PlanLimits {
    pub notional_limit: i64,
    pub max_activations: i64,
    pub features_enabled: Vec<String>,
}

pub fn resolve_limits(snapshot: &PlanSnapshot<'_>) -> PlanLimits {
    PlanLimits {
        notional_limit: snapshot.notional_limit,
        max_activations: snapshot.max_activations,
        features_enabled: snapshot.features_enabled.to_vec(),
    }
}
```

¿Por qué no le paso directamente la fila `Plan` (la de `persistence::plan_tier_quota::Plan`) a `resolve_limits`? Porque el Core (FCIS, ADR-0002) tiene prohibido depender de un tipo de la Shell — si `domain::plan_tier_quota` importara `persistence::plan_tier_quota::Plan`, la "lógica pura" quedaría acoplada a "cómo se guarda en SQLite", y cualquier cambio de esquema (renombrar una columna, añadir un campo de auditoría) obligaría a tocar el Core aunque la REGLA de negocio no haya cambiado en nada. `PlanSnapshot` es la frontera: la Shell (`orchestrator::plan_tier_quota::build_plan_limits_for_tier`) hace el trabajo de leer la fila de SQLite y traducirla a este snapshot mínimo; el Core solo ve esos tres campos y no sabe (ni le importa) que vinieron de una tabla.

La composición completa queda así, en la Shell:

```rust
// crates/shared/src/orchestrator/plan_tier_quota.rs
pub async fn build_plan_limits_for_tier(
    pool: &SqlitePool,
    clock: &dyn Clock,
    tier: PlanTier,
) -> Result<PlanLimits, BuildPlanLimitsError> {
    let repo = PlanRepository::new(pool, clock);
    let plan = repo.find_latest_by_tier(tier).await?.ok_or(BuildPlanLimitsError::PlanNotFound(tier.as_str()))?;

    Ok(resolve_limits(&PlanSnapshot {
        tier: plan.tier,
        notional_limit: plan.notional_limit,
        max_activations: plan.max_activations,
        features_enabled: &plan.features_enabled,
    }))
}
```

### Por qué la tabla `plans` es mutable (`row_version`, no `event_sequence_id`)

ADR-0141 distingue dos familias de tablas según qué representan:

- **Tablas append-only (event-store):** cada fila es un EVENTO que ya ocurrió — nunca se corrige, solo se agregan más filas. Usan `event_sequence_id UNIQUE` (una posición monótona global). Ejemplo: `audit_events`.
- **Tablas mutables:** cada fila es el ESTADO VIGENTE de una entidad, que cambia con el tiempo. Usan `row_version` (un contador que empieza en 1 y sube con cada UPDATE de ESA fila específica).

Un plan es, por definición, mutable: la Feature dice explícitamente "cuando se cambia el límite de un plan, las licencias de ese plan lo reflejan en la siguiente revalidación" (`docs/features/plan-tier-quota.md` "Comportamientos Observables"). Eso significa que el precio de HOY del plan Paid puede no ser el precio de MAÑANA, sobre la MISMA fila — no se crea un plan nuevo cada vez que cambia un número, se actualiza el existente. Por eso la migración usa `row_version`, no `event_sequence_id`:

```sql
-- migrations/0009_plan_tier_quota.sql
row_version        INTEGER NOT NULL,  -- Contador de versión de esta fila; arranca en 1, +1 en cada revisión
```

`PlanRepository::update_limits` es la operación que sube ese contador — y aquí aparece la pieza más delicada de todo el archivo: la concurrencia optimista.

### Concurrencia optimista: por qué el UPDATE lleva `row_version` en el `WHERE`

Imaginemos dos procesos (o dos hilos) que leen el MISMO plan al mismo tiempo, ambos ven `row_version = 1`, y ambos deciden escribir un cambio distinto. Sin ninguna protección, el segundo UPDATE simplemente pisaría el resultado del primero, y nadie se enteraría — el sistema terminaría en un estado que ninguno de los dos escritores realmente quiso, y la cadena de auditoría (`audit_chain_hash`) quedaría bifurcada sin que quede registro de qué pasó.

La solución (ya usada en `licensing-system`, STORY-028, y repetida aquí) es que el UPDATE filtre por `id` **y** por el `row_version` que el escritor cree que es el vigente:

```rust
// crates/shared/src/persistence/plan_tier_quota.rs
let result = sqlx::query(
    "UPDATE plans SET updated_at = ?, ..., row_version = ?, notional_limit = ?, ... \
     WHERE id = ? AND row_version = ?",
)
// ...
.bind(&plan.id)
.bind(plan.row_version)   // el row_version que ESTE escritor leyó, no el que hay ahora en disco
.execute(self.pool)
.await?;

if result.rows_affected() == 0 {
    return Err(PlanRepositoryError::VersionConflict { id: plan.id.clone(), expected: plan.row_version });
}
```

Si otro escritor ya adelantó la fila a `row_version = 2` entre que este escritor leyó y escribió, el `WHERE id = ? AND row_version = 1` no encuentra NINGUNA fila que actualizar (la fila real ya está en `row_version = 2`) — `rows_affected()` devuelve `0`, y el código lo interpreta como "perdiste la carrera, vuelve a leer y reintenta" en vez de asumir que todo salió bien. La prueba discriminante de este criterio demuestra el efecto exacto: dos "vistas" del mismo plan en memoria, la primera actualización tiene éxito, la segunda (que sigue creyendo estar en la versión vieja) recibe `VersionConflict`, y la fila en disco conserva el cambio del PRIMERO, no del segundo:

```rust
#[tokio::test]
async fn concurrent_updates_from_same_version_conflict_instead_of_overwriting() {
    let updated = repo.update_limits(&first_writer_view, 5_000_000_000_000, 1, 0).await.expect("...");
    assert_eq!(updated.row_version, 2);

    let conflict = repo.update_limits(&second_writer_view, 9_000_000_000_000, 1, 0).await;
    assert!(matches!(conflict, Err(PlanRepositoryError::VersionConflict { expected: 1, .. })));
}
```

La lección general (independiente de Rust): "concurrencia optimista" se llama así porque no bloquea nada por adelantado (no hay un `LOCK` explícito reteniendo la fila mientras alguien piensa) — apuesta a que los conflictos son raros, y cuando SÍ ocurren, los detecta después del hecho comparando versiones, en vez de prevenirlos con un candado costoso que ralentizaría a todos los lectores/escritores todo el tiempo.

### Codificación determinista de un conjunto sin orden natural (`features_enabled`)

`features_enabled` es una LISTA de nombres de features habilitadas para un plan. Pero una lista, en el sentido de "conjunto de cosas habilitadas", no tiene un orden que importe al negocio — `["vps_headless", "priority_support"]` y `["priority_support", "vps_headless"]` significan EXACTAMENTE lo mismo. El problema es que, si se persiste tal cual como JSON, dos inserciones del "mismo" conjunto en distinto orden producirían DOS textos distintos guardados en la columna — lo cual rompe el principio de determinismo (ADR-0002/0004: mismo dato lógico, misma representación física, siempre).

La solución es una función pura que ORDENA alfabéticamente y elimina duplicados ANTES de serializar a JSON:

```rust
// crates/shared/src/domain/plan_tier_quota.rs
pub fn canonical_features_json(features: &[String]) -> String {
    let mut sorted: Vec<&str> = features.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.dedup();
    serde_json::to_string(&sorted).expect("Vec<&str> siempre serializa a JSON válido")
}
```

`sort_unstable()` ordena el vector en su lugar (más rápido que `sort()` porque no garantiza mantener el orden relativo de elementos iguales — algo irrelevante aquí porque los strings duplicados son, por definición, indistinguibles entre sí). `dedup()` SOLO elimina duplicados CONSECUTIVOS — por eso tiene que ir DESPUÉS de ordenar: una lista `["b", "a", "b"]` sin ordenar tiene los dos `"b"` separados por una `"a"`, y `dedup()` no los vería como consecutivos; ordenada (`["a", "b", "b"]`), sí. La prueba discriminante confirma que el orden de entrada no afecta el resultado:

```rust
#[test]
fn canonical_features_json_is_order_independent() {
    let a = vec!["vps_headless".to_string(), "advanced_backtest".to_string()];
    let b = vec!["advanced_backtest".to_string(), "vps_headless".to_string()];
    assert_eq!(canonical_features_json(&a), canonical_features_json(&b));
}
```

Se eligió JSON-de-lista-en-una-columna-`TEXT` en vez de una tabla hija M:N (el "Patrón M:N" que documenta ADR-0141) porque el conjunto de features de un plan no tiene atributos propios (ni peso, ni fecha de alta, ni ninguna otra columna que justificaría una fila aparte) — es, en esencia, una etiqueta compuesta del plan mismo, no una relación con entidad propia.

### Cuando dos Stories necesitan "el mismo" tipo pero una ya está sellada

El catálogo de tipos de puerto (ADR-0137) dice que `PlanLimits` es el tipo que ESTA Feature produce. Pero `licensing-system` (STORY-028, cimiento #2) se construyó ANTES de que este cimiento existiera, y ya declaró su PROPIO `PlanLimits` como marcador temporal (`domain::licensing_system::PlanLimits`, con solo dos campos, sin `notional_limit`). Esa Story ya está cerrada, con sus propios tests en verde usando construcciones literales como `PlanLimits { max_activations: 3, features_enabled: vec![] }` — si el `PlanLimits` de `plan-tier-quota` se llamara IGUAL y se intentara exponer al mismo nivel plano de `public_interface.rs`, Rust rechazaría el archivo entero con un error de nombre duplicado (`E0255`), Y la Orden de esta Story prohíbe expresamente arreglar eso tocando el código sellado de licensing-system (ese re-cableado es un "follow-up de integración" aparte, con su propio QA).

La solución práctica: el `PlanLimits` real de esta Story vive bajo su propio sub-módulo público, en vez de aplanarse junto a los demás tipos de `public_interface.rs`:

```rust
// crates/shared/src/public_interface.rs
pub mod plan_tier_quota {
    pub use crate::domain::plan_tier_quota::{ ..., PlanLimits, ... };
    // ...
}
```

Quien necesite el `PlanLimits` REAL lo importa como `shared::public_interface::plan_tier_quota::PlanLimits`; el stub viejo sigue disponible, sin cambios, como `shared::public_interface::PlanLimits`. El día que el follow-up de integración re-cablee `licensing-system` para consumir el real, el stub se retira y ambos convergen — pero eso es trabajo de otra Story, no de esta. La lección general: cuando dos piezas de un sistema modelan el MISMO concepto de negocio en momentos distintos del desarrollo, unificarlas de inmediato no siempre es seguro ni está autorizado — a veces la solución correcta A CORTO PLAZO es dejarlas convivir bajo rutas distintas, documentando explícitamente la deuda, en vez de forzar una unificación que reabre código ya probado y cerrado.

### Una caché con TTL, pero con una llave (a diferencia de sus hermanas)

`IdentityCache` (STORY-027) y `ExecutionGateCache` (STORY-028) guardan un solo valor: tiene sentido, porque una instancia tiene una sola identidad y una sola licencia activa. Pero `plan-tier-quota` puede necesitar resolver Free Y Paid en la misma corrida (por ejemplo, para comparar planes en una futura pantalla de upgrade) — un solo valor cacheado no alcanza. La solución es la misma IDEA (un `Mutex` en memoria + comparación contra el reloj inyectado), pero con un `HashMap` en vez de un único `Option`:

```rust
// crates/shared/src/orchestrator/plan_tier_quota.rs
pub struct PlanLimitsCache {
    clock: Arc<dyn Clock>,
    config: PlanLimitsCacheConfig,
    entries: StdMutex<HashMap<PlanTier, CachedPlanLimits>>,
}

pub fn get(&self, tier: PlanTier) -> Option<PlanLimits> {
    let now_ns = self.clock.timestamp_ns();
    let guard = self.entries.lock().expect("...");
    match guard.get(&tier) {
        Some(cached) if now_ns - cached.cached_at_ns < self.config.ttl_ns => Some(cached.limits.clone()),
        _ => None,
    }
}
```

`PlanTier` necesita derivar `Hash` (además de `Eq`) para poder ser llave de un `HashMap` — sin eso, el compilador rechaza el `HashMap<PlanTier, _>` con un error pidiendo la implementación de `Hash`. La prueba discriminante confirma que las entradas son independientes entre sí: expirar Free no debe afectar a Paid, aunque compartan la misma instancia de caché:

```rust
#[test]
fn plan_limits_cache_keys_entries_independently_per_tier() {
    cache.set(PlanTier::Free, sample_limits());
    det_clock.advance(500);
    cache.set(PlanTier::Paid, paid_limits.clone());
    det_clock.advance(600); // Free ya lleva 1_100ns (expirado); Paid solo 600ns (vigente).

    assert_eq!(cache.get(PlanTier::Free), None, "Free debe haber expirado");
    assert_eq!(cache.get(PlanTier::Paid), Some(paid_limits), "Paid sigue vigente de forma independiente");
}
```

## Trucos de Senior

- Cuando una función pura necesita convertir un `Vec<&str>` a JSON y ese `Vec` no puede fallar en serializar (no tiene claves de mapa raras ni ciclos), un `.expect("...")` documentado con el porqué es preferible a propagar un `Result` que, en la práctica, nunca puede ser `Err` — propagar un error imposible solo obliga a quien llama a manejar un caso que jamás ocurre.
- `sort_unstable()` + `dedup()` es el patrón idiomático de Rust para "conjunto determinista sin usar `HashSet`" cuando además se necesita un ORDEN canónico de salida (un `HashSet` no garantiza ningún orden de iteración, lo cual sería tan no-determinista como no ordenar en absoluto). El orden de las dos llamadas importa: `dedup()` solo colapsa duplicados CONSECUTIVOS, así que siempre va después de `sort`, nunca antes.
- Cuando dos structs de un mismo crate necesitan compartir un nombre de tipo pero uno de los dos ya está congelado por una Story anterior, un sub-módulo público (`pub mod nombre_feature { pub use ...; }`) dentro del mismo archivo de interfaz pública es una forma barata de aislar el nuevo tipo sin renombrarlo ni tocar el código viejo — el costo es que quien lo consume debe escribir la ruta calificada (`plan_tier_quota::PlanLimits`) en vez del nombre desnudo, un precio pequeño comparado con reabrir una Story ya cerrada y auditada.
- Derivar `Hash` en un enum simple (`#[derive(Hash)]` junto a `PartialEq, Eq`) es gratis en términos de código (una línea) pero habilita usarlo como llave de `HashMap`/`HashSet` — vale la pena añadirlo por adelantado en cualquier enum pequeño que pueda terminar siendo una llave de caché o índice, en vez de esperar a que el compilador lo exija y tener que volver a tocar la definición del tipo.
