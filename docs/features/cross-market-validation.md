# Cross-Market Validation

**Carpeta:** `./features/cross-market-validation/`
**Estado:** En Diseño
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0049, ADR-0108, ADR-0110

## 1. ¿Qué es esta feature?
Una prueba de robustez fundamental que somete la estrategia a la iteración en mercados hermanos (correlacionados) sin reoptimizar parámetros. 
Resuelve el problema del sobreajuste (Curve Fitting). Si una ineficiencia es real, debe manifestarse en ecosistemas con impulsores macroeconómicos similares (ej. EURUSD vs GBPUSD).

**Validación Cruzada de la Máscara del Genoma de Régimen y Filtro de Entorno (ADR-0108/ADR-0110):** cuando ese genoma está activo, la máscara binaria Permitido/Prohibido resuelta sobre el mercado principal se aplica también a la(s) cesta(s) correlacionada(s) de `CORRELATION_BASKET`. Si la máscara produce una distribución de Permitido/Prohibido sustancialmente distinta entre el mercado principal y el correlacionado, se interpreta como evidencia de sobreajuste del genoma de régimen al mercado principal.

## 2. Comportamientos Observables
- [ ] El motor ejecuta la estrategia en su mercado principal y en una o más cestas correlacionadas.
- [ ] Si la curva de rendimiento colapsa (excede degradación máxima) en el mercado correlacionado, la estrategia se descarta antes de llegar a la incubación.
- [ ] Cuando el Genoma de Régimen y Filtro de Entorno (ADR-0110) está activo, el motor compara la distribución temporal de la máscara Permitido/Prohibido entre el mercado principal y cada mercado de `CORRELATION_BASKET`.

## 3. Restricciones
- NUNCA iterar la validación sobre mercados descorrelacionados (ej. divisas vs metales). Probar un modelo de Forex en Oro fallará naturalmente y no probaría sobreajuste.

## 4. Parámetros Configurables
| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| CORRELATION_BASKET | [] | Lista de Cadenas | Símbolos correlacionados requeridos para prueba | CONFIG |
| MAX_DEGRADATION | 0.40 | 0.0 - 1.0 | Porcentaje de caída máxima tolerable en rendimiento vs principal | CONFIG |

## 5. Estructura Interna (FCIS)
- **Core (Lógica Pura):** Motor estadístico comparativo (degradación de métricas como Profit Factor o Sharpe).
- **Shell (Infraestructura):** Generación concurrente de backtests sin renderizado UI.
- **Frontera Pública:** Interfaz que recibe el AST y retorna la Matriz de Degradación.

## 6. Ciclo de Vida de la Feature
### Entrada
- AST de la estrategia validada preliminarmente.
- Serie histórica del símbolo correlacionado.
### Proceso
- Inicia el backtest en la cesta secundaria bajo las reglas exactas del principal.
- Computa la variación de ratios de riesgo/retorno.
### Salida
- Matriz de Robustez Cruzada.
- Veredicto de Sobreajuste.
### Contextos de Uso
- **Contexto 1: Validación Pre-Incubación (validate)**
  - Muro de fuego contra estrategias frágiles.

## 7. Tareas (TTRs)
### **TTR-001: Ejecución de Canasta Correlacionada**
* **¿Cuál es el problema?** El motor genético es experto en sobreajustar ruido; necesitamos evidencia externa de robustez.
* **¿Qué tiene que pasar?** El módulo orquesta ejecuciones clonadas en los símbolos dictados y extrae la matriz de resultados comparativa.
* **¿Cómo sé que está hecho?**
  - [ ] Proveo una estrategia a EURUSD, la prueba corre también en GBPUSD automáticamente.
  - [ ] Si la degradación de Sharpe es del 60%, el sistema la clasifica como "RECHAZADA".
* **¿Qué no puede pasar?**
  - Que se pruebe en un mercado sin correlación comprobada, causando falsos positivos de fragilidad.

### **TTR-002: Validación Cruzada de la Máscara de Régimen y Filtro (ADR-0108/ADR-0110)**
* **¿Cuál es el problema?** Un Genoma de Régimen y Filtro de Entorno puede sobreajustarse a la idiosincrasia de un único mercado, produciendo una máscara Permitido/Prohibido que no generaliza a mercados hermanos.
* **¿Qué tiene que pasar?** Cuando ese genoma está activo, el módulo recalcula los Genes de Condición de Estado (Hurst, entropía de Shannon, pendientes Hull MA, `regime_label`) y la máscara resultante sobre cada mercado de `CORRELATION_BASKET`, y compara la proporción de barras Permitidas entre mercados.
* **¿Cómo sé que está hecho?**
    - [ ] Si la proporción de barras Permitidas difiere más de `MAX_DEGRADATION` entre el mercado principal y un mercado correlacionado, el Manifest se clasifica como "RECHAZADA".
* **¿Qué no puede pasar?** Esta validación no se omite para Manifests con el Genoma de Régimen y Filtro de Entorno activo, incluso si el resto de la estrategia ya pasó Cross-Market Validation en una corrida previa sin ese genoma.

## 8. Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020 V2):**
  - **Perfil B (IA / R&D):** motor estadístico comparativo entre mercados; la etiqueta "Ops/Validación" no existe en la tabla canónica.
  - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
  - **II. Soberanía & Propiedad:** `owner_id`, `manifest_id`.
  - **III. Pesos/Arquitectura (subset):** `logic_hash`, `data_snapshot_id`, `version_node_id`, `parent_id` (linaje de la validación).
  - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
  - **Híbrido (V Gobernanza):** `portfolio_container_id` — agrupador de portafolio; campo de Grupo V que esta feature B necesita, documentado aquí como híbrido (mantra de inclusión).
  - **Rastro de Evidencia:** El nivel de degradación en mercados hermanos se envía a `feedback` como rastro de estabilidad.
- **Genomas Modulares por Dominio (ADR-0108/ADR-0110):** esta feature es parte de la Compuerta de Robustez del Dominio de Régimen y Filtro de Entorno (validación cruzada de la máscara Permitido/Prohibido). Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
