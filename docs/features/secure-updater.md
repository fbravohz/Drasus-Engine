# Actualizador Seguro (Secure Updater)

**Carpeta:** `./features/secure-updater/`  
**Estado:** Lista para implementar  
**Última actualización:** 2026-06-04  
**Decisión Arquitectónica Asociada:** ADR-0020 (Inundación de Fundaciones)  

---

## 1. ¿Qué es esta feature?

El actualizador seguro gestiona el ciclo de vida de las actualizaciones de software del núcleo binario de Rust y la interfaz gráfica de Flutter, garantizando que el usuario ejecute siempre la versión estable y libre de vulnerabilidades, de manera segura y sin intervención manual compleja.

* **Problema:** Las actualizaciones completas de aplicaciones financieras requieren descargas masivas de cientos de megabytes que aumentan los costos de red, y los instaladores tradicionales son vulnerables a ataques de secuestro de tráfico (Man-in-the-Middle) para inyectar ejecutables maliciosos.
* **Comportamiento observable:** El usuario ve una alerta en la interfaz indicando la existencia de una nueva versión. Al aceptar, el sistema descarga únicamente el parche con las diferencias entre la versión actual y la nueva, valida las firmas y aplica los cambios instantáneamente reiniciando la aplicación.
* **Mecanismos clave:**
  * **Actualizaciones Diferenciales (Differential Patches):** Descarga exclusiva de bytes modificados para reducir el uso de red.
  * **Firma Criptográfica Ed25519:** Verificación matemática de la autenticidad del binario descargado antes de cualquier ejecución en el disco duro.

---

## Narrativa de Producto

**Capacidad:** `CAP-09` — Una interfaz donde se ve el razonamiento

**Historia de usuario:** Como usuario, quiero que las actualizaciones del sistema lleguen ligeras y verificadas, para no descargar cientos de megabytes cada vez ni arriesgarme a instalar algo que alguien manipuló en el camino.

**El problema (técnico):** las aplicaciones financieras suelen requerir descargas completas de cientos de megabytes en cada actualización, lo que además de costoso en ancho de banda, abre una ventana de ataque: un instalador tradicional es vulnerable a que alguien intercepte el tráfico y sustituya el binario legítimo por uno malicioso antes de que llegue a la máquina del usuario.

**La solución (técnica):** el sistema descarga solo la diferencia de bytes entre la versión actual y la nueva —nunca el paquete completo—, valida la firma criptográfica del binario resultante contra la clave autorizada antes de ejecutar nada, y si la firma no coincide, destruye el archivo descargado y aborta sin tocar la instalación existente. La aplicación guarda su estado antes de reiniciar en la nueva versión.

**Explicado sin tecnicismos:** imagina que en vez de mandarte un mueble armado completo cada vez que cambia una pieza, te mandan solo el tornillo que cambió, con una firma notariada que certifica que ese tornillo salió realmente del fabricante. Si la firma no cuadra, ni siquiera lo desempacas.

Así actualiza Drasus: baja solo lo que cambió, revisa que la firma sea auténtica antes de instalar nada, y si algo no cuadra, se detiene en seco y avisa — nunca deja pasar un archivo sospechoso solo por ahorrar tiempo.

---

## 2. Comportamientos Observables

* **Detección de Nuevas Versiones:**
  * El actualizador consulta de forma asíncrona un manifiesto firmado en el servidor de distribución.
  * Si detecta una versión superior compatible, calcula el tamaño del parche diferencial estimado.

* **Validación de Integridad y Firma:**
  * Al completarse la descarga del parche, el actualizador aplica la verificación de la firma pública `Ed25519` incrustada de origen en el binario local.
  * Si la firma no coincide o la suma de verificación del archivo resultante falla, el actualizador destruye inmediatamente el archivo descargado, aborta el proceso y emite una alerta de seguridad de alta prioridad en los registros locales del sistema.

* **Aplicación de Parches en Caliente (Hot patching):**
  * La aplicación de escritorio se cierra ordenadamente salvando el estado transaccional en la base de datos local SQLite (WAL).
  * El cargador del actualizador reemplaza los ejecutables locales y reinicia el sistema en la nueva versión en menos de un tiempo límite predefinido.

---

## 3. Restricciones

* **PROHIBIDO** aplicar actualizaciones que no estén firmadas con la clave autorizada de desarrollo.
* **PROHIBIDO** descargar o ejecutar archivos en directorios temporales fuera del área reservada de la aplicación sin permisos del sistema operativo.
* **PROHIBIDO** interrumpir operaciones de trading en vivo en ejecución para forzar una actualización (las actualizaciones se encolan para el siguiente arranque o cierre de operaciones).

---

## 4. Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| CHECK_FREQUENCY | 24 horas | 1 - 168 horas | Frecuencia de consulta del manifiesto de actualizaciones en el servidor. | CONFIG |
| MAX_UPDATE_TIMEOUT | 30 segundos | 10 - 120 segundos | Límite máximo de espera para completar la descarga del parche diferencial antes de cancelar. | CONFIG |
| FORCE_CRITICAL_UPDATES | true | true / false | Determina si las actualizaciones de parches de seguridad críticos bloquean el uso hasta aplicarse. | CONFIG |

---

## 5. Estructura Interna (FCIS)

* **Core (Lógica Pura):**
  * Motor de verificación de firmas criptográficas Ed25519.
  * Algoritmo de reconstrucción y parcheado de archivos binarios utilizando diferencias de bytes.
* **Shell (Infraestructura):**
  * Cliente de red para la descarga asíncrona de parches.
  * Gestor de archivos para la sustitución de ejecutables y control del proceso de reinicio del sistema operativo.
* **Frontera Pública:**
  * API para iniciar comprobaciones manuales, recibir progresos de descarga y disparar la secuencia de instalación.

---

## 6. Ciclo de Vida de la Feature

### Entrada
* Binario actual en ejecución.
* Manifiesto de actualización firmado.
* Clave pública Ed25519 integrada en el sistema local.

### Proceso
* Descarga el manifiesto y valida su firma.
* Compara el hash del binario local con la tabla de diferencias.
* Descarga las diferencias binarias crudas.
* Aplica el algoritmo de parcheado para generar el nuevo binario.
* Verifica la firma Ed25519 sobre el nuevo binario resultante en disco.

### Salida
* Nuevo binario validado listo para su ejecución.
* Estado del proceso: ACTUALIZADO / ERROR_DE_FIRMA / SIN_CAMBIOS.

---

## 7. Tareas (TTRs)

### TTR-001: Verificación de Firma Digital Ed25519
* **¿Cuál es el problema?**  
  Los ejecutables descargados pueden ser manipulados por atacantes de red para infectar la máquina del usuario final con malware o código de robo de credenciales.
* **¿Qué tiene que pasar?**  
  El actualizador calcula la firma digital del binario resultante y la valida contra la firma declarada en el manifiesto oficial utilizando una curva criptográfica segura de clave pública. Si falla, el binario se descarta.
* **¿Cómo sé que está hecho?**  
  * [ ] Si se introduce un solo byte de modificación en el archivo descargado, el sistema falla la validación de firma y no lo ejecuta.
  * [ ] El actualizador aprueba ejecuciones de archivos firmados únicamente con la clave privada de producción correspondiente.
* **¿Qué no puede pasar?**  
  * No se puede delegar la validación al navegador web o a certificados TLS de red únicamente; la validación debe ocurrir a nivel de archivo.

### TTR-002: Reconstrucción Diferencial de Binarios
* **¿Cuál es el problema?**  
  Descargar 100MB+ en cada actualización de parche menor es ineficiente y problemático para usuarios con conexiones inestables.
* **¿Qué tiene que pasar?**  
  El sistema descarga un archivo que contiene exclusivamente la diferencia de bytes (delta). El actualizador lee el binario local, le aplica el delta de bytes y reconstruye el nuevo ejecutable localmente.
* **¿Cómo sé que está hecho?**  
  * [ ] El tamaño de descarga de un parche menor es inferior al 10% del binario completo.
  * [ ] El binario reconstruido localmente coincide bit a bit con el binario completo distribuido en el servidor de origen.
* **¿Qué no puede pasar?**  
  * No se puede generar un archivo corrupto que deje la aplicación inutilizable (se debe mantener un respaldo del binario anterior hasta que el nuevo inicie con éxito).

---

## 8. Gobernanza y Estándares (Fijos)

* **Local-First (ADR-0016):** 100% Local en su lógica de reconstrucción e instalación. Las peticiones de descarga se realizan asíncronamente.
* **Inundación de Fundaciones (ADR-0020):**
  * **Perfil D (Ops / Auditoría):** actualizador firmado; lo relevante es la trazabilidad de auditoría de cada parche y su firma, no la latencia del hot-path de mercado.
  * **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
  * **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`.
  * **IV. Infraestructura & Ops:** `node_id`, `process_id`, `session_id`.
  * **V. Auditoría (Gobernanza):** `signature_hash` (firma criptográfica del binario descargado/parche; integridad cripto), `risk_audit_id`.
  * **Hooks de Auditoría:** Registro de marcas de tiempo del inicio y finalización del parcheado, sumas de comprobación previas y posteriores, y resultados de firma en el registro local.
* **Contrato de Persistencia:**  
  SQLite almacena el historial de versiones aplicadas con el Grupo I completo + Perfil D arriba (incl. `signature_hash` para la auditoría de firmas).
