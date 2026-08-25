// canvas_inspector_registry.dart — el mecanismo GENÉRICO que hace que un
// nodo del lienzo abra el panel de su feature (STORY-073 Etapa 4, criterios
// de aceptación #8 y #9).
//
// El problema que este archivo resuelve NO es "cómo abro el panel del
// asistente de importación" — es que `canvas_tab.dart` no tenía forma de
// abrir NINGÚN panel: los nodos eran rectángulos inertes sin manejador de
// pulsación. Por eso este registro es una tabla de despacho por nombre de
// feature, no un `if` especial para una sola feature. Adoptar una feature
// nueva en el lienzo (incluida `hybrid-data-transformer`, huérfana hasta
// ahora) es añadir UNA entrada aquí — nunca tocar `canvas_tab.dart`.
//
// ADR-0136 (FIJO): un feature node siempre abre panel lateral, nunca
// pantalla completa — el builder que cada entrada registra es el CONTENIDO
// del panel; el chrome (superficie de vidrio, cabecera, botón de cierre) lo
// pone `canvas_tab.dart` una sola vez para todas las features.

import 'package:flutter/material.dart';

import 'data_import_wizard/data_import_wizard_inspector.dart';
import 'hybrid_data_transformer/hybrid_data_transformer_inspector.dart';

// Constructor del contenido de un panel — recibe el `BuildContext` del sitio
// de montaje (el lienzo), como cualquier `WidgetBuilder` de Flutter.
typedef CanvasPanelBuilder = Widget Function(BuildContext context);

// Tabla de despacho: nombre de la feature (el mismo `String` que viaja como
// payload del `Draggable` en `canvas_tab.dart`) -> constructor de su panel.
// Cada feature con superficie de Inspector Panel construida se registra
// aquí exactamente una vez.
final Map<String, CanvasPanelBuilder> kCanvasFeaturePanels = {
  'Asistente de Importación': (context) =>
      const DataImportWizardInspectorPanel(),
  // Adopción del panel huérfano (STORY-073 criterio #9): ya existía,
  // construido y probado en STORY-051, pero ningún lienzo lo alcanzaba.
  'Transformador de Datos de Mercado': (context) =>
      const HybridDataTransformerInspectorPanel(),
};

// Resuelve el builder de una feature, con una caída honesta a un panel
// "sin construir" para features que el usuario puede arrastrar al lienzo
// (`_kFeatureList` en `canvas_tab.dart`) pero que todavía no declaran
// Inspector Panel propio (ej. las fundaciones EPIC-0, que hoy viven en el
// Panel Operativo, no en el Canvas). Nunca un `no-op` silencioso: el usuario
// siempre ve que pulsó algo, incluso si ese algo no tiene contenido todavía.
CanvasPanelBuilder resolveCanvasPanelBuilder(String featureLabel) {
  return kCanvasFeaturePanels[featureLabel] ??
      (context) => _UnavailablePanel(featureLabel: featureLabel);
}

// Panel de reserva: mismo chrome que cualquier otro panel del registro, pero
// con un estado vacío honesto en vez de contenido inventado.
class _UnavailablePanel extends StatelessWidget {
  final String featureLabel;
  const _UnavailablePanel({required this.featureLabel});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Text(
          '"$featureLabel" todavía no tiene un Inspector Panel construido.',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyMedium,
        ),
      ),
    );
  }
}
