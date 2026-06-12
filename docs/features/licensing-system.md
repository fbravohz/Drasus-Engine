# Sistema de Licenciamiento (Licensing System)

**Carpeta:** `./features/licensing-system/`  
**Estado:** Lista para implementar  
**Última actualización:** 2026-06-04  
**Decisión Arquitectónica Asociada:** ADR-0020 V2 (Inundación de Fundaciones)  

---

## 1. ¿Qué es esta feature?

El sistema de licenciamiento regula los niveles de acceso del usuario al ecosistema Drasus Engine sin comprometer la privacidad o el rendimiento local. Permite la validación de licencias comerciales y el control del modelo dual (Sovereign Tier y Explorer Tier).

* **Problema:** Los sistemas de licenciamiento tradicionales dependen de telemetría constante y llamadas síncronas de red, violando el principio `Local-First` y exponiendo el sistema a fallos si se pierde la conexión a internet.
* **Comportamiento observable:** El usuario puede usar la plataforma de forma offline. El sistema valida periódicamente el estado de la licencia sin interrumpir las operaciones críticas.
* **Niveles de Licencia (Modelos):**
  * **Sovereign Tier:** Privacidad absoluta. Cero telemetría y cero envío de datos. Requiere validación manual o de latencia extendida.
  * **Explorer Tier:** Licencia de costo reducido a cambio de compartir estadísticas operativas y datos de rendimiento anonimizados del sistema.

---

## 2. Comportamientos Observables

* **Validación de Huella de Hardware:**
  * Al iniciar, la aplicación lee los identificadores físicos de la máquina (placa base, CPU) y genera una firma criptográfica única.
  * Si los identificadores no coinciden con la firma registrada en el archivo local de licencia, el sistema deshabilita las operaciones de trading en vivo y muestra una alerta al usuario.

* **Validación de Heartbeat (Periodo de Gracia):**
  * El sistema permite la operación sin conexión a internet durante un periodo configurable.
  * Al aproximarse al límite sin conexión, la interfaz muestra notificaciones preventivas sugiriendo al usuario una conexión momentánea para el refresco del certificado de la licencia.
  * Si se supera el límite absoluto sin validación, el motor restringe la creación de nuevos backtests y operaciones en vivo hasta que se valide la firma.

---

## 3. Restricciones

* **PROHIBIDO** realizar llamadas síncronas de validación de red en el bucle principal de ejecución de órdenes (*Hot-Path*).
* **PROHIBIDO** almacenar claves privadas de firma de licencias dentro del ejecutable o código fuente del cliente.
* **PROHIBIDO** deshabilitar el funcionamiento del actualizador de emergencia o de la auditoría local si la licencia expira.

---

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| HEARTBEAT_INTERVAL | 90 días | 30 - 360 días | Tiempo límite permitido de ejecución local antes de requerir un refresco en línea. | CONFIG |
| RECHECK_WINDOW | 5 días | 1 - 15 días | Ventana previa al vencimiento del heartbeat donde se inician las alertas visuales en la interfaz. | CONFIG |
| GRACE_PERIOD | 7 días | 0 - 30 días | Días adicionales de ejecución permitida tras vencer el heartbeat antes del bloqueo funcional. | CONFIG |

---

## 5. Estructura Interna (FCIS)

* **Core (Lógica Pura):**
  * Algoritmo de hashing y firma criptográfica para validar el archivo de licencia contra los identificadores de hardware.
  * Comparador determinista de marcas de tiempo y validez del certificado.
* **Shell (Infraestructura):**
  * Lectores de datos físicos del sistema operativo (interfaz con el hardware local).
  * Gestor de persistencia del archivo de licencia en la base de datos local y almacenamiento seguro del sistema.
* **Frontera Pública:**
  * Interfaz de consulta para comprobar la validez de la licencia y el tier activo (Sovereign / Explorer).

---

## 6. Ciclo de Vida de la Feature

### Entrada
* Identificadores crudos del hardware de la máquina local.
* Archivo de licencia firmado criptográficamente.
* Reloj del sistema (validado contra fuentes de tiempo locales protegidas).

### Proceso
* Combina los identificadores de hardware y aplica un algoritmo de firma criptográfica `HMAC-SHA256` utilizando la clave pública incrustada.
* Compara el hash resultante con el contenido en el archivo de licencia.
* Verifica si la fecha actual es menor a la fecha de expiración del heartbeat local.

### Salida
* Veredicto de validación: LICENCIA_VÁLIDA / LICENCIA_INVÁLIDA / REQUIERE_REFRESCO.
* Nivel de acceso autorizado (Sovereign o Explorer).

---

## 7. Tareas (TTRs)

### TTR-001: Generación de Huella Digital de Hardware
* **¿Cuál es el problema?**  
  Necesitamos ligar la licencia a una máquina específica para evitar la clonación no autorizada del software comercial en múltiples servidores, sin violar la privacidad del usuario ni almacenar datos personales.
* **¿Qué tiene que pasar?**  
  El sistema recopila datos de hardware locales estables y genera un hash único empleando un algoritmo `HMAC-SHA256`. Este hash se valida contra el archivo de licencia importado por el usuario.
* **¿Cómo sé que está hecho?**  
  * [ ] El hash se genera de manera idéntica en el mismo equipo en múltiples arranques del sistema.
  * [ ] Si se altera el archivo de configuración de hardware simulado, el sistema detecta el cambio de firma.
* **¿Qué no puede pasar?**  
  * No se pueden transmitir los identificadores de hardware en crudo a ningún servidor externo.

### TTR-002: Verificación de Heartbeat Temporal
* **¿Cuál es el problema?**  
  El software debe verificar periódicamente que la licencia no ha sido cancelada o modificada (ej. reembolsos de Stripe), pero debe hacerlo de forma silenciosa e invisible para no molestar a los usuarios honestos.
* **¿Qué tiene que pasar?**  
  El sistema mantiene una fecha límite en el archivo de licencia local. Si el sistema detecta que la fecha límite se aproxima, activa alertas en la interfaz gráfica. Si se excede el periodo de gracia, suspende operaciones comerciales.
* **¿Cómo sé que está hecho?**  
  * [ ] El sistema inicia alertas cuando el tiempo restante es menor que la ventana de verificación.
  * [ ] El sistema desactiva el trading en vivo al llegar al límite absoluto si no hay conexión para revalidar.
* **¿Qué no puede pasar?**  
  * No se puede bloquear la aplicación de inmediato ante una pérdida repentina de conexión a internet.

---

## 8. Gobernanza y Estándares (Fijos)

* **Local-First (ADR-0016):** 100% Local. La validación se realiza en la máquina del usuario; la red solo se utiliza asíncronamente para refrescar el token de heartbeat.
* **Inundación de Fundaciones (ADR-0020 V2):**
  * **Perfil Ops / Auditoría:** Foco en Identidad del Hardware, Soberanía de los Datos del Cliente y Auditoría Local de Accesos.
  * **Hooks Forenses:** Registro de intentos fallidos de validación de firma de hardware en el log local protegido.
* **Contrato de Persistencia:**  
  Los metadatos de la licencia se guardan cifrados en el almacén local del sistema utilizando claves derivadas de la huella digital.
