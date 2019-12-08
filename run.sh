#!/usr/bin/env bash

cargo build --release && \
    cp target/release/rs-cc-matching-engine cc-matching-engine.out && \
    strip cc-matching-engine.out && \
    clear && \
    ./cc-matching-engine.out
