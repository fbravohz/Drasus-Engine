# Fundamental Indicator Projector — Proyección de Evento a Indicador Estándar

**Carpeta:** `./features/fundamental-indicator-projector/`
**Estado:** En Diseño
**Última actualización:** 2026-06-18
**Decisión Arquitectónica Asociada:** ADR-0128 (Normalización por Instrumento), ADR-0125 (Frontera Determinista)

---

## ¿Qué es?

Es la pieza que convierte eventos puntuados en una **serie numérica de indicador** que el resto del motor consume **exactamente igual que cualquier indicador técnico** (RSI, ATR, etc.). Toma el coeficiente de impacto del [`event-impact-scorer`](./event-impact-scorer.md), lo combina con la relevancia por activo del [`asset-exposure-map`](./asset-exposure-map.md), y produce un valor **acotado y normalizado por instrumento**.

**Problema:** un coeficiente de impacto suelto no sirve a una estrategia. La estrategia necesita un indicador en el formato estándar, alineado a la rejilla temporal del activo, sin lógica especial. Y necesita que sea **relativo al activo**: la misma estrategia en dos activos recibe indicadores distintos, coherentes con la exposición de cada uno.

**Por qué la necesitamos:** es el puente que hace la fusión fundamental **invisible** para el hot-path — el motor solo lee una serie numérica más.

---

## Comportamientos Observables

- [ ] Una estrategia referencia el "indicador fundamental" igual que referencia un RSI → lo recibe como serie numérica estándar.
- [ ] El mismo evento global produce indicadores **distintos** en dos activos según su exposición (normalización por instrumento).
- [ ] El indicador está acotado (no se dispara a infinito ante un evento extremo) y alineado a la rejilla temporal del activo.
- [ ] El indicador solo agrega eventos cuya relevancia supera el umbral configurado para ese activo.
- [ ] En backtest, el indicador en la barra `t` solo refleja eventos publicados hasta `t` (PIT-correcto).

---

## Restricciones

- **NUNCA** se inyecta un valor fundamental global idéntico a todos los activos; siempre es relativo al instrumento.
- **NUNCA** el indicador en la barra `t` refleja un evento publicado después de `t` (sin look-ahead).
- **NUNCA** el hot-path ejecuta lógica fundamental: solo lee la serie ya proyectada (cero coste de latencia adicional).
- El indicador respeta el contrato estándar de indicador del motor (mismo formato que un indicador técnico).

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| NORMALIZATION_METHOD | configurable | — | Cómo se acota/normaliza el indicador por instrumento | CONFIG |
| DECAY_PROFILE | configurable | — | Cómo decae el efecto de un evento con el tiempo tras su publicación | CONFIG |
| AGGREGATION_MODE | configurable | — | Cómo se agregan varios eventos relevantes simultáneos | CONFIG |
| GRID_ALIGNMENT | configurable | — | Rejilla temporal a la que se alinea el indicador (la del activo) | CONFIG |

---

## Ciclo de Vida de la Feature — Fundamental Indicator Projector

### Entrada
- Coeficiente de impacto del evento ([`event-impact-scorer`](./event-impact-scorer.md)).
- Relevancia evento→activo ([`asset-exposure-map`](./asset-exposure-map.md)).
- Rejilla temporal del activo y parámetros de normalización/decaimiento.

### Proceso
- Filtra los eventos cuya relevancia supera el umbral del activo.
- Combina coeficiente × relevancia, aplica el perfil de decaimiento temporal y agrega eventos simultáneos.
- Normaliza/acota el resultado por instrumento y lo alinea a la rejilla temporal.

### Salida
- Serie numérica de indicador fundamental por activo, en el contrato estándar del motor.

### Contextos de Uso
**Contexto 1: Descubrimiento (Módulo Generate)**
- Las estrategias pueden incluir el indicador fundamental como una entrada más de su lógica.

**Contexto 2: Backtest (Módulo Validate)**
- El indicador se consume PIT-correcto sobre el histórico de eventos.

**Contexto 3: Ejecución (Módulo Execute)**
- El indicador se lee en tiempo real para confirmar o ponderar la decisión, sin lógica fundamental en el hot-path.

---

## Tareas (TTRs)

### TTR-001: Proyección a serie de indicador estándar
*   **¿Cuál es el problema?** Un coeficiente suelto no es consumible por una estrategia; hace falta una serie en formato estándar.
*   **¿Qué tiene que pasar?** El sistema combina impacto × relevancia, aplica decaimiento y agregación, y emite una serie alineada a la rejilla del activo, en el contrato estándar de indicador.
*   **¿Cómo sé que está hecho?**
    - [ ] Una estrategia referencia el indicador fundamental igual que un RSI y lo recibe.
    - [ ] El indicador está acotado y alineado temporalmente al activo.
*   **¿Qué no puede pasar?** Que el hot-path tenga que ejecutar lógica fundamental para leerlo.

### TTR-002: Normalización por instrumento (PIT-correcta)
*   **¿Cuál es el problema?** El mismo evento impacta distinto a cada activo; y el indicador no puede ver el futuro.
*   **¿Qué tiene que pasar?** El indicador es relativo al activo (normalizado por instrumento) y en la barra `t` solo refleja eventos publicados hasta `t`.
*   **¿Cómo sé que está hecho?**
    - [ ] Dos activos con la misma estrategia reciben indicadores distintos para el mismo evento.
    - [ ] En backtest, el indicador en `t` no contiene eventos posteriores a `t`.
*   **¿Qué no puede pasar?** Un valor global idéntico para todos los activos, o look-ahead de eventos.

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

**Perfil C. Ops / Hot-Path:** Identidad (I) + Soberanía (II) + Hardware (IV) + Latencia, subset Ejecución (V).

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la serie proyectada |
| | `created_at` | Timestamp de proyección |
| | `updated_at` | Timestamp de última modificación |
| | `audit_hash` | Hash de la serie proyectada |
| | `audit_chain_hash` | Hash encadenado a la proyección anterior |
| | `event_sequence_id` | Secuencia de recuperación |
| **II. Soberanía** | `owner_id` | Dueño de la proyección |
| | `institutional_tag` | Entorno (PROD/PAPER/CHALLENGE) |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de proyección |
| **V. Forense/Ejecución** | `indicator_state_hash` | Snapshot técnico del indicador fundamental por activo |
| | `source_signal_id` | Link a los eventos agregados en el valor |
| | `execution_latency_ms` | Latencia de lectura del indicador (hot-path ≤1ms) |

**Rastro de Evidencia:** emite a `feedback` qué eventos componen el valor del indicador en cada barra, para reconstruir la causalidad de una decisión basada en fundamentales.

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** combinación impacto × relevancia, decaimiento, agregación y normalización por instrumento — sin I/O.
- **Shell (Infraestructura):** carga de coeficientes y relevancias; persistencia de la serie alineada.
- **Frontera Pública:** contrato de indicador estándar, idéntico al de un indicador técnico, para consumo por `generate`, `validate` y `execute`.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% local.
- **Decisión Arquitectónica Asociada:** ADR-0128 (Normalización por Instrumento), ADR-0125 (Frontera Determinista), ADR-0020.

---

## Dependencias
**Depende de:**
- [`event-impact-scorer`](./event-impact-scorer.md) — para el coeficiente de impacto.
- [`asset-exposure-map`](./asset-exposure-map.md) — para la relevancia por activo.

**Consumido por:**
- [`generate`](../modules/generate.md) — como entrada de la lógica de estrategias.
- [`validate`](../modules/validate.md) — para backtest PIT-correcto.
- [`execute`](../modules/execute.md) — para lectura en tiempo real.
- [`manage`](../modules/manage.md) — para ponderar riesgo/posición.

**Contrato de Integración UI (ADR-0117):**
- **Ventana de Verificación:** superficie de `generate`/`validate` (selector de indicadores de estrategia). El observable concreto: el indicador fundamental aparece como serie seleccionable junto a los indicadores técnicos, con su valor por barra visible y persistido tras recargar.
