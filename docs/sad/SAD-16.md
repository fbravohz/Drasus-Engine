## 16. Grafo de Dependencias Técnicas (Arquitectura Hexagonal — ADR-0137)

> ⚠️ **Actualizado 2026-06-23** — La estructura de dependencias anterior ("módulo X depende de módulo Y") fue reemplazada por el modelo hexagonal (ADR-0137): cada feature crate depende SOLO de `shared`. Los módulos como dueños runtime no existen; el pipeline es un preset de cableado, no una cadena de dependencias de compilación.

### Regla de dependencia (FIJO)

Toda feature crate de `crates/features/<dominio>/<feature>/` declara UNA sola dependencia:

```
feature crate → shared  (tipos ADR-0137 + plumbing)
```

Prohibido: feature → feature, feature → preset.

### Grafo real (star topology)

```
                    ┌──────────────────┐
                    │     shared       │
                    │  (tipos + infra) │
                    └────────┬─────────┘
           ┌─────────┬───────┼───────┬─────────┐
           ▼         ▼       ▼       ▼         ▼
      backtest-  monte-    wfa-   nsga2-    portfolio-
      engine     carlo     ...    opt       optimizer
```

Cada feature compila aislada. Cambiar `monte-carlo` no recompila `backtest-engine`.

### Orden del pipeline (preset, no dependencia de compilación)

El orden `ingest → generate → validate → incubate → manage → execute → feedback → withdraw` es una **recomendación de cableado**, implementada en el preset `standard-pipeline` (crate `crates/presets/standard-pipeline/`). Un usuario experto puede ignorar este orden en el Canvas [Forge/Reactor] y cablear features directamente por sus puertos tipados.

### Dependencias de datos (runtime, no compilación)

Las features se conectan en runtime a través de sus puertos tipados (ADR-0137). Ejemplo:

```
sovereign-fetcher ──Bars──► data-sanitizer ──SanitizedDataframe──► backtest-engine ──BacktestResult──► wfa-analyzer
```

La validez de cada conexión la verifica el Canvas comparando los tipos de los puertos. Una conexión `Bars → Signal` se marca inválida en el canvas (`criticalCrimson`).

### Orden de construcción (ADR-0118)

El orden de DESARROLLO sí sigue el pipeline, por dependencias de datos reales:
- WFA necesita backtests → backtest necesita datos limpios
- Monte Carlo necesita resultados del motor → motor necesita estrategias generadas

Pero estas son dependencias de DATOS (lo que una feature consume como input), no dependencias de COMPILACIÓN. Cada feature se construye en el momento que su primer consumidor del pipeline la necesita, pero compila de forma aislada contra `shared`.

---

