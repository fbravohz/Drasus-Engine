# Librería de Componentes — Drasus Engine UI

> Índice maestro de la librería `ui/lib/components/`.
> **Consultar antes de crear un componente nuevo** — anti-duplicación (ADR-0139).
> Actualizar en la **misma Story** que añade o toca un componente (Definición de Terminado).

---

## Convención canónica (ADR-0138 enmienda 2026-06-29)

### Ubicación y namespace

Cada componente vive en `ui/lib/components/<nombre_snake>.dart`.
El barrel `ui/lib/components/components.dart` reexporta todos.
Los consumidores importan con namespace:

```dart
import 'package:drasus_ui/components/components.dart' as ui;
// Uso:
ui.Button(onPressed: ..., label: 'Confirmar')
ui.Table(columns: ..., rows: ...)
```

### Nombres neutrales

Clases **desnudas**, sin prefijo de estilo (`Glow`, `Glass`) ni de proyecto (`Drasus`).
Las colisiones con widgets Material se resuelven por el namespace `ui.`.
El sistema es **rename-proof**: cambiar el nombre del producto no toca ningún componente.

### Contrato funcional obligatorio

Todo componente expone su contrato de interacción y datos:
- **Callbacks**: `onPressed`, `onChanged`, `onSubmit`, etc.
- **Controllers/value**: `controller`, `value`, `selected`, etc.
- **Estados**: `default / hover / focus / disabled / error / loading` según aplique.

**PROHIBIDO** un componente solo-estético sin binding.

### Estilo por tema global, nunca por el componente

El componente consume tokens dinámicos (`Gx.*`, wrappers de superficie del tema)
y nunca ramifica por modo ni hornea color. Antes de entregar un componente:

```bash
grep -nE "Color\(0x|Glow|Glass|Drasus" ui/lib/components/<archivo>.dart  # debe dar 0
```

El único color literal admisible es `Colors.transparent`.

### Galería = consumidora

La galería (`ui/lib/gallery/`) muestra cada componente con **mocks** (datos y
callbacks simulados). Las features consumen los mismos componentes con datos
reales. La galería NO define componentes; los importa de `ui.`.

---

## Tabla de componentes

> Se llena a partir del Batch 1. Columnas: Componente · Archivo · Propósito · Props/Callbacks · Estados · Consumidores.

| Componente | Archivo | Propósito | Props / Callbacks | Estados | Consumidores |
|---|---|---|---|---|---|
| `Button` | `button.dart` | Botón de acción con gradiente semántico, glow y pulso de luz | `label`, `onPressed`, `variant` (ButtonVariant), `enabled`, `loading` | default, hover, down, loading, disabled | gallery_registry |
| `Input` | `input.dart` | Campo de texto con foco-glow y estado de error | `controller?`, `hint?`, `onChanged?`, `errorText?`, `enabled` | default, focus, error, disabled | gallery_registry |
| `Dropdown<T>` | `dropdown.dart` | Desplegable genérico con panel animado y glow en foco | `value?` (T), `items` (List\<DropdownItem\<T>>), `onChanged?`, `hint?` | closed, open (glow activo), sin-selección | gallery_registry |
| `Segmented` | `segmented.dart` | Control pill de selección única con énfasis dinámico | `options` (List\<String>), `selected?` (int), `onChanged?` | default, active (borde + glow), hover | gallery_registry |
| `Switch` | `switch.dart` | Palanca ON/OFF con knob animado y glow | `value?` (bool), `onChanged?` | OFF, ON (gradiente + glow), animación knob | gallery_registry |
| `Slider` | `slider.dart` | Slider arrastrable con track degradado y knob glow | `value?` (double), `initialValue`, `onChanged?`, `min`, `max` | idle, dragging | gallery_registry |
| `Surface` | `surface.dart` | Wrapper de superficie neutral — delega al modo global del tema | `child`, `padding?`, `radius`, `glow?` | glass / tint / solid / enhancedGlass (auto) | gallery_registry |
| `Table` | `table.dart` | Tabla de datos con cabecera, hover de fila y separadores por token | `columns` (List\<TableColumn>), `rows` (List\<List\<Widget>>), `onRowTap?` | default, row-hover | gallery_registry |
| `Banner` | `banner.dart` | Mensaje contextual con borde semántico izquierdo y glow suave | `message`, `type` (BannerType) | info, success, warning, error | gallery_registry |
| `Tooltip` | `tooltip.dart` | Popup flotante al hover con superficie del tema | `message`, `child` | hidden (default), visible (hover) | gallery_registry |
| `Empty` | `empty.dart` | Estado vacío con orbe latente, mensaje y subtítulo opcional | `message`, `icon?` (IconData), `subtitle?` | — | gallery_registry |
| `Chip` | `chip.dart` | Etiqueta/chip con estado semántico de vitalidad y glow neón | `label`, `status?` (ChipStatus), `pill` | neutro, optima, transition, alert, critical | gallery_registry |
| `Badge` | `badge.dart` | Indicador numérico/etiqueta superpuesto o standalone | `count?` (int), `label?`, `child?` | dot (sin contenido), number, label; standalone / overlay | gallery_registry |
| `Card` | `card.dart` | Tarjeta de contenido genérica con superficie card del tema | `child`, `padding?`, `radius`, `glow?` | glass / tint / solid / enhancedGlass (auto) | gallery_registry |
| `KeyValue` | `key_value.dart` | Fila etiqueta → valor con separador inferior por token de borde | `label`, `value`, `valueColor?` (Color), `mono` | — | gallery_registry |
| `DatePicker` | `date_picker.dart` | Selector de fecha compacto con grilla mensual y navegación | `value?` (DateTime), `onChanged?`, `firstDate?`, `lastDate?` | default, día-seleccionado, día-hover, fuera-de-rango | gallery_registry |
| `Combobox<T>` | `combobox.dart` | Autocomplete genérico: campo de texto + lista de sugerencias filtradas | `value?` (T), `items` (List\<ComboboxItem\<T>>), `onChanged?`, `hint?` | closed, open, filtrado, sin-selección | gallery_registry |
| `MultiSelect<T>` | `multi_select.dart` | Selección múltiple con chips; usa ui.Chip para los seleccionados | `selected?` (Set\<T>), `items` (List\<MultiSelectItem\<T>>), `onChanged?` | default, ítem-seleccionado (chip transition), ítem-disponible (neutro) | gallery_registry |
| `NumberInput` | `number_input.dart` | Campo numérico con botones +/− que respetan rango y paso | `value?` (double), `initialValue?`, `onChanged?`, `min`, `max`, `step` | default, en-mínimo (−deshabilitado), en-máximo (+deshabilitado) | gallery_registry |
| `Textarea` | `textarea.dart` | Campo de texto multilínea con glow de foco y soporte de controller externo | `controller?`, `hint?`, `onChanged?`, `maxLines`, `enabled` | default, focus (glow+borde), disabled | gallery_registry |
| `OtpInput` | `otp_input.dart` | Entrada OTP/PIN de N cajas con avance automático de foco entre dígitos | `length`, `onCompleted?`, `onChanged?` | caja-vacía, caja-con-foco (glow+borde+cursor), caja-rellena | gallery_registry |
| `Rating` | `rating.dart` | Valoración por N indicadores circulares neón; activos en alertAmber | `value?` (int), `onChanged?`, `max` | reposo (borde neutro), activo (fondo tenue + glow alertAmber) | gallery_registry |
| `FormField` | `form_field.dart` | Wrapper de layout: etiqueta + campo hijo + texto de ayuda o error | `label`, `child` (Widget), `errorText?`, `helperText?` | normal (helperText), error (errorText en criticalCrimson) | gallery_registry |
| `MentionInput` | `mention_input.dart` | Campo de texto con detección de @menciones y dropdown de sugerencias | `controller?`, `suggestions` (List\<String>), `onChanged?`, `hint?` | sin-mención (normal), mencionando (dropdown visible) | gallery_registry |
| `Cascader<T>` | `cascader.dart` | Selector jerárquico dos columnas: nivel 1 → nivel 2 (hijos) | `value?` (T), `nodes` (List\<CascaderNode\<T>>), `onChanged?` | reposo, nivel1-activo (énfasis dinámico), hoja-seleccionada | gallery_registry |
| `ToggleButton` | `toggle_button.dart` | Botón ON/OFF con gradiente transitionIndigo y glow; modo controlado y no controlado | `value?` (bool), `onChanged?`, `label`, `labelOff`, `icon?`, `initial` | OFF (surfaceFill+borde neutro), ON (gradiente+glow) | gallery_registry |
| `ButtonGroup` | `button_group.dart` | Fila de botones agrupados con selección única; radio solo en extremos | `items` (List\<String>), `selected?` (int), `onChanged?` | reposo (surfaceFill), activo (gradiente transitionIndigo+glow) | gallery_registry |
| `Fab` | `fab.dart` | Botón circular flotante con gradiente reactor; hover escala 1.05 | `icon` (IconData), `onPressed?`, `tooltip?` | reposo, hover (escala+glow+1.3), deshabilitado (opacity 0.45) | gallery_registry |
| `SplitButton` | `split_button.dart` | Botón dividido: acción principal (gradReactor) + chevron + panel de opciones animado | `label`, `onPressed?`, `actions` (List\<String>), `onActionSelected?` | cerrado, abierto (panel animado) | gallery_registry |
| `NotificationCard` | `notification_card.dart` | Tarjeta con borde semántico izquierdo, punto "no leída" y botón de descarte | `title`, `message`, `type` (NotificationCardType), `read?`, `onTap?`, `onDismiss?` | no-leída (glow+punto), leída (sin glow ni punto) | gallery_registry |
| `Popconfirm` | `popconfirm.dart` | Panel de confirmación inline bajo widget ancla; acción destructiva con gradiente crítico | `message`, `description?`, `confirmLabel`, `cancelLabel`, `onConfirm?`, `onCancel?`, `child` | oculto, visible (panel crítico animado) | gallery_registry |
| `Stepper` | `stepper.dart` | Indicador de N pasos con estados completado/activo/pendiente y barra de progreso | `steps` (List\<String>), `currentStep?`, `onStepTapped?` | completado (optimaCyan), activo (transitionIndigo+glow), pendiente (muted) | gallery_registry |
| `Accordion` | `accordion.dart` | Lista de secciones plegables; una abierta a la vez con cuerpo animado | `items` (List\<AccordionItem>), `openIndex?`, `onChanged?` | cabecera-activa (surfaceRaised+borde semántico), cabecera-cerrada (surfacePanel+borde neutro) | gallery_registry |

---

## Roadmap de batches

| Batch | Componentes | Estado |
|---|---|---|
| **0 — Cimiento** | Estructura de directorios, barrel, rename de tema, este índice | ✅ 2026-06-29 |
| **1 — Básicos (16)** | Button, Input, Dropdown, Segmented, Switch, Slider, Surface, Table, Banner, Tooltip, Empty, Chip, Badge, Card, KeyValue, DatePicker | ✅ 2026-06-29 |
| **2 — Inputs extendidos (9)** | Combobox, MultiSelect, NumberInput, Textarea, OtpInput, Rating, FormField, MentionInput, Cascader | ✅ 2026-06-29 |
| **3 — Botones/acciones + feedback (9)** | ToggleButton, LoadingButton→Button, ButtonGroup, Fab, SplitButton, NotificationCard, Popconfirm, Stepper, Accordion | ✅ 2026-06-30 |
| **4 — Data display + nav + pickers + overlays (17)** | ProgressCircular, TreeTable, Carousel, Pagination, TreeView, Scrollspy, BackToTop, TransferList, DateRangePicker, TimePicker, ColorPicker, Dropzone, Calendar, BentoCard, Tabs, Dialog, Sheet | Pendiente |

---

*Última actualización: 2026-06-30 — Batch 3 (STORY-025)*
