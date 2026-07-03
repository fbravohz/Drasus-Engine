// form_field.dart — Componente FormField (ADR-0138 enmienda 2026-06-29).
// Wrapper de etiqueta + campo + texto de ayuda / error. No incluye lógica de validación.
// El estilo lo decide el tema global vía tokens Gx; prohibido hardcodear color.
//
// Nota: el nombre FormField colisiona con el widget Material del mismo nombre.
// Los consumidores importan con namespace `import ... as ui;` → `ui.FormField`.
// Dentro de este archivo se oculta el FormField de Material con `hide FormField`.

// ignore: undefined_hidden_name — FormField se oculta para evitar la colisión de nombres.
import 'package:flutter/material.dart' hide FormField;
import '../theme/gx_tokens.dart';

// Wrapper de campo de formulario con etiqueta superior y texto de ayuda/error inferior.
// Contrato funcional:
//   [label]      etiqueta visible encima del campo.
//   [child]      widget del campo (cualquier Input, Combobox, etc.).
//   [errorText]  mensaje de error bajo el campo (reemplaza a helperText si hay error).
//   [helperText] mensaje de ayuda bajo el campo en estado normal.
// Este wrapper es puramente de layout: no gestiona estado de validación propio.
// La lógica de error/ayuda viene del padre, que puede pasar ambos y seleccionar
// cuál mostrar según su estado. Si [errorText] no es null, tiene prioridad visual.
class FormField extends StatelessWidget {
  final String label;
  final Widget child;
  final String? errorText;
  final String? helperText;

  // No es const: lee getters dinámicos de Gx que cambian con el tema.
  FormField({
    super.key,
    required this.label,
    required this.child,
    this.errorText,
    this.helperText,
  });

  @override
  // Columna: etiqueta → campo → texto de ayuda o error.
  Widget build(BuildContext context) {
    final hasError = errorText != null && errorText!.isNotEmpty;
    final subText = hasError ? errorText : helperText;
    final subColor = hasError ? Gx.criticalCrimson : Gx.textBaseMuted;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Etiqueta del campo: token dinámico label (legible sobre cualquier fondo de paleta).
        Text(
          label,
          style: Gx.uiSans(fontSize: 12, color: Gx.textBaseLabel),
        ),
        const SizedBox(height: 4),
        // Campo: cualquier widget que el padre quiera (Input, Combobox, Select…).
        child,
        // Texto de ayuda o error — visible solo cuando hay contenido.
        if (subText != null && subText.isNotEmpty) ...[
          const SizedBox(height: 4),
          Text(
            subText,
            style: Gx.uiSans(fontSize: 11, color: subColor),
          ),
        ],
      ],
    );
  }
}
