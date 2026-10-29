#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
tooling_dir="$(dirname -- "$project_dir")/.tooling"
if [[ -x "$tooling_dir/cargo/bin/cargo" ]]; then
  export CARGO_HOME="$tooling_dir/cargo"
  export RUSTUP_HOME="$tooling_dir/rustup"
  export PATH="$CARGO_HOME/bin:$PATH"
fi
cd -- "$project_dir"
exec cargo "$@"
