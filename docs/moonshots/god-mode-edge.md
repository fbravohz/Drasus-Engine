# God Mode — Edge Deployment

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0033 (Arquitectura de Despliegue Trimodal), ADR-0016 (Local-First Processing)

---

## ¿Qué es?

Permite empaquetar de forma automatizada y con un solo clic el Grafo de Lógica visual (Strategy AST) aprobado en un contenedor Docker Headless ultraliviano. Este contenedor incluye la versión mínima optimizada de NautilusTrader y las dependencias numéricas requeridas, desplegándose directamente en servidores virtuales de baja latencia física hacia los exchanges (ej. AWS ECS / Fargate o VPS dedicados en la nube). Esto permite la ejecución persistente de la estrategia 24/7 sin requerir que la interfaz de escritorio local del usuario permanezca encendida.

---

## Comportamientos Observables

- [ ] **Empaquetado Headless Automático:** Al presionar "Desplegar en la Nube", el backend Rust genera un sub-árbol AST optimizado y compila una imagen Docker minimalista a partir de plantillas pre-construidas.
- [ ] **Despliegue Remoto 1-Clic:** El sistema interactúa con las credenciales de AWS/VPS configuradas por el usuario para subir la imagen y levantar el contenedor en el exchange destino.
- [ ] **Canal gRPC de Telemetría:** El contenedor remoto expone un endpoint gRPC seguro bajo demanda. La UI local en Flutter se conecta a este canal para monitorear el rendimiento en vivo, posiciones y telemetría de ejecución del VPS.
- [ ] **Persistencia Local Dual:** El contenedor remoto guarda logs y trades localmente en su SQLite WAL y los sincroniza de forma asíncrona hacia la base de datos de la máquina local del usuario.

---

## Restricciones

- **OBLIGATORIO:** Mantener el principio de Soberanía (ADR-0016); las credenciales de despliegue y claves API de exchanges se almacenan de forma local y cifrada, nunca en servidores centralizados de Drasus Engine.
- **NUNCA** permitir que la pérdida de conexión a internet de la máquina local interfiera con la toma de decisiones y el motor de ejecución del contenedor Fargate remoto (el contenedor es 100% autónomo).
- **FIJO:** El contenedor Docker debe compilarse sin entorno gráfico (Headless) y con un consumo de recursos en reposo inferior a 256MB de RAM.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| SYNC_INTERVAL_SEC | 10 | 1 - 60 | Intervalo de sincronización de datos de ejecución VPS hacia base de datos local | CONFIG |
| RETRY_CONNECTION_ATTEMPTS | 5 | 1 - 20 | Intentos de reconexión de telemetría gRPC antes de disparar alerta silenciosa | CONFIG |
| SHADOW_MODE | false | true / false | Ejecutar el contenedor remoto en modo shadow (operaciones simuladas en paralelo) | CONFIG |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Generación de especificaciones de configuración de contenedor (Dockerfile, docker-compose) y validación de esquemas de despliegue.
- **Shell (Infraestructura):** Integración con Docker CLI y APIs de proveedores en la nube (AWS SDK) para control y despliegue del contenedor.

---

## Ciclo de Vida de la Feature — Edge Deployment

### Entrada
- Grafo de estrategia validado y firmado (JSON AST).
- Credenciales locales cifradas de proveedor en la nube.
- Variables de entorno del exchange (claves API de trading).

### Proceso
- Rust compila el AST y genera el manifiesto del contenedor.
- Se ejecuta la llamada para levantar la instancia Fargate/ECS de forma remota.
- El Daemon del VPS inicializa NautilusTrader y se acopla al flujo del exchange.

### Salida
- Identificador de proceso de ejecución en la nube (`process_id`).
- Conexión gRPC activa para telemetría remota.

---

## Tareas (TTRs)

### **TTR-001: Compilador de Imágenes Docker Headless**
*   **¿Cuál es el problema?** Configurar manualmente servidores VPS con dependencias exactas de Rust, NautilusTrader y librerías numéricas es complejo y propenso a errores humanos.
*   **¿Qué tiene que pasar?** Crear una tarea en Rust que ensamble dinámicamente un Dockerfile a partir de los requerimientos de la estrategia (indicadores utilizados). Compila el AST visual de la estrategia en un archivo de configuración autocontenido que el contenedor levanta en el arranque.
*   **¿Cómo sé que está hecho?**
    - [ ] Al presionar el botón de compilación, se genera una imagen Docker local válida en menos de 60 segundos.
    - [ ] El tamaño de la imagen final headless es menor a 400MB.
*   **¿Qué no puede pasar?**
    - No se deben incluir archivos temporales de compilación o datos históricos del backtesting en la imagen final.

### **TTR-002: Orquestador gRPC de Telemetría VPS**
*   **¿Cuál es el problema?** El usuario necesita vigilar la operativa en vivo ejecutándose en la nube sin exponer puertos inseguros ni saturar el ancho de banda.
*   **¿Qué tiene que pasar?** Implementar un canal de comunicación gRPC seguro (TLS mutual) en el binario headless. La interfaz de usuario de Flutter local se suscribe a este canal para recibir actualizaciones del estado de la cuenta, margen y ejecuciones de órdenes en vivo enviadas por el VPS de forma eficiente.
*   **¿Cómo sé que está hecho?**
    - [ ] La UI local de Flutter muestra la curva de capital del VPS remoto actualizándose segundo a segundo.
    - [ ] Al cerrar la aplicación local, el VPS sigue operando de forma autónoma; al abrir la app de nuevo, se reconecta y recupera el historial de la sesión sin fricciones.
*   **¿Qué no puede pasar?**
    - La telemetría no debe exponer claves de API o datos sensibles sin cifrado de transporte robusto.

---

## Gobernanza y Estándares (ADR-0020)

### Perfil Ops / Hot-Path
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de despliegue Edge |
| | `created_at` | Timestamp de levantamiento de la instancia |
| | `audit_hash` | Hash de la imagen Docker desplegada |
| **II. Soberanía** | `owner_id` | Identificador de firma del propietario |
| | `institutional_tag` | Etiqueta de entorno (Live / Demo VPS) |
| **IV. Hardware** | `node_id` | Identificador de instancia de máquina virtual remota |
| | `process_id` | AWS Task ARN o ID de contenedor de servicio |
| | `execution_latency_ms` | Latencia de red de ida y vuelta al gateway del exchange |
