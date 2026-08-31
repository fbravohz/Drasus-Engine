# Ventana Adaptativa DTW (Matriz Orgánica) — SQX Mod 23

**Carpeta:** `./features/dtw-adaptive-window/`
**Estado:** Especificación (Extraída de saas_sqx_improvements Mod 23)
**Última actualización:** 2026-06-06

---

## ¿Qué es?

Es el motor de **segmentación temporal adaptativa** de Drasus Engine. Reemplaza el corte de historia en bloques rígidos de tamaño fijo (el defecto de la Matriz Walk-Forward clásica) por un corte que respira con el mercado.

**Problema que resuelve:** La validación Walk-Forward tradicional asume que el mercado cambia a ritmo constante, partiendo el historial en bloques cuadrados inamovibles (ej. siempre 2 años de entrenamiento). Eso es mentira: en un mercado tendencial largo, los parámetros aguantan meses; en una crisis, se rompen en días. Cortar todo igual mezcla regímenes distintos en una misma ventana y contamina el veredicto de robustez.

**Qué hace:** Usa **Dynamic Time Warping (DTW)** — un algoritmo que mide similitud entre dos series temporales que avanzan a distinta velocidad — para detectar dónde se expande o contrae realmente la volatilidad macro. Con esa señal, agranda la ventana de optimización en periodos estables y la encoge en periodos de crisis. La "Matriz Orgánica" o **Breathing Matrix**: el segmentador respira.

Es un componente **reutilizable**: cualquier motor que necesite cortar historia por régimen (no por calendario) lo consume.

---

## Narrativa de Producto

**Capacidad:** `CAP-04` — El guantelete de robustez

**Historia de usuario:** Como investigador de estrategias, quiero que las ventanas de prueba se ajusten al ritmo real del mercado en vez de cortarse en bloques de calendario fijos, para que un periodo de calma no se mezcle con una crisis dentro de la misma ventana y contamine el veredicto.

**El problema (técnico):** cortar el historial en bloques de tamaño fijo —por ejemplo, siempre dos años— asume que el mercado cambia a un ritmo constante, y eso es falso: en una tendencia larga los parámetros aguantan meses, en una crisis se rompen en días. Un bloque fijo puede mezclar sin darse cuenta un tramo estable con uno de pánico, y el resultado combinado no refleja honestamente ninguno de los dos — esconde la ruptura dentro de un promedio que parece razonable.

**La solución (técnica):** mide qué tan parecidos son entre sí los tramos consecutivos de historia usando una técnica que tolera que el mercado avance a distinta velocidad (comparación de series temporales elásticas), y agrupa en una sola ventana los tramos que se parecen, cortando donde la similitud se rompe bruscamente. El resultado es una segmentación que se expande en periodos estables y se contrae en periodos de crisis, siempre dentro de un mínimo y un máximo configurables, y siempre de forma reproducible: la misma historia con la misma configuración produce siempre los mismos cortes.

**Explicado sin tecnicismos:** imagina medir el tiempo de un viaje en carretera siempre en bloques fijos de una hora, sin importar si vas por autopista despejada o atrapado en un embotellamiento. Un bloque de una hora en autopista cubre mucha distancia sin sobresaltos; el mismo bloque de una hora en el embotellamiento puede esconder tres frenones bruscos que nunca se distinguen en el promedio.

Esta pieza corta el viaje donde el ritmo real cambia, no donde marca el reloj: ventanas largas cuando el camino está despejado, ventanas cortas y frecuentes cuando el tráfico se pone denso. Así ningún frenón se queda escondido dentro de un tramo que, en promedio, se veía tranquilo.

## Comportamientos Observables

- [ ] Usuario corre validación sobre 10 años de historia con segmentación adaptativa activada
  → En la zona de tendencia larga 2016-2017, el sistema produce ventanas grandes (pocos cortes)
  → En la zona de crisis (ej. marzo 2020), el sistema produce ventanas pequeñas y densas (muchos cortes)
- [ ] Usuario compara el mismo backtest con ventana fija vs ventana adaptativa
  → Ve que la curva de robustez adaptativa detecta rupturas que la fija escondía dentro de un bloque grande
- [ ] El sistema reporta un "Índice de Respiración" por tramo
  → Indica cuánto se expandió/contrajo la ventana respecto al tamaño base
- [ ] Usuario fija un "Pivote" de tolerancia de contracción
  → Define la ventana mínima que su infraestructura puede re-optimizar; el motor nunca corta más fino que eso

---

## Restricciones

- NUNCA altera el historial pasado. La segmentación solo decide DÓNDE cortar para validar/optimizar el comportamiento futuro. Reescribir el pasado = trampa de sobreajuste (misma invariante que el Analizador Walk-Forward).
- NUNCA produce una ventana menor al mínimo configurable (protege contra re-optimización imposible para la latencia de red del operador).
- NUNCA produce una ventana mayor al máximo configurable (evita esconder rupturas dentro de un bloque gigante).
- El corte adaptativo es determinista: misma historia + misma config → mismos límites de ventana (reproducibilidad total).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MIN_WINDOW | 3 días | 1-30 días | Ventana mínima en crisis (pivote de contracción) | CONFIG |
| MAX_WINDOW | 12 meses | 1-36 meses | Ventana máxima en tendencia estable | CONFIG |
| SIMILARITY_THRESHOLD | 0.7 | 0.3-0.95 | Cuánta similitud DTW exige para mantener un tramo unido | CONFIG |
| CONTRACTION_SENSITIVITY | media | baja/media/alta | Qué tan agresivo encoge la ventana ante shocks de volatilidad | CONFIG |
| REGIME_SOURCE | hmm | hmm / volatility-only | De dónde toma la señal de régimen | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Cálculo de distancia DTW entre tramos y algoritmo de decisión de límites de ventana. Sin IO, sin reloj de sistema.
- **Shell (Infraestructura):** Lectura de la señal de régimen y entrega de los límites de ventana al motor consumidor.
- **Frontera Pública:** Contrato que recibe historial + señal de régimen y devuelve una lista ordenada de límites de ventana variables.

---

## Ciclo de Vida de la Feature — DTW Adaptive Window

### Entrada
- Historial de barras OHLC completo del activo.
- Señal de régimen de mercado (expansión/contracción de volatilidad).
- Configuración de pivotes (ventana mínima, máxima, sensibilidad).

### Proceso
1. Mide la similitud temporal entre tramos consecutivos usando Dynamic Time Warping (tolerante a que el mercado avance a distinta velocidad).
2. Mientras dos tramos vecinos son lo bastante similares, los mantiene unidos en una sola ventana grande (mercado estable → respiración expandida).
3. Cuando la similitud cae bruscamente (shock de volatilidad), corta y abre ventanas pequeñas y densas (crisis → respiración contraída).
4. Acota cada ventana resultante entre el mínimo y el máximo configurados.
5. Emite el Índice de Respiración por tramo (cuánto se desvió del tamaño base).

### Salida
- Lista ordenada de límites de ventana de tamaño variable (no calendario fijo).
- Índice de Respiración por tramo.
- Marca de los puntos de quiebre de régimen detectados.

### Contextos de Uso

**Contexto 1: Validación de Estrategia (Módulo validate)**
- Entrada: historial + señal de régimen.
- Pregunta que responde: "¿En qué tramos reales debo re-optimizar, en vez de cada N meses ciegos?"
- Impacto: alimenta al Analizador Walk-Forward con ventanas honestas por régimen, mejorando la detección de rupturas.

**Contexto 2: Cadencia de Rebalanceo de Portafolio (Módulo manage)**
- Entrada: historial del portafolio + señal de régimen.
- Pregunta que responde: "¿Cada cuánto debe re-pesarse el portafolio según la volatilidad real?"
- Impacto: rebalanceo más frecuente en crisis, más espaciado en calma.

---

## Tareas (TTRs)

### TTR-001: Segmentador de similitud temporal (DTW)
*   **¿Cuál es el problema?** Cortar la historia por calendario fijo mezcla un mercado en calma con uno en pánico dentro de la misma ventana, y eso miente sobre la robustez.
*   **¿Qué tiene que pasar?** El sistema mide qué tan parecidos son tramos de historia vecinos y los agrupa cuando se parecen, los separa cuando no.
*   **¿Cómo sé que está hecho?**
    - [ ] En una zona de tendencia larga, veo pocas ventanas grandes.
    - [ ] En una zona de crisis, veo muchas ventanas chicas.
    - [ ] Corro dos veces con la misma config y obtengo exactamente los mismos cortes.
*   **¿Qué no puede pasar?** Producir ventanas fuera del rango mínimo/máximo configurado. Alterar barras históricas.

### TTR-002: Pivote de contracción y acotado de ventanas
*   **¿Cuál es el problema?** Si el motor corta más fino de lo que la infraestructura del operador puede re-optimizar, genera ventanas inútiles.
*   **¿Qué tiene que pasar?** El usuario fija una ventana mínima (pivote) y el sistema nunca corta por debajo de ella, ni por encima del máximo.
*   **¿Cómo sé que está hecho?**
    - [ ] Bajo el pivote a 1 día y veo más cortes; lo subo a 7 días y veo menos.
    - [ ] Ninguna ventana emitida es menor al pivote.

### TTR-003: Emisión del Índice de Respiración y entrega al consumidor
*   **¿Cuál es el problema?** El motor Walk-Forward necesita recibir los límites variables y el usuario necesita ver cuánto respiró cada tramo.
*   **¿Qué tiene que pasar?** El sistema entrega la lista de ventanas variables al consumidor y reporta, por tramo, cuánto se expandió o contrajo.
*   **¿Cómo sé que está hecho?**
    - [ ] El Analizador Walk-Forward consume estas ventanas en lugar de las fijas.
    - [ ] Veo un Índice de Respiración por tramo en el rastro de auditoría.

### TTR-004: Render observable de la cadencia (UI — Flutter/Impeller)
*   **¿Cuál es el problema?** El usuario necesita "sentir" dónde el sistema validó denso vs espaciado sin leer una tabla.
*   **¿Qué tiene que pasar?** La interfaz dibuja la cadencia de ventanas sobre la línea de tiempo: tramos anchos donde respiró expandido, tramos densos donde se contrajo. El usuario clava el pivote arrastrando sobre la línea de tiempo.
*   **¿Cómo sé que está hecho?**
    - [ ] Veo visualmente las ventanas anchas y angostas alineadas con tendencia y crisis.
    - [ ] Al arrastrar el pivote, la cadencia se re-dibuja en tiempo real.

---

## Features Consumidas (Reutilizables)

- **[`hmm-regime-detection`](./hmm-regime-detection.md)** — Provee la señal de régimen (expansión/contracción de volatilidad) que dispara la respiración del segmentador.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% local. El cálculo DTW corre sobre Parquet local vía el stack analítico existente.
- **Inundación de Fundaciones (ADR-0020):**
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
- **Rastro de Evidencia (causalidad → feedback):** Emite los límites de ventana usados y el Índice de Respiración por tramo, para que `feedback` pueda explicar por qué un veredicto de robustez se calculó sobre esa cadencia y no otra.

---

## Decisión Arquitectónica Asociada

- Sin ADR nuevo. Es un requisito algorítmico de robustez (§7.5: algoritmo específico → no ADR). Hereda las invariantes anti-curve-fitting del Analizador Walk-Forward.

---

## Dependencias

**Consumido por:** [`walk-forward-analyzer`](./walk-forward-analyzer.md) (`validate`, `manage`).
**Depende de:** [`hmm-regime-detection`](./hmm-regime-detection.md).
