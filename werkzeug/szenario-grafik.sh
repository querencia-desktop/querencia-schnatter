#!/bin/bash
# Prüfhülle Schnatter, Stufe L: Grafik-Rückfall (0.1.1, QD-169).
# Läuft IN der Hülle (lauf.sh grafik). Misst die Erkennung auf zwei Wegen
# (eigene Probe „--grafik-probe“ gegen eglinfo -B, surfaceless) und startet
# die Hülle gegen die Prüfseite in vier Fällen: Einstellung automatisch, an,
# aus und „Umgebung“ (die drei Variablen schon gesetzt, wie von einer
# Sitzung), dazu „vergleich“ mit einer Vergleichs-Binary (0.1.0, lauf.sh
# VERGLEICH_BIN). Je Fall: Protokollzeile, Ereignis, Bibliotheken und
# Grafik-Variablen der Hülle und der Web-Prozesse (/proc/<pid>/maps und
# environ, karten.py), Bildbetrachter der Prüfseite (Taste v: Dekodier- und
# Zeichenzeit, Foto, kein Absturz), Renderer im Info-Dialog (Foto).
# Ergebnis: $OUT/ergebnis.txt.
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
pruefe() { # NAME MUSTER DATEI ZEIT
  if warte "$2" "$3" "$4"; then echo "PASS $1" >>"$E"; else echo "FAIL $1 (fehlt: $2 in $(basename "$3"))" >>"$E"; fi
}
ja() { if "$@"; then echo PASS; else echo FAIL; fi; }
nicht() { ! "$@"; }
aktion() { # win|app NAME
  local pfad=/org/querencia/Schnatter
  [ "$1" = win ] && pfad=$pfad/window/1
  gdbus call --session --dest org.querencia.Schnatter --object-path "$pfad" \
    --method org.gtk.Actions.Activate "$2" '[]' '{}' >>"$L/szenario.log" 2>&1
}
renderer() { sed -n 's/^renderer=//p; s/^ *OpenGL ES profile renderer: //p' | head -1; }

# 1. Zwei Wege der Erkennung, ohne und mit den drei Variablen
eglinfo -B -p surfaceless >"$L/eglinfo.txt" 2>&1
"$BIN" --grafik-probe >"$L/probe.txt" 2>>"$L/probe.err"; rc=$?
R_EGL=$(renderer <"$L/eglinfo.txt"); R_PROBE=$(renderer <"$L/probe.txt")
echo "INFO eglinfo-surfaceless $R_EGL" >>"$E"
echo "INFO probe rc=$rc $R_PROBE" >>"$E"
echo "$(ja [ -n "$R_PROBE" ] && [ "$R_PROBE" = "$R_EGL" ]) probe-gleich-eglinfo" >>"$E"
SW=(LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe GALLIUM_DRIVER=llvmpipe)
env "${SW[@]}" eglinfo -B -p surfaceless >"$L/eglinfo-sw.txt" 2>&1
env "${SW[@]}" "$BIN" --grafik-probe >"$L/probe-sw.txt" 2>>"$L/probe.err"
R_EGL_SW=$(renderer <"$L/eglinfo-sw.txt"); R_PROBE_SW=$(renderer <"$L/probe-sw.txt")
echo "INFO eglinfo-surfaceless-software $R_EGL_SW" >>"$E"
echo "INFO probe-software $R_PROBE_SW" >>"$E"
echo "$(ja [ -n "$R_PROBE_SW" ] && [ "$R_PROBE_SW" = "$R_EGL_SW" ]) probe-gleich-eglinfo-software" >>"$E"
for i in 1 2 3; do
  s=${EPOCHREALTIME/,/.}; "$BIN" --grafik-probe >/dev/null 2>&1
  awk -v a="$s" -v b="${EPOCHREALTIME/,/.}" 'BEGIN { printf "INFO probe-dauer %.0f ms\n", (b - a) * 1000 }' >>"$E"
done
case $R_PROBE in *zink*NVK*|*zink*nouveau*) ERKANNT=1 ;; *) ERKANNT=0 ;; esac
echo "INFO zink-auf-nouveau-hier $ERKANNT" >>"$E"

python3 -I "$W/attrappe_benachrichtigungen.py" "$L/attrappe.log" "=-" &
export QUERENCIA_SCHNATTER_DEBUG=1 QUERENCIA_SCHNATTER_PRUEFSEITE=$W/pruefseite.html
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1
INI=$QN_PROFIL/config/querencia-schnatter/einstellungen.ini
. "$W/eingabe.sh"

# 2. Die Hülle in vier Fällen (und mit der Vergleichs-Binary)
FAELLE=${GRAFIK_FAELLE:-"automatisch an aus umgebung"}
VB=$QN_PROFIL/opt/vergleich/querencia-schnatter
[ -x "$VB" ] && FAELLE="vergleich $FAELLE"
for fall in $FAELLE; do
  sed -i '/^\[Grafik\]/,$d' "$INI"
  modus=$fall; [ "$fall" = umgebung ] && modus=automatisch
  printf '[Grafik]\nsoftware=%s\n' "$modus" >>"$INI"
  A=$L/app-$fall.log
  if [ "$fall" = umgebung ]; then env "${SW[@]}" "$BIN" >"$L/app-$fall.out" 2>"$A" &
  elif [ "$fall" = vergleich ]; then "$VB" >"$L/app-$fall.out" 2>"$A" &
  else "$BIN" >"$L/app-$fall.out" 2>"$A" &
  fi
  APP=$!
  pruefe "$fall-seite-geladen" "teil1_fertig" "$A" 40
  sleep 2
  # Bildbetrachter (wie bei WhatsApp Web): dreimal öffnen und schließen, Zeiten je Öffnen
  for i in 1 2 3; do
    q "key v"
    for _ in $(seq 200); do [ "$(grep -c '"k":"bildbetrachter","v"' "$A")" -ge "$i" ] && break; sleep 0.1; done
    [ "$i" = 1 ] && { sleep 0.3; foto "bildbetrachter-$fall"; }
    q "key Escape" "wait 300"
  done
  echo "$(ja [ "$(grep -c '"k":"bildbetrachter","v"' "$A")" = 3 ]) $fall-bildbetrachter-dreimal-offen" >>"$E"
  grep -o '"k":"bildbetrachter","v":{[^}]*}' "$A" | sed "s/^.*\"v\":/INFO $fall bildbetrachter /" >>"$E"
  sleep 1
  grep -q "web-process-terminated" "$A" && echo "FAIL $fall-bildbetrachter-ohne-absturz" >>"$E" || echo "PASS $fall-bildbetrachter-ohne-absturz" >>"$E"
  G=$(grep -m1 "querencia-schnatter: Grafik:" "$A")
  echo "INFO $fall ${G#*querencia-schnatter: }" >>"$E"
  python3 -I "$W/karten.py" "$APP" >"$L/karten-$fall.txt" 2>&1
  sed "s/^/INFO $fall /" "$L/karten-$fall.txt" >>"$E"
  WEB=$(grep '^karten WebKitWebProces' "$L/karten-$fall.txt" | grep -v 'nicht lesbar')
  UMG=$(grep '^karten WebKitWebProces' "$L/karten-$fall.txt" | sed -n 's/.* umgebung=//p' | sort -u)
  echo "INFO $fall webprozess-umgebung ${UMG//$'\n'/ | }" >>"$E"
  case $fall in
    vergleich) SOFT=0 ;;
    automatisch)
      if [ "$ERKANNT" = 1 ]; then
        pruefe automatisch-erkannt "grafik modus=automatisch grund=Erkannt gesetzt=\[\"LIBGL_ALWAYS_SOFTWARE\", \"MESA_LOADER_DRIVER_OVERRIDE\", \"GALLIUM_DRIVER\"\]" "$A" 1
        SOFT=1
      else
        pruefe automatisch-hardware "grafik modus=automatisch grund=Hardware gesetzt=\[\]" "$A" 1
        SOFT=0
      fi ;;
    an) pruefe an-einstellung "grafik modus=an grund=Einstellung gesetzt=\[\"LIBGL_ALWAYS_SOFTWARE\", \"MESA_LOADER_DRIVER_OVERRIDE\", \"GALLIUM_DRIVER\"\]" "$A" 1; SOFT=1 ;;
    aus) pruefe aus-keine-probe "grafik modus=aus grund=Aus gesetzt=\[\]" "$A" 1; SOFT=0 ;;
    umgebung) pruefe umgebung-keine-probe "grafik modus=automatisch grund=Umgebung gesetzt=\[\]" "$A" 1; SOFT=1 ;;
  esac
  ALLE3="GALLIUM_DRIVER=llvmpipe,LIBGL_ALWAYS_SOFTWARE=1,MESA_LOADER_DRIVER_OVERRIDE=llvmpipe"
  if [ -z "$WEB" ]; then echo "FAIL $fall-webprozess-karten (keiner lesbar)" >>"$E"
  elif [ "$SOFT" = 1 ]; then
    echo "$(ja grep -q 'gallium=ja' <<<"$WEB") $fall-webprozess-libgallium" >>"$E"
    echo "$(ja nicht grep -q 'vulkan_nouveau=ja' <<<"$WEB") $fall-webprozess-ohne-libvulkan_nouveau" >>"$E"
    echo "$(ja [ "$UMG" = "$ALLE3" ]) $fall-webprozess-environ-drei-variablen" >>"$E"
  else
    [ "$ERKANNT" = 1 ] && echo "$(ja grep -q 'vulkan_nouveau=ja' <<<"$WEB") $fall-webprozess-zink-auf-nvk" >>"$E"
    echo "$(ja [ "$UMG" = "-" ]) $fall-webprozess-environ-ohne-variablen" >>"$E"
  fi
  if [ "$fall" = vergleich ]; then
    aktion app beenden
    for _ in $(seq 50); do kill -0 "$APP" 2>/dev/null || break; sleep 0.2; done
    kill -0 "$APP" 2>/dev/null && echo "FAIL $fall-beenden" >>"$E" || echo "PASS $fall-beenden" >>"$E"
    continue
  fi
  # Info-Dialog: Renderer, wie ihn die Web-Prozesse sehen
  aktion win info
  pruefe "$fall-info-renderer" "E [0-9.]+ grafik-info Gelesen" "$A" 10
  R_INFO=$(grep -m1 'grafik-info Gelesen' "$A" | sed 's/.*renderer: "\([^"]*\)".*/\1/')
  echo "INFO $fall info-renderer $R_INFO" >>"$E"
  if [ "$SOFT" = 1 ]; then echo "$(ja grep -q llvmpipe <<<"$R_INFO") $fall-info-llvmpipe" >>"$E"; fi
  sleep 0.8; foto "info-$fall"
  q "key Escape" "wait 300"
  if [ "$fall" = automatisch ]; then
    aktion win einstellungen; sleep 1; foto einstellungen-grafik; q "key Escape" "wait 300"
  fi
  aktion app beenden
  for _ in $(seq 50); do kill -0 "$APP" 2>/dev/null || break; sleep 0.2; done
  kill -0 "$APP" 2>/dev/null && echo "FAIL $fall-beenden" >>"$E" || echo "PASS $fall-beenden" >>"$E"
  grep -q "web-process-terminated" "$A" && echo "FAIL $fall-kein-absturz" >>"$E" || echo "PASS $fall-kein-absturz" >>"$E"
done
cp "$L"/*.log "$L"/*.txt "$OUT/" 2>/dev/null
echo "== $(grep -c '^PASS' "$E") PASS, $(grep -c '^FAIL' "$E") FAIL" >>"$E"
cat "$E"
! grep -q '^FAIL' "$E"
