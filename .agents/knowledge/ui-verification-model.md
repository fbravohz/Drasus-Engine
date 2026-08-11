# Modelo de Verificación de UI — SVF y Galería

Cómo se estratifican la Superficie de Verificación Funcional (SVF) y la Galería de componentes, y el Contrato de Integración UI (ADR-0117) que clasifica cada feature entre Superficie propia y plomería.

Slug ID: `UI-VERIFICATION-MODEL`.

Tamaño: `9311 caracteres (~2328 tokens aprox., ratio 4:1)`.

---

## UI-VERIFICATION-MODEL-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[docs/adr/ADR-0117.md]] | Contrato de Integración UI — detalle canónico completo. | Al declarar o auditar "Superficie propia" vs. "Ventana de Verificación" de una feature. | UI-Designer, TL, Flutter-Engineer |
| [[docs/adr/ADR-0137.md]] | Catálogo de tipos de dominio y criterio de plomería (`shared` vs. crate hexagonal). | Al aplicar el test de clasificación de `UI-VERIFICATION-MODEL-2`. | UI-Designer, TL |

---

## UI-VERIFICATION-MODEL-FV — Filtro de Verificación

**Aplica a:** UI-Designer al diseñar la Cáscara Visual, y Tech-Lead al auditar el cierre de una Story.

* **UI-VERIFICATION-MODEL-FV-1:** ¿la SVF y la Galería se diseñaron como capas distintas, sin confundir vocabulario con pantalla cableada? (`UI-VERIFICATION-MODEL-1`)
* **UI-VERIFICATION-MODEL-FV-2:** ¿la feature se clasificó con el test de dos preguntas (tipo de dominio + configuración de usuario), no por intuición? (`UI-VERIFICATION-MODEL-2`)
* **UI-VERIFICATION-MODEL-FV-3:** si la feature tiene Superficie propia, ¿su SVF se entrega en la misma Story que el backend, nunca diferida? (`UI-VERIFICATION-MODEL-3`)
* **UI-VERIFICATION-MODEL-FV-4:** ¿los componentes de galería que la Cáscara referencia existen como widgets funcionales, no solo dibujados? (`UI-VERIFICATION-MODEL-4`)

De ser satisfactoria cada validación de la lista, declara `UI-VERIFICATION-MODEL - FILTRO DE VERIFICACION APROBADO`.

---

## UI-VERIFICATION-MODEL-1 — SVF y Galería No Se Duplican, Se Estratifican

La SVF y la Galería resuelven preguntas distintas sobre la misma UI — no son versiones redundantes una de otra.

**Aplica a:** UI-Designer, Tech-Lead, Flutter-Engineer.

* **UI-VERIFICATION-MODEL-1.1 — Qué Prueba Cada Una:**

  | | **SVF** | **Galería** |
  |---|---|---|
  | Qué prueba | El comportamiento de una feature (backend) | El catálogo de componentes de UI reutilizables |
  | Pregunta que responde | ¿La fontanería funciona? (entra JSON → sale respuesta real) | ¿Qué piezas visuales tengo y cómo se ven/comportan? |
  | Datos | Backend real por FFI | Mocks |

* **UI-VERIFICATION-MODEL-1.2 — La Relación es de Capas, No de Duplicado:**

  la pantalla de la SVF está **construida CON** componentes de la Galería — el input block, el botón enviar, la surface de respuesta **son** componentes de Galería. Galería = **vocabulario** de piezas; SVF = una **pantalla** hecha con ese vocabulario, cableada a un backend real.

* **UI-VERIFICATION-MODEL-1.3 — Patrón Canónico de la SVF:**

  tab con selector de feature → izquierda: input block con el JSON precargado → centro: botón enviar → derecha: respuesta del backend en block read-only. Es la gemela GUI de `drasus verify` (mismo contrato `verify_<feature>`) → conviene un **harness SVF genérico construido UNA vez**, al que cada feature (incluidas las ya cerradas) se enchufa casi gratis.

* **UI-VERIFICATION-MODEL-1.4 — Ninguna de las Dos Depende de Adaptadores Diferidos:**

  ni la SVF ni los mocks de Galería dependen de adaptadores de red diferidos (p. ej. la Cabina de Mando del proveedor, ADR-0143). Se diseñan y corren contra el backend local real. "El adaptador es futuro" nunca es excusa para diferir la SVF o la Galería — la SVF prueba el backend local que sí existe hoy.

**Filtro de verificación:** `UI-VERIFICATION-MODEL-FV-1`.

---

## UI-VERIFICATION-MODEL-2 — Contrato de Integración UI — Superficie Propia vs. Plomería

Toda feature declara en su spec ([[docs/templates/FEATURE.md]] → "Dependencias y Bloqueantes") exactamente una de dos clasificaciones. Si la spec no declara ninguna, el Tech-Lead la completa derivándola con el test de `UI-VERIFICATION-MODEL-2.3`.

**Aplica a:** UI-Designer, Tech-Lead, al clasificar cualquier feature antes de la Etapa 0.5.

* **UI-VERIFICATION-MODEL-2.1 — Superficie Propia:**

  la feature tiene pantalla → activa Etapas 0.5/3/4 (UI-Designer + Bridge + Flutter). Todo feature-node con Superficie propia abre un **Inspector Panel** lateral con su UI (ADR-0136).

* **UI-VERIFICATION-MODEL-2.2 — Ventana de Verificación (Plomería):**

  la feature es plomería sin pantalla propia → NO se despacha UI-Designer/Flutter directamente para ella (correcto y justificado). Su observable (estado, conteo, timestamp, resultado) DEBE quedar registrado como deuda de integración contra la pestaña de una feature consumidora, en `PROGRESS.md`/ROADMAP. Ninguna feature de plomería se sella sin haber registrado su Ventana de Verificación.

* **UI-VERIFICATION-MODEL-2.3 — Test de Clasificación:**

  una feature es plomería (Ventana de Verificación) **solo si** NO produce un tipo de dominio del catálogo ADR-0137 **Y** NO toma configuración del usuario — son los tipos de infraestructura sin puerto de dominio (`clock`, `audit-log`, `telemetry`, etc., ADR-0137 §plomería). Si **produce un tipo de dominio** (`Tick`, `Bars`, `Signal`, `BacktestResult`…) y/o **recibe parámetros del usuario** (símbolo, rango, timeframe, broker), es un nodo del canvas con **Superficie propia = Inspector Panel** — activa Etapas 0.5/3/4, aunque su motor de cálculo ya esté completo en backend.
  * ❌ *Incorrecto:* clasificar como plomería una feature que produce `Tick`/`Bars` y toma configuración de broker/símbolo/fechas, solo porque su parte de backend luce como un motor de descarga.
  * ✅ *Correcto:* aplicar las dos preguntas del test — tipo de dominio producido, configuración de usuario recibida — antes de fijar la clasificación.

**Filtro de verificación:** `UI-VERIFICATION-MODEL-FV-2`.

---

## UI-VERIFICATION-MODEL-3 — Definición de Terminado Depende de la Clasificación

Una feature no se marca `Completado` sin su Superficie de Verificación Funcional (ADR-0117) — es el único canal que permite probar el round-trip front→FFI→back→DB sin leer código.

**Aplica a:** Tech-Lead, al auditar el cierre de cualquier Story.

* **UI-VERIFICATION-MODEL-3.1 — Qué Exige Cada Clasificación:**

  si la feature tiene **Superficie propia** → su tab SVF en el Panel Operativo Fundacional (botón que dispara la operación real vía `public_interface` + resultado real por FFI + observable persistido visible tras recargar), patrón `ui/lib/tabs/clock_tab.dart`. Si es **plomería** → su observable visible en el tab de la feature consumidora declarada (Ventana de Verificación).

* **UI-VERIFICATION-MODEL-3.2 — El Backend en Verde No Es Terminado:**

  la UI de la SVF se entrega en la MISMA Story que el backend — prohibido diferirla a una "Story de UI separada" o al Canvas DAG de una épica posterior (eso es UX de producción, no verificación).

* **UI-VERIFICATION-MODEL-3.3 — Canal #2 No Reemplaza la SVF:**

  el subcomando `drasus verify <feature>` (probador CLI, ADR-0142) es complementario, no sustituto: terminal para el desarrollador, GUI para probar sin leer código. Cerrar una feature con solo el Canal #2 y diferir la SVF es violación de la DoD.

* **UI-VERIFICATION-MODEL-3.4 — Gate de Integración:**

  antes de cerrar una feature consumidora, se revisa si alguna feature de plomería completada previamente la declaró como su Ventana de Verificación. Si la hay, el observable correspondiente DEBE quedar visible en su Cáscara Delgada — si no, la Etapa 4 no está completa.

**Filtro de verificación:** `UI-VERIFICATION-MODEL-FV-3`.

---

## UI-VERIFICATION-MODEL-4 — La Galería Es un Showcase Render-Only, No una Librería Garantizada

Un componente que aparece en el catálogo de Galería no implica que sea usable de inmediato: catálogo ≠ librería usable.

**Aplica a:** Tech-Lead, antes de despachar cualquier Cáscara Visual al Flutter-Engineer.

* **UI-VERIFICATION-MODEL-4.1 — Verificación de Cimiento Obligatoria:**

  antes de despachar, verifica que los componentes que la Cáscara referencia existen como widgets **funcionales** (no solo dibujados): la galería (`ui/lib/gallery/`) es un showcase render-only, y componentes pueden no exponer callbacks/binding (botón sin `onPressed`, dropdown/segmented sin `onChanged`, input sin `controller`) o no existir siquiera como clase. Corre `grep -rnE "class Glow<X>" ui/lib/` (o el prefijo de design-system vigente) para los componentes interactivos clave que la Cáscara necesita.

* **UI-VERIFICATION-MODEL-4.2 — Si Faltan, No Se Despacha:**

  si los componentes faltan o son insuficientes, el Flutter-Engineer NO los reimplementa inline (deriva del design-system) — se escala para extender/construir la librería primero.

* **UI-VERIFICATION-MODEL-4.3 — Bindings FFI Reales, No el Plan de la Orden:**

  verifica también que las notas de implementación del UI-Designer calcen con el binding FFI real (`ui/lib/src/rust/api/<feature>.dart`) — p. ej. no aceptar una nota de polling si el binding generado es `await`.

**Filtro de verificación:** `UI-VERIFICATION-MODEL-FV-4`.
