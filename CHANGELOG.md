# Änderungsprotokoll

Format nach [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), Versionen
nach [Semantic Versioning](https://semver.org/lang/de/). Solange die Hauptversion
0 ist, kann sich alles ändern. Jede Zeile nennt, was Nutzer merken; Unfertiges
steht als „unfertig“ da.

## [0.3.1] – 2026-10-26

### Hinzugefügt
- Oberfläche auf Italienisch, Spanisch und Französisch (`po/it.po`,
  `po/es.po`, `po/fr.po`, je 105 Texte) samt Desktop-Datei und MetaInfo
  (Zusammenfassung, Beschreibung, Stichwörter). Die Tastennamen der Menüs
  kommen aus dem Katalog `querencia-oberflaeche` der Runtime (it/es/fr seit
  oberflaeche 0.1.2); `werkzeug/i18n.py mo` braucht deshalb die Runtime mit
  diesen Katalogen.

## [0.3.0] – 2026-10-26 „Schnatter“

### Geändert
- Neuer Name: **Schnatter** (deutsch und englisch gleich), im Menü unter
  Internet; Beschreibung „WhatsApp Web als eigenes Fenster“. Die Marke
  WhatsApp steht weiter nur beschreibend da, nie im Namen oder im Symbol.
  Programmdatei `querencia-schnatter`, Anwendungskennung
  `org.querencia.Schnatter`, Paket `querencia-schnatter` (ersetzt
  `querencia-webhuelle`).
- Neues Programmsymbol: eine grüne Sprechblase mit Auge und Schnabel (statt
  des Fensters mit Globus).
- Menüleiste mit Querencia Oberfläche 0.1.1 (wie Querencia Dateien 0.2.2).

### Neu
- Umzug von Querencia Webhülle 0.2.0 und Querencia Nachrichten 0.1.x: Beim
  ersten Start übernimmt Schnatter Anmeldung, Einstellungen und Cache aus
  den Ordnern `querencia-webhuelle` – oder, wer 0.2.0 übersprungen hat, aus
  `querencia-nachrichten` (umbenannt, nichts kopiert oder gelöscht). Läuft
  eines der alten Programme noch, sagt das Fenster es und Schnatter startet
  erst nach dem Schließen. Gibt es schon Ordner von Schnatter, bleiben die
  alten unangetastet liegen.

## [0.2.0] – 2026-10-24 „Webhülle“

### Geändert
- Neuer Name: **Querencia Webhülle** (englisch „Querencia Web Wrapper“), im
  Menü „Webhülle“ unter Internet. Das Programm ist eine Hülle für WhatsApp
  Web und kein Produkt von WhatsApp oder Meta; die Marke steht nur noch
  beschreibend da („Hülle für WhatsApp Web“), nie im Namen oder im Symbol.
  Programmdatei `querencia-webhuelle`, Anwendungskennung
  `org.querencia.Webhuelle`.
- Eigenes Programmsymbol: ein Fenster mit Adresszeile und Globus in
  Querencia-Orange (statt des Chat-Symbols aus dem Symbolthema).
- Der Info-Dialog nennt „Hülle für WhatsApp Web“ und „Kein Produkt von
  WhatsApp oder Meta und nicht mit ihnen verbunden“.
- Querencia Webhülle ist nicht mehr Teil der Standardinstallation, sondern
  ein eigenes Paket, das man wahlweise installiert.

### Neu
- Umzug von Querencia Nachrichten: Beim ersten Start übernimmt die Webhülle
  Anmeldung, Einstellungen und Cache aus den Ordnern
  `querencia-nachrichten` (umbenannt, nichts kopiert oder gelöscht). Läuft
  Querencia Nachrichten noch, sagt das Fenster es und die Webhülle startet
  erst nach dem Schließen. Gibt es schon Ordner der Webhülle, bleiben die
  alten unangetastet liegen.

## [0.1.1] – 2026-10-23

### Neu
- Software-Grafik für die Webseite, wo die Grafikhardware versagt: Läuft
  OpenGL über zink auf nouveau (NVIDIA mit freiem Treiber, Mesa 26), stürzte
  die Seite beim Öffnen eines Bildes ab (QD-169). Nachrichten erkennt das
  beim Start und zeichnet die Seite dann in Software (llvmpipe). Bearbeiten →
  Einstellungen → „Software-Grafik“: automatisch (Vorgabe), an, aus; gilt ab
  dem nächsten Start. Der Info-Dialog nennt den Renderer der Seite und den
  Grund. Kostet auf der Anmeldeseite rund 240 MiB mehr Speicher.
- Links mit anderen Schemata als http, https und mailto (z. B. `ssh:`,
  `smb:`, eigene Schemata) öffnen erst nach der Rückfrage „Link mit anderem
  Programm öffnen?“, die das Ziel zeigt; Vorgabe ist „Abbrechen“.

### Geändert
- Die Seite kann das Fenster nicht mehr auf `data:`-, `file:`- oder fremde
  `blob:`-Adressen wechseln; jede Ablehnung steht ohne Inhalt im Protokoll.
  Eingebettete Rahmen dürfen `data:` weiter laden.
- Im Protokoll stehen Adressen anderer Schemata als http(s) nur noch als
  Schema (kein Inhalt aus `mailto:` oder `data:`).

### Behoben
- Die Meldung „Der Datenordner ist nicht nutzbar“ war nicht übersetzt.

### Bekannt
- `javascript:`-Adressen fragt WebKit 2.52 nicht bei der Hülle an; es führt
  sie nur im eigenen Ursprung der Seite aus, wie ein Skript der Seite.

## [0.1.0] – 2026-10-23 „Hülle“

### Neu
- Fenster mit WhatsApp Web (WebKitGTK 6.0) und der klassischen Querencia-
  Menüleiste: Datei (Neu laden, Abmelden und Daten löschen, Beenden),
  Bearbeiten (Ausschneiden, Kopieren, Einfügen, Alles auswählen,
  Einstellungen), Ansicht (Vergrößern, Verkleinern, Normale Größe, Vollbild),
  Hilfe (Hilfe, Info). Fenstergröße und Zoom werden gemerkt.
- Dauerhafte Anmeldung; Sitzungsdaten in eigenen Ordnern (0700).
  „Abmelden und Daten löschen“ entfernt sie nach Rückfrage.
- Meldungen neuer Nachrichten über die Benachrichtigungen des Desktops; ein
  Klick holt das Fenster nach vorn. Ungelesene Nachrichten im Fenstertitel.
- Downloads ohne Rückfrage in den Download-Ordner („foto 2.jpg“ statt
  Überschreiben), danach eine Meldung mit „Öffnen“ und „Ordner öffnen“.
- Mikrofon, Kamera und Lesen der Zwischenablage: eine Frage je Art, die
  Antwort wird gemerkt; Bearbeiten → Einstellungen vergisst sie.
- Töne und Sprachnachrichten spielen ohne Klick (nur in diesem Fenster);
  abschaltbar in den Einstellungen.
- Fremde Links öffnet der Standardbrowser.
- Rechtschreibprüfung Deutsch (Sprache der Oberfläche) und Englisch; die
  Seite bekommt die Sprache der Oberfläche (WhatsApp Web deutsch, auch wenn
  `LANG` englisch ist).
- Nach einem Absturz des Web-Prozesses lädt das Fenster neu und sagt es.
- Browserkennung umschaltbar: WebKit-Standard oder Linux-Kennung.
- Deutsch und Englisch; Starter „Nachrichten“ im Menü Internet.

### Bekannt (unfertig)
- Keine Anrufe (Ubuntus WebKitGTK ohne WebRTC).
- Kein Panel-Symbol, Schließen beendet das Programm (Welle 0.2).
- Meldungen ohne Absenderbild; ob ein Klick das Fenster unter labwc nach
  vorn holt, hängt am Aktivierungs-Token (Welle 0.2).
- Programmsymbol vorerst aus dem Symbolthema.
