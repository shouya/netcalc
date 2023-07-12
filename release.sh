#!/bin/bash

set -xe

# build wasm and helper js
rm -rf pkg
wasm-pack build --target no-modules

# prepare for release
rm -rf gh-pages
mkdir -p gh-pages/.github/workflows gh-pages/dist

# or the github action won't run
cp .github/workflows/gh-pages.yml gh-pages/.github/workflows/
cp pkg/* gh-pages/dist/
cp www/* gh-pages/dist/

# print the content of the dist folder
tree -ah gh-pages/

# release to github pages
# npx gh-pages --dotfiles --dist gh-pages
