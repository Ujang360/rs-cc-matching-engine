#!/usr/bin/env bash

clear && \
    RUST_BACKTRACE=1 cargo test && \
    cargo build --release && \
    cp target/release/rs-cc-matching-engine cc-matching-engine.out && \
    strip cc-matching-engine.out && \
    clear && \
    ./cc-matching-engine.out
