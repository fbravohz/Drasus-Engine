# Binary Arrow Transport

**Carpeta:** `./features/binary-arrow-transport.md`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0013 (Stack Tecnológico - Apache Arrow)

## ¿Qué es esta feature?

El **Transporte Binario Arrow** permite la transmisión de alta velocidad de grandes conjuntos de datos (series temporales, curvas de equidad) entre el backend (Rust) y el frontend (Flutter/Arrow JS) a través de los canales de **FFI/gRPC**. Utiliza el formato de memoria de Apache Arrow para evitar la costosa serialización/deserialización a JSON, permitiendo el streaming de millones de registros con latencia mínima.

## Comportamientos Observables

- [ ] Envío de datos de mercado en vivo via gRPC/WebSocket (Puerto 8001).
- [ ] Transferencia de resultados de backtest masivos hacia la UI para visualización en el lienzo fractal (ZUI).
- [ ] **Throttling Visual:** Los datos se agrupan y emiten cada 100 milisegundos para evitar la saturación del tráfico y garantizar una visualización fluida (>10,000 puntos).
- [ ] Aplicación de **Downsampling** dinámico en el servidor antes de empaquetar en Arrow.
- [ ] Cero latencia perceptible al hacer zoom en gráficos de alta densidad.

## Restricciones

- **NUNCA** enviar datos crudos > 10,000 puntos sin aplicar downsampling previo.
- **OBLIGATORIO:** Las cabeceras de los mensajes gRPC/WebSocket deben indicar el esquema Arrow para que el cliente pueda reconstruir la tabla.
- El transporte debe ser agnóstico al protocolo (gRPC/WebSocket o HTTP asíncrono).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| MAX_FEATHER_SIZE | 50MB | 1MB - 500MB | Tamaño máximo de chunk de datos | CONFIG |
| DOWNSAMPLING_THRESHOLD | 10000 | 1000 - 50000 | Puntos a partir de los cuales se activa reducción | CONFIG |
| COMPRESSION_CODEC | ZSTD | LZ4, ZSTD, None | Algoritmo de compresión para el transporte | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmo de downsampling (ej: Largert-Triangle-Three-Buckets - LTTB) implementado en Rust SIMD/Rayon.
- **Shell (Infraestructura):** Serializadores de PyArrow y manejadores de gRPC/WebSocket de Orquestador Rust.

## Ciclo de Vida de la Feature — Binary Transport

### Entrada
- DataFrame de Polars o tabla de PyArrow.
- Parámetros de vista del usuario (rango temporal visible).

### Proceso
1. **Downsampling:** Filtra puntos críticos (O, H, L, C / Picos y Valles).
2. **Serialization:** Convierte a formato Feather/Arrow FFI/gRPC.
3. **Transmission:** Envía bytes crudos por el túnel gRPC/WebSocket.

### Salida
- Stream de bytes Arrow.

## Tareas (TTRs)

### **TTR-001: Motor de Downsampling LTTB Vectorizado**
* **¿Cuál es el problema?** Intentar visualizar 1M de puntos de datos crudos en el navegador bloquea el hilo de renderizado y satura el ancho de banda del túnel local.
* **¿Qué tiene que pasar?** Implementar el algoritmo LTTB (Largest-Triangle-Three-Buckets) en Rust SIMD/Rayon para reducir el dataset a una resolución visual óptima (ej: 1000 puntos) manteniendo la integridad de picos y valles.
* **¿Cómo sé que está hecho?**
    - [ ] El motor procesa 1M de puntos en < 5ms.
    - [ ] La curva reducida mantiene exactamente los mismos valores de Max Drawdown que la original.
* **¿Qué no puede pasar?** NUNCA enviar más de 10,000 puntos hacia la UI por mensaje individual.

### **TTR-002: Bridge Orquestador Rust ↔ Arrow JS (Binary Streaming)**
* **¿Cuál es el problema?** Los Canales FFI/gRPC de Flutter FFI están orientados a JSON (texto), lo que añade un overhead de serialización inaceptable para streaming de alta frecuencia.
* **¿Qué tiene que pasar?** Configurar el endpoint `/ws/live` para transmitir directamente buffers binarios de Apache Arrow con cabeceras de esquema autodescriptivas.
* **¿Cómo sé que está hecho?**
    - [ ] El intercambio de una tabla de 100K registros ocurre en < 50ms.
    - [ ] El frontend de Flutter recibe y decodifica el stream binario sin usar `JSON.parse`.
* **¿Qué no puede pasar?** PROHIBIDO el uso de Base64 para el transporte de datos Arrow.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada paquete de datos transmitido o registrado por esta feature debe portar el set maestro de 25 campos para auditoría forense:

| Campo | Tipo | Descripción |
| :--- | :--- | :--- |
| `id` | UUID | Identificador único del paquete/stream |
| `created_at` | INT64 | Timestamp de creación (nanosegundos) |
| `updated_at` | INT64 | Última modificación del buffer |
| `audit_chain_hash` | VARCHAR | Hash de la cadena de integridad |
| `owner_id` | UUID | Identificador del dueño del proceso |
| `institutional_tag` | VARCHAR | Etiqueta de cumplimiento (ADR-0020 V2) |
| `manifest_id` | UUID | Referencia al Design Manifest origen |
| `access_token_id` | UUID | Token que autorizó la transmisión |
| `logic_hash` | VARCHAR | Hash del algoritmo de downsampling usado |
| `data_snapshot_id` | UUID | Puntero al dataset original en Parquet |
| `transformation_id` | UUID | ID de la operación de reducción LTTB |
| `indicator_state_hash` | VARCHAR | Hash del estado de indicadores al momento del envío |
| `process_id` | INT32 | PID del proceso que generó el stream |
| `session_id` | UUID | Sesión de ejecución activa |
| `node_id` | VARCHAR | Identificador del hardware (ID de nodo) |
| `event_sequence_id` | INT64 | Número secuencial en el bus de datos |
| `parent_id` | UUID | Relación con el proceso padre |
| `compliance_status_id` | INT32 | Flag de cumplimiento normativo |
| `risk_audit_id` | UUID | ID del log de riesgos asociado |
| `signature_hash` | VARCHAR | Firma digital del paquete (si aplica) |
| `execution_latency_ms` | DOUBLE | Latencia medida de serialización |
| `source_signal_id` | UUID | ID de la señal que disparó la transmisión |
| `audit_hash` | VARCHAR | Hash de verificación forense |
| `version_node_id` | UUID | Hash del nodo en el DAG de versiones |

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Soberanía de Datos (ADR-0032):** El transporte es cifrado entre procesos locales si se requiere (Zero-Trust).
