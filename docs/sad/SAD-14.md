## 14. Glosario Técnico

* **Functional Core:** La lógica pura y determinista del sistema. Sin side-effects, sin I/O. Compatible con optimización vectorial SIMD y compilación Ahead-Of-Time. 
  * *Sinónimos prohibidos:* "Business Logic" (demasiado vago), "Service Layer" (eso es el Shell).

* **Imperative Shell:** Todo lo que no es Core: Controllers, Services de orquestación, Repositories, I/O.
  * *Sinónimos prohibidos:* "Glue Code" (suena despectivo), "Infrastructure" (eso es solo una parte del Shell).

* **Entidad Pura:** Objeto de datos (estructuras de Rust) que representa un agregado del dominio. Nunca un modelo de base de datos ORM físico.
  * *Sinónimos prohibidos:* "DTO" (confunde con patrón diferente).

* **Invariante:** Una regla del dominio que NUNCA puede violarse. Ejemplo: "margen no puede ser negativo".
  * *Sinónimos prohibidos:* "Constraint" (SQL, demasiado físico).

* **Transacción ACID:** Cambio de estado garantizado atómico en persistencia. La capa Service/Repository es responsable.
  * *Sinónimos prohibidos:* "Cambio de estado" a secas (ambiguo si es atomic o no).

* **Máquina de Estados:** Conjunto de situaciones definidas exhaustivamente; cambios entre ellas sólo ocurren cuando se deben.

* **Compilación Automática:** Compilador que convierte código de alto nivel a código máquina (velocidad similar a C).

* **Acceso a Memoria Compartida:** Compartir datos entre partes sin copiar (copiar es costoso).

* **Control de Cambios de Esquema:** Herramienta que versionea cambios de estructura de tablas.

* **Un Binario, Muchos Módulos:** Una sola aplicación ejecutable, múltiples partes independientes, sin latencia de red.

* **Interfaz Pública:** El punto de entrada de cada módulo; única forma que otros módulos lo usan.

---

