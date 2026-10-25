#!/bin/bash
# Prüfhülle Schnatter, Stufe L: die Hülle gegen die lokale
# Prüfseite (pruefseite.html), ohne Netz. Läuft IN der Hülle (lauf.sh).
# Meldungen gehen an die Attrappe, Öffnen an testoeffner.sh, Eingaben über
# qeingabe (virtuelle Tastatur im eigenen labwc). Ergebnis: $OUT/ (Fotos,
# Protokolle, ergebnis.txt mit einer Zeile je Prüfpunkt).
set -u
. "$QN_PROFIL/env.sh"
Q=$W/bin/qeingabe; QA=$W/bin/qausgabe
L=$QN_PROFIL/logs
E=$OUT/ergebnis.txt
: > "$E"
"$QA" "$RES" >>"$L/szenario.log" 2>&1
n=0
foto() { n=$((n + 1)); grim "$(printf '%s/%02d-%s-%s-%s-%s.png' "$OUT" "$n" "$1" "$SPR" "$RES" "$SCHEMA")"; }
# wartet, bis $2 (Datei) eine Zeile mit dem Muster $1 hat (höchstens $3 s)
warte() { local i; for i in $(seq 1 $(($3 * 10))); do grep -qE "$1" "$2" 2>/dev/null && return 0; sleep 0.1; done; return 1; }
pruefe() { # NAME MUSTER DATEI ZEIT
  if warte "$2" "$3" "$4"; then echo "PASS $1" >>"$E"; else echo "FAIL $1 (fehlt: $2 in $(basename "$3"))" >>"$E"; fi
}
aktion() { # win|app NAME
  local pfad=/org/querencia/Schnatter
  [ "$1" = win ] && pfad=$pfad/window/1
  gdbus call --session --dest org.querencia.Schnatter --object-path "$pfad" \
    --method org.gtk.Actions.Activate "$2" '[]' '{}' >>"$L/szenario.log" 2>&1
}
webprozesse() { # PIDs der WebKitWebProcess unter der App
  local p
  for p in /proc/[0-9]*; do
    [ "$(cat "$p/comm" 2>/dev/null)" = WebKitWebProces ] && echo "${p#/proc/}"
  done
}

# Attrappe: Meldung der Seite → Klick auf die Meldung; Download 2 → „Ordner
# öffnen“, Download 3 → „Öffnen“ (Beschriftungen je Sprache).
if [ "$SPR" = de ]; then AUF="Öffnen"; ORD="Ordner öffnen"; else AUF="Open"; ORD="Open Folder"; fi
python3 -I "$W/attrappe_benachrichtigungen.py" "$L/attrappe.log" \
  "probe-download 2.txt=$ORD" "probe-download 3.txt=$AUF" "probe-download=-" &
warte '"name": "erhalten"' "$L/attrappe.log" 10 || echo "FAIL attrappe" >>"$E"
# fremde Hauptseite ohne Geste (Taste u): ein Server auf 127.0.0.1 in der Hülle
mkdir -p "$QN_PROFIL/www" && echo '<title>fremd</title>fremd' > "$QN_PROFIL/www/fremd-ohne-geste"
(cd "$QN_PROFIL/www" && exec python3 -I -m http.server 8765 --bind 127.0.0.1) >>"$L/http.log" 2>&1 &

export QUERENCIA_SCHNATTER_DEBUG=1 QUERENCIA_SCHNATTER_PRUEFSEITE=$W/pruefseite.html
# Verschachteltes bwrap ist in der Hülle verboten (gemessen): WebKits eigene
# Sandbox nur hier aus; gemessen wird sie in Stufe E.
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1
"$BIN" >"$L/app.out" 2>"$L/app.log" &
APP=$!
. "$W/eingabe.sh"
# Konsole der Seite (PRUEF-Zeilen) kommt mit den Ereignissen auf stderr
A=$L/app.log; O=$A
pruefe start "E [0-9.]+ start dienst=pruefseite" "$A" 20
pruefe seite-geladen "teil1_fertig" "$O" 30
pruefe sicherer-kontext '"k":"secure_context","v":true' "$O" 1
pruefe benachrichtigung-vorab-erlaubt '"k":"notification_permission","v":"granted"' "$O" 1
pruefe permissions-notifications '"k":"permissions_notifications","v":"granted"' "$O" 1
pruefe permissions-mikrofon-prompt '"k":"permissions_microphone","v":"prompt"' "$O" 1
pruefe keine-webrtc '"k":"rtc_peer_connection","v":false' "$O" 1
pruefe meldung-an-attrappe '"summary": "Probe-Titel"' "$L/attrappe.log" 10
pruefe meldung-klick-onclick '"k":"notification_onclick","v":"probe-1"' "$O" 10
pruefe meldung-klick-hülle "notification-clicked" "$A" 1
pruefe download-2 "download-finished .*/Downloads/probe-download 2.txt" "$A" 10
pruefe download-3 "download-finished .*/Downloads/probe-download 3.txt" "$A" 10
pruefe download-meldung '"summary": "(Download finished|Download abgeschlossen)"' "$L/attrappe.log" 10
pruefe download-ordner-oeffnen "ordner (file://)?.*/heim/Downloads/?$" "$L/geoeffnet.log" 10
pruefe download-datei-oeffnen "datei (file://)?.*/heim/Downloads/probe-download( |%20)3.txt" "$L/geoeffnet.log" 10
if [ "$(cat "$QN_PROFIL/heim/Downloads/probe-download.txt")" = "schon da" ]; then echo "PASS download-ueberschreibt-nicht" >>"$E"; else echo "FAIL download-ueberschreibt-nicht" >>"$E"; fi
pruefe autoplay-audiocontext '"k":"audio_context","v":"running"' "$O" 1
pruefe autoplay-element '"k":"audio_element","v":"spielt"' "$O" 1
pruefe titel-3 'titel ungelesen=Some\(3\)' "$A" 5
pruefe titel-12 'titel ungelesen=Some\(12\)' "$A" 5
pruefe titel-0 'titel ungelesen=None fenster="Schnatter – (Prüfseite|test page)"' "$A" 5
pruefe rechtschreibung "rechtschreibung angefragt=.*angenommen=\[\"de_DE\", \"en_US\"\]|rechtschreibung angefragt=.*angenommen=\[\"en_US\"\]" "$A" 1
sleep 1; foto start
# 0.1.1 (QD-169): Grafik-Erkennung der Hülle gegen eglinfo -B, Bibliotheken der Prozesse
G=$(grep -m1 "querencia-schnatter: Grafik:" "$A")
echo "INFO grafik ${G#*Grafik: }" >>"$E"
pruefe grafik-ereignis "E [0-9.]+ grafik modus=automatisch grund=" "$A" 1
python3 -I "$W/karten.py" "$APP" >"$L/karten.txt" 2>&1
sed 's/^/INFO /' "$L/karten.txt" >>"$E"

# Mikrofon: Frage, Ablehnen (Vorgabeknopf), zweite Anfrage aus dem Speicher
q "key m"
pruefe mikrofon-frage "recht mikrofon frage" "$A" 5
sleep 0.5; foto frage-mikrofon
q "key Return" "wait 300" "key Escape"
pruefe mikrofon-abgelehnt '"k":"getusermedia_audio","v":"NotAllowedError"' "$O" 5
q "key m"
pruefe mikrofon-gespeichert "recht mikrofon gespeichert abgelehnt" "$A" 5
# Kamera: Frage, Erlauben (Tab, Eingabe); ohne Kamera in der Hülle meldet die Seite den Gerätefehler
q "key k"
pruefe kamera-frage "recht kamera frage" "$A" 5
q "key Tab" "key Return" "wait 300" "key Escape"
pruefe kamera-erlaubt "recht kamera beantwortet erlaubt" "$A" 5
warte '"k":"getusermedia_video"' "$O" 5; grep -o '"k":"getusermedia_video","v":"[^"]*"' "$O" | tail -1 | sed 's/^/INFO kamera-ergebnis /' >>"$E"
# Zwischenablage lesen (eigene Zwischenablage des Hüllen-labwc): Frage → Ablehnen
q "key z"
if warte "recht zwischenablage-lesen frage" "$A" 4; then
  foto frage-zwischenablage; q "key Return" "wait 300" "key Escape"
  pruefe zwischenablage-frage "recht zwischenablage-lesen beantwortet abgelehnt" "$A" 5
else
  echo "INFO zwischenablage-ohne-frage $(grep -o '"k":"zwischenablage","v":"[^"]*"' "$O" | tail -1)" >>"$E"
fi
q "key g"
pruefe standort-abgelehnt '"k":"standort","v":"abgelehnt 1"|recht standort -> abgelehnt' "$O" 5
# fremde Links mit Geste → Standardbrowser (testoeffner), Fenster bleibt
q "key f"
pruefe link-fremd "browser https://example.org/fremd\?x=1" "$L/geoeffnet.log" 5
q "key n"
pruefe link-neues-fenster "browser https://example.org/neu" "$L/geoeffnet.log" 5
q "key w"
pruefe window-open "browser https://example.org/fenster" "$L/geoeffnet.log" 5
q "key e"
pruefe link-eigen '"k":"hash","v":"#eigen"' "$O" 5
q "key p"
pruefe seite-noch-da '"k":"audio_zustand"' "$O" 5
# Dateiwahl: WebKits Standardweg (GTK-Dialog; in der Hülle ohne Portal)
q "key d"
pruefe dateiwahl-signal "run-file-chooser mehrfach=true" "$A" 5
sleep 1.5; foto dateiwahl
q "key Escape"
sleep 1
# Rechtschreibung: Textfeld, ein falsches Wort tippen
q "key t" "type  Feler" "wait 1500"
foto rechtschreibung
q "key Escape"
# Zoom (gemerkt)
q "chord ctrl plus" "chord ctrl plus" "wait 800"
foto zoom
# Die Einstellung erst abwarten: in en/1024×768/dunkel kamen die zwei Tasten
# in 0.3.0 bis zu 2,5 s später an (Hauptschleife nach dem Rechtschreib-Schritt
# belegt); die Wartezeit steht als INFO im Ergebnis
t0=$(date +%s%N)
if warte '^zoom=1\.25' "$QN_PROFIL/config/querencia-schnatter/einstellungen.ini" 5; then
  echo "PASS zoom-gemerkt" >>"$E"
else
  echo "FAIL zoom-gemerkt ($(grep zoom "$QN_PROFIL/config/querencia-schnatter/einstellungen.ini"))" >>"$E"
fi
echo "INFO zoom-gemerkt nach $(( ($(date +%s%N) - t0) / 1000000 )) ms (nach 0,8 s Wartezeit der Tasten)" >>"$E"
q "chord ctrl 0" "wait 500"
# Menüs
for m in "d datei" "b bearbeiten" "a ansicht" "h hilfe"; do
  [ "$SPR" = en ] && case ${m%% *} in d) m="f ${m#* }" ;; b) m="e ${m#* }" ;; a) m="v ${m#* }" ;; esac
  q "chord alt ${m%% *}" "wait 700"
  foto "menue-${m#* }"
  q "key Escape" "wait 300"
done
# Dialoge
aktion win info; sleep 1; foto info; q "key Escape" "wait 500"
q "key F1" "wait 1000"; foto hilfe; q "key Escape" "wait 500"
aktion win einstellungen; sleep 1; foto einstellungen; q "key Escape" "wait 500"
# Vollbild ohne Menüleiste
q "key F11" "wait 1000"; foto vollbild; q "key F11" "wait 800"
# fremde Hauptseite ohne Geste: Antwort wird verworfen, die Seite bleibt
q "key u"
pruefe hauptseite-fremd-verworfen "decide-policy Antwort \{ hauptseite: true \} http://127.0.0.1:8765/fremd-ohne-geste -> Verwerfen" "$A" 10
if grep -q "browser http://127.0.0.1" "$L/geoeffnet.log" 2>/dev/null; then echo "FAIL hauptseite-ohne-geste-oeffnet-browser" >>"$E"; else echo "PASS hauptseite-ohne-geste-kein-browser" >>"$E"; fi
q "key p"
sleep 1
[ "$(grep -c '"k":"audio_zustand"' "$O")" -ge 2 ] && echo "PASS seite-bleibt-nach-umleitung" >>"$E" || echo "FAIL seite-bleibt-nach-umleitung" >>"$E"

# 0.1.1 (QD-162 a): andere Schemata nur nach Rückfrage; Abbrechen ist Vorgabe
lebt() { # NAME: die Prüfseite antwortet noch (Taste p, eine Antwort mehr)
  local vor; vor=$(grep -c '"k":"audio_zustand"' "$O")
  q "key p"
  for _ in $(seq 30); do [ "$(grep -c '"k":"audio_zustand"' "$O")" -gt "$vor" ] && break; sleep 0.1; done
  [ "$(grep -c '"k":"audio_zustand"' "$O")" -gt "$vor" ] && echo "PASS $1" >>"$E" || echo "FAIL $1" >>"$E"
}
q "key s"
pruefe ssh-frage "fremdes-schema frage ssh:" "$A" 5
sleep 0.8; foto frage-fremdes-schema
q "key Escape"
pruefe ssh-abgebrochen "fremdes-schema ssh: abgebrochen" "$A" 5
q "key s"
warte "fremdes-schema frage ssh:.*" "$A" 5; sleep 0.5
q "key Return"
pruefe ssh-vorgabe-abbrechen "fremdes-schema ssh: abgebrochen" "$A" 3
[ "$(grep -c 'fremdes-schema ssh: abgebrochen' "$A")" = 2 ] && echo "PASS ssh-zweimal-abgebrochen" >>"$E" || echo "FAIL ssh-zweimal-abgebrochen" >>"$E"
sleep 0.5
if grep -q "ssh://" "$L/geoeffnet.log" 2>/dev/null; then echo "FAIL ssh-ohne-zustimmung-geoeffnet" >>"$E"; else echo "PASS ssh-ohne-zustimmung-nicht-geoeffnet" >>"$E"; fi
q "key s"
warte "fremdes-schema frage ssh:" "$A" 5; sleep 0.5
q "key Tab" "key Return"
pruefe ssh-geoeffnet "fremdes-schema ssh: geoeffnet" "$A" 5
pruefe ssh-an-programm "fremd ssh://nutzer@server.example/pfad\?x=1" "$L/geoeffnet.log" 5
q "key o"
pruefe smb-neues-fenster-frage "fremdes-schema frage smb:" "$A" 5
q "key Escape"
pruefe smb-abgebrochen "fremdes-schema smb: abgebrochen" "$A" 5
q "key j"
pruefe mailto-ohne-frage "browser mailto:jemand@example.org\?subject=Probe" "$L/geoeffnet.log" 5
grep -q "fremdes-schema frage mailto" "$A" && echo "FAIL mailto-fragte" >>"$E" || echo "PASS mailto-fragt-nicht" >>"$E"
lebt seite-bleibt-nach-schemata
# 0.1.1 (QD-162 b): die Hauptseite wechselt nie auf data:, file:, fremdes blob:, javascript:
q "key i"
pruefe data-klick-gesperrt "decide-policy Antwort \{ hauptseite: true \} data:text/html,%3Ctitle%3Edata-klick.* -> Sperren\(data:\)" "$A" 5
pruefe data-klick-protokoll "Wechsel auf data: abgelehnt" "$A" 1
lebt seite-bleibt-nach-data-klick
q "key l"
pruefe data-rahmen-erlaubt '"k":"rahmen_data","v":"geladen"' "$O" 5
q "key x"
warte '"k":"hauptseite_data"' "$O" 3; sleep 4.5
grep -E "decide-policy .*data:text/html,%3Ctitle%3Edata-weg" "$A" | sed 's/^.*decide-policy /INFO data-ohne-geste decide-policy /' >>"$E"
if grep -qE "decide-policy Antwort \{ hauptseite: true \} data:text/html,%3Ctitle%3Edata-weg.* -> Sperren\(data:\)" "$A"; then
  echo "PASS data-ohne-geste-an-der-antwort-gesperrt" >>"$E"
else
  grep -iE "top frame to data|data-weg" "$A" "$L/app.out" 2>/dev/null | head -3 | sed 's/^/INFO data-ohne-geste (keine Antwortprüfung) /' >>"$E"
fi
lebt seite-bleibt-nach-data-ohne-geste
# file: und fremdes blob: ohne Geste: WebKit selbst oder die Hülle lehnt ab
q "key q"
pruefe file-abgelehnt "decide-policy .* file:///etc/hostname -> Sperren\(file:\)|Not allowed to load local resource: file:///etc/hostname" "$A" 6
grep -E "decide-policy .* file:///etc/hostname|local resource: file:///etc/hostname" "$A" | sed 's/^/INFO file /' >>"$E"
lebt seite-bleibt-nach-file
q "key b"
pruefe blob-fremd-abgelehnt "decide-policy .* blob:http://127.0.0.1:8765/.* -> Sperren\(blob: mit fremdem Ursprung\)|Not allowed to load local resource: blob:http://127.0.0.1:8765/" "$A" 6
grep -E "decide-policy .* blob:http://127.0.0.1|local resource: blob:http://127.0.0.1" "$A" | sed 's/^/INFO blob /' >>"$E"
lebt seite-bleibt-nach-blob
grep -c "Wechsel auf .* abgelehnt" "$A" | sed 's/^/INFO ablehnungen-im-protokoll /' >>"$E"
sleep 0.5; foto nach-haertung
# javascript: ohne Geste: WebKit 2.52 fragt nicht (keine Entscheidung), die
# Adresse läuft als Skript im eigenen Ursprung; gemessen, nicht bewertet
q "key y"
warte '"k":"hauptseite_javascript"' "$O" 3; sleep 4.5
grep -c "decide-policy .*javascript:" "$A" | sed 's/^/INFO javascript-entscheidungen /' >>"$E"
echo "INFO javascript-nach-3s uri=$(grep 'load-changed' "$A" | tail -1 | awk '{print $NF}') seite-antwortet=$(q "key p"; sleep 1; tail -1 "$O" | grep -c audio_zustand)" >>"$E"
sleep 0.5; foto nach-javascript
# Absturz des Web-Prozesses → Hinweis und Neuladen
WP=$(webprozesse | tr '\n' ' ')
echo "INFO webprozesse $WP" >>"$E"
for p in $WP; do kill -KILL "$p"; done
pruefe absturz-erkannt "web-process-terminated Crashed" "$A" 10
pruefe absturz-neu-geladen "neu-laden nach Absturz 1" "$A" 5
warte "load-changed Finished" "$A" 15; sleep 2; foto absturz-hinweis
# Abmelden und Daten löschen (Rückfrage → zweiter Knopf)
D=$QN_PROFIL/data/querencia-schnatter
echo "INFO daten-vorher $(find "$D" -type f | wc -l) Dateien, $(du -sk "$D" | cut -f1) KiB" >>"$E"
aktion win abmelden; sleep 1; foto frage-abmelden
q "key Tab" "key Return"
pruefe abmelden-geloescht "abmelden daten gelöscht" "$A" 15
warte "teil1_fertig" "$O" 20
echo "INFO daten-nach-abmelden $(find "$D" -type f | wc -l) Dateien, $(du -sk "$D" | cut -f1) KiB" >>"$E"
for o in data cache config; do
  m=$(stat -c %a "$QN_PROFIL/$o/querencia-schnatter")
  [ "$m" = 700 ] && echo "PASS ordner-$o-0700" >>"$E" || echo "FAIL ordner-$o-0700 ($m)" >>"$E"
done
# Beenden über das Menü (Strg+Q) merkt die Fenstergröße; Neustart leert die vorgemerkten Ordner
q "chord ctrl q"
for _ in $(seq 50); do kill -0 "$APP" 2>/dev/null || break; sleep 0.2; done
kill -0 "$APP" 2>/dev/null && echo "FAIL beenden" >>"$E" || echo "PASS beenden" >>"$E"
pruefe fenstergroesse-gemerkt "fenster-gemerkt $FENSTER maximiert=false" "$A" 1
[ -e "$QN_PROFIL/config/querencia-schnatter/daten-loeschen" ] && echo "PASS loeschen-vorgemerkt" >>"$E" || echo "FAIL loeschen-vorgemerkt" >>"$E"
# Ton aus (wie über die Einstellungen gespeichert) gilt beim nächsten Start
printf '\n[Seite]\nton=false\n' >> "$QN_PROFIL/config/querencia-schnatter/einstellungen.ini"
"$BIN" >"$L/app2.out" 2>"$L/app2.log" &
APP=$!
pruefe neustart-loescht "vorgemerkte Löschung der Sitzungsdaten ausgeführt" "$L/app2.log" 15
warte "teil1_fertig" "$L/app2.log" 30
pruefe neustart-ton-aus "kennung standard .* ton=false" "$L/app2.log" 1
sleep 1; foto neustart
pruefe neustart-rechte-vergessen '"k":"permissions_camera","v":"prompt"' "$L/app2.log" 5
aktion app beenden
for _ in $(seq 50); do kill -0 "$APP" 2>/dev/null || break; sleep 0.2; done
cp "$L"/*.log "$L"/*.out "$OUT/" 2>/dev/null
cp "$QN_PROFIL/config/querencia-schnatter/einstellungen.ini" "$OUT/" 2>/dev/null
echo "== $(grep -c '^PASS' "$E") PASS, $(grep -c '^FAIL' "$E") FAIL" >>"$E"
cat "$E"
! grep -q '^FAIL' "$E"
