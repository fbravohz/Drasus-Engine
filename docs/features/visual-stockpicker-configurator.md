# Visual StockPicker Configurator

**Carpeta:** `./features/visual-stockpicker-configurator/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión)

---

## ¿Qué es esta feature?

El `Visual StockPicker Configurator` es la interfaz gráfica que permite al operador configurar los criterios de selección y rotación del universo de activos (equities). Provee deslizadores (sliders) e interruptores interactivos para establecer filtros basados en volumen, capitalización de mercado, volatilidad y fundamentales básicos, traduciendo la configuración visual a reglas de ingesta y filtrado en el core.

---

## Comportamientos Observables

- [ ] El usuario interactúa con los sliders en la UI para ajustar umbrales del universo:
  - Capitalización de mercado mínima/máxima.
  - Volumen promedio diario de transacciones (ADTV).
  - Rango de precios admitido.
- [ ] La UI actualiza de forma instantánea el número de activos que cumplen las condiciones en el universo actual.
- [ ] Permite guardar y cargar plantillas de universos definidos (ej: "S&P 500 Líquido", "Small Caps de Alta Volatilidad").

---

## Restricciones

- **NUNCA** realizar el filtrado de miles de activos en el hilo principal de Flutter; el motor de base de datos DuckDB/Polars realiza la consulta en caliente y envía el conteo de activos coincidentes al frontend.
- **NUNCA** permitir la ejecución de backtests de portafolios sin haber especificado un universo de activos validado.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DYNAMIC_RECONCILIATION | true | true/false | Actualiza el conteo de activos coincidentes automáticamente en cada cambio de slider | CONFIG |
| MAX_UNIVERSE_ASSETS | 1000 | 100 - 5000 | Límite máximo de activos permitidos en el universo activo | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmo de parseo de reglas de filtrado de universos a sintaxis SQL de DuckDB.
- **Shell (Infraestructura):** Consultor de catálogos y metadatos de activos en el módulo Ingest utilizando DuckDB.
- **Frontera Pública:** Contrato de consulta de universos que acepta reglas de filtrado y retorna la lista y conteo de activos elegibles.

---

## Ciclo de Vida de la Feature — Visual StockPicker Configurator

### Entrada
- Catálogo maestro de activos y sus metadatos históricos (precios, volúmenes, fundamentales).
- Reglas y parámetros de filtrado establecidos por el usuario en la UI.

### Proceso
- Traduce los rangos de los sliders a cláusulas WHERE de SQL.
- Consulta DuckDB y Polars para aislar la lista de símbolos coincidentes.

### Salida
- Conteo cuantitativo de activos coincidentes y lista de símbolos resultantes.

---

## Tareas (TTRs)

### **TTR-001: Motor SQL de Filtrado de Universos (Rust)**
*   **¿Cuál es el problema?** Conciliar filtros de múltiples variables sobre miles de activos financieros en tiempo real puede introducir retraso en la interfaz gráfica.
*   **¿Qué tiene que pasar?** Implementar consultas parametrizadas dinámicas en DuckDB que ejecuten agregaciones rápidas sobre los metadatos de activos.
*   **¿Cómo sé que está hecho?**
    - [ ] DuckDB procesa las consultas de filtrado de universos en menos de 2ms.

### **TTR-002: Panel de Configuración de Universos (Flutter)**
*   **¿Cuál es el problema?** Ajustar múltiples umbrales numéricos mediante cuadros de texto tradicionales es propenso a errores y poco intuitivo.
*   **¿Qué tiene que pasar?** Crear un panel visual en Flutter con sliders fluidos e indicadores interactivos que reporten el tamaño dinámico del universo resultante.
*   **¿Cómo sé que está hecho?**
    - [ ] Al arrastrar el slider de volumen, el número de activos del universo se actualiza fluidamente a 60 FPS.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil Datos / Ingest. Registra `data_snapshot_id` (linaje de origen), `audit_hash`, `node_id`.
- **Rastro de Evidencia:** Emite cambios en la definición del universo para el módulo de `feedback`.

---

## Dependencias
- **Depende de:** `/features/databank-manager.md`, `/features/duckdb-sql-engine.md`
- **Bloquea:** `/modules/ingest.md`
