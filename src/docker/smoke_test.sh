#!/usr/bin/env bash
# Test de fumee headless : lance l'exe Windows sous Wine + Xvfb (pas
# d'affichage reel necessaire), attend quelques secondes, puis verifie que
# le processus est toujours vivant (= n'a pas plante au demarrage).
set -u

EXE="/workspace/target/x86_64-pc-windows-gnu/release/hus-clean.exe"
export WINEPREFIX=/tmp/wineprefix
export WINEDEBUG=-all
export DISPLAY=:99

Xvfb :99 -screen 0 1280x1024x24 >/tmp/xvfb.log 2>&1 &
XVFB_PID=$!
sleep 2

/usr/lib/wine/wine64 "$EXE" >/tmp/wine_stdout.log 2>/tmp/wine_stderr.log &
WINE_PID=$!

sleep 6

import -display :99 -window root /workspace/docker/smoke_test_screenshot.png 2>/tmp/import.log || true

if kill -0 "$WINE_PID" 2>/dev/null && pgrep -f hus-clean.exe >/dev/null; then
    echo "SMOKE_TEST_RESULT=OK (processus toujours actif apres 6s)"
    RESULT=0
else
    echo "SMOKE_TEST_RESULT=FAIL (processus termine prematurement)"
    RESULT=1
fi

echo "--- stdout wine ---"
cat /tmp/wine_stdout.log
echo "--- stderr wine ---"
cat /tmp/wine_stderr.log

kill "$WINE_PID" 2>/dev/null
pkill -f hus-clean.exe 2>/dev/null
/usr/lib/wine/wineserver64 -k 2>/dev/null
kill "$XVFB_PID" 2>/dev/null

exit $RESULT
