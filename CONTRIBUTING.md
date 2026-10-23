# Mitmachen bei Querencia Nachrichten

Danke für das Interesse. Querencia ist in einer frühen Alpha-Phase; bitte
rechne mit Rückfragen und langsamen Antworten.

## Fehler melden

In der gemeinsamen Meldestelle:
<https://github.com/querencia-desktop/querencia/issues/new/choose>.

**Sicherheitslücken nicht öffentlich melden:** siehe [SECURITY.md](SECURITY.md).

## Vor einem Pull Request

- Nie in der laufenden Sitzung testen. GUI-Prüfungen laufen in der
  Prüfhülle (`werkzeug/huelle.sh`: eigener labwc ohne Bildschirm, eigenes
  `HOME`, `XDG_RUNTIME_DIR`, eigene Busse; siehe docs/testing.md).
- **Nie mit einem WhatsApp-Konto testen.** Automatische Prüfungen laufen nur
  gegen die lokale Prüfseite und die Anmeldeseite.
- Bei Änderungen an der Oberfläche zusätzlich ein isolierter GUI-Lauf in
  1024×768 und 1920×1080 mit Bildschirmfotos als Beleg.

Checkliste:

- [ ] Build: `cargo build --locked`
- [ ] Format: `cargo fmt --all --check`
- [ ] Lint: `cargo clippy --all-targets --locked -- -D warnings`
- [ ] Tests: `cargo test --locked`; jede Fehlerbehebung bekommt einen Test,
      der vorher scheitert
- [ ] Lizenzkopf bzw. Lizenzangabe: neue Dateien stehen unter der Lizenz des
      Repos; Fremdmaterial mit Herkunft und Lizenz in `COPYRIGHT`
- [ ] Keine Geheimnisse (Schlüssel, Passwörter, Tokens), keine privaten Pfade
      oder Adressen
- [ ] `CHANGELOG.md` ergänzt; Unfertiges ist als unfertig gekennzeichnet

Über Merge und Release entscheiden die Betreuer des Projekts.

## Regeln für den Code

1. Vorab ein Issue, wenn es um mehr als eine kleine Korrektur geht –
   besonders bei Bedienung und Optik. Maßstab sind die übrigen Querencia-Programme (GNOME 2).
2. Zweig von `main`, kleine, thematisch geschlossene Commits
   (Conventional Commits).
3. App-IDs, Desktop-Dateien und D-Bus-Namen folgen `org.querencia.*`.
4. Keine fest verdrahteten Pfade (`/home/<name>`, `~/projects`, `~/opt`,
   `target/`), keine privaten Adressen in Code, Tests und Doku.
5. Symbole: nie 48×48 zusammen mit `scalable` installieren (GTK-4.22-Absturz,
   siehe Regel M in `querencia-packaging`).
6. Neue Abhängigkeiten nur mit Begründung; die Lizenz muss zu GPL-3.0-or-later
   passen und in `COPYRIGHT` stehen. Nichtfreie Anteile (z. B. UnRAR) sind
   ausgeschlossen.
7. Quellcode und Texte ohne Fremdmaterial: keine Übernahme aus den Vorbild­
   programmen ZapZap und WasIstLos (siehe README, „Vorbilder“).
8. Dienstbezogenes gehört nach `src/dienst/whatsapp.rs`, alles andere in die
   dienstneutrale Hülle `src/huelle/`.

## Lizenz der Beiträge

Mit einem Beitrag stimmst du zu, dass er unter der Lizenz dieses Repos
(GPL-3.0-or-later) veröffentlicht wird. Du versicherst, dass du ihn selbst
geschrieben hast oder die Lizenz des Ursprungs es erlaubt.
