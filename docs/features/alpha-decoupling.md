# Alpha Decoupling Module

**Carpeta:** `./features/alpha-decoupling/`
**Estado:** En Diseño
**Última actualización:** 2026-04-28
**Decisión Arquitectónica Asociada:** ADR-0048

## 1. ¿Qué es esta feature?
Aisla el rendimiento puro de la estrategia (Alpha) eliminando el efecto inercial del mercado general (Beta). 
Resuelve el problema de falsos positivos donde una estrategia parece ser brillante, pero sus ganancias solo provienen de estar comprada en un mercado alcista masivo.

## 2. Comportamientos Observables
- [ ] Al validar una estrategia, el sistema calcula qué porcentaje de su rendimiento proviene de seguir pasivamente el mercado.
- [ ] La interfaz expone el "Alpha Puro" separado del "Retorno de Benchmark".

## 3. Restricciones
- El usuario DEBE proveer un activo de referencia (benchmark). Si no se define, la métrica no se calcula.
- NUNCA se debe usar esta métrica como único factor de aprobación; es un filtro adicional de robustez.

## 4. Parámetros Configurables
| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| BENCHMARK_SYMBOL | SPY / BTC | Cadenas | Define qué activo se usa para neutralizar el Beta | CONFIG |
| MIN_PURE_ALPHA | 0.05 | 0.0 - 1.0 | Umbral de Alpha mínimo aceptado para aprobar | CONFIG |

## 5. Estructura Interna (FCIS)
- **Core (Lógica Pura):** Computación de covarianza y varianza vectorizada (Polars/Rust SIMD-Rayon).
- **Shell (Infraestructura):** Registro persistente de la métrica y del Beta de la versión.
- **Frontera Pública:** Interfaz que recibe serie de equidad de la estrategia y serie de benchmark.

## 6. Ciclo de Vida de la Feature
### Entrada
- Serie temporal de rendimiento (PnL) de la estrategia.
- Serie temporal (OHLCV) del benchmark.
### Proceso
- Descompone la exposición al riesgo de mercado.
- Computa el Beta estadístico.
- Sustrae la contribución del Beta al PnL total para extraer el Alpha.
### Salida
- Métrica de Alpha Puro y Score de exposición sistemática.
### Contextos de Uso
- **Contexto 1: Validación (validate)**
  - Asegura que el algoritmo haya descubierto ineficiencia real, no solo sesgo direccional.
- **Contexto 2: Retroalimentación (feedback)**
  - Audita el desgaste paulatino del Alpha Puro durante la ejecución real (Paper/Live).

## 7. Tareas (TTRs)
### **TTR-001: Extracción Analítica de Alpha**
* **¿Cuál es el problema?** Distinguir habilidad real (Alpha) vs suerte por mercado alcista (Beta).
* **¿Qué tiene que pasar?** El sistema procesa los retornos del algoritmo, los enfrenta al benchmark y elimina el componente de inercia.
* **¿Cómo sé que está hecho?**
  - [ ] Si configuro un algoritmo "Buy and Hold SPY" frente a un benchmark SPY, el Alpha resultante debe ser 0.
  - [ ] El reporte de validación imprime explícitamente "Alpha: X%, Beta: Y".
* **¿Qué no puede pasar?**
  - Cálculos de covarianza iterativos lentos; deben ser vectorizados.

## 8. Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad:** Ejecución asíncrona sobre matrices `ndarray` Rust nativo post-simulación.
- **Inundación de Fundaciones (ADR-0020 V2):**
  - **Perfil R&D / Auditoría:** Identidad + Soberanía + Hardware.
  - **Contrato de Persistencia:** Campos de auditoría maestra (id, created_at, audit_hash, version_node_id, logic_hash, indicator_state_hash, institutional_tag).
  - **Rastro de Evidencia:** El valor del Alpha puro y su curva de degradación se emite continuamente a `feedback`.
