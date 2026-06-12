# Setup de Infraestructura

**Carpeta:** `./features/setup-infraestructura/`
**Estado:** Pendiente
**Última actualización:** 2026-04-09
**Decisiones Arquitectónicas:** ADR-0003, ADR-0006, ADR-0007, ADR-0011, ADR-0012, ADR-0013

---

## ¿Qué es?

Antes de escribir cualquier módulo del sistema, necesitamos preparar el terreno: la estructura de carpetas, la base de datos, el sistema de logs, y las herramientas de prueba. Sin esto, nada más puede arrancar.

Es como construir la cimentación de una casa antes de levantar las paredes.

---

## Comportamientos Observables

- [ ] Puedo clonar el repositorio y con un solo comando instalar todo
- [ ] Puedo correr las pruebas y ver resultados aunque no haya código de negocio aún
- [ ] La base de datos arranca y puedo ver su versión actual
- [ ] Los logs aparecen en formato estructurado (fácil de leer por máquinas)
- [ ] El linter no reporta errores en código limpio
- [ ] **[NUEVO]** El Job Executor recupera tareas pendientes (`RUNNING/QUEUED`) tras un reinicio inesperado.
- [ ] **[NUEVO]** El Feature Router valida que todas las dependencias entre módulos se cumplan antes de iniciar.

---

## Restricciones

- El proyecto debe correr con Rust o superior
- La base de datos es SQLite local (sin servidor externo)
- Pruebas deben poder correr sin internet ni broker conectado
- Cada archivo de código tiene un máximo de 400 líneas (se parte si excede)

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| DATABASE_URL | sqlite local | Ruta donde se guarda la base de datos |
| LOG_LEVEL | INFO | Nivel de detalle de bitácora (DEBUG..ERROR) |
| MAX_CONCURRENT_JOBS | 5 | Máximo de tareas paralelas en el Job Executor |
| BUSY_TIMEOUT | 5000 | Tiempo de espera en ms (SQLite WAL) |
| BROKER_API_KEY | vacío | Clave para conectar al broker (se llena después) |

---

---

## Tareas (TTRs)

### **TTR-001: Arquitectura de Soporte (Folders & Env)**
*   **Descripción:** Prepara la estructura de cimentación y aislamiento de dependencias.
*   **Reglas de Negocio:**
    * El entorno DEBE configurarse vía `Environment Variables` (ADR-0012).
    * Ninguna carpeta de módulo puede tener dependencias circulares.
*   **Entrada:** `ProjectSpec`, `FeatureRegistry`.
*   **Salida:** `DirStructure`, `VirtualEnv`.
*   **Precondición:** Rust verificado.
*   **Postcondición:** Linter configurado y libre de errores de importación.

### **TTR-002: Motor de Migraciones e Inundación de Esquemas (ADR-0006/0020)**
*   **Descripción:** Sistema de persistencia evolutiva con hooks de auditoría institucional.
*   **Reglas de Negocio:**
    * Toda tabla DEBE incluir los 25 campos fundacionales de ADR-0020 V2.
    * Migraciones irreversibles (data-loss) requieren confirmación de `force` flag.
*   **Entrada:** `MigrationsDir`, `SQLXSchemaDefinitions`.
*   **Salida:** `DatabaseSchema (Ready)`.
*   **Precondición:** SQLite configurado con modo WAL (ADR-0013).
*   **Postcondición:** `audit_hash` inicial de la base de datos generado.

### **TTR-003: Orquestador de Tareas y Auto-Recovery (ADR-0011)**
*   **Descripción:** Implementar el motor de asincronía paralelo con persistencia de estado de jobs.
*   **Reglas de Negocio:**
    * Los jobs `RUNNING` interrumpidos deben marcarse como `RECOVERY_PENDING` al reiniciar.
    * Límite estricto de `MAX_CONCURRENT_JOBS` para evitar saturación de CPU.
*   **Entrada:** `JobDefinition`, `priority`.
*   **Salida:** `JobHandle`, `AuditTrail`.
*   **Precondición:** Sistema de persistencia (TTR-002) activo.
*   **Postcondición:** Registro del job en `audit-log` con `process_id`.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Las tablas base del sistema (`system_settings`, `jobs`, `logs`) registran el set universal de **25 campos mandatorios** (ver ADR-0020 V2 V2).
    - Metadatos de hardware y cimiento: `node_id`, `logic_hash`, `audit_chain_hash`, `event_sequence_id`.
    - Soberanía central: `owner_id`, `manifest_id`.

- **Decisión Arquitectónica Asociada:**
    - ADR-0003: Estructura Modular.
    - ADR-0011: Operaciones Asincrónicas.
    - ADR-0013: Stack Tecnológico (SQLite WAL).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- Ninguna. Es la raíz de la infraestructura.

**Consumido por:**
- **Todos los Módulos:** Para servicios de persistencia, concurrencia y auditoría.
