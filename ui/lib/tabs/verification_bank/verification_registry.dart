// Registro central del Banco de Verificación.
//
// Cada entrada representa una feature verificable: tiene un título, un ícono
// y un builder que construye su sección bajo demanda (igual que la galería de
// componentes). Para agregar una feature futura solo hay que añadir una entrada
// a la lista kVerificationRegistry — el tab maestro-detalle la recoge
// automáticamente sin tocar otro archivo.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import 'sovereign_data_fetcher_section.dart';

// ---------------------------------------------------------------------------
// VerificationEntry — descriptor de una sección verificable.
// ---------------------------------------------------------------------------

// title: nombre legible que aparece en el menú lateral.
// icon: IconData del ícono representativo.
// builder: construye la sección bajo demanda cuando el usuario la selecciona.
class VerificationEntry {
  final String title;
  final IconData icon;
  final WidgetBuilder builder;

  const VerificationEntry({
    required this.title,
    required this.icon,
    required this.builder,
  });
}

// ---------------------------------------------------------------------------
// kVerificationRegistry — lista de features verificables.
// Para agregar una feature: añadir una VerificationEntry aquí.
// ---------------------------------------------------------------------------
final List<VerificationEntry> kVerificationRegistry = [
  // Sección SVF de Sovereign Data Fetcher — verifica el ciclo completo
  // de descarga histórica soberana (FFI → Rust → SQLite → UI).
  VerificationEntry(
    title: 'Datos Soberanos',
    icon: IconsaxPlusLinear.document_download,
    builder: (ctx) => const SovereignDataFetcherSection(),
  ),
];
