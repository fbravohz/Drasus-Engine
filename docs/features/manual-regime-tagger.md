# Manual Regime Tagger

**Carpeta:** `./features/manual-regime-tagger/`
**Estado:** En Diseño / Prioritario
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0057 (Glass-Box — el humano define el contexto crítico), ADR-0008 (Configurabilidad Universal)

---

## ¿Qué es?

Herramienta de **etiquetado visual manual de regímenes históricos**. Complementa la detección automática (`hmm-regime-detection`, `regime-guard`, `dtw-adaptive-window`): aquí el humano sombrea con el ratón periodos del gráfico y los etiqueta empíricamente ("Crash 2008", "Flash Crash 2010", "Lateral 2015", "Covid 2020"). Luego puede imponer una **regla dura por zona**: exigir que la estrategia mantenga un desempeño mínimo específicamente durante las zonas etiquetadas como crisis.

**Problema que resuelve:** SQX particiona In-Sample / Out-of-Sample a ciegas. Los Quants saben que no todos los años son iguales; un periodo de crisis no debe diluirse en el promedio.

**Por qué la necesitamos:** El humano aporta contexto empírico que ningún clasificador automático garantiza, y restringe drásticamente el campo de acción de la generación de estrategias.

---

## Comportamientos Observables

- [ ] El usuario sombrea con el ratón un rango de fechas en el gráfico de un activo de referencia y le pone una etiqueta de texto.
  → El periodo queda marcado visualmente con su nombre (ej: "Crisis Covid 2020").
- [ ] El usuario añade una regla dura por zona vía UI: exigir un desempeño mínimo (ej: Profit Factor por encima de un umbral) específicamente dentro de las zonas etiquetadas como crisis.
  → La regla queda activa en el embudo de robustez.
- [ ] Durante la validación, una estrategia que no cumple la regla en las zonas etiquetadas es rechazada o degradada.
  → El usuario ve qué zona específica la reprobó.
- [ ] Las etiquetas son reutilizables: una vez definidas, se aplican a futuras validaciones del mismo activo.

---

## Ciclo de Vida de la Feature — Manual Regime Tagger

### Entrada
- Serie histórica de un activo de referencia para sombrear.
- Rangos de fechas y etiquetas definidos por el humano.
- Reglas duras por zona (umbral de desempeño mínimo exigido en cada etiqueta).

### Proceso
- Persiste las zonas etiquetadas como metadatos del activo/proyecto.
- Al validar, recorta la curva de la estrategia a cada zona etiquetada y evalúa la regla dura correspondiente.
- Marca aprobación/rechazo por zona.

### Salida
- Conjunto de zonas etiquetadas reutilizables.
- Veredicto por zona crítica (CUMPLE / NO CUMPLE).
- Restricción efectiva sobre el embudo de robustez.

### Contextos de Uso
**Contexto 1: Ingesta (Módulo Ingest)**
- Define las zonas de régimen empírico que acompañarán al dataset.
**Contexto 2: Validación (Módulo Validate)**
- Aplica las reglas duras por zona como filtro de robustez.

---

## Restricciones

- NUNCA una zona etiquetada modifica los datos: es solo metadato de intervalo temporal.
- NUNCA una estrategia pasa la validación si incumple una regla dura activa en una zona crítica (salvo que la regla se desactive explícitamente).
- Las zonas pueden solaparse, pero cada regla dura se evalúa de forma independiente por etiqueta.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ZONE_MIN_METRIC | profit_factor > 1.0 | métrica + umbral | Regla dura exigida dentro de una zona | CONFIG |
| REFERENCE_ASSET | índice principal | cualquier activo | Activo sobre el que se sombrea | CONFIG |
| ZONE_RULE_ENFORCEMENT | rechazo | rechazo / degradación | Qué ocurre si se incumple la regla | CONFIG |
| ZONE_LABELS | vacío | lista editable | Catálogo de etiquetas del usuario | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Evaluación de reglas duras sobre subconjuntos temporales de la curva, sin IO.
- **Shell (Infraestructura):** Persiste zonas/etiquetas y las inyecta en el embudo de validación.
- **Frontera Pública:** Contrato que recibe curva + zonas etiquetadas y devuelve veredicto por zona.

---

## Slice Visual (Flutter / Impeller / FFI)
- Gráfico del activo de referencia con selección de rango por arrastre (brush) renderizado con Impeller.
- Editor de etiquetas y de reglas duras por zona; eventos cruzan FFI hacia el Core Rust.
- Persistencia de zonas vía `duckdb-sql-engine`.
- Modo Headless (SaaS): frontera por gRPC.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** Hereda la del backtest que evalúa cada zona.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2 · Perfil Ops/Auditoría)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador de la zona etiquetada |
| | `created_at` | Timestamp de creación de la etiqueta |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del intervalo + regla dura |
| | `audit_chain_hash` | Hash encadenado del historial del registro |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Analista que definió la zona |
| | `manifest_id` | Estrategia evaluada contra la zona |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la regla dura aplicada |

- **Rastro de Evidencia:** Emite a `feedback` qué zona crítica reprobó cada estrategia (causa del rechazo).

---

## Dependencias
**Consumido por:** `ingest`, `validate`.
**Depende de:** `duckdb-sql-engine`, `equity-curve-tracker`, `binary-arrow-transport`.
**Relación:** Complementa (no reemplaza) la detección automática de `hmm-regime-detection` y `regime-guard`.
**Bloqueantes:** Ninguno.

---

## Tareas (TTRs)

### TTR-001: Sombrear y etiquetar periodos históricos
* **¿Cuál es el problema?** El analista necesita marcar empíricamente periodos especiales (crisis, laterales) que el dataset no distingue por sí solo.
* **¿Qué tiene que pasar?** El usuario arrastra el ratón sobre un rango de fechas del gráfico y le asigna una etiqueta; la zona queda marcada y guardada.
* **¿Cómo sé que está hecho?**
  - [ ] Sombreo marzo 2020 y lo etiqueto "Covid 2020"; la zona aparece marcada con su nombre.
  - [ ] La zona persiste y reaparece al reabrir el proyecto.
* **¿Qué no puede pasar?** No puede alterarse el dato histórico subyacente; la zona es solo metadato.

### TTR-002: Imponer regla dura de desempeño por zona
* **¿Cuál es el problema?** El analista quiere exigir que la estrategia aguante específicamente en las crisis, no solo en promedio.
* **¿Qué tiene que pasar?** El usuario define un desempeño mínimo dentro de una zona etiquetada y esa regla entra al embudo de robustez.
* **¿Cómo sé que está hecho?**
  - [ ] Configuro "Profit Factor > 1.0 en zonas Crisis" y la regla queda activa.
  - [ ] Una estrategia que pierde en la zona Covid es rechazada/degradada según la configuración.
* **¿Qué no puede pasar?** No puede aprobarse una estrategia que incumple una regla dura activa en zona crítica.

### TTR-003: Veredicto por zona visible en validación
* **¿Cuál es el problema?** El analista necesita saber exactamente qué crisis reprobó a la estrategia.
* **¿Qué tiene que pasar?** La validación muestra, por cada zona etiquetada, si la estrategia cumple o no la regla.
* **¿Cómo sé que está hecho?**
  - [ ] Veo un veredicto CUMPLE/NO CUMPLE por cada zona etiquetada.
  - [ ] El veredicto por zona se emite a feedback.
* **¿Qué no puede pasar?** No puede ocultarse cuál zona específica causó el rechazo.
