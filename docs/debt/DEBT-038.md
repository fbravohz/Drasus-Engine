# DEBT-038 · El check M13 nace incumplido por siete columnas anteriores a él

| Campo | Valor |
|---|---|
| **ID** | DEBT-038 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-27 |
| **Origen** | Barrido del Tech-Lead inmediatamente después de que el Architect canonizara el check `M13` (ADR-0141, enmienda 2026-07-27, TASK-072) |
| **Disparador de pago** | El próximo trabajo que toque cada migración implicada, o un barrido único en EPIC-1 antes del congelamiento a BROWNFIELD |

---

## El hecho

`M13` (ADR-0141) exige que **todo campo timestamp que no sea `created_at`/`updated_at` lleve el sufijo `_ns`** en columna SQL, struct Rust y puerto, sin excepción. La regla se escribió para que el descuido de `venue_operational_since` (TASK-072) lo cace un checklist y no un barrido manual.

El barrido de comprobación encontró que **el check nace incumplido por siete columnas más**, todas anteriores a la regla:

| Migración | Columna | Unidad real según su comentario |
|---|---|---|
| `0003_jobs.sql` | `completed_at` | Nanosegundos desde epoch (puerto Clock) |
| `0008_licensing_system.sql` | `issued_at` | Nanosegundos UTC |
| `0008_licensing_system.sql` | `heartbeat_expires_at` | Nanosegundos UTC |
| `0011_consent_registry.sql` | `accepted_at` | Nanosegundos UTC |
| `0017_instance_continuity.sql` | `snapshot_at` | Nanosegundos UTC |
| `0022_instrument_session_catalog.sql` | `expiration_date` | Nanosegundos UTC |
| `0022_instrument_session_catalog.sql` | `contract_expiration_date` | Nanosegundos UTC |

`0023.accepted_at` está en la misma situación pero **no entra en esta deuda**: se corrigió junto al retrofit de TASK-072 (2026-07-27), por ser el mismo crate y la misma tanda.

**Dato que conviene tener presente al pagarla:** en `0011_consent_registry` la desalineación es **solo de columna**. El lado Rust ya sella la unidad y lo hace explícito en la traducción — `accepted_at_ns: row.get("accepted_at")`. Es decir, quien escribió ese código ya sabía que el nombre de la columna se quedaba corto y lo compensó en su frontera. Refuerza el diagnóstico: el patrón se corrige donde alguien mira, y sobrevive donde nadie mira.

## Por qué importa, y por qué no es urgente

Los dos casos con nombre de "fecha" son los de riesgo real: `expiration_date` y `contract_expiration_date` **invitan a leerse como una fecha civil** (`AAAAMMDD` o días desde epoch), no como un instante en nanosegundos. Es la misma trampa de TASK-072 con el agravante de que el nombre sugiere activamente la lectura equivocada. Ambos son de futuros — el vencimiento de contrato gobierna el rolo, y equivocarse de unidad ahí no da error, da un rolo en la fecha incorrecta.

Los cinco `_at` restantes son de menor riesgo: el sufijo `_at` ya se lee por convención como "instante", y ninguno cruza fronteras de feature como parámetro suelto. Pero la regla no admite grados, y dejarlos fuera convierte `M13` en una norma con excepciones tácitas — que es exactamente la forma en que una convención deja de detectar nada.

**No es urgente porque ninguno está mal hoy:** todos guardan nanosegundos y todos los consumidores los leen así. Lo que falta es que el nombre lo diga.

## Qué habría que hacer

Renombrado puro, sin cambio de comportamiento ni de tipo: columna SQL (edición in-situ, fase GREENFIELD), structs Rust, puertos y fichas de `docs/data-model/`. No añade ni elimina mutantes — basta verificar que el recuento no varía.

Conviene hacerlo **en una sola pasada antes del congelamiento a BROWNFIELD**: después, cada renombrado deja de ser una edición in-situ y pasa a ser una migración incremental sobre datos de usuarios reales, por un defecto puramente cosmético. El coste de la deuda no crece con el tiempo — se dispara de golpe en la fecha del primer release distribuido.

## Relación con otras deudas

- Causa directa: [TASK-072](../execution/TASK-072-architect-escalation-unidad-venue-operational-since.md), que canonizó `M13`.
- Misma familia que la lección de unidades de [STORY-068](../execution/STORY-068-mt5-bar-reader.md) (el spread en puntos, no en pips): el código correcto y el dato mudo.
