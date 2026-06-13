# Component Isolation (The Monkey Test)

**Carpeta:** `./features/component-isolation/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020 V2

---

## ¿Qué es?

El "Monkey Test" (Aislamiento de Componentes) es una auditoría de sentido común estadístico. Su objetivo es evitar la falsa confianza en una estrategia mediante la evaluación aislada del mérito real de su lógica de entrada y su lógica de salida.

**Problema que resuelve:** Muchas estrategias parecen rentables, pero en realidad su lógica de salida (Take Profit / Stop Loss) no aporta nada y el éxito es puramente aleatorio o dependiente de la inercia del mercado. Si aleatorizar las salidas produce el mismo resultado que la lógica programada, el componente de salida es inútil y añade "ruido" al sistema.

---

## Comportamientos Observables

- [ ] **Prueba de Salida Aleatoria:** El sistema ejecuta la estrategia con las entradas originales, pero cierra los trades en momentos 100% aleatorios.
- [ ] **Prueba de Entrada Aleatoria:** El sistema entra al mercado al azar (mono lanzando dardos) pero utiliza la lógica de salida (Stop/Target) original.
- [ ] Si la métrica de la salida aleatoria es igual o mejor a la salida original, se dictamina que la salida programada es inútil.
- [ ] La lógica inútil se marca para ser eliminada.

---

## Restricciones

- **FIJO:** El generador de aletoriedad debe usar una semilla estocástica (Random Seed) determinista para permitir la repetición del backtest con los mismos resultados en modo auditoría.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| RANDOM_ITERATIONS | 500 | 100 - 5000 | Cuántas veces el mono opera al azar para el benchmark. | CONFIG |
| USELESSNESS_PVALUE | 0.05 | 0.01 - 0.10 | P-Value para determinar si la lógica aporta valor sobre el azar. | CONFIG |

---

## Ciclo de Vida de la Feature — Component Isolation

### Entrada
- Reglas lógicas de Entrada de la estrategia.
- Reglas lógicas de Salida de la estrategia.
- Datos Históricos.

### Proceso
- Ejecuta simulación N veces alterando uno de los componentes (Entrada o Salida) por ruido blanco (entradas o salidas aleatorias).
- Compara el retorno esperado de la lógica real vs. la media de las simulaciones aleatorias.

### Salida
- `entry_edge_score` (Mérito propio de la entrada).
- `exit_edge_score` (Mérito propio de la salida).
- Veredicto de Aislamiento (VALUABLE / USELESS).

### Contextos de Uso
**Contexto 1: Extirpación de Falsas Esperanzas (Validate)**
- Permite simplificar las estrategias (navaja de Ockham) quitando componentes que solo engañan al creador pero que matemáticamente son equivalentes a lanzar una moneda.

---

## Tareas (TTRs)

### **TTR-001: Simulación de Salida Aleatoria (Monkey Exit)**
*   **¿Cuál es el problema?** No sabemos si la estrategia gana por la señal, o si el Take Profit es bueno.
*   **¿Qué tiene que pasar?** El sistema respeta las entradas, pero asigna barras de cierre (exit) al azar. Corre esto 500 veces y obtiene un PnL promedio.
*   **¿Cómo sé que está hecho?**
    - [ ] El log indica "Ejecutando simulación Monkey Exit (500 iteraciones)".
    - [ ] Hay un reporte que compara PnL de salida real vs PnL salida mono.
*   **¿Qué no puede pasar?** Las salidas no pueden adelantarse a la entrada (no look-ahead bias en la aletoriedad).

### **TTR-002: Simulación de Entrada Aleatoria (Monkey Entry)**
*   **¿Cuál es el problema?** Evaluar si el Stop Loss / Trailing Stop es lo que realmente genera rentabilidad.
*   **¿Qué tiene que pasar?** El sistema entra al azar, pero utiliza el bloque de cierre original.
*   **¿Cómo sé que está hecho?**
    - [ ] Se genera un score de confianza sobre el componente de salida.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`, `indicator_state_hash`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
