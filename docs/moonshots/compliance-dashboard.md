# Compliance Dashboard

**Carpeta:** `./moonshots/compliance-dashboard/`
**Estado:** Incubación
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión), ADR-0093 (Arquitectura de Seguridad Soberana)

---

## ¿Qué es esta feature?

El `Compliance Dashboard` es un panel de auditoría diseñado para cumplir de forma proactiva con los estándares futuros de la regulación de Inteligencia Artificial (específicamente la EU AI Act). Centraliza las bitácoras de auditoría de los modelos neuronales, documenta el linaje de los datos sanitizados utilizados y certifica la transparencia de las decisiones automatizadas.

---

## Comportamientos Observables

- [ ] El usuario visualiza los registros inmutables de auditoría de la IA (modelos generativos, hiperparámetros de Monte Carlo y pesos de redes neuronales profundas).
- [ ] Exporta reportes firmados criptográficamente que detallan que las estrategias no incurren en prácticas prohibidas o manipulación de mercado.
- [ ] Visualiza los certificados de linaje de datos de cada estrategia promovida.

---

## Restricciones

- **NUNCA** permitir la alteración o purga de los registros de auditoría de cumplimiento; la tabla en SQLite es estrictamente inmutable y de solo inserción (Append-Only).
- **NUNCA** enviar reportes a entidades gubernamentales automáticamente sin la autorización e intervención manual del usuario.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| COMPLIANCE_SEVERITY_LEVEL | HighRisk | Minimal/Medium/HighRisk | Nivel de estrictez de auditoría de modelos a aplicar | CONFIG |
| CRYPTO_SIGN_REPORTS | true | true/false | Firma de reportes con claves criptográficas locales | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de validación de hashes concatenados de auditoría y firma criptográfica de reportes.
- **Shell (Infraestructura):** Generadores de archivos de log estructurados y accesos cifrados en SQLite WAL.
- **Frontera Pública:** Interfaz de auditoría de cumplimiento que valida la integridad de los datos históricos de generación.

---

## Tareas (TTRs)

### **TTR-001: Auditor de Integridad Forense (Rust)**
*   **¿Cuál es el problema?** Validar que la bitácora de auditoría no ha sido manipulada localmente requiere encadenamiento criptográfico estricto.
*   **¿Qué tiene que pasar?** Rust recorre los registros verificando el hash-chain (`audit_chain_hash`) de cada fila contra el anterior.
*   **¿Cómo sé que está hecho?**
    - [ ] El verificador detecta cualquier alteración maliciosa en las filas de la base de datos.

### **TTR-002: Exportador de Declaraciones de Conformidad**
*   **¿Cuál es el problema?** Proveer a terceros evidencia verificable del entrenamiento soberano del modelo requiere firmas inmutables.
*   **¿Qué tiene que pasar?** Firmar criptográficamente el manifiesto del modelo utilizando claves locales del usuario.
*   **¿Cómo sé que está hecho?**
    - [ ] El sistema genera una firma válida del manifiesto de la estrategia.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil Ops / Auditoría. Registra `id`, `created_at`, `audit_hash`, `compliance_status_id`.
- **Rastro de Evidencia:** Emite veredictos de conformidad para el módulo de `feedback`.
