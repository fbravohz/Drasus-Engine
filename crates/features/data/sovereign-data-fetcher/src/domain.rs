//! Functional Core del Sovereign Data Fetcher — lógica pura, sin I/O.
//!
//! PROHIBIDO: imports de `reqwest`, `tokio`, `std::fs`, `sqlx` o cualquier
//! dependencia que toque el sistema operativo o la red.
//! PERMITIDO: tipos de la librería estándar de Rust que sean puramente de datos
//! (`std::cmp`, `std::fmt`, colecciones, etc.).
//!
//! Toda función de este módulo es determinista: los mismos argumentos de
//! entrada producen exactamente la misma salida, sin efectos secundarios.

// ── Tipos de datos de dominio (puras estructuras de datos) ──────────────────

/// Representa un rango de tiempo como par de timestamps en nanosegundos
/// desde el Unix epoch (Jan 1, 1970 UTC).
///
/// Invariante: `start_ns <= end_ns`. Un rango vacío tiene `start_ns == end_ns`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    /// Inicio del rango, inclusive (nanosegundos).
    pub start_ns: i64,
    /// Fin del rango, inclusive (nanosegundos).
    pub end_ns: i64,
}

impl TimeRange {
    /// Crea un rango de tiempo. Devuelve `None` si `start_ns > end_ns`
    /// (un rango hacia atrás en el tiempo no tiene sentido).
    pub fn new(start_ns: i64, end_ns: i64) -> Option<Self> {
        if start_ns > end_ns {
            None
        } else {
            Some(Self { start_ns, end_ns })
        }
    }

    /// Indica si este rango cubre completamente al rango `other`.
    /// "Cubre" significa: `self.start_ns <= other.start_ns` Y `self.end_ns >= other.end_ns`.
    pub fn covers(&self, other: &TimeRange) -> bool {
        self.start_ns <= other.start_ns && self.end_ns >= other.end_ns
    }

    /// Indica si este rango se solapa (tiene intersección) con `other`.
    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start_ns <= other.end_ns && self.end_ns >= other.start_ns
    }
}

/// Metadatos de un archivo Bulk disponible para descarga.
///
/// Un archivo Bulk es un volcado comprimido (ej. `.zip`) publicado por el
/// broker en un bucket público. Cada archivo cubre un rango temporal concreto
/// (ej. un día o un mes de datos históricos).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkFileInfo {
    /// Nombre del archivo en el servidor remoto (ej. `BTCUSDT-1m-2024-01.zip`).
    pub filename: String,
    /// URL completa para descargar el archivo.
    pub download_url: String,
    /// Inicio del rango temporal que cubre el archivo (nanosegundos).
    pub start_ns: i64,
    /// Fin del rango temporal que cubre el archivo (nanosegundos).
    pub end_ns: i64,
    /// Tamaño estimado del archivo en bytes (para la verificación de disco).
    pub estimated_size_bytes: u64,
}

impl BulkFileInfo {
    /// Devuelve el rango temporal que cubre este archivo como un `TimeRange`.
    pub fn time_range(&self) -> TimeRange {
        // Es seguro construir directamente porque start_ns <= end_ns
        // siempre es invariante en datos de mercado (cronología válida).
        TimeRange {
            start_ns: self.start_ns,
            end_ns: self.end_ns,
        }
    }
}

/// El plan de descarga: qué archivos Bulk descargar y qué rango Delta
/// debe cubrir la API REST.
///
/// La lógica Bulk-first garantiza que el rango Delta es el mínimo residual
/// que el Bulk no puede cubrir — nunca se pide por REST lo que ya está en Bulk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadPlan {
    /// Archivos Bulk a descargar (en orden cronológico).
    pub bulk_files: Vec<BulkFileInfo>,
    /// Rango que la API REST debe cubrir. `None` si el Bulk cubre todo el
    /// rango solicitado.
    pub delta_range: Option<TimeRange>,
    /// Suma de bytes estimados de todos los archivos Bulk del plan.
    pub total_estimated_bytes: u64,
}

/// Resultado de la verificación de espacio en disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskSpaceResult {
    /// Hay espacio suficiente para continuar la descarga.
    Sufficient,
    /// No hay espacio suficiente. La descarga debe abortarse antes de empezar.
    Insufficient {
        /// Bytes que necesita el plan de descarga.
        required_bytes: u64,
        /// Bytes disponibles actualmente en disco.
        available_bytes: u64,
    },
}

// ── Funciones de dominio puras ───────────────────────────────────────────────

/// Calcula el rango Delta: el hueco temporal entre el fin del último archivo
/// Bulk (`bulk_cutoff_ns`) y el momento actual (`now_ns`).
///
/// La API REST solo debe cubrir este hueco — los datos anteriores a
/// `bulk_cutoff_ns` ya están cubiertos por los volcados Bulk.
///
/// Devuelve `None` si no hay hueco (el Bulk llega hasta "ahora" o más allá).
pub fn compute_delta_range(bulk_cutoff_ns: i64, now_ns: i64) -> Option<TimeRange> {
    // Si el Bulk ya llega al presente o más allá, no hay nada que pedir por REST.
    if bulk_cutoff_ns >= now_ns {
        return None;
    }
    // El rango Delta empieza justo después del último dato Bulk (+ 1 nanosegundo
    // para evitar duplicar el timestamp de corte).
    Some(TimeRange {
        start_ns: bulk_cutoff_ns + 1,
        end_ns: now_ns,
    })
}

/// Planifica la descarga combinada Bulk + Delta para el rango solicitado.
///
/// Estrategia Bulk-first (restricción invariante de la feature):
/// 1. Selecciona todos los archivos Bulk cuyo rango se solapa con `requested`.
/// 2. Calcula el rango residual que los Bulk no cubren (si lo hay).
/// 3. Ese residual es lo que Delta debe cubrir — nunca más.
///
/// Devuelve un `DownloadPlan` con la lista de archivos Bulk y el rango Delta
/// opcional.
pub fn plan_downloads(requested: TimeRange, bulk_inventory: &[BulkFileInfo]) -> DownloadPlan {
    // Selecciona los archivos Bulk que se solapan con el rango solicitado,
    // ordenados cronológicamente para procesar en secuencia.
    let mut relevant_files: Vec<&BulkFileInfo> = bulk_inventory
        .iter()
        .filter(|f| f.time_range().overlaps(&requested))
        .collect();
    relevant_files.sort_by_key(|f| f.start_ns);

    let total_estimated_bytes = relevant_files.iter().map(|f| f.estimated_size_bytes).sum();

    // Calcula hasta qué punto en el tiempo llegan los archivos Bulk seleccionados.
    // Si no hay archivos, el Bulk no cubre nada del rango solicitado.
    let bulk_coverage_end_ns = relevant_files.iter().map(|f| f.end_ns).max();

    // El rango Delta cubre el hueco entre el final del Bulk y el fin del rango
    // solicitado. Si el Bulk cubre todo (o no hay Bulk), ajustamos en consecuencia.
    let delta_range = match bulk_coverage_end_ns {
        // No hay archivos Bulk: Delta debe cubrir todo el rango solicitado.
        None => Some(requested),
        // El Bulk cubre hasta `cutoff_ns`. Si no llega al final del rango,
        // Delta cubre el tramo residual.
        Some(cutoff_ns) if cutoff_ns < requested.end_ns => Some(TimeRange {
            start_ns: cutoff_ns + 1,
            end_ns: requested.end_ns,
        }),
        // El Bulk cubre todo el rango solicitado; Delta no necesita nada.
        Some(_) => None,
    };

    DownloadPlan {
        bulk_files: relevant_files.into_iter().cloned().collect(),
        delta_range,
        total_estimated_bytes,
    }
}

/// Verifica si hay suficiente espacio en disco para ejecutar el plan de descarga.
///
/// Devuelve `DiskSpaceResult::Sufficient` si `available_bytes >= required_bytes`,
/// o `DiskSpaceResult::Insufficient` con los valores exactos en caso contrario.
pub fn check_disk_space(required_bytes: u64, available_bytes: u64) -> DiskSpaceResult {
    if available_bytes >= required_bytes {
        DiskSpaceResult::Sufficient
    } else {
        DiskSpaceResult::Insufficient {
            required_bytes,
            available_bytes,
        }
    }
}

/// Reconcilia el borde Bulk↔Delta: elimina timestamps duplicados de la unión
/// de los dos conjuntos de datos.
///
/// Ambos slices deben estar ordenados de forma ascendente (cronológicamente).
/// El resultado también está ordenado de forma ascendente y sin duplicados.
///
/// Esta función garantiza que ningún tick o barra aparezca dos veces al unir
/// los datos descargados del Bulk con los descargados por REST.
pub fn reconcile_boundary(bulk_timestamps: &[i64], delta_timestamps: &[i64]) -> Vec<i64> {
    // Precondición: ambos slices deben estar ordenados ascendentemente.
    // La fusión merge-sort produce resultados incorrectos con entradas desordenadas.
    // En builds de debug, verificamos activamente la invariante para detectar
    // errores del caller temprano; en release, asumimos que el caller la respeta.
    debug_assert!(
        bulk_timestamps.windows(2).all(|w| w[0] <= w[1]),
        "reconcile_boundary: bulk_timestamps debe estar ordenado ascendentemente"
    );
    debug_assert!(
        delta_timestamps.windows(2).all(|w| w[0] <= w[1]),
        "reconcile_boundary: delta_timestamps debe estar ordenado ascendentemente"
    );

    // Combina los dos slices y usa el proceso de fusión de dos listas ordenadas
    // (merge sort), eliminando duplicados en el paso de fusión.
    let mut result = Vec::with_capacity(bulk_timestamps.len() + delta_timestamps.len());
    let mut bulk_idx = 0;
    let mut delta_idx = 0;

    while bulk_idx < bulk_timestamps.len() && delta_idx < delta_timestamps.len() {
        let b = bulk_timestamps[bulk_idx];
        let d = delta_timestamps[delta_idx];

        if b < d {
            // El timestamp del Bulk es anterior: lo añadimos y avanzamos.
            result.push(b);
            bulk_idx += 1;
        } else if d < b {
            // El timestamp del Delta es anterior: lo añadimos y avanzamos.
            result.push(d);
            delta_idx += 1;
        } else {
            // Ambos son iguales (solapamiento en el borde): añadimos solo una
            // copia y avanzamos ambos índices para eliminar el duplicado.
            result.push(b);
            bulk_idx += 1;
            delta_idx += 1;
        }
    }

    // Añade el resto del Bulk (si sobra alguno).
    result.extend_from_slice(&bulk_timestamps[bulk_idx..]);
    // Añade el resto del Delta (si sobra alguno).
    result.extend_from_slice(&delta_timestamps[delta_idx..]);

    result
}

// ── Pruebas unitarias del núcleo puro ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Criterio 1: compute_delta_range ─────────────────────────────────

    /// Verifica que el rango Delta se calcula correctamente a partir del
    /// punto de corte del Bulk y el timestamp "ahora".
    ///
    /// Escenario: el Bulk llega hasta t=100, el presente es t=200.
    /// El Delta debe cubrir [101, 200].
    #[test]
    fn delta_range_computed_from_bulk_cutoff() {
        let bulk_cutoff_ns = 100_000_000_000i64; // nanosegundos
        let now_ns = 200_000_000_000i64;

        let range = compute_delta_range(bulk_cutoff_ns, now_ns);

        assert!(range.is_some(), "debe haber un rango Delta cuando el Bulk no llega al presente");
        let range = range.unwrap();
        // El Delta empieza 1 nanosegundo después del corte del Bulk.
        assert_eq!(range.start_ns, bulk_cutoff_ns + 1);
        assert_eq!(range.end_ns, now_ns);
    }

    /// Si el Bulk ya llega al presente (o más allá), no hay hueco que pedir por REST.
    #[test]
    fn delta_range_is_none_when_bulk_reaches_present() {
        let now_ns = 100_000_000_000i64;
        // El Bulk llega exactamente hasta "ahora".
        assert!(compute_delta_range(now_ns, now_ns).is_none());
        // El Bulk llega más allá de "ahora" (datos del futuro, edge case).
        assert!(compute_delta_range(now_ns + 1, now_ns).is_none());
    }

    // ── Criterio 2: plan_downloads (Bulk-first) ──────────────────────────

    /// Verifica que un tramo cubierto por archivos Bulk NUNCA se pide por REST.
    ///
    /// Escenario: se piden [0, 1000]. Hay un Bulk que cubre [0, 800].
    /// El Delta debe cubrir solo [801, 1000], nunca [0, 800].
    #[test]
    fn bulk_covered_range_never_uses_rest() {
        let requested = TimeRange { start_ns: 0, end_ns: 1_000 };
        let inventory = vec![BulkFileInfo {
            filename: "bulk-jan.zip".to_string(),
            download_url: "https://example.com/bulk-jan.zip".to_string(),
            start_ns: 0,
            end_ns: 800,
            estimated_size_bytes: 1024,
        }];

        let plan = plan_downloads(requested, &inventory);

        // El Bulk cubre [0, 800] — exactamente ese archivo en el plan.
        assert_eq!(plan.bulk_files.len(), 1);
        assert_eq!(plan.bulk_files[0].filename, "bulk-jan.zip");

        // Delta solo cubre el residuo [801, 1000].
        assert!(plan.delta_range.is_some());
        let delta = plan.delta_range.unwrap();
        // El Delta no empieza desde 0 (eso ya lo cubrió el Bulk).
        assert!(delta.start_ns > 800, "Delta no debe solaparse con el Bulk: start={}", delta.start_ns);
        assert_eq!(delta.end_ns, 1_000);
    }

    /// Si el Bulk cubre todo el rango solicitado, Delta es None.
    #[test]
    fn delta_is_none_when_bulk_covers_full_range() {
        let requested = TimeRange { start_ns: 0, end_ns: 500 };
        let inventory = vec![BulkFileInfo {
            filename: "bulk-total.zip".to_string(),
            download_url: "https://example.com/bulk.zip".to_string(),
            start_ns: 0,
            end_ns: 1_000, // Cubre más que lo solicitado.
            estimated_size_bytes: 2048,
        }];

        let plan = plan_downloads(requested, &inventory);

        assert_eq!(plan.bulk_files.len(), 1);
        // No hay residuo para Delta.
        assert!(plan.delta_range.is_none(), "Delta debe ser None cuando el Bulk cubre todo");
    }

    /// Si no hay archivos Bulk, Delta debe cubrir todo el rango solicitado.
    #[test]
    fn delta_covers_full_range_when_no_bulk_available() {
        let requested = TimeRange { start_ns: 100, end_ns: 900 };
        let plan = plan_downloads(requested, &[]);

        assert!(plan.bulk_files.is_empty());
        assert_eq!(plan.delta_range, Some(requested));
        assert_eq!(plan.total_estimated_bytes, 0);
    }

    // ── Criterio 3: check_disk_space ────────────────────────────────────

    /// Verifica que la ingesta aborta antes de descargar cuando el espacio
    /// en disco es insuficiente.
    #[test]
    fn ingest_aborts_when_disk_insufficient() {
        let required = 1_000_000u64; // 1 MB requerido
        let available = 500_000u64;  // solo 500 KB disponibles

        let result = check_disk_space(required, available);

        assert_eq!(
            result,
            DiskSpaceResult::Insufficient {
                required_bytes: required,
                available_bytes: available,
            },
            "debe detectar espacio insuficiente"
        );
    }

    /// Con espacio justo suficiente, la descarga puede proceder.
    #[test]
    fn disk_check_passes_when_space_is_exactly_sufficient() {
        let bytes = 500_000u64;
        // Exactamente suficiente (igual, no mayor).
        assert_eq!(check_disk_space(bytes, bytes), DiskSpaceResult::Sufficient);
    }

    // ── Criterio 4: reconcile_boundary ──────────────────────────────────

    /// Verifica que el borde Bulk↔Delta queda sin timestamps duplicados.
    ///
    /// Escenario: el Bulk tiene [10, 20, 30, 40] y el Delta tiene [30, 40, 50, 60].
    /// Hay solapamiento en [30, 40]. El resultado debe ser [10, 20, 30, 40, 50, 60].
    #[test]
    fn bulk_delta_boundary_has_no_duplicates() {
        let bulk = vec![10i64, 20, 30, 40];
        let delta = vec![30i64, 40, 50, 60];

        let merged = reconcile_boundary(&bulk, &delta);

        // El resultado debe estar ordenado y sin duplicados.
        assert_eq!(merged, vec![10i64, 20, 30, 40, 50, 60]);
        // Verificar explícitamente que no hay duplicados.
        let unique_count = merged.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, merged.len(), "no debe haber timestamps duplicados");
    }

    /// Caso base: ambos slices sin solapamiento, el resultado es la unión ordenada.
    #[test]
    fn reconcile_non_overlapping_ranges_merges_correctly() {
        let bulk = vec![1i64, 2, 3];
        let delta = vec![4i64, 5, 6];
        assert_eq!(reconcile_boundary(&bulk, &delta), vec![1i64, 2, 3, 4, 5, 6]);
    }

    /// Caso extremo: el Delta es vacío; el resultado es solo el Bulk.
    #[test]
    fn reconcile_with_empty_delta_returns_bulk() {
        let bulk = vec![1i64, 2, 3];
        assert_eq!(reconcile_boundary(&bulk, &[]), vec![1i64, 2, 3]);
    }

    /// Caso extremo: el Bulk es vacío; el resultado es solo el Delta.
    #[test]
    fn reconcile_with_empty_bulk_returns_delta() {
        let delta = vec![4i64, 5, 6];
        assert_eq!(reconcile_boundary(&[], &delta), vec![4i64, 5, 6]);
    }

    // ── Pruebas auxiliares de TimeRange ─────────────────────────────────

    /// `TimeRange::new` rechaza rangos hacia atrás en el tiempo.
    #[test]
    fn time_range_rejects_inverted_range() {
        assert!(TimeRange::new(100, 50).is_none());
        assert!(TimeRange::new(100, 100).is_some()); // rango vacío permitido
    }

    /// `TimeRange::covers` identifica correctamente la cobertura total.
    #[test]
    fn time_range_covers_detects_full_coverage() {
        let wide = TimeRange { start_ns: 0, end_ns: 1000 };
        let narrow = TimeRange { start_ns: 100, end_ns: 900 };
        assert!(wide.covers(&narrow));
        assert!(!narrow.covers(&wide));
    }
}
