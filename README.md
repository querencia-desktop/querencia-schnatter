# Querencia Webhülle

Eine Hülle für WhatsApp Web: die Seite im eigenen Fenster, für alle, die am
Rechner schreiben wollen, ohne einen Browser-Reiter offen zu halten. Ein
Zusatzprogramm zu [Querencia Desktop](https://querencialinux.org) – einer
klassischen Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust,
GTK 4) –, **nicht Teil der Standardinstallation**, sondern ein eigenes,
wahlweise installierbares Paket (`querencia-webhuelle`). Bis Version 0.1.1
hieß das Programm „Querencia Nachrichten“.

*English summary:* „Querencia Web Wrapper“ is a wrapper for WhatsApp Web:
it shows the page in a window of its own (GTK 4, WebKitGTK 6.0), with the
classic Querencia menu bar. Not a product of WhatsApp or Meta and not
affiliated with them. No calls. Optional package, not part of the default
installation. Alpha software; documentation is in German.

> **Alpha.** Keine Anrufe (Ubuntus WebKitGTK ist ohne WebRTC gebaut). Meldungen
> kommen nur, solange das Fenster offen ist; Schließen beendet das Programm
> (Symbol im Panel und Hintergrundbetrieb folgen später). **Kein Produkt von
> WhatsApp oder Meta und nicht mit ihnen verbunden**; WhatsApp ist eine Marke
> ihres Inhabers und steht nur beschreibend da, nie im Programmnamen oder im
> Symbol.

## Zweck

- `https://web.whatsapp.com` in einer WebKitGTK-6.0-Ansicht, darüber nur die
  klassische Menüleiste (Datei, Bearbeiten, Ansicht, Hilfe)
- dauerhafte Anmeldung (Sitzung unter `~/.local/share/querencia-webhuelle`
  und `~/.cache/querencia-webhuelle`, Einstellungen unter
  `~/.config/querencia-webhuelle`, Ordner 0700); „Datei → Abmelden und
  Daten löschen“ leert Sitzung und Cache nach Rückfrage
- Umzug von Querencia Nachrichten: Beim ersten Start benennt 0.2.0 die drei
  Ordner `querencia-nachrichten` in `querencia-webhuelle` um (nichts wird
  kopiert oder gelöscht, die Anmeldung bleibt); läuft Nachrichten noch, sagt
  das Fenster es und startet nicht; gibt es den neuen Ordner schon, bleibt
  der alte liegen (`src/huelle/umzug.rs`)
- Meldungen neuer Nachrichten über Querencia Benachrichtigungen (WebKits
  Standardweg); ein Klick holt das Fenster nach vorn
- ungelesene Nachrichten im Fenstertitel („(3) Querencia Webhülle“)
- Downloads ohne Rückfrage in den Download-Ordner, nie über eine vorhandene
  Datei, danach eine Meldung mit „Öffnen“ und „Ordner öffnen“
- Mikrofon (Sprachnachrichten), Kamera und Lesen der Zwischenablage: je Art
  einmal fragen, Antwort merken (Bearbeiten → Einstellungen vergisst sie)
- fremde Links öffnet der Standardbrowser (http, https) bzw. das Mailprogramm
  (mailto); das Fenster bleibt bei WhatsApp Web
- Rechtschreibprüfung in der Sprache der Oberfläche und Englisch (nur
  installierte hunspell-Wörterbücher)
- Zoom und Fenstergröße werden gemerkt, Vollbild mit F11
- stürzt der Web-Prozess ab, lädt das Fenster neu und sagt es
- Browserkennung umschaltbar (Bearbeiten → Einstellungen): WebKit-Standard
  (WhatsApp sieht Safari auf einem Mac) oder Linux-Kennung
- Software-Grafik (Bearbeiten → Einstellungen): automatisch, an, aus; bei
  OpenGL über zink auf nouveau zeichnet WebKit die Seite automatisch in
  Software (der Bildbetrachter stürzte sonst ab, QD-169); der Info-Dialog
  nennt den Renderer
- Links mit anderen Schemata als http, https, mailto erst nach Rückfrage; die
  Hauptseite wechselt nie auf `data:`, `file:` oder fremde `blob:`
- eigenes Programmsymbol (Fenster mit Adresszeile und Globus,
  `data/icons/org.querencia.Webhuelle.svg`), kein Logo eines Dienstes
- Bewusst **nicht** enthalten: Anrufe, Panel-Symbol, Verstecken statt
  Beenden, Meldungen mit Absenderbild, mehrere Konten, `whatsapp://`-Links

Name: deutsch „Querencia Webhülle“, englisch „Querencia Web Wrapper“, im Menü
„Webhülle“ / „Web Wrapper“ (Internet) · Beschreibung: „Hülle für WhatsApp
Web“ / „Wrapper for WhatsApp Web“ · Anwendungskennung (App-ID, D-Bus-Name):
`org.querencia.Webhuelle` · Binary: `querencia-webhuelle` · Debian-Paket:
`querencia-webhuelle` (optional, nicht im Metapaket `querencia-desktop`) ·
bis 0.1.1: `org.querencia.Nachrichten`, `querencia-nachrichten`

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
| Version | 0.2.0 (Tag `webhuelle/0.2.0`, Zweig `webhuelle`): Umbenennung in Querencia Webhülle mit Umzug der Ordner und eigenem Symbol, zur Freigabe bereit (Integrationsfassung `integration/webhuelle-020/`, Tag `webhuelle/0.2.0-int`); live ist Querencia Nachrichten 0.1.1 seit 23.10.2026 22:39 (`nachrichten/0.1.1-int`) |
| Letzte Prüfung | 24.10.2026 (0.2.0): 39 Unit-Tests, Clippy sauber, `i18n.py pruefen` (104 Texte); Prüfhülle Stufe L: Prüfseite de/1920×1080/hell und en/1024×768/dunkel je 74 von 74, Grafik-Szenario je 44 von 44, Umzug von 0.1.1 je 58 von 58; Anmeldeseiten-Wächter Stufe E; Integrationsprobe siehe [docs/integration.md](docs/integration.md) |
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
von ihnen unterstützt. Kein WhatsApp-Logo im Programm; das Programmsymbol ist
ein eigenes (Fenster mit Adresszeile und Globus in Querencia-Orange).

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
