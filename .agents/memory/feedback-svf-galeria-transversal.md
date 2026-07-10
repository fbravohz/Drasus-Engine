---
name: feedback-svf-galeria-transversal
description: "SVF y mocks de galería NO dependen de adaptadores diferidos (Cabina de Mando); toda feature incl. plomería pasa por Etapa 0.5 (UI-Designer) y entrega SVF + galería; SVF ≠ galería (se estratifican)."
metadata:
  node_type: memory
  type: feedback
  originSessionId: 121318b9-21ba-4d1e-a76b-a3648b955c7b
---

Corrección del usuario (2026-07-04) sobre cómo trato la UI de verificación de las features del substrato de monetización:

1. **La SVF y los mocks de galería NO dependían nunca del servidor central.** El TL los mezcló mal con el adaptador de red diferido (Cabina de Mando). La **Cabina de Mando Central** es SOLO el servidor del proveedor (autentica/licencia/telemetría/agrega, NUNCA computa) — NO es el motor de Drasus. Lo único que espera a ese sprint lejano es el **cable de red final**; toda la fontanería (puertos, esquema, lógica, cachés, **stubs** que sustituyen al servidor) se conecta YA. La SVF corre contra ese backend local real por FFI; la galería con mocks.

2. **Toda feature —incluida plomería— pasa por fase de diseño (Etapa 0.5, UI-Designer)** para su SVF y su representación en galería con mocks. `central-identity`/`licensing-system` NO tuvieron esa fase (se cerraron con solo CLI Canal #2) → violación de la DoD ([[verification-surface-svf]]) y deuda a saldar. Las entradas/salidas de una feature, aun de plomería, deben **recorrer transversalmente** front→back→DB.

3. **SVF ≠ Galería (no se duplican, se estratifican).** SVF = verifica el comportamiento de una **feature** (JSON entra → respuesta real por FFI sale). Modelo canónico del usuario: tab con selector de feature; izquierda = input block con el JSON precargado; centro = botón enviar; derecha = respuesta del backend en block read-only. Es la gemela GUI de `drasus verify` → conviene un **harness SVF genérico construido una vez**, al que cada feature se enchufa casi gratis. Galería = catálogo de **componentes de UI reutilizables** (inputs, botones, desplegables, nativos clásicos + compuestos estilo Material) con mocks. La SVF está construida CON componentes de galería.

**Why:** el usuario es perfil frontend con tiempo limitado; necesita probar y entender cada feature sin leer código, en la app, no solo en terminal. El CLos (Canal #2) le gusta pero NO reemplaza la SVF; son complementarios.

**How to apply:** de #3 en adelante cada cimiento entrega backend + CLI verify + SVF tab + componentes de galería con mocks; solo se difiere el adaptador de red + la UI productiva cableada. Retroactivo: saldar SVF+galería de #1/#2 en la tanda de UI. Grabado también en `.claude/skills/tech-lead/SKILL.md` (DoD, ~L269). Enlaza con [[verification-surface-svf]], [[galeria-componentes-estado]], [[pricing-foundations-saas]].
