# Data Multiplexing Bus (Pub/Sub)

**Carpeta:** `./features/data-bus-pubsub/`
**Estado:** Lista para implementar
**Última actualización:** 2026-05-12
**Decisión Arquitectónica Asociada:** ADR-0085 (Bus de Datos Pub/Sub Zero-Copy)

## ¿Qué es esta feature?

En sistemas con múltiples agentes operando simultáneamente, la redundancia de datos de mercado es un cuello de botella crítico. Si 50 estrategias operan el mismo símbolo (ej: BTC/USDT), abrir 50 conexiones gRPC/WebSocket individuales no solo consume ancho de banda innecesario, sino que provoca el baneo inmediato de la IP por parte del exchange.

Esta feature implementa un **Bus de Datos Multiplexado** basado en el patrón Pub/Sub nativo de Rust. El sistema levanta una única conexión física hacia el mercado por cada símbolo y distribuye los eventos (ticks, cambios en el libro de órdenes) a todos los agentes suscritos mediante **paso por referencia en memoria (Zero-Copy)**.

## Comportamientos Observables

- [ ] El sistema levanta una sola conexión gRPC/WebSocket hacia Binance/IBKR para el símbolo "EURUSD", independientemente de cuántas estrategias lo usen.
- [ ] Al añadir una nueva estrategia que usa un símbolo ya activo, el sistema la conecta al bus existente instantáneamente sin latencia de negociación de red.
- [ ] El uso de memoria RAM se mantiene estable aunque el número de agentes aumente, ya que todos leen el mismo objeto de datos en memoria.
- [ ] Si la conexión principal cae, todos los agentes reciben la notificación de desconexión simultáneamente desde el bus central.

## Restricciones

- **NUNCA** permitir que un agente modifique los datos del bus (los datos recibidos deben ser inmutables/solo lectura para los suscriptores).
- **NUNCA** abrir más de una conexión física por símbolo y por proveedor de datos.
- **Límite Técnico:** La latencia de distribución interna desde el bus hacia el agente debe ser inferior a 100 nanosegundos.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MAX_SUBSCRIBERS_PER_BUS | 1000 | 10 - 10000 | Límite de agentes por cada bus de símbolo | CONFIG |
| BUS_BUFFER_SIZE | 1024 | 128 - 1048576 | Tamaño del buffer circular de eventos | CONFIG |
| DROP_SLOW_SUBSCRIBERS | true | true/false | Desconectar agentes que no procesan a la velocidad del mercado | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Gestión de suscripciones y enrutamiento de mensajes.
- **Shell (Infraestructura):** Canales asíncronos de Rust (`tokio::sync::broadcast`), gestión de WebSockets externos.
- **Frontera Pública:** Interfaz para `subscribe_to_symbol(symbol)`, `unsubscribe_from_symbol(symbol)` y `broadcast_market_event(event)`.

## Ciclo de Vida de la Feature

### Entrada
- Eventos crudos desde el gRPC/WebSocket del exchange.
- Peticiones de suscripción de nuevos agentes.

### Proceso
- Valida y normaliza el evento entrante al formato interno de NautilusTrader.
- Inyecta el evento en el canal de difusión (`broadcast channel`).
- Notifica a todos los suscriptores activos en paralelo.

### Salida
- Flujo de datos en tiempo real para cada agente.
- Telemetría de latencia de distribución.

## Tareas (TTRs)

### **TTR-001: Implementación del Single Data Client**
*   **¿Cuál es el problema?** El sistema necesita una forma de saber si ya hay alguien escuchando un símbolo para no abrir una conexión duplicada.
*   **¿Qué tiene que pasar?** Al solicitar datos de un símbolo, el gestor de conexiones debe buscar en su registro interno. Si ya existe, devuelve un nuevo receptor del bus. Si no, inicializa la conexión y crea el bus.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo abrir 10 instancias del mismo símbolo y verificar en los logs que solo hubo una negociación de conexión con el exchange.
    - [ ] No recibo errores de "Rate Limit" del broker.

### **TTR-002: Distribución Zero-Copy mediante Referencias**
*   **¿Cuál es el problema?** Clonar objetos grandes de Order Book para 100 agentes consume mucha CPU y RAM.
*   **¿Qué tiene que pasar?** El bus debe distribuir punteros inteligentes (`Arc<T>`) o referencias inmutables para que todos los agentes lean la misma dirección de memoria.
*   **¿Cómo sé que está hecho?**
    - [ ] El perfil de memoria (heap) no aumenta proporcionalmente al número de agentes para el mismo símbolo.
    - [ ] Las pruebas de latencia muestran que el tiempo de entrega es casi nulo.

## Dependencias

**Depende de:**
- Módulo `ingest` (para la gestión de conexiones de red).
- Estructuras de datos de `NautilusTrader`.

**Bloquea:**
- Escalado masivo de agentes en tiempo real.
