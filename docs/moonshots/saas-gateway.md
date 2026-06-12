# Puerta de Enlace SaaS (SaaS Gateway)

**Carpeta:** `./moonshots/saas-gateway/`  
**Estado:** Archivada como Moonshot  
**Última actualización:** 2026-06-04  
**Decisión Arquitectónica Asociada:** ADR-0020 V2 (Inundación de Fundaciones)  

---

## 1. ¿Qué es esta feature?

El Gateway central de acceso regula los flujos de comunicación externa en la nube entre los Thin Clients (Flutter local) y el clúster de ejecución orquestado de Rust Core, centralizando la seguridad y controlando los límites de uso de los endpoints.

* **Problema:** En arquitecturas de nube distribuida, la comunicación sin un nodo centralizado expone directamente los sockets de los workers a la red, facilitando ataques dirigidos, y dificulta la validación uniforme de permisos, control de tasas de peticiones y cuotas asignadas.
* **Comportamiento observable:** El cliente de Flutter local se conecta exclusivamente al dominio del Gateway. El Gateway autentica la sesión, inspecciona la integridad de la petición y enruta internamente el tráfico gRPC hacia el contenedor de ejecución correspondiente en el clúster, sin revelar detalles de la red interna.

---

## 2. Comportamientos Observables

* **Autenticación Multi-Factor y JWT:**
  * Al iniciar sesión, el usuario proporciona credenciales seguras. El Gateway valida el registro e inicia la autenticación de dos factores (TOTP).
  * Tras la verificación exitosa, emite un token criptográfico firmado temporal de acceso junto con su correspondiente token de actualización (Refresh Token).

* **Control de Accesos Basado en Roles (RBAC):**
  * Cada petición enviada por la interfaz incluye el token criptográfico de acceso.
  * El Gateway intercepta la llamada, lee los privilegios asociados y aprueba o deniega el paso hacia los endpoints del motor central basándose en el nivel del plan de suscripción del usuario.

* **Limitación de Tasa Adaptativa (Rate Limiting):**
  * El Gateway mide la frecuencia de llamadas del usuario. Si el volumen excede el umbral configurado para el endpoint específico o para el total de la cuenta, rechaza las llamadas excedentes con un mensaje estandarizado, previniendo abusos y ataques de denegación de servicio.

---

## 3. Restricciones

* **PROHIBIDO** transmitir credenciales de sesión en texto plano (todo canal debe emplear cifrado de transporte TLS).
* **PROHIBIDO** almacenar el estado de la sesión o tokens criptográficos en el Gateway sin cifrado en reposo.
* **PROHIBIDO** omitir la validación de tokens en llamadas internas entre el Gateway y los workers distribuidos del clúster.

---

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ACCESS_TOKEN_EXPIRY | 15 minutos | 5 - 60 minutos | Tiempo de validez del token de acceso criptográfico antes de requerir refresco. | CONFIG |
| REFRESH_TOKEN_EXPIRY | 7 días | 1 - 30 días | Tiempo de validez del token de actualización de sesión antes de forzar el re-login. | CONFIG |
| MAX_REQUESTS_PER_MINUTE | 120 | 10 - 1000 | Límite máximo de peticiones gRPC permitidas por usuario en la ventana temporal. | CONFIG |

---

## 5. Estructura Interna (FCIS)

* **Core (Lógica Pura):**
  * Evaluador de permisos RBAC frente a la matriz de endpoints autorizados.
  * Lógica de generación y decodificación criptográfica de tokens y códigos multifactor.
* **Shell (Infraestructura):**
  * Servidor web gRPC asíncrono basado en Rust.
  * Integrador del sistema de control de tasa en memoria distribuida y consultas al esquema relacional de usuarios.
* **Frontera Pública:**
  * Puertos expuestos al cliente local (Flutter) para el ciclo de autenticación y envío de comandos de simulación.

---

## 6. Ciclo de Vida de la Feature

### Entrada
* Petición entrante del cliente Flutter conteniendo tokens y parámetros.
* Base de datos del SaaS con las cuotas y roles actualizados.

### Proceso
* Intercepta la petición y valida la firma del token criptográfico de acceso.
* Compara el rol del usuario con la política exigida por el endpoint solicitado.
* Comprueba si la tasa de peticiones del usuario está dentro de los límites de consumo del intervalo actual.

### Salida
* Mensaje enrutado hacia el worker de destino / Código de error de autorización o denegación de tasa.

---

## 7. Tareas (TTRs)

### TTR-001: Autenticación gRPC con Validación de Token
* **¿Cuál es el problema?**  
  Los endpoints de trading y simulación en la nube consumen recursos físicos costosos y manipulan datos confidenciales del usuario, por lo que deben protegerse bajo un esquema de autenticación infranqueable de baja latencia.
* **¿Qué tiene que pasar?**  
  El Gateway recibe flujos gRPC seguros del cliente local, valida las firmas de los tokens adjuntos en las cabeceras de metadatos criptográficos, y extrae los permisos y el identificador de usuario para las capas internas.
* **¿Cómo sé que está hecho?**  
  * [ ] Las peticiones sin token o con firmas inválidas se rechazan inmediatamente en la frontera de red.
  * [ ] El retraso añadido por el Gateway para descodificar y validar el token es inferior a una latencia controlada mínima.
* **¿Qué no puede pasar?**  
  * No se pueden aceptar conexiones desde clientes que utilicen esquemas criptográficos obsoletos o comprometidos.

### TTR-002: Middleware de Control de Tasa Dinámico
* **¿Cuál es el problema?**  
  Los bots maliciosos o fallos en los clientes pueden saturar el Gateway con miles de peticiones por segundo, degradando el rendimiento para el resto de los usuarios de la plataforma centralizada.
* **¿Qué tiene que pasar?**  
  El Gateway aplica un filtro que rastrea el consumo del identificador de usuario. Si se supera el límite parametrizado en la ventana temporal, el Gateway retorna un aviso de límite excedido de inmediato sin saturar los recursos del worker.
* **¿Cómo sé que está hecho?**  
  * [ ] Un usuario que exceda el número de peticiones configuradas es suspendido temporalmente en sus llamadas hasta la siguiente ventana limpia.
* **¿Qué no puede pasar?**  
  * El almacenamiento del contador de llamadas no puede causar cuellos de botella en la respuesta general del sistema.

---

## 8. Gobernanza y Estándares (Fijos)

* **Local-First (ADR-0016):** No aplica a esta feature de nube (exclusivo para modelo distribuido comercial SaaS).
* **Inundación de Fundaciones (ADR-0020 V2):**
  * **Perfil Ops / Auditoría:** Enfoque en la Soberanía de los Datos de Acceso, Identidad de Conexión y Hardware del Gateway.
  * **Hooks Forenses:** Registro inmutable de eventos de acceso denegado por IP y usuario con marcas de tiempo.
* **Contrato de Persistencia:**  
  La bitácora de accesos del Gateway se guarda en base de datos en la nube y se procesa para análisis de telemetría de seguridad.
