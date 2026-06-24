# Plantilla: Feature (Especificación Completa de Módulo/Feature)

**¿Cuándo usar?** Cuando describes una funcionalidad COMPLETA (módulo, feature). Este es el documento **maestro** que vive en `./features/[feature-name].md`.

> Nota de restauración (2026-06-16): esta plantilla usaba numeración `1, 2, 3…` con dos secciones repetidas como "6." y dos como "8." (defecto del monolito original `TEMPLATES.md`). El corpus real de Features (`docs/features/*.md`) nunca siguió esa numeración — usa encabezados con nombre, sin números. Esta plantilla se reescribe para coincidir con el corpus real; los encabezados son los mismos, solo sin los números rotos.

## Encabezado

```
# [Nombre de Feature]

**Carpeta:** `./features/[kebab-case]/`
**Estado:** [Pendiente | En Diseño | Lista para implementar]
**Última actualización:** [fecha]
**Decisión Arquitectónica Asociada:** ADR-XXXX (Título del ADR)
```

## ¿Qué es esta feature?

Descripción COMPLETA:
- Qué problema resuelve (contexto)
- Comportamiento observable (qué hace el usuario)
- Por qué la necesitamos

## Comportamientos Observables

Lo que se ve/prueba:
- Lista de "cuando X pasa, el sistema hace Y"
- Ejemplos concretos

## Restricciones

Lo que NUNCA puede pasar:
- Límites técnicos
- Límites de negocio

## Parámetros Configurables (ADR-0008)

Tabla: Parámetro | Default | Rango | Qué hace | FIJO/CONFIG. Indica qué parámetros puede cambiar el usuario vs cuáles son decisiones fijas.

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Qué funciones no tocan DB/IO.
- **Shell (Infraestructura):** Cómo maneja el orquestador la persistencia y eventos.
- **Frontera Pública:** El contrato para interactuar con otros módulos.

## Ciclo de Vida de la Feature — [Nombre] (OBLIGATORIO si es componente reutilizable)

Describe entrada → proceso → salida, **sin detalles de implementación**. Ver formato completo más abajo (sección "Ciclo de Vida de Feature — Formato Detallado").

## Tareas (TTRs)

- TTR-001, TTR-002, TTR-003, etc. (numeración simple, local a esta Feature — ver plantilla [`TTR.md`](./TTR.md))
- Cada una con su propia especificación

## Puertos de Integración (ADR-0137)

> Obligatorio en toda feature. Define los tipos de dato que la feature acepta (inputs) y produce (outputs).
> Los IDs de tipo deben pertenecer al catálogo de ADR-0137. Un puerto sin tipo declarado es inválido en el Canvas [Forge/Reactor].

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `[nombre_puerto]` | `[Bars \| Signal \| Order \| Position \| RobustnessScore \| Capital \| ...]` | Input / Output | `1` / `0..1` / `0..N` / `1..N` | [qué dato entra/sale y qué representa en esta feature] |

> **Cardinalidad:** `1` = exactamente uno · `0..1` = opcional · `0..N` = múltiple · `1..N` = al menos uno.
> **Plomería sin puertos de usuario:** si la feature es plomería pura (bus, clock, queue), declara aun así sus puertos técnicos (ej. `job_in: AsyncJob | Input | 1..N`).

## Cáscara Visual (Thin Shell)

> Autoridad: ADR-0106 · ADR-0136 · ADR-0117 · ADR-0135 · `docs/DESIGN.md` · `docs/DESIGN.md §"Catálogo de Componentes"`
>
> **Producida por el skill `ui-designer` (Etapa 0.5 del Tech Lead) — NUNCA rellenar a mano sin leer DESIGN.md primero.**
> Si la feature es Plomería (declara "Ventana de Verificación"), escribe solo la nota de Ventana de Verificación Visual y omite el resto.

### Contexto de superficie (ADR-0136)
**[Dashboard widget | Canvas — Vista Relacional | Canvas — Vista Interior | Inspector Panel | Plomería]** — [una frase: qué ve el usuario y en qué parte del Canvas o Dashboard]

### Superficie y Densidad
- **Superficie principal:** [panel-solid `panelSolid #0E1426` | panel-glass `glassFill 0x73141C36`]
- **Densidad:** [densa — Dashboard / Vista Relacional | cómoda — Vista Interior / Inspector Panel]
- **Lienzo de fondo:** `deepSpace #080A18` + telón cósmico tenue

### Componentes
| ID de catálogo (DESIGN.md §4–§11) | Rol en esta feature | Tokens clave | Estados semánticos |
|---|---|---|---|
| `[id-kebab-case]` | [qué representa] | [tokens de DESIGN.md] | [óptimo / transición / alerta / crítico] |

### Estados Semánticos (Espectro de Vitalidad)
| Estado de negocio | Color token | Tratamiento visual completo |
|---|---|---|
| [estado] | `optimaCyan #54E8D0` | chip: texto `#54E8D0`, fondo `#08251F`, borde 1px `#1E5E4F`, radio 8px · `glow(optimaCyan)` |

### Layout
- [descripción: grid, columnas, agrupaciones, scroll, paneles laterales]

### Animaciones Aplicables
- [ ] Zoom canvas (relacional ↔ interior) · [ ] Clic botón (propagación de luz) · [ ] Hover (glow intensificado) · [ ] Foco input (glow limpio, sin aberración RGB) · [ ] Dropdown (AnimatedSize + chevron) · [ ] Chip parpadeante · [ ] Loader/progreso (glow pulsante) · [ ] Switch (knob deslizante)

### Notas de implementación para el Flutter Engineer
[Instrucciones específicas: orden de painters, z-index de capas, restricciones de rendimiento]

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** [100% Local | Justificar dependencia externa].
- **Fidelidad (ADR-0017):** [Baja (Barras) | Alta (4-ticks) | Institucional (Slippage Dinámico)].
- **Inundación de Fundaciones (ADR-0020 V2):**
  - **OBLIGATORIO:** El **Grupo I (Identidad & Integridad)** es universal — va siempre, en toda tabla, con sus **6 campos completos**: `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`. No omitas ninguno (causa raíz del patrón P1: faltaban típicamente `updated_at` y `event_sequence_id`). Además, asigna a la feature UNO de los 4 Perfiles Técnicos de la **tabla canónica de ADR-0020 V2** (no la copies aquí, referénciala):
    - **A. Datos / Ingest:** Identidad + Linaje de Datos (III) + Hardware (IV).
    - **B. IA / R&D:** Identidad + Soberanía (II) + Pesos/Arquitectura, subset de III (IV).
    - **C. Ops / Hot-Path:** Identidad + Soberanía (II) + Hardware (IV) + Latencia, subset de V (≤1ms).
    - **D. Ops / Auditoría:** Identidad + Soberanía (II) + Hardware (IV).
  - **PROHIBIDO copy-paste masivo:** de cada grupo asignado, toma solo los campos concretos relevantes para esta tabla — no el grupo completo.
  - **Hooks Forenses:** [Describir ganchos específicos: ej. latencia de DOM, linaje de genoma, firmas digitales].

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Tabla `Categoría | Campo | Descripción` con el Grupo I completo + los campos concretos del Perfil elegido (ver [`backtest-engine.md`](../features/backtest-engine.md) como ejemplo de formato real).

**Rastro de Evidencia:** Qué datos específicos emite para el módulo de `feedback` (causalidad).

## Decisión Arquitectónica Asociada (Solo si aplica)

Referencia al ADR correspondiente. Omitir si la feature NO tiene decisión arquitectónica nueva.

## Dependencias y Bloqueantes

- Qué otras features dependen de esta.
- Qué la bloquea (ej: otra feature no implementada).
- **"Consumido por" = accede al puerto, no al esquema (ADR-0003, Regla de Tabla Única):** cuando esta sección (o un campo "Consumido por") lista varios módulos que usan esta Feature, esos módulos la consumen a través de su `public_interface`. La tabla de la Feature se crea UNA SOLA VEZ, en el módulo dueño (el primer consumidor del pipeline, ADR-0118). Los módulos consumidores NO corren su propia migración ni duplican el esquema; un TTR de integración enchufa el puerto, nunca crea una tabla. Si un consumidor necesita datos propios relacionados, los guarda en sus tablas con una referencia al dato de la Feature.
- **Contrato de Integración UI (OBLIGATORIO — ADR-0117):** declara una de las dos:
  - **Superficie propia:** nombre de la pestaña/sección que esta Feature aporta al Panel Operativo Fundacional, y cuál es su Superficie de Verificación Funcional (SVF) bajo el Techo Fijo — el control que dispara la operación real + el resultado real (FFI/gRPC) + el observable persistido visible tras recargar.
  - **Ventana de Verificación (si es Feature de plomería, sin superficie propia):** nombre de la Feature consumidora y de su pestaña/sección, y el observable concreto de ESTA feature (estado, conteo, timestamp, resultado) que debe quedar visible ahí. Esa visibilidad es la prueba de funcionamiento de la plomería — sin ella, la Cáscara Delgada de la Feature consumidora no se considera completa (Gate de Integración, ADR-0117).

## Regla de Soberanía Técnica (Solo para Módulos)

- Los Módulos son **Orquestadores Puros** (Thin Shells).
- Sus TTRs deben ser de integración y DEBEN incluir hipervínculos de Markdown a los archivos de especificación (`[nombre](../features/nombre.md)`) de las features que consumen.

---

## Ejemplo Feature (CORRECTO)

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

### TTR-001: Generar población inicial
(Ver especificación completa abajo)

### TTR-002: Evolucionar población (cruce + mutación)
(Ver especificación completa abajo)

### TTR-003: Detectar estancamiento y renovar
(Ver especificación completa abajo)

## Decisión Arquitectónica Asociada

- ADR-0008: Configurabilidad Universal (todos los parámetros de Genetic Builder son ajustables)

## Dependencias

**Depende de:**
- Módulo ingest (datos OHLCV validados)

**Bloquea:**
- Módulo validar (necesita candidatos para validar)

## Contrato de Integración UI

**Superficie propia:** pestaña "Generación" en el Panel Operativo Fundacional.
- SVF: botón "Generar candidatos" dispara una corrida real vía `public_interface`; la pestaña muestra el contador de estrategias generadas devuelto por el Core; tras recargar, el contador y el reporte de diversidad siguen visibles (persistidos en el Databank).
```

---

## Ciclo de Vida de Feature — Formato Detallado (Componentes Reutilizables)

**¿Qué son las features?** Componentes funcionales **reutilizables** que pueden ser usados en múltiples módulos o contextos. No son exclusivas de un módulo — pueden compartirse.

**Ejemplo:**
- Feature: **Walk Forward Analysis (WFA)**
  - Usada en `validate`: para probar robustez de **estrategias individuales**
  - Usada en `manage`: para validar robustez del **portafolio completo**
  - Usada en `incubate`: para comparar paper trading en **ventanas de tiempo distintas**

### Sección Obligatoria: "Ciclo de Vida de la Feature"

Cada Feature debe incluir una sección que describe entrada → proceso → salida, **sin detalles de implementación**:

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

Ver reglas transversales (Lo Prohibido, Regla de Oro de Configurabilidad, Checklist) en [`TEMPLATES.md`](./TEMPLATES.md).
