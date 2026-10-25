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
Vollständigkeit von `po/de.po`; seit 0.1.1 auch Rückfrage vor anderen
Schemata, Sperren der Hauptseite (`data:`, `file:`, fremdes `blob:`,
`javascript:`), Erkennung „zink auf nouveau“ aus dem Renderer, Entscheidung
je Einstellung, Ausgabe der Grafik-Probe; seit 0.2.0 der Umzug der Ordner
(alle drei umbenannt mit Inhalt und Rechten, neuer Ordner besteht → alter
bleibt, Link oder Datei unter dem alten Namen bleibt unberührt, altes
Programm läuft → nichts, scheiterndes Umbenennen → zurückbenannt, Suche
nach dem laufenden Programm über `/proc/<pid>/exe`).

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
mit `QUERENCIA_SCHNATTER_PRUEFSEITE=werkzeug/pruefseite.html` (lokal unter
`http://localhost/`, Hosts und Benachrichtigungsrecht für `localhost`) und
`QUERENCIA_SCHNATTER_DEBUG=1`, eine Attrappe von
`org.freedesktop.Notifications` (`attrappe_benachrichtigungen.py`, klickt
nach Regel), einen Prüföffner statt Browser, Dateimanager und Texteditor
(`testoeffner.sh`) und einen HTTP-Server auf 127.0.0.1 in der Hülle. Echte
Nutzergesten kommen über `qeingabe` (virtuelle Tastatur im eigenen labwc),
Menüaktionen über `org.gtk.Actions` auf dem privaten Bus. Auf der Prüfseite
schreibt die Hülle die Konsole der Seite ins Protokoll und schaltet WebKits
Attrappen-Mikrofon und -Kamera ein (die Hülle hat keine Geräte; ohne Gerät
fragt WebKit gar nicht).

Geprüft (74 Punkte seit 0.1.1, `nachweis/funktionen/<lauf>/ergebnis.txt`):
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

Seit 0.1.1 (QD-162 a/b, Tasten der Prüfseite `s o j i l x q b y`; das Profil
trägt Prüföffner auch für `ssh:` und `smb:` ein): ein `ssh:`-Link fragt
„Link mit anderem Programm öffnen?“ (Foto); Escape und Eingabe (Vorgabe)
brechen ab, nichts wird geöffnet; erst „Link öffnen“ gibt ihn an den
Prüföffner. Ein `smb:`-Link mit `target=_blank` fragt ebenso; `mailto:` geht
ohne Frage an das Mailprogramm. Ein `data:`-Link (Klick) und `data:` ohne
Geste werden an der Antwort des Hauptrahmens gesperrt und protokolliert, ein
`data:`-Rahmen lädt weiter; `file:` und fremdes `blob:` lehnt schon WebKit
ab („Not allowed to load local resource“), die Seite bleibt. `javascript:`
ohne Geste: WebKit 2.52 fragt die Hülle nicht (0 Entscheidungen) und führt
die Adresse im eigenen Ursprung aus; das Szenario hält das nur fest.

## Umzug von Querencia Webhülle und Querencia Nachrichten (Stufe L, 0.3.0)

```bash
ALT_ABLAGE=~/opt/querencia/webhuelle/ab424b7fc122d7e9 bash werkzeug/lauf.sh umzug de 1920x1080 hell
ALT_ABLAGE=~/opt/querencia/nachrichten/7ce4b992509920bb bash werkzeug/lauf.sh umzug en 1024x768 dunkel
```

`werkzeug/szenario-umzug.sh` prüft den Umzug der Ordner `querencia-schnatter`
(0.2.0) bzw. `querencia-nachrichten` (0.1.1) → `querencia-schnatter` **nur mit
einer Attrappen-Anmeldung im Wegwerfprofil**: `ALT_ABLAGE` kopiert die
Ablage 0.2.0 oder 0.1.1 (nur gelesen) ins Profil, das Szenario erkennt an der
Binary, welche es ist; die Einstellungen liegen dort unter dem alten Namen.
Im Folgenden ist „alt“ diese Fassung. Die Prüfseite legt mit der
Taste `a` einen Zufallswert als Cookie, in localStorage und in IndexedDB an
(Taste `r` und jeder Start lesen ihn, Zeile „speicher“). Ablauf: (1) alt
legt die Attrappe an und merkt Zoom 1,25; danach bekommt jeder alte Ordner
eine Marke mit Zufallsinhalt, Inode und Dateiliste werden notiert. (2) alt
läuft, 0.3.0 startet: Ereignis `umzug blockiert name=… pid=…`, Hinweisfenster
(Foto), kein Start, kein neuer Ordner. (3) 0.3.0 allein: `umzug
daten=verschoben-<webhuelle|nachrichten> …` für alle drei Ordner, je Ordner eine Protokollzeile, alte
Namen weg, neue 0700 mit **derselben Inode** (umbenannt, nicht kopiert),
Marke byte-gleich, keine Datei verloren, Attrappe in localStorage und
IndexedDB da, Zoom übernommen; Info-Dialog (Foto). (4) Zweiter Start: nichts
zu tun. (5) Rückfall wie `integration/schnatter-030` `zurueck` (`mv` neu →
alt): alt findet die Attrappe; wieder vor: 0.3.0 auch. (6) Ein alter Ordner
taucht neben dem neuen auf: `daten=alt-bleibt`, er bleibt unberührt. Am Ende
läuft keines der Programme mehr. 58 Punkte. Die Fälle „beide alten Namen
liegen nebeneinander“, „Link unter dem neueren Namen“ und „Fehler beim
dritten Umbenennen“ decken die Unit-Tests in `src/huelle/umzug.rs` ab.

Gemessen in 0.2.0 (24.10.2026, Umzug von 0.1.1, je 58 von 58): Cookies überstehen schon in 0.1.1 keinen Neustart (WebKit ohne
dauerhaften Cookie-Speicher, kein `cookies.sqlite` im Datenordner; Zeilen
„INFO cookie … null“) – unabhängig vom Umzug. WhatsApp Web hält seine
Sitzung in IndexedDB und localStorage; beide überstehen Umzug, Rückfall und
Wiedervorlauf.

## Grafik-Rückfall (Stufe L, 0.1.1, QD-169)

```bash
VERGLEICH_BIN=<0.1.0-Binary> bash werkzeug/lauf.sh grafik de 1920x1080 hell
VERGLEICH_BIN=<0.1.0-Binary> bash werkzeug/lauf.sh grafik en 1024x768 dunkel
```

Dazu `de 1024x768 hell` und `en 1920x1080 dunkel` (alle vier Kombinationen).
`VERGLEICH_BIN` (wahlweise) kopiert eine zweite Binary ins Profil, nur
gelesen; sie läuft als Fall „vergleich“. Seit 0.3.0 nur mit Binaries ab
0.3.0 (Anwendungskennung, Ordner und Prüfvariablen heißen bei 0.1.x und 0.2.0 anders);
ohne `VERGLEICH_BIN` hat das Szenario 44 Punkte (24.10.2026: je 44 von 44). `GRAFIK_FAELLE`
legt Fälle und Reihenfolge fest, `NAMENSZUSATZ` hängt einen Zusatz an den
Namen des Laufs.

`werkzeug/szenario-grafik.sh` misst die Erkennung auf zwei Wegen: die
Grafik-Probe des Programms (`querencia-schnatter --grafik-probe`,
surfaceless EGL, GLES 2) gegen `eglinfo -B -p surfaceless`, je ohne und mit
den drei Variablen; dann startet es die Hülle gegen die Prüfseite viermal
(Einstellung automatisch, an, aus, dazu „Umgebung“ mit schon gesetzten
Variablen und „vergleich“) und liest je Fall die Protokollzeile „Grafik:“,
das Ereignis `grafik`, die geladenen Bibliotheken und die Grafik-Variablen
der Hülle und der Web-Prozesse aus `/proc/<pid>/maps` und
`/proc/<pid>/environ` (`werkzeug/karten.py`: libgallium, libvulkan_nouveau,
übrige Vulkan-Treiber, libEGL_mesa, die drei Variablen) und den Renderer im
Info-Dialog (Foto). Dazu öffnet es je Fall dreimal den Bildbetrachter der
Prüfseite (Taste `v`): ein Bild mit 3000×2000 Pixeln aus einem Canvas, als
PNG-Blob, blendet in einer eigenen Ebene mit Zoom und Deckkraft ein wie bei
WhatsApp Web. Gemessen werden Dekodieren, die erste Zeichnung nach dem
Einfügen und die Bildabstände der Animation (`requestAnimationFrame`, 0,9 s);
dazu gibt es ein Foto, und das Szenario prüft, dass kein Absturz folgt.

Gemessen (23.10.2026, RTX 3060 mit nouveau, Mesa 26.0.8, je 32 von 32):
Probe und eglinfo nennen beide „zink Vulkan 1.4(NVIDIA GeForce RTX 3060
(NVK GA106) (MESA_NVK))“, mit den Variablen beide „llvmpipe (LLVM 21.1.8,
256 bits)“; die Probe braucht 0,29–0,35 s (Kindprozess, ≈ 106 MiB nur
während der Probe). Automatisch, an und Umgebung: Web-Prozesse mit
libgallium und libEGL_mesa, **ohne** libvulkan_nouveau, Info-Dialog
„llvmpipe“; aus: Web-Prozesse mit libvulkan_nouveau (zink auf NVK). Die Hülle
selbst lädt in allen Fällen libvulkan_nouveau: GTK zeichnet über Vulkan, das
die Variablen nicht berühren.

Nachtrag (23.10.2026, alle vier Kombinationen, je 50 von 50, mit 0.1.0 als
„vergleich“): `environ` der Web-Prozesse mit genau den drei Variablen bei
automatisch, an und Umgebung, ohne sie bei aus und 0.1.0; das gilt auch in
Stufe E mit WebKits Sandbox. Im `environ` der Hülle selbst stehen sie nicht,
weil der Kernel dort die Umgebung beim Start zeigt; die Hülle setzt sie erst
danach. Der Bildbetrachter öffnet in allen Fällen ohne Absturz.

Zeichenzeit des Bildbetrachters (ms, Hülle ohne Bildschirm mit 60 Hz;
Dekodieren / erste Zeichnung / Bildabstand im Mittel / p95 / größter):

| Fall | Dekodieren | erste Zeichnung | Mittel | p95 | größter |
|---|---|---|---|---|---|
| 0.1.0 und 0.1.1 aus (zink auf NVK) | 67–77 | 17–31 | 17,0–18,5 | 17 | 78–137 |
| 0.1.1 llvmpipe, warmer Shader-Cache | 66–109 | 17–61 | 16,4–19,7 | 17–21 | 43–187 |
| 0.1.1 llvmpipe, erstes Öffnen mit kaltem Cache | 70–76 | 45–90 | ein Bild nach 1191–1255 | – | 1255 |

Die Animation läuft mit llvmpipe so flüssig wie mit zink. Einmalig, beim
ersten Bild mit llvmpipe in einem Profil, steht die Animation rund 1,2 s:
llvmpipe übersetzt dann seine Shader und legt sie im Mesa-Cache ab
(`~/.cache/mesa_shader_cache`). Mit umgekehrter Reihenfolge der Fälle trifft
es den jeweils ersten llvmpipe-Fall, nicht den Modus. Im echten Betrieb
geschieht das einmal je Mesa-Stand.

## Anmeldeseiten-Wächter (Stufe E, Muster `probe.py` der Machbarkeit)

```bash
bash werkzeug/lauf.sh anmeldeseite E de 1920x1080 hell standard
bash werkzeug/lauf.sh anmeldeseite E de 1024x768 dunkel linux
```

`werkzeug/szenario-anmeldeseite.sh` lädt web.whatsapp.com mit
`QUERENCIA_SCHNATTER_ZUSTAND=1` (Merkmale der Seite alle 2 s, ohne
Seitentext), wartet auf den QR-Code, fotografiert den ganzen Bildschirm,
misst Speicher und Sandbox (`messen.py`: PSS aus `smaps_rollup`,
Namensräume, Seccomp, NoNewPrivs je Prozess), beendet das Programm über
`app.beenden` und **löscht die Sitzung** des Profils (R9). Niemand scannt
den QR-Code; er gehört zu einer Sitzung, die mit dem Lauf endet. Kennung
über `einstellungen.ini` (`standard` oder `linux`), Schema über gsettings
im Profil. Vor jedem Release laufen lassen (R1).

Gemessen (23.10.2026, 8 Läufe, Binary `4c70dede…`): QR-Code nach 4,6–4,7 s;
Standard: macOS-Safari-Kennung (Quirk), Seite wirbt für die Mac-App; Linux:
WebKits Linux-Kennung, keine Mac-Werbung; Sprache der Seite `de`; die
Anmeldeseite selbst bleibt hell, auch wenn `prefers-color-scheme: dark`
gilt. Speicher (PSS) gesamt 748–787 MiB (Hülle 188–208, Netzwerkprozess
40–48, Web-Prozesse ≈ 124 und 395–412); mit `GSK_RENDERER=cairo` 668 MiB
(Hülle 88). Sandbox: beide Web-Prozesse in eigenen Namensräumen user, pid,
net, mnt, ipc, uts, Seccomp 2, NoNewPrivs 1; `xdg-dbus-proxy` je
Web-Prozess; Netzwerkprozess ohne Sandbox (wie vorgesehen).

0.1.1 gegen 0.1.0 (23.10.2026, Stufe E, de, 1920×1080, hell, Kennung
Standard; PSS in MiB, Hülle / Netzwerk / Web-Prozesse):

| Binary | Grafik | PSS gesamt | Aufteilung | QR-Code |
|---|---|---|---|---|
| 0.1.0 `4c70dede…` (2 Läufe) | zink auf NVK | 737, 739 | 184–191 / 42–48 / 120–121 + 382–383 | 6,6 s |
| 0.1.1 automatisch (2 Läufe) | llvmpipe | 976, 980 | 189–195 / 44 / 149–156 + 585–590 | 4,7–6,5 s |
| 0.1.1 aus | zink auf NVK | 746 | 196 / 44 / 120 + 384 | 6,7 s |
| 0.1.1 automatisch, 1024×768 dunkel, Linux-Kennung | llvmpipe | 937 | 178 / 43 / 155 + 558 | 4,8 s |
| 0.1.0, 1920×1080 dunkel | zink auf NVK | 758 | 196 / – / 121 + 392 | 4,7 s |
| 0.1.1 automatisch, 1920×1080 dunkel | llvmpipe | 997 | 200 / – / 157 + 594 | 4,5 s |
| 0.1.1 automatisch, 1024×768 hell | llvmpipe | 940 | 177 / – / 147 + 570 | 4,5 s |

Software-Grafik kostet auf der Anmeldeseite rund 240 MiB, fast alles im
Web-Prozess der Seite (llvmpipe hält Texturen im Hauptspeicher). Die Karten
der Web-Prozesse bestätigen den Weg: 0.1.0 und „aus“ mit libvulkan_nouveau,
0.1.1 automatisch ohne. Sandbox unverändert (user, pid, net, mnt, ipc, uts,
Seccomp 2, NoNewPrivs 1).

Seit 0.2.0 (24.10.2026, Binary `ab424b7f…`): de/1920×1080/hell Kennung
Standard QR nach 10,7 s, PSS 936 MiB; en/1024×768/dunkel Kennung Linux QR
nach 6,6 s, PSS 773 MiB; Web-Prozesse mit libgallium ohne libvulkan_nouveau,
WebKit-Sandbox an, Sitzung danach gelöscht.

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
- Der Umzug mit einer echten Anmeldung (nur mit Konto; geprüft mit einer
  Attrappe in localStorage und IndexedDB, Berichte der Wellen webhuelle und
  schnatter).
- Der Bildbetrachter von WhatsApp Web mit Software-Grafik (nur mit Konto;
  der Besitzer hat den Weg mit den drei Variablen in der Shell bestätigt, QD-169).
- Die Grafik-Probe auf anderer Hardware (AMD, Intel, NVIDIA-Treiber): geprüft
  sind dort nur die Regeln der Erkennung (Unit-Tests).
