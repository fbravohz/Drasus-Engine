# Vista en Vivo

**Carpeta:** `./features/live-strategy-view/`
**Estado:** En Diseño
**Última actualización:** 2026-07-21
**Decisión Arquitectónica Asociada:** ADR-0157 (Visualización de Estrategia — un motor de render, dos entradas separadas), ADR-0027 (Event Sourcing), ADR-0114 (Contrato de Consistencia Conservadora)

---

## ¿Qué es esta feature?

Vista en Vivo permite **observar una estrategia operando mientras se simula**, en tiempo controlable, al estilo del *strategy tester* de MetaTrader 5 — pero determinista. El usuario ve el gráfico, los indicadores dibujados, los cruces, las señales de compra/venta, los niveles de stop y profit, los trailing stops y el movimiento del precio desde que se abre una posición hasta que toca profit o stop.

**Problema que resuelve:** aunque la aplicación es para trading algorítmico y no manual, un trader le cree a una estrategia cuando la ve cumplir al 100% sus reglas teóricas frente a sus ojos. Es una herramienta de **depuración y confianza**: confirmar visualmente que la lógica (cualquier indicador, niveles de stop/profit, trailing, o gestión de riesgo compleja) hace exactamente lo que se espera. Es hacia adelante: *"lo estoy viendo suceder"*. Comparte el motor de render con [`run-replay`](./run-replay.md) (Repetición de Corrida), pero se invoca distinto y sirve otro propósito (observar en vivo vs. revisar lo ya ocurrido).

---

## Narrativa de Producto

**Capacidad:** `CAP-05` — Cuarentena antes del dinero real

**Historia de usuario:** Como trader que no confía a ciegas en un algoritmo, quiero ver a la estrategia operar en cámara —con sus indicadores, señales y niveles de stop dibujados sobre el precio— para confirmar con mis propios ojos que hace exactamente lo que dice que hace.

**El problema (técnico):** un árbol de reglas generado automáticamente puede tener docenas de condiciones encadenadas, y ninguna cifra de rendimiento por sí sola convence a un humano de que la lógica es correcta. Un reporte estático tampoco muestra el instante exacto en que la estrategia decidió entrar o salir — solo el resultado final.

**La solución (técnica):** un observador que se engancha a una corrida en curso —sin frenarla ni alterar su resultado en absoluto— y dibuja sobre el gráfico, en cualquier tipo de vela que la estrategia use, los indicadores, las señales de entrada/salida y los niveles de stop y de toma de ganancia. El usuario controla la velocidad de reproducción, y al pasar el cursor sobre una barra ve una tabla con el cálculo exacto de cada indicador en ese instante.

**Explicado sin tecnicismos:** es la diferencia entre leer el reporte de una cirugía y ver la cirugía en video. El reporte dice que salió bien; el video te deja comprobarlo tú mismo, paso a paso, y en cámara lenta si hace falta.

Aquí puedes ver a la estrategia "operar" frente a tus ojos, pausar el momento exacto de una entrada, y revisar por qué compró o vendió justo ahí — sin que el hecho de estar mirando cambie en nada lo que en realidad sucede por dentro del motor.

---

## Comportamientos Observables

- [ ] El usuario arrastra al canvas un nodo **Vista en Vivo** y lo conecta a una corrida en curso; ve la estrategia operar en tiempo controlable.
- [ ] Puede **ralentizar o acelerar** el tiempo de la simulación observada para confirmar visualmente el comportamiento; al soltar el foco, el motor sigue a máxima velocidad.
- [ ] Los indicadores, cruces, señales de entrada/salida, stop/profit y trailing se dibujan sobre el gráfico, sobre **cualquier tipo de barra** que la estrategia use (velas, Renko, Rango, Volumen, Tick, Dólar…), no solo velas japonesas.
- [ ] Al pasar el cursor sobre una barra, una tabla muestra sus valores (`open/high/low/close`, volumen, metadatos) **y el cálculo exacto de cada indicador de la estrategia en esa barra** (inspector estilo StrategyQuant X).
- [ ] Mientras el usuario observa una corrida, las demás (millones de candidatas en minería, optimizaciones, WFO, Monte Carlo, multi-símbolo) siguen a plena velocidad sin degradarse.

---

## Restricciones

- **Invariante de No-Perturbación (NUNCA):** observar jamás frena, gatea ni altera la velocidad del motor headless. Vista en Vivo es un **observador opcional** que se engancha bajo demanda a **una** corrida concreta (una estrategia, un símbolo, un destino) y materializa un flujo reproducible; nunca es algo que la simulación tenga que ir alimentando. Enganchar o soltar el observador no cambia el resultado ni el rendimiento de la flota.
- **Reproducibilidad Determinista (NUNCA):** lo que se renderiza es exactamente lo que el motor ejecutó — no puede mostrar un fill más optimista que el que tomó el motor (respeta Bar-Open Alignment, regla intrabar pesimista y lectura `bid`/`ask` de ADR-0114).
- **Memoria Acotada (NUNCA):** no se cargan datasets completos en RAM para la sesión de observación; consultas indexadas y perezosas.
- **Nomenclatura (NUNCA):** prohibido cualquier término del campo semántico de la muerte en el nombre de la feature o sus nodos.

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PLAYBACK_SPEED | 1.0 | 0.1 - 100.0 | Multiplicador de velocidad de la simulación observada | CONFIG |
| ATTACH_TARGET | (ninguno) | una corrida | La corrida concreta (estrategia/símbolo/destino) a la que se engancha el observador | CONFIG |
| INDICATOR_INSPECTOR | on | on/off | Muestra la tabla de OHLCV + cálculo de indicadores al pasar el cursor | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** proyección del estado de la estrategia (indicadores, señales, posición, stops) a marcas visuales sobre el tipo de barra activo; muestreo temporal para el control de velocidad.
- **Shell (Infraestructura):** suscripción de solo lectura al flujo de eventos de una corrida en curso, sin bloquear al productor; serialización Arrow hacia el puente FFI de Flutter.
- **Frontera Pública:** interfaz para enganchar/soltar el observador a una corrida y emitir el stream de render.

---

## Ciclo de Vida de la Feature — Vista en Vivo

### Entrada
- Referencia a una corrida en curso (backtest o simulación) y su flujo de eventos.
- El tipo de barra activo y las series de indicadores de la estrategia.
- Comandos de control de velocidad del usuario.

### Proceso
- Se suscribe de solo lectura al flujo de la corrida objetivo, sin frenar al productor.
- Proyecta el estado (indicadores, señales, stops, posición) a marcas visuales sobre el gráfico.
- Aplica el multiplicador de velocidad para el render, desacoplado de la velocidad real del motor.

### Salida
- Render animado de la estrategia operando, con inspector de barra bajo el cursor.
- Ningún efecto sobre el resultado ni el rendimiento de la corrida observada.

### Contextos de Uso
- **Depuración y confianza (validate / incubate):** confirmar visualmente que una estrategia cumple sus reglas antes de confiar en ella.

---

## Tareas (TTRs)

### **TTR-001: Suscripción de Solo Lectura sin Perturbar al Productor**
*   **¿Cuál es el problema?** Observar una corrida no puede frenar ni alterar el motor headless que la ejecuta.
*   **¿Qué tiene que pasar?** Un canal de solo lectura, sin contrapresión sobre el productor, que materialice el flujo de eventos para el render; si el observador no consume a tiempo, se degrada el render (se saltan cuadros), nunca la corrida.
*   **¿Cómo sé que está hecho?**
    - [ ] Enganchar y soltar el observador no cambia el resultado bit a bit de la corrida ni su tiempo total.
    - [ ] Una prueba mide throughput de la flota con y sin observador enganchado y confirma que no se degrada.

### **TTR-002: Render Agnóstico al Tipo de Barra con Inspector de Barra**
*   **¿Cuál es el problema?** El visor debe pintar indicadores, señales y stops sobre cualquier tipo de barra, y mostrar el cálculo exacto de cada indicador al pasar el cursor.
*   **¿Qué tiene que pasar?** Motor de render que toma el tipo de barra activo y las series de indicadores y produce las marcas visuales + la tabla del inspector.
*   **¿Cómo sé que está hecho?**
    - [ ] Una estrategia sobre barras Renko muestra sus indicadores y señales correctamente, igual que sobre velas.
    - [ ] Al pasar el cursor sobre una barra, la tabla muestra OHLCV + el valor de cada indicador y coincide con la bitácora del motor.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `bars_in` | `Bars` | Input | `1..N` | Series de barras de las fuentes de la estrategia (cualquier `bar_type`); múltiple por la topología de ADR-0158 |
| `signal_in` | `Signal` | Input | `0..N` | Señales de entrada/salida producidas por la estrategia observada |
| `order_in` | `Order` | Input | `0..N` | Órdenes disparadas durante la simulación |
| `position_in` | `Position` | Input | `0..N` | Estado de posición, stops y trailing durante la corrida |

> Es un observador terminal (nodo de canvas / Inspector Panel): consume el estado de una corrida y lo renderiza; no produce un dato de puerto hacia el pipeline.

## Cáscara Visual (Thin Shell)

> **Pendiente de la Etapa 0.5** — la produce el skill `ui-designer` leyendo `docs/DESIGN.md` antes del despacho al Flutter Engineer. No rellenar a mano.

### Contexto de superficie (ADR-0136)
**Canvas — Vista Interior / Inspector Panel** — nodo "Vista en Vivo" que, al enfocarse, muestra el gráfico animado de la estrategia operando con el inspector de barra.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** hereda la fidelidad de la corrida observada; no altera el fill.
- **Inundación de Fundaciones (ADR-0020):** Perfil D (Ops / Auditoría) — registra el Grupo I completo (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) + `process_id`, `node_id`. La sesión de observación es efímera; lo persistido es la referencia a la corrida observada.

## Dependencias y Bloqueantes
**Depende de:**
- [`run-replay`](./run-replay.md) — comparte el motor de render y de reconstrucción de estado.
- [`backtest-engine`](./backtest-engine.md) — provee el flujo de eventos de la corrida.

**Contrato de Integración UI (ADR-0117):**
- **Superficie propia:** nodo/pestaña "Vista en Vivo". SVF: enganchar el observador a una corrida real dispara el stream vía `public_interface`; el usuario ve la estrategia operar y el inspector de barra; una prueba confirma que el throughput de la flota no cambia con el observador enganchado.
