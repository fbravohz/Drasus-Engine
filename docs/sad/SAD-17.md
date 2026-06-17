## 17. Gobernanza Operacional (Protocolos de Salud)

Para garantizar la integridad a largo plazo del monolito modular, se establecen los siguientes protocolos de mantenimiento obligatorio:

### 17.1 Protocolo de Propagación de Contratos (Interface Drift)
Ante cualquier cambio en una **Frontera Pública** o **Schema** de un módulo:
1.  Identificar todos los módulos consumidores (Clientes).
2.  Actualizar los TTRs de integración en los Clientes para reflejar el nuevo contrato.
3.  Validar que el "Evidence Trail" (ADR-0015) no se haya roto por el cambio de esquema.

### 17.2 Protocolo de Cierre de Bucle (Feedback Harvest)
Al añadir o modificar cualquier métrica técnica o de negocio en una Feature:
1.  Evaluar utilidad para el Control de Calidad Estadístico (MOD-07).
2.  Definir el punto de emisión en el **Rastro de Evidencia** de la feature.
3.  Actualizar la especificación de `feedback.md` para integrar la nueva fuente de aprendizaje.

### 17.3 Protocolo de Soberanía de Datos (Cross-Module Shield)
Si un módulo requiere datos persistidos por otro módulo:
1.  **PROHIBIDO** el acceso a la DB ajena.
2.  Crear un Puerto de consulta en la `public_interface.rs` del módulo dueño.
3.  El módulo consultor debe tratar el dato como inmutable y conforme al Contrato Global (ADR-0020 V2).

### 17.4 Protocolo de Neutralización del Masterplan (Legacy Extraction)
Al extraer requisitos de documentos legacy (como el Masterplan):
1.  Mapear a la Feature correspondiente o crear una nueva si no existe.
2.  Integrar los TTRs bajo la regla de **Evolución Incremental (ADR-0014)**.
3.  Ejecutar **Inundación de Fundaciones (ADR-0020 V2)** sobre el nuevo requerimiento de inmediato.

### 17.5 Protocolo de Preservación de Performance (SLA Guard)
Toda feature que impacte en el "Hot Path" (Ingest, Generate, Validate) debe ser auditada contra el criterio competitivo relativo (más rápido que MT5/SQX/QuantConnect en igual hardware, ADR-0114; sin KPI absoluto):
1.  **Vectorización Obligatoria:** Uso de Polars/Arrow para manipulación masiva de datos.
2.  **Native Compliance:** Cualquier loop secuencial debe ser implementado en Rust nativo optimizado.
3.  **IO Inundation:** Los campos inyectados por ADR-0020 V2 deben escribirse mediante transacciones batch en SQLite WAL.

### 17.6 Protocolo de Madurez (Transition Audit)
Al mover una Feature de estado `Especificación` a `Implementación`:
1.  Auditar cumplimiento de Gobernanza (ADR-0016, 0017, ADR-0020 V2).
2.  Verificar que los TTRs no sean ambiguos y tengan criterios de éxito técnicos.
3.  Asegurar que el orquestador del módulo posee los Puertos necesarios para la nueva lógica.

### 17.7 Protocolo de Auto-Evolución del Skill (Meta-Governance)
El conocimiento arquitectónico generado en sesiones de alta densidad debe ser "decriptado" en instrucciones técnicas para el agente:
1.  **Sync-Trigger:** Todo nuevo patrón aprobado en SAD/ADR debe evaluarse para su inclusión en el SKILL del rol correspondiente (`.claude/skills/<rol>/SKILL.md`, ej. `.claude/skills/architect/SKILL.md`).
2.  **Cierre de Brecha Cognitiva:** Si el agente requiere aclaración sobre un estándar >2 veces, se debe formalizar una sección en el workflow para evitar la recurrencia.
3.  **Refactorización de Skill:** Las instrucciones del agente se consideran "Código Vivo" y deben ser refactorizadas para eliminar ambigüedad tras cada hito arquitectónico.

### 17.8 Protocolo de Integridad Cruzada (Cross-Document Integrity - CODI)
Ningún documento es una isla. Todo cambio técnico significativo conlleva una revisión de impacto transversal:
1.  **Análisis de Sprint:** Ante un cambio en una Feature, auditar: SAD (Topología), ADR (Decisiones), TEMPLATES (Estándares) y Workflow (Operación).
2.  **Sincronización Atómica:** El cambio no se considera "commiteado" hasta que todos los mirrors y referencias cruzadas han sido actualizados.
3.  **Trazabilidad de Impacto:** Mantener las dependencias explícitas para facilitar la identificación del radio de acción de cada cambio.

### 17.9 Protocolo de Inundación Institucional (Audit Readiness)
Al crear o refactorizar cualquier entidad de persistencia (Tabla, Archivo Parquet, Evento):
1.  **Inundación Obligatoria (selectiva por perfil):** Inyectar el **grupo I (Identidad & Integridad)** de forma universal en toda entidad, y el resto de los **25 campos del contrato lógico** de forma **selectiva según el Perfil Técnico** (A. Datos/Ingest, B. IA/R&D, C. Ops/Hot-Path, D. Ops/Auditoría), conforme a la tabla canónica de Filtro de Relevancia definida en [ADR-0020 V2](../adr/ADR-0020.md). El contrato es un vocabulario lógico obligatorio, no 25 columnas calcadas en cada tabla.

    > **Ejemplo concreto (dos capas que NO deben confundirse):** la tabla `foundation_master_fields` (migración 0001) es el **catálogo de referencia** con las 25 columnas — existe UNA sola vez en todo el sistema, no se replica. Las tablas propias de cada módulo/feature (ADR-0003: cada módulo es dueño de sus tablas) NUNCA tienen esas 25 columnas; tienen sus columnas de dominio + el Grupo I completo (6 columnas, universal) + solo los campos concretos de su Perfil Técnico. Ej: la tabla de `adaptive-volume-indicators` (Perfil B / IA-R&D) lleva sus valores de indicador + Grupo I + (`owner_id`, `institutional_tag`, `manifest_id` de II) + (`logic_hash`, `data_snapshot_id`, `indicator_state_hash`, `version_node_id` de III) + (`node_id`, `process_id`, `execution_latency_ms` de IV) — nada de Grupo V, porque su perfil no lo cubre.

2.  **Hooks Forenses:** Definir el rastro de evidencia específico (latencias, estados internos) para alimentar el módulo de `feedback`.
3.  **Soberanía Multi-tenant:** Asegurar que `institutional_tag` y `owner_id` están correctamente mapeados en la capa de interacción (Shell).

### Resumen Visual
```
MOD-01 (ingest)
    ↓
MOD-02 (generate)
    ↓
MOD-03 (validate)
    ↓
MOD-04 (incubate)
    ↓
MOD-05 (manage)
    ↓
MOD-06 (execute)
    ↓
MOD-07 (feedback)
    ↓
MOD-08 (withdraw)
    │
    └─► [Aprendizaje para MOD-02]
```

---

