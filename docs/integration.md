# Integration in die laufende Sitzung

Zwei Fassungen, beide mit derselben Binary (`4c70dede…`, Code-Tag
`nachrichten/0.1.0`):

| Ordner | Tag | Inhalt |
|---|---|---|
| `integration/` | `nachrichten/0.1.0-int` | erste Fassung; seit 23.10.2026 17:14 live aktiviert, **Rückfallpfad im Journal, wird nie geändert** |
| `integration/int2/` | `nachrichten/0.1.0-int3` | Fassung mit Symbol-Rückfall: Starter mit `Icon=org.querencia.Nachrichten`, sobald das Symbol im Thema oder in hicolor liegt (QD-166), sonst `Icon=empathy`; übernimmt einen mit der ersten Fassung aktivierten Stand (dessen Starter ist ihr Zustand „thema“) |

Der Tag `nachrichten/0.1.0-int2` ist überholt (die Fassung lag dort kurz am
Rückfallpfad) und wird nicht benutzt.

| Datei (je Ordner) | Zweck |
|---|---|
| `nachrichten-integrieren.sh` | Skript (`pruefen`, `aktivieren`, `zurueck`), aus der `.sh.in` mit eingetragenen Prüfsummen |
| `vorlage-erzeugen.sh <code-commit>` | baut `vorlage/` (Release-Binary, zwei Kataloge, Starter-Vorlage(n), `ablage.sha256`, `COMMIT`, `SHA256SUMS`) und erzeugt das Skript |
| `probe.sh <ordner unter gui-test/>` | Probe mit umgelenktem `HOME` (`int2`: 117/0) |
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
