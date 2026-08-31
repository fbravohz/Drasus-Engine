# Hierarchical Parameter Optimization

**Carpeta:** `./features/hierarchical-parameter-optimization/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020

---

## ¿Qué es?

La Optimización Jerárquica de Parámetros es un proceso controlado de mapeo secuencial. En lugar de optimizar todas las variables a la vez ("fuerza bruta" ciega), este módulo ordena los parámetros desde el más macro (inercia, contexto de mercado) hasta el más micro (ajuste fino de salida) para optimizarlos en cascada, mapeando y respetando sus dependencias lógicas.

**Problema que resuelve:** Optimizar todos los parámetros al mismo tiempo crea "Frankensteins" estadísticos donde un indicador cancela al otro y todo se mezcla (conflicto de variables). La optimización jerárquica aísla la estacionalidad variable-por-variable encontrando el verdadero centro de gravedad paramétrico.

---

## Narrativa de Producto

**Capacidad:** `CAP-03` — Búsqueda automática en el espacio de estrategias

**Historia de usuario:** Como investigador de estrategias, quiero ajustar los parámetros de mi estrategia en un orden lógico —primero lo estructural, después lo fino— para no terminar con una mezcla de números que casualmente cuadra en el pasado pero no significa nada.

**El problema (técnico):** optimizar todos los parámetros de una estrategia a la vez —el filtro de tendencia, el momentum, la volatilidad, el stop— deja que cada uno compense los errores de los demás hasta que el conjunto se ve bien sin que ninguna pieza individual tenga sentido: un "Frankenstein" estadístico donde un indicador cancela al otro.

**La solución (técnica):** una optimización en cascada que respeta una jerarquía declarada de lo macro a lo micro —tendencia primero, luego momentum, luego volatilidad, luego salida—. Fija todos los parámetros excepto los del primer nivel, encuentra su mejor valor, lo bloquea, y avanza al siguiente nivel. El resultado es una ruta de ajuste que un humano puede seguir y entender, no una mezcla opaca de números que cuadraron juntos por casualidad.

**Explicado sin tecnicismos:** imagina que armas un mueble siguiendo el instructivo en orden —primero la base, luego las paredes, después las puertas— en vez de atornillar todo a la vez esperando que encaje.

Si ajustas todos los tornillos de un mueble simultáneamente sin orden, es fácil terminar con una pieza torcida que compensó el error de otra sin que ninguna de las dos esté realmente bien puesta. Esta pieza ajusta tu estrategia en ese mismo orden: primero lo grande, después lo fino, un nivel a la vez.

---

## Comportamientos Observables

- [ ] Identifica automáticamente (o por configuración) la jerarquía paramétrica: 1º Trend, 2º Momentum, 3º Volatilidad, 4º Stop Loss.
- [ ] **[OLD-SCHOOL] Sequential Optimization:** Motor iterativo variable-por-variable que optimiza de forma sucesiva mapeando la estacionalidad de "Centros de Gravedad" hiper-espaciales.
- [ ] Fija todos los parámetros en su valor base y optimiza de forma sucesiva el primer nivel (Trend).
- [ ] Elige el mejor centro de gravedad para el Nivel 1. Lo bloquea, y procede a optimizar el Nivel 2.
- [ ] Crea una ruta de optimización lógica comprensible por un humano y matemáticamente estable.

---

## Restricciones

- **FIJO:** Los parámetros deben estar obligatoriamente etiquetados (tagged) en su genoma con su Nivel de Jerarquía. Una variable Nivel 3 nunca se optimizará antes que una Nivel 1.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| HIERARCHY_STAGES | 4 | 2 - 6 | Número de etapas de bloqueo secuencial (ej. Macro -> Signal -> Risk). | CONFIG |

---

## Ciclo de Vida de la Feature — Hierarchical Optimization

### Entrada
- Estrategia con parámetros tagueados por jerarquía.
- Motor de Simulación Histórica.

### Proceso
- Lee jerarquía de componentes lógicos.
- Itera variables de `Etapa N`, dejando inmutables `Etapas N+1, N+2...` en su Default.
- Fija `Etapa N` en la meseta óptima encontrada. Avanza.

### Salida
- `sequentially_optimized_parameters`.
- Historial de Centros de Gravedad.

### Contextos de Uso
**Contexto 1: Optimización de Vieja Escuela (Old-School Validations)**
- Permite mapear la estacionalidad secuencial de manera inmensamente más segura que un algoritmo genético bruto, forzando la convergencia hacia componentes estables y estructurados lógicamente.

---

## Tareas (TTRs)

### **TTR-001: Etiquetado Automático de Jerarquías Paramétricas**
*   **¿Cuál es el problema?** El sistema no sabe qué parámetro es más importante.
*   **¿Qué tiene que pasar?** El sistema pre-etiqueta filtros de largo plazo (SMA 200) como Nivel 1, y filtros tácticos (Trailing Stop) como Nivel 4.
*   **¿Cómo sé que está hecho?**
    - [ ] El AST (genoma) muestra cada parámetro con un `hierarchy_level`.

### **TTR-002: Motor de Optimización Secuencial (Bloqueo Cascada)**
*   **¿Cuál es el problema?** Hay que evitar que la optimización revuelva todas las variables a la vez.
*   **¿Qué tiene que pasar?** El motor fija todas las variables excepto la Nivel 1. La optimiza, encuentra su centro y la fija. Sigue al Nivel 2, y repite el proceso recursivamente.
*   **¿Cómo sé que está hecho?**
    - [ ] El log imprime "Optimizando Nivel 1 (Trend). Parámetro Fijado en 150... Procediendo a Nivel 2 (Risk)."

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
