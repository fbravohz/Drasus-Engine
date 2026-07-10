# Lecciones — Glass, Performance y Temas (2026-06-25)

## Glassmorphism Apple sobre fondo oscuro

**Problema:** El vidrio `#141C36 @ 50-85%` (azul oscuro translúcido) sobre fondo `#080A18` (casi negro) producía una suma más oscura aún — el vidrio era indistinguible del fondo.

**Solución:** El vidrio Apple sobre fondo oscuro requiere un **tinte CLARO**. El fill se cambió a `0x40F0F2FF` (blanco frío ~10%). La física es: fondo oscuro + tinte claro = contraste visible. Sobre fondo claro (slate/paper) el vidrio debería ser inverso (tinte oscuro a baja opacidad).

**Regla canónica:** `glassFill` es un color CLARO a ~10-15% de opacidad. NUNCA se multiplica con `.withOpacity()` — el alpha ya es el correcto. Todos los usos de vidrio (`frosted()`, `GlassSurface`, inline `BackdropFilter`) deben usar `glassFill` directamente sin modificar su opacidad.

**Error común corregido:** `frosted()` aplicaba `.withOpacity(0.85)` y `.withOpacity(0.5)` sobre `glassFill`, reduciendo el alpha efectivo de ~10% a ~8.5% y ~5% respectivamente — invisible. Se eliminó toda multiplicación de opacidad.

**Patrón estandarizado (frosted + GlassSurface usan los mismos valores):**
- fill: `0x40F0F2FF`
- blur: `36.0`
- tint: `0x14AAAAFF` → transparent
- rim color: `0x20A096FF`
- edge opacity: `0.28`

## Performance de CustomPainter

**Problema:** Cluster 3D con 5000 puntos + saveLayer(blur 8) + Monte Carlo con MaskFilter.blur por segmento = ~145ms/frame (UI 45ms, Raster ~100ms).

**Optimizaciones aplicadas:**
1. **Eliminar saveLayer + ImageFilter.blur del Cluster.** El blur de nebulosa costaba ~90ms GPU. Reemplazado por círculos grandes (r=4px) a muy baja opacidad (12%) — mismo efecto de nube de polvo estelar, cero offscreen buffer.
2. **Eliminar MaskFilter.blur por segmento en MC.** 4800 blur ops/frame reemplazadas por líneas más gruesas a baja opacidad. Sin blur → sin convolución gaussiana por segmento.
3. **Eliminar MaskFilter.blur del scan line.** Misma optimización: halo de 3 capas (20px/6px/1.5px) a distintas opacidades produce el mismo glow sin GPU blur.
4. **Caché de proyección 3D en State.** Los 5000 puntos se proyectan y ordenan una vez por frame en el State (no en paint()). El painter solo itera y dibuja — sin trigonometría ni sort.

**Resultado:** UI 45ms → 15ms, Raster 100ms → 23ms (profile). En release: 60fps estables.

**Regla canónica de performance:**
- `saveLayer` + `ImageFilter.blur` sobre lienzo grande = prohibido en animación (cuesta ~90ms).
- `MaskFilter.blur` por elemento en loops de >100 iteraciones = prohibido. Usar capas de grosor/opacidad.
- Proyección matemática en `paint()` = mover al State con caché por ángulo.
- `drawPicture()` con Picture pre-grabado = la forma más barata de renderizar fondos estáticos.

## Sistema de temas dinámicos

**Problema:** `DrasusThemeState` y `SettingsDrawer` existían como archivos pero no estaban cableados a ningún entrypoint. La galería (`gallery_preview_main.dart`) montaba `GalleryTab` en un `MaterialApp` desnudo sin tema.

**Solución:** Se cableó `DrasusTheme` + `DrasusThemeState` tanto al entrypoint principal (`main.dart`) como al de la galería (`gallery_preview_main.dart`). `SettingsDrawer` se monta como `endDrawer` en ambos. El `CosmicBackdropPainter` lee `deepSpace` dinámico desde `DrasusTheme.of(context)` — cambia en tiempo real con la paleta.

**Lección de drawer:** `Scaffold.of(context).openEndDrawer()` falla si `context` está por encima del `Scaffold`. Solución: `GlobalKey<ScaffoldState>` o `Builder` para capturar contexto dentro del Scaffold.

## Fuentes y tipografía

**Problema:** Muchos `TextStyle(...)` inline sin `fontFamily` — usaban la fuente del sistema en vez de Space Grotesk/Inter/JetBrains Mono. El título _hero_ de la galería no usaba Space Grotesk.

**Solución:** Los helpers `Gx.displayGrotesque()`, `Gx.uiSans()`, `Gx.dataMono()` son la ÚNICA forma de crear `TextStyle` en la galería. Todo `TextStyle` inline debe ser reemplazado por llamadas a estos helpers. El cambio más notorio: hero title usa `Gx.zuiTitle.copyWith(color: Colors.white)`.

## Monte Carlo — percentiles

**Problema:** `_percentileLine(p)` hacía 180 sorts de `_rawTrajectories.length` elementos (60 pasos × 3 percentiles). Para 10K líneas: ~24M comparaciones en el main thread sincrónico.

**Solución:** `_computePercentiles()` usa un solo sort por equity final y toma las trayectorias en los índices p5/p50/p95 como aproximación. 180× más rápido, visualmente indistinguible.
