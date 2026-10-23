# Querencia Nachrichten

WhatsApp Web im eigenen Fenster, für alle, die am Rechner schreiben wollen,
ohne einen Browser-Reiter offen zu halten. Teil von
[Querencia Desktop](https://querencialinux.org) – einer klassischen
Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust, GTK 4).

*English summary:* „Querencia Messages“ shows WhatsApp Web in a window of
its own (GTK 4, WebKitGTK 6.0), with the classic Querencia menu bar.
Not affiliated with WhatsApp or Meta. No calls. Alpha software;
documentation is in German.

> **Alpha.** Keine Anrufe (Ubuntus WebKitGTK ist ohne WebRTC gebaut). Meldungen
> kommen nur, solange das Fenster offen ist; Schließen beendet das Programm
> (Symbol im Panel und Hintergrundbetrieb folgen in 0.2). **Nicht mit
> WhatsApp oder Meta verbunden**; WhatsApp ist eine Marke ihres Inhabers.

## Zweck

- `https://web.whatsapp.com` in einer WebKitGTK-6.0-Ansicht, darüber nur die
  klassische Menüleiste (Datei, Bearbeiten, Ansicht, Hilfe)
- dauerhafte Anmeldung (Sitzung unter `~/.local/share/querencia-nachrichten`
  und `~/.cache/querencia-nachrichten`, Ordner 0700); „Datei → Abmelden und
  Daten löschen“ leert beides nach Rückfrage
- Meldungen neuer Nachrichten über Querencia Benachrichtigungen (WebKits
  Standardweg); ein Klick holt das Fenster nach vorn
- ungelesene Nachrichten im Fenstertitel („(3) Querencia Nachrichten“)
- Downloads ohne Rückfrage in den Download-Ordner, nie über eine vorhandene
  Datei, danach eine Meldung mit „Öffnen“ und „Ordner öffnen“
- Mikrofon (Sprachnachrichten), Kamera und Lesen der Zwischenablage: je Art
  einmal fragen, Antwort merken (Bearbeiten → Einstellungen vergisst sie)
- fremde Links öffnet der Standardbrowser; das Fenster bleibt bei WhatsApp Web
- Rechtschreibprüfung in der Sprache der Oberfläche und Englisch (nur
  installierte hunspell-Wörterbücher)
- Zoom und Fenstergröße werden gemerkt, Vollbild mit F11
- stürzt der Web-Prozess ab, lädt das Fenster neu und sagt es
- Browserkennung umschaltbar (Bearbeiten → Einstellungen): WebKit-Standard
  (WhatsApp sieht Safari auf einem Mac) oder Linux-Kennung
- Bewusst **nicht** Teil von 0.1: Anrufe, Panel-Symbol, Verstecken statt
  Beenden, Meldungen mit Absenderbild, mehrere Konten, `whatsapp://`-Links

Anwendungskennung (App-ID): `org.querencia.Nachrichten` · Binary:
`querencia-nachrichten` · Debian-Paket (später): `querencia-nachrichten`

| Taste | Wirkung |
|---|---|
| Strg+R, F5 | Neu laden |
| Strg+Q | Beenden |
| Strg+X, Strg+C, Strg+V, Strg+A | Ausschneiden, Kopieren, Einfügen, Alles auswählen (die Seite selbst) |
| Strg+Plus/Strg+=, Strg+Minus, Strg+0 | Zoom, normale Größe |
| F11 | Vollbild ein/aus |
| F1 | Hilfe |

## Stand

| | |
|---|---|
| Version | 0.1.0 „Hülle“ (Tag `nachrichten/0.1.0`), nicht live |
| Letzte Prüfung | 23.10.2026: 26 Unit-Tests, Clippy sauber, `i18n.py pruefen`; Prüfhülle Stufe L gegen die Prüfseite de/1920×1080/hell und en/1024×768/dunkel je 51 von 51; Anmeldeseiten-Wächter Stufe E, beide Kennungen, hell/dunkel, 1920×1080 und 1024×768: QR-Code nach 4,6–4,7 s; Integrationsprobe 99/0 |
| Bekannte Lücken | siehe [docs/testing.md](docs/testing.md), Abschnitt „Nicht getestet“ |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, WebKitGTK 2.52.6, Rust 1.93 |
| Nicht getestet | alles nach der Anmeldung (nur mit Konto, Abnahme durch den Besitzer), HiDPI, Mehrmonitor |

Änderungen: [CHANGELOG.md](CHANGELOG.md).

## Bauen und Prüfen

`cargo build --release --locked`, danach `cargo fmt --all --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`
und `python3 werkzeug/i18n.py pruefen` (ohne Bildschirm). Voraussetzung:
`libwebkitgtk-6.0-dev`; der Checkout von `querencia-runtime` liegt neben
diesem Repo (siehe unten). Einzelheiten: [docs/building.md](docs/building.md);
Prüfhülle, Prüfseite und Anmeldeseiten-Wächter (`bash werkzeug/lauf.sh …`,
nie in der laufenden Sitzung): [docs/testing.md](docs/testing.md);
Integration: [docs/integration.md](docs/integration.md).

**Veröffentlichungspunkt:** `querencia-oberflaeche` und
`querencia-dateiarbeit` sind in 0.1 Pfad-Abhängigkeiten auf
`../querencia-runtime/crates/…` (Stand `bd420d7`, Entscheidung E11). Vor der
Veröffentlichung werden sie Git-Abhängigkeiten mit Tag; bis dahin baut die CI
dieses Repos nicht für sich allein.

## Vorbilder

Die Bedienideen stammen aus zwei freien Hüllen um WhatsApp Web, deren
Quelltext gelesen, aber **nicht übernommen** wurde (andere Sprache und
Werkzeuge; Entscheidung E8):

- **ZapZap** (Python/Qt, GPL-3.0-or-later): Ungelesen-Zahl aus dem
  Seitentitel, Rechte je Art merken, Neuladen nach einem Absturz.
- **WasIstLos** (C++/gtkmm 3, WebKitGTK, GPL-3): Benachrichtigungsrecht vorab
  für `https://web.whatsapp.com`, Rechtschreibung in der Systemsprache.

Das Projekt ist mit ZapZap, WasIstLos, WhatsApp und Meta weder verbunden noch
von ihnen unterstützt. Kein WhatsApp-Logo im Programm; das Programmsymbol
kommt vorerst aus dem Symbolthema.

## Lizenz

- Programm: GPL-3.0-or-later, siehe [LICENSE](LICENSE). Copyright 2026
  Querencia Desktop contributors; Urheberhinweis in [COPYRIGHT](COPYRIGHT).
- Bibliotheken: WebKitGTK 6.0 (LGPL-2.1 und BSD, Ubuntu-Paket
  `libwebkitgtk-6.0-4`), GTK 4 und GLib (LGPL); die Rust-Bindings gtk-rs und
  webkit6-rs stehen unter MIT.
- `werkzeug/virtual-keyboard-unstable-v1.xml` (nur Testwerkzeug) behält den
  MIT-Vermerk im Dateikopf.
- Übersetzungen (`po/`) stehen unter derselben Lizenz wie das Programm.

Name und Logo „Querencia“ sind keine freien Inhalte ([MARKE.md](MARKE.md)).

## Fehler melden

Fehler, Wünsche und Fragen zu Querencia gehen an die gemeinsame Meldestelle:
<https://github.com/querencia-desktop/querencia/issues/new/choose>.
Sicherheitslücken bitte nicht öffentlich melden, siehe [SECURITY.md](SECURITY.md).
