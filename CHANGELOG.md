# Änderungsprotokoll

Format nach [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), Versionen
nach [Semantic Versioning](https://semver.org/lang/de/). Solange die Hauptversion
0 ist, kann sich alles ändern. Jede Zeile nennt, was Nutzer merken; Unfertiges
steht als „unfertig“ da.

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
