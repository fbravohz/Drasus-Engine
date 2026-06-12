# La Nube Gestionada (SaaS Cloud Engine)

## 1. Visión del Moonshot
El "SaaS Cloud Engine" es la fase comercial definitiva de Drasus Engine. Representa la evolución del sistema desde una herramienta "Local-First" hacia una plataforma de grado institucional gestionada en la nube (SaaS), sin comprometer la filosofía de soberanía del usuario. 

El objetivo es permitir a usuarios sin conocimientos técnicos de infraestructura orquestar miles de simulaciones en paralelo (Minería Genética) utilizando clústeres de servidores dedicados de alta densidad, pagando únicamente una suscripción mensual, mientras mantienen su Interfaz de Usuario local en Flutter para una visualización premium sin latencia de red.

## 2. Arquitectura de Orquestación y Despliegue
Esta fase rompe temporalmente la regla `Zero-Docker` exclusiva para el despliegue del Core en la nube, pero nunca obliga al usuario a instalar contenedores en su máquina local.

### 2.1 Clúster de Alta Densidad (Bare-Metal)
- **Infraestructura Base:** Arrendamiento de servidores dedicados Bare-Metal (Ej. AMD EPYC de 64 Núcleos, 256 GB RAM en Hetzner o OVH). Costo operativo ultra-bajo comparado con AWS EC2 o Serverless, permitiendo un margen de negocio >90%.
- **Aislamiento Ligero:** Cada instancia de Rust Core por usuario se ejecuta dentro de un contenedor Podman/Docker.
- **Asignación Inteligente:** El orquestador limita cada contenedor a (ej. 2 a 4 hilos de CPU y 4GB RAM). Esto permite empaquetar hasta 50 usuarios simultáneos ejecutando simulaciones pesadas en un solo nodo físico.
- **Orquestación en Rust Nativo:** Se descarta Python (Celery/Ray). El flujo de tareas distribuidas se realiza mediante trabajadores en Rust que consumen tareas de colas de mensajes ligeras basadas en memoria compartida u orquestadores eficientes de Rust en red.

### 2.2 Ciclo de Vida del Contenedor (Sin Fricción)
1. **Autenticación (SSO):** El usuario inicia sesión en su App local de Flutter.
2. **Aprovisionamiento:** El Gateway central en Rust verifica la suscripción y envía una orden a la orquestación (Nomad/K3s). Se levanta un contenedor de Rust aislado en segundos.
3. **Conexión Automática:** El contenedor emite credenciales o tokens (gRPC seguro) que se sincronizan con la App local del usuario.
4. **Ejecución Desacoplada:** El usuario puede enviar tareas (ej. Optimización NSGA-II), cerrar la aplicación local, y el demonio Rust en la nube continuará procesando ininterrumpidamente 24/7.

## 3. Estrategia de Persistencia Distribuida (SQLite a S3)
Resolver el problema de estado inactivo de SQLite en sistemas distribuidos es el reto clave.

- **Hot Path (Ejecución):** Mientras el usuario está activo, el contenedor de Rust lee y escribe en el disco NVMe local del servidor (SQLite WAL), garantizando velocidades de procesamiento sub-milisegundo.
- **Cold Storage (S3 Sync):** Se emplea replicación asíncrona por bloques para sincronizar los cambios del WAL hacia almacenamiento de objetos descentralizado (AWS S3, Cloudflare R2).
- **Destrucción y Rehidratación:** Si el usuario cierra su app por más de un tiempo límite, el contenedor se destruye, liberando el espacio NVMe. Al reconectarse, el estado de SQLite se rehidrata instantáneamente desde el almacenamiento de objetos.

## 4. Esquema de Base de Datos Multi-Tenant (SaaS Schema)
El almacenamiento centralizado del SaaS requiere las siguientes estructuras relacionales aisladas por identificador de usuario:

* **Estructura de Usuarios (users):** Gestión de credenciales cifradas, información del plan comercial y niveles de acceso asignados.
* **Estructura de Licencias (licenses):** Registro de firmas criptográficas de hardware (`HMAC-SHA256`) asociadas a cada cuenta de usuario.
* **Estructura de Suscripciones (subscriptions):** Trazabilidad de identificadores de pago (Stripe) y fechas de vencimiento de la facturación.
* **Estructura de Cuotas de Uso (usage_quotas):** Límites diarios y mensuales del consumo de recursos en la nube para optimizaciones y backtests distribuidos.

## 5. Estado de Investigación
*Fase actual: Ideación Teórica (Moonshot).*
No iniciar la codificación de esta infraestructura hasta que el "Core Local FFI" sea 100% estable y genere Alpha real. Es un componente archivado temporalmente hasta finalizar el periodo de prueba de 6 meses (Regla Client Zero).
