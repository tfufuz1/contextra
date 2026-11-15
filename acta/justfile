default:
    @just --list

gates:
    cargo xtask gates

check:
    cargo check --workspace --all-targets

test:
    cargo test --workspace

lint:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all
