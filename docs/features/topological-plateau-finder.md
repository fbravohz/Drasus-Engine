# Topological Plateau Finder (Auto-Optimizer)

**Carpeta:** `./features/topological-plateau-finder/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020 V2

---

## ¿Qué es?

El "Buscador Topológico de Mesetas" es un analizador del "vecindario" hiperespacial de parámetros (Optimization Profile). Evalúa automáticamente la estabilidad de los parámetros de una estrategia variando sus valores un ±X% en todas las direcciones del espacio dimensional.

**Problema que resuelve:** En lugar de dejar que el usuario trate de interpretar un gráfico 3D manual con picos de ganancia, esta IA escanea el terreno. Si la estrategia obtiene ganancias porque un parámetro cayó justo en el pico de una montaña (muy inestable, cambiar de 14 a 15 arruina todo), la rechaza. Si está en una llanura (una "meseta plana"), centra los parámetros automáticamente en el medio de la zona segura.

---

## Comportamientos Observables

- [ ] La UI renderiza una malla tridimensional interactiva (Topografía de Parámetros) cuando el usuario optimiza dos parámetros (ej. Periodo SMA vs Multiplicador ATR).
- [ ] El motor perturba ±10% cada parámetro en la red matricial (Neighborhood Analysis).
- [ ] Mide el gradiente del rendimiento ($\nabla f \approx 0$). Si es cercano a 0, significa que la zona es plana y estable (Plateau).
- [ ] El usuario puede hacer clic directamente en la zona más plana y estable de la malla 3D para fijar los parámetros finales en el modelo.
- [ ] El sistema (Auto-Configurador) elige automáticamente los valores paramétricos que se encuentran en el centro geométrico de la meseta más grande.
- [ ] Emite un "Veredicto de Estabilidad" basado en el ancho y profundidad de la meseta.

---

## Restricciones

- **FIJO:** Ningún parámetro será validado si reside en un pico (donde la derivada primera es abrupta). La IA forza invariablemente los parámetros al centroide de la meseta plana.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| NEIGHBORHOOD_RADIUS | 15% | 5% - 25% | Radio de exploración paramétrica alrededor del valor óptimo. | CONFIG |
| MIN_PLATEAU_WIDTH | 3 | 2 - 10 | Número de pasos estables requeridos para considerar que es una meseta. | CONFIG |

---

## Ciclo de Vida de la Feature — Topological Plateau Finder

### Entrada
- Espacio paramétrico de una estrategia (Ej: Periodo MA = 14, Umbral RSI = 30).
- Motor SQL/DuckDB de iteración.

### Proceso
- Genera permutaciones de los parámetros en un vecindario ($14 \pm 2$, $30 \pm 5$).
- Calcula la ganancia para cada nodo del vecindario.
- Mapea topológicamente dónde está la planicie y el acantilado.

### Salida
- Coordenadas de los parámetros re-centrados (`centered_parameters`).
- `neighborhood_score` (Índice de Robustez de Entorno).
- `plateau_verdict` (STABLE / UNSTABLE).

### Contextos de Uso
**Contexto 1: Auto-Optimización Robusta (Validate)**
- Reemplaza la visualización manual de Heatmaps en gráficos 3D. El motor automatiza la elección de la zona que sufre menos ante cambios sutiles del mercado.

---

## Tareas (TTRs)

### **TTR-001: Mapeo Topológico y Gradiente (Neighborhood Analysis)**
*   **¿Cuál es el problema?** Saber si un parámetro ganador está en un pico afilado o en una llanura sólida.
*   **¿Qué tiene que pasar?** El sistema altera el hiperespacio un ±X%, ejecuta micro-backtests y computa el gradiente.
*   **¿Cómo sé que está hecho?**
    - [ ] Se reporta un "Plateau Found: Yes".
    - [ ] Un pico aislado lanza advertencia "Isolated Peak Detected: Fragile".

### **TTR-002: Auto-Configuración y Centrado Geométrico**
*   **¿Cuál es el problema?** El mejor PnL matemático suele estar en el borde del acantilado. Queremos estar en el centro de la meseta, aunque paguemos menos.
*   **¿Qué tiene que pasar?** La IA toma todos los puntos de la meseta, calcula el centroide, y reescribe los parámetros fijos de la estrategia hacia esa coordenada segura.
*   **¿Cómo sé que está hecho?**
    - [ ] Si la meseta abarca RSI desde 25 hasta 35, el sistema setea automáticamente `RSI_Threshold = 30` (el centro).
    - [ ] El log indica "Parámetros centrados en el medio geométrico para maximizar robustez."

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
