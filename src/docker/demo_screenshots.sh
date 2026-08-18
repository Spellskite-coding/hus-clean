#!/usr/bin/env bash
# Genere une serie de captures d'ecran du flux complet (scan -> nettoyage)
# pour illustrer la procedure utilisateur, en pilotant l'appli sous Wine
# via xdotool (clics simules), sans interface graphique reelle (Xvfb).
set -u

EXE=/workspace/target/x86_64-pc-windows-msvc/release/hus-clean.exe
OUT=/workspace/docker/demo
export WINEPREFIX=/tmp/wineprefix
export WINEDEBUG=-all
export DISPLAY=:99

mkdir -p "$OUT"

Xvfb :99 -screen 0 1280x1024x24 >/tmp/xvfb.log 2>&1 &
sleep 2

# Pre-cree quelques faux fichiers temp "realistes" dans le faux C: de Wine
# pour que la capture de la liste de resultats soit parlante.
TEMP_DIR="$WINEPREFIX/drive_c/users/$(whoami)/Temp"
mkdir -p "$TEMP_DIR/sub"
echo x > "$TEMP_DIR/setup_8f21ac.tmp"
echo x > "$TEMP_DIR/update_cache.log"
echo x > "$TEMP_DIR/sub/download_442a.tmp"

/usr/lib/wine/wine64 "$EXE" >/tmp/out.log 2>/tmp/err.log &
sleep 6
import -display :99 -window root "$OUT/1_accueil.png"
chown -R "$(id -u):$(id -g)" "$OUT"
echo "STEP1_DONE"
