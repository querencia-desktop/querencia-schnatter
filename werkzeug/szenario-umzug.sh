#!/bin/bash
# Prüfhülle Schnatter, Stufe L: Umzug der Ordner eines früheren Programms
# auf 0.3.0 (querencia-schnatter): Querencia Webhülle 0.2.0
# (querencia-webhuelle) oder Querencia Nachrichten 0.1.1
# (querencia-nachrichten), je nach ALT_ABLAGE in lauf.sh; nur mit einer
# Attrappen-Anmeldung im Wegwerfprofil (nie mit echten Daten).
# Läuft IN der Hülle (lauf.sh umzug …). Ablauf („alt“ = 0.2.0 bzw. 0.1.1):
#   1 alt legt die Attrappe an (Cookie, localStorage, IndexedDB; Taste a)
#     und merkt Zoom 1,25 – alles unter dem alten Namen
#   2 alt läuft, 0.3.0 startet: kein Umzug, Hinweisfenster, nichts Neues
#   3 0.3.0 allein: Umzug (rename, gleiche Inodes), Attrappe und Zoom da
#   4 zweiter Start: nichts mehr zu tun
#   5 Rückfall wie integration/schnatter-030 `zurueck` (mv neu → alt):
#     alt findet die Attrappe; wieder vor (mv alt → neu): 0.3.0 auch
#   6 alter Ordner taucht wieder auf, neuer besteht: der alte bleibt liegen
# Ergebnis: $OUT/ergebnis.txt (eine Zeile je Prüfpunkt), Fotos.
set -u
. "$QN_PROFIL/env.sh"
Q=$W/bin/qeingabe; QA=$W/bin/qausgabe
L=$QN_PROFIL/logs
E=$OUT/ergebnis.txt
: > "$E"
"$QA" "$RES" >>"$L/szenario.log" 2>&1
n=0
foto() { n=$((n + 1)); grim "$(printf '%s/%02d-%s-%s-%s-%s.png' "$OUT" "$n" "$1" "$SPR" "$RES" "$SCHEMA")"; }
warte() { local i; for i in $(seq 1 $(($3 * 10))); do grep -qE "$1" "$2" 2>/dev/null && return 0; sleep 0.1; done; return 1; }
wartef() { local i; for i in $(seq 1 $(($3 * 10))); do grep -qF "$1" "$2" 2>/dev/null && return 0; sleep 0.1; done; return 1; }
pruefe() { if warte "$2" "$3" "$4"; then echo "PASS $1" >>"$E"; else echo "FAIL $1 (fehlt: $2 in $(basename "$3"))" >>"$E"; fi; }
pruefef() { if wartef "$2" "$3" "$4"; then echo "PASS $1" >>"$E"; else echo "FAIL $1 (fehlt: $2 in $(basename "$3"))" >>"$E"; fi; }
ja() { if "${@:2}"; then echo "PASS $1" >>"$E"; else echo "FAIL $1" >>"$E"; fi; }
aktion() { # alt|neu|neu-fenster NAME
  local ziel=org.querencia.Schnatter pfad=/org/querencia/Schnatter
  [ "$1" = alt ] && { ziel=org.querencia.$ALT_ID; pfad=/org/querencia/$ALT_ID; }
  [ "$1" = neu-fenster ] && pfad=$pfad/window/1
  gdbus call --session --timeout 10 --dest "$ziel" --object-path "$pfad" \
    --method org.gtk.Actions.Activate "$2" '[]' '{}' >>"$L/szenario.log" 2>&1
}
ende() { # PID: wartet höchstens 10 s auf das Ende
  local i; for i in $(seq 50); do kill -0 "$1" 2>/dev/null || return 0; sleep 0.2; done; return 1
}
# Welches frühere Programm (lauf.sh legt genau eines nach opt/alt/)
if [ -x "$QN_PROFIL/opt/alt/querencia-webhuelle" ]; then
  ALT_NAME=querencia-webhuelle ALT_ID=Webhuelle ALT_VERSION=0.2.0 ALT_VAR=QUERENCIA_WEBHUELLE
else
  ALT_NAME=querencia-nachrichten ALT_ID=Nachrichten ALT_VERSION=0.1.1 ALT_VAR=QUERENCIA_NACHRICHTEN
fi
ALT_KURZ=${ALT_NAME#querencia-}
ALT=$(ls "$QN_PROFIL/opt/alt/$ALT_NAME")
echo "INFO alt $ALT_NAME $ALT_VERSION" >>"$E"
NAMEN="data cache config"
alt_d() { echo "$QN_PROFIL/$1/$ALT_NAME"; }
neu_d() { echo "$QN_PROFIL/$1/querencia-schnatter"; }
gibt() { [ -e "$1" ] || [ -L "$1" ]; }

export "${ALT_VAR}_DEBUG=1" "${ALT_VAR}_PRUEFSEITE=$W/pruefseite.html"
export QUERENCIA_SCHNATTER_DEBUG=1 QUERENCIA_SCHNATTER_PRUEFSEITE=$W/pruefseite.html
# Verschachteltes bwrap ist in der Hülle verboten: WebKits Sandbox nur hier aus
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1
. "$W/eingabe.sh"

# ---- 1: alt legt die Attrappen-Anmeldung an
for o in $NAMEN; do gibt "$(neu_d "$o")" && echo "FAIL ausgang-ohne-neue-ordner ($o)" >>"$E"; done
"$ALT" >"$L/alt1.out" 2>"$L/alt1.log" &
APP=$!
A=$L/alt1.log
pruefe alt-start "E [0-9.]+ start dienst=pruefseite version=$ALT_VERSION" "$A" 20
pruefe alt-seite "teil1_fertig" "$A" 30
pruefef alt-speicher-leer '"k":"speicher","v":{"cookie":null,"local":null,"idb":null}' "$A" 1
q "key a"
warte '"k":"speicher_angelegt"' "$A" 5
T=$(grep -o '"k":"speicher_angelegt","v":"[0-9a-f]*"' "$A" | tail -1 | sed 's/.*"v":"//; s/"$//')
echo "INFO attrappe $T" >>"$E"
pruefef alt-speicher-angelegt "\"k\":\"speicher\",\"v\":{\"cookie\":\"$T\",\"local\":\"$T\",\"idb\":\"$T\"}" "$A" 5
# Was einen Neustart überdauern muss: localStorage und IndexedDB (dort hält
# WhatsApp Web seine Sitzung). Cookies hält WebKit ohne dauerhaften
# Cookie-Speicher nur bis zum Ende des Programms – schon seit 0.1.1 (gemessen,
# Zeile „INFO cookie“ je Start); das ist keine Folge des Umzugs.
SOLL="\"local\":\"$T\",\"idb\":\"$T\"}"
cookie() { echo "INFO cookie $1 $(grep -o '"k":"speicher","v":{"cookie":[^,]*' "$2" | head -1 | sed 's/.*"cookie"://')" >>"$E"; }
q "chord ctrl plus" "chord ctrl plus" "wait 800"
sleep 0.5; foto alt-$ALT_KURZ-attrappe
aktion alt beenden
ja alt-beendet ende "$APP"
for o in $NAMEN; do ja "alt-ordner-$o" test -d "$(alt_d "$o")"; done
ja alt-zoom-gemerkt grep -q '^zoom=1.25' "$(alt_d config)/einstellungen.ini"
# Marken (Zufallsinhalt) und Inodes der alten Ordner
for o in $NAMEN; do head -c 4096 /dev/urandom >"$(alt_d "$o")/attrappe-marke.bin"; done
: >"$L/vorher.txt"
for o in $NAMEN; do
  echo "$o inode=$(stat -c %i "$(alt_d "$o")") marke=$(sha256sum <"$(alt_d "$o")/attrappe-marke.bin" | cut -c1-64) dateien=$(find "$(alt_d "$o")" -type f | wc -l)" >>"$L/vorher.txt"
done
(cd "$QN_PROFIL" && for o in $NAMEN; do find "$o/$ALT_NAME" -type f -printf '%P\n'; done | LC_ALL=C sort) >"$L/namen-vorher.txt"
sed 's/^/INFO vorher /' "$L/vorher.txt" >>"$E"

# ---- 2: alt läuft noch → 0.3.0 zieht nicht um und startet nicht
"$ALT" >"$L/alt2.out" 2>"$L/alt2.log" &
APP=$!
warte "teil1_fertig" "$L/alt2.log" 30
pruefef alt2-attrappe-ohne-umzug "$SOLL" "$L/alt2.log" 5
cookie alt2-ohne-umzug "$L/alt2.log"
"$BIN" >"$L/neu-block.out" 2>"$L/neu-block.log" &
NEU=$!
B=$L/neu-block.log
pruefe block-ereignis "E [0-9.]+ umzug blockiert name=$ALT_NAME pid=$APP" "$B" 15
pruefe block-protokoll "Umzug steht an, aber $ALT_NAME läuft noch \(PID $APP\)" "$B" 1
sleep 1.5; foto umzug-blockiert
for o in $NAMEN; do
  ja "block-alt-$o-bleibt" test -d "$(alt_d "$o")"
  ja "block-kein-neuer-$o" bash -c "! [ -e '$(neu_d "$o")' ]"
done
grep -q "start dienst=" "$B" && echo "FAIL block-startete-trotzdem" >>"$E" || echo "PASS block-kein-start" >>"$E"
aktion neu beenden
ja block-beendet ende "$NEU"
aktion alt beenden
ja alt2-beendet ende "$APP"

# ---- 3: 0.3.0 allein: Umzug
"$BIN" >"$L/neu1.out" 2>"$L/neu1.log" &
APP=$!
A=$L/neu1.log
pruefe umzug-ereignis "E [0-9.]+ umzug daten=verschoben-$ALT_KURZ cache=verschoben-$ALT_KURZ konfig=verschoben-$ALT_KURZ" "$A" 15
for o in data cache config; do
  pruefe "umzug-protokoll-$o" "Umzug: .*/$o/$ALT_NAME nach .*/$o/querencia-schnatter umbenannt" "$A" 1
done
pruefe neu-start "E [0-9.]+ start dienst=pruefseite version=0.3.0" "$A" 20
pruefe neu-seite "teil1_fertig" "$A" 30
pruefef umzug-attrappe-ueberlebt "$SOLL" "$A" 5
cookie neu1 "$A"
ja umzug-zoom-uebernommen grep -q '^zoom=1.25' "$(neu_d config)/einstellungen.ini"
for o in $NAMEN; do
  ja "umzug-alt-$o-weg" bash -c "! [ -e '$(alt_d "$o")' ]"
  ja "umzug-neu-$o-0700" test "$(stat -c %a "$(neu_d "$o")")" = 700
  ja "umzug-$o-gleicher-inode" grep -q "^$o inode=$(stat -c %i "$(neu_d "$o")") " "$L/vorher.txt"
  ja "umzug-$o-marke-gleich" grep -q "^$o .* marke=$(sha256sum <"$(neu_d "$o")/attrappe-marke.bin" | cut -c1-64) " "$L/vorher.txt"
done
(cd "$QN_PROFIL" && for o in $NAMEN; do find "$o/querencia-schnatter" -type f -printf '%P\n'; done | LC_ALL=C sort) >"$L/namen-nachher.txt"
fehlend=$(LC_ALL=C comm -23 "$L/namen-vorher.txt" "$L/namen-nachher.txt" | wc -l)
echo "INFO dateien vorher $(wc -l <"$L/namen-vorher.txt"), nachher $(wc -l <"$L/namen-nachher.txt"), verschwunden $fehlend" >>"$E"
ja umzug-keine-datei-verloren test "$fehlend" = 0
sleep 1; foto neu-030-nach-umzug
aktion neu-fenster info; sleep 1.2; foto info-030; q "key Escape" "wait 500"
aktion neu beenden
ja neu1-beendet ende "$APP"

# ---- 4: zweiter Start
"$BIN" >"$L/neu2.out" 2>"$L/neu2.log" &
APP=$!
pruefe zweiter-start-nichts "E [0-9.]+ umzug daten=nichts cache=nichts konfig=nichts" "$L/neu2.log" 15
pruefef zweiter-start-attrappe "$SOLL" "$L/neu2.log" 30
cookie neu2 "$L/neu2.log"
aktion neu beenden
ja neu2-beendet ende "$APP"

# ---- 5: Rückfall (wie `zurueck`: mv neu → alt), alt; wieder vor, 0.3.0
for o in $NAMEN; do mv -T -- "$(neu_d "$o")" "$(alt_d "$o")"; done
"$ALT" >"$L/alt3.out" 2>"$L/alt3.log" &
APP=$!
pruefef rueckfall-alt-attrappe "$SOLL" "$L/alt3.log" 30
cookie alt3 "$L/alt3.log"
ja rueckfall-alt-zoom grep -q '^zoom=1.25' "$(alt_d config)/einstellungen.ini"
sleep 1; foto rueckfall-$ALT_KURZ
aktion alt beenden
ja alt3-beendet ende "$APP"
for o in $NAMEN; do mv -T -- "$(alt_d "$o")" "$(neu_d "$o")"; done
"$BIN" >"$L/neu3.out" 2>"$L/neu3.log" &
APP=$!
pruefe wieder-vor-nichts "E [0-9.]+ umzug daten=nichts cache=nichts konfig=nichts" "$L/neu3.log" 15
pruefef wieder-vor-attrappe "$SOLL" "$L/neu3.log" 30
aktion neu beenden
ja neu3-beendet ende "$APP"

# ---- 6: ein alter Ordner taucht wieder auf (z. B. alt nach dem Rückfall
# ohne Rück-Umzug gestartet): er bleibt liegen, der neue gilt
mkdir -m 700 "$(alt_d data)" && echo "alt" >"$(alt_d data)/spaeter.txt"
"$BIN" >"$L/neu4.out" 2>"$L/neu4.log" &
APP=$!
pruefe beide-alt-bleibt "E [0-9.]+ umzug daten=alt-bleibt cache=nichts konfig=nichts" "$L/neu4.log" 15
pruefe beide-protokoll "Umzug: .*/data/$ALT_NAME bleibt liegen" "$L/neu4.log" 1
pruefef beide-attrappe "$SOLL" "$L/neu4.log" 30
aktion neu beenden
ja neu4-beendet ende "$APP"
ja beide-alter-inhalt-unberuehrt grep -qx alt "$(alt_d data)/spaeter.txt"

# Reste: kein Prozess eines der Programme
r=0; for p in /proc/[0-9]*; do case $(readlink "$p/exe" 2>/dev/null) in */querencia-nachrichten|*/querencia-webhuelle|*/querencia-schnatter) r=$((r + 1)) ;; esac; done
ja keine-programme-uebrig test "$r" = 0
cp "$L"/*.log "$L"/*.txt "$OUT/" 2>/dev/null
echo "== $(grep -c '^PASS' "$E") PASS, $(grep -c '^FAIL' "$E") FAIL" >>"$E"
cat "$E"
! grep -q '^FAIL' "$E"
