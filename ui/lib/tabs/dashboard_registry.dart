// Registro de widgets del Dashboard.
// Define los metadatos de cada widget disponible (o próximo) en el catálogo
// del Dashboard. No contiene lógica de negocio: es un catálogo declarativo
// de lo que el usuario puede agregar al tablero.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';

// ---------------------------------------------------------------------------
// DashboardWidgetMeta — descriptor de un widget del catálogo.
// ---------------------------------------------------------------------------

// id: identificador único del widget (kebab-case), usado para persistencia.
// name: nombre legible que aparece en el catálogo.
// description: una línea que explica qué muestra el widget.
// icon: IconData del ícono representativo.
// available: false = el widget aún no existe, se muestra como "Próximamente".
class DashboardWidgetMeta {
  final String id;
  final String name;
  final String description;
  final IconData icon;
  final bool available;

  const DashboardWidgetMeta({
    required this.id,
    required this.name,
    required this.description,
    required this.icon,
    required this.available,
  });
}

// ---------------------------------------------------------------------------
// kDashboardRegistry — catálogo inicial de widgets.
// Todos están marcados available: false (ninguno implementado en EPIC-0).
// ---------------------------------------------------------------------------
const List<DashboardWidgetMeta> kDashboardRegistry = [
  DashboardWidgetMeta(
    id: 'reloj-determinista',
    name: 'Reloj Determinista',
    description: 'Timestamp del reloj global del Core de Drasus.',
    icon: IconsaxPlusLinear.clock,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'async-job-queue',
    name: 'Cola de Trabajos',
    description: 'Estado de los últimos trabajos asíncronos en ejecución.',
    icon: IconsaxPlusLinear.element_1,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'audit-log',
    name: 'Bitácora de Auditoría',
    description: 'Eventos recientes del registro de auditoría.',
    icon: IconsaxPlusLinear.shield_tick,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'telemetria',
    name: 'Telemetría',
    description: 'Métricas de rendimiento del motor en tiempo real.',
    icon: IconsaxPlusLinear.chart,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'mcp-gateway',
    name: 'MCP Gateway',
    description: 'Estado de las conexiones del gateway de agentes.',
    icon: IconsaxPlusLinear.graph,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'equity-curve-widget',
    name: 'Curva de Equity',
    description: 'Evolución del capital a lo largo del tiempo.',
    icon: IconsaxPlusLinear.chart_1,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'drawdown-widget',
    name: 'Drawdown',
    description: 'Caídas máximas respecto al pico de capital.',
    icon: IconsaxPlusLinear.chart_2,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'regime-chip-widget',
    name: 'Chip de Régimen',
    description: 'Régimen de mercado detectado por el clasificador.',
    icon: IconsaxPlusLinear.magicpen,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'monte-carlo-widget',
    name: 'Monte Carlo',
    description: 'Distribución de resultados simulados con Monte Carlo.',
    icon: IconsaxPlusLinear.warning_2,
    available: false,
  ),
  DashboardWidgetMeta(
    id: 'galaxy-3d-widget',
    name: 'Vista Galáctica 3D',
    description: 'Visualización tridimensional del universo de instrumentos.',
    icon: IconsaxPlusLinear.element_plus,
    available: false,
  ),
  // Widget de última descarga de históricos soberanos — implementado en STORY-024.
  DashboardWidgetMeta(
    id: 'sovereign-data-fetcher',
    name: 'Datos Soberanos',
    description:
        'Estado de la última descarga: ID, timestamp y endpoint fuente.',
    icon: IconsaxPlusLinear.document_download,
    available: true,
  ),
];
