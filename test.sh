#!/bin/bash

cargo test --test cases --features preserve_order $1
cargo test --test sorted $1
