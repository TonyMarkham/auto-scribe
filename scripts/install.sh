#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  ./install.sh
  ./install.sh auto-scribe-linux-x86_64-cuda.tar.gz

Run without arguments from an extracted Auto Scribe release directory, or pass
the release archive path and this script will extract it before installing.
USAGE
}

resolve_data_home() {
    if [[ -n "${XDG_DATA_HOME:-}" ]]; then
        printf '%s\n' "${XDG_DATA_HOME}"
        return
    fi

    if [[ -z "${HOME:-}" ]]; then
        echo "error: HOME is not set and XDG_DATA_HOME is empty" >&2
        exit 1
    fi

    printf '%s\n' "${HOME}/.local/share"
}

install_file() {
    local source_path="$1"
    local target_path="$2"
    local mode="$3"
    local temp_path="${target_path}.tmp.$$"

    cp "${source_path}" "${temp_path}"
    chmod "${mode}" "${temp_path}"
    mv "${temp_path}" "${target_path}"
}

source_dir_from_archive() {
    local archive_path="$1"
    local extract_dir="$2"

    tar -xzf "${archive_path}" -C "${extract_dir}"
    find "${extract_dir}" -mindepth 1 -maxdepth 1 -type d -print -quit
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if [[ "$#" -gt 1 ]]; then
    usage >&2
    exit 1
fi

temp_dir=""
if [[ "$#" -eq 1 ]]; then
    temp_dir="$(mktemp -d)"
    trap 'rm -rf "${temp_dir}"' EXIT
    source_dir="$(source_dir_from_archive "$1" "${temp_dir}")"
else
    source_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
fi

if [[ -z "${source_dir}" || ! -x "${source_dir}/auto-scribe" ]]; then
    echo "error: auto-scribe executable was not found in the release payload" >&2
    exit 1
fi

shopt -s nullglob
provider_libs=("${source_dir}"/libonnxruntime*.so*)
if [[ "${#provider_libs[@]}" -eq 0 ]]; then
    echo "error: no libonnxruntime*.so* files were found in the release payload" >&2
    exit 1
fi

bin_dir="$(resolve_data_home)/auto-scribe/bin"
mkdir -p "${bin_dir}"

install_file "${source_dir}/auto-scribe" "${bin_dir}/auto-scribe" 755

for lib_path in "${provider_libs[@]}"; do
    install_file "${lib_path}" "${bin_dir}/$(basename "${lib_path}")" 644
done

echo "Installed Auto Scribe to ${bin_dir}"
echo "Run with: ${bin_dir}/auto-scribe"
