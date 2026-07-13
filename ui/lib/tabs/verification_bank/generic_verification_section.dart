// generic_verification_section.dart — Sección genérica del Banco de Verificación.
//
// Sirve para CUALQUIER feature verificable: tres zonas (izquierda = editor del
// JSON de input precargado, centro = botón Enviar, derecha = respuesta REAL
// del backend por FFI, read-only). Enchufar una feature nueva es aportar solo
// su id + label + ícono + JSON de ejemplo en verification_registry.dart — esta
// clase no cambia. Es la gemela GUI de `drasus verify <feature-id>` (mismo
// contrato, ver crates/app/src/main.rs).
//
// Estados manejados: idle / enviando / éxito / input-inválido / error de backend.
// El veredicto de "input bien/mal formado para el puente FFI" viene del backend
// (VerificationOutcome.inputStatus) — esta sección solo lo presenta, nunca lo
// calcula (Cáscara Delgada: cero lógica de negocio en Dart).

import 'dart:convert';

import 'package:flutter/material.dart';

import '../../components/components.dart' as custom_ui;
import '../../theme/gx_tokens.dart';
import 'verification_bridge.dart';

// Estado de presentación local del ciclo enviar/responder. No es lógica de
// negocio: solo gobierna qué zona de la UI se muestra.
enum _VerifyState { idle, sending, success, invalidInput, backendError }

// Firma de la función de verificación FFI real (misma firma que
// `verifyFeature` de verification_bridge.dart). Seam de inyección MÍNIMO
// (DEBT-022): permite a los tests de widget sustituirla por un doble que
// devuelve un VerificationOutcome fabricado, sin cruzar el puente FFI real
// (no ejecutable dentro de un widget test). Producción nunca la pasa —
// siempre cae al default `verifyFeature` real (ver constructor).
typedef VerifyFn = Future<VerificationOutcome> Function({
  required String featureId,
  required String inputJson,
});

// GenericVerificationSection — widget único reutilizado por todas las
// entradas del registro. StatefulWidget porque gestiona el texto editable
// del input y el resultado del último envío.
class GenericVerificationSection extends StatefulWidget {
  // Identificador en kebab-case que el backend usa para despachar la
  // verificación (idéntico al `feature_id` de `drasus verify`).
  final String featureId;
  final String title;
  final IconData icon;
  // JSON de ejemplo precargado en el editor — string crudo (se formatea al
  // montar el widget solo para presentación, no altera el contrato).
  final String defaultInputJson;
  // Seam de inyección opcional para tests de widget (DEBT-022). Producción
  // nunca lo pasa: queda `null` y `_enviar()` cae al `verifyFeature` real.
  final VerifyFn? verifyOverride;

  const GenericVerificationSection({
    super.key,
    required this.featureId,
    required this.title,
    required this.icon,
    required this.defaultInputJson,
    this.verifyOverride,
  });

  @override
  State<GenericVerificationSection> createState() =>
      _GenericVerificationSectionState();
}

class _GenericVerificationSectionState
    extends State<GenericVerificationSection> {
  // Controller del editor de la Zona A — precargado con el JSON de ejemplo.
  late final TextEditingController _inputCtrl;

  _VerifyState _state = _VerifyState.idle;
  // Guardan el detalle del último resultado según el estado alcanzado.
  String? _invalidReason;
  String? _backendError;
  String? _outputJsonPretty;

  @override
  void initState() {
    super.initState();
    _inputCtrl = TextEditingController(text: _prettyOrRaw(widget.defaultInputJson));
  }

  @override
  void dispose() {
    _inputCtrl.dispose();
    super.dispose();
  }

  // Formatea un JSON crudo con indentación de 2 espacios para lectura humana.
  // Puramente presentacional: si no es JSON válido, muestra el texto tal cual
  // (nunca inventa ni corrige contenido).
  String _prettyOrRaw(String raw) {
    try {
      final decoded = jsonDecode(raw);
      return const JsonEncoder.withIndent('  ').convert(decoded);
    } catch (_) {
      return raw;
    }
  }

  // Dispara verifyFeature() por FFI con el JSON actual del editor.
  Future<void> _enviar() async {
    setState(() {
      _state = _VerifyState.sending;
      _invalidReason = null;
      _backendError = null;
      _outputJsonPretty = null;
    });

    // Usa el doble inyectado en tests si existe; en producción `verifyOverride`
    // siempre es null y esto resuelve al `verifyFeature` real (FFI).
    final verify = widget.verifyOverride ?? verifyFeature;
    final outcome = await verify(
      featureId: widget.featureId,
      inputJson: _inputCtrl.text,
    );

    if (!mounted) return;

    // Prioridad de lectura del resultado: primero el veredicto de forma del
    // input (bridge FFI, unión sealed InputStatus generada por freezed),
    // luego el resultado del backend.
    final status = outcome.inputStatus;
    if (status is InputStatus_Invalid) {
      setState(() {
        _state = _VerifyState.invalidInput;
        _invalidReason = status.reason;
      });
      return;
    }

    if (!outcome.ok) {
      setState(() {
        _state = _VerifyState.backendError;
        _backendError = outcome.error ?? 'Error no informado.';
      });
      return;
    }

    setState(() {
      _state = _VerifyState.success;
      _outputJsonPretty = _prettyOrRaw(outcome.outputJson);
    });
  }

  // ---------------------------------------------------------------------------
  // Build
  // ---------------------------------------------------------------------------

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildFeatureHeader(),
        const SizedBox(height: Gx.space16),
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Zona izquierda — editor del JSON de input.
              Expanded(flex: 4, child: _buildInputZone()),
              const SizedBox(width: Gx.space16),
              // Zona central — botón Enviar.
              SizedBox(width: 140, child: _buildActionZone()),
              const SizedBox(width: Gx.space16),
              // Zona derecha — respuesta real del backend, read-only.
              Expanded(flex: 5, child: _buildOutputZone()),
            ],
          ),
        ),
      ],
    );
  }

  // Encabezado: ícono + título + chip con el id de la feature (contrato FFI).
  Widget _buildFeatureHeader() {
    return Row(children: [
      Icon(widget.icon, size: 16, color: Gx.textBaseLabel),
      const SizedBox(width: 8),
      Text(
        '${widget.title} — Verificación FFI',
        style: Gx.uiSans(fontSize: 14, color: Gx.textBase, weight: FontWeight.w500),
      ),
      const SizedBox(width: 10),
      custom_ui.Chip(label: widget.featureId, pill: true),
    ]);
  }

  // ---------------------------------------------------------------------------
  // Zona izquierda — editor del JSON de input.
  // ---------------------------------------------------------------------------

  Widget _buildInputZone() {
    return custom_ui.Surface(
      padding: const EdgeInsets.all(Gx.space12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Input JSON', style: Gx.microLabel),
          const SizedBox(height: Gx.space8),
          custom_ui.Textarea(
            controller: _inputCtrl,
            maxLines: 26,
            enabled: _state != _VerifyState.sending,
            hint: '{ ... }',
          ),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Zona central — botón Enviar.
  // ---------------------------------------------------------------------------

  Widget _buildActionZone() {
    final texto = _inputCtrl.text.trim();
    // Guarda de UX mínima (no es validación de negocio): no dispara con el
    // editor vacío. La validez real del JSON la decide el backend.
    final canSubmit = texto.isNotEmpty && _state != _VerifyState.sending;

    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text('Acción', style: Gx.microLabel),
          const SizedBox(height: Gx.space12),
          custom_ui.Button(
            label: _state == _VerifyState.sending ? 'Enviando...' : 'Enviar',
            onPressed: canSubmit ? _enviar : null,
            variant: custom_ui.ButtonVariant.primary,
            enabled: canSubmit,
            loading: _state == _VerifyState.sending,
          ),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Zona derecha — respuesta real del backend, read-only.
  // ---------------------------------------------------------------------------

  Widget _buildOutputZone() {
    return custom_ui.Surface(
      padding: const EdgeInsets.all(Gx.space12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Respuesta', style: Gx.microLabel),
          const SizedBox(height: Gx.space8),
          // REQUISITO CLAVE: estado inequívoco de forma del input, separado
          // del resultado del backend.
          _buildStatusIndicator(),
          const SizedBox(height: Gx.space8),
          Expanded(child: _buildOutputBody()),
        ],
      ),
    );
  }

  // Indicador de estado: idle / enviando / input inválido / éxito / error backend.
  Widget _buildStatusIndicator() {
    switch (_state) {
      case _VerifyState.idle:
        return custom_ui.Chip(label: 'Sin enviar aún', pill: true);

      case _VerifyState.sending:
        return Row(children: [
          const custom_ui.ProgressCircular(size: 16),
          const SizedBox(width: Gx.space8),
          custom_ui.Chip(
            label: 'Enviando...',
            status: custom_ui.ChipStatus.transition,
            pill: true,
          ),
        ]);

      case _VerifyState.invalidInput:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            custom_ui.Chip(
              label: 'Input inválido',
              status: custom_ui.ChipStatus.critical,
              pill: true,
            ),
            const SizedBox(height: Gx.space8),
            custom_ui.Banner(
              message: 'Input inválido para el puente FFI: ${_invalidReason ?? ''}',
              type: custom_ui.BannerType.error,
            ),
          ],
        );

      case _VerifyState.backendError:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Wrap(spacing: Gx.space8, children: [
              custom_ui.Chip(
                label: 'Input válido',
                status: custom_ui.ChipStatus.optima,
                pill: true,
              ),
              custom_ui.Chip(
                label: 'Backend: error',
                // Ámbar, NO rojo: un error de backend con input válido no se
                // corrige editando el JSON (regla FIJO de ADR-0152: distinguir
                // "input mal formado" de "input válido pero la operación falló").
                status: custom_ui.ChipStatus.alert,
                pill: true,
              ),
            ]),
            const SizedBox(height: Gx.space8),
            custom_ui.Banner(
              message: 'Error del backend: ${_backendError ?? ''}',
              type: custom_ui.BannerType.warning,
            ),
          ],
        );

      case _VerifyState.success:
        return Wrap(spacing: Gx.space8, children: [
          custom_ui.Chip(
            label: 'Input válido',
            status: custom_ui.ChipStatus.optima,
            pill: true,
          ),
          custom_ui.Chip(
            label: 'Backend: OK',
            status: custom_ui.ChipStatus.optima,
            pill: true,
          ),
        ]);
    }
  }

  // Cuerpo de la respuesta: JSON real del backend en bloque bloqueado
  // (read-only, seleccionable para copiar) o mensaje informativo si no hay
  // resultado todavía.
  Widget _buildOutputBody() {
    if (_outputJsonPretty == null) {
      return Text(
        'La respuesta del backend aparecerá aquí tras enviar.',
        style: Gx.uiSans(fontSize: 12, color: Gx.textBaseMuted),
      );
    }
    return SingleChildScrollView(
      child: SelectableText(
        _outputJsonPretty!,
        style: Gx.dataMono(fontSize: 12, color: Gx.textBase),
      ),
    );
  }
}
