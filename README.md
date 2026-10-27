# Schnatter

Eine Hülle für WhatsApp Web: die Seite im eigenen Fenster, für alle, die am
Rechner schreiben wollen, ohne einen Browser-Reiter offen zu halten. Ein
Zusatzprogramm zu [Querencia Desktop](https://querencialinux.org) – einer
klassischen Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust,
GTK 4) –, **nicht Teil der Standardinstallation**, sondern ein eigenes,
wahlweise installierbares Paket (`querencia-schnatter`). Installiert wird es
aus der APT-Quelle von Querencia Desktop mit `sudo apt install
querencia-schnatter`; Anleitung auf <https://querencialinux.org>.

**Name:** Bis 0.1.1 hieß das Programm „Querencia Nachrichten“, in 0.2.0
„Querencia Webhülle“, seit 0.3.0 (Alpha 3.2, 26.10.2026) „Schnatter“. Das
Paket `querencia-webhuelle` ist seitdem nur noch ein Übergangspaket auf
`querencia-schnatter`.

*English summary:* „Schnatter“ is a wrapper for WhatsApp Web:
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
- dauerhafte Anmeldung (Sitzung unter `~/.local/share/querencia-schnatter`
  und `~/.cache/querencia-schnatter`, Einstellungen unter
  `~/.config/querencia-schnatter`, Ordner 0700); „Datei → Abmelden und
  Daten löschen“ leert Sitzung und Cache nach Rückfrage
- Umzug von Querencia Webhülle und Querencia Nachrichten: Beim ersten Start
  benennt 0.3.0 die drei Ordner `querencia-webhuelle` (0.2.0) – oder, wer
  0.2.0 übersprungen hat, `querencia-nachrichten` (bis 0.1.1) – in
  `querencia-schnatter` um (nichts wird kopiert oder gelöscht, die Anmeldung
  bleibt); läuft eines der alten Programme noch, sagt das Fenster es und
  startet nicht; gibt es den neuen Ordner schon, bleiben die alten liegen
  (`src/huelle/umzug.rs`)
- Meldungen neuer Nachrichten über Querencia Benachrichtigungen (WebKits
  Standardweg); ein Klick holt das Fenster nach vorn
- ungelesene Nachrichten im Fenstertitel („(3) Schnatter“)
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
- eigenes Programmsymbol (grüne Sprechblase mit Schnabel,
  `data/icons/org.querencia.Schnatter.svg`), kein Logo eines Dienstes
- Bewusst **nicht** enthalten: Anrufe, Panel-Symbol, Verstecken statt
  Beenden, Meldungen mit Absenderbild, mehrere Konten, `whatsapp://`-Links

Name: „Schnatter“ (deutsch und englisch gleich), im Menü unter Internet ·
Beschreibung: „WhatsApp Web als eigenes Fenster“ / „WhatsApp Web in its own
window“ · Anwendungskennung (App-ID, D-Bus-Name): `org.querencia.Schnatter` ·
Binary: `querencia-schnatter` · Debian-Paket: `querencia-schnatter`
(optional, nicht im Metapaket `querencia-desktop`; ersetzt
`querencia-webhuelle`) · 0.2.0: `org.querencia.Webhuelle`,
`querencia-webhuelle` · bis 0.1.1: `org.querencia.Nachrichten`,
`querencia-nachrichten`

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
| Version | 0.3.1 (Tag `schnatter/0.3.1`): Italienisch, Spanisch, Französisch; 0.3.0 (Tag `schnatter/0.3.0`, Zweig `schnatter`): Umbenennung in Schnatter mit Umzug der Ordner beider früheren Namen und neuem Symbol, zur Freigabe (Integrationsfassung `integration/schnatter-030/`, Tag `schnatter/0.3.0-int`); live ist Querencia Webhülle 0.2.0 seit 24.10.2026 14:48 (`webhuelle/0.2.0-int`) |
| Letzte Prüfung | 26.10.2026 (0.3.0): siehe [docs/testing.md](docs/testing.md) („Messungen 0.3.0“) und [docs/integration.md](docs/integration.md) |
| Bekannte Lücken | siehe [docs/testing.md](docs/testing.md), Abschnitt „Nicht getestet“ |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, WebKitGTK 2.52.6, Rust 1.93 |
| Nicht getestet | andere Distributionen, alles nach der Anmeldung (automatische Prüfungen nie mit Konto), HiDPI, Mehrmonitor |

Änderungen: [CHANGELOG.md](CHANGELOG.md).

## Bauen und Prüfen

`cargo build --release --locked`, danach `cargo fmt --all --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`
und `python3 werkzeug/i18n.py pruefen` (ohne Bildschirm). Voraussetzung:
`libwebkitgtk-6.0-dev`; die Runtime-Bibliotheken liegen als Checkout
`../querencia-runtime` neben diesem Repo (siehe unten). Einzelheiten: [docs/building.md](docs/building.md);
Prüfhülle, Prüfseite und Anmeldeseiten-Wächter (`bash werkzeug/lauf.sh …`,
nie in der laufenden Sitzung): [docs/testing.md](docs/testing.md);
Integration: [docs/integration.md](docs/integration.md).

**Runtime daneben:** `querencia-oberflaeche` (0.1.1) und
`querencia-dateiarbeit` sind Pfad-Abhängigkeiten auf
`../querencia-runtime/crates/…`. Die Runtime ist das öffentliche Repo
[querencia-shell](https://github.com/querencia-desktop/querencia-shell)
(ab Tag `dateien/0.2.2`); es wird unter dem Namen `querencia-runtime` neben
dieses Repo geklont:

```bash
git clone https://github.com/querencia-desktop/querencia-shell.git ../querencia-runtime
```

Die CI nimmt einen festen Commit davon (`8f102f1`, Schritt „Runtime daneben“
in `.github/workflows/ci.yml`).

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
ein eigenes (grüne Sprechblase mit Auge und einem Schnabel in
Querencia-Orange – kein Kreis, kein Telefonhörer).

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
