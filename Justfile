set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

semantic_graph_extract := ".refactor-radar/bin/semantic-graph-extract"
rust_workspace := "rust-workspace"
fts := "fts"
release_name := "auto-scribe-linux-x86_64-cuda"

confidence:
    clear
    cargo fmt
    @just --justfile {{justfile()}} seperator
    cargo check
    @just --justfile {{justfile()}} seperator
    cargo clippy --all-targets -- -D warnings
    @just --justfile {{justfile()}} seperator
    cargo build
    @just --justfile {{justfile()}} seperator
    cargo build --release
    @just --justfile {{justfile()}} seperator
    cargo test
    @just --justfile {{justfile()}} seperator
    {{semantic_graph_extract}} {{rust_workspace}}
    @just --justfile {{justfile()}} seperator
    {{semantic_graph_extract}} {{fts}}
    @just --justfile {{justfile()}} seperator
    @just --justfile {{justfile()}} publish

refresh:
    just --justfile {{justfile()}} seperator
    {{semantic_graph_extract}} {{rust_workspace}}
    just --justfile {{justfile()}} seperator
    {{semantic_graph_extract}} {{fts}}
    just --justfile {{justfile()}} seperator

publish:
    mkdir -p /home/tony/.local/share/auto-scribe/bin
    cp target/release/auto-scribe /home/tony/.local/share/auto-scribe/bin/auto-scribe.tmp
    mv /home/tony/.local/share/auto-scribe/bin/auto-scribe.tmp /home/tony/.local/share/auto-scribe/bin/auto-scribe
    for lib in target/release/libonnxruntime*.so*; do cp "${lib}" "/home/tony/.local/share/auto-scribe/bin/$(basename "${lib}").tmp"; mv "/home/tony/.local/share/auto-scribe/bin/$(basename "${lib}").tmp" "/home/tony/.local/share/auto-scribe/bin/$(basename "${lib}")"; done

package:
    cargo build --release
    rm -rf dist/{{release_name}}
    mkdir -p dist/{{release_name}}
    cp target/release/auto-scribe dist/{{release_name}}/
    cp target/release/libonnxruntime*.so* dist/{{release_name}}/
    cp scripts/install.sh dist/{{release_name}}/
    cp README.md LICENSE dist/{{release_name}}/
    chmod +x dist/{{release_name}}/auto-scribe dist/{{release_name}}/install.sh
    cp scripts/install.sh dist/install-auto-scribe.sh
    chmod +x dist/install-auto-scribe.sh
    tar -C dist -czf dist/{{release_name}}.tar.gz {{release_name}}

seperator:
    @echo
    @echo // *============================================================================================* //
    @echo
