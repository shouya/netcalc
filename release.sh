#!/bin/bash

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# build wasm
(
  cd "$SCRIPT_DIR";
  wasm-pack build
)

# build html and js
mkdir -p dist
cp pkg/* dist/
cp www/* dist/

# release to github pages
git subtree push --prefix dist origin gh-pages
