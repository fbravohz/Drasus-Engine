# Motor de Copy-Trading

**Carpeta:** `./features/copy-trading-engine/`  
**Estado:** En Diseño  
**Última actualización:** 2026-06-04  
**Decisión Arquitectónica Asociada:** ADR-0092 (Copy-Trading mediante Relé Ciego de Señales)

---

## 1. ¿Qué es esta feature?

El Motor de Copy-Trading permite a los traders maestros (Masters) distribuir la ejecución de sus estrategias en tiempo real a múltiples clientes (Copiers) autorizados de manera segura, sin exponer su lógica interna ni su dirección IP pública. El sistema utiliza una arquitectura basada en un servidor intermedio neutro (Signal Relay) y encriptación de extremo a extremo.

### Problema que resuelve
El copy-trading tradicional peer-to-peer (P2P) directo satura el ancho de banda del máster cuando tiene decenas de clientes, y expone su infraestructura a ataques maliciosos al revelar su dirección IP pública. Además, la copia centralizada en un único broker limita la flexibilidad del cliente para elegir su propio intermediario financiero.

### Comportamiento general
1.  **Master:** Emite un flujo continuo de señales cifradas (comportamiento en vivo) mediante una única conexión saliente segura hacia el Signal Relay.
2.  **Signal Relay:** Recibe los bytes cifrados y los distribuye asíncronamente a los Copiers conectados y autorizados. No conoce el contenido del mensaje (Zero-Knowledge).
3.  **Copier:** Descarga un cliente ligero, se autentica con una clave criptográfica, descifra localmente la señal, calcula el tamaño de orden adecuado a su capital mediante un algoritmo de escalado de riesgo y la envía a su broker local.

---

## 2. Comportamientos Observables

*   [ ] **Distribución en un Solo Sentido:** El máster envía una señal de orden ejecutada. El sistema emite un único payload cifrado al relé. Los copiers reciben el mensaje en menos de 50ms.
*   [ ] **Validación Criptográfica:** Si el mensaje es alterado en el relé, la verificación HMAC falla en la máquina del copier. La señal se rechaza y se genera un log de alerta de alteración de datos.
*   [ ] **Rechazo de Señales Caducas:** Si la marca de tiempo de la señal tiene una antigüedad mayor a la ventana máxima permitida (ej. 5 segundos), el copier rechaza automáticamente la orden para evitar deslizamientos de precio.
*   [ ] **Escalado de Riesgo con Límite de Capital:** Si la cantidad proporcional calculada (según ratio de capital máster/copier) excede el porcentaje de riesgo máximo por operación del copier en base a la volatilidad histórica (ATR), la cantidad se reduce automáticamente al límite permitido.
*   [ ] **Hedge Local Temporal:** Si el copier cierra una posición manualmente antes que el máster, la terminal del copier ignora señales posteriores sobre ese activo y previene reaperturas automáticas hasta que el máster cierre su posición original.
*   [ ] **Autopausa de Emergencia:** Si el copier sufre una desconexión de broker de más de 30 segundos, suspende temporalmente la copia de señales nuevas y notifica a la UI del usuario.

---

## 3. Restricciones

*   **FIJO — NO CONFIGURABLE:** El Signal Relay nunca almacena ni lee las claves simétricas de cifrado de sesión. La desencriptación se realiza estrictamente en la máquina local del copier.
*   **FIJO — NO CONFIGURABLE:** El copier nunca puede apalancar la cuenta por encima del apalancamiento relativo utilizado por el máster.
*   **FIJO — NO CONFIGURABLE:** Queda prohibido el uso de conexiones entrantes directas (puertos abiertos) en la terminal del Master para la distribución de señales.

---

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| MAX_SIGNAL_AGE_SECS | 5.0 | 1.0 - 10.0 | Ventana de tiempo máxima para aceptar una señal antes de descartarla. | CONFIG |
| VOLATILITY_ADJUSTMENT | true | true / false | Si está activo, reduce la cantidad de orden ante alta volatilidad (ATR). | CONFIG |
| COPIER_MAX_RISK_PCT | 2% | 0.5% - 5.0% | Porcentaje de riesgo máximo del capital del copier expuesto por trade. | CONFIG |
| COPIER_MAX_DRAWDOWN | 20% | 5.0% - 50.0% | Drawdown acumulado local máximo permitido antes de la pausa automática. | CONFIG |
| PING_INTERVAL_SECS | 10.0 | 5.0 - 60.0 | Frecuencia de latidos de vida (heartbeat) entre terminales y el relé. | CONFIG |

---

## 5. Estructura Interna (FCIS)

### Core (Lógica Pura)
*   **Cifrado y Compresión:** Funciones para compactar datos, cifrarlos usando AES-256-GCM y generar firmas HMAC-SHA256 con claves criptográficas.
*   **Risk Scaler Engine:** Algoritmo matemático puro que toma la cantidad del máster, el capital de ambas partes, el ATR del activo y los parámetros de riesgo del copier para retornar la cantidad exacta a enviar al broker (retorna 0 si es inferior al lote mínimo del broker).

### Shell (Infraestructura)
*   **Conector del Relé (Client):** Gestor de la conexión socket (WebSocket/gRPC) para mantener el canal abierto hacia el Signal Relay.
*   **Manejador de Señales (Orquestador):** Descompone la señal entrante en el copier, valida la firma, ejecuta el risk scaling local y despacha la orden al orquestador del módulo `execute`.

### Frontera Pública
*   El motor expone un puerto de entrada para recibir la señal cruda binaria del relé.
*   El motor del máster expone una interfaz de salida para despachar señales cifradas inmediatamente después de la confirmación de llenado de sus órdenes en el broker.

---

## 6. Ciclo de Vida de la Feature

```
[Señal del Master] ──> (Cifrado y Firma en Master) ──> [Payload Cifrado] ──> (Transmisión al Relay)
                                                                                  │
┌─────────────────────────────────── [Recepción en Copier] ───────────────────────┘
│
▼
(Verificar HMAC y Timestamp) ──> (Calcular Risk Scaling con ATR local) ──> [Orden Local Replicada]
```

### Entrada
*   Payload de orden en origen (instrumento, precio, stop loss, take profit, cantidad, dirección).
*   Configuraciones locales del copier (capital, límites de riesgo, clave simétrica).

### Proceso
*   El máster cifra los datos de orden en origen y los transmite al relé.
*   El relé replica la señal cifrada a todos los copiers autenticados.
*   El copier verifica la integridad del payload y calcula el tamaño proporcional ajustado por riesgo y ATR.

### Salida
*   Orden local escalada despachada al broker en la máquina del copier.
*   Log inmutable de auditoría del copiado.

---

## 7. Tareas (TTRs)

### **TTR-001: Streaming de señales encriptadas desde el Master**
*   **¿Cuál es el problema?** El Master necesita enviar detalles de sus operaciones a los copiers con rapidez sin exponer los datos en texto plano por la red ni saturar su ancho de banda de subida con múltiples sockets de clientes.
*   **¿Qué tiene que pasar?** El sistema captura la orden confirmada del máster, empaqueta los campos requeridos en una estructura, la comprime, la cifra mediante AES-256-GCM y firma el mensaje con HMAC. Posteriormente, envía el payload resultante a través de una única conexión saliente WebSocket/gRPC TLS activa hacia el relé ciego de señales.
*   **¿Cómo sé que está hecho?**
    *   [ ] Se verifica en logs de red que solo existe una conexión saliente activa del Master al Relay.
    *   [ ] El payload capturado en tránsito por la red es ilegible en texto plano.
    *   [ ] Las marcas de tiempo (timestamps) de salida se registran en nanosegundos exactos.
*   **¿Qué no puede pasar?**
    *   No se puede enviar ningún dato de la orden sin cifrado previo.
    *   No se permiten conexiones entrantes al socket del Master.

### **TTR-002: Autenticación y distribución asíncrona en el Signal Relay**
*   **¿Cuál es el problema?** El servidor relé necesita validar la legitimidad de las conexiones de los copiers y redistribuir los payloads cifrados con latencia mínima sin tener acceso a la clave simétrica de encriptación.
*   **¿Qué tiene que pasar?** El relé valida las claves criptográficas firmadas por el Master de los copiers en la conexión inicial HTTPS. Una vez establecida la sesión WebSocket/gRPC, el relé recibe los payloads cifrados del máster y los retransmite de forma asíncrona a todos los copiers conectados correspondientes al ID del máster.
*   **¿Cómo sé que está hecho?**
    *   [ ] Copiers con firmas de clave inválidas o expiradas son rechazados en el handshake inicial.
    *   [ ] El relé retransmite el payload intacto sin modificar ni una sola firma HMAC.
    *   [ ] La distribución asíncrona maneja múltiples copiers concurrentes sin degradación de velocidad.
*   **¿Qué no puede pasar?**
    *   El relé no puede almacenar logs descifrados de operaciones (ya que carece de la clave).
    *   No se permiten retransmisiones de señales a copiers que no pertenezcan al grupo de suscripción autorizado del Master.

### **TTR-003: Procesamiento y Risk Scaling en el Copier**
*   **¿Cuál es el problema?** El Copier recibe el payload de señal cifrada y necesita verificar su validez y calcular el lotaje correspondiente a su capital local y límites de drawdown, protegiendo su cuenta de un sobreapalancamiento catastrófico.
*   **¿Qué tiene que pasar?** El copier recibe la señal del relé, valida el timestamp (descarta si la latencia de llegada > 5 segundos) y la firma HMAC. Tras el descifrado, ejecuta el algoritmo de Risk Scaling: calcula el tamaño base proporcional al capital y lo ajusta a la baja si el ATR del activo muestra alta volatilidad o si el riesgo en dólares supera el riesgo máximo local (ej. 2% por trade).
*   **¿Cómo sé que está hecho?**
    *   [ ] El copier rechaza señales cuya firma HMAC no sea válida y emite un log de alerta.
    *   [ ] Si la señal es válida, se calcula el lotaje escalado y se despacha la orden localmente en menos de 5ms.
    *   [ ] Si la volatilidad ATR es alta, el tamaño de orden se reduce en la proporción configurada.
*   **¿Qué no puede pasar?**
    *   No se permite procesar señales que superen la latencia máxima parametrizada.
    *   Bajo ninguna circunstancia la cantidad de orden calculada puede exceder el límite de riesgo local en dólares de la cuenta.

### **TTR-004: Gestión de la máquina de estados local para cierres y modo "locally_closed"**
*   **¿Cuál es el problema?** Si el copier decide cerrar una posición de forma manual y anticipada en su broker local, el sistema debe evitar que las señales posteriores del máster para el mismo activo abran nuevas posiciones erróneas.
*   **¿Qué tiene que pasar?** Al detectar un cierre local manual en el broker, la terminal del copier marca el activo/estrategia con una bandera de estado temporal. El sistema continúa procesando y verificando las señales de salida del máster para sincronizar el cierre formal, pero bloquea cualquier orden de entrada nueva de esa estrategia hasta que el máster cierre su posición original.
*   **¿Cómo sé que está hecho?**
    *   [ ] Al cerrar una posición en el broker del copier, el sistema entra en estado inactivo para ese activo en particular.
    *   [ ] Nuevas señales de entrada del Master para el activo inactivo son ignoradas localmente.
    *   [ ] Una vez que el máster emite la señal de cierre total, la bandera de estado se limpia y la copia se reanuda con normalidad.
*   **¿Qué no puede pasar?**
    *   No se permiten desincronizaciones infinitas de posiciones locales.

---

## 8. Gobernanza y Estándares (Fijos)

### Local-First (ADR-0016)
La feature opera de forma híbrida: el relé corre de forma externa (VPS / Nube) para permitir conectividad pública sin exponer la IP del máster, pero todo el cifrado en origen, descifrado, almacenamiento de credenciales y escalado de riesgo opera estrictamente de forma local en las máquinas del Master y del Copier.

### Fidelidad (ADR-0017)
Las señales replicadas deben modelarse con la fricción del broker local del copier (comisiones y spreads específicos del broker destino), calculando el impacto de slippage en tiempo real antes del disparo de la orden replicada.

### Inundación de Fundaciones (ADR-0020 V2)
Esta feature pertenece al perfil **Ops / Hot-Path**. Cada evento de copia y orden replicada en el copier debe registrar en SQLite los siguientes campos mandatorios de auditoría:
*   `id` (UUID de la orden local)
*   `created_at` (timestamp local de origen en nanosegundos)
*   `audit_hash` (firma digital del payload de orden)
*   `audit_chain_hash` (firma acumulada del ledger de la sesión de copiado)
*   `owner_id` (identificador del copier local)
*   `compliance_status_id` (resultado de validación táctica local)
*   `node_id` (identificación del hardware físico local del copier)
*   `process_id` (PID de la terminal en ejecución)
*   `execution_latency_ms` (tiempo medido en milisegundos desde la creación de la señal en el máster hasta su despacho local en el broker)

### Rastro de Evidencia
La feature emite los siguientes datos para el módulo de `feedback`:
*   Latencia de tránsito de la señal (tiempo máster -> copier).
*   Slippage de ejecución (precio propuesto por máster vs precio de fill en broker local).
*   Eventos de desconexión del relé o del broker local.
*   Ratios de órdenes rechazadas por límites de riesgo.

---

## 9. Dependencias y Bloqueantes

**Depende de:**
*   [`execute`](../modules/execute.md) — para despachar las órdenes replicadas al broker local.
*   [`broker-connector`](../features/broker-connector.md) — para la conexión física con las APIs de los brokers.
*   [`slippage-models`](../features/slippage-models.md) — para estimar spreads en origen.

**Bloquea:**
*   (Ninguna)
