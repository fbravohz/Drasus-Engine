# Licenciamiento Comercial e Integración de Pagos (Monetization Stripe)

**Carpeta:** `./moonshots/monetization-stripe/`  
**Estado:** Archivada como Moonshot  
**Última actualización:** 2026-06-04  
**Decisión Arquitectónica Asociada:** ADR-0020 (Inundación de Fundaciones)  

---

## 1. ¿Qué es esta feature?

El sistema de monetización conecta el ecosistema de facturación externa (Stripe) con la estructura de control de accesos del SaaS, regulando qué características funcionales están disponibles para el usuario según su tipo de suscripción activa.

* **Problema:** Los sistemas de cobros tradicionales a menudo acoplan fuertemente el código del negocio con la API de pago, lo que causa fallos si la API de facturación cambia o está inactiva, y expone vulnerabilidades si la validación de límites de cuotas no se realiza en el servidor.
* **Comportamiento observable:** El usuario selecciona un plan comercial en el sitio web o en la interfaz. El sistema genera la pasarela segura. Una vez completado el pago, los límites operativos (cuota de simulaciones genéticas, acceso a datos de ticks, número de brokers conectados) se actualizan dinámicamente en la aplicación del usuario.

---

## 2. Comportamientos Observables

* **Sincronización de Suscripciones vía Webhooks:**
  * Al producirse un pago, renovación o cancelación en la pasarela externa, el servidor de pagos emite un evento firmado criptográficamente.
  * El middleware de monetización valida el origen del evento y actualiza inmediatamente las tablas de suscripciones en la base de datos centralizada.

* **Middleware de Restricción Funcional (Feature Gating):**
  * Cada vez que un usuario intenta invocar un módulo de cálculo intensivo (ej. optimizador Bayesiano o cascada pesada de CPCV), el sistema valida sus límites actuales.
  * Si el plan activo no incluye la característica o se han agotado las cuotas de cálculo genético en la nube del periodo en curso, la acción se bloquea informando detalladamente las opciones de escalado de plan.

---

## 3. Restricciones

* **PROHIBIDO** almacenar claves secretas de pasarelas de pago o certificados privados de webhooks en las instancias del cliente local de escritorio.
* **PROHIBIDO** basar la validación final del estado del plan o cuota de uso exclusivamente en la interfaz gráfica del cliente (toda restricción debe validarse en la base de datos de control del servidor).
* **PROHIBIDO** guardar números de tarjetas de crédito o información financiera confidencial del cliente dentro de la base de datos de Drasus Engine.

---

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| STRIPE_WEBHOOK_TIMEOUT | 10 segundos | 5 - 30 segundos | Tiempo máximo de espera para procesar y confirmar la recepción del evento de pago antes de reintentar. | CONFIG |
| RETRY_ATTEMPTS | 3 | 1 - 10 | Reintentos del proceso de rehidratación de cuotas ante fallos de base de datos tras un pago. | CONFIG |

---

## 5. Estructura Interna (FCIS)

* **Core (Lógica Pura):**
  * Algoritmo de mapeo de planes de pago a límites de uso y permisos funcionales.
  * Verificador del estado del plan temporal y límites acumulados.
* **Shell (Infraestructura):**
  * Manejador de eventos webhooks de Stripe con validación de firmas SSL.
  * Repositorio de persistencia para el esquema relacional de suscripciones y cuotas de uso.
* **Frontera Pública:**
  * Endpoint receptor de notificaciones de la pasarela y consultas del Gateway para control de paso del feature router.

---

## 6. Ciclo de Vida de la Feature

### Entrada
* Eventos firmados de facturación de la pasarela externa.
* Identificadores de usuario y planes comerciales.

### Proceso
* Valida criptográficamente el origen del webhook de pago.
* Identifica la cuenta de usuario vinculada al identificador de Stripe.
* Modifica la tabla de suscripciones y actualiza las cuotas de recursos del plan.

### Salida
* Cuotas de uso actualizadas e inyección del estado en la base de datos local del Gateway.

---

## 7. Tareas (TTRs)

### TTR-001: Procesador de Eventos Webhooks Firmados
* **¿Cuál es el problema?**  
  Los atacantes pueden emular llamadas webhooks de pago falsas para obtener planes premium de forma gratuita si el servidor no valida rigurosamente la firma de cada mensaje.
* **¿Qué tiene que pasar?**  
  El servidor recibe la petición de pago, lee la firma digital adjunta en los encabezados, la compara empleando la clave secreta del webhook configurada de forma segura en las variables del servidor, y procesa la actualización de cuota del cliente si es válida.
* **¿Cómo sé que está hecho?**  
  * [ ] Las peticiones de webhook con firmas falsas o modificadas son rechazadas inmediatamente.
  * [ ] Al procesar una firma correcta de pago de suscripción, el plan del usuario se actualiza a Pro en la base de datos en menos de 5 segundos.
* **¿Qué no puede pasar?**  
  * No se pueden duplicar cuotas de uso o extender periodos de licencia si el webhook emite múltiples llamadas repetidas (procesamiento idempotente).

### TTR-002: Middleware de Feature Gating y Control de Cuotas
* **¿Cuál es el problema?**  
  Los usuarios con planes básicos o gratuitos podrían intentar evadir las restricciones locales llamando directamente a los endpoints gRPC en la nube del clúster de optimización.
* **¿Qué tiene que pasar?**  
  Cada endpoint de optimización distribuye una llamada al middleware para corroborar que el identificador de usuario cuenta con cuota remota disponible antes de admitir la tarea. Si la cuota de tiempo de procesamiento es cero, cancela el enrutamiento.
* **¿Cómo sé que está hecho?**  
  * [ ] Un usuario del nivel básico que intente llamar a optimizadores avanzados recibe un rechazo gRPC inmediato.
  * [ ] El sistema reduce correctamente el crédito disponible de la cuota diaria del usuario a medida que se completan backtests distribuidos.
* **¿Qué no puede pasar?**  
  * La llamada de control de cuotas no debe saturar la latencia de las tareas de ejecución de órdenes en vivo si estas se canalizan por la misma red.

---

## 8. Gobernanza y Estándares (Fijos)

* **Local-First (ADR-0016):** No aplica. Feature de control centralizado de facturación en la nube.
* **Inundación de Fundaciones (ADR-0020):**
  * **Perfil Ops / Auditoría:** Foco en la Soberanía de los Datos Financieros y la Identidad de Facturación.
  * **Hooks Forenses:** Registro de reintentos de webhooks y discrepancias de facturación en la base de datos analítica del backend de la nube.
* **Contrato de Persistencia:**  
  La base de datos relacional de la nube registra de forma permanente las marcas temporales de inicio de ciclo de facturación, cancelaciones y uso diario de cómputo.
