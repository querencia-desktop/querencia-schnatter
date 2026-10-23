# Prüfen

**Nie mit einem WhatsApp-Konto.** Automatische Prüfungen laufen gegen die
lokale Prüfseite und gegen die Anmeldeseite (QR-Code), nie weiter. Alles
nach der Anmeldung prüft der Besitzer selbst (Prüfliste im Bericht der
Welle).

## Unit-Tests (ohne Bildschirm)

```bash
bash werkzeug/bauen.sh pruefen
```

`cargo test` prüft die reinen Teile: Titelzähler, erlaubte Hosts und
Navigationsregeln, Dateinamen und Kopienamen der Downloads, Einstellungen
(Vorgaben, Unsinn, Rechte vergessen), Ordner 0700 (auch wenn sie offen
bestanden; Link statt Ordner wird abgelehnt), vorgemerkte Löschung,
Zoomstufen, Sprachen der Rechtschreibung und der Seite, Menüs und Kürzel,
Vollständigkeit von `po/de.po`.

## Prüfhülle

`werkzeug/huelle.sh` folgt dem Prüfrahmen von Querencia Dateien
(`querencia-runtime/werkzeug/dateien-pruefrahmen/huelle.sh`,
pruefrahmen/0.1.1) mit drei Stufen:

| Stufe | Namensräume | Netz | WebKit-Sandbox | Zweck |
|---|---|---|---|---|
| L | bwrap: user, pid, net (nur lo), ipc, uts | nein | **aus** (verschachteltes bwrap verboten) | Prüfseite |
| N | wie L, aber Netz des Rechners, nur `/run/systemd/resolve` sichtbar | ja | aus | Anmeldeseite als Vergleich |
| E | keine (nur Umgebung, eigene Busse, eigener labwc, `setsid`) | ja | **an**, wie im echten Betrieb | Anmeldeseite, Speicher, Sandbox |

Alle Stufen: `env -i`, eigenes `HOME` und XDG-Ordner im Profil,
`XDG_RUNTIME_DIR=/tmp/qn-runtime-*` (nie `/tmp/qd-*`), privater
Session-Bus, Attrappen-System-Bus, kein Portal (`GTK_USE_PORTAL=0`,
`GDK_DEBUG=no-portals`), Restprüfung (`reste.py`) am Ende. In Stufe E
beendet die Hülle jeden Prozess ihrer Sitzung und jeden mit ihrem
Laufzeitordner (`eigene.py`).

Gemessen (23.10.2026): In L/N kann WebKit seine Sandbox nicht starten
(„No permissions to create a new namespace“); mit Sandbox bricht das
Programm in N beim ersten Web-Prozess ab („Failed to fully launch
dbus-proxy“). Deshalb setzen die Szenarien in L/N
`WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1`; die äußere Hülle ist dort die
Sandbox. In E läuft WebKits Sandbox (siehe unten).

## Prüfseite (Stufe L)

```bash
bash werkzeug/lauf.sh funktionen de 1920x1080 hell
bash werkzeug/lauf.sh funktionen en 1024x768 dunkel
```

`werkzeug/szenario-funktionen.sh` startet die Ablage-Kopie des Release-Baus
mit `QUERENCIA_NACHRICHTEN_PRUEFSEITE=werkzeug/pruefseite.html` (lokal unter
`http://localhost/`, Hosts und Benachrichtigungsrecht für `localhost`) und
`QUERENCIA_NACHRICHTEN_DEBUG=1`, eine Attrappe von
`org.freedesktop.Notifications` (`attrappe_benachrichtigungen.py`, klickt
nach Regel), einen Prüföffner statt Browser, Dateimanager und Texteditor
(`testoeffner.sh`) und einen HTTP-Server auf 127.0.0.1 in der Hülle. Echte
Nutzergesten kommen über `qeingabe` (virtuelle Tastatur im eigenen labwc),
Menüaktionen über `org.gtk.Actions` auf dem privaten Bus. Auf der Prüfseite
schreibt die Hülle die Konsole der Seite ins Protokoll und schaltet WebKits
Attrappen-Mikrofon und -Kamera ein (die Hülle hat keine Geräte; ohne Gerät
fragt WebKit gar nicht).

Geprüft (51 Punkte, `nachweis/funktionen/<lauf>/ergebnis.txt`):
Benachrichtigungsrecht vorab, Meldung an die Attrappe, Klick → `onclick`
und Fenster nach vorn; zwei Downloads gleichen Namens neben einer
vorhandenen Datei („probe-download 2.txt“, „… 3.txt“), Meldung mit
„Ordner öffnen“ und „Öffnen“; Autoplay; Titelzähler 3/12/0;
Rechtschreibung de_DE + en_US; Mikrofon (Frage, Ablehnen, gemerkt),
Kamera (Frage, Erlauben), Zwischenablage (Frage), Standort (abgelehnt);
fremder Link, Link in neuem Fenster, `window.open` → Prüföffner; eigener
Link bleibt; Dateiwahl (Signal, GTK-Dialog); Zoom gemerkt; fremde
Hauptseite ohne Geste wird an der Antwort verworfen, kein Browser;
Absturz des Web-Prozesses → Hinweis und Neuladen; Abmelden und Daten
löschen (Rückfrage, Löschen, vorgemerkte Löschung beim nächsten Start,
Rechte vergessen); Ordner 0700; Strg+Q, Fenstergröße gemerkt; Ton aus.
Dazu Fotos der Menüs, Dialoge, Vollbild, Hinweisleiste.

## Anmeldeseiten-Wächter (Stufe E, Muster `probe.py` der Machbarkeit)

```bash
bash werkzeug/lauf.sh anmeldeseite E de 1920x1080 hell standard
bash werkzeug/lauf.sh anmeldeseite E de 1024x768 dunkel linux
```

`werkzeug/szenario-anmeldeseite.sh` lädt web.whatsapp.com mit
`QUERENCIA_NACHRICHTEN_ZUSTAND=1` (Merkmale der Seite alle 2 s, ohne
Seitentext), wartet auf den QR-Code, fotografiert den ganzen Bildschirm,
misst Speicher und Sandbox (`messen.py`: PSS aus `smaps_rollup`,
Namensräume, Seccomp, NoNewPrivs je Prozess), beendet das Programm über
`app.beenden` und **löscht die Sitzung** des Profils (R9). Niemand scannt
den QR-Code; er gehört zu einer Sitzung, die mit dem Lauf endet. Kennung
über `einstellungen.ini` (`standard` oder `linux`), Schema über gsettings
im Profil. Vor jedem Release laufen lassen (R1).

Gemessen (23.10.2026, 8 Läufe): QR-Code nach 4,6–12,7 s; Standard:
macOS-Safari-Kennung (Quirk), Seite wirbt für die Mac-App; Linux: WebKits
Linux-Kennung, keine Mac-Werbung; Sprache der Seite `de`; die Anmeldeseite
selbst bleibt hell, auch wenn `prefers-color-scheme: dark` gilt. Speicher
(PSS) gesamt 730–773 MiB (Hülle 192–213, Netzwerkprozess 44–49,
Web-Prozesse 110–127 und 389–407); mit `GSK_RENDERER=cairo` 657 MiB
(Hülle 89). Sandbox: beide Web-Prozesse in eigenen Namensräumen user, pid,
net, mnt, ipc, uts, Seccomp 2, NoNewPrivs 1; `xdg-dbus-proxy` je
Web-Prozess; Netzwerkprozess ohne Sandbox (wie vorgesehen).

## Nicht getestet

- Alles nach der Anmeldung: Chats, Bilder senden, Sprachnachricht
  aufnehmen und abspielen, echte Meldungen von WhatsApp, Verhalten über
  Stunden, Speicher mit großen Chats (Abnahme durch den Besitzer).
- Dateiwahl über das Portal (in der Hülle gibt es kein Portal; gemessen ist
  der GTK-eigene Dialog) und Ziehen und Ablegen.
- Ob ein Klick auf eine Meldung das Fenster unter labwc nach vorn holt (R6,
  ohne Aktivierungs-Token); in der Hülle kommt `present` an.
- Echte Audioausgabe und echtes Mikrofon (die Hülle hat kein PipeWire;
  geprüft sind Autoplay-Zustand und WebKits Attrappen-Geräte).
- HiDPI, Mehrmonitor, Ruhezustand, Netzwechsel.
