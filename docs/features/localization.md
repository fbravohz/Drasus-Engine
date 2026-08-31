# Localización

**Carpeta:** `./features/localization/`
**Estado:** En Diseño
**Última actualización:** 2026-07-21
**Decisión Arquitectónica Asociada:** ADR-0160 (Internacionalización desde el Día Uno)

---

## ¿Qué es esta feature?

La capa de **localización** de la interfaz: toda cadena visible al usuario se resuelve por su clave a través de esta capa, nunca escrita literal en un widget. Los idiomas base son **español e inglés**; agregar un idioma nuevo es soltar un archivo de recursos, sin tocar código. Usa el mecanismo estándar de Flutter (archivos de recursos por idioma vía `flutter_localizations`/`intl`).

**Problema que resuelve:** en GREENFIELD es el único momento barato para fijar esto. Retrofittear internacionalización sobre una UI con cientos de cadenas ya incrustadas es masivo y doloroso. El propietario no quiere software clavado a un idioma; quiere facilidad para sumar idiomas, con español e inglés como principales.

---

## Narrativa de Producto

**Capacidad:** `CAP-09` — Una interfaz donde se ve el razonamiento

**Historia de usuario:** Como usuario, quiero usar la aplicación completa en mi propio idioma, y que un mensaje de error del motor me llegue traducido en vez de como un código técnico, para entender el sistema sin tener que aprender su lenguaje interno.

**El problema (técnico):** fijar un idioma único desde el principio significa que cada mensaje queda escrito directamente en el código de la interfaz. Deshacer eso más adelante —cuando ya existen cientos de textos incrustados— es un trabajo enorme y propenso a error; y si el motor interno emitiera directamente prosa en un idioma, ese idioma quedaría cocinado dentro de la lógica misma, imposible de cambiar sin tocar el núcleo.

**La solución (técnica):** toda cadena visible se resuelve por una clave, nunca se escribe directa en la pantalla, y el motor interno solo emite códigos de estado estables — la traducción a prosa ocurre siempre en la capa de interfaz. Cambiar el idioma en los ajustes reescribe toda la pantalla al instante, y agregar un idioma nuevo es soltar un archivo de traducción, sin tocar una sola línea de código.

**Explicado sin tecnicismos:** es como un restaurante que en vez de imprimir el menú directo sobre el papel en un solo idioma, numera cada plato y tiene un libro de traducciones aparte para cada idioma. Cambiar el idioma del comensal es solo entregarle el libro correcto — nunca hay que reimprimir el menú entero.

Aquí pasa igual: cuando el motor dice "ocurrió el error número 47", la interfaz busca ese número en el libro de tu idioma y te lo entrega ya traducido, con palabras que entiendes. Y si mañana alguien quiere sumar el libro en portugués, solo agrega ese libro nuevo — el menú original ni se toca.

---

## Comportamientos Observables

- [ ] El usuario cambia el idioma en ajustes y toda la interfaz cambia de inmediato.
- [ ] Un colaborador agrega un idioma nuevo soltando un archivo de traducción; no toca código.
- [ ] Un mensaje de error del motor aparece en el idioma del usuario, aunque el núcleo solo emitió un código estable.
- [ ] Fechas, números y moneda se muestran según el locale del usuario; el reloj interno y el almacenamiento siguen en UTC y escala entera.

---

## Restricciones

- **NUNCA una cadena visible al usuario se escribe literal en el widget** — siempre se referencia por su clave de localización.
- **NUNCA el backend localiza prosa:** el núcleo Rust emite códigos de error/estado estables; solo la UI convierte código → prosa localizada.
- **SIEMPRE paridad de idiomas base:** una clave nueva nace en español e inglés a la par.
- **Display ≠ storage:** el formato local afecta solo la presentación; no altera la representación interna (UTC, escala entera).

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ACTIVE_LOCALE | es | es / en / (archivos agregados) | Idioma activo de la interfaz | CONFIG |
| BASE_LOCALES | es, en | es, en | Idiomas base mantenidos a la par | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** resolución de clave → cadena según el locale activo; formateo local de fecha/número/moneda.
- **Shell (Infraestructura):** carga de archivos de recursos por idioma; mapeo de códigos de error del núcleo a claves de UI.
- **Frontera Pública:** interfaz para resolver una clave y para cambiar el locale activo.

---

## Ciclo de Vida de la Feature — Localización

### Entrada
- Clave de cadena (o código de error del núcleo) y el locale activo.
- Archivos de recursos por idioma.

### Proceso
- Resuelve la clave contra el archivo del locale activo; formatea valores locales según el locale.

### Salida
- Cadena localizada lista para render; presentación local de fecha/número/moneda.

### Contextos de Uso
- **Transversal a toda la UI:** cualquier feature con superficie propia consume esta capa para su texto.

---

## Tareas (TTRs)

### **TTR-001: Capa de Resolución de Claves y Cambio de Locale**
*   **¿Cuál es el problema?** Sin una capa central, las cadenas se incrustan y el idioma queda hardcodeado.
*   **¿Qué tiene que pasar?** Toda cadena de UI se resuelve por clave; cambiar el locale re-renderiza la interfaz; agregar un idioma es un archivo nuevo.
*   **¿Cómo sé que está hecho?**
    - [ ] Cambiar de español a inglés en ajustes traduce toda la interfaz sin reiniciar.
    - [ ] Un archivo de idioma nuevo (ej. portugués) queda disponible sin cambios de código.

### **TTR-002: Mapeo de Códigos de Error del Núcleo a Prosa Localizada**
*   **¿Cuál es el problema?** Si el backend emitiera prosa, hornearía un idioma en la lógica.
*   **¿Qué tiene que pasar?** El núcleo Rust emite códigos estables; la UI los mapea a claves y los localiza.
*   **¿Cómo sé que está hecho?**
    - [ ] Un error de desbordamiento del transformador aparece en el idioma del usuario; el núcleo solo emitió un código.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `error_code_in` | `Signal` | Input | `0..N` | Códigos de error/estado estables emitidos por el núcleo, a mapear a prosa localizada |

> Plomería de UI transversal, sin puerto de dominio propio en el canvas. **Ventana de Verificación:** el selector de idioma en Ajustes; el observable es que toda cadena visible cambia de idioma y un texto de ejemplo se muestra traducido.

## Cáscara Visual (Thin Shell)

> **Pendiente de la Etapa 0.5** — la produce el skill `ui-designer`. No rellenar a mano.

### Contexto de superficie (ADR-0136)
**Plomería** — selector de idioma en Ajustes; el resto es transversal a toda la UI.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil D (Ops / Auditoría) — Grupo I completo + `process_id`, `node_id`. La preferencia de locale del usuario es un ajuste persistido.

## Dependencias y Bloqueantes
**Consumido por:** todas las features con superficie propia (transversal a la UI).

**Contrato de Integración UI (ADR-0117):**
- **Ventana de Verificación:** selector de idioma en Ajustes; al cambiarlo, toda la interfaz se re-renderiza en el idioma elegido — esa visibilidad es la prueba de funcionamiento de la capa.
