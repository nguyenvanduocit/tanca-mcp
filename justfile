build:
    cargo build --release

build-cli:
    cargo build --release --bin tanca-cli

install-cli:
    cargo install --path . --bin tanca-cli

test:
    cargo test

check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test
