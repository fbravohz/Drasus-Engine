# Marketplace de Cajas Negras (Zero-Knowledge Nodes)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0099 (Marketplace de Cajas Negras con Zero-Knowledge Nodes)

---

## ¿Qué es?

Permite a creadores de estrategias empaquetar, encriptar y monetizar subgrafos complejos de lógica visual como un solo nodo cerrado ("Caja Negra") dentro de un marketplace local descentralizado. Los compradores de estos nodos pueden agregarlos a su lienzo para recibir señales dinámicas, pero **nunca** pueden ver, inspeccionar o revertir el Árbol de Sintaxis Abstracta (AST) ni la lógica matemática interna, protegiendo al 100% la propiedad intelectual del autor.

---

## Comportamientos Observables

- [ ] **Empaquetado y Encriptación local:** El creador selecciona un conjunto de nodos en el editor visual, ingresa un precio/suscripción y presiona "Empaquetar como Caja Negra". El sistema genera un archivo empaquetado binario cifrado mediante claves asimétricas.
- [ ] **Descubrimiento Descentralizado:** Los usuarios navegan por el marketplace local indexado a partir de metadatos compartidos P2P, visualizando estadísticas de rendimiento (Sharpe, Drawdown, Profit Factor) sin poder acceder al código fuente.
- [ ] **Ejecución Opaca en Canvas:** El comprador arrastra el nodo de Caja Negra a su lienzo de diseño. El nodo expone puertos de entrada estándar (datos de mercado) y de salida (señales operativas). Al ejecutar el backtesting o live trading, la evaluación del subgrafo se realiza dentro de una caja de ejecución aislada mediante llaves efímeras sin filtrar logs de ejecución interna.
- [ ] **Gestión de Suscripción Local:** El sistema valida la firma digital del contrato de suscripción contra el ID de hardware soberano local antes de inicializar el nodo en el motor de ejecución.

---

## Restricciones

- **NUNCA** exponer los sub-nodos lógicos del AST de la Caja Negra en la interfaz gráfica del comprador ni a través de volcados de memoria en el runtime.
- **NUNCA** almacenar la llave de descifrado simétrica de la lógica en el disco duro del comprador; se maneja de forma exclusiva en la memoria volátil del Hot-Path durante la ejecución.
- **FIJO:** El rastro de auditoría del marketplace se registra localmente en SQLite, manteniendo un log de firmas digitales para verificar el linaje y licencias vigentes.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| LICENCE_CHECK_INTERVAL | 24h | 1h - 7d | Intervalo para re-verificar de forma asíncrona la validez criptográfica de la licencia | CONFIG |
| RETRY_GRACE_PERIOD | 3 | 0 - 10 | Intentos permitidos de fallo de verificación antes de pausar temporalmente el nodo | CONFIG |
| IS_SANDBOXED | true | true - false | Ejecutar la evaluación de la caja negra en una sandbox de baja prioridad | [FIJO] |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Criptografía asimétrica y descifrado en caliente del AST de la estrategia. Evaluación determinista del flujo del grafo.
- **Shell (Infraestructura):** Integración con la red de distribución P2P de metadatos del marketplace y persistencia en SQLite de contratos de licencia.

---

## Ciclo de Vida de la Feature — Black-Box Marketplace

### Entrada
- Subgrafo lógico de nodos (JSON/Protobuf AST).
- Claves criptográficas de autor y de cliente (Hardware ID).
- Datos de mercado crudos/procesados.

### Proceso
- Cifrado del AST original con clave asimétrica.
- Al importar, descifrado efímero en la memoria RAM restringida del backend Rust.
- Evaluación del subgrafo ante ticks de mercado entrantes dentro del NautilusTrader Event-Loop.

### Salida
- Señal operativa normalizada (comprar, vender, mantener) y nivel de confianza.
- Logs de rendimiento y latencia, sin trazas internas del subgrafo.

---

## Tareas (TTRs)

### **TTR-001: Empaquetador y Cifrador Asimétrico de Subgrafos**
*   **¿Cuál es el problema?** Si un creador quiere vender su lógica, los formatos normales JSON revelan toda la combinación de indicadores y umbrales, permitiendo el robo de IP.
*   **¿Qué tiene que pasar?** El creador selecciona subgrafos y el sistema los compila a un binario plano cifrado con AES-256-GCM y firmado digitalmente mediante Ed25519. El resultado final se expone en el catálogo de metadatos locales.
*   **¿Cómo sé que está hecho?**
    - [ ] Se exporta el nodo empaquetado y al intentar leer su contenido con un editor de texto solo se visualiza ruido criptográfico.
    - [ ] El archivo de metadatos expone el Sharpe y el linaje sin revelar conexiones de nodos.
*   **¿Qué no puede pasar?**
    - No se deben incluir nombres de variables ni fórmulas legibles en los metadatos exportados.

### **TTR-002: Motor de Evaluación Aislado en Memoria (Zero-Knowledge Engine)**
*   **¿Cuál es el problema?** Al evaluar la estrategia en el backtest del comprador, la memoria o los logs del motor podrían revelar la estructura de la caja negra.
*   **¿Qué tiene que pasar?** El orquestador Rust del backend descifra el AST directamente en la memoria volátil utilizando llaves simétricas temporales de sesión. La evaluación del grafo se realiza a nivel nativo en Rust sin escribir trazas de depuración de los sub-nodos lógicos.
*   **¿Cómo sé que está hecho?**
    - [ ] El backtesting se ejecuta correctamente y consume la lógica de la caja negra para abrir operaciones en el sandbox de simulación.
    - [ ] El archivo de log general (`audit-log.md`) contiene la latencia del nodo pero no las variables internas del subgrafo.
*   **¿Qué no puede pasar?**
    - Si el descifrado falla en memoria, el motor debe detener la ejecución de inmediato (Fail-Fast) sin comprometer el capital.

---

## Gobernanza y Estándares (ADR-0020)

### Perfil Ops / Auditoría
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de licencia de nodo único |
| | `created_at` | Timestamp de activación del nodo |
| | `audit_hash` | SHA-256 de la caja negra binaria cifrada |
| | `audit_chain_hash` | Encadenamiento con transacciones de compra anteriores |
| **II. Soberanía** | `owner_id` | Identificador del comprador de la licencia |
| | `institutional_tag` | Entorno de despliegue local/VPS |
| **IV. Hardware** | `node_id` | Identificador físico del hardware del usuario |
| | `process_id` | PID de ejecución en la máquina |
| | `execution_latency_ms` | Latencia consumida en el ciclo de ejecución interna |
