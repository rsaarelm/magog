#!/bin/sh

# XXX: We want some extra-slow binary size optimizing flags for the release
# build. Rust's debug builds can be very slow so we also want faster-building
# release builds during development, and cargo does not support more profiles,
# so this will generate a temporary cargo file that has the additional flags
# set and build everything using it.

WORK_DIR=`mktemp -d`
git clone . $WORK_DIR
pushd $WORK_DIR

cat >> Cargo.toml << EOF
[profile.release]
lto = true
codegen-units = 1
panic = 'abort'
EOF

cargo +stable build --release
strip -s target/release/magog
echo "Built release binary `readlink -f target/release/magog`"
