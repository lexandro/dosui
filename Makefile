# dosui — build, test, and install.
#
#   make            build the release binary
#   make run        run from source
#   make check      fmt + clippy + test (what CI runs)
#   make check-docker  the same gate + MSRV, in a container (needs no local GTK)
#   make install    install into $(PREFIX) (honours $(DESTDIR))
#   make uninstall  remove an installed copy
#   make appimage   build the bundled AppImage (packaging/build-appimage.sh)
#   make clean      cargo clean + remove dist/

CARGO   ?= cargo
INSTALL ?= install
PREFIX  ?= /usr/local
DESTDIR ?=

BIN      := target/release/dosui
APP_ID   := io.github.dosui
ICON_SIZES := 16 32 48 64 128 256 512
bindir   := $(DESTDIR)$(PREFIX)/bin
appsdir  := $(DESTDIR)$(PREFIX)/share/applications
iconbase := $(DESTDIR)$(PREFIX)/share/icons/hicolor
metadir  := $(DESTDIR)$(PREFIX)/share/metainfo

MSRV        := 1.88
DOCKER      ?= docker
DOCKER_IMAGE := dosui-ci

.PHONY: all build run test check check-docker fmt clippy appimage install uninstall clean

all: build

build:
	$(CARGO) build --release

run:
	$(CARGO) run

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

# Mirror the CI gate locally.
check:
	$(CARGO) fmt --all --check
	$(CARGO) clippy --all-targets -- -D warnings
	$(CARGO) test

# The same gate plus an MSRV check, inside a container that carries the GTK 4
# development libraries. For machines that cannot build dosui natively; a Linux
# box with libgtk-4-dev wants plain `make check`.
#
# `target/` lives in a named volume so a container build and a host build do not
# overwrite each other's artifacts.
check-docker:
	$(DOCKER) build -t $(DOCKER_IMAGE) -f packaging/Dockerfile.test packaging
	$(DOCKER) run --rm \
		-v "$(CURDIR)":/work \
		-v dosui-target:/target \
		-e RUSTFLAGS=-D\ warnings \
		$(DOCKER_IMAGE) sh -euc '\
			cargo fmt --all --check; \
			cargo clippy --all-targets --all-features; \
			cargo test --all-features; \
			cargo +$(MSRV) check --locked --all-targets --all-features'

appimage:
	./packaging/build-appimage.sh

$(BIN): build

install: $(BIN)
	$(INSTALL) -Dm755 $(BIN) $(bindir)/dosui
	$(INSTALL) -Dm644 data/$(APP_ID).desktop $(appsdir)/$(APP_ID).desktop
	$(INSTALL) -Dm644 data/$(APP_ID).metainfo.xml $(metadir)/$(APP_ID).metainfo.xml
	$(foreach s,$(ICON_SIZES),$(INSTALL) -Dm644 data/icons/hicolor/$(s)x$(s)/apps/$(APP_ID).png $(iconbase)/$(s)x$(s)/apps/$(APP_ID).png;)
ifeq ($(DESTDIR),)
	-update-desktop-database $(PREFIX)/share/applications 2>/dev/null || true
	-gtk-update-icon-cache -qtf $(PREFIX)/share/icons/hicolor 2>/dev/null || true
endif
	@echo "Installed dosui to $(PREFIX)."

uninstall:
	rm -f $(bindir)/dosui
	rm -f $(appsdir)/$(APP_ID).desktop
	rm -f $(metadir)/$(APP_ID).metainfo.xml
	$(foreach s,$(ICON_SIZES),rm -f $(iconbase)/$(s)x$(s)/apps/$(APP_ID).png;)
	@echo "Removed dosui from $(PREFIX)."

clean:
	$(CARGO) clean
	rm -rf dist
