// Main panel of Drasus Engine.
// Contains EPIC-0 observable tabs + component gallery
// + theme config drawer (accent and background palette).

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import 'tabs/clock_tab.dart';
import 'tabs/jobs_tab.dart';
import 'tabs/audit_tab.dart';
import 'tabs/dashboard_tab.dart';
import 'tabs/canvas_tab.dart';
import 'tabs/settings_drawer.dart';
import 'gallery/gallery_tab.dart';

class OperationalPanel extends StatelessWidget {
  const OperationalPanel({super.key});

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 6,
      child: Scaffold(
        appBar: AppBar(
          title: Text(
            'Drasus Engine — Operational Panel',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          bottom: const TabBar(
            isScrollable: true,
            tabs: [
              Tab(icon: Icon(Icons.access_time), text: 'Clock'),
              Tab(icon: Icon(Icons.queue), text: 'Jobs'),
              Tab(icon: Icon(Icons.security), text: 'Audit'),
              Tab(icon: Icon(IconsaxPlusLinear.element_plus), text: 'Components'),
              Tab(icon: Icon(IconsaxPlusLinear.element), text: 'Dashboard'),
              Tab(icon: Icon(IconsaxPlusLinear.bezier), text: 'Canvas'),
            ],
          ),
          actions: [
            Builder(
              builder: (ctx) => IconButton(
                icon: const Icon(Icons.settings),
                tooltip: 'Themes',
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
