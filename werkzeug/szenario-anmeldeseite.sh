#!/bin/bash
# Prüfhülle Querencia Nachrichten: Anmeldeseiten-Wächter (Muster: probe.py der
# Machbarkeit). Lädt web.whatsapp.com bis zum QR-Code, macht ein Foto des
# ganzen Bildschirms, misst Speicher (PSS) und Sandbox und beendet die Hülle.
# Meldet sich NIE an: niemand scannt, die dauerhafte Sitzung liegt im
# Wegwerfprofil und wird mit ihm gelöscht. Läuft IN der Hülle (lauf.sh
# anmeldeseite), Stufe E (echte WebKit-Sandbox) oder N.
set -u
. "$QN_PROFIL/env.sh"
QA=$W/bin/qausgabe
L=$QN_PROFIL/logs
E=$OUT/ergebnis.txt
: > "$E"
"$QA" "$RES" >>"$L/szenario.log" 2>&1
warte() { local i; for i in $(seq 1 $(($3 * 10))); do grep -qE "$1" "$2" 2>/dev/null && return 0; sleep 0.1; done; return 1; }
# Keine Meldung erwartet; falls doch, nur aufschreiben (nicht klicken).
python3 -I "$W/attrappe_benachrichtigungen.py" "$L/attrappe.log" "=-" &
export QUERENCIA_NACHRICHTEN_DEBUG=1 QUERENCIA_NACHRICHTEN_ZUSTAND=1
if [ "$STUFE" = N ] && [ "${SANDBOX:-aus}" = aus ]; then
  export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1
  echo "INFO webkit-sandbox aus (Stufe N: verschachteltes bwrap verboten)" >>"$E"
else
  echo "INFO webkit-sandbox an" >>"$E"
fi
"$BIN" >"$L/app.out" 2>"$L/app.log" &
APP=$!
A=$L/app.log
if warte 'zustand .*"qr":true' "$A" 90; then
  t=$(grep -m1 'zustand .*"qr":true' "$A" | awk '{print $3}')
  echo "PASS qr-code nach ${t} s (Zustandsabfrage alle 2 s)" >>"$E"
  sleep 4
else
  echo "FAIL qr-code (kein QR in 90 s)" >>"$E"
fi
grim "$(printf '%s/anmeldeseite-%s-%s-%s-%s.png' "$OUT" "$KENNUNG" "$SPR" "$RES" "$SCHEMA")"
python3 -I "$W/messen.py" "$APP" "$OUT/messung.json" > "$OUT/messung.txt" 2>&1
# 0.1.1 (QD-169): Grafik-Entscheidung und Bibliotheken der Prozesse (maps)
grep -m1 "querencia-nachrichten: Grafik:" "$A" | sed 's/^.*Grafik: /INFO grafik /' >>"$E"
python3 -I "$W/karten.py" "$APP" > "$OUT/karten.txt" 2>&1
sed 's/^/INFO /' "$OUT/karten.txt" >>"$E"
grep 'E [0-9.]* zustand ' "$A" | tail -1 | sed 's/^.* zustand /INFO zustand /' >>"$E"
grep -m1 'E [0-9.]* kennung ' "$A" | sed 's/^.* kennung /INFO kennung /' >>"$E"
grep 'web-process-terminated\|load-failed' "$A" | sed 's/^/INFO /' >>"$E"
tail -1 "$OUT/messung.txt" | sed 's/^/INFO /' >>"$E"
gdbus call --session --dest org.querencia.Nachrichten --object-path /org/querencia/Nachrichten \
  --method org.gtk.Actions.Activate beenden '[]' '{}' >>"$L/szenario.log" 2>&1
for _ in $(seq 50); do kill -0 "$APP" 2>/dev/null || break; sleep 0.2; done
kill -0 "$APP" 2>/dev/null && echo "FAIL beenden" >>"$E" || echo "PASS beenden" >>"$E"
# R9: die Sitzung des Wächters (ohne Konto, mit abgelaufenem QR-Code) nie behalten
rm -rf -- "$QN_PROFIL/data/querencia-nachrichten" "$QN_PROFIL/cache/querencia-nachrichten"
[ -e "$QN_PROFIL/data/querencia-nachrichten" ] && echo "FAIL sitzung-geloescht" >>"$E" || echo "PASS sitzung-geloescht" >>"$E"
cp "$L"/*.log "$OUT/" 2>/dev/null
cat "$E"
! grep -q '^FAIL' "$E"
