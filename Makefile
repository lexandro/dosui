# dosui — build, test, and install.
#
#   make            build the release binary
#   make run        run from source
#   make check      fmt + clippy + test (what CI runs)
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
bindir   := $(DESTDIR)$(PREFIX)/bin
appsdir  := $(DESTDIR)$(PREFIX)/share/applications
icondir  := $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps
metadir  := $(DESTDIR)$(PREFIX)/share/metainfo

.PHONY: all build run test check fmt clippy appimage install uninstall clean

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

appimage:
	./packaging/build-appimage.sh

$(BIN): build

install: $(BIN)
	$(INSTALL) -Dm755 $(BIN) $(bindir)/dosui
	$(INSTALL) -Dm644 data/$(APP_ID).desktop $(appsdir)/$(APP_ID).desktop
	$(INSTALL) -Dm644 data/$(APP_ID).svg $(icondir)/$(APP_ID).svg
	$(INSTALL) -Dm644 data/$(APP_ID).metainfo.xml $(metadir)/$(APP_ID).metainfo.xml
ifeq ($(DESTDIR),)
	-update-desktop-database $(PREFIX)/share/applications 2>/dev/null || true
	-gtk-update-icon-cache -qtf $(PREFIX)/share/icons/hicolor 2>/dev/null || true
endif
	@echo "Installed dosui to $(PREFIX)."

uninstall:
	rm -f $(bindir)/dosui
	rm -f $(appsdir)/$(APP_ID).desktop
	rm -f $(icondir)/$(APP_ID).svg
	rm -f $(metadir)/$(APP_ID).metainfo.xml
	@echo "Removed dosui from $(PREFIX)."

clean:
	$(CARGO) clean
	rm -rf dist
