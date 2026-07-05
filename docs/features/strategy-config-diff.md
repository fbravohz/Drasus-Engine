# Strategy Config Diff

**Carpeta:** `./features/strategy-config-diff/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión), ADR-0005 (Strategy-Portfolio Git-Like Versioning con DAG)

---

## ¿Qué es esta feature?

El `Strategy Config Diff` es una herramienta visual en la interfaz de usuario que permite comparar la configuración de parámetros activa de un proyecto de estrategia (`Current Project Config`) contra la configuración guardada del último test de robustez exitoso (`Strategy Last Test Config`). Ayuda al operador a identificar divergencias críticas en categorías como Datos, Money Management (MM) y Advanced Trade Management (ATM), determinando si es necesario forzar un retest de la estrategia para asegurar su robustez.

---

## Comportamientos Observables

- [ ] El usuario abre la vista de diferencias de una estrategia y visualiza una comparación lado a lado de sus configuraciones.
- [ ] La UI resalta visualmente los parámetros modificados, eliminados o añadidos:
  - **Fondo Rojo:** Parámetro eliminado o divergente en la versión histórica.
  - **Fondo Verde:** Parámetro nuevo o modificado en la configuración actual.
- [ ] El sistema muestra un indicador de estado:
  - **Sincronizado (Verde):** No hay cambios en parámetros que afecten la lógica operativa.
  - **Desincronizado (Amarillo/Rojo):** Existen desviaciones en variables críticas, activando una advertencia de "Retest Recomendado".
- [ ] Categoriza los parámetros por pestañas: `Data Ingest`, `Money Management`, `ATM Rules`.

---

## Restricciones

- **NUNCA** permitir la promoción de una estrategia a incubación o producción si el `Strategy Config Diff` detecta desviaciones críticas en parámetros lógicos sin haber ejecutado una re-validación previa.
- **NUNCA** hardcodear las claves de comparación; la estructura se valida dinámicamente mediante la lectura recursiva de los esquemas Serde definidos en el core.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| RE_VALIDATION_MANDATORY | true | true/false | Determina si las diferencias obligan a invalidar la aprobación previa | [FIJO] |
| IGNORE_METADATA_DIFFS | true | true/false | Ignorar diferencias en campos descriptivos (ej. descripción, notas, nombres) | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmo de comparación recursiva (diff) de JSON AST de estrategias y detección de gravedad de cambios.
- **Shell (Infraestructura):** Consultor de histórico de versiones SQLite de estrategias y configuraciones activas en el espacio de trabajo.
- **Frontera Pública:** Interfaz de comparación que recibe dos identificadores de versión y retorna el mapa estructurado de diferencias.

---

## Ciclo de Vida de la Feature — Strategy Config Diff

### Entrada
- JSON AST de la configuración activa (`Current Project Config`).
- JSON AST de la última configuración validada (`Strategy Last Test Config`).

### Proceso
- Analiza de forma recursiva ambos árboles de parámetros.
- Identifica claves modificadas, agregadas o eliminadas.
- Clasifica la severidad del cambio según su categoría técnica (ej: cambiar el stop loss es crítico; cambiar las etiquetas es informativo).

### Salida
- Objeto JSON con el mapa de diferencias formateado para la UI.
- Indicador de necesidad de retest: `SIN_CAMBIOS` / `CAMBIOS_SOPORTADOS` / `RETEST_MANDATORIO`.

---

## Tareas (TTRs)

### **TTR-001: Motor de Comparación AST (Rust)**
*   **¿Cuál es el problema?** Identificar diferencias estructurales profundas en grandes JSON AST puede volverse propenso a errores y lento en la interfaz si se hace a mano.
*   **¿Qué tiene que pasar?** Rust calcula recursivamente las diferencias y asigna categorías de severidad a los campos modificados.
*   **¿Cómo sé que está hecho?**
    - [ ] Rust genera el diff de dos AST y marca la variable `RETEST_MANDATORIO` como verdadera al alterar parámetros de indicadores.

### **TTR-002: Visor de Diferencias Lado a Lado (Flutter)**
*   **¿Cuál es el problema?** El operador requiere comprender qué parámetros operativos han cambiado exactamente en un formato legible.
*   **¿Qué tiene que pasar?** Renderizar una tabla lado a lado con colores de fondo contrastantes para resaltar las líneas y valores divergentes.
*   **¿Cómo sé que está hecho?**
    - [ ] La UI muestra con claridad las diferencias en las variables operativas y activa el botón de "Lanzar Retest".

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil IA / R&D. Registra `version_node_id`, `parent_id`, `logic_hash`.
- **Rastro de Evidencia:** Emite alertas de desincronización de configuraciones al módulo de `feedback`.

---

## Dependencias
- **Depende de:** `/features/strategy-versioning.md`, `/features/ast-compiler.md`
- **Bloquea:** `/modules/validate.md`
