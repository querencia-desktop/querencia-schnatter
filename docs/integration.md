# Integration in die laufende Sitzung

## Fassung 0.3.0: Schnatter löst Querencia Webhülle ab (`integration/schnatter-030/`)

`schnatter-030-integrieren.sh` (Tag `schnatter/0.3.0-int`, Code-Tag
`schnatter/0.3.0`) wechselt den Live-Stand Querencia Webhülle 0.2.0 (Ablage
`ab424b7fc122d7e9`) auf Schnatter 0.3.0 und mit `zurueck` wieder auf 0.2.0.
Live braucht es `SCHNATTER_FREIGABE=ja` (Freigabe der Status-Session); mit
umgelenktem `HOME` arbeitet es nur dort (Probe).

| Kürzel | Komponente | aktivieren | zurueck |
|---|---|---|---|
| A | Ablage `~/opt/querencia/schnatter/<sha16>/` (Binary 0555, `share/locale/de/LC_MESSAGES/querencia-schnatter.mo`, `querencia-oberflaeche.mo`, `share/icons/hicolor/scalable/apps/org.querencia.Schnatter.svg`, `SHA256SUMS` 0444; `.herkunft`) | anlegen (unter Hilfsnamen, geprüft, umbenannt) | bleibt liegen |
| PN | Link `~/opt/querencia/bin/querencia-schnatter` | setzen | entfernen |
| S | Symbol-Link `~/.local/share/icons/hicolor/scalable/apps/org.querencia.Schnatter.svg` → Ablage (danach `touch` auf `~/.local/share/icons/hicolor`) | setzen | entfernen |
| DN | Starter `~/.local/share/applications/org.querencia.Schnatter.desktop` („Schnatter“, Internet, `Icon=org.querencia.Schnatter`) | setzen | entfernen |
| M | Ordner `~/.local/share`, `~/.cache`, `~/.config`: `querencia-webhuelle` ↔ `querencia-schnatter` | umbenennen alt → neu (`mv`, nichts kopiert oder gelöscht) | umbenennen neu → alt |
| PA | Link `~/opt/querencia/bin/querencia-webhuelle` (0.2.0) | **erst nach gelungenem Umzug** entfernen | wiederherstellen |
| DA | Starter `org.querencia.Webhuelle.desktop` (0.2.0, Vorlage aus `integration/webhuelle-020`) | **erst nach gelungenem Umzug** entfernen | wiederherstellen |

- Der Symbol-Link `org.querencia.Webhuelle.svg` der Webhülle bleibt stehen
  (er zeigt in die bleibende Ablage 0.2.0; der Rückfall braucht ihn).
- Läuft Querencia Nachrichten, Querencia Webhülle oder Schnatter
  (`/proc/<pid>/exe`), verweigert das Skript jeden anstehenden Umzug
  (Exit 2): Der Netzwerkprozess schriebe sonst weiter in den verschobenen
  Ordner.
- Besteht ein Ordner unter beiden Namen, liegt ein Link oder eine Datei an
  einer der Stellen oder ist von Querencia Nachrichten noch etwas da (Link,
  Starter oder ein Ordner `querencia-nachrichten`), verweigert es (Exit 2);
  nichts wird zusammengeführt. Wer 0.2.0 übersprungen hat, bekommt den
  Umzug von `querencia-nachrichten` vom Programm selbst (oder vom Paket).
- `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` müssen fehlen oder
  auf die Vorgaben zeigen.
- Sperren: dieselben wie die Skripte von Nachrichten und Webhülle
  (`nachrichten-integrieren.lock`, `webhuelle-integrieren.lock` unter
  `~/.local/state/querencia/`) und eine eigene
  (`schnatter-integrieren.lock`). Sicherung
  `~/.local/state/querencia/sicherung-schnatter-030-*`: Zustand, Link-Ziele,
  Starter, Journal – **nie die Ordner selbst**. Bei einem Fehler oder
  Signal rollt es zurück (Exit 1), auch einen schon gemachten Umzug.
- Startet nichts. Ab dem nächsten Start aus dem Menü läuft Schnatter.
- Rückbau-Reihenfolge: zuerst dieses Skript (→ 0.2.0), dann bei Bedarf
  `integration/webhuelle-020` (→ 0.1.1). Solange 0.3.0 aktiv ist,
  verweigert `integration/webhuelle-020` (weder der Link
  `querencia-webhuelle` noch `querencia-nachrichten` ist da).

```bash
bash integration/schnatter-030/schnatter-030-integrieren.sh pruefen
```

```bash
SCHNATTER_FREIGABE=ja bash integration/schnatter-030/schnatter-030-integrieren.sh aktivieren
```

```bash
SCHNATTER_FREIGABE=ja bash integration/schnatter-030/schnatter-030-integrieren.sh zurueck
```

Probe: `bash integration/schnatter-030/probe.sh gui-test/integration-schnatter-030`
baut im Wegwerf-HOME den Live-Stand 0.2.0 nach (Kopien der Ablagen 0.2.0,
0.1.1 und 0.1.0, Link, Starter, Symbol-Link) mit Ordnern unter dem Namen der
Webhülle, die nur eine Attrappe enthalten, und prüft aktivieren/zurueck
(Inode, Inhalt, Rechte der Ordner), Idempotenz, laufende Programme unter
allen drei Namen, Ordnerzustände, Reste von Nachrichten, Fehler und Signale
an jeder Stelle, Fremdes, veränderte Vorlagen und Ablagen, alle drei
Sperren und die Kette mit `integration/webhuelle-020` (0.3.0 → 0.2.0 →
0.1.1 → 0.2.0 → 0.3.0, Attrappe und Inode bleiben). Live nur zwei
Verweigerungen ohne Freigabe; am Ende unverändert (verglichen werden nur
Existenz und Inode der Sitzungsordner, nie ihr Inhalt).

## Fassung 0.2.0: Querencia Webhülle löst Querencia Nachrichten ab (`integration/webhuelle-020/`)

`webhuelle-020-integrieren.sh` (Tag `webhuelle/0.2.0-int`, Code-Tag
`webhuelle/0.2.0`) wechselt den Live-Stand Querencia Nachrichten 0.1.1
(Ablage `7ce4b992509920bb`) auf Querencia Webhülle 0.2.0 und mit `zurueck`
wieder auf 0.1.1. Live braucht es `WEBH_FREIGABE=ja` (Freigabe der
Status-Session); mit umgelenktem `HOME` arbeitet es nur dort (Probe).

| Kürzel | Komponente | aktivieren | zurueck |
|---|---|---|---|
| A | Ablage `~/opt/querencia/webhuelle/<sha16>/` (Binary 0555, `share/locale/de/LC_MESSAGES/querencia-webhuelle.mo`, `querencia-oberflaeche.mo`, `share/icons/hicolor/scalable/apps/org.querencia.Webhuelle.svg`, `SHA256SUMS` 0444; `.herkunft`) | anlegen (unter Hilfsnamen, geprüft, umbenannt) | bleibt liegen |
| PN | Link `~/opt/querencia/bin/querencia-webhuelle` | setzen | entfernen |
| S | Symbol-Link `~/.local/share/icons/hicolor/scalable/apps/org.querencia.Webhuelle.svg` → Ablage (danach `touch` auf `~/.local/share/icons/hicolor`) | setzen | entfernen |
| DN | Starter `~/.local/share/applications/org.querencia.Webhuelle.desktop` („Webhülle“, Internet, `Icon=org.querencia.Webhuelle`) | setzen | entfernen |
| M | Ordner `~/.local/share`, `~/.cache`, `~/.config`: `querencia-nachrichten` ↔ `querencia-webhuelle` | umbenennen alt → neu (`mv`, nichts kopiert oder gelöscht) | umbenennen neu → alt |
| PA | Link `~/opt/querencia/bin/querencia-nachrichten` (0.1.1) | **erst nach gelungenem Umzug** entfernen | wiederherstellen |
| DA | Starter `org.querencia.Nachrichten.desktop` (0.1.1, Vorlage aus `integration/011`, Symbol-Rückfall wie dort) | **erst nach gelungenem Umzug** entfernen | wiederherstellen |

- Läuft Querencia Nachrichten oder Querencia Webhülle (`/proc/<pid>/exe`),
  verweigert das Skript jeden anstehenden Umzug (Exit 2): Der
  Netzwerkprozess schriebe sonst weiter in den verschobenen Ordner.
- Besteht ein Ordner unter beiden Namen oder liegt ein Link oder eine Datei
  an einer der Stellen, verweigert es (Exit 2); nichts wird zusammengeführt.
- `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` müssen fehlen oder
  auf die Vorgaben zeigen.
- Sperren: dieselbe wie die Nachrichten-Skripte
  (`~/.local/state/querencia/nachrichten-integrieren.lock`) und eine eigene
  (`webhuelle-integrieren.lock`). Sicherung
  `~/.local/state/querencia/sicherung-webhuelle-020-*`: Zustand, Link-Ziele,
  Starter, Journal – **nie die Ordner selbst** (Benutzerdaten). Bei einem
  Fehler oder Signal rollt es zurück (Exit 1), auch einen schon gemachten
  Umzug.
- Startet nichts. Ab dem nächsten Start aus dem Menü läuft die Webhülle; das
  Programm selbst würde den Umzug beim ersten Start genauso machen
  (`src/huelle/umzug.rs`) und findet danach nichts mehr zu tun.
- Rückbau-Reihenfolge: zuerst dieses Skript (→ 0.1.1), dann bei Bedarf
  `integration/011` (→ 0.1.0). Solange 0.2.0 aktiv ist, verweigert
  `integration/011` (der Link `querencia-nachrichten` fehlt).

```bash
bash integration/webhuelle-020/webhuelle-020-integrieren.sh pruefen
```

```bash
WEBH_FREIGABE=ja bash integration/webhuelle-020/webhuelle-020-integrieren.sh aktivieren
```

```bash
WEBH_FREIGABE=ja bash integration/webhuelle-020/webhuelle-020-integrieren.sh zurueck
```

Probe: `bash integration/webhuelle-020/probe.sh gui-test/integration-webhuelle-020`
baut im Wegwerf-HOME den Live-Stand 0.1.1 nach (Kopien der Ablagen 0.1.0 und
0.1.1, Link, Starter) mit Ordnern unter dem alten Namen, die nur eine
Attrappe enthalten, und prüft aktivieren/zurueck (Inode, Inhalt, Rechte der
Ordner), Idempotenz, laufende Programme (eine Kopie von `sleep` unter dem
Programmnamen), Ordnerzustände, Fehler und Signale an jeder Stelle,
Fremdes, veränderte Vorlagen und Ablagen, beide Sperren und das
Zusammenspiel mit `integration/011`. Live nur zwei Verweigerungen ohne
Freigabe; am Ende unverändert (verglichen werden nur Existenz und Inode der
Sitzungsordner, nie ihr Inhalt).

## Ältere Fassungen (Querencia Nachrichten)

Drei Fassungen:

| Ordner | Tag | Binary | Inhalt |
|---|---|---|---|
| `integration/` | `nachrichten/0.1.0-int` | 0.1.0 `4c70dede…` | erste Fassung; seit 23.10.2026 17:14 live aktiviert, **Rückfallpfad im Journal, wird nie geändert** |
| `integration/int2/` | `nachrichten/0.1.0-int3` | 0.1.0 `4c70dede…` | Fassung mit Symbol-Rückfall: Starter mit `Icon=org.querencia.Nachrichten`, sobald das Symbol im Thema oder in hicolor liegt (QD-166), sonst `Icon=empathy`; übernimmt einen mit der ersten Fassung aktivierten Stand (dessen Starter ist ihr Zustand „thema“) |
| `integration/011/` | `nachrichten/0.1.1-int` | 0.1.1 `7ce4b992…` (Code-Tag `nachrichten/0.1.1`) | Versionswechsel 0.1.0 ↔ 0.1.1, Symbol-Rückfall wie `int2`; siehe unten |

Der Tag `nachrichten/0.1.0-int2` ist überholt (die Fassung lag dort kurz am
Rückfallpfad) und wird nicht benutzt.

| Datei (je Ordner) | Zweck |
|---|---|
| `nachrichten-integrieren.sh` | Skript (`pruefen`, `aktivieren`, `zurueck`), aus der `.sh.in` mit eingetragenen Prüfsummen |
| `vorlage-erzeugen.sh <code-commit>` | baut `vorlage/` (Release-Binary, zwei Kataloge, Starter-Vorlage(n), `ablage.sha256`, `COMMIT`, `SHA256SUMS`) und erzeugt das Skript |
| `probe.sh <ordner unter gui-test/>` | Probe mit umgelenktem `HOME` (`int2`: 117/0, `011`: 149/0) |
| `vorlage/` | Vorlage, schreibgeschützt (0444/0555); Binary und Kataloge nicht im Git |

**Nur bereitgestellt.** Aktivieren macht die Status-Session nach Prüfung und
Freigabe.

## Was `aktivieren` tut

Alle Pfade folgen aus `$HOME`. Ist `HOME` das echte Heimatverzeichnis, ist
der Lauf live und braucht `NACH_FREIGABE=ja`.

1. Ablage `~/opt/querencia/nachrichten/<sha16>/` (Verzeichnisse 0555):
   Binary 0555, `share/locale/de/LC_MESSAGES/querencia-nachrichten.mo` und
   `querencia-oberflaeche.mo` sowie `SHA256SUMS` 0444, daneben `.herkunft`.
   Sie entsteht unter einem Hilfsnamen, wird geprüft und erst dann
   umbenannt.
2. Link `~/opt/querencia/bin/querencia-nachrichten` auf die Ablage.
3. Starter `~/.local/share/applications/org.querencia.Nachrichten.desktop`
   (Exec/TryExec auf die Ablage, „Nachrichten“ unter Internet, kein
   MimeType, daher kein `update-desktop-database`). `int2`: Symbol nach
   Rückfall (siehe oben); ein erneutes `aktivieren` stellt um, sobald das
   eigene Symbol da ist.
4. Schlusskontrolle (Zustände, Invarianten), Journal
   `~/opt/querencia/protokoll.txt` mit Rückfallbefehl.

Vorher sichert das Skript Link und Starter nach
`~/.local/state/querencia/sicherung-nachrichten-010-*`. Bei einem Fehler
oder Signal rollt es zurück (Exit 1). Es startet nichts und legt keine
Sitzungsdaten an; die Ordner `~/.local/share/`, `~/.cache/` und
`~/.config/querencia-nachrichten` (0700) entstehen erst beim ersten Start.

## Was `zurueck` tut

Starter entfernen, Link entfernen. Die Ablage bleibt liegen (unveränderliche
Ablagen werden nie gelöscht), ebenso die Sitzungsordner (sie enthalten die
Anmeldung; leeren mit Datei → Abmelden und Daten löschen).

## Befehle

```bash
bash integration/int2/nachrichten-integrieren.sh pruefen
```

```bash
NACH_FREIGABE=ja bash integration/int2/nachrichten-integrieren.sh aktivieren
```

```bash
NACH_FREIGABE=ja bash integration/int2/nachrichten-integrieren.sh zurueck
```

Exit-Codes: 0 erledigt, 1 zurückgerollt, 2 verweigert (fremder Zustand,
veränderte Vorlage, fehlende Freigabe, belegte Sperre), 3 Rückrollen
unvollständig.

## Fassung 0.1.1 (`integration/011/`)

`nachrichten-011-integrieren.sh` wechselt den Live-Stand 0.1.0 auf 0.1.1 und
mit `zurueck` wieder auf 0.1.0. Komponenten wie oben; „alt“ ist 0.1.0 (Link
auf die Ablage `4c70dede91b11648`, Starter 0.1.0), „neu“ ist 0.1.1. Der
Starter ändert sich gegenüber 0.1.0 nur in Exec/TryExec; die
Starter-Vorlagen sind byte-gleich mit `int2` (`vorlage-erzeugen.sh` prüft
das). Symbol wie `int2`: `Icon=org.querencia.Nachrichten`, sobald das Symbol
da ist, sonst `Icon=empathy`; `zurueck` setzt den Starter 0.1.0 mit derselben
Regel. Ohne Stand 0.1.0 (Link oder Starter fehlt) verweigert es; die
Erstinstallation bleibt Sache von `integration/` bzw. `int2/`.

- `aktivieren`: neue Ablage (wie oben, `.herkunft` nennt 0.1.1), Link,
  Starter; die Ablage 0.1.0 bleibt unberührt. Sicherung
  `~/.local/state/querencia/sicherung-nachrichten-011-*`, dieselbe Sperre wie
  die 0.1.0-Skripte, Journal mit Rückfallbefehl. Ein offenes Fenster läuft
  bis zum Schließen mit 0.1.0 weiter; die Einstellung „Software-Grafik“ fehlt
  in den Einstellungen von 0.1.0 und gilt als automatisch.
- `zurueck`: Starter und Link wieder auf 0.1.0; die Ablage 0.1.1 bleibt.
- Rückbau-Reihenfolge: erst dieses Skript (→ 0.1.0), dann bei Bedarf das
  0.1.0-Skript; vorher meldet es Link und Starter von 0.1.1 als fremd und
  verweigert (Probe).
- QD-162 c: keine Prüfung mit `grep -q` am Ende einer Pipe (bei `pipefail`
  kann `grep -q` die Pipe vorzeitig schließen); Zeichenketten werden erst
  ganz erzeugt und dann geprüft (`grep -qx … <<<"$soll"`). Die Probe sucht das
  Muster im Skript.

Probe (`bash integration/011/probe.sh gui-test/integration-011`, 149/0):
baut im Wegwerf-HOME den Live-Stand 0.1.0 nach (Kopie der Ablage, Link,
Starter aus der 0.1.0-Vorlage) und prüft aktivieren/zurueck, Idempotenz,
Symbol-Rückfall (auch Ausgang mit `int3`-Starter und Umstellung), Fehler und
Signale an jeder Stelle (Exit 1, Baum unverändert), Fremdes und fehlendes
0.1.0 (Exit 2), Mischzustände, veränderte Ablagen und Vorlage, belegte
Sperre und das Zusammenspiel mit dem 0.1.0-Skript; live nur zwei
Verweigerungen ohne Freigabe, am Ende unverändert.

```bash
bash integration/011/nachrichten-011-integrieren.sh pruefen
```

```bash
NACH_FREIGABE=ja bash integration/011/nachrichten-011-integrieren.sh aktivieren
```

```bash
NACH_FREIGABE=ja bash integration/011/nachrichten-011-integrieren.sh zurueck
```
