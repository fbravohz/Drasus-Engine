// Sección STD faltantes — piezas del catálogo §5–§7 pendientes.
// GlowCascader y GlowMentionInput migrados a components/ en Batch 2 (STORY-025).
// Resto de clases Glow* se migrarán en Batch 3/4.

import 'package:flutter/material.dart';
import '../../theme/gx_tokens.dart';
import '../../theme/surfaces.dart';

// ===========================================================================
// §6 INPUTS — TRANSFER / DUAL-LIST
// ===========================================================================

// Muestra dos listas (disponibles / seleccionados) con botones de transferencia
// entre ellas. Los datos son símbolos hardcodeados.
// ===========================================================================
// §6 INPUTS — DATE-RANGE PICKER
// ===========================================================================

// Muestra dos campos de fecha (inicio / fin) con un rango hardcodeado
// seleccionado. Cada campo tiene glow en foco y el rango se resalta
// en el mini-calendario debajo.
// ===========================================================================
// §6 INPUTS — TIME PICKER
// ===========================================================================

// Muestra un selector de hora con dos columnas deslizables (horas / minutos)
// de estilo rueda. El ítem central está resaltado con glow.
// ===========================================================================
// §6 INPUTS — COLOR PICKER
// ===========================================================================

// Muestra una paleta de colores del espectro de vitalidad + una muestra
// del color seleccionado. Sin rueda HSV — la paleta de Drasus es semántica.
// ===========================================================================
// §6 INPUTS — FILE UPLOAD / DROPZONE
// ===========================================================================

// Muestra una zona de arrastre de archivos con estado: reposo, arrastrando
// (activado al pasar el mouse), y "cargando" (simulado al tocar).
// Estados internos de la dropzone.
enum _DropState { idle, hover, loading }

// GlowMentionInput migrado a ui/lib/components/mention_input.dart (Batch 2, STORY-025).

// GlowSplitButton migrado a ui/lib/components/split_button.dart (Batch 3, STORY-025).
// La galería consume ui.SplitButton vía gallery_registry.dart.

// ===========================================================================
// §5 NAVEGACIÓN — BACK TO TOP
// ===========================================================================

// Botón flotante de "volver arriba" con vidrio Apple y glow.
// En la galería se muestra como cáscara estática (sin scroll real).
