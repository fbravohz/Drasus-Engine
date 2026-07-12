// Registro central del Banco de Verificación.
//
// Cada entrada representa una feature verificable: id (kebab-case, idéntico
// al `feature_id` que usa `drasus verify` en crates/app/src/main.rs), título,
// ícono y builder. Para agregar una feature futura basta con una llamada a
// _generic() con su id + label + ícono + JSON de ejemplo — la sección misma
// (GenericVerificationSection) es compartida por todas, así que enchufar una
// feature nueva es casi una línea, no una pantalla nueva.
//
// El tab maestro-detalle (verification_bank_tab.dart) intenta reemplazar esta
// lista por la que devuelva el backend real (listVerifiableFeatures(), FFI);
// mientras ese binding no exista, este registro es el que puebla el selector
// lateral — ver verification_bridge.dart para el punto de cableado.
//
// Los JSON de ejemplo de las 14 features del substrato de monetización se
// copiaron literalmente de los doc-comments de `drasus verify` en
// crates/app/src/main.rs (mismos payloads que usa el propio CLI) — no son
// inventados.

import 'package:flutter/material.dart';
import 'package:iconsax_plus/iconsax_plus.dart';
import 'generic_verification_section.dart';
import 'sovereign_data_fetcher_section.dart';

// ---------------------------------------------------------------------------
// VerificationEntry — descriptor de una sección verificable.
// ---------------------------------------------------------------------------

// id: identificador kebab-case que el backend usa para despachar la
//     verificación (idéntico al `feature_id` de `drasus verify`).
// title: nombre legible que aparece en el menú lateral.
// icon: IconData del ícono representativo.
// builder: construye la sección bajo demanda cuando el usuario la selecciona.
// isHandCrafted: true si la sección NO usa el patrón genérico (aporta
//     visualización propia, ej. historial persistido) — el merge con el
//     backend (ver verification_bank_tab.dart) preserva estas entradas en
//     vez de reemplazarlas por la genérica.
class VerificationEntry {
  final String id;
  final String title;
  final IconData icon;
  final WidgetBuilder builder;
  final bool isHandCrafted;

  const VerificationEntry({
    required this.id,
    required this.title,
    required this.icon,
    required this.builder,
    this.isHandCrafted = false,
  });
}

// Construye una entrada que usa el patrón genérico (el caso por defecto,
// ADR-0117 §SVF): un id + label + ícono + JSON de ejemplo bastan para
// enchufar una feature completa al Banco. La ValueKey por id fuerza a
// Flutter a reconstruir el estado del editor al cambiar de feature.
VerificationEntry _generic({
  required String id,
  required String title,
  required IconData icon,
  required String exampleJson,
}) {
  return VerificationEntry(
    id: id,
    title: title,
    icon: icon,
    builder: (ctx) => GenericVerificationSection(
      key: ValueKey(id),
      featureId: id,
      title: title,
      icon: icon,
      defaultInputJson: exampleJson,
    ),
  );
}

// ---------------------------------------------------------------------------
// kVerificationRegistry — lista de features verificables.
// Para agregar una feature: añadir una entrada aquí (normalmente _generic()).
// ---------------------------------------------------------------------------
final List<VerificationEntry> kVerificationRegistry = [
  // ── Plomería (fetcher) ───────────────────────────────────────────────────
  // Sección a mano: aporta visualización extra sobre el patrón genérico
  // (Zona C con el historial persistido en sovereign_download_records, con
  // sus 3 columnas reales) — se conserva en vez de migrarse (SKILL.md §2d:
  // "si aporta visualización extra" es el criterio para no migrar).
  VerificationEntry(
    id: 'sovereign-data-fetcher',
    title: 'Datos Soberanos',
    icon: IconsaxPlusLinear.document_download,
    builder: (ctx) => const SovereignDataFetcherSection(),
    isHandCrafted: true,
  ),

  // ── Los 14 cimientos del substrato de monetización (ADR-0143..0149) ─────
  // Todos comparten GenericVerificationSection — el registro solo aporta
  // metadatos + el JSON de ejemplo real que ya usa `drasus verify <id>`.
  _generic(
    id: 'central-identity',
    title: 'Identidad Central',
    icon: IconsaxPlusLinear.profile_tick,
    exampleJson: '{"email":"a@b.com"}',
  ),
  _generic(
    id: 'licensing-system',
    title: 'Licencias',
    icon: IconsaxPlusLinear.key_square,
    exampleJson: '{"tier":"SOVEREIGN"}',
  ),
  _generic(
    id: 'plan-tier-quota',
    title: 'Plan y Cuotas',
    icon: IconsaxPlusLinear.speedometer,
    exampleJson: '{"tier":"FREE"}',
  ),
  _generic(
    id: 'usage-metering',
    title: 'Medición de Uso',
    icon: IconsaxPlusLinear.activity,
    exampleJson:
        '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}',
  ),
  _generic(
    id: 'consent-registry',
    title: 'Registro de Consentimiento',
    icon: IconsaxPlusLinear.shield_tick,
    exampleJson:
        '{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}',
  ),
  _generic(
    id: 'enriched-domain-events',
    title: 'Eventos de Dominio',
    icon: IconsaxPlusLinear.note_2,
    exampleJson:
        '{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}',
  ),
  _generic(
    id: 'institutional-report-engine',
    title: 'Motor de Reportes',
    icon: IconsaxPlusLinear.receipt_text,
    exampleJson:
        '{"report_type":"VALIDATION","metrics":{"sharpe_e8":150000000,"max_drawdown_e8":-8000000},"source_event_refs":["evt-1","evt-2"]}',
  ),
  _generic(
    id: 'third-party-api-gateway',
    title: 'Pasarela de APIs',
    icon: IconsaxPlusLinear.link_square,
    exampleJson:
        '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}',
  ),
  _generic(
    id: 'data-aggregation',
    title: 'Agregación de Datos',
    icon: IconsaxPlusLinear.category_2,
    exampleJson:
        '{"seed":42,"min_cohort":5,"external_sale_enabled":false,"events":[{"metric_e8":150000000,"consent":"COVERED"}]}',
  ),
  _generic(
    id: 'verified-account-registry',
    title: 'Cuentas Verificadas',
    icon: IconsaxPlusLinear.verify,
    exampleJson:
        '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN"},"consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}',
  ),
  _generic(
    id: 'instance-continuity',
    title: 'Continuidad de Instancia',
    icon: IconsaxPlusLinear.refresh_circle,
    exampleJson:
        '{"master_secret":"correct horse battery staple","plaintext":"snapshot-bytes","nonce_seed":42,"custody":{"titular_node_id":"node-A","custody_epoch":3},"my_node_id":"node-A"}',
  ),
  _generic(
    id: 'master-account-hierarchy',
    title: 'Jerarquía de Cuentas',
    icon: IconsaxPlusLinear.hierarchy,
    exampleJson:
        '{"parent_owner_id":"fund-X","child_owner_id":"trader-7","node_id":"node-A","consent":"COVERED","command_kind":"ARCHIVE","target_ref":"strategy-42","justification":"riesgo excedido"}',
  ),
  _generic(
    id: 'data-portability',
    title: 'Portabilidad de Datos',
    icon: IconsaxPlusLinear.export,
    exampleJson:
        '{"owner_id":"user-42","institutional_tag":"LIVE","node_id":"node-A","request_type":"FORGET"}',
  ),
  _generic(
    id: 'operator-roles',
    title: 'Roles de Operador',
    icon: IconsaxPlusLinear.user_tag,
    exampleJson:
        '{"owner_id":"acc-1","institutional_tag":"LIVE","node_id":"node-A","access_token_id":"tok-owner","capability_key":"generate.run_search","pipeline":"GENERATE"}',
  ),
];
