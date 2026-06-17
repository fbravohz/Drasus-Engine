# Plantilla: SAD (Documento de Arquitectura — Lo Fundamental)

**¿Cuándo usar?** Cuando actualizas la visión general del sistema, flujos, o invariantes que duran años.

## Secciones

* **Introducción:** ¿Qué hace el sistema? (1 párrafo)
* **Decisiones Base:** Tabla de ADRs que lo fundamentan
* **Flujos Principales:** Cómo se mueven los datos entre módulos
* **Invariantes:** Lo que NUNCA puede pasar (ej: "margen negativo = error")
* **Propiedades:** Latencia, throughput, disponibilidad

---

## Ejemplo (CORRECTO)

**Sección de Invariantes:**

* **Margen nunca es negativo:** Si una orden llevaría el margen a negativo, se rechaza antes de enviarla al broker. Razón: margen negativo = llamada de margen = fuera de control.

* **Datos sin validar no se usan:** Antes de que cualquier módulo use datos (precios, órdenes, posiciones), pasan por validación. Si fallan, se loguean como anomalía y se descartan. Razón: datos malos contaminan todo (backtests, decisiones, auditoría).

* **Estados son auditables:** Cada cambio de estado (orden PENDIENTE → ENVIADA) se loguea con timestamp. Razón: regulación y debugging.

---

Ver reglas transversales (Lo Prohibido, Regla de Oro, Checklist) en [`TEMPLATES.md`](./TEMPLATES.md).
