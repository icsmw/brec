#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

dry_run=1
execute=0
dry_run_requested=0
execute_requested=0
wait_seconds=30
start_from=""
only=""
skip_published=1
registry=""
registry_token="${CARGO_REGISTRY_TOKEN:-}"

publish_args=()

usage() {
    cat <<'USAGE'
Usage:
  scripts/cargo/publish.sh [options]

Options:
  --dry-run              Run cargo publish with --dry-run. This is the default.
  --execute              Publish crates for real.
  --no-skip-published    Try publishing even if crate version exists already.
  --allow-dirty          Pass --allow-dirty to cargo publish.
  --no-verify            Pass --no-verify to cargo publish.
  --registry NAME        Publish to a named registry.
  --token TOKEN          Pass an explicit registry token.
  --wait SECONDS         Wait between real publishes. Default: 30.
  --start-from CRATE     Skip crates before CRATE.
  --only CRATE           Publish only one crate.
  -h, --help             Show this help.
USAGE
}

while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --dry-run)
            dry_run_requested=1
            dry_run=1
            shift
            ;;
        --execute)
            execute_requested=1
            execute=1
            dry_run=0
            shift
            ;;
        --no-skip-published)
            skip_published=0
            shift
            ;;
        --allow-dirty)
            publish_args+=(--allow-dirty)
            shift
            ;;
        --no-verify)
            publish_args+=(--no-verify)
            shift
            ;;
        --registry)
            if [[ "$#" -lt 2 ]]; then
                echo "Missing value for --registry"
                exit 1
            fi
            registry="$2"
            publish_args+=(--registry "$2")
            shift 2
            ;;
        --token)
            if [[ "$#" -lt 2 ]]; then
                echo "Missing value for --token"
                exit 1
            fi
            registry_token="$2"
            publish_args+=(--token "$2")
            shift 2
            ;;
        --wait)
            if [[ "$#" -lt 2 ]]; then
                echo "Missing value for --wait"
                exit 1
            fi
            wait_seconds="$2"
            shift 2
            ;;
        --start-from)
            if [[ "$#" -lt 2 ]]; then
                echo "Missing value for --start-from"
                exit 1
            fi
            start_from="$2"
            shift 2
            ;;
        --only)
            if [[ "$#" -lt 2 ]]; then
                echo "Missing value for --only"
                exit 1
            fi
            only="$2"
            shift 2
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1"
            usage
            exit 1
            ;;
    esac
done

if [[ "$execute_requested" == "1" && "$dry_run_requested" == "1" ]]; then
    echo "--execute and --dry-run cannot be used together"
    exit 1
fi

if [[ "$execute" == "1" && "$skip_published" == "1" && -n "$registry" ]]; then
    echo "Automatic published-version checks are supported only for crates.io"
    echo "Use --no-skip-published with --registry if you want to publish to a custom registry"
    exit 1
fi

if ! [[ "$wait_seconds" =~ ^[0-9]+$ ]]; then
    echo "--wait must be a non-negative integer"
    exit 1
fi

if [[ "$dry_run" == "1" ]]; then
    publish_args+=(--dry-run)
fi

crates=(
    "brec_consts:lib/consts"
    "brec_macros_parser:generator/parser"
    "brec_scheme:generator/scheme"
    "brec_inter_tools:integration/tools"

    "brec_node_gen:integration/node/gen"
    "brec_node_gen_macro:integration/node/macro"
    "brec_node_lib:integration/node/lib"

    "brec_csharp_gen:integration/csharp/gen"
    "brec_csharp_lib:integration/csharp/lib"

    "brec_java_gen:integration/java/gen"
    "brec_java_gen_macro:integration/java/macro"
    "brec_java_lib:integration/java/lib"

    "brec_wasm_gen:integration/wasm/gen"
    "brec_wasm_gen_macro:integration/wasm/macro"
    "brec_wasm_lib:integration/wasm/lib"

    "brec_macros:generator/macros"
    "brec:lib/core"

    "brec_node_cli:integration/node/cli"
    "brec_csharp_cli:integration/csharp/cli"
    "brec_java_cli:integration/java/cli"
    "brec_wasm_cli:integration/wasm/cli"
)

dry_run_patch_args_for() {
    local current="$1"
    local patch_entry
    local patch_name
    local patch_path

    if [[ "$dry_run" != "1" ]]; then
        return
    fi

    for patch_entry in "${crates[@]}"; do
        patch_name="${patch_entry%%:*}"
        patch_path="${patch_entry#*:}"

        if [[ "$patch_name" == "$current" ]]; then
            break
        fi

        printf '%s\n' --config "patch.crates-io.${patch_name}.path=\"${patch_path}\""
    done
}

crate_version_from_manifest() {
    local manifest="$1"
    sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$manifest" | head -n 1
}

crate_version_is_published() {
    local name="$1"
    local version="$2"
    local status
    local curl_args=()

    if [[ "$skip_published" != "1" || "$dry_run" == "1" ]]; then
        return 1
    fi

    if ! command -v curl >/dev/null 2>&1; then
        echo "curl is required to check whether $name v$version is already published"
        exit 1
    fi

    curl_args=(
        --silent
        --show-error
        --location
        --user-agent "brec-publish-script/1.0"
        --header "Accept: application/json"
        --output /dev/null
        --write-out "%{http_code}"
    )
    if [[ -n "$registry_token" ]]; then
        curl_args+=(--header "Authorization: ${registry_token}")
    fi

    status="$(curl "${curl_args[@]}" "https://crates.io/api/v1/crates/${name}/${version}")"

    case "$status" in
        200)
            return 0
            ;;
        404)
            return 1
            ;;
        *)
            echo "Failed to check $name v$version on crates.io: HTTP $status"
            exit 1
            ;;
    esac
}

selected=()
started=0
if [[ -z "$start_from" ]]; then
    started=1
fi

for entry in "${crates[@]}"; do
    name="${entry%%:*}"

    if [[ "$started" == "0" ]]; then
        if [[ "$name" == "$start_from" ]]; then
            started=1
        else
            continue
        fi
    fi

    if [[ -n "$only" && "$name" != "$only" ]]; then
        continue
    fi

    selected+=("$entry")
done

if [[ "${#selected[@]}" == "0" ]]; then
    echo "No crates selected"
    exit 1
fi

for index in "${!selected[@]}"; do
    entry="${selected[$index]}"
    name="${entry%%:*}"
    path="${entry#*:}"
    manifest="$path/Cargo.toml"
    version="$(crate_version_from_manifest "$manifest")"

    if [[ -z "$version" ]]; then
        echo "Cannot detect package version in $manifest"
        exit 1
    fi

    if crate_version_is_published "$name" "$version"; then
        echo "Skipping $name v$version: already published"
        continue
    fi

    echo "Publishing $name from $path"
    mapfile -t dry_run_patch_args < <(dry_run_patch_args_for "$name")
    cargo publish --manifest-path "$manifest" "${publish_args[@]}" "${dry_run_patch_args[@]}"

    if [[ "$dry_run" == "0" && "$index" -lt "$((${#selected[@]} - 1))" ]]; then
        echo "Waiting ${wait_seconds}s for registry index propagation"
        sleep "$wait_seconds"
    fi
done
