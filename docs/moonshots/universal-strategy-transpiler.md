# Universal Strategy Transpiler

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0101 (Transpilación Basada en Plantillas Tera para Modelos AST)

---

## ¿Qué es?

Permite exportar y traducir de forma nativa la lógica del Grafo de Lógica visual (Strategy AST) de Drasus Engine a múltiples lenguajes de programación y plataformas de trading externas (MQL4/MQL5 para MetaTrader, NinjaScript para NinjaTrader en C#, EasyLanguage/PowerLanguage para TradeStation, y scripts estructurados en Python). Incorpora además un generador automático de manuales y especificaciones técnicas (Documentation Engine) que detalla el comportamiento de la estrategia en lenguaje natural para auditorías y desarrollo manual.

---

## Comportamientos Observables

- [ ] **Exportación Multi-Lenguaje:** El usuario abre el visor de código de una estrategia aprobada, selecciona la plataforma destino (ej: MetaTrader 5) y presiona "Generar Código". El sistema traduce la lógica visual a un archivo `.mq5` compilable.
- [ ] **Paridad de Indicadores:** El transpilador mapea los nodos lógicos de indicadores nativos de Drasus Engine a sus funciones equivalentes estándar en el SDK de destino (ej: `iRSI()` en MQL5 o `RSI()` en NinjaScript).
- [ ] **Generador de Manuales:** Creación de un archivo PDF/Markdown estructurado que explica de forma semántica las reglas de entrada, salida, gestión de riesgo y parámetros configurables de la estrategia.

---

## Restricciones

- **NUNCA** incrustar lógica compleja propietaria de Drasus Engine en el código exportado; la traducción debe valerse estrictamente de las funciones nativas estándar de la plataforma destino.
- **NUNCA** permitir que la transpilación altere el comportamiento matemático del AST (paridad de señales de entrada/salida 1:1 en el backtesting externo).
- **FIJO:** El motor utiliza plantillas Tera para aislar el parser del AST de la sintaxis del lenguaje destino.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| INCLUDE_RISK_LOGIC | true | true / false | Incluye reglas de Stop-Loss y gestión de capital en el código exportado | CONFIG |
| TARGET_PLATFORM_VERSION | mt5 | mt4 / mt5 / ninja8 / python | Versión específica de la API de la plataforma destino | CONFIG |
| AUTO_GENERATE_DOCS | true | true / false | Lanza el Documentation Engine al completar la exportación | CONFIG |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Parser de Árbol de Sintaxis Abstracta (AST Parser) y formateador sintáctico del código de salida.
- **Shell (Infraestructura):** Generador de archivos locales en el disco y motor de renderizado de plantillas.

---

## Ciclo de Vida de la Feature — Strategy Transpiler

### Entrada
- Contrato del Grafo de Lógica visual (JSON AST).
- Selección de plataforma destino y configuraciones de exportación.

### Proceso
- El backend en Rust parsea el JSON AST y valida la integridad de las ramas lógicas.
- Se inyecta el árbol de datos en la plantilla de transpilación correspondiente (Tera).
- Mapeo de indicadores, operadores lógicos y funciones transaccionales.
- Formateo del archivo fuente resultante.

### Salida
- Archivo de código fuente exportado (ej. `.mq5`, `.cs`, `.py`).
- Documento de especificación técnica de la estrategia en Markdown.

---

## Tareas (TTRs)

### **TTR-001: Motor de Transpilación AST a MQL5 / NinjaScript**
*   **¿Cuál es el problema?** Los traders a menudo operan en brokers que no soportan conexiones FFI directas y exigen ejecutar la lógica nativamente en plataformas propietarias como MetaTrader.
*   **¿Qué tiene que pasar?** Desarrollar un parser de AST escrito en Rust que traduzca el Grafo Dirigido Acíclico (DAG) visual a código imperativo estructurado en MQL5 y C#. El parser debe reemplazar los nodos de decisión visuales por estructuras de control de flujo equivalentes (`if-else`) y mapear los parámetros dinámicos a variables configurables (`input` / `extern`).
*   **¿Cómo sé que está hecho?**
    - [ ] El código `.mq5` generado compila en MetaEditor sin advertencias ni errores de sintaxis.
    - [ ] Las señales de compra y venta generadas en MetaTrader coinciden exactamente con las marcas de tiempo obtenidas en el simulador local de Drasus Engine.
*   **¿Qué no puede pasar?**
    - No se deben utilizar indicadores personalizados no incluidos en la plataforma destino a menos que se exporte su lógica matemática explícita en el archivo.

### **TTR-002: Generador de Documentación de Estrategias**
*   **¿Cuál es el problema?** Los desarrolladores y auditores de riesgo necesitan entender la lógica matemática de una estrategia visual sin tener que leer código fuente o inspeccionar grafos enmarañados.
*   **¿Qué tiene que pasar?** Crear un Documentation Engine local que consuma el JSON AST y genere una descripción redactada en lenguaje natural del comportamiento de la estrategia. Debe documentar de forma estructurada los indicadores utilizados, las condiciones de disparo y los parámetros de gestión monetaria.
*   **¿Cómo sé que está hecho?**
    - [ ] Al presionar "Exportar Documentación", se escribe un archivo Markdown local legible.
    - [ ] El texto describe con precisión las condiciones (ej: "Compra si el RSI de 14 periodos cruza por debajo de 30 y el precio está por encima de la media móvil simple de 200 periodos").
*   **¿Qué no puede pasar?**
    - La documentación no debe incluir trazas de código crudas; debe traducirse 100% a explicaciones conceptuales claras.

---

## Gobernanza y Estándares (ADR-0020)

### Perfil Ops / Auditoría
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID del Job de transpilación |
| | `created_at` | Timestamp de exportación |
| | `audit_hash` | Hash SHA-256 del archivo de código fuente exportado |
| **III. Linaje Alpha** | `logic_hash` | Hash del AST visual de origen |
| **IV. Hardware** | `node_id` | Identificador único de la máquina |
| | `process_id` | PID de la tarea de transpilación |
| | `execution_latency_ms` | Tiempo total de parseo y renderizado de plantillas |
