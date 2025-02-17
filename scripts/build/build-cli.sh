#!/bin/bash
# Copyright 2020-2021 The Databend Authors.
# SPDX-License-Identifier: Apache-2.0.

SCRIPT_PATH="$(cd "$(dirname "$0")" >/dev/null 2>&1 && pwd)"
cd "$SCRIPT_PATH/../.." || exit
build-cli () {
  echo "Build(RELEASE) start..."
  cargo build --bin=bendctl --release
  echo "All done..."
}

install-cli () {
  echo "Install(RELEASE) start..."
  cargo install --bin=bendctl --path ./cli
  echo "All done..."
  echo "Take a look at cli/README.md for further instructions"
}


"$@"