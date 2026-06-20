set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

semantic_graph_extract := ".refactor-radar/bin/semantic-graph-extract"
rust_workspace := "rust-workspace"
fts := "fts"

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

seperator:
    @echo
    @echo // *============================================================================================* //
    @echo
