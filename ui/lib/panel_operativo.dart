// Panel principal del Drasus Engine.
// Contiene las 3 pestañas del observable de EPIC-0: Reloj, Trabajos, Auditoría.

import 'package:flutter/material.dart';
import 'tabs/clock_tab.dart';
import 'tabs/jobs_tab.dart';
import 'tabs/audit_tab.dart';

// Widget raíz del Panel Operativo. Es StatelessWidget porque el estado de
// qué pestaña está activa lo gestiona DefaultTabController, no este widget.
class PanelOperativo extends StatelessWidget {
  const PanelOperativo({super.key});

  // build() retorna el árbol completo del panel: controlador de pestañas,
  // barra superior con tabs, y el área de contenido que cambia según el tab.
  @override
  Widget build(BuildContext context) {
    // DefaultTabController sincroniza automáticamente el TabBar (las
    // etiquetas en la barra) con el TabBarView (el contenido debajo).
    // length indica cuántas pestañas hay — debe coincidir con los hijos
    // de TabBar y TabBarView o Flutter lanzará una excepción.
    return DefaultTabController(
      length: 3,
      child: Scaffold(
        // AppBar es la barra superior. Aquí contiene el título de la app
        // y el TabBar con las 3 pestañas navegables.
        appBar: AppBar(
          // Título visible en la barra superior de la ventana.
          title: const Text(
            'Drasus Engine — Panel Operativo',
            // Estilo monoespaciado para datos técnicos: cada carácter
            // ocupa el mismo ancho, lo que alinea columnas naturalmente.
            style: TextStyle(fontFamily: 'monospace', fontSize: 16),
          ),
          // El TabBar vive dentro del AppBar — al asignarlo a "bottom",
          // aparece justo debajo del título como una segunda fila de la barra.
          bottom: const TabBar(
            tabs: [
              // Cada Tab es una pestaña seleccionable. icon + text lo hacen
              // identificable visualmente sin necesidad de leer.
              Tab(icon: Icon(Icons.access_time), text: 'Reloj'),
              Tab(icon: Icon(Icons.queue), text: 'Trabajos'),
              Tab(icon: Icon(Icons.security), text: 'Auditoría'),
            ],
          ),
        ),
        // TabBarView es el área de contenido. Cada hijo se corresponde
        // posicionalmente con el Tab del mismo índice en TabBar.
        // DefaultTabController gestiona cuál hijo es visible.
        body: const TabBarView(
          children: [
            // Pestaña 0 — muestra el timestamp del reloj determinista de Drasus.
            ClockTab(),
            // Pestaña 1 — lista los últimos 20 trabajos y su estado.
            JobsTab(),
            // Pestaña 2 — lista los últimos 50 eventos de la bitácora de auditoría.
            AuditTab(),
          ],
        ),
      ),
    );
  }
}
