# Complexity Penalization (Ockham's Razor)

**Carpeta:** `./features/complexity-penalization/`
**Estado:** En Diseño
**Última actualización:** 2026-06-11
**Decisión Arquitectónica Asociada:** ADR-0020 V2, ADR-0108, ADR-0111

---

## ¿Qué es?

La penalización por complejidad es la aplicación directa de la Navaja de Ockham ("en igualdad de condiciones, la explicación más sencilla suele ser la correcta"). Se encarga de castigar el "Overfitting" (sobreajuste) calculando cuántos parámetros lógicos tiene una estrategia comparado con el número de veces que operó en la historia.

**Problema que resuelve:** Una estrategia con 20 parámetros diferentes que solo hizo 50 operaciones en 5 años casi seguro está memorizando el pasado en lugar de entenderlo. Este módulo destruye esas ilusiones rebajando masivamente su Fitness Score.

**Extensión Multi-Dominio (ADR-0108):** un Manifest puede portar simultáneamente varios genomas de dominio (Señal, Riesgo y Gestión de Posición, Régimen y Filtro de Entorno, Portafolio y Correlación), unos activos y otros congelados ("Wildcard Invertido"). Este módulo suma los grados de libertad de **todos** los genomas presentes contra el mismo denominador de operaciones, para que la complejidad evolutiva no pueda "migrar" hacia el dominio que no está siendo penalizado en esa corrida.

---

## Comportamientos Observables

- [ ] El sistema cuenta el número exacto de parámetros optimizables (nodos lógicos) en la estrategia.
- [ ] Calcula el "Degrees of Freedom Ratio" dividiendo los trades históricos entre los parámetros.
- [ ] Se aplica un "Complexity Penalty Score" que resta puntos proporcionales al tamaño de la fórmula lógica.
- [ ] Estrategias con ratios paupérrimos (ej. 2 trades por parámetro) son penalizadas a muerte y rechazadas en el backtest.
- [ ] Cuando el Manifest porta más de un genoma de dominio (ADR-0108), el conteo de parámetros agrega los de **todos** los genomas presentes —activos y congelados— contra el mismo total de trades, y reporta el desglose por dominio.
- [ ] Cuando el Dominio de Portafolio y Correlación está activo (ADR-0111), el sistema calcula además un ratio independiente de "miembros de cartera por parámetro del genoma de portafolio".

---

## Restricciones

- **FIJO:** El motor debe contar *todo* parámetro que no sea un invariante físico, incluyendo ventanas móviles, umbrales de indicadores, multiplicadores ATR y tamaños fijos de pip.
- **FIJO (ADR-0108):** la penalización de un Manifest con múltiples genomas de dominio se calcula sobre la suma de parámetros de todos los genomas presentes, nunca sobre un solo genoma de forma aislada.
- **FIJO (ADR-0111):** el Genoma de Portafolio y Correlación se penaliza con su propio ratio "miembros de cartera por parámetro" además de — y sin sustituir a — el ratio `Total_Trades / N_p` de cada Manifest miembro.
- **FIJO (Reglas Genómicas Cruzadas, ADR-0108):** un parámetro perteneciente a una Regla Genómica que combina genes de más de un dominio activo cuenta una sola vez en `N_p` total, y se refleja en el desglose de **todos** los dominios cuyos genes participan en esa regla.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MIN_TRADES_PER_PARAM | 30 | 10 - 100 | Mínimo de trades requeridos por cada parámetro lógico existente. | CONFIG |
| PENALTY_EXPONENT | 1.5 | 1.0 - 3.0 | Exponente de castigo por parámetros extra (fuerza de la Navaja de Ockham). | CONFIG |
| MIN_MEMBERS_PER_PORTFOLIO_PARAM | 1.0 | 0.5 - 5.0 | Mínimo de miembros de cartera requeridos por cada parámetro del Genoma de Portafolio y Correlación (ADR-0111). | CONFIG |

---

## Ciclo de Vida de la Feature — Complexity Penalization

### Entrada
- Árbol de sintaxis abstracta (AST) o genoma de la estrategia, incluyendo todos los genomas de dominio presentes en el Manifest (ADR-0108).
- Resultados en crudo del backtest (número de operaciones); para el Dominio de Portafolio y Correlación, también el número de miembros de la cartera.

### Proceso
- Analiza el genoma y cuenta la cardinalidad de la matriz de parámetros $P$, sumando los parámetros de todos los genomas de dominio presentes (activos y congelados).
- Intercepta el fitness global y lo castiga severamente si no se cumple el ratio de grados de libertad mínimo.
- Si el Genoma de Portafolio y Correlación está activo, calcula adicionalmente el ratio "miembros de cartera / parámetros del genoma de portafolio" contra `MIN_MEMBERS_PER_PORTFOLIO_PARAM`.

### Salida
- `degrees_of_freedom_ratio` (ej: 45 trades/param), con desglose por dominio genómico.
- `complexity_penalty_factor` (ej: -15% al score de robustez final).
- `portfolio_degrees_of_freedom_ratio` (ej: 5 miembros / 8 parámetros) cuando el Dominio de Portafolio y Correlación está activo.

### Contextos de Uso
**Contexto 1: Control de Población (Validate/Generate)**
- Al final de la generación de una estrategia o durante su validación, funciona como la "Guillotina del Overfitting". Si es innecesariamente compleja, no sobrevive.

**Contexto 2: Generación Multi-Dominio (Generate — ADR-0108)**
- Durante la evolución de cualquier genoma de dominio (Riesgo y Gestión, Régimen y Filtro, Portafolio y Correlación), la penalización agregada de todos los genomas del Manifest interviene en el fitness de NSGA-II, evitando que un dominio "absorba" complejidad para que otro luzca simple.

---

## Tareas (TTRs)

### **TTR-001: Auditoría de Grados de Libertad (Degrees of Freedom Monitor)**
*   **¿Cuál es el problema?** Si hay más variables que muestras, es una memorización.
*   **¿Qué tiene que pasar?** El sistema lee la matriz estructural, cuenta la cantidad de parámetros optimizables ($N_p$) y verifica que `Total_Trades / N_p > MIN_TRADES_PER_PARAM`.
*   **¿Cómo sé que está hecho?**
    - [ ] Una estrategia de 20 parámetros con 50 trades arroja un error "INSUFFICIENT_DEGREES_OF_FREEDOM".
    - [ ] Aparece en los logs el contador "Parameters Found: 14".

### **TTR-002: Aplicación del Complexity Penalty Score**
*   **¿Cuál es el problema?** Entre dos estrategias con el mismo PnL, la que tenga menos indicadores debe ganar siempre.
*   **¿Qué tiene que pasar?** Se aplica la fórmula de penalización: $Fitness\_Final = Fitness\_Base \times (1 - Penalty)$.
*   **¿Cómo sé que está hecho?**
    - [ ] Al comparar un algoritmo sencillo de Cruce EMA y una malla de 12 indicadores (ambos con el mismo PnL), el sencillo obtiene mayor score final.

### **TTR-003: Conteo de Grados de Libertad Multi-Dominio (ADR-0108)**
*   **¿Cuál es el problema?** Un Manifest puede tener simultáneamente un Genoma de Señal congelado y un Genoma de Riesgo y Gestión de Posición (o de Régimen y Filtro de Entorno) activo. Si el conteo de parámetros solo considera el genoma activo en esta corrida, la complejidad total real del Manifest queda subestimada y la penalización puede evadirse desplazando complejidad hacia el genoma congelado.
*   **¿Qué tiene que pasar?** El sistema debe sumar la cardinalidad de parámetros de **todos** los genomas de dominio presentes en el Manifest (activos y congelados) contra el mismo `Total_Trades`, antes de aplicar `MIN_TRADES_PER_PARAM`.
*   **¿Cómo sé que está hecho?**
    - [ ] Un Manifest con un Genoma de Señal de 8 parámetros y un Genoma de Riesgo y Gestión de 6 parámetros reporta `N_p = 14`, no 6 ni 8 por separado.
    - [ ] El log de "Parameters Found" refleja el total agregado, con desglose por dominio disponible para auditoría.
*   **¿Qué no puede pasar?** No se puede aplicar `MIN_TRADES_PER_PARAM` usando solo el conteo del dominio activo cuando el Manifest contiene más de un genoma de dominio.

### **TTR-004: Denominador de Cartera para el Genoma de Portafolio y Correlación (ADR-0111)**
*   **¿Cuál es el problema?** `Total_Trades / N_p` no es una medida significativa para el Genoma de Portafolio y Correlación, cuyos Genes de Condición Cruzada operan sobre el número de miembros de la cartera, no sobre el número de operaciones de una estrategia individual.
*   **¿Qué tiene que pasar?** Cuando el Dominio de Portafolio y Correlación está activo, el sistema calcula un segundo ratio independiente: `Miembros_de_Cartera / N_p_portafolio`, comparado contra `MIN_MEMBERS_PER_PORTFOLIO_PARAM`, adicional al ratio estándar `Total_Trades / N_p` aplicado a cada Manifest miembro individualmente.
*   **¿Cómo sé que está hecho?**
    - [ ] Una configuración de cartera de 5 miembros con un Genoma de Portafolio y Correlación de 8 parámetros arroja un error "INSUFFICIENT_PORTFOLIO_DEGREES_OF_FREEDOM" si `MIN_MEMBERS_PER_PORTFOLIO_PARAM` no se cumple.
    - [ ] El reporte distingue el ratio de cartera del ratio individual de cada miembro.
*   **¿Qué no puede pasar?** No se puede aprobar un Genoma de Portafolio y Correlación basándose únicamente en los ratios individuales de sus miembros, ignorando el ratio de cartera.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`, `indicator_state_hash`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
- **Registro de Dominios Genómicos (ADR-0108):** este motor es el único punto de cómputo de grados de libertad del sistema; toda nueva instancia de dominio admitida al Registro debe declarar su contribución de parámetros aquí, sin motores de penalización paralelos.
