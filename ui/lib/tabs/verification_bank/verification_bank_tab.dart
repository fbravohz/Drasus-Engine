// Banco de Verificación — tab maestro-detalle del Panel Operativo.
//
// Sigue el patrón de gallery_tab.dart: menú lateral con ícono + label para
// cada feature verificable, y un panel de detalle que construye la sección
// seleccionada bajo demanda (builder por entrada en kVerificationRegistry).
//
// El registro kVerificationRegistry en verification_registry.dart es el punto
// de extensión local: para agregar una feature futura solo se añade una
// VerificationEntry allí. Además, al montar el tab se intenta reemplazar esa
// lista por la que devuelva el backend real (listVerifiableFeatures(), FFI) —
// mientras el binding no exista (verification_bridge.dart), el registro local
// sigue poblando el selector sin dejarlo vacío.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import '../../theme/gx_tokens.dart';
import 'generic_verification_section.dart';
import 'verification_bridge.dart' show listVerifiableFeatures, FeatureDescriptor;
import 'verification_registry.dart';

// VerificationBankTab — raíz del Banco de Verificación.
// StatefulWidget porque gestiona qué feature está seleccionada en el menú y
// la lista de entradas (local o fusionada con el backend).
class VerificationBankTab extends StatefulWidget {
  const VerificationBankTab({super.key});

  @override
  State<VerificationBankTab> createState() => _VerificationBankTabState();
}

class _VerificationBankTabState extends State<VerificationBankTab> {
  // Índice de la feature seleccionada en el menú lateral.
  // Inicia en 0 para mostrar la primera feature al abrir el tab.
  int _selectedIndex = 0;

  // Entradas del selector — arranca con el registro local para no dejar el
  // panel vacío mientras se resuelve la consulta al backend.
  List<VerificationEntry> _entries = kVerificationRegistry;

  @override
  void initState() {
    super.initState();
    _cargarDesdeBackend();
  }

  // Consulta listVerifiableFeatures() por FFI (llamada síncrona: cruza el
  // puente una sola vez y no toca I/O) y fusiona el resultado con el
  // registro local (icono + secciones a mano se preservan por id). Si el
  // backend devuelve catálogo vacío (no debería, pero es defensivo), el
  // registro local queda como única fuente.
  Future<void> _cargarDesdeBackend() async {
    final backend = listVerifiableFeatures();
    if (!mounted || backend.isEmpty) return;
    setState(() {
      _entries = _mergeConBackend(backend);
      // Si el índice seleccionado quedó fuera de rango tras la fusión,
      // vuelve a la primera entrada.
      if (_selectedIndex >= _entries.length) _selectedIndex = 0;
    });
  }

  // Fusiona la lista del backend con el registro local: conserva las
  // secciones a mano (isHandCrafted) por id, y construye la sección genérica
  // para el resto usando el label/JSON de ejemplo que informa el backend
  // (autoridad del contrato) y el ícono local si existe (o uno por defecto).
  List<VerificationEntry> _mergeConBackend(List<FeatureDescriptor> backend) {
    final porId = {for (final e in kVerificationRegistry) e.id: e};
    return [
      for (final d in backend)
        if (porId[d.id]?.isHandCrafted ?? false)
          porId[d.id]!
        else
          VerificationEntry(
            id: d.id,
            title: d.displayName,
            icon: porId[d.id]?.icon ?? IconsaxPlusLinear.verify,
            builder: (ctx) => GenericVerificationSection(
              key: ValueKey(d.id),
              featureId: d.id,
              title: d.displayName,
              icon: porId[d.id]?.icon ?? IconsaxPlusLinear.verify,
              defaultInputJson: d.exampleInputJson,
            ),
          ),
    ];
  }

  // build(): layout maestro-detalle de dos columnas.
  // Columna izquierda: navRail (240 px, fondo Gx.navRail).
  // Columna derecha: panel de detalle expandido con padding.
  @override
  Widget build(BuildContext context) {
    // Fondo del lienzo: deepSpace de la paleta activa.
    final bg = Gx.canvasBase;

    return Container(
      color: bg,
      child: Row(
        children: [
          // Panel lateral fijo de 240 px.
          SizedBox(
            width: 240,
            child: _buildSidebar(),
          ),
          // Divisor vertical de 1 px con color de borde base.
          VerticalDivider(
              width: 1, thickness: 1, color: Gx.borderBase),
          // Panel de detalle — ocupa el espacio restante.
          Expanded(child: _buildDetail()),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Panel lateral — menú de features verificables.
  // ---------------------------------------------------------------------------

  // Muestra el encabezado del Banco y la lista de entradas del registro.
  Widget _buildSidebar() {
    return Container(
      color: Gx.navRail,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Encabezado del Banco de Verificación.
          _buildSidebarHeader(),
          const SizedBox(height: 8),
          // Lista de features verificables del registro.
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.only(bottom: 24),
              itemCount: _entries.length,
              itemBuilder: (ctx, i) =>
                  _buildSidebarEntry(ctx, i),
            ),
          ),
        ],
      ),
    );
  }

  // Encabezado del panel lateral: título "Verificación" + subtítulo.
  Widget _buildSidebarHeader() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 20, 16, 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Título con gradiente cósmico — igual que la galería.
          ShaderMask(
            shaderCallback: (rect) =>
                const LinearGradient(colors: Gx.gradCosmic)
                    .createShader(rect),
            child: Text(
              'Verificación',
              style: TextStyle(
                fontFamily: Gx.fontDisplay,
                fontSize: 20,
                fontWeight: FontWeight.w500,
                letterSpacing: -0.4,
                color: Gx.pureWhite,
              ),
            ),
          ),
          const SizedBox(height: 4),
          Text(
            'Banco de pruebas FFI',
            style: Gx.microLabel.copyWith(color: Gx.textBaseMuted),
          ),
        ],
      ),
    );
  }

  // Entrada individual del menú lateral para una feature verificable.
  // La entrada activa: borde izquierdo 2 px transitionIndigo + glow + fondo surfaceRaised.
  Widget _buildSidebarEntry(BuildContext ctx, int index) {
    final entry = _entries[index];
    final isSelected = _selectedIndex == index;

    return InkWell(
      onTap: () => setState(() => _selectedIndex = index),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        // Fondo elevado cuando la entrada está seleccionada.
        color: isSelected ? Gx.surfaceRaisedDynamic : Colors.transparent,
        child: Row(
          children: [
            // Borde izquierdo de acento: solo visible cuando está seleccionada.
            if (isSelected)
              Container(
                width: 2,
                height: 18,
                margin: const EdgeInsets.only(right: 10),
                decoration: BoxDecoration(
                  // Gradiente aurora para el acento de selección.
                  gradient: Gx.linear(
                    Gx.gradAurora,
                    begin: Alignment.topCenter,
                    end: Alignment.bottomCenter,
                  ),
                  boxShadow: Gx.glow(
                    Gx.transitionIndigo,
                    blur: 6,
                    opacity: 0.7,
                  ),
                ),
              )
            else
              const SizedBox(width: 12),
            // Ícono de la feature.
            Icon(
              entry.icon,
              size: 16,
              color: isSelected
                  ? Gx.transitionIndigo
                  : Gx.textBaseLabel,
              shadows: isSelected
                  ? Gx.textGlow(Gx.transitionIndigo, 8)
                  : null,
            ),
            const SizedBox(width: 10),
            // Nombre de la feature.
            Expanded(
              child: Text(
                entry.title,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontFamily: Gx.fontSans,
                  fontSize: 13,
                  color: isSelected ? Gx.transitionIndigo : Gx.textBaseLabel,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Panel de detalle — sección construida bajo demanda.
  // ---------------------------------------------------------------------------

  // Construye la sección de la feature seleccionada usando su builder.
  // El builder se invoca solo al seleccionar — construcción bajo demanda.
  Widget _buildDetail() {
    // Guarda de índice fuera de rango (precaución si el registro cambia).
    if (_selectedIndex >= _entries.length) {
      return const SizedBox.shrink();
    }
    final entry = _entries[_selectedIndex];

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Encabezado de sección con barra de acento aurora.
          _buildDetailHeader(entry.title),
          const SizedBox(height: 16),
          // Sección de la feature bajo demanda — llena el espacio restante.
          Expanded(child: entry.builder(context)),
        ],
      ),
    );
  }

  // Encabezado del panel de detalle con barra de acento + título de sección.
  Widget _buildDetailHeader(String title) {
    return Row(children: [
      Container(
        width: 3,
        height: 20,
        margin: const EdgeInsets.only(right: 10),
        decoration: BoxDecoration(
          gradient: Gx.linear(
            Gx.gradAurora,
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
          ),
          boxShadow:
              Gx.glow(Gx.transitionIndigo, blur: 10, opacity: 0.7),
        ),
      ),
      Text(title, style: Gx.sectionHeading),
    ]);
  }
}
