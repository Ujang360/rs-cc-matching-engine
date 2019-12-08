#!/usr/bin/env bash

clear && \
    RUST_BACKTRACE=full cargo test && \
    cargo build --release && \
    cp target/release/cc-matching-engine cc-matching-engine.out && \
    strip cc-matching-engine.out && \
    clear && \
    ./cc-matching-engine.out
