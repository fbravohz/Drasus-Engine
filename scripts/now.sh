#!/usr/bin/env bash
# Reloj de pared determinista para el Tech-Lead.
# Lee el reloj del sistema (fuente autoritativa) y lo imprime en los formatos que
# necesito para NO estimar mal un reset de límite de sesión ni la fecha de una entrada.
# Uso: scripts/now.sh
set -euo pipefail
echo "UTC:            $(date -u +'%Y-%m-%d %H:%M:%S %Z')"
echo "Mexico_City:    $(TZ=America/Mexico_City date +'%Y-%m-%d %H:%M:%S %Z')"
echo "ISO-8601 (UTC): $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
echo "epoch:          $(date +%s)"
