#!/usr/bin/env bash
RUSTDOCFLAGS="--html-in-header rustdoc/katex-header.html --document-private-items" cargo doc --no-deps
