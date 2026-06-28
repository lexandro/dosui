# dosui

Egy **könnyűsúlyú, natív Linux frontend a DOSBox** elé — game/profil launcher és
konfigurációkezelő.

## Miért?

A létező megoldások (pl. **DBGL – DOSBox Game Launcher**, **D-Fend Reloaded**)
nehézsúlyúak: Java runtime-ot igényelnek vagy Windows-központúak. Célunk egy
**lightweight**, gyorsan induló, kevés függőséget igénylő alternatíva, ami
natívan illeszkedik a Linux desktophoz.

## Célok

- **Játék-/profilkezelés**: DOSBox profilok (játékonként külön konfiguráció)
  létrehozása, szerkesztése, indítása egy listából.
- **DOSBox konfiguráció GUI-ból**: a `dosbox.conf` legfontosabb beállításai
  (CPU cycles, memória, render, sound, mount-ok stb.) grafikus felületről,
  kézi fájlszerkesztés nélkül.
- **Könnyűsúly**: gyors indulás, kis memórialábnyom, minimális futásidejű
  függőség. **Nincs Java.**
- **Natív Linux érzet**: a desktop környezethez illeszkedő megjelenés és
  viselkedés.
- **Egyszerű telepíthetőség**: lehetőleg egyetlen bináris / csomag.

## Tervezett funkciók (vázlat)

- Profilok listája borítóképpel / metaadattal.
- Új profil varázsló (mount könyvtár/IMG, futtatandó .exe/.bat kiválasztása).
- Per-profil DOSBox beállítások szerkesztője.
- Globális alapértelmezések, amiket a profilok örökölnek.
- DOSBox indítása a kiválasztott profillal.
- (Később) profil import/export, csomagolt játékok kezelése.

## Állapot

✅ **Működő, D-Fend-szerű frontend.** Stack: **Rust + GTK4** (libadwaita nélkül),
motor: **dosbox-staging**.

Funkciók:
- Borító-rács + részletek panel, indítás (Play / dupla-katt / Enter), „last played".
- Tabos profilszerkesztő (General / Mounts & Run / CPU / Graphics / Sound / MIDI /
  Advanced), élő `dosbox.conf` előnézet.
- Új-profil varázsló (mappa → automatikus exe/bat szkennelés → metaadat).
- Globális defaultok + per-profil öröklés; beállítások dialógus.
- **Menüsor + toolbar + jobbklikk context-menü**; gyorsbillentyűk.
- Kategória-oldalsáv (műfaj/fejlesztő/év/kedvencek) + find-as-you-type kereső.
- Kedvencek, duplikálás, törlés, mappanyitás, **tömeges metaadat-szerkesztés**.
- Import: **dosbox.conf** (D-Fend/DBGL) és **zip** (drag & drop is).
- Egyetlen **AppImage** becsomagolt dosbox-staginggel, a gazdagép témáját követve.

## Fejlesztés

```
cargo run                  # futtatás (RUST_LOG=debug a bőbeszédű loghoz)
cargo test                 # a GTK-mentes core unit-tesztjei
cargo clippy && cargo fmt  # lint + formázás commit előtt
```

Előfeltételek (Debian/Ubuntu/Mint): `sudo apt install build-essential libgtk-4-dev`
(+ `librsvg2-common` az SVG ikonokhoz).

## Csomagolás (AppImage)

```
./packaging/build-appimage.sh      # -> dist/dosui-x86_64.AppImage
```

Egyetlen önálló fájl: tartalmazza a GTK4 runtime-ot és a **dosbox-staging**-et,
mégis a gazdagép GTK-témáját követi. A dosbox-staging hordozható buildjét a
`~/.local/opt/dosbox-staging*` mappából veszi (vagy `DOSBOX_STAGING_DIR`).

## Nem cél (egyelőre)

- DOSBox maga (a frontend a meglévő `dosbox` / `dosbox-x` bináris elé kerül).
- Windows/macOS támogatás az első körben (Linux-first).
