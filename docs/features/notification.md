# Notification — Abstracción de Canales de Notificación

**Carpeta:** `./features/notification/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

---

## ¿Qué es?

Abstrae canales de notificación (email, webhook, Slack, SMS). El Core dispara eventos sin saber por qué canal se enviarán.

---

## Comportamientos Observables

- [ ] Execute dispara evento "KILL_SWITCH_ACTIVATED"
  → Notification enruta a: email al admin, Slack al channel #trading, webhook a URL configurada
  → Usuario recibe múltiples notificaciones simultáneamente

- [ ] El usuario configura canal Slack
  → Siguiente evento se envía automáticamente a Slack (sin cambiar código)

---

## Restricciones

- **NUNCA notificación sin destinatarios.**
- **NUNCA se logguean datos sensibles en canal público.**

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| CHANNELS | email | email / slack / webhook / sms | Canales activos |

---

## Tareas (TTRs)

### **TTR-001: Enviar notificación a destinatarios**

**Qué hace:** Envía notificación a uno o más destinatarios a través de canal(es) configurado(s).

**Entrada:**
- Tipo de notificación (cadena)
- Payload (contenido)
- Lista de destinatarios

**Salida:**
- Bool: envío exitoso

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (Orquestación de envíos). El mensaje se genera localmente.
- **Inundación de Fundaciones (ADR-0020):** 
    - Aplica el **Grupo I (universal)** + solo los campos de su Perfil Técnico listados abajo (Filtro de Relevancia, ADR-0020); NO el catálogo completo de 25 campos.
    - Metadatos de comunicación: `audit_chain_hash` (Secuencia de alertas), `logic_hash` (Template version), `event_sequence_id`.
    - Soberanía & Destreza: `owner_id`, `access_token_id`.
    - Integridad: `node_id`, `process_id`, `signature_hash`.

- **Rastro de Evidencia:** El registro inmutable de qué notificaciones críticas se enviaron y si fueron recibidas.

---

## Dependencias

**Depende de:**
- Ninguna

**Depende de ella:**
- `execute` (alertas de kill switch)
- `portfolio-rules` (notificaciones de reglas SOFT)
- `feedback` (reportes de anomalías)
