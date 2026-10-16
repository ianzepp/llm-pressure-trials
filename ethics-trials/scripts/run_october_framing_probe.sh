#!/usr/bin/env bash
set -euo pipefail

# Runs drone-doctrine / framing-probe; cohort file(s): october-models.txt; expected trials: 33 models x 5 units x 3 repeats = 495.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
models_file_0="${repo_root}/studies/mercy-dilemma/october-models.txt"

model_args=()
if [[ ! -f "${models_file_0}" ]]; then
  echo "missing model cohort file: ${models_file_0}" >&2
  exit 1
fi

while IFS=$'\t' read -r model_id _rest; do
  if [[ -z "${model_id}" || "${model_id}" == \#* ]]; then
    continue
  fi
  model_args+=(--model "${model_id}")
done < "${models_file_0}"

if [[ ${#model_args[@]} -eq 0 ]]; then
  echo "no models found in cohort files" >&2
  exit 1
fi

echo "models: $((${#model_args[@]})) total"

cd "${repo_root}"
exec cargo run -- run \
  --study drone-doctrine \
  --experiment framing-probe \
  --launch-interval-ms 100 \
  --high-water 48 \
  --low-water 24 \
  --continue-on-error \
  "${model_args[@]}" \
  "$@"
