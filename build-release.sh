#!/bin/sh

WORK_DIR=`mktemp -d`
git clone . $WORK_DIR
pushd $WORK_DIR

cat >> cargo.toml << EOF
[profile.release]
lto = 1
codegen-units = 1
panic = 'abort'
EOF

cargo +nightly build --release
strip -s target/release/magog
readlink -f target/release/magog
