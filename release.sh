#!/bin/bash

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# build wasm
cd "$SCRIPT_DIR"
wasm-pack build

# build html and js
cd www
npm run build

# release to github pages
cd ..
git subtree push --prefix www/dist origin gh-pages

