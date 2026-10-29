#!/usr/bin/env bash
# Local stdio entry point. stdout belongs exclusively to MCP.
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
runtime_dir="$project_dir/.compass/mcp"
if (( $# > 1 )); then
  echo 'Usage: compass-mcp.sh [config.json]' >&2
  exit 2
fi
config_path="${1:-$runtime_dir/config.json}"
if [[ ! -x "$runtime_dir/bin/compass" || ! -r "$config_path" ]]; then
  echo 'Compass MCP runtime or config missing; see docs/MCP-INTEGRACAO.md.' >&2
  exit 2
fi
exec "$runtime_dir/bin/compass" mcp --config "$config_path"
