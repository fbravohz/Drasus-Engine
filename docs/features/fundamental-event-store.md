# Fundamental Event Store — Almacén PIT de Eventos Fundamentales

**Carpeta:** `./features/fundamental-event-store/`
**Estado:** En Diseño
**Última actualización:** 2026-06-18
**Decisión Arquitectónica Asociada:** ADR-0126 (Sourcing y Soberanía), ADR-0127 (PIT de Eventos)

---

## ¿Qué es?

Es la puerta de entrada y el almacén de los **eventos del mundo**: publicaciones macro programadas (PIB, empleo, inflación), resultados corporativos, decisiones de política monetaria, cambios de calificación. Recibe el **hecho crudo** desde un proveedor estructurado externo, lo guarda **localmente con su linaje** y lo conserva con corrección Point-In-Time: cada evento queda sellado con su **instante exacto de publicación** y con **todas sus versiones** (el primer dato publicado y las revisiones posteriores), sin que una versión nueva borre a la anterior.

**Problema:** un backtest fundamental miente en silencio si usa una cifra revisada meses después o si "ve" la noticia antes de que se publicara. Este almacén impide ambas cosas: guarda *qué se sabía y exactamente cuándo*.

**Por qué la necesitamos:** sin un almacén PIT de eventos, no hay forma honesta de medir el impacto de una noticia histórica ni de reproducirlo.

---

## Narrativa de Producto

**Capacidad:** `CAP-01` — Datos en los que se puede confiar

**Historia de usuario:** Como investigador de estrategias, quiero saber exactamente qué se sabía del mundo en cada instante del pasado —y no lo que sabemos hoy de ese instante— para poder medir el efecto real de una noticia sin hacer trampa con el diario de mañana.

**El problema (técnico):** los datos económicos se revisan. La cifra de empleo que se publica un viernes cambia semanas después, a veces sustancialmente. Casi todas las fuentes guardan **solo la versión final**, sobrescribiendo la original. Un estudio hecho sobre esa fuente usa, sin saberlo, una cifra que en su momento nadie tuvo — y produce un resultado excelente e irreproducible. El problema tiene dos caras: el **instante** de publicación (ver la noticia antes de que existiera) y la **versión** del dato (ver la corrección antes de que se corrigiera).

**La solución (técnica):** un almacén que guarda el hecho crudo con su linaje de proveedor y lo versiona: la primera publicación y cada revisión posterior conviven como registros separados, ninguna sobrescribe a la anterior, y cada una lleva sellado su instante exacto de publicación. Una consulta por una fecha del pasado devuelve lo que existía entonces. El sistema **no puntúa ni interpreta el evento aquí** — eso vive aparte, por fórmula declarada, para que el hecho crudo permanezca separado de la opinión sobre el hecho.

**Explicado sin tecnicismos:** los datos económicos se corrigen. El gobierno publica una cifra de empleo un viernes, y tres meses después dice "en realidad era otra".

Casi todas las bases de datos guardan **solo la corregida** y borran la original. Y ahí está la trampa: si estudias qué habrías hecho aquel viernes usando la cifra corregida, estás usando información que ese viernes no existía. Tu estrategia parece adivina. No adivinaba: leyó el periódico de tres meses después.

Esta pieza guarda **todas las versiones, con la fecha exacta en que cada una se publicó**. Cuando el sistema pregunta "¿qué se sabía aquel viernes?", recibe lo que se sabía aquel viernes. Ni más.

## Comportamientos Observables

- [ ] El usuario conecta una fuente estructurada (calendario macro / resultados) → el sistema descarga el hecho crudo y lo guarda con su linaje (proveedor, instante de publicación, licencia, latencia declarada).
- [ ] Una cifra macro se revisa semanas después → el sistema guarda la revisión como **versión nueva** vinculada al evento, y conserva el *first-print* intacto.
- [ ] Pregunto "¿qué se sabía de este evento a fecha del evento?" → obtengo el *first-print* y el consenso previo, no la cifra corregida.
- [ ] Un consumidor (backtest) intenta leer un evento antes de su instante de publicación → el sistema lo impide (sin look-ahead).
- [ ] Si la fuente externa desaparece → los eventos ya ingestados y su linaje siguen disponibles localmente.

---

## Restricciones

- **NUNCA** una versión nueva (revisión) sobrescribe el *first-print*; se almacena como versión adicional.
- **NUNCA** un evento es visible para un consumidor antes de su instante de publicación.
- **NUNCA** se persiste un score, sentimiento o señal **interpretada** por un tercero; solo el hecho crudo medible (cifra real, consenso, fecha/hora, entidad).
- **NUNCA** un evento entra sin linaje completo (proveedor, instante de publicación, licencia).
- Todo descarte o rechazo de un evento crudo se registra (nunca silencio).

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ACTIVE_SOURCES | configurable | 1-N | Qué proveedores estructurados se conectan | CONFIG |
| SOURCE_PRIORITY | por clase de activo | — | Orden de preferencia de fuentes por clase de activo | CONFIG |
| RETAIN_REVISIONS | true | true/false | Si conserva todas las revisiones (vintage) o solo first-print | [FIJO] (true) |
| PUBLICATION_TZ_NORMALIZE | true | true/false | Normaliza el instante de publicación a reloj interno determinista | CONFIG |

---

## Ciclo de Vida de la Feature — Fundamental Event Store

### Entrada
- Hecho crudo de un proveedor estructurado externo (evento con entidad, fecha/hora, cifra real, consenso previo cuando aplica).
- Metadatos de linaje del proveedor (origen, licencia, latencia declarada).

### Proceso
- Normaliza el instante de publicación a un reloj determinista.
- Sella el evento como first-print; si llega una revisión del mismo evento, la guarda como versión nueva enlazada.
- Aplica la guardia PIT para garantizar que el evento no se exponga antes de su instante de publicación.
- Registra el linaje completo.

### Salida
- Evento fundamental almacenado, versionado (as-of) y consultable por instante.
- Respuesta a consultas "as-of": la versión vigente del dato en una fecha dada.

### Contextos de Uso
**Contexto 1: Ingesta (Módulo Ingest)**
- Entrada: hecho crudo del proveedor. Pregunta: ¿es PIT-correcto y tiene linaje? Impacto: solo entra dato trazable y sin look-ahead.

**Contexto 2: Scoring (Feature event-impact-scorer)**
- Entrada: evento versionado + serie de precio. Pregunta: ¿qué se sabía en el instante del evento? Impacto: alimenta el cálculo determinista del impacto con el first-print.

---

## Tareas (TTRs)

### TTR-001: Ingesta del hecho crudo con linaje
*   **¿Cuál es el problema?** Un evento sin origen ni instante de publicación no se puede auditar ni usar PIT.
*   **¿Qué tiene que pasar?** Cada evento descargado queda guardado con proveedor, instante de publicación, licencia y latencia declarada.
*   **¿Cómo sé que está hecho?**
    - [ ] Descargo un calendario y veo cada evento con su linaje completo.
    - [ ] Un evento sin instante de publicación es rechazado y registrado.
*   **¿Qué no puede pasar?** Persistir un evento sin linaje, o persistir un score interpretado por el tercero.

### TTR-002: Versionado vintage / as-of (first-print + revisiones)
*   **¿Cuál es el problema?** Las cifras macro se revisan; usar la revisión en un backtest del evento original es look-ahead.
*   **¿Qué tiene que pasar?** La revisión se guarda como versión nueva; el first-print queda intacto; puedo reconstruir "qué se sabía a fecha T".
*   **¿Cómo sé que está hecho?**
    - [ ] Inyecto una revisión y el first-print sigue consultable.
    - [ ] Una consulta as-of a la fecha del evento devuelve el first-print, no la revisión.
*   **¿Qué no puede pasar?** Que una revisión sobrescriba el first-print.

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

**Perfil A. Datos / Ingest:** Identidad (I) + Linaje (III) + Hardware (IV).

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del registro de evento |
| | `created_at` | Timestamp de ingesta del registro |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del contenido del evento |
| | `audit_chain_hash` | Hash encadenado al registro anterior |
| | `event_sequence_id` | Secuencia de recuperación post-crash |
| **III. Linaje** | `data_snapshot_id` | Snapshot PIT del evento (instante de publicación) |
| | `parent_id` | Puntero al first-print cuando este registro es una revisión |
| | `version_node_id` | Nodo de versión en el historial vintage del evento |
| | `transformation_id` | ID de la normalización aplicada al hecho crudo |
| | `logic_hash` | Hash del driver de ingesta del proveedor |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del job de ingesta |

**Rastro de Evidencia:** emite a `feedback` el linaje del evento y su versión vigente, para que la causalidad de una decisión pueda rastrearse hasta el hecho crudo exacto que la originó.

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** normalización del instante de publicación y resolución de versión vigente (as-of) — sin I/O.
- **Shell (Infraestructura):** driver del proveedor externo y persistencia local (Parquet/SQLite) con linaje.
- **Frontera Pública:** contrato para consultar un evento por instante/as-of y para suscribirse a eventos nuevos, expuesto en la `public_interface` de `ingest`.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** el hecho crudo se persiste local con linaje; el motor opera sobre lo ingestado sin estar en línea con el proveedor.
- **Decisión Arquitectónica Asociada:** ADR-0126 (Sourcing y Soberanía), ADR-0127 (PIT de Eventos), ADR-0020 (Inundación de Fundaciones).

---

## Dependencias
**Depende de:**
- [`pit-data-validator`](./pit-data-validator.md) — como guardia anti-look-ahead en la alineación temporal de eventos.

**Consumido por:**
- [`ingest`](../modules/ingest.md) — orquesta la ingesta y persistencia de eventos.
- [`event-impact-scorer`](./event-impact-scorer.md) — consume el evento versionado para puntuar su impacto.

**Contrato de Integración UI (ADR-0117):**
- **Ventana de Verificación:** Feature consumidora [`event-impact-scorer`](./event-impact-scorer.md) (y, aguas abajo, la superficie de `generate`/`validate`). El observable concreto de esta feature que debe quedar visible: el conteo de eventos ingestados con su instante de publicación y el indicador de versión vigente (first-print vs revisión) por evento, persistido y visible tras recargar.
