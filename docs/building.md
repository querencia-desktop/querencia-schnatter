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
**Pfad-Abhängigkeiten** auf `../querencia-runtime/crates/…` (E11, Stand
`bd420d7`): Der Checkout von `querencia-runtime` liegt neben diesem Repo.
Vor der Veröffentlichung werden daraus Git-Abhängigkeiten mit Tag. Alle
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
  (`/usr/src/querencia-nachrichten`), den Runtime-Checkout
  (`/usr/src/querencia-runtime`) und das Cargo-Home (`/usr/share/cargo`)
  nach `target/remap/release/`. Das Skript zählt am Ende die Home-Pfade im
  Binary (Soll 0).
- Protokolle: `nachweis/bau/<art>.log`, Prüfsumme
  `nachweis/bau/release-sha256.txt`.

## Übersetzungen

Englisch steht im Code (msgid), Deutsch in `po/de.po`. Werkzeug ist
`werkzeug/i18n.py` aus Querencia Dokumente (nur Python-Standardbibliothek):

```bash
python3 werkzeug/i18n.py pot      # po/querencia-nachrichten.pot aus src/ und data/*.desktop.in
python3 werkzeug/i18n.py update   # po/de.po nachziehen, dann übersetzen
python3 werkzeug/i18n.py desktop data/org.querencia.Nachrichten.desktop.in data/org.querencia.Nachrichten.desktop
python3 werkzeug/i18n.py pruefen  # muss PASS melden
python3 werkzeug/i18n.py mo <ziel>
msgfmt -c -o /dev/null po/de.po   # Gegenprobe mit GNU gettext
```

`i18n.py mo` legt neben `querencia-nachrichten.mo` auch
`querencia-oberflaeche.mo` ab (aus dem po-Ordner der Bibliothek im
Runtime-Checkout; Tastennamen „Strg“ in den Menüs). Die Binary sucht die
Kataloge in dieser Reihenfolge: `QUERENCIA_NACHRICHTEN_LOCALEDIR`,
`<Ordner der Binary>/share/locale`, `<Präfix>/share/locale`,
`/usr/share/locale`. Ohne Katalog ist die Oberfläche englisch.

## Symbol

Noch kein eigenes Programmsymbol (QD-166, Symbole-Worker). Starter und
Fenster nehmen `empathy` aus dem Symbolthema (Chat-Symbol aus GNOME 2,
`gnome-colors-common`, das Querencia über gnome-human erbt); `huelle::SYMBOL`
und `Icon=` in der Desktop-Vorlage tauschen, sobald es da ist (nie
`hicolor/48x48`, GTK-4.22-Absturz).

## Paket (später)

- `/usr/bin/querencia-nachrichten`
- `/usr/share/locale/de/LC_MESSAGES/querencia-nachrichten.mo` (und die
  Kataloge der Runtime-Bibliotheken aus deren Paket)
- `/usr/share/applications/org.querencia.Nachrichten.desktop`
- `/usr/share/metainfo/org.querencia.Nachrichten.metainfo.xml`
- Depends `libwebkitgtk-6.0-4`; Recommends `hunspell-de-de`,
  `hunspell-en-us`, `gstreamer1.0-plugins-good`; `debian/copyright` mit
  WebKitGTK (LGPL-2.1/BSD) und webkit6-rs (MIT).
