# Integration in die laufende Sitzung

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
