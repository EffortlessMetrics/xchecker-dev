#!/bin/bash
sed -i 's/content\.clone()/content/g' crates/xchecker-packet/src/builder.rs
cargo bench -p xchecker-packet
