# Änderungsprotokoll

Format nach [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), Versionen
nach [Semantic Versioning](https://semver.org/lang/de/). Solange die Hauptversion
0 ist, kann sich alles ändern. Jede Zeile nennt, was Nutzer merken; Unfertiges
steht als „unfertig“ da.

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
