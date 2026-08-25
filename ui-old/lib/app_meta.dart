// Metadatos visibles de la aplicación — fuente única del nombre de producto.
//
// El nombre del producto NO se hardcodea disperso por la UI: vive aquí para
// que un rebautizo (p.ej. Drasus → Titan → Vectron) sea una sola línea.
// Alinea con la neutralidad de nombre de la librería de componentes (ADR-0138):
// nada del chrome de producto debe atar el código al nombre actual.

/// Nombre visible del producto (barra de título, cabeceras, "acerca de").
const String kAppName = 'Drasus Engine';

/// Versión visible del producto.
const String kAppVersion = '0.1.0-α';

/// Nombre de exhibición del lienzo visual (breadcrumb, títulos del canvas).
///
/// El CÓDIGO y las variables del lienzo permanecen neutrales ("canvas"); solo
/// este string de marca cambia. Valor de marca TBD (ADR-0136, enmienda
/// 2026-07-11): "Forge" es el placeholder actual — este es el punto ÚNICO donde
/// se cambia, jamás horneado disperso.
const String kCanvasName = 'Forge';
