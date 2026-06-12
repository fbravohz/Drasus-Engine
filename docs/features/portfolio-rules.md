# Portfolio Rules & Rules Wrapper

**Carpeta:** `./features/portfolio-rules/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0010, ADR-0015, ADR-0020 V2, ADR-0079, ADR-0108, ADR-0111

---

---

## ¿Qué es?

Componente de gobernanza encargado de imponer los límites de seguridad globales del portafolio. Actúa como el **Filtro de Invariantes** y una **Capa Envolvente de Reglas (Rules Wrapper)**: asegura que la operativa colectiva nunca viole los límites técnicos o de negocio (ej. Prop Firm Compliance, Challenge Mode), independientemente de lo que pidan las estrategias individuales o el número de estrategias que lo conformen.

**Gen de Condición de Estado del Genoma de Portafolio y Correlación — DD/Volatilidad Agregada de Cartera (ADR-0108/ADR-0111):** los Límites Duros sobre Drawdown del Portafolio y el Prop Firm Drawdown Tracker (Equity vs Balance, Trailing Max Drawdown) son la instancia FIJA del Gen de Condición "DD/Volatilidad Agregada de Cartera" del Dominio de Portafolio y Correlación. Cuando ese genoma está activo, el `RuleVerdict` agregado de la cartera en cada barra se expone como entrada de solo lectura a los Genes de Acción del genoma de portafolio (activación/desactivación de miembro, rotación de pesos, cobertura sintética), sin alterar la jerarquía de veto FIJA de esta feature.

---

## Comportamientos Observables

- [ ] Evalúa **Límites Duros** (ej: Max Drawdown Portafolio) que detienen la operativa global.
- [ ] Emite **Alertas Blandas** cuando las métricas se acercan a los umbrales de seguridad.
- [ ] Bloquea órdenes individuales si la exposición total del portafolio excede el capital o infringe reglas regulatorias.
- [ ] **Challenge Mode:** Inyección de perfiles de riesgo específicos por fases (Fase 1, Fase 2, Funded) adaptando los umbrales de drawdown y metas.
- [ ] **Prop Firm Drawdown Tracker:** Seguimiento Tick-by-Tick de Equity vs Balance (Midnight rule) y Trailing Max Drawdown.
- [ ] **Temporal & News Blackouts:** Bloqueo de operaciones y liquidación antes de eventos macro de alto impacto.
- [ ] **Inventory Constraints:** Forzado de modo netting y reglas FIFO si la cuenta lo requiere.
- [ ] Cuando el Genoma de Portafolio y Correlación (ADR-0111) está activo, el `RuleVerdict` agregado (APROBADA/BLOQUEADA/VETADA) de cada barra se expone como Gen de Condición de Estado de DD/Volatilidad de Cartera para el motor evolutivo de co-evolución.

---

## Ciclo de Vida de la Feature — Portfolio Rules

### Entrada
- Intención de orden de una estrategia o estado actual del portafolio.
- Tabla de reglas configuradas (Hard/Soft Limits, Prop Firm Rules, Challenge Mode).
- Contexto de mercado / noticias macro.

### Proceso
- Intercepta cada solicitud de cambio de posición.
- Simula el impacto de la orden en el margen y riesgo total del portafolio.
- Valida contra la jerarquía de reglas (Global Rules Wrapper > Estrategia).

### Salida
- **Veredicto de Regla:** APROBADA / BLOQUEADA / VETADA.
- **Razón del Veto:** Detalle técnico del límite violado.

### Contextos de Uso
**Contexto 1: Ejecución (Módulo Execute)**
- Última aduana antes de enviar la orden al broker.
**Contexto 2: Gestión (Módulo Manage)**
- Define el estado de salud del portafolio y activa protocolos de defensa.

---

---

## Tareas (TTRs)

### **TTR-001: Implementar Jerarquía de Reglas y Veto (Hard/Soft)**
*   **Descripción:** Lógica que impone la soberanía del portafolio sobre las estrategias individuales (ADR-0010).
*   **Reglas de Negocio:**
    * Un `HARD_LIMIT` activa automáticamente un `kill_switch` global sin intervención (ADR-0010).
    * Toda violación debe incluir el `audit_hash` del estado del portafolio en ese nanosegundo.
*   **Entrada:** `rule_config`, `portfolio_state`, `order_intent`.
*   **Salida:** `RuleVerdict` (APPROVED | BLOCKED | VETOED), `reason`.
*   **Precondición:** Estado del portafolio actualizado en `equity-curve-tracker`.
*   **Postcondición:** Registro inmutable del veto en `rule_violations` con `process_id` (ADR-0020 V2).

### **TTR-002: Monitor de Límites en Tiempo Real (SLA < 10ms)**
*   **Descripción:** Evaluación de latencia ultra-baja para respuesta ante brecha de riesgo sistémico.
*   **Reglas de Negocio:**
    * El motor de reglas debe estar pre-compilado nativamente en Rust para evitar overhead en el hot-path.
    * Si la latencia de evaluación excede los 20ms, disparar un `FAIL_SAFE` que pausa la operativa.
*   **Entrada:** `real_time_equity_stream`.
*   **Salida:** `RiskStatusEnum` (HEALTHY | WARNING | CRITICAL).
*   **Precondición:** Motor de reglas inicializado en memoria.
*   **Postcondición:** Notificación enviada vía `notification` feature en caso de `WARNING` o superior.

### **TTR-003: Orquestación del Envolvente (Rules Wrapper & Challenge Mode)**
*   **Descripción:** Inyección y validación dinámica de perfiles de riesgo y reglas de fondeo.
*   **Reglas de Negocio:**
    * Evaluación en tiempo real del *Max Daily Drawdown* (Regla de medianoche: Equity vs Balance de cierre previo) y *Trailing Max Drawdown*.
    * Aplicación de blackouts para noticias macro de alto impacto (ej. NFP, FOMC) y liquidación de fin de semana opcional.
    * Forzado de FIFO / modo netting.
*   **Entrada:** `challenge_profile`, `portfolio_equity_stream`, `news_events`.
*   **Salida:** `RuleVerdict` (APPROVED | BLOCKED | VETOED).
*   **Precondición:** Perfil inyectado al inicio del portafolio.
*   **Postcondición:** Acceso denegado a órdenes que infrinjan restricciones de la fase activa del challenge.

### **TTR-004: Gen de Condición — DD/Volatilidad Agregada de Cartera del Genoma de Portafolio y Correlación (ADR-0108/ADR-0111)**
*   **¿Cuál es el problema?** El Dominio de Portafolio y Correlación necesita leer el estado de salud agregado de la cartera (DD agregado, volatilidad, `RiskStatusEnum`) como Gen de Condición de Estado para sus Genes de Acción de co-evolución, sin que esto debilite la jerarquía de veto FIJA del Rules Wrapper.
*   **¿Qué tiene que pasar?** El `RuleVerdict` y el `RiskStatusEnum` de cada barra se exponen como entradas de solo lectura al motor evolutivo cuando `ACTIVE_GENOME_DOMAINS` incluye Portafolio y Correlación.
*   **¿Cómo sé que está hecho?**
    - [ ] Un Genoma de Portafolio y Correlación activo puede componer una condición que combine `RiskStatusEnum = WARNING` con la correlación de cartera (TTR-002 de [`fit-to-portfolio-search`](./fit-to-portfolio-search.md)) para activar un Gen de Acción de rotación de pesos.
    - [ ] Sin ese genoma activo, `portfolio-rules` opera exactamente como hoy (jerarquía de veto sin cambios).
*   **¿Qué no puede pasar?** El motor evolutivo no puede escribir `RuleVerdict` ni omitir un `BLOCKED`/`VETOED` emitido por esta feature; solo puede leerlo.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda evaluación de reglas registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del veredicto |
| | `created_at` | Timestamp del check de regla |
| | `audit_hash` | Hash del estado del portafolio T-0 |
| | `audit_chain_hash` | Hash del timeline de cumplimiento |
| **II. Soberanía** | `owner_id` | Dueño del entorno |
| | `manifest_id` | ID del contrato de diseño legal |
| | `access_token_id` | Token de autorización de cambios |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del set de reglas activo |
| | `indicator_state_hash` | Veredicto final del Juez de Invariantes |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del monitor de riesgo |
| | `execution_latency_ms` | Latencia del check (< 10ms target) |

---

## Gobernanza y Estándares (Fijos)

- **Decisiones Arquitectónicas Asociadas:**
    - ADR-0010: Reglas Dinámicas (Hard Limits vs Soft Alerts).
    - ADR-0015: Arquitectura de Causalidad (Vetos como evidencia de fallo).
    - ADR-0020 V2: Inundación de Fundaciones.
    - ADR-0079: Rules Wrappers for Portfolios & Universal Rules Injection (Challenge Mode).
- **Genomas Modulares por Dominio (ADR-0108/ADR-0111):** El `RuleVerdict` agregado de cartera (APROBADA/BLOQUEADA/VETADA por DD/Volatilidad) es un Gen de Condición de Estado del Dominio de Portafolio y Correlación. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.

---

## Dependencias
**Depende de:**
- [`order-fsm`](../features/order-fsm.md) — para validación de márgenes de órdenes individuales.
- [`audit-log`](../features/audit-log.md) — para registro inmutable de infracciones.

**Consumido por:**
- [`manage`](../modules/manage.md) — para definición de límites y rebalanceo.
- [`execute`](../modules/execute.md) — para ejecución de checks pre-trade.

