# simply build all targets those every time
.PHONY: gh-pages pkg clean publish dev-server

RUST_SRC_FILES = $(wildcard src/**/*)
WWW_SRC_FILES = $(wildcard www/**/*)

# Serve locally for development
dev-server: gh-pages
	npx http-server -c-1 --cors gh-pages/dist

# Publish to github pages
publish: gh-pages
	npx gh-pages --dotfiles --dist $<

# Cause a rebuild
rebuild: clean gh-pages

# Target to build and copy the files to the output directory
gh-pages: .github/workflows/gh-pages.yml $(WWW_SRC_FILES) pkg
	mkdir -p $@
	mkdir -p $@/dist $@/.github/workflows/
	cp .github/workflows/gh-pages.yml $@/.github/workflows/
	cp pkg/* $@/dist/
	cp www/* $@/dist/
	tree -ah $@

# Target to build the wasm files
pkg: $(RUST_SRC_FILES)
	wasm-pack build --target no-modules --release -d $@

# Target to clean up the generated files
clean:
	rm -rf pkg gh-pages
