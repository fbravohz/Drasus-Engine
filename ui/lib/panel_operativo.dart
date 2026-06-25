// Panel principal del Drasus Engine.
// Contiene las pestañas del observable de EPIC-0 + galería de componentes
// + drawer de configuración de temas (acento y paleta de fondo).

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import 'tabs/clock_tab.dart';
import 'tabs/jobs_tab.dart';
import 'tabs/audit_tab.dart';
import 'tabs/dashboard_tab.dart';
import 'tabs/canvas_tab.dart';
import 'tabs/settings_drawer.dart';
import 'gallery/gallery_tab.dart';

class PanelOperativo extends StatelessWidget {
  const PanelOperativo({super.key});

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 6,
      child: Scaffold(
        appBar: AppBar(
          title: const Text(
            'Drasus Engine — Panel Operativo',
            style: TextStyle(fontFamily: 'JetBrainsMono', fontSize: 16),
          ),
          bottom: const TabBar(
            isScrollable: true,
            tabs: [
              Tab(icon: Icon(Icons.access_time), text: 'Reloj'),
              Tab(icon: Icon(Icons.queue), text: 'Trabajos'),
              Tab(icon: Icon(Icons.security), text: 'Auditoría'),
              Tab(icon: Icon(IconsaxPlusLinear.element_plus), text: 'Components'),
              Tab(icon: Icon(IconsaxPlusLinear.element), text: 'Dashboard'),
              Tab(icon: Icon(IconsaxPlusLinear.bezier), text: 'Canvas'),
            ],
          ),
          actions: [
            Builder(
              builder: (ctx) => IconButton(
                icon: const Icon(Icons.settings),
                tooltip: 'Temas',
                onPressed: () => Scaffold.of(ctx).openEndDrawer(),
              ),
            ),
          ],
        ),
        endDrawer: const SettingsDrawer(),
        body: const TabBarView(
          children: [
            ClockTab(),
            JobsTab(),
            AuditTab(),
            GalleryTab(),
            DashboardTab(),
            CanvasTab(),
          ],
        ),
      ),
    );
  }
}
