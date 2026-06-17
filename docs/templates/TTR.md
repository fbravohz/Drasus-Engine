# Plantilla: TTR (Tarea de Funcionalidad — Cambia Rápido)

**¿Cuándo usar?** Cuando describes qué funcionalidad hay que construir. ESTO VA DENTRO de una Feature ([`FEATURE.md`](./FEATURE.md)).

## Nombramiento de TTRs

**Convención SIMPLE:**
- Cada TTR se nombra: `TTR-001`, `TTR-002`, `TTR-003`, etc.
- La numeración es **local a la Feature** (no global)
- Ej: `genetic-builder/` tiene TTR-001, TTR-002, TTR-003
- Ej: `ingest/` tiene su propio TTR-001, TTR-002 (no conflicto)
- ✅ Simple: "En genetic-builder, implementa TTR-002"

---

## Formato (SIMPLE Y CLARO)

**Título:** [Qué se necesita hacer - presente]

* **¿Cuál es el problema?** (Por qué lo hacemos)
  - Explicación en 2-3 frases, como si le hablaras a alguien que NO conoce el proyecto

* **¿Qué tiene que pasar?** (Comportamiento observable)
  - Ej: "El sistema genera estrategias que no son todas iguales" (NO: "evita convergencia prematura")
  - Ej: "Cuando hay datos malos, el sistema los rechaza" (NO: "validación en frontera")

* **¿Cómo sé que está hecho?** (Pruebas reales que puedo hacer)
  - [ ] Prueba A
  - [ ] Prueba B
  - [ ] Puedo ver el comportamiento en logs/UI

* **¿Qué no puede pasar?** (Restricciones)
  - Esto se RECHAZA
  - Esto es PROHIBIDO
  - Esto NUNCA debe ocurrir

* **¿Hay problemas o dudas?** (Bloqueantes)
  - Si hay algo que no sabemos, lo dejamos aquí

---

## Ejemplo TTR #1 (CORRECTO)

**TTR-001: Estrategias generadas deben ser diferentes entre sí** *(Dentro de Genetic Builder)*

* **¿Cuál es el problema?**
  Cuando el algoritmo genético corre 1000 generaciones, a veces termina creando 100 estrategias que son casi idénticas. Eso es inútil — queremos 100 estrategias DIFERENTES para elegir la mejor.

* **¿Qué tiene que pasar?**
  Después de que el algoritmo termina, miro la lista de 100 mejores estrategias y veo que son DIFERENTES unas de otras (no todas iguales). Por ejemplo: estrategia #1 usa MACD+RSI, estrategia #2 usa Bollinger+ATR, estrategia #3 usa otra cosa distinto.

* **¿Cómo sé que está hecho?**
  - [ ] Puedo correr el algoritmo genético y ver >50 estrategias diferentes en la lista final (mido diferencia por indicadores usado)
  - [ ] Hay un reporte que dice "Diversidad: 85%" (significa estrategias son muy diferentes)
  - [ ] Si corro el algoritmo 2 veces, obtengo resultados diferentes (variedad real, no fijo)

* **¿Qué no puede pasar?**
  - No pueden generarse 100 estrategias todas iguales (eso es un BUG)
  - Si veo que convergen rápido, el sistema debe crear nuevas estrategias raras para mantener variedad
  - No puedo permitir que 90% de estrategias usen el mismo indicador (eso es falta de diversidad)

* **¿Hay problemas o dudas?**
  - Aún no sabemos exactamente cómo medir "diferentes" (¿número de indicadores distintos? ¿parámetros?) — eso lo decide el equipo

---

## Ejemplo TTR #2 (CORRECTO)

**TTR-002: Sistema se adapta según si estamos descubriendo o trading con dinero real** *(Dentro de Genetic Builder)*

* **¿Cuál es el problema?**
  Cuando estoy descubriendo estrategias sin dinero, quiero máximas ganancias (tomo riesgo). Cuando estoy operando con capital real, quiero máxima estabilidad (conservador). El mismo algoritmo genético NO puede optimizar para ambos a la vez.

* **¿Qué tiene que pasar?**
  - En "modo descubrimiento" (sin dinero): el sistema prioriza ganancias altas, aunque haya riesgo
  - En "modo live" (con dinero): el sistema prioriza estabilidad, aunque ganancias sean menores
  - Son modos distintos — no cambias de uno a otro sin detener y reiniciar

* **¿Cómo sé que está hecho?**
  - [ ] En modo descubrimiento, veo estrategias con Sharpe = 2.5 (buena ganancia)
  - [ ] En modo live, veo estrategias con Sharpe = 1.2 pero DrawDown = -5% (más conservadora)
  - [ ] Estrategias generadas en "descubrimiento" son distintas a las de "live"
  - [ ] En los logs veo qué modo estoy usando

* **¿Qué no puede pasar?**
  - No puedo cambiar de modo a mitad de evolución (confunde el algoritmo)
  - No puedo generar estrategias "live" que sean agresivas
  - No puedo generar estrategias "descubrimiento" que sean defensivas

* **¿Hay problemas o dudas?**
  - ¿Exactamente qué parámetros cambian de un modo a otro? (Lo define el equipo técnico)

---

## Ejemplo TTR #3 (CORRECTO)

**TTR-001: Sistema rechaza datos de precios que están rotos** *(Dentro de Ingest)*

* **¿Cuál es el problema?**
  A veces el broker envía datos raros (precio negativo, volumen 0, OHLC inversos). Si usamos esos datos, todo el backtest falla.

* **¿Qué tiene que pasar?**
  Cuando llegan datos rotos, el sistema:
  1. Los detecta
  2. NO los guarda en la base de datos
  3. Loguea qué pasó (para que veas que hubo un problema)

* **¿Cómo sé que está hecho?**
  - [ ] Si envío un precio negativo, el sistema lo rechaza y veo un log que dice "Precio rechazado: -5.23"
  - [ ] Si envío OHLC inversas (Open=100, High=50), el sistema lo rechaza y loguea
  - [ ] Si envío volumen=0, lo rechaza
  - [ ] En la base de datos NO hay datos rotos (los rechazados no están guardados)

* **¿Qué no puede pasar?**
  - No puedo guardar un precio negativo
  - No puedo guardar volumen negativo
  - No puedo guardar High < Low
  - No puedo guardar datos sin loguear por qué se rechazaron

* **¿Hay problemas o dudas?**
  - ¿Exactamente qué constituye "dato roto"? (Lista completa la decide el equipo)

---

Ver reglas transversales (Lo Prohibido, Regla de Oro de Configurabilidad, Checklist) en [`TEMPLATES.md`](./TEMPLATES.md).
