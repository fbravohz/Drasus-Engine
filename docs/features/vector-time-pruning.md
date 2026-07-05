# Vector-Time Pruning (Poda Temporal Autónoma)

**Carpeta:** `./features/vector-time-pruning/`
**Estado:** En Diseño
**Decisiones Arquitectónicas Asociadas:** ADR-0046, ADR-0020, ADR-0108, ADR-0110

---

## ¿Qué es? (Explicado Simple)

Imagina un robot trader que gana dinero consistentemente casi toda la semana, pero por alguna razón, **siempre** pierde dinero los viernes a las 2:00 PM cuando hay anuncios de la Reserva Federal (Noticias Macro). O peor: imagina un robot que tiene un rendimiento falso debido a un "Cisne Negro Positivo" (un golpe de suerte monumental) que distorsiona toda su curva de ganancias.

En el pasado, descartarías al robot entero, o te engañarías creyendo que el golpe de suerte es la norma. El **Vector-Time Pruning** actúa como un cirujano en dos frentes (New Era):
1. **Vector Excision (Poda Quirúrgica de Anomalías):** Extirpa las operaciones con ganancias irreales (`Z-score Profit > +3`) para recalcular una curva basada puramente en ventaja estructural, no en suerte.
2. **Excision Macro-Eventos & Contagio Sintético:** Amputa operativas situadas a ±15 minutos de eventos NFP/FOMC e inyecta aislamientos asincrónicos cruzados indexados en picos de volatilidad (VIX > +3 STDDev).

Convierte datos estadísticos anómalos (positivos o negativos) en vetos físicos, inyectados directamente en el motor de ejecución y backtesting.

**Precedente del Veto Vectorial para la Máscara del Genoma de Régimen y Filtro de Entorno (ADR-0108/ADR-0110):** el "Veto Vectorial" de esta feature es el precedente arquitectónico de la máscara binaria Permitido/Prohibido del Dominio de Régimen y Filtro de Entorno. La diferencia es de origen: los vetos de Vector-Time Pruning son FIJOS (estadísticos, basados en Z-Score y calendario macro) y se aplican siempre; la máscara de ADR-0110 es evolutiva, condicionada por sus Genes de Condición de Estado (Hurst, entropía de Shannon, pendientes Hull MA, `regime_label`), y solo actúa cuando ese genoma está activo. Ambos mecanismos coexisten: primero se aplican los vetos fijos de esta feature, y sobre el resultado, la máscara evolutiva del Genoma de Régimen y Filtro.

---

## El Proceso de 3 Pasos (Pipeline Quirúrgico)

1. **Detección Dual (Rayos X y Radar Macro):** El sistema detecta Z-Scores atípicos (positivos o negativos). Paralelamente, vía una API REST intercepta el calendario fundamental (NFP/FOMC).
2. **Confirmación y Recálculo:** No se bloquea un horario solo por 2 ocurrencias. Pero si un solo trade genera una anomalía de +3 Z-Score, este "golpe de suerte exuberante" se extirpa y la curva se recalcula *in-situ* sin rechazar toda la estrategia.
3. **Inyección (La Cirugía y el Contagio Sintético):** Genera el "Veto Vectorial" (bloqueo en NautilusTrader) invisible en el backend.

---

## Parámetros Configurables

| Parámetro | Default | Descripción (Simple) |
|---|---|---|
| `time_pruning_z_threshold` | ±1.5 | ¿Qué tan anormal deben ser las operaciones para iniciar la poda de "cánceres" o "falsos milagros"? |
| `excision_max_z_score` | +3.0 | Extirpación transaccional sobre ganancias exorbitantes irreales. |
| `time_pruning_min_occurrences` | 10 | ¿Cuántas veces tiene que equivocarse (en negativos) para confirmar veto? |
| `macro_event_window_minutes` | 15 | Ventana temporal (± minutos) a aislar alrededor de NFP o FOMC. |
| `vix_stddev_filter` | +3.0 | Filtro de contagio sintético (STDDev de VIX) para aislamientos cruzados. |

---

## Tareas (TTRs)

### TTR-001: Motor Analítico de Agrupación (Rust SIMD/Rayon)
- **Descripción:** Segmenta trades en "Día/Hora" detectando Z-Scores atípicos y aplicando la extirpación transaccional sobre ganancias exuberantes (`Z-Score > +3`) recalculando íntegramente la curva.

### TTR-002: Radar Macro-API y Contagio Sintético
- **Descripción:** Implementa la intercepción REST/API al calendario fundamental para identificar NFP y FOMC, inyectando bloqueos temporales estáticos (± 15 min) y vetos dinámicos (VIX +3 STDDev).

### TTR-003: Inyector de Barrera (Nautilus Wrapper)
- **Descripción:** Serializa los "Horarios Tóxicos" y eventos macro para inyectarlos en `NautilusTrader`, apagando operaciones de forma autónoma.

### TTR-004: Generalización del Veto Vectorial a Máscara Evolutiva de Régimen (ADR-0108/ADR-0110)
- **Descripción:** Documentar y exponer la interfaz de "Veto Vectorial" (`is_vetoed(bar) -> bool`) como el contrato compartido que también implementa la máscara Permitido/Prohibido del Genoma de Régimen y Filtro de Entorno (ADR-0110), de forma que ambos mecanismos se apliquen en el mismo punto de la cadena de ejecución sin duplicar la lógica de bloqueo.
- **Criterio de Éxito:** Una barra vetada por Vector-Time Pruning (FIJO) nunca llega a evaluarse contra la máscara evolutiva de Régimen y Filtro; ambos vetos se registran de forma distinguible en el rastro forense.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020):** 
    - Perfil: Ops / Auditoría.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
    - **V. Forense & Ejecución:** `compliance_status_id`, `risk_audit_id`.

- **Regla Inquebrantable:** La poda temporal NUNCA altera el historial pasado. Su único propósito es emitir "Leyes" para el comportamiento *futuro* (Paper Trading y Live). Si se altera el pasado, incurrimos en trampa de sobreajuste (Curve Fitting).
- **Genomas Modulares por Dominio (ADR-0108/ADR-0110):** Esta feature es el precedente FIJO de la máscara Permitido/Prohibido del Dominio de Régimen y Filtro de Entorno. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
- **Dependencias:** Utilizado como un puente desde `validate` (donde se detecta) hacia `manage` y `execute` (donde se aplica la cirugía).
