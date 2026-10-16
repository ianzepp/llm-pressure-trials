#!/usr/bin/env bash
set -euo pipefail

# Runs naval-mercy / odds-matched; cohort file(s): october-models.txt, january-anchor-models.txt; expected trials: 43 models (33 october + 10 january anchors) x 4 conditions x 3 repeats = 516.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
models_file_0="${repo_root}/studies/mercy-dilemma/october-models.txt"
models_file_1="${repo_root}/studies/mercy-dilemma/january-anchor-models.txt"

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

if [[ ! -f "${models_file_1}" ]]; then
  echo "missing model cohort file: ${models_file_1}" >&2
  exit 1
fi

while IFS=$'\t' read -r model_id _rest; do
  if [[ -z "${model_id}" || "${model_id}" == \#* ]]; then
    continue
  fi
  model_args+=(--model "${model_id}")
done < "${models_file_1}"

if [[ ${#model_args[@]} -eq 0 ]]; then
  echo "no models found in cohort files" >&2
  exit 1
fi

echo "models: $((${#model_args[@]})) total"

cd "${repo_root}"
exec cargo run -- run \
  --study naval-mercy \
  --experiment odds-matched \
  --launch-interval-ms 100 \
  --high-water 48 \
  --low-water 24 \
  --continue-on-error \
  "${model_args[@]}" \
  "$@"
