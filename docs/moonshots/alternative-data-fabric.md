# Alternative Data Fabric (Evolución SQX Data Manager)

> 🔶 **Especializado por ADR-0125/0126/0127/0128** — El **núcleo determinista** de la indicatorización de datos fundamentales se extrajo a Features de producto: [`fundamental-event-store`](../features/fundamental-event-store.md), [`event-impact-scorer`](../features/event-impact-scorer.md), [`asset-exposure-map`](../features/asset-exposure-map.md) y [`fundamental-indicator-projector`](../features/fundamental-indicator-projector.md) (ver [SAD-21](../sad/SAD-21.md)). Este moonshot conserva únicamente la capa **visual y experimental** (lienzo de nodos, fuentes no estructuradas como sentimiento social y satélite, extracción NLP de texto libre) que se apoya en ese núcleo y permanece en R&D hasta validarse.

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Origen:** Propuesta CPO "Del Archivo Histórico al Alternative Data Fabric" (saas_with_llms_sqx_improvements). NO prioritario retail-first → incubación R&D.

---

## ¿Qué es?

Orquestador visual de **datos alternativos** que reemplaza la simple tabla de símbolos del Data Manager por un lienzo de nodos donde el usuario conecta fuentes no convencionales de alpha: sentimiento social (X/Twitter), datos satelitales de cadenas de suministro, flujos macro. La plataforma alinea automáticamente las marcas de tiempo y normaliza las fuentes (el "trabajo sucio"), de modo que el Quant pueda crear reglas lógicas cruzadas sin escribir código de normalización.

**Por qué es moonshot:** El acceso a datos alternativos institucionales es caro y de nicho; aporta valor R&D pero no es prioritario para el operador individual.

---

## Comportamientos Observables

- [ ] El usuario arrastra un nodo de precio (ej. TSLA) y un nodo de sentimiento social y los conecta en el lienzo.
- [ ] El sistema alinea timestamps y normaliza escalas entre fuentes heterogéneas automáticamente.
- [ ] El usuario crea reglas lógicas cruzadas (ej. "si el sentimiento social cae 20% en 1h pero el precio sube, buscar cortos").

---

## Tareas (TTRs)

### **TTR-001: Orquestador de Alineación Multimodal de Series**
*   **¿Cuál es el problema?** Las fuentes alternativas llegan con frecuencias y husos distintos; alinearlas a mano es tedioso y propenso a leakage.
*   **¿Qué tiene que pasar?** El sistema alinea Point-In-Time las fuentes heterogéneas a una rejilla temporal común sin introducir look-ahead.
*   **¿Cómo sé que está hecho?**
    - [ ] Conecto dos fuentes de distinta frecuencia y obtengo una serie alineada sin huecos lógicos.
*   **¿Qué no puede pasar?** NUNCA introducir look-ahead bias al alinear fuentes de distinta latencia de publicación.

---

## Gobernanza y Estándares (ADR-0020)
- Perfil Datos / Ingest: Identidad + Linaje de Datos + Hardware. Linaje obligatorio de cada fuente alternativa (proveedor, latencia de publicación, licencia).
