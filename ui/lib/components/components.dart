// Barrel de la librería de componentes de UI (ADR-0138 enmienda 2026-06-29).
//
// Este archivo reexporta todos los componentes de 'ui/lib/components/'.
// Los consumidores (galería, tabs de features, Cáscaras Delgadas) importan
// con namespace para evitar colisiones con widgets de Material:
//
//   import 'package:drasus_ui/components/components.dart' as ui;
//
// Uso:  ui.Button(...)  ui.Table(...)  ui.Input(...)  ui.Switch(...)
//
// Convención (FIJO — ADR-0138 §Enmienda 2026-06-29):
//   • Cada componente en su propio archivo snake_case en este mismo directorio.
//   • Nombres de clase desnudos, sin prefijo de estilo (Glow/Glass) ni de
//     proyecto (Drasus). Rename-proof: cambiar el nombre del producto no
//     toca ningún componente.
//   • El estilo lo decide siempre el tema global; el componente consume
//     tokens dinámicos y wrappers de superficie — NUNCA hardcodea color.
//   • Cada componente expone su contrato funcional: callbacks, controllers,
//     value y estados (default/hover/focus/disabled/error/loading).
//   • Este barrel se actualiza en la MISMA Story que añade o toca un componente
//     (Definición de Terminado — ui/COMPONENTS.md).
//
// ─── Batch 1 — Básicos ───────────────────────────────────────────────────────
export 'button.dart';      // Button, ButtonVariant
export 'input.dart';       // Input
export 'dropdown.dart';    // Dropdown<T>, DropdownItem<T>
export 'segmented.dart';   // Segmented
export 'switch.dart';      // Switch
export 'slider.dart';      // Slider
export 'surface.dart';     // Surface
export 'table.dart';       // Table, TableColumn
export 'banner.dart';      // Banner, BannerType
export 'tooltip.dart';     // Tooltip
export 'empty.dart';       // Empty
export 'chip.dart';        // Chip, ChipStatus
export 'badge.dart';       // Badge
export 'card.dart';        // Card
export 'key_value.dart';   // KeyValue
export 'date_picker.dart'; // DatePicker

// ─── Batch 2 — Inputs extendidos ─────────────────────────────────────────────
export 'combobox.dart';      // Combobox<T>, ComboboxItem<T>
export 'multi_select.dart';  // MultiSelect<T>, MultiSelectItem<T>
export 'number_input.dart';  // NumberInput
export 'textarea.dart';      // Textarea
export 'otp_input.dart';     // OtpInput
export 'rating.dart';        // Rating
export 'form_field.dart';    // FormField
export 'mention_input.dart'; // MentionInput
export 'cascader.dart';      // Cascader<T>, CascaderNode<T>

// ─── Batch 3 — Botones/acciones + feedback ────────────────────────────────────
export 'toggle_button.dart';     // ToggleButton
export 'button_group.dart';      // ButtonGroup
export 'fab.dart';               // Fab
export 'split_button.dart';      // SplitButton
export 'notification_card.dart'; // NotificationCard, NotificationCardType
export 'popconfirm.dart';        // Popconfirm
export 'stepper.dart';           // Stepper
export 'accordion.dart';         // Accordion, AccordionItem
// Nota: LoadingButton se consolidó en ui.Button (parámetro loading:true).
// La galería demuestra esta capacidad con un wrapper demo _LoadingButtonDemo.

// ─── Batch 4 — Data display, navegación, pickers, overlays ────────────────────
export 'progress_circular.dart'; // ProgressCircular
export 'tree_table.dart';        // TreeTable
export 'carousel.dart';          // Carousel
export 'pagination.dart';        // Pagination
export 'tree_view.dart';         // TreeView
export 'scrollspy.dart';         // Scrollspy
export 'back_to_top.dart';       // BackToTop
export 'transfer_list.dart';     // TransferList
export 'date_range_picker.dart'; // DateRangePicker
export 'time_picker.dart';       // TimePicker
export 'color_picker.dart';      // ColorPicker
export 'dropzone.dart';   // Dropzone
export 'calendar.dart';   // Calendar
export 'bento_card.dart'; // BentoCard
export 'tabs.dart';       // Tabs, TabItem
export 'dialog.dart';     // Dialog, showAppDialog
export 'sheet.dart';      // Sheet, showAppSheet
