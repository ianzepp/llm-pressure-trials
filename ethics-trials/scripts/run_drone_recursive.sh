#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
models_file="${repo_root}/studies/mercy-dilemma/broad-models.txt"

if [[ ! -f "${models_file}" ]]; then
  echo "missing model cohort file: ${models_file}" >&2
  exit 1
fi

model_args=()
while IFS=$'\t' read -r model_id _rest; do
  if [[ -z "${model_id}" || "${model_id}" == \#* ]]; then
    continue
  fi
  model_args+=(--model "${model_id}")
done < "${models_file}"

if [[ ${#model_args[@]} -eq 0 ]]; then
  echo "no models found in ${models_file}" >&2
  exit 1
fi

echo "models: ${#model_args[@]} total"

cd "${repo_root}"
exec cargo run -- run \
  --study drone-doctrine \
  --experiment recursive \
  --launch-interval-ms 100 \
  --high-water 48 \
  --low-water 24 \
  --continue-on-error \
  --judge \
  "${model_args[@]}" \
  "$@"
