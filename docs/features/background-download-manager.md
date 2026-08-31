# Background Download Manager

**Carpeta:** `./features/background-download-manager/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0011 (Async Job Pattern)

## ¿Qué es?

Es el orquestador visual y técnico que permite gestionar las descargas de datos históricos en segundo plano sin bloquear la interfaz de usuario. Proporciona la infraestructura para que el usuario pueda ver el progreso real (bytes, tiempo estimado) de procesos masivos de ingesta.

**Problema:** Las descargas masivas pueden tardar minutos y el usuario no sabe si el sistema se ha colgado o sigue trabajando.
**Solución:** Un gestor de jobs asíncronos que emite señales de progreso constantes hacia la UI.

---

## Narrativa de Producto

**Capacidad:** `CAP-01` — Datos en los que se puede confiar

**Historia de usuario:** Como usuario, quiero seguir trabajando mientras se descargan años de historia, y ver en todo momento cuánto lleva y cuánto falta, para no tener que elegir entre esperar mirando la pantalla o no saber si el programa se colgó.

**El problema (técnico):** una descarga masiva tarda minutos u horas. Si bloquea la interfaz, el usuario no puede hacer nada más. Si corre sin reportar, el usuario no distingue entre "sigue trabajando" y "se colgó" — y la respuesta humana ante esa duda es cancelar y reintentar, que desperdicia todo lo avanzado.

**La solución (técnica):** las descargas corren como trabajos en segundo plano que emiten progreso real —volumen transferido y tiempo estimado, no una animación decorativa— mientras la interfaz sigue respondiendo. Un trabajo interrumpido por un cierre o un corte se retoma donde iba en vez de empezar de cero.

**Explicado sin tecnicismos:** bajar diez años de historia tarda un rato largo. Dos formas de hacerlo mal: congelar el programa entero hasta que termine, o dejarlo trabajar sin decir nada.

La segunda parece inofensiva y es la peor. Cuando algo tarda y no da señales, uno asume que se trabó, cancela y vuelve a empezar — tirando a la basura todo lo que ya se había bajado.

Así que Drasus te enseña el avance real: cuánto lleva, cuánto falta, cuánto tiempo queda. Mientras tanto sigues usando el programa. Y si se cierra a media descarga, al volver **retoma donde iba**, no desde el principio.

## Comportamientos Observables

- [ ] El usuario inicia una descarga masiva y puede seguir navegando por otras secciones de la app.
- [ ] En la barra lateral o sección de "Download Center", se ve una lista de descargas activas con:
  - Nombre del activo (BTCUSDT).
  - Porcentaje de progreso (EJ: 45%).
  - Velocidad (EJ: 12 MB/s).
  - Tiempo estimado restante.
- [ ] El usuario puede pausar o cancelar una descarga desde la UI.

## Restricciones

- NUNCA se permiten más de N descargas simultáneas (configurable) para no saturar el disco o la red.
- NUNCA se pierde el progreso de una descarga si el backend se reinicia (debe poder reanudarse si el servidor lo soporta).
- Las señales de progreso deben emitirse con una frecuencia controlada (EJ: cada 500ms) para no saturar el gRPC/WebSocket.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MAX_SIMULTANEOUS_DOWNLOADS | 3 | 1 - 5 | Límite de jobs de descarga activos | CONFIG |
| PROGRESS_EMIT_INTERVAL_MS | 500 | 100 - 5000 | Frecuencia de actualización de la UI | CONFIG |
| AUTO_RESUME | True | True/False | Intentar reanudar descargas al arrancar | CONFIG |

## Ciclo de Vida de la Feature — Background Download Manager

### Entrada
- Job ID de descarga.
- Fuente (URL Bulk o API Delta).
- Destino local.

### Proceso
- Valida espacio en disco.
- Inicia el worker de descarga (Sidecar).
- Monitorea el stream de datos y calcula la velocidad.
- Publica eventos de "DownloadProgress" en el bus interno.

### Salida
- Archivo descargado y verificado (Checksum).
- Notificación de "Completado" en la UI.
- Registro en `job_history`.

### Contextos de Uso

**Contexto 1: Centro de Descargas (UI)**
- Visualización de jobs activos y control manual por el usuario.

**Contexto 2: Ingesta Automática (Auto-Sync)**
- Gestión silenciosa de descargas Delta para mantener el sistema actualizado.

## Tareas (TTRs)

### **TTR-001: Orquestador de Descargas Asíncronas**
- Implementa la lógica de gestión de cola de descargas y persistencia del estado del job en SQLite.

### **TTR-002: Emisor de Telemetría de Progreso**
- Implementa el cálculo de métricas en tiempo real (ETA, MB/s) y su envío vía gRPC/WebSocket.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (gestión de archivos locales).
- **Fidelidad (ADR-0017):** N/A (Feature de infraestructura).
## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada job de descarga registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del job |
| | `created_at` | Timestamp de inicio |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del job de descarga |
| | `audit_chain_hash` | Hash de integridad de la sesión |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | URL/Endpoint de la descarga (linaje de origen) |
| | `session_id` | Sesión global vinculada |
| **IV. Hardware** | `node_id` | ID del hardware físico receptor |
| | `process_id` | PID del manager de descargas |
