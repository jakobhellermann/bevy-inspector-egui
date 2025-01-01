#!/bin/sh

set -eu

cargo publish -p bevy-inspector-egui-derive "$@"
cargo publish -p bevy-inspector-egui --features winit/x11 "$@"
