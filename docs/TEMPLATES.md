---
name: Plantillas de Distribución de Contenido
description: Cómo escribir decisiones arquitectónicas vs requisitos de funcionalidades - lenguaje simple
type: templates
---

# 📋 Cómo Escribir Documentación: ARQUITECTURA vs FUNCIONALIDAD

---

## 1. PLANTILLA ADR (Decisiones Arquitectónicas - Duran Años)

**¿Cuándo usar?** Cuando decides algo que NO va a cambiar en 6 meses. Ej: "Usamos Python", "Estructura de 8 módulos", "Precios siempre enteros".

### Formato ADR

**Título:** [Decisión corta]

* **¿Qué decidimos?** (1 línea clara)
  
* **¿Por qué lo decidimos?** (El problema que resuelve)
  
* **¿Qué restricciones tiene?** (Lo que NUNCA puede pasar)
  
* **¿Cómo se vería en el sistema?** (Efecto observable, SIN código)
  
* **¿Qué cuesta?** (Trade-off real)
  
* **Trazabilidad:** [Nombre de las Features que implementan este ADR]

---

### Ejemplo ADR (CORRECTO):

**ADR-0007: Estrategias pueden pausarse antes de retirarse**

* **Decisión:** Entre "está operando" y "está retirada", siempre hay un estado "pausada" donde el usuario puede cambiar de idea.

* **Problema que resuelve:** Si una estrategia tiene una mala semana, queremos poder pausarla 1-2 días sin borrarla para siempre. Después el usuario decide si la reactiva o la retira.

* **Restricciones:** 
  - No puedes retirar una estrategia directamente (siempre pasa por PAUSED primero)
  - La ventana de veto (tiempo para cambiar idea) es configurable pero NO infinita
  
* **Efecto observable:**
  - Usuario ve opción "Pausar por 1 día"
  - Después de ese día, puede reactivarla o retirarla
  - Si no decide nada, se retira automáticamente

* **Costo:** Complejidad extra en la máquina de estados (más transiciones). Beneficio: flexibilidad operacional sin perder datos históricos.

---

## 2. PLANTILLA SAD (Documento de Arquitectura - Lo Fundamental)

**¿Cuándo usar?** Cuando actualizas la visión general del sistema, flujos, o invariantes que duran años.

### Secciones del SAD

* **Introducción:** ¿Qué hace el sistema? (1 párrafo)
* **Decisiones Base:** Tabla de ADRs que lo fundamentan
* **Flujos Principales:** Cómo se mueven los datos entre módulos
* **Invariantes:** Lo que NUNCA puede pasar (ej: "margen negativo = error")
* **Propiedades:** Latencia, throughput, disponibilidad

---

### Ejemplo SAD (CORRECTO):

**Sección de Invariantes:**

* **Margen nunca es negativo:** Si una orden llevaría el margen a negativo, se rechaza antes de enviarla al broker. Razón: margen negativo = llamada de margen = fuera de control.

* **Datos sin validar no se usan:** Antes de que cualquier módulo use datos (precios, órdenes, posiciones), pasan por validación. Si fallan, se loguean como anomalía y se descartan. Razón: datos malos contaminan todo (backtests, decisiones, auditoría).

* **Estados son auditables:** Cada cambio de estado (orden PENDIENTE → ENVIADA) se loguea con timestamp. Razón: regulación y debugging.

---

## 3. PLANTILLA  (Especificación Completa de Módulo/Feature)

**¿Cuándo usar?** Cuando describes una funcionalidad COMPLETA (módulo, feature). Este es el documento **maestro** que vive en `./features/[feature-name].md`.

### Estructura 

**Encabezado:**
```
# [Nombre de Feature]

**Carpeta:** `./features/[kebab-case]/`
**Estado:** [Pendiente | En Diseño | Lista para implementar]
**Última actualización:** [fecha]
**Decisión Arquitectónica Asociada:** ADR-XXXX (Título del ADR)
```

**Secciones:**

1. **¿Qué es esta feature?** (Descripción COMPLETA)
   - Qué problema resuelve (contexto)
   - Comportamiento observable (qué hace el usuario)
   - Por qué la necesitamos

2. **Comportamientos Observables** (Lo que se ve/prueba)
   - Lista de "cuando X pasa, el sistema hace Y"
   - Ejemplos concretos

3. **Restricciones** (Lo que NUNCA puede pasar)
   - Límites técnicos
   - Límites de negocio

4. **Parámetros Configurables** (ADR-0008)
   - Tabla: Parámetro | Default | Rango | Qué hace | FIJO/CONFIG
   - Indica qué parámetros puede cambiar el usuario vs cuáles son decisiones fijas.

5. **Estructura Interna (FCIS — ADR-0002)**
   - **Core (Lógica Pura):** Qué funciones no tocan DB/IO.
   - **Shell (Infraestructura):** Cómo maneja el orquestador la persistencia y eventos.
   - **Frontera Pública:** El contrato para interactuar con otros módulos.

6. **Ciclo de Vida de la Feature** (OBLIGATORIO si es componente reutilizable)
   - Entrada: Qué necesita
   - Proceso: Qué transformación realiza (lenguaje observable)
   - Salida: Qué produce
   - Contextos de Uso: En qué módulos se aplica y cómo cambia su rol
   - (Ver sección 3B de este template para formato completo)

6. **Tareas (TTRs)** (Partes atacables)
   - TTR-001, TTR-002, TTR-003, etc. (numeración simple dentro de este )
   - Cada una con su propia especificación

7. **Gobernanza y Estándares (Fijos)**
   - **Local-First (ADR-0016):** [100% Local | Justificar dependencia externa].
   - **Fidelidad (ADR-0017):** [Baja (Barras) | Alta (4-ticks) | Institucional (Slippage Dinámico)].
   - **Inundación de Fundaciones (ADR-0020 V2):** 
      - **OBLIGATORIO:** El **Grupo I (Identidad & Integridad)** es universal — va siempre, en toda tabla. Además, asigna a la feature UNO de los 4 Perfiles Técnicos de la **tabla canónica de ADR-0020 V2** (no la copies aquí, referénciala):
         - **A. Datos / Ingest:** Identidad + Linaje de Datos (III) + Hardware (IV).
         - **B. IA / R&D:** Identidad + Soberanía (II) + Pesos/Arquitectura, subset de III (IV).
         - **C. Ops / Hot-Path:** Identidad + Soberanía (II) + Hardware (IV) + Latencia, subset de V (≤1ms).
         - **D. Ops / Auditoría:** Identidad + Soberanía (II) + Hardware (IV).
      - **PROHIBIDO copy-paste masivo:** de cada grupo asignado, toma solo los campos concretos relevantes para esta tabla — no el grupo completo.
      - **Hooks Forenses:** [Describir ganchos específicos: ej. latencia de DOM, linaje de genoma, firmas digitales].
   - **Contrato de Persistencia:** 
     - Tabla `Categoría | Campo | Descripción` con el Grupo I completo + los campos concretos del Perfil elegido (ver `features/adaptive-volume-indicators.md` como ejemplo de formato).
   - **Rastro de Evidencia:** Qué datos específicos emite para el módulo de `feedback` (causalidad).

8. **Decisión Arquitectónica Asociada** (Solo si aplica)
   - Referencia al ADR correspondiente
   - (Omitir si la feature NO tiene decisión arquitectónica nueva)

8. **Dependencias y Bloqueantes**
   - Qué otras features dependen de esta.
   - Qué la bloquea (ej: otra feature no implementada).

9. **Regla de Soberanía Técnica (§7.3)** (Solo para Módulos):
   - Los Módulos son **Orquestadores Puros** (Thin Shells).
   - Sus TTRs deben ser de integración y DEBEN incluir hipervínculos de Markdown a los archivos `` de las features que consumen.

---

### Ejemplo  (CORRECTO):

```markdown
# Genetic Builder

**Carpeta:** `./features/genetic-builder/`
**Estado:** En Diseño
**Última actualización:** 2026-04-04

## ¿Qué es?

El motor genético es el corazón del módulo "generar". 
Toma datos históricos de barras y crea automáticamente estrategias candidatas 
usando evolución genética (selecciona y cruza las mejores, descarta las peores).

**Problema:** Descubrir estrategias nuevas es tedioso si las haces a mano. 
El motor genético lo automatiza — genera 1000 variaciones, prueba todas, 
devuelve las mejores.

## Comportamientos Observables

- [ ] Usuario presiona "Generar candidatos" con 2 años de datos históricos
  → Sistema crea 100 estrategias diferentes (no todas iguales)
  → Se loguea progreso cada 10 generaciones

- [ ] Estrategias generadas tienen indicadores variados 
  (Ej: #1 es MACD+RSI, #2 es Bollinger+ATR, #3 es otra combinación)
  → Usuario ve diversidad (no clones)

- [ ] Si en 50 generaciones la calidad no mejora, sistema renueva población
  → Mata las 20 peores, genera 20 nuevas aleatorias
  → Registra evento "renovación por estancamiento"

## Restricciones

- NUNCA una estrategia > 10 indicadores (demasiada complejidad = overfitting)
- NUNCA período indicador > 500 barras (datos históricos mínimos)
- NUNCA fitness score negativo (indicaría error de cálculo)

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| POPULATION_SIZE | 100 | 50-500 | Cuántas estrategias en cada generación | CONFIG |
| GENERATIONS | 100 | 10-500 | Cuántas generaciones evolucionar | CONFIG |
| STAGNATION_THRESHOLD | 1% | 0.1%-5% | Si mejora < 1% en N gen → renovar | CONFIG |
| MAX_INDICATORS | 10 | 3-20 | Máximo indicadores por estrategia | CONFIG |
| LONG_SHORT_MODE | asymmetric | symmetric / asymmetric | Reglas L/S iguales o distintas | [FIJO] |

## Tareas (TTRs)

### **TTR-001: Generar población inicial**
(Ver especificación completa abajo)

### **TTR-002: Evolucionar población (cruce + mutación)**
(Ver especificación completa abajo)

### **TTR-003: Detectar estancamiento y renovar**
(Ver especificación completa abajo)

## Decisión Arquitectónica Asociada

- ADR-0008: Configurabilidad Universal (todos los parámetros de Genetic Builder son ajustables)

## Dependencias

**Depende de:**
- Módulo ingest (datos OHLCV validados)

**Bloquea:**
- Módulo validar (necesita candidatos para validar)
```

---

## 3B. CICLO DE VIDA DE FEATURE (Componentes Reutilizables)

**¿Qué son las features?** Componentes funcionales **reutilizables** que pueden ser usados en múltiples módulos o contextos. No son exclusivas de un módulo — pueden compartirse.

**Ejemplo:**
- Feature: **Walk Forward Analysis (WFA)**
  - Usada en `validate`: para probar robustez de **estrategias individuales**
  - Usada en `manage`: para validar robustez del **portafolio completo**
  - Usada en `incubate`: para comparar paper trading en **ventanas de tiempo distintas**

### Sección Obligatoria: "Ciclo de Vida de la Feature"

Cada  debe incluir una sección que describe entrada → proceso → salida, **sin detalles de implementación**:

```
## Ciclo de Vida de la Feature — [Nombre]

### Entrada
Qué datos, objetos, o condiciones necesita para funcionar:
- Datos históricos completos (ej: barras validadas de últimos 2 años)
- Parámetros de configuración (ej: tamaño de ventanas, umbrales)
- Estado del sujeto (ej: estrategia con parámetros fijos, portafolio con pesos asignados)

### Proceso
Qué transformación realiza **en lenguaje observable** (sin código, sin nombres de variables):
- Divide los datos históricos en períodos secuenciales
- Ejecuta el sujeto (estrategia/portafolio) en cada período de forma independiente
- Compara resultados entre períodos

### Salida
Qué produce y en qué estado queda:
- Reporte de robustez por período (ej: "funcionó bien en 8 de 10 períodos")
- Score de consistencia (ej: "variación promedio de 12% entre períodos")
- Veredicto: ROBUSTO / DUDOSO / FRÁGIL

### Contextos de Uso
En qué módulos/features se aplica y cómo cambia su rol:

**Contexto 1: Validación de Estrategia Individual**
- Entrada: Estrategia candidata + datos históricos
- Preguntas que responde: "¿Esta estrategia funciona igual en todos los períodos históricos?"
- Impacto: Score de robustez afecta veredicto (APROBADA vs RECHAZADA)

**Contexto 2: Validación de Portafolio**
- Entrada: Portafolio optimizado + datos históricos  
- Preguntas que responde: "¿Esta combinación de estrategias mantiene balance en mercados distintos?"
- Impacto: Decide si portafolio puede ir a LIVE o necesita rebalanceo

**Contexto 3: Comparativa Paper vs Live**
- Entrada: Resultados paper trading históricos vs performance actual
- Preguntas que responde: "¿Hay degradación consistente desde que fue aprobada?"
- Impacto: Detecta si estrategia debe pausarse por cambios de mercado
```

**Regla de Oro:** La sección de Ciclo de Vida describe **QUÉ ENTRA, QUÉ OCURRE, QUÉ SALE** independientemente del contexto. El apartado "Contextos de Uso" explica cómo ese ciclo se materializa en distintos módulos.

---

## 4. PLANTILLA TTR (Tarea de Funcionalidad - Cambia Rápido)

**¿Cuándo usar?** Cuando describes qué funcionalidad hay que construir. ESTO VA DENTRO DE .

### Nombramiento de TTRs

**Convención SIMPLE:**
- Cada TTR se nombra: `TTR-001`, `TTR-002`, `TTR-003`, etc.
- La numeración es **local al ** (no global)
- Ej: `genetic-builder/` tiene TTR-001, TTR-002, TTR-003
- Ej: `ingest/` tiene su propio TTR-001, TTR-002 (no conflicto)
- ✅ Simple: "En genetic-builder, implementa TTR-002"

---

### Formato TTR (SIMPLE Y CLARO)

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

### Ejemplo TTR (CORRECTO):

**TTR-001: Estrategias generadas deben ser diferentes entre sí** *(Dentro de Genetic Builder )*

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

### Ejemplo TTR #2 (CORRECTO):

**TTR-002: Sistema se adapta según si estamos descubriendo o trading con dinero real** *(Dentro de Genetic Builder )*

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

### Ejemplo TTR #3 (CORRECTO):

**TTR-001: Sistema rechaza datos de precios que están rotos** *(Dentro de Ingest )*

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
  - ¿Exactamente qué constituve "dato roto"? (Lista completa la decide el equipo)

---

## 4. CHECKLIST Y PROHIBICIONES CLARAS

### 4.0 LO PROHIBIDO: NO ESCRIBAS NUNCA

**PSEUDOCÓDIGO / ESPECULACIÓN TÉCNICA — ESTÁ PROHIBIDO**

| ❌ PROHIBIDO | ✅ USA EN CAMBIO |
|---|---|
| "Crear clase `BarRepository`" | "El módulo tiene acceso a barras históricos" |
| "Importar `from modules.ingest import...`" | "Integración entre módulos vía API pública" |
| "Paso 1: Crear `/src/modules/ingest/schemas.py`" | "Validación de datos en frontera de entrada" |
| "Función `validate_ohlcv(raw_data: dict)`" | "Datos OHLCV se validan antes de usarse" |
| "Parámetro `test_type`, `renewal_interval`" | "Hay parámetro configurable que..." |
| "Código JSON: `{"cagr": 0.30, "sharpe": 0.40}`" | "Hay parámetros de peso para CAGR y Sharpe" |
| "FSM: `OrderState(IntEnum): PENDING=1`" | "Estados de orden son numerados y deterministas" |
| "Variable `max_conditions = 10`" | "Hay límite configurable de complejidad" |

---

### 4.1 La Regla de Oro: TODO es Configurable

**PRINCIPIO:** Cualquier número, umbral o regla que escribas es configurable A MENOS que digas explícitamente "ESTO NO ES CONFIGURABLE" o sea un invariante arquitectónico (ej: "precios siempre enteros", "margen nunca negativo").

**Ejemplos:**
- ❌ MALO: "Sharpe debe ser > 2 para aprobar"
- ✅ BIEN: "Hay parámetro MIN_SHARPE (default: 2.0) para definir cuándo aprueban estrategias"

- ❌ MALO: "Máximo drawdown 30%"
- ✅ BIEN: "Hay parámetro MAX_DD (default: -0.30 = -30%) configurable por usuario"

- ❌ MALO: "8 módulos en pipeline"
- ✅ BIEN: "Pipeline: ingest → generar → validar → incubar → gestionar → ejecutar → retirar → retroalimentar. Cada módulo se puede activar/desactivar"

**¿Cuándo SÍ está fijo?**
- Invariantes arquitectónicas ("precios = int64", "sin datetime.now() en Core")
- Estructura física ("8 módulos", "/src/modules/ingest/")
- EXPLÍCITAMENTE marcado: "**FIJO:** No se puede cambiar"

---

### 4.2 Cómo Escribir Parámetros

En lugar de fijar un valor, describe el parámetro:

```
Parámetro: MIN_SHARPE
  Default: 2.0
  Rango válido: 0.5 - 10.0
  Impacto: Estrategias con Sharpe < MIN_SHARPE son rechazadas en validación
```

O simplemente en texto natural:
```
Hay un parámetro configurable que define el Sharpe mínimo para aprobar 
(default: 2.0, pero usuarios pueden bajar a 1.0 o subir a 5.0 según su riesgo).
```

---

✅ **HACER:**
- Explicar como si le hablaras a alguien que NO conoce el proyecto
- Usar ejemplos concretos ("precio negativo") NO abstractos ("valor inválido")
- Una prueba = una cosa observable que puedo verificar
- Restricción = algo que RECHAZAR o que NUNCA suceda
- Si escribo un número: indicar si es **FIJO** (invariante) o **CONFIGURABLE** (parámetro)

❌ **NO HACER:**
- Usar palabras técnicas sin explicar (ej: "convergencia prematura", "overfitting", "vectorización")
- Describir CÓMO implementarlo (ej: "crear clase X", "usar algoritmo Y")
- Nombres de variables/archivos ficticios
- Pseudocódigo
- Diagramas de flujo a nivel técnico
- Mencionar funcionalidades en SAD/ADR (solo decisiones base)
- Mencionar arquitectura en BACKLOG (solo requisitos)
- Valores hardcodeados (siempre preguntarse: ¿esto debería ser configurable?)

---

## 5. CHECKLIST ANTES DE GUARDAR

**Antes de distribuir contenido:**
- [ ] ¿Es arquitectura? → SAD/ADR
- [ ] ¿Es funcionalidad? → `./features/[feature-name].md`

**Dentro de  o TTR:**
- [ ] ¿Un niño de 10 años entiende qué se necesita?
- [ ] ¿Hay ejemplos concretos (números, casos reales)?
- [ ] ¿Las pruebas son cosas que puedo MEDIR o VER?
- [ ] ¿ZERO jerga sin explicación inline?
- [ ] ¿ZERO pseudocódigo o nombres ficticios?
- [ ] ¿Parámetros están marcados como "configurable" o "[FIJO]"?

**Estructura de features:**
- [ ] ¿Cada feature vive en su propia carpeta? (`./features/kebab-case/`)
- [ ] ¿Hay  que describe la feature COMPLETA?
- [ ] ¿TTRs están separadas dentro del  (no en BACKLOG)?
- [ ] ¿Hay un README.md en `./features/` como índice?
- [ ] **Módulos:** ¿Los TTRs son de orquestación e incluyen hipervínculos a las Features?
