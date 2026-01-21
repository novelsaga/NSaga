#!/usr/bin/env bash
# Convenience wrapper for cargo xtask
exec cargo run --package xtask --bin xtask -- "$@"
