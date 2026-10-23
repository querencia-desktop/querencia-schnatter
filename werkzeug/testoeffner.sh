#!/bin/bash
# Prüfhülle: steht im Profil an Stelle von Browser, Dateimanager und
# Texteditor (profil.sh) und schreibt nur auf, was geöffnet werden sollte.
echo "$(date +%T.%N | cut -c1-12) $*" >> "${QN_PROFIL:?}/logs/geoeffnet.log"
