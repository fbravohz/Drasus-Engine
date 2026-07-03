// Main panel of Drasus Engine.
// Contains EPIC-0 observable tabs + component gallery
// + theme config drawer (accent and background palette).
// Tabs rewired to ui.Tabs (Batch 4 STORY-025): reemplaza DefaultTabController
// + Material TabBar + TabBarView por el componente encapsulado ui.Tabs.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import 'tabs/clock_tab.dart';
import 'tabs/jobs_tab.dart';
import 'tabs/audit_tab.dart';
import 'tabs/dashboard_tab.dart';
import 'tabs/canvas_tab.dart';
import 'tabs/settings_drawer.dart';
import 'gallery/gallery_tab.dart';
import 'tabs/verification_bank/verification_bank_tab.dart';
import 'components/components.dart' as ui;

// Panel principal con 7 pestañas de observabilidad + galería + cajón de tema.
// ui.Tabs gestiona el DefaultTabController, la barra estilizada con tokens Gx
// y el TabBarView — sin necesidad de DefaultTabController en este nivel.
class OperationalPanel extends StatelessWidget {
  const OperationalPanel({super.key});

  @override
  // Muestra la AppBar con título y acciones + el body con ui.Tabs de 7 pestañas.
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(
          'Drasus Engine — Operational Panel',
          style: Theme.of(context).textTheme.titleLarge,
        ),
        actions: [
          Builder(
            builder: (ctx) => IconButton(
              icon: const Icon(Icons.settings),
              tooltip: 'Themes',
              // Abre el cajón lateral de configuración de tema desde el Scaffold.
              onPressed: () => Scaffold.of(ctx).openEndDrawer(),
            ),
          ),
        ],
      ),
      endDrawer: const SettingsDrawer(),
      // ui.Tabs encapsula DefaultTabController + barra estilizada + TabBarView.
      // No es const: lee Gx.accentDynamic y Gx.textBaseSecondary (getters dinámicos).
      body: ui.Tabs(
        isScrollable: true,
        tabs: [
          ui.TabItem(
            icon: const Icon(Icons.access_time),
            label: 'Clock',
            child: const ClockTab(),
          ),
          ui.TabItem(
            icon: const Icon(Icons.queue),
            label: 'Jobs',
            child: const JobsTab(),
          ),
          ui.TabItem(
            icon: const Icon(Icons.security),
            label: 'Audit',
            child: const AuditTab(),
          ),
          ui.TabItem(
            icon: Icon(IconsaxPlusLinear.element_plus),
            label: 'Components',
            child: const GalleryTab(),
          ),
          ui.TabItem(
            icon: Icon(IconsaxPlusLinear.element),
            label: 'Dashboard',
            child: const DashboardTab(),
          ),
          ui.TabItem(
            icon: Icon(IconsaxPlusLinear.bezier),
            label: 'Canvas',
            child: const CanvasTab(),
          ),
          ui.TabItem(
            icon: Icon(IconsaxPlusLinear.verify),
            label: 'Verificación',
            child: const VerificationBankTab(),
          ),
        ],
      ),
    );
  }
}
