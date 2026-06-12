# Fragility Gradient Auditor (System Parameter Permutation)

**Carpeta:** `./features/fragility-gradient-auditor/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020 V2

---

## ¿Qué es?

La Auditoría de Gradiente de Fragilidad Descendente es la evolución (New Era) del clásico análisis de varianza/mediana. 

**Problema que resuelve:** En lugar de mostrar histogramas donde una ligera desviación cambia todo el panorama (graficados en visores 3D estáticos), este sistema cruza derivadas segundas matriciales. Si descubre que cambiar *un milímetro* en un input numérico (ej: Stop Loss de 20 a 21) compromete más de un 40% del PnL global de la estrategia, entonces "condena íntegramente a todo el genoma", desarmando por completo los falsos encierros optimizadores.

---

## Comportamientos Observables

- [ ] **[OLD-SCHOOL] Evaluador Básico:** Muta cada parámetro de forma aislada para generar histogramas de desplazamiento rojo/verde. Compara el `Valor Original` vs `Valor de Mediana`.
- [ ] **[NEW-ERA] Auditoría Gradiente de Fragilidad Descendente:** Cruza derivadas segundas matriciales para detectar inestabilidades milimétricas.
- [ ] Si `Delta_PnL > 40%` ante un `Delta_Input = 1%`, el genoma es marcado como "Estructuralmente Frágil" y erradicado automáticamente.
- [ ] Erradica las distorsiones visuales de los gráficos 3D estáticos; todo se realiza en base a tensores (matrices) de forma invisible.

---

## Restricciones

- **FIJO:** La erradicación es absoluta y no apelable. Una inestabilidad superior al umbral configurado condena el genoma íntegro, sin importar cuán perfecto era el resultado principal.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MAX_ALLOWED_FRAGILITY | 40% | 20% - 60% | Caída máxima permitida en el Alpha global ante una alteración mínima del input. | CONFIG |
| PERMUTATION_MICRO_STEP | 1% | 0.5% - 5% | Tamaño del paso milimétrico de alteración en el gradiente. | CONFIG |

---

## Ciclo de Vida de la Feature — Fragility Gradient Auditor

### Entrada
- Espacio paramétrico nominal de la estrategia ganadora.
- Funcional de PnL de simulación base.

### Proceso
- Desplaza iterativamente cada parámetro un `PERMUTATION_MICRO_STEP`.
- Deriva el cambio en el rendimiento respecto al cambio en el parámetro ($\Delta PnL / \Delta Param$).
- Detecta sensibilidades extremas (acantilados de optimización).

### Salida
- `gradient_fragility_matrix`.
- `fragility_condemnation_verdict` (CONDEMNED / SECURE).

### Contextos de Uso
**Contexto 1: Destrucción de Encierros Optimizadores (Validate)**
- Actúa como la última defensa matemática profunda que aniquila estrategias sobreajustadas en dimensiones muy específicas.

---

## Tareas (TTRs)

### **TTR-001: Calculador de Gradiente Milimétrico y Varianza (OLD-SCHOOL)**
*   **¿Cuál es el problema?** Necesitamos visualizar si un componente produjo rendimientos espurios.
*   **¿Qué tiene que pasar?** El sistema muta parámetros, genera histogramas y compara el valor original contra la mediana de resultados.
*   **¿Cómo sé que está hecho?**
    - [ ] El sistema genera histogramas de desplazamiento rojo/verde.
    - [ ] Se reporta la desviación respecto a la mediana paramétrica.

### **TTR-002: Auditoría Gradiente Descendente (NEW-ERA)**
*   **¿Cuál es el problema?** Cambiar un parámetro de 20 a 21 no debería causar la quiebra.
*   **¿Qué tiene que pasar?** El sistema calcula derivadas segundas. Si la inestabilidad destruye >40% del Alpha, condena la matriz entera.
*   **¿Cómo sé que está hecho?**
    - [ ] Log inmutable emite: "Estrategia condenada: Inestabilidad milimétrica destruye >40% Alpha."

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `audit_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
