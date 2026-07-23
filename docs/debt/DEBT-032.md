# DEBT-032 · El patrón de ledger que se copia tiene dos huecos, y se propagan

| Campo | Valor |
|---|---|
| **ID** | DEBT-032 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Gate de QA sobre STORY-051; el QA identificó correctamente que no es un defecto de esta Story |
| **Dominio** | Transversal — todos los ledgers append-only |

## Causa raíz

Cada feature nueva construye su ledger append-only copiando el patrón de una feature anterior. Es la práctica correcta y así se pidió explícitamente en STORY-051. El problema es que **el patrón de referencia tiene dos huecos**, y copiarlo fielmente los replica.

### Hueco 1 — La prueba companion del clasificador no prueba nada

El patrón exige tres pruebas acompañantes. La segunda debe verificar que el clasificador de errores distingue un conflicto de escritura **transitorio** (contención: hay que reintentar) de uno **permanente** (clave duplicada: reintentar es inútil y el bucle no termina nunca).

En `sovereign-data-fetcher` (STORY-046) esa prueba **nunca llama a la función que dice cubrir**: verifica el mensaje de un `INSERT` crudo, con un comentario auto-consciente que admite que se comprueba "indirectamente". STORY-051 copió el patrón fielmente, incluido el hueco. La mutación lo confirma: el clasificador puede mutarse a **devolver siempre "transitorio"** sin que ninguna prueba falle.

Consecuencia si el clasificador se rompiera: el ledger reintentaría en bucle una escritura que no puede tener éxito jamás, hasta agotar los intentos, y reportaría contención donde hay un error permanente de datos.

### Hueco 2 — Migraciones sin triggers anti UPDATE/DELETE

Seis de los ocho ledgers del repositorio (migraciones 0010, 0014, 0015, 0016, 0018, 0019, 0020) declaran triggers que hacen que la base de datos **rechace** cualquier modificación o borrado. Dos no: `sovereign-data-fetcher` (0006) y `hybrid-data-transformer` (0021).

STORY-051 heredó la ausencia precisamente porque se le indicó seguir el patrón de `sovereign-data-fetcher`. **La 0021 se corrige dentro de STORY-051** (disposición del Tech-Lead, 2026-07-18). La 0006 queda aquí.

Un ledger cuyo carácter inmutable depende solo de que nadie escriba el `UPDATE` equivocado no es append-only: es una convención de buena voluntad. La inmutabilidad debe estar en la base de datos, no en la disciplina del programador.

## Impacto actual

Ninguno vivo. Ningún código escribe `UPDATE` sobre esos ledgers, y ningún clasificador está roto hoy. Ambos son huecos de **detección**: si algo se rompiera, nada avisaría.

## Estado del molde (actualizado 2026-07-19)

**El molde ya está corregido y declarado canónico:** `.agents/knowledge/ledger-pattern.md`. Contiene los 7 puntos del patrón, las 4 pruebas obligatorias, y la historia de los dos huecos para que nadie los repita.

**Regla nueva, que es lo que de verdad cierra la causa raíz:** cuando una Orden diga "sigue el patrón de ledger", se refiere a ESE documento — nunca a "la feature que ya lo tiene implementado". Elegir referencia por antigüedad ("la primera que se construyó") es el peor criterio posible: fue exactamente lo que propagó estos dos huecos.

Lo que queda abierto en esta deuda es **el retrofit del código existente**, no el molde.

## ⚠️ El alcance real es mayor de lo que esta ficha decía (barrido 2026-07-19)

El retrofit de `sovereign-data-fetcher` se ejecutó y **el barrido de confirmación destapó que el marco de "ocho ledgers, dos con hueco" era falso**. Cifras verificadas de forma independiente por el Tech-Lead, no tomadas del reporte:

- **16 implementaciones** del clasificador en el repo, no 8.
- **21 migraciones** mencionan `event_sequence_id`.

### Hueco 1 (clasificador sin prueba directa) — tres niveles

| Nivel | Dónde | Gravedad |
|---|---|---|
| **Cero llamadas directas** — el hueco original completo | `shared/src/persistence/audit_log.rs` (0002), `shared/src/persistence/third_party_api_gateway.rs` (0014) | El de `0014` repite literalmente el patrón tautológico: verifica el mensaje de un INSERT crudo, nunca el clasificador |
| **1 de 3 casos canónicos** | `shared/src/persistence/mcp_gateway.rs` (0005) | Cubre "permanente"; falta el control no-BD y el transitorio |
| **2 de 3 casos** — falta el caso positivo con llamada directa | `data_portability.rs` (0019), `usage_metering.rs` (0010), `data_aggregation.rs` (0015), `verified_account_registry.rs` (0016), `master_account_hierarchy.rs` (0018), `operator_roles.rs` (0020), `job.rs` (0003), `consent_registry.rs` (0011), `instance_continuity.rs` (0017), `institutional_report_engine.rs` (0013), `enriched_domain_events.rs` (0012) | El caso transitorio solo se ejerce indirectamente vía la prueba de contención con candado real |

**Lo más incómodo:** `data_portability` (0019) es **la referencia limpia que `ledger-pattern.md` cita**, y tampoco cumple literalmente el punto 7 del molde en sus tres casos. El molde se escribió describiendo lo que la 0019 hacía bien (los candados), sin auditar lo que hacía a medias.

### Hueco 2 (candados) — un ledger más, no mencionado antes

`migrations/0004_telemetry.sql` — `telemetry_samples` tiene `event_sequence_id INTEGER NOT NULL UNIQUE` (es un ledger append-only) **sin candados anti UPDATE/DELETE**. Verificado por el Tech-Lead: es el único caso restante.

Peor: su repositorio **no implementa el patrón en absoluto** — sin `BEGIN IMMEDIATE`, sin clasificador, sin reintento. Eso no es un hueco del molde: es un ledger que nunca lo siguió, y roza el terreno de DEBT-001 (atomicidad). **Requiere verificación aparte** antes de decidir si se arregla aquí o se escala.

**Descartados tras verificación** (no son ledgers, y están correctamente documentados como mutables por ADR-0141): `0007_central_identity`, `0008_licensing_system`, `0009_plan_tier_quota` usan `row_version`, no `event_sequence_id UNIQUE`. `0001_foundation_master_fields` es la tabla ancla de referencia del contrato, mutable por diseño.

### Lección de método

**Se corrigió el molde apuntando a una referencia que tampoco se había auditado.** Es el mismo error de origen, un nivel más arriba: se eligió "la que se ve más limpia" en lugar de verificarla punto por punto. La regla de `ledger-pattern.md` —una referencia se declara Y se audita— aplicaba también al acto de escribir la propia regla.

## Qué hay que hacer

1. **Prueba real del clasificador en `sovereign-data-fetcher`**: ejercitar la función directamente con los casos que debe distinguir, y verificar por mutación que sobrevive a la inversión de su lógica.
2. **Triggers anti UPDATE/DELETE en la migración 0006**, siguiendo el patrón de la 0019, con la prueba que demuestra que la base rechaza modificación y borrado. Fase GREENFIELD: se edita in-situ (ADR-0006).
3. ~~**Corregir el patrón de referencia en la documentación**~~ — ✅ **hecho el 2026-07-19**: `.agents/knowledge/ledger-pattern.md`, declarado canónico e indexado.
4. Barrer los seis ledgers restantes para confirmar que su prueba del clasificador sí es discriminante y no comparte el mismo defecto.

## Disparador de pago

**Antes de construir el siguiente ledger append-only.** El coste de la deuda no es el hueco actual: es que cada feature nueva lo copia. Cuanto más tarde se corrija el patrón, más sitios hay que arreglar.
