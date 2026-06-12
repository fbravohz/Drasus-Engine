# Executable Container — Contrato Unificado Strategy/Portfolio

**Carpeta:** `./features/executable-container/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-09
**Decisión Arquitectónica Asociada:** ADR-0009 (Interfaz Unificada Strategy-Portfolio)

---

## ¿Qué es?

El Executable Container es un contrato técnico (Interface o Abstract Base Class) que estandariza cómo viajan los datos de una Estrategia o un Portafolio a través del pipeline.

**Problema:** Si Estrategia y Portafolio tienen estructuras distintas, módulos como `validate` o `incubate` tendrían que duplicar su lógica o tener muchos `if/else`.

**Solución:** Ambos implementan los mismos campos obligatorios (`config`, `rules`, `test_results`, `live_results`). El pipeline solo ve un "Executable" y lo procesa sin conocer su tipo interno.

**Resultado observable:** Un solo código de backtesting (`validate`) puede procesar una estrategia individual o un portafolio completo de forma intercambiable.

---

## Contrato de Datos (Frozen Object)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `uuid` | UUID | Identificador único e inmutable del contenedor. |
| `type` | Enum | `STRATEGY` o `PORTFOLIO`. |
| `config` | Dict | Parámetros específicos (lógica de trading o estructura de pesos). |
| `rules` | List[Rule] | Hard Limits y Soft Alerts aplicables (ADR-0010). |
| `test_results` | History | Historial de auditoría de pruebas pasadas (ADR-0005). |
| `test_analysis` | Analysis | Veredicto final del último audit (Sharpe, Drawdown, etc.). |
| `live_results` | P&L Store | Estado actual de ejecución en tiempo real o papel. |

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Tipo | Descripción |
|-----------|---------|------|-------------|
| `validation_strict` | `true` | bool | Si true, rechaza contenedores con campos nulos o tipos incorrectos. |
| `serialization_format` | `"json"` | string | Formato de persistencia en la tabla de jobs. |

---

## Restricciones

- **NUNCA un módulo accede a campos privados de Strategy/Portfolio.** Solo usa la interfaz pública del Container.
- **NUNCA un Container cambia de tipo.** `STRATEGY` nunca se convierte en `PORTFOLIO`.
- **NUNCA se pierde la trazabilidad.** Cada snapshot del container guarda el ID de la versión (ADR-0005).

---

## Tareas (TTRs)

### **TTR-EXEC-CONTAINER-001: Definir Interfaz ExecutableContainer**
- **Acción:** Crear la clase base con los 7 campos obligatorios usando validación estricta (esquemas Serde).
- **Invariante:** El objeto debe ser serializable a JSON para ser guardado en la tabla de `jobs`.

### **TTR-EXEC-CONTAINER-002: Implementar Mapeadores de Dominio**
- **Acción:** Implementar convertidores que tomen un `Strategy` o `Portfolio` de la base de datos y produzcan un `ExecutableContainer` puro para el Core.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Los contenedores de ejecución residen en la base de datos local encryptada.
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda cápsula de ejecución registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del contenedor |
| | `created_at` | Timestamp de creación |
| | `audit_hash` | Hash de integridad del contenedor |
| | `audit_chain_hash` | Hash del linaje de despliegue |
| **II. Soberanía** | `owner_id` | Autor responsable |
| | `manifest_id` | ID del contrato de diseño |
| | `access_token_id` | Token de autorización |
| **III. Pesos/Arquitectura** | `logic_hash` | Bytecode/Config fingerprint |
| | `indicator_state_hash` | Snapshot de métricas IS/OOS |
| | `version_node_id` | ID de la versión en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del orquestador de ejecución |


---

## Referencias
- `ADR.md` → ADR-0009: Interfaz Unificada
- `ADR.md` → ADR-0005: Versioning DAG
