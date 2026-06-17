# Plantillas de Distribución de Contenido — Índice

Cómo escribir decisiones arquitectónicas vs requisitos de funcionalidad, en lenguaje simple. Cada plantilla vive en su propio archivo bajo esta carpeta (mismo patrón que [`docs/ADR.md`](../ADR.md) y [`docs/SAD.md`](../SAD.md)); este archivo es el índice + las reglas transversales que aplican a TODAS las plantillas. Para una plantilla concreta, abre su archivo — no hace falta cargar las demás.

| Plantilla | Cuándo usar | Archivo |
|---|---|---|
| **ADR** | Decisión arquitectónica que dura años (no cambia en 6 meses) | [`ADR.md`](./ADR.md) |
| **SAD** | Visión general del sistema, flujos, invariantes | [`SAD.md`](./SAD.md) |
| **Feature** | Especificación funcional completa de un módulo/componente | [`FEATURE.md`](./FEATURE.md) |
| **TTR** | Tarea concreta de funcionalidad, dentro de una Feature | [`TTR.md`](./TTR.md) |

**Checklist rápido para decidir cuál usar:**
- [ ] ¿Es arquitectura (dura años, NUNCA/SIEMPRE)? → ADR
- [ ] ¿Es la visión/invariante de TODO el sistema? → SAD
- [ ] ¿Es una funcionalidad completa (módulo/componente reutilizable)? → Feature
- [ ] ¿Es una tarea concreta dentro de una Feature ya definida? → TTR

---

## Lo Prohibido (Reglas Transversales — Todas las Plantillas)

### Pseudocódigo / Especulación Técnica — ESTÁ PROHIBIDO

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

## Regla de Oro: Todo es Configurable

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

## Cómo Escribir Parámetros

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

## Checklist Antes de Guardar

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
- Mencionar arquitectura en una Feature/TTR (solo requisitos)
- Valores hardcodeados (siempre preguntarse: ¿esto debería ser configurable?)

**Antes de distribuir contenido:**
- [ ] ¿Es arquitectura? → SAD/ADR
- [ ] ¿Es funcionalidad? → `./features/[feature-name].md`

**Dentro de una Feature o TTR:**
- [ ] ¿Un niño de 10 años entiende qué se necesita?
- [ ] ¿Hay ejemplos concretos (números, casos reales)?
- [ ] ¿Las pruebas son cosas que puedo MEDIR o VER?
- [ ] ¿ZERO jerga sin explicación inline?
- [ ] ¿ZERO pseudocódigo o nombres ficticios?
- [ ] ¿Parámetros están marcados como "configurable" o "[FIJO]"?

**Estructura de features:**
- [ ] ¿Cada feature vive en su propia carpeta o archivo? (`./features/kebab-case.md`)
- [ ] ¿Hay una Feature que describe la funcionalidad COMPLETA?
- [ ] ¿TTRs están dentro de la Feature (no sueltos en el ROADMAP)?
- [ ] **Módulos:** ¿Los TTRs son de orquestación e incluyen hipervínculos a las Features?
