# Data Anonymization & Aggregation

**Carpeta:** `./features/data-aggregation/`
**Estado:** En Diseño
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (cimiento #9) · ADR-0102 (anonimización) · ADR-0143 (tiers) · consentimiento

## ¿Qué es esta feature?

El puerto que toma datos crudos de ejecución (de usuarios cuyo tier o consentimiento lo permite), los **anonimiza** (privacidad diferencial + hash unidireccional, ADR-0102) y los **agrega** en índices vendibles (sentimiento, régimen, fricción de bróker, correlación). Ahora se entrega el **puerto + el registro de consentimiento**; el pipeline de agregación es un adaptador posterior.

- **Problema:** los datos de ejecución valen solo cuando se acumulan en volumen y solo se pueden vender anonimizados y con consentimiento. Si no se capturan desde el usuario #1, se pierden meses de historia.
- **Comportamiento observable:** el sistema convierte miles de ejecuciones individuales en índices agregados donde ningún usuario es reconocible.
- **Por qué:** es la base de todos los productos de datos alternativos.

## Comportamientos Observables

- Cuando llega un evento de ejecución de un usuario con consentimiento → se anonimiza (ruido + hash) antes de entrar a cualquier agregado.
- Cuando se consulta un índice agregado → refleja el consenso de N usuarios sin exponer a ninguno.
- Cuando un usuario ejerce opt-out → sus datos dejan de alimentar los agregados.
- Regla de oro: los datos crudos NUNCA salen hacia terceros; solo salen agregados anonimizados.

## Restricciones

- NUNCA sale un dato identificable hacia un tercero (ADR-0102, `consent-registry`).
- NUNCA se agrega un dato sin consentimiento vigente que lo cubra.
- PROHIBIDO producir un agregado que permita operar contra los propios usuarios (front-running — descartado por ADR-0144).
- El uso **interno** de datos crudos del tier gratuito (mejora, inteligencia, recreación con capital propio) es lícito por ToS (ADR-0143), pero sigue **separado** del canal de venta externa.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DP_NOISE_LEVEL | (definido) | rango | Ruido de privacidad diferencial en las métricas | CONFIG |
| MIN_COHORT_SIZE | (definido) | ≥ N | Tamaño mínimo de cohorte para publicar un agregado (k-anonimato) | FIJO |
| EXTERNAL_SALE_ENABLED | false | true/false | Habilita el canal de venta externa (requiere consentimiento) | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** anonimización (ruido gaussiano, hash unidireccional), agregación (índices de sentimiento/régimen/fricción/correlación), verificación de tamaño mínimo de cohorte.
- **Shell (Infraestructura):** lectura de eventos, verificación de consentimiento, persistencia de agregados, exposición por la API de terceros.
- **Frontera Pública:** puerto que expone los índices agregados; consumido por la API de terceros y los feeds de datos (moonshot `aggregated-data-feeds`).

## Ciclo de Vida de la Feature — Data Anonymization & Aggregation

### Entrada
Eventos de ejecución enriquecidos + el veredicto de consentimiento por usuario.

### Proceso
Anonimiza cada dato cubierto, lo suma a los agregados y verifica el tamaño mínimo de cohorte.

### Salida
Índices agregados anonimizados (sentimiento, régimen, fricción de bróker, correlación) listos para consumo interno o venta externa.

## Tareas (TTRs)

- **TTR-001:** Puerto de anonimización (Core: ruido + hash) con verificación de consentimiento.
- **TTR-002:** Agregación en índices con tamaño mínimo de cohorte (Core: k-anonimato).
- **TTR-003:** Separación estricta canal interno (crudo, tier gratuito) vs. canal externo (agregado, consentido).

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `event_in` | `EnrichedDomainEvent` (plomería, ADR-0144) | Input | `0..N` | Eventos de ejecución a anonimizar/agregar. |
| `consent_in` | `ConsentVerdict` (plomería, ADR-0144) | Input | `1..N` | Cobertura de consentimiento por usuario. |
| `aggregate_out` | `AggregatedIndex` (tipo técnico nuevo — plomería, ADR-0144) | Output | `1..N` | Índices agregados anonimizados. |

## Cáscara Visual (Thin Shell)

> Plomería (Ventana de Verificación). El UI-Designer escribe la nota de observable. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** la anonimización previa ocurre donde está el dato; solo el agregado viaja. Datos crudos nunca salen hacia terceros.
- **Inundación de Fundaciones (ADR-0020 V2):** Grupo I completo + **Perfil B (IA/R&D)** para los agregados (Identidad + Soberanía II + subset de Linaje III + Hardware IV) — el agregado es un producto de datos derivado.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Tabla de índices agregados con Grupo I + Perfil B. Campos propios fuera del catálogo (marcados): tipo de índice, ventana temporal, tamaño de cohorte, nivel de ruido aplicado. `STRICT`, UUIDv7 (ADR-0141). Referencia de linaje (`data_snapshot_id`) al conjunto fuente.

## Dependencias y Bloqueantes

- **Depende de:** `enriched-domain-events`, `consent-registry`, ADR-0102 (protocolo de anonimización).
- **Bloquea a:** `third-party-api-gateway` (feeds) y el moonshot `aggregated-data-feeds`.
- **Contrato de Integración UI (ADR-0117) — Ventana de Verificación:** su observable (índices agregados disponibles + tamaño de cohorte) queda visible en un panel de datos agregados; hasta entonces, deuda de integración registrada.
