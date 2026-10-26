# Bauen

## Voraussetzungen

Ubuntu 26.04 mit `build-essential pkg-config rustc cargo libgtk-4-dev
libwebkitgtk-6.0-dev gettext` (Rust 1.93 aus dem Archiv reicht;
`libwebkitgtk-6.0-dev` zieht `libjavascriptcoregtk-6.0-dev` und
`libsoup-3.0-dev` mit, `libgtk-4-dev` über `libepoxy-dev` auch `libegl-dev`:
seit 0.1.1 linkt das Programm `libEGL.so.1` direkt für die Grafik-Probe).
Zur Laufzeit: `libwebkitgtk-6.0-4` (2.52.x),
empfohlen `hunspell-de-de`, `hunspell-en-us`, `gstreamer1.0-plugins-good`.

Crates: gtk4 0.11 (Feature `v4_12`), webkit6 0.6.1 (Feature `v2_52`, E1),
gettext-rs 0.8 (`gettext-system`), dazu die Runtime-Crates
`querencia-oberflaeche` (klassische Menüs, Symbolthema, Farbschema,
Settings-Portal) und `querencia-dateiarbeit` (freie Dateinamen) als
**Pfad-Abhängigkeiten** auf `../querencia-runtime/crates/…` (E11): Der
Checkout der Runtime – öffentlich das Repo `querencia-shell`, ab Tag
`dateien/0.2.2` (Querencia Oberfläche 0.1.1; die CI nimmt einen festen Commit
darüber) – liegt als `querencia-runtime`
neben diesem Repo (README, „Runtime daneben“). Alle
Versionen in `Cargo.lock`; die Bauskripte arbeiten offline
(`--offline --locked`).

## Release mit Prüfungen

```bash
bash werkzeug/bauen.sh            # fmt, Clippy, Tests, dann Release
bash werkzeug/bauen.sh pruefen    # nur fmt, Clippy, Tests
bash werkzeug/bauen.sh release    # nur Release
```

- Alles läuft auf einem der gemeinsamen Bau-Plätze
  (`~/.local/state/querencia/bau-platz.sh`, `nice -n 10`, 5 Jobs) mit
  eigenem `TMPDIR` (`tmp/`). GUI- und Messläufe laufen nie unter einem
  Bau-Platz.
- Der Release-Build läuft mit `--remap-path-prefix` für das Repo
  (`/usr/src/querencia-schnatter`), den Runtime-Checkout
  (`/usr/src/querencia-runtime`) und das Cargo-Home (`/usr/share/cargo`)
  nach `target/remap/release/`. Das Skript zählt am Ende die Home-Pfade im
  Binary (Soll 0).
- Protokolle: `nachweis/bau/<art>.log`, Prüfsumme
  `nachweis/bau/release-sha256.txt`.

## Übersetzungen

Englisch steht im Code (msgid), Deutsch in `po/de.po`. Werkzeug ist
`werkzeug/i18n.py` aus Querencia Dokumente (nur Python-Standardbibliothek):

```bash
python3 werkzeug/i18n.py pot      # po/querencia-schnatter.pot aus src/ und data/*.desktop.in
python3 werkzeug/i18n.py update   # po/de.po nachziehen, dann übersetzen
python3 werkzeug/i18n.py desktop data/org.querencia.Schnatter.desktop.in data/org.querencia.Schnatter.desktop
python3 werkzeug/i18n.py pruefen  # muss PASS melden
python3 werkzeug/i18n.py mo <ziel>
msgfmt -c -o /dev/null po/de.po   # Gegenprobe mit GNU gettext
```

`i18n.py mo` legt neben `querencia-schnatter.mo` auch
`querencia-oberflaeche.mo` ab (aus dem po-Ordner der Bibliothek im
Runtime-Checkout; Tastennamen „Strg“ in den Menüs). Die Binary sucht die
Kataloge in dieser Reihenfolge: `QUERENCIA_SCHNATTER_LOCALEDIR`,
`<Ordner der Binary>/share/locale`, `<Präfix>/share/locale`,
`/usr/share/locale`. Ohne Katalog ist die Oberfläche englisch.

## Symbol

Eigenes Programmsymbol `data/icons/org.querencia.Schnatter.svg` (seit 0.3.0;
0.2.0 ein Fenster mit Globus, bis 0.1.1 `empathy` aus dem Symbolthema): eine
grüne Sprechblase mit dunkelgrünem Rand und Zipfel, ein Auge und ein
Schnabel in Querencia-Orange, in der Blase drei helle Punkte. Kein Logo
eines Dienstes (kein Kreis, kein Telefonhörer), nicht das Querencia-Logo; Lizenz wie das Programm (GPL-3.0-or-later). Es liegt nur als
SVG in `hicolor/scalable/apps/` (nie zusätzlich `hicolor/48x48`,
GTK-4.22-Absturz). Im Thema Querencia gibt es das Symbol noch nicht; GTK
findet es über hicolor.

## Paket

Quellpaket `querencia-schnatter` im Packaging-Repo (`querencia-packaging`,
Ordner `querencia-schnatter/`), **nicht** im Metapaket `querencia-desktop`:
wer es will, installiert es selbst. Es ersetzt `querencia-schnatter` (0.2.0)
und `querencia-nachrichten` (Replaces, Breaks, Provides); beim Upgrade zieht
das Programm die Ordner beim ersten Start um. Inhalt:

- `/usr/bin/querencia-schnatter`
- `/usr/share/locale/de/LC_MESSAGES/querencia-schnatter.mo` (die Kataloge
  der Runtime-Bibliotheken kommen aus `querencia-shell`)
- `/usr/share/applications/org.querencia.Schnatter.desktop`
- `/usr/share/metainfo/org.querencia.Schnatter.metainfo.xml`
- `/usr/share/icons/hicolor/scalable/apps/org.querencia.Schnatter.svg`
- Depends `libwebkitgtk-6.0-4` (aus den gelinkten Bibliotheken); dieses
  Paket hängt selbst schon von `bubblewrap` und `xdg-dbus-proxy` (WebKits
  Sandbox der Web-Prozesse) und `gstreamer1.0-plugins-good` ab. Recommends
  `querencia-shell` (Kataloge der Runtime-Bibliotheken, z. B. „Strg“ in
  den Menüs), `hunspell-de-de`, `hunspell-en-us`; `debian/copyright` mit
  WebKitGTK (LGPL-2.1/BSD) und webkit6-rs (MIT).
