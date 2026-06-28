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

🚧 **Korai tervezési fázis.** A megvalósítás nyelve és a GUI toolkit még
nincs eldöntve — lásd a `docs/` mappát a tervezési jegyzetekhez.

## Nem cél (egyelőre)

- DOSBox maga (a frontend a meglévő `dosbox` / `dosbox-x` bináris elé kerül).
- Windows/macOS támogatás az első körben (Linux-first).
