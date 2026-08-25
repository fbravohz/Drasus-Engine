// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'verification.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$InputStatus {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is InputStatus);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'InputStatus()';
  }
}

/// @nodoc
class $InputStatusCopyWith<$Res> {
  $InputStatusCopyWith(InputStatus _, $Res Function(InputStatus) __);
}

/// Adds pattern-matching-related methods to [InputStatus].
extension InputStatusPatterns on InputStatus {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(InputStatus_Valid value)? valid,
    TResult Function(InputStatus_Invalid value)? invalid,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case InputStatus_Valid() when valid != null:
        return valid(_that);
      case InputStatus_Invalid() when invalid != null:
        return invalid(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(InputStatus_Valid value) valid,
    required TResult Function(InputStatus_Invalid value) invalid,
  }) {
    final _that = this;
    switch (_that) {
      case InputStatus_Valid():
        return valid(_that);
      case InputStatus_Invalid():
        return invalid(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(InputStatus_Valid value)? valid,
    TResult? Function(InputStatus_Invalid value)? invalid,
  }) {
    final _that = this;
    switch (_that) {
      case InputStatus_Valid() when valid != null:
        return valid(_that);
      case InputStatus_Invalid() when invalid != null:
        return invalid(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? valid,
    TResult Function(String reason)? invalid,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case InputStatus_Valid() when valid != null:
        return valid();
      case InputStatus_Invalid() when invalid != null:
        return invalid(_that.reason);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() valid,
    required TResult Function(String reason) invalid,
  }) {
    final _that = this;
    switch (_that) {
      case InputStatus_Valid():
        return valid();
      case InputStatus_Invalid():
        return invalid(_that.reason);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? valid,
    TResult? Function(String reason)? invalid,
  }) {
    final _that = this;
    switch (_that) {
      case InputStatus_Valid() when valid != null:
        return valid();
      case InputStatus_Invalid() when invalid != null:
        return invalid(_that.reason);
      case _:
        return null;
    }
  }
}

/// @nodoc

class InputStatus_Valid extends InputStatus {
  const InputStatus_Valid() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is InputStatus_Valid);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'InputStatus.valid()';
  }
}

/// @nodoc

class InputStatus_Invalid extends InputStatus {
  const InputStatus_Invalid({required this.reason}) : super._();

  final String reason;

  /// Create a copy of InputStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $InputStatus_InvalidCopyWith<InputStatus_Invalid> get copyWith =>
      _$InputStatus_InvalidCopyWithImpl<InputStatus_Invalid>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is InputStatus_Invalid &&
            (identical(other.reason, reason) || other.reason == reason));
  }

  @override
  int get hashCode => Object.hash(runtimeType, reason);

  @override
  String toString() {
    return 'InputStatus.invalid(reason: $reason)';
  }
}

/// @nodoc
abstract mixin class $InputStatus_InvalidCopyWith<$Res>
    implements $InputStatusCopyWith<$Res> {
  factory $InputStatus_InvalidCopyWith(
          InputStatus_Invalid value, $Res Function(InputStatus_Invalid) _then) =
      _$InputStatus_InvalidCopyWithImpl;
  @useResult
  $Res call({String reason});
}

/// @nodoc
class _$InputStatus_InvalidCopyWithImpl<$Res>
    implements $InputStatus_InvalidCopyWith<$Res> {
  _$InputStatus_InvalidCopyWithImpl(this._self, this._then);

  final InputStatus_Invalid _self;
  final $Res Function(InputStatus_Invalid) _then;

  /// Create a copy of InputStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? reason = null,
  }) {
    return _then(InputStatus_Invalid(
      reason: null == reason
          ? _self.reason
          : reason // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

// dart format on
