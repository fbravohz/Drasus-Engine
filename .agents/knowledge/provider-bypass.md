# Detección de Bypass del Provider

Cualquier capa de código que defina valores de diseño con literales (hardcoded), en paralelo al theme o provider del sistema, constituye deuda técnica y debe detectarse — principalmente en Flutter/UI.

Slug ID: `PROVIDER-BYPASS`.

Tamaño: `2429 caracteres (~607 tokens aprox., ratio 4:1)`.

---

## PROVIDER-BYPASS-1 — Patrones de Error y Protocolo de Actuación

**Aplica a:** ingenieros de código (Flutter/UI principalmente), TL.

* **PROVIDER-BYPASS-1.1 — Patrones a Detectar:**

  helpers estáticos que devuelven `TextStyle` con `fontSize:` literal sin delegar en el `TextTheme`; colores explícitos (`Colors.*`, `Color(0xFF...)`) en widgets que deberían consumir tokens dinámicos; constantes de espaciado o radio numéricas duplicadas donde ya existe un token (`Gx.space8`, `Gx.rPanel`); lógica de estado o cálculos financieros replicados en la capa de UI (Dart) que deberían provenir de la capa de lógica pura (Rust) mediante el Bridge.
  * ❌ *Incorrecto:* usar `Color(0xFF...)` literal en un widget en vez del token del tema.
  * ✅ *Correcto:* consumir `Theme.of(context).textTheme` o el token `Gx.*` correspondiente.

* **PROVIDER-BYPASS-1.2 — Protocolo al Detectarlo:**

  1. Reportar inmediatamente al usuario citando archivo y línea exacta.
  2. Clasificar la gravedad: 🟡 bypass del provider (el valor debe venir del tema pero no lo hace, corregir de inmediato); 🟠 duplicación de constantes (existe un token de sistema pero se usó un literal, migrar al token); 🔴 lógica de negocio en UI (violó FCIS/Cáscara Delgada, escalar al Tech Lead).
  3. Proponer la solución aplicando el patrón correcto.
  * ❌ *Incorrecto:* corregir en silencio sin reportar ni clasificar la gravedad.
  * ✅ *Correcto:* reportar archivo/línea, clasificar como 🟡/🟠/🔴, y proponer la corrección.

* **PROVIDER-BYPASS-1.3 — Dónde Aplicar por Capa:**

  Flutter usa `Theme.of(context).textTheme`, `DrasusThemeState.*`, `Gx.*`; Rust usa constantes compartidas en `shared/` o config del módulo; Bridge usa contratos tipados, nunca re-derivar tipos en Dart.

**Filtro de verificación:** `PROVIDER-BYPASS-2.1`.

---

## PROVIDER-BYPASS-2 — Filtro de Verificación

**Aplica a:** ingenieros de código (Flutter/UI principalmente), TL, como último paso al tocar código de UI.

* **PROVIDER-BYPASS-2.1:** ¿reporté y clasifiqué (🟡/🟠/🔴) cualquier bypass del provider que encontré, con archivo y línea exactos (`PROVIDER-BYPASS-1`)?
