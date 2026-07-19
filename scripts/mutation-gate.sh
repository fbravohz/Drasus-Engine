#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Gate de mutación (ADR-0133 capa 8) — punto de entrada ÚNICO y determinista.
#
# Uso:
#   scripts/mutation-gate.sh <crate> [jobs]
#   scripts/mutation-gate.sh hybrid-data-transformer
#   scripts/mutation-gate.sh sovereign-data-fetcher 4
#
# Por qué existe este script en vez de escribir el comando a mano:
#
#   `cargo-mutants` copia el árbol de fuentes COMPLETO al directorio temporal
#   del sistema y compila ahí una copia por cada trabajo en paralelo. En esta
#   máquina `/tmp` es tmpfs — es decir, RAM disfrazada de disco. El árbol
#   arrastra Polars, cuyos binarios de depuración pesan cientos de MB y cuyo
#   enlazado necesita varios GB de memoria de trabajo por proceso. Con el
#   paralelismo por defecto (= número de núcleos) arrancan tantos enlazadores
#   simultáneos como núcleos haya, se agota la RAM, y el enlazador muere con
#   `ld terminated with signal 7 [Bus error]`.
#
#   Ese error APARENTA un fallo de compilación grave y cuesta un buen rato
#   diagnosticar que en realidad era falta de memoria (ocurrió el 2026-07-18).
#
# Las tres medidas que aplica este script:
#   1. TMPDIR en disco real (NVMe), no en tmpfs.
#   2. Paralelismo acotado — por defecto 2 enlazadores concurrentes.
#   3. Perfil sin símbolos de depuración: los binarios de mutación no se
#      depuran nunca, y quitar la info de debug reduce drásticamente tanto el
#      tamaño como la memoria que el enlazador necesita.
#
# Salida: imprime el resumen y deja el detalle en el directorio de resultados.
# Código de salida: 0 si no hay supervivientes; 1 si los hay (gate NO APTO).
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

CRATE="${1:?Uso: scripts/mutation-gate.sh <crate> [jobs]}"
JOBS="${2:-2}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Base en disco real. Se deriva de $HOME para no clavar la ruta de un usuario.
MUTANTS_BASE="${HOME}/.cache/drasus-mutants"
RUN_DIR="${MUTANTS_BASE}/${CRATE}"

mkdir -p "${MUTANTS_BASE}/tmp" "${RUN_DIR}"

# Comprobación explícita: si la base cayera en tmpfs, abortamos con un mensaje
# que explica el problema, en vez de dejar que el enlazador muera sin contexto.
FS_TYPE="$(df --output=fstype "${MUTANTS_BASE}" | tail -1 | tr -d ' ')"
if [[ "${FS_TYPE}" == "tmpfs" ]]; then
    echo "ABORTADO: ${MUTANTS_BASE} está en tmpfs (RAM), no en disco real." >&2
    echo "El enlazado de Polars agotaría la memoria y moriría con Bus error." >&2
    exit 2
fi

echo "── Gate de mutación · crate '${CRATE}' · ${JOBS} trabajos en paralelo ──"
echo "   Temporales en: ${MUTANTS_BASE}/tmp  (${FS_TYPE})"
echo "   Resultados en: ${RUN_DIR}"
echo

cd "${REPO_ROOT}"

set +e
TMPDIR="${MUTANTS_BASE}/tmp" \
CARGO_PROFILE_DEV_DEBUG=0 \
CARGO_PROFILE_TEST_DEBUG=0 \
    cargo mutants -p "${CRATE}" --jobs "${JOBS}" --output "${RUN_DIR}"
MUTANTS_EXIT=$?
set -e

MISSED_FILE="${RUN_DIR}/mutants.out/missed.txt"
MISSED_COUNT=0
[[ -f "${MISSED_FILE}" ]] && MISSED_COUNT="$(wc -l < "${MISSED_FILE}" | tr -d ' ')"

echo
echo "──────────────────────────────────────────────────────────────"
if [[ "${MISSED_COUNT}" -eq 0 ]]; then
    echo "GATE APTO · 0 supervivientes en '${CRATE}'"
    exit 0
fi

echo "GATE NO APTO · ${MISSED_COUNT} supervivientes en '${CRATE}':"
echo
sed "s|crates/[^ ]*/src/||; s| in [0-9].*||" "${MISSED_FILE}"
echo
echo "Detalle completo: ${MISSED_FILE}"
echo
echo "Recordatorio: un superviviente solo se acepta como EQUIVALENTE con una"
echo "demostración escrita de por qué ninguna prueba puede matarlo (el bit que"
echo "la mutación altera no alcanza el valor de retorno). 'Es trivial' NO vale."
exit 1
