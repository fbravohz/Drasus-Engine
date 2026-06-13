# Federated Portfolio

**Carpeta:** `./features/federated-portfolio/`
**Estado:** Lista para implementar
**Última actualización:** 2026-05-31
**Decisión Arquitectónica Asociada:** ADR-0090 (Arquitectura de Portafolios Federados)

---

## ¿Qué es esta feature?

El **Federated Portfolio** es una arquitectura avanzada que permite la coexistencia y coordinación de múltiples portafolios independientes y aislados (contenedores de portafolios) que operan simultáneamente dentro de un único macro-sistema o clúster. 

*   **Problema que resuelve:** Los traders diversificados suelen operar estrategias heterogéneas (ej. futuros y criptomonedas) en silos o instancias de ejecución separadas, lo que destruye la sinergia de datos, limita el uso eficiente de adaptadores de broker comunes y complica la observabilidad unificada del riesgo sistémico.
*   **Comportamiento observable:** El operador puede crear múltiples subportafolios lógicos, asignarles un subconjunto de estrategias específicas y definir un conjunto de reglas inmutables (ruleset) propio para cada uno. El sistema ejecuta las órdenes y calcula los márgenes de forma totalmente aislada por contenedor, pero provee un dashboard analítico consolidado del rendimiento del clúster completo.
*   **Por qué la necesitamos:** Permite escalar la infraestructura local-first para soportar esquemas multi-estrategia y multi-cuenta sin incurrir en la sobrecarga y latencia de ejecutar múltiples instancias de NautilusTrader en procesos separados.

---

## Comportamientos Observables

- [ ] **Aislamiento de Reglas:** Cuando una estrategia en el Portafolio A viola su límite de correlación local, el sistema interviene únicamente sobre las estrategias del Portafolio A. Las estrategias del Portafolio B continúan operando con normalidad, incluso si contienen las mismas estrategias o activos correlacionados.
- [ ] **Consolidación de Telemetría:** El panel principal de visualización calcula y proyecta la equidad agregada, el Sharpe Ratio de todo el clúster, y muestra la matriz de correlación inter-portafolio en tiempo real.
- [ ] **Acceso a Infraestructura Compartida:** El clúster multiplexa los feeds de mercado (un único WebSocket activo por símbolo) y utiliza los mismos adaptadores físicos de broker, pero segmenta lógicamente la procedencia y el balance de cada orden.
- [ ] **Kill Switch Global Atómico:** El operador dispone de un botón de emergencia centralizado. Al activarse, el sistema intercepta todas las tareas activas de todos los contenedores y ejecuta un barrido y cierre total de posiciones en paralelo en menos de 5 segundos.

---

## Restricciones

- **NUNCA** una regla definida en un contenedor de portafolio puede anular, modificar o influenciar las decisiones operativas de otro contenedor del clúster.
- **NUNCA** se permite la ejecución de un rebalanceo o modificación de posición sin que el sistema valide que el contenedor de origen posee la asignación de capital aislada suficiente.
- **NUNCA** la latencia agregada por la clasificación y el enrutamiento lógico del contenedor en la aduana de ejecución puede exceder de 1 milisegundo.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| REBALANCING_FREQUENCY | weekly | daily, weekly, monthly, custom | Frecuencia de llamada al daemon de rebalanceo del contenedor | CONFIG |
| MAX_CORRELATION_LIMIT | 0.7 | 0.3 - 0.95 | Límite de correlación interna tolerada entre estrategias de este contenedor | CONFIG |
| VOLATILITY_TARGET | 15% | 5% - 40% | Objetivo de volatilidad anualizada asignado a este contenedor | CONFIG |
| PORTFOLIO_CAPITAL_ALLOCATION | 100,000 | 1,000 - 100,000,000 | Límite de capital dedicado de forma aislada a este contenedor | CONFIG |
| KILL_SWITCH_FLATTEN_MODE | market | market, limit_post | Modo de ejecución de cierre ante detención de emergencia | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

### Core (Lógica Pura)
*   **Validador de Reglas Aislado:** Módulo matemático que evalúa de forma determinista el estado de un contenedor (correlaciones internas, drawdowns, varianza local) contra su ruleset específico sin interactuar con persistencia ni red.
*   **Consolidador de Métricas del Clúster:** Calcula el Sharpe, Sortino y varianza del clúster agregando vectorialmente las curvas de equidad de cada contenedor.

### Shell (Infraestructura)
*   **Enrutador de Órdenes Federado:** Intercepta las solicitudes de las estrategias de cada contenedor y les inyecta los metadatos de identidad del subportafolio de origen antes de transferirlas al `pre-trade-validator`.
*   **Gestor de Persistencia en Base de Datos:** Administra la tabla relacional de configuraciones del contenedor en SQLite y guarda el historial de rendimiento de los subportafolios en Parquet.

### Frontera Pública (Contrato)
*   `evaluate_ruleset(container_id, context)`: Evalúa el cumplimiento local de límites y reglas.
*   `route_order(container_id, order_intent)`: Enruta y etiqueta una orden agregando el linaje de origen.
*   `trigger_global_kill_switch()`: Dispara la detención atómica de todos los contenedores federados.

---

## Ciclo de Vida de la Feature — Federated Portfolio

### Entrada
*   Esquema de configuración JSON inmutable del contenedor.
*   Historial de operaciones y saldo en tiempo real por subportafolio.
*   Feeds de mercado y ticks compartidos.

### Proceso
*   Segmenta lógicamente la telemetría y el balance de capital por contenedor.
*   Aplica la evaluación jerárquica de reglas local en cada contenedor independientemente.
*   Agrega y consolida el riesgo sistémico de todos los contenedores activos.

### Salida
*   Órdenes etiquetadas y validadas ruteadas al adaptador de broker.
*   Métricas agregadas del clúster expuestas en la interfaz de usuario.
*   Alertas de incumplimiento localizadas por contenedor.

### Contextos de Uso

**Contexto 1: Gestión (Módulo Manage)**
*   Organiza los contenedores en memoria y orquestación, programando los triggers de rebalanceo locales para cada uno en el daemon central.

**Contexto 2: Ejecución (Módulo Execute)**
*   Asegura el etiquetado preciso y la validación pre-trade con latencia ultrabaja (<1ms) aislando los riesgos y capitales por canal de origen.

---

## Tareas (TTRs)

### **TTR-001: Estructura de Contenedor de Portafolio y persistencia JSON**
*   **¿Cuál es el problema?** El sistema necesita guardar de forma persistente y estructurada la gobernanza, las estrategias y los límites de cada portafolio federado de forma multi-tenant y local-first.
*   **¿Qué tiene que pasar?** Se crea la tabla relacional `portfolio_containers` en SQLite para el estado operativo y el esquema JSON inmutable para registrar de forma unificada el ruleset del contenedor (rebalanceo, correlaciones, drawdowns, capital asignado).
*   **¿Cómo sé que está hecho?**
    - [ ] Se verifica la creación correcta de la tabla con campos específicos de auditoría en la base de datos relacional local.
    - [ ] El sistema puede guardar y rehidratar un contenedor con sus límites de forma inmutable sin pérdida de información.
*   **¿Qué no puede pasar?**
    - No se permite almacenar contraseñas o datos de conexión del broker en la tabla de configuración.
    - No se permite violar los tipos estructurados definidos en la validación inicial del esquema.

### **TTR-002: Aislamiento estricto de gobernanza en ejecutor**
*   **¿Cuál es el problema?** Las estrategias del Portafolio A podrían verse afectadas por los límites del Portafolio B si el motor de reglas no segmenta el contexto de evaluación de forma atómica.
*   **¿Qué tiene que pasar?** El motor de reglas de ejecución segmenta la evaluación barra a barra de manera lógica. Cuando el Portafolio A evalúa sus límites de correlación o VaR, el proceso ignora completamente los activos y posiciones correspondientes al Portafolio B.
*   **¿Cómo sé que está hecho?**
    - [ ] Pruebas unitarias demuestran que una violación de drawdown en el Portafolio A suspende la operativa de A pero deja al Portafolio B operando con normalidad.
    - [ ] La latencia agregada del enrutador de órdenes al clasificar la procedencia es menor a 1ms.
*   **¿Qué no puede pasar?**
    - No se permite evaluar reglas de forma cruzada o agregada a menos que se llame explícitamente al validador del clúster completo.
    - No se permiten desbordamientos de asignación de margen entre subportafolios en el modo de capital aislado.

### **TTR-003: Monitoreo agregado centralizado y Kill Switch Global**
*   **¿Cuál es el problema?** El operador necesita visibilidad del riesgo agregado total de sus subportafolios y la capacidad de detener todo instantáneamente si ocurre un evento catastrófico global.
*   **¿Qué tiene que pasar?** El sistema unifica la telemetría de todos los contenedores activos, proyecta el Sharpe agregado y la matriz de correlación inter-portafolio en la UI, y provee un botón físico de Kill Switch Global que cancela y liquida todas las operaciones federadas concurrentemente en paralelo.
*   **¿Qué no puede pasar?**
    - El Kill Switch no puede ser bloqueado o encolado detrás de tareas pesadas de R&D; opera con prioridad absoluta P0.

---

## Gobernanza y Estándares (Fijos)

### Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda transacción y cambio de estado del portafolio federado registra los metadatos de relevancia técnica. **Perfil C (Ops / Hot-Path)** (I + II + IV + V latencia/gobernanza):

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del evento federado |
| | `created_at` | Timestamp del evento con precisión de nanosegundos |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash SHA-256 del estado consolidado del clúster |
| | `audit_chain_hash` | Hash de la cadena temporal para trazabilidad inmutable |
| | `event_sequence_id` | Secuencia de recuperación del evento federado |
| **II. Soberanía** | `owner_id` | Identificador del operador (Trader A, B, etc.) |
| | `manifest_id` | ID del manifiesto de diseño inmutable |
| | `access_token_id` | Firma de autorización del proceso local |
| **IV. Hardware** | `node_id` | Identificador único de la máquina local |
| | `process_id` | PID del hilo del ejecutor reservado (`Core Pinning`) |
| **V. Forense & Ejecución** | `execution_latency_ms` | Latencia exacta de enrutamiento y validación (<1ms target) |
| | `source_signal_id` | Señal de origen del evento federado |
| | `portfolio_container_id` | Contenedor de portafolio federado (Gobernanza) |
| | `compliance_status_id` | Veredicto de cumplimiento del ruleset del contenedor |

*   **Rastro de Evidencia:** El contenedor emite señales estructuradas del cumplimiento del ruleset local y logs detallados de cualquier descarte de señal por veto de correlación o VaR para alimentar el análisis causal del módulo `feedback`.

---

## Dependencias

**Depende de:**
*   [`portfolio-rules`](../features/portfolio-rules.md) — para la lógica base del envolvente de reglas.
*   [`pre-trade-validator`](../features/pre-trade-validator.md) — para interceptar órdenes en el hot-path.

**Consumido por:**
*   [`manage`](../modules/manage.md) — para la optimización de pesos y control del clúster.
*   [`execute`](../modules/execute.md) — para el ruteo de ejecuciones en el LiveNode.
