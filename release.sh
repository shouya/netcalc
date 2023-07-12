#!/bin/bash

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# build wasm
(
  cd "$SCRIPT_DIR";
  wasm-pack build
)

# build html and js
git subtree add --prefix=dist origin gh-pages || true
rm -rf dist/*
cp pkg/* dist/
cp www/* dist/

# print the content of the dist folder
ls dist

# release to github pages
(
  cd dist;
  git add -A;
  git commit -m "release";
  git push
)
