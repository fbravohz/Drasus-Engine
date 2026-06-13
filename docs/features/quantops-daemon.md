# QuantOps Daemonized Pipelines (Cron CI/CD)

**Carpeta:** `./features/quantops-daemon/`
**Estado:** En Diseño
**Última actualización:** 2026-04-28
**Decisión Arquitectónica Asociada:** ADR-0052 (QuantOps Daemonized Pipelines), ADR-0054 (Inter-Project Chaining & External Hooks)

---

## 1. ¿Qué es esta feature?

QuantOps Daemon es la evolución de la automatización manual de flujos de trabajo hacia una infraestructura de "Continuous Integration / Continuous Deployment" (CI/CD) completamente autónoma para estrategias de trading. Opera como un *daemon* asíncrono en servidores Linux (modo Headless) o local, ejecutando fábricas completas 24/7 sin involucramiento humano directo.

**Problema:** La investigación cuantitativa requiere ejecutar repetitivamente el mismo flujo de validación, generación y filtrado, lo que consume tiempo del analista. 
**Solución:** Múltiple perfiles de automatización (desde control manual vía GUI hasta Daemons Headless 24/7) que implementan un **Flujo Autónomo Múltiple**: `Generate → Metric Filtering → Robustness Retest (WFA/MC) → Approve → Export/Portfolio/Databank`.

## 2. Comportamientos Observables

- [ ] **Trabajo Programado (Cron):** El usuario programa tareas recurrentes (ej. "Generar y validar 10,000 estrategias cada domingo a las 02:00").
- [ ] **Encadenamiento de Proyectos:** Al completar con éxito el Pipeline A (ej. Forex), el sistema dispara automáticamente el Pipeline B (ej. Crypto) vía `on_complete_action: {type: "start_pipeline", target_id: UUID}`.
- [ ] **Conectores de Scripts Externos:** Un nodo suspende la ejecución del pipeline, exporta el dataset a un CSV/Parquet y llama un proceso externo (ejecutable nativo, binario de Rust, webhook) de Machine Learning. El pipeline espera la respuesta y reanuda.
- [ ] **Conectores de Notificación:** El sistema envía notificaciones vía SMTP o webhooks configurables al completar un pipeline o aprobar un Filtro de Calidad. Desactivado por defecto.
- [ ] **Niveles de Velocidad de Validación:** El usuario clasifica los pipelines de validación en 3 velocidades:
  - **Rápido:** Simulaciones What-If, Monte Carlo básico sobre trades.
  - **Lento:** Validación en mercados cruzados, Optimización Secuencial.
  - **Muy Lento:** Permutación exhaustiva de parámetros, WFM/WFO completo.

## 3. Restricciones

- NUNCA se ejecuta un Conector de Script Externo sin aislarlo del proceso principal (Sandboxing o suspensión asíncrona) para evitar bloqueos del GIL.
- Las notificaciones (Conectores de Notificación) deben estar configuradas en SQLite y ser OPT-IN (Soberanía Prioritaria).
- El Flujo Autónomo no sortea Filtros de Calidad; si una estrategia falla el Design Manifest, es archivada, nunca promovida automáticamente.

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| NOTIFICATIONS_ENABLED | False | True/False | Activa/Desactiva ganchos de notificación (SMTP/Webhook) | CONFIG |
| MAX_CHAIN_DEPTH | 5 | 1-20 | Límite de encadenamiento para evitar bucles infinitos (A->B->A) | CONFIG |
| EXTERNAL_SCRIPT_TIMEOUT | 3600 | 10-7200 | Tiempo máximo en segundos a esperar un script externo antes de fallar | CONFIG |

## 5. Estructura Interna (FCIS)

- **Core (Lógica Pura):** Parseo de expresiones cron, resolución topológica de dependencias en Encadenamiento de Proyectos, mapeo de Niveles de Velocidad a funciones de validación.
- **Shell (Infraestructura):** Daemon asíncrono de Rust (`tokio` o supervisor OS-level), gestor de subprocesos para Conectores Externos (`std::process::Command`), integraciones SMTP/HTTP (`reqwest`).
- **Frontera Pública:** Interfaz para registrar pipelines, adjuntar cron jobs, y registrar callbacks de completitud.

## 6. Ciclo de Vida de la Feature

### Entrada
- Definición topológica del Pipeline (Flujo Autónomo Múltiple).
- Reglas de cron (`Trabajo Programado`).
- Punteros a dependencias de Filtros de Calidad (`Design Manifest`).

### Proceso
- El Daemon despierta según el cron.
- Invoca la orquestación asíncrona de los módulos (`Generate` -> `Validate` -> `Incubate`).
- Suspende la ejecución si alcanza un Conector de Script Externo.
- Cruza los resultados con el Design Manifest (Filtro de Calidad).

### Salida
- Estrategias promovidas al Databank/Portfolio.
- Trigger de `Encadenamiento de Proyectos` a los siguientes UUIDs de pipelines.
- Alerta por `Conectores de Notificación`.

### Contextos de Uso
- **Contexto 1: Generación Periódica:** Mantenimiento de diversidad de portafolio sin intervención humana (Minería de fin de semana).
- **Contexto 2: Validación ML Externa:** Extracción temporal de datos vía Hooks para que el usuario corra Scikit-Learn y devuelva predictores inyectados al pipeline de Drasus Engine.

## 7. Tareas (TTRs)

### **TTR-001: Motor Daemon y Trabajo Programado**
- **Problema:** Necesitamos que el sistema pueda correr tareas (como minar estrategias) de forma programada sin la presencia de un humano ni UI abierta.
- **Qué tiene que pasar:** Un proceso asíncrono (Daemon) puede ejecutar un "Pipeline Job" en un horario específico (Cron) y ejecutar el ciclo de minería y validación hasta el Databank.
- **Cómo sé que está hecho:** 
  - [ ] Puedo programar un trabajo para "mañana a las 2AM".
  - [ ] A las 2AM el trabajo se inicia solo.
- **Restricción:** No debe bloquear la memoria ni la UI si está conectada.

### **TTR-002: Encadenamiento de Proyectos y Conectores Externos**
- **Problema:** Los pipelines son aislados y es difícil integrarlos con scripts de Rust externos del investigador o enlazar minería de divisas y cripto.
- **Qué tiene que pasar:** Al terminar un pipeline, el sistema lee `on_complete_action` y dispara el siguiente UUID. Además, soporta nodos de pausa para invocar binarios/scripts externos.
- **Cómo sé que está hecho:** 
  - [ ] Termina Pipeline A y automáticamente inicia Pipeline B.
  - [ ] Pipeline invoca un `mi_modelo` nativo, espera respuesta, y sigue con los resultados.

### **TTR-003: Niveles de Velocidad de Validación**
- **Problema:** Algunas validaciones toman minutos y otras días; el usuario necesita perfiles de velocidad para no perder tiempo en ideas tempranas.
- **Qué tiene que pasar:** Clasificación en 3 perfiles (Rápido, Lento, Muy Lento) que limitan la profundidad de Monte Carlo y WFO.

## 8. Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Daemon corre en el servidor de la máquina anfitriona (Linux Headless o Windows Background).
- **Inundación de Fundaciones (ADR-0020 V2):** 
   - **Perfil D (Ops / Auditoría):** daemon CI/CD de orquestación, no ruta crítica de latencia.
   - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
   - **II. Soberanía & Propiedad:** `owner_id` (quién configuró el cron), `institutional_tag`.
   - **IV. Infraestructura & Ops:** `node_id`, `process_id` (del daemon asíncrono), `session_id`.
- **Contrato de Persistencia:** Tabla `notification_configs` y `pipeline_schedules` integradas a la metadata del sistema.

## 9. Dependencias y Bloqueantes
- **Depende de:** `visual-dag-editor` (Para configurar los pipelines antes de delegarlos al daemon), `async-job-executor`.
- **Bloquea a:** Ninguno.

## 10. Regla de Soberanía Técnica
- El Daemon es un orquestador que consume las APIs públicas de `Generate`, `Validate` e `Incubate`. No implementa lógica de ML ni validación por su cuenta.
