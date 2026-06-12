# La Colmena — Red Descentralizada de Minería de Estrategias

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 1 - Investigación de Arquitectura)
**Última actualización:** 2026-05-28

---

## ¿Qué es?

La Colmena es una propuesta de red de cómputo voluntario y distribuido diseñada para acelerar la búsqueda masiva y optimización de estrategias cuantitativas en Drasus Engine. Permite a los usuarios ("Nodos Mineros") instalar un cliente ligero en segundo plano que consume recursos de CPU y GPU inactivos para ejecutar tareas de backtesting y exploración generativa sin exponer la propiedad intelectual ni detalles operativos. Los resultados exitosos se transmiten al servidor central para enriquecer el fondo de estrategias de Drasus Engine, recompensando a los mineros mediante cuotas por cómputo o regalías de ejecución.

---

## Comportamientos Observables

- [ ] **Instalación y Configuración del Minero:** El usuario instala un binario ultra-ligero y asocia su firma digital para recibir incentivos. El cliente opera en segundo plano y de forma silenciosa sin interrumpir la experiencia de usuario clásica.
- [ ] **Orquestación Asíncrona:** El servidor central distribuye "Trabajos de Minería" (definiciones parametrizadas de espacios de búsqueda y regímenes de mercado) de forma asíncrona hacia los clientes activos.
- [ ] **Ejecución Segura en Sandbox:** Las tareas de exploración se ejecutan dentro de un contenedor web inactivo o una máquina virtual segura (Wasm), aislando la computación de los recursos locales del sistema operativo del minero.
- [ ] **Prueba de Trabajo Cuantitativa (Proof-of-Quant):** El servidor central valida los backtests y candidatos reportados re-ejecutando de manera aleatoria un subconjunto de los trades para confirmar la veracidad matemática antes de otorgar incentivos.

---

## Tareas (TTRs)

### **TTR-001: Diseño del Protocolo de Distribución de Trabajos**
*   **Descripción:** Definir el canal de comunicación y esquema de mensajería asíncrona entre el servidor central de orquestación y los clientes mineros dispersos, garantizando la compresión y el uso mínimo de ancho de banda.

### **TTR-002: Sandbox de Ejecución y Aislamiento de IP**
*   **Descripción:** Diseñar el entorno de ejecución aislado (Wasm/Static Engine) para que las máquinas cliente puedan ejecutar backtests e indicadores sin requerir el motor nativo completo de NautilusTrader ni exponer la propiedad intelectual de las librerías propietarias.

### **TTR-003: Validador de Prueba de Trabajo Cuantitativa**
*   **Descripción:** Construir el validador probabilístico en el servidor que recibe las candidatas descubiertas y confirma su veracidad mediante backtests de control con bajo costo de CPU, evitando que nodos maliciosos alteren resultados para recibir pagos ilegítimos.

### **TTR-004: Sistema de Modelos de Recompensas**
*   **Descripción:** Definir y parametrizar los tres flujos de incentivos para los mineros: regalías basadas en fondos activos, cuota fija por flujo de backtesting ejecutado, o pagos por estrategias que superen el score de robustez mínima.

---

## Gobernanza y Estándares (ADR-0020 V2)

Esta característica opera bajo un perfil híbrido de **Datos/Ingest** e **IA/R&D**, requiriendo el registro mandatorio en base de datos de los siguientes campos de inundación de fundaciones:

### 1. Identidad de Origen y Linaje
- **Identificador Único del Minero:** Firma criptográfica asociada al nodo que descubrió el candidato.
- **Identificador de Trabajo:** Hash del trabajo de minería y su parametrización asociada.
- **Score de Confianza de Verificación:** Porcentaje de acierto en las re-ejecuciones de control del validador.

### 2. Recursos y Hardware
- **Tipo de Cómputo Empleado:** Registro de CPU, GPU, o híbrido.
- **Tiempo de Procesamiento Usado:** Duración exacta en milisegundos de la exploración en el nodo.

### 3. Evidencia Causal (Feedback Loop)
- Los datos de rendimiento minado se reportan de manera cifrada al módulo de control de calidad estadístico para catalogar la eficiencia general de la red distribuida.
