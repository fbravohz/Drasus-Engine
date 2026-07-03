// Sección §8 Data Display extendidos — avatar, timeline, kbd, code-block,
// description-list, empty-state, image/thumbnail, popover, progress-circular,
// tree-table, carousel.
// Render-only con estado de UI local. Sin lógica de negocio ni FFI.

import 'package:flutter/material.dart';
import '../gallery_tokens.dart';
import '../gallery_fx.dart';

// ---------------------------------------------------------------------------
// Avatar / Avatar Group
// ---------------------------------------------------------------------------

// Muestra un avatar circular de placeholder y un grupo de 4 avatares solapados.
Widget avatarGroup() {
  final colors = [
    Gx.optimaCyan,
    Gx.transitionIndigo,
    Gx.alertAmber,
    Gx.criticalCrimson
  ];
  final initials = ['N7', 'N1', 'N3', 'N9'];
  return Row(
    mainAxisSize: MainAxisSize.min,
    children: List.generate(4, (i) {
      return Transform.translate(
        offset: Offset(-i * 12.0, 0),
        child: Container(
          width: 32,
          height: 32,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: Gx.surfaceCard,
            border: Border.all(color: colors[i], width: 1.5),
            boxShadow: Gx.glow(colors[i], blur: 8, opacity: 0.4),
          ),
          child: Center(
            child: Text(initials[i],
                style: Gx.dataMono(fontSize: 10, color: colors[i])),
          ),
        ),
      );
    }),
  );
}

// ---------------------------------------------------------------------------
// Timeline — línea de eventos con colores de estado
// ---------------------------------------------------------------------------

// Muestra una línea de tiempo vertical con 3 eventos de ejemplo, cada uno
// con un color semántico según su estado.
Widget timeline() {
  final events = [
    ('Backtest completado', '10:42', Gx.optimaCyan),
    ('Deriva detectada', '11:15', Gx.alertAmber),
    ('Retiro ejecutado', '11:30', Gx.criticalCrimson),
  ];
  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    mainAxisSize: MainAxisSize.min,
    children: events.asMap().entries.map((e) {
      final isLast = e.key == events.length - 1;
      final ev = e.value;
      return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        // Eje de la línea de tiempo.
        Column(
          children: [
            Container(
              width: 10,
              height: 10,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: ev.$3,
                boxShadow: Gx.glow(ev.$3, blur: 8, opacity: 0.6),
              ),
            ),
            if (!isLast)
              Container(
                width: 1,
                height: 30,
                color: Gx.divider,
              ),
          ],
        ),
        const SizedBox(width: 10),
        // Contenido del evento.
        Expanded(
          child: Padding(
            padding: const EdgeInsets.only(bottom: 6),
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Text(ev.$1,
                  style: Gx.uiSans(fontSize: 13, color: Gx.textBase)),
              Text(ev.$2,
                  style: Gx.dataMono(fontSize: 11, color: Gx.textBaseMuted)),
            ]),
          ),
        ),
      ]);
    }).toList(),
  );
}

// ---------------------------------------------------------------------------
// Code Block — bloque de código con fondo cardInner y dataMono
// ---------------------------------------------------------------------------

// Muestra un bloque de código formateado estilo terminal con fondo oscuro.
Widget codeBlock() {
  return Container(
    padding: const EdgeInsets.all(12),
    decoration: BoxDecoration(
      color: Gx.surfaceCard,
      borderRadius: BorderRadius.circular(Gx.rPanel),
      border: Border.all(color: Gx.borderBase),
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('fn execute_order(node: &str) {',
            style: Gx.dataMono(fontSize: 12, color: Gx.optimaCyan)),
        Text('    let regime = detect_regime();',
            style: Gx.dataMono(fontSize: 12, color: Gx.textBaseSecondary)),
        Text('    if regime == Regime::Volatile {',
            style: Gx.dataMono(fontSize: 12, color: Gx.textBaseSecondary)),
        Text('        return; // pausa en volátil',
            style: Gx.dataMono(fontSize: 12, color: Gx.textBaseMuted)),
        Text('    }',
            style: Gx.dataMono(fontSize: 12, color: Gx.textBaseSecondary)),
        Text('}',
            style: Gx.dataMono(fontSize: 12, color: Gx.optimaCyan)),
      ],
    ),
  );
}

// ---------------------------------------------------------------------------
// Kbd — tecla de teclado decorativa
// ---------------------------------------------------------------------------

// Muestra atajos de teclado estilo "key" con borde y fondo de tarjeta.
Widget kbdRow() {
  final keys = ['⌘', 'K'];
  return Row(
    mainAxisSize: MainAxisSize.min,
    children: keys.map((k) => Container(
          margin: const EdgeInsets.symmetric(horizontal: 2),
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: Gx.surfaceCard,
            borderRadius: BorderRadius.circular(Gx.rChip),
            border: Border.all(color: Gx.borderBase),
            boxShadow: Gx.glow(Gx.transitionIndigo, blur: 6, opacity: 0.15),
          ),
          child:
              Text(k, style: Gx.dataMono(fontSize: 12, color: Gx.textBaseSecondary)),
        )).toList(),
  );
}

// ---------------------------------------------------------------------------
// Description List — lista término-definición
// ---------------------------------------------------------------------------

// Muestra pares clave-valor en formato lista de descripción (dl/dt/dd).
Widget descriptionList() {
  final pairs = [
    ('Estrategia', 'node-07'),
    ('Régimen', 'Tendencia'),
    ('Sharpe', '1.84'),
    ('Estado', 'Óptimo'),
  ];
  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    mainAxisSize: MainAxisSize.min,
    children: pairs.map((p) => Padding(
          padding: const EdgeInsets.symmetric(vertical: 4),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SizedBox(
                width: 80,
                child: Text(p.$1,
                    style: Gx.uiSans(fontSize: 12, color: Gx.textBaseLabel)),
              ),
              Expanded(
                child: Text(p.$2,
                    style: Gx.dataMono(fontSize: 12, color: Gx.textBase)),
              ),
            ],
          ),
        )).toList(),
  );
}

// ---------------------------------------------------------------------------
// Empty State — estado vacío con orbe latente y mensaje
// ---------------------------------------------------------------------------

// Muestra el estado vacío: orbe de cristal tenue + mensaje + acción secundaria.
Widget emptyState() {
  return Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      // Orbe de cristal en modo latente (tenue, sin glow fuerte).
      Container(
        width: 48,
        height: 48,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          gradient: RadialGradient(
            colors: [Gx.transitionIndigo, Gx.surfaceCard],
            stops: [0.0, 1.0],
          ),
          border: Border.all(color: Gx.borderBase),
        ),
      ),
      const SizedBox(height: 12),
      Text('Sin estrategias activas',
          style: Gx.uiSans(fontSize: 14, color: Gx.textBaseMuted)),
      const SizedBox(height: 6),
      Text('Crea tu primera célula para comenzar.',
          style: Gx.uiSans(fontSize: 12, color: Gx.textBaseMuted)),
    ],
  );
}

// ---------------------------------------------------------------------------
// Image / Thumbnail — placeholder de imagen con borde tintado
// ---------------------------------------------------------------------------

// Placeholder de imagen: caja con gradiente sutil y borde de panel.
Widget imageThumbnail() {
  return Container(
    width: 120,
    height: 68,
    decoration: BoxDecoration(
      gradient: Gx.linear([Gx.surfacePanel, Gx.canvasBase]),
      borderRadius: BorderRadius.circular(Gx.rPanel),
      border: Border.all(color: Gx.transitionIndigo.withAlpha(80)),
      boxShadow: Gx.glow(Gx.transitionIndigo, blur: 20, opacity: 0.15),
    ),
    child: Center(
      child: Icon(Gx.iconChart, size: 24, color: Gx.textBaseMuted),
    ),
  );
}

// ---------------------------------------------------------------------------
// Progress Circular — indicador de progreso circular con glow
// ---------------------------------------------------------------------------

// Anillo de progreso circular con glow del color de estado; tamaño ajustable.
// ---------------------------------------------------------------------------
// Popover — contenido flotante contextual
// ---------------------------------------------------------------------------

// Muestra un popover de vidrio con una descripción de métrica.
Widget popoverExample() {
  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    mainAxisSize: MainAxisSize.min,
    children: [
      // Ancla del popover (el elemento sobre el que "flota").
      Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        decoration: BoxDecoration(
          color: Gx.surfaceFill,
          borderRadius: BorderRadius.circular(Gx.rChip),
          border: Border.all(color: Gx.borderBase),
        ),
        child: Text('Sharpe 1.84',
            style: Gx.dataMono(fontSize: 12, color: Gx.optimaCyan)),
      ),
      const SizedBox(height: 6),
      // Popover flotante.
      panelSurface(
        radius: Gx.rTooltip,
        glow: Gx.glow(Gx.transitionIndigo, blur: 14, opacity: 0.2),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Sharpe ajustado',
                style: Gx.uiSans(fontSize: 12, color: Gx.textBaseSecondary,
                    weight: FontWeight.w500)),
            const SizedBox(height: 4),
            Text('Ratio de Sharpe ponderado por régimen\nen los últimos 90 días.',
                style: Gx.uiSans(fontSize: 11, color: Gx.textBaseLabel)),
          ],
        ),
      ),
    ],
  );
}

// ---------------------------------------------------------------------------
// Tree Table — tabla con nodos expandibles
// ---------------------------------------------------------------------------

// Tabla jerárquica: la fila raíz se expande para mostrar hijos con padding.
// ---------------------------------------------------------------------------
// Carousel — carrusel de tarjetas con navegación de puntos
// ---------------------------------------------------------------------------

// Carrusel de 3 tarjetas; los puntos de navegación muestran la página activa.
