# Prop-Firm Grader (El Filtro de Fondeo)

**Carpeta:** `./features/prop-firm-grader/`
**Estado:** En Diseño
**Decisiones Arquitectónicas Asociadas:** ADR-0045, ADR-0020 V2

---

## ¿Qué es? (Explicado Simple)

Es un **verdugo implacable**. Las firmas de fondeo modernas (como FTMO o TopStep) tienen reglas clarísimas y muy estrictas: si pierdes más de un 5% en un solo día, o un 10% en total, te quitan la cuenta y pierdes tu dinero.

El **Prop-Firm Grader** opera el **Embudo Tóxico de Estrés (ADR-0061)**. Vigila a las estrategias durante su validación Monte Carlo. Si una mutación de la estrategia toca, aunque sea por un milisegundo, uno de esos límites diarios absolutos (ej. Drawdown > 4.5% Intradiario), el sistema la "mata" automáticamente marcándola como RECHAZADA. 

**La magia (Cero Código Quemado):** El sistema no sabe qué es "FTMO" o "TopStep" internamente. Todo se inyecta desde una configuración (configuración tipada validada en Rust / Serde), por lo que mañana puedes agregar las reglas de una firma nueva de Dubai sin tocar una sola línea de código fuente.

---

## Comportamientos Observables (La Regla de Nulidad)

- **Vigilancia al Vuelo:** Mientras la estrategia simulada opera en el motor Monte Carlo, evalúa el Drawdown Diario absoluto.
- **Muerte Súbita (Regla de Nulidad Intransigente):** Si el drawdown intradiario excede el límite del perfil, el veredicto cambia a `RECHAZADA` de forma inmediata. No hay indulgencias estadísticas ni promedios temporales macro.
- **Filtrado de Cohortes:** En una corrida de 10,000 mutaciones, si el porcentaje de supervivencia diaria es < umbral (ej. 90%), la estrategia falla el test tóxico.

---

## Parámetros Configurables (Configuración Tipada Serde)

Se inyecta mediante el objeto `PropFirmComplianceConfig`. Aquí algunos perfiles de ejemplo (configurables en JSON/YAML):

| Parámetro | Perfil FTMO | Perfil TopStep | Qué Mide |
|---|---|---|---|
| `profit_factor_threshold` | 1.30 | 1.30 | Ganancia vs Pérdida Bruta Mínima |
| `max_daily_drawdown_pct` | 5.0 | 4.0 | El límite de dolor en un solo día (El Asesino de Cuentas) |
| `max_total_drawdown_pct` | 10.0 | 8.0 | Límite máximo de pérdida antes de quemar la cuenta |

---

## Tareas (TTRs)

### TTR-001: Implementación del Evaluador Intransigente
- **Descripción:** Crear el evaluador `PropFirmGrader` que reciba `PropFirmComplianceConfig` como inyección de dependencias. Debe tener la autoridad para abortar una validación en progreso apenas se rompa una regla de fondeo (Short-Circuit Evaluation), ahorrando ciclos de CPU.
- **Criterio de Éxito:** Cambiar entre el perfil de FTMO y TopStep desde el archivo `.env` sin modificar código Rust, logrando que estrategias agresivas sean rechazadas más rápido bajo TopStep.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2): Perfil D (Ops / Auditoría)** — calificación de cumplimiento de prop-firm.
    - **I. Identidad & Integridad (Grupo I completo):** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`.
    - **IV. Infraestructura & Ops:** `node_id`, `process_id`.
    - **V. Forense (Gobernanza):** `compliance_status_id` (veredicto final), `risk_audit_id` (razón del rechazo si aplica), `portfolio_container_id` (cuenta de fondeo evaluada).
- **Dependencias:** Utilizado primordialmente en `validate`, `execute` y `withdraw`.
