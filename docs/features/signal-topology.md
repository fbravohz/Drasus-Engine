# Topología de Fuentes y Destinos

**Carpeta:** `./features/signal-topology/`
**Estado:** En Diseño
**Última actualización:** 2026-07-21
**Decisión Arquitectónica Asociada:** ADR-0158 (Topología de Fuentes de Señal y Destinos de Ejecución), ADR-0114 (conocibilidad por reloj), ADR-0111 (riesgo agregado)

---

## ¿Qué es esta feature?

Define y resuelve los **dos ejes de una estrategia**: de dónde lee señales y dónde ejecuta. Hoy una estrategia solo lee de su propio símbolo y timeframe; esta feature la generaliza:

- **Fuentes de Señal:** conjunto no vacío de pares (símbolo × timeframe) — propios, ajenos o sintéticos — de los que la estrategia lee gatillos, condiciones y señales de entrada/salida.
- **Destinos de Ejecución:** conjunto no vacío de símbolos donde la estrategia abre y cierra posición.

**Problema que resuelve:** una estrategia limitada a su propio símbolo/timeframe deja fuera una superficie entera de alfa (adelanto/retraso entre activos, señales inter-mercado, confirmación multi-temporalidad). Se **elimina la noción de "mercado principal"** de MT5/SQX: una estrategia es un array de Fuentes y un array de Destinos desde su definición, sin símbolo ancla.

---

## Comportamientos Observables

- [ ] Al definir una estrategia, el usuario declara sus Fuentes (N símbolos × N timeframes) y sus Destinos (N símbolos) — cualquier combinación, mínimo uno de cada.
- [ ] Una estrategia puede buscar el gatillo en un símbolo/timeframe y ejecutar en otro (buscar en un índice, operar EURJPY), o buscar en el propio y ejecutar en varios a la vez.
- [ ] Cuando ejecuta en varios destinos, el reporte de riesgo muestra la exposición **agregada** de las posiciones correlacionadas, no N riesgos aislados.
- [ ] La lectura de una fuente en el instante `t` solo usa barras cuyo cierre es `≤ t` (sin mirar el futuro), en cualquier tipo de barra.
- [ ] El validador de robustez cuenta las combinaciones de fuente/destino como grados de libertad al corregir por pruebas múltiples.

---

## Restricciones

- **NUNCA look-ahead entre fuentes:** una fuente es conocible en `t` solo si tiene una barra cerrada con `bar_close_timestamp_ns ≤ t` (hereda ADR-0114, generalizado a toda fuente).
- **NUNCA lectura cross-timeframe sobre barra en formación:** se usa la última barra CERRADA de la fuente, nunca una en curso.
- **NUNCA relleno invisible de huecos:** si una fuente no tiene barra conocible en `t`, se marca "sin dato conocible", jamás forward-fill silencioso (hereda ADR-0154).
- **NUNCA riesgo de destinos sin auditar:** las N posiciones simultáneas pasan por el control de riesgo agregado de cartera (ADR-0111); nunca se dimensionan independientes.
- **NUNCA topología ilimitada:** los ejes están acotados (ver parámetros) — sin topes, N fuentes garantiza falsos positivos por data dredging.

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MAX_SIGNAL_SOURCES | 5 | 1 - 20 | Máximo de fuentes de señal por estrategia | CONFIG |
| MAX_EXECUTION_DESTINATIONS | 3 | 1 - 20 | Máximo de destinos de ejecución simultáneos | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** resolución de conocibilidad por reloj sobre múltiples fuentes; ensamblado del contexto multi-fuente alineado punto-en-el-tiempo que consume la lógica de la estrategia.
- **Shell (Infraestructura):** carga y alineación de las series de las fuentes desde el almacén; enrutamiento de la ejecución a los destinos declarados.
- **Frontera Pública:** interfaz que recibe la definición (Fuentes, Destinos) y expone el contexto alineado a la estrategia y el enrutamiento de órdenes.

---

## Ciclo de Vida de la Feature — Topología de Fuentes y Destinos

### Entrada
- Definición de la estrategia: array de Fuentes (símbolo × timeframe) y array de Destinos (símbolo).
- Series históricas de cada fuente.

### Proceso
- En cada instante de decisión, resuelve qué barras de cada fuente son conocibles (cierre `≤ t`), marcando las que no.
- Ensambla el contexto multi-fuente alineado y lo entrega a la lógica de la estrategia.
- Enruta las órdenes resultantes a los destinos, con dimensionamiento sujeto al riesgo agregado.

### Salida
- Contexto multi-fuente punto-en-el-tiempo para la estrategia.
- Órdenes enrutadas a los destinos con exposición agregada auditada.

### Contextos de Uso
- **Generación / minería (generate):** el espacio de búsqueda incluye el eje fuente/destino, acotado y contado en el sesgo de selección.
- **Validación (validate):** la robustez cruzada corre cada destino contra su cesta correlacionada como simulaciones independientes.
- **Ejecución (execute):** enrutamiento real a los destinos con riesgo agregado.

---

## Tareas (TTRs)

### **TTR-001: Resolución de Conocibilidad Multi-Fuente por Reloj**
*   **¿Cuál es el problema?** Leer de N fuentes heterogéneas (símbolos, timeframes, sintéticos) sin filtrar futuro.
*   **¿Qué tiene que pasar?** Para toda fuente y todo instante `t`, exponer solo barras con cierre `≤ t`; marcar "sin dato" donde no haya barra conocible, sin rellenar.
*   **¿Cómo sé que está hecho?**
    - [ ] Una prueba con fuentes de distinto timeframe confirma que ninguna barra en formación se lee antes de cerrar.
    - [ ] Un índice cerrado el fin de semana se marca "sin dato" mientras una fuente 24/7 sigue viva, sin forward-fill.

### **TTR-002: Enrutamiento a Destinos con Riesgo Agregado**
*   **¿Cuál es el problema?** N posiciones simultáneas de una misma estrategia pueden ser la misma apuesta correlacionada sin que nadie lo mida.
*   **¿Qué tiene que pasar?** Enrutar la ejecución a cada destino declarado, dimensionando bajo el control de riesgo agregado de cartera (ADR-0111).
*   **¿Cómo sé que está hecho?**
    - [ ] Una estrategia que opera EURUSD+GBPUSD+AUDUSD reporta exposición agregada, no 3 riesgos aislados.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `sources_in` | `Bars` | Input | `1..N` | Series de las Fuentes de Señal (símbolo × timeframe, propias/ajenas/sintéticas) |
| `signal_in` | `Signal` | Input | `1..N` | Señales que la lógica de la estrategia produce sobre el contexto multi-fuente |
| `orders_out` | `Order` | Output | `1..N` | Órdenes enrutadas a los Destinos de Ejecución |
| `positions_out` | `Position` | Output | `0..N` | Posiciones abiertas por destino, con riesgo agregado |

## Cáscara Visual (Thin Shell)

> **Pendiente de la Etapa 0.5** — la produce el skill `ui-designer`. No rellenar a mano.

### Contexto de superficie (ADR-0136)
**Canvas — Vista Interior** — el usuario compone las Fuentes y los Destinos de una estrategia directamente en el canvas (sin símbolo ancla).

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil B (IA / R&D) — Grupo I completo + Soberanía (`owner_id`, `manifest_id`) + Linaje (`logic_hash`, `data_snapshot_id`) + Hardware (`process_id`, `node_id`).

## Dependencias y Bloqueantes
**Depende de:**
- [`synthetic-symbols`](./synthetic-symbols.md) — una fuente puede ser un símbolo sintético.
- [`cross-market-validation`](./cross-market-validation.md) — consume los Destinos para la robustez cruzada.
- **Bloqueante duro:** TASK-059 (contrato de conocibilidad atado al campo de cierre, ampliado a toda fuente) debe cerrarse antes de implementar.

**Consumido por:** generate, validate, execute — a través de su `public_interface`.

**Contrato de Integración UI (ADR-0117):**
- **Superficie propia:** editor de Fuentes/Destinos en la definición de estrategia. SVF: declarar fuentes y destinos y correr un backtest real vía `public_interface`; el resultado muestra señales leídas de las fuentes y órdenes en los destinos, con riesgo agregado; persiste tras recargar.
