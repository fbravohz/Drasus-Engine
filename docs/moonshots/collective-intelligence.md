# Collective Intelligence (Harvesting)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0102 (Anonimización Criptográfica local-first en Collective Intelligence)

---

## ¿Qué es?

Permite el intercambio y análisis cooperativo descentralizado de métricas y patrones de trading entre usuarios de la plataforma de forma 100% anónima. A través de un Meta-Learner local y un extractor de patrones, el sistema analiza las bases de datos de veredictos y rendimientos de miles de usuarios (sabiduría de la multitud) para descubrir combinaciones robustas de indicadores y regímenes exitosos sin comprometer en ningún momento la lógica íntima de las estrategias individuales ni los datos financieros de las cuentas.

---

## Comportamientos Observables

- [ ] **Anonimización en Origen:** El sistema purga cualquier metadato sensible, nombres de variables o ecuaciones explícitas de la base de datos de veredictos antes de preparar el paquete de exportación.
- [ ] **Suscripción al Data Exchange:** El usuario decide de forma voluntaria activar la contribución de datos. A cambio, recibe acceso al feed consolidado de "huellas digitales de Alpha" de otros participantes de la red.
- [ ] **Extracción de Patrones Globales:** El motor de R&D analiza las combinaciones de indicadores correlacionadas con altos Sharpe ratios y drawdowns controlados en la red global, proponiendo plantillas base de estrategias (Alpha Blueprints) al Genetic Builder local.

---

## Restricciones

- **OBLIGATORIO:** Cifrar y anonimizar todos los datos mediante hashing criptográfico y agregar ruido estadístico (privacidad diferencial) antes de subirlos a la red de intercambio.
- **NUNCA** transmitir fórmulas matemáticas legibles del AST, llaves API, balances en dólares o IPs de servidores de los usuarios.
- **FIJO:** La participación en el Data Exchange se rige por un esquema de consentimiento explícito (opt-in) desactivado por defecto en la instalación.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| ANONYMIZATION_NOISE_LEVEL | 0.05 | 0.01 - 0.20 | Nivel de ruido aleatorio inyectado para garantizar privacidad diferencial | CONFIG |
| UPDATE_FEED_FREQUENCY_DAYS | 7 | 1 - 30 | Cada cuántos días descargar la base de conocimiento colectiva consolidada | CONFIG |
| EXCLUDE_PRO_STRATEGIES | true | true / false | Excluye estrategias marcadas como altamente confidenciales de la exportación | [FIJO] |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmos de privacidad diferencial, hashing criptográfico de firmas de indicadores y motor de Meta-Learning.
- **Shell (Infraestructura):** Cliente HTTP/P2P para transmisión de payloads anonimizados y persistencia en SQLite de logs de intercambio.

---

## Ciclo de Vida de la Feature — Collective Intelligence

### Entrada
- Historial de veredictos locales, métricas de rendimiento (Sharpe, Drawdown) y huella digital de combinación de indicadores (sin parámetros explícitos).
- Base de datos colectiva consolidada descargada de la red.

### Proceso
- Purga estricta y adición de ruido estocástico (Differential Privacy) sobre las métricas de rendimiento de las estrategias aprobadas.
- Hashing unidireccional de la topología de la estrategia.
- Carga de datos globales agregados en el extractor de patrones para buscar correlaciones robustas.

### Salida
- Sugerencias de topología de indicadores (Alpha Blueprints) para inyectar en el módulo de generación.

---

## Tareas (TTRs)

### **TTR-001: Extractor de Firmas de Estrategia Anonimizadas**
*   **¿Cuál es el problema?** Si los usuarios comparten sus estrategias en crudo, pierden su IP y ventaja competitiva. Pero si no comparten nada, perdemos la oportunidad de aprender de los errores y aciertos colectivos.
*   **¿Qué tiene que pasar?** Desarrollar un extractor en Rust que reduzca la estrategia a una "firma de topografía de indicadores" utilizando hashes unidireccionales (SHA-256) de los nombres de los indicadores en secuencia, omitiendo sus periodos y umbrales (ej: `SHA256(RSI+MACD+EMA)`). El rendimiento en porcentaje se normaliza y se altera levemente mediante ruido gaussiano controlado.
*   **¿Cómo sé que está hecho?**
    - [ ] El payload de exportación generado no contiene números de periodos, coeficientes matemáticos ni nombres de variables legibles.
    - [ ] Se reconstruye la firma hash y se asocia con éxito a su Sharpe Ratio ruidoso en el JSON final.
*   **¿Qué no puede pasar?**
    - No debe haber forma de recuperar la ecuación matemática original a partir de la firma hash exportada.

### **TTR-002: Motor de Aprendizaje Cooperativo (Meta-Learner)**
*   **¿Cuál es el problema?** Los mercados evolucionan y las combinaciones de indicadores que funcionaban hace un año pueden fallar hoy de forma global.
*   **¿Qué tiene que pasar?** Implementar un motor de minería asociativa local (ej: algoritmo Apriori adaptado) que procese el dataset consolidado de firmas colectivas. Clasifica y detecta cuáles firmas muestran un incremento de rendimiento global bajo regímenes de mercado específicos informados por el HMM.
*   **¿Cómo sé que está hecho?**
    - [ ] El Meta-Learner detecta que en regímenes volátiles las combinaciones de tipo oscilador tienen una probabilidad de fallo del 80%.
    - [ ] El sistema genera una alerta local sugiriendo desactivar o reestructurar estrategias basadas en esos indicadores ruidosos.
*   **¿Qué no puede pasar?**
    - No se deben usar procesos sincrónicos pesados que bloqueen el hilo de UI de Flutter durante el entrenamiento de asociación de datos.

---

## Gobernanza y Estándares (ADR-0020 V2)

### Perfil IA / R&D
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de la sesión de contribución de datos |
| | `created_at` | Timestamp de exportación de firmas |
| | `audit_hash` | Hash de integridad del payload colectado |
| **II. Soberanía** | `owner_id` | Identificador de firma anónima de usuario |
| **III. Pesos/Modelos** | `logic_hash` | Hash del algoritmo de anonimización |
| | `data_snapshot_id` | Puntero al dataset global analizado |
| **IV. Hardware** | `node_id` | Identificador único del nodo contribuyente |
| | `process_id` | PID de la tarea de sincronización P2P |
| | `execution_latency_ms` | Latencia consumida en el cifrado y transporte de datos |
