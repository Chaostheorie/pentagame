#!/usr/bin/env bash
# -*- coding: utf-8 -*-

# Script for compiling frontend stylsheets for pentagame-online
# Requires being run from npm or npm with purgecss and sass (dart) being installed
# WARNING: This requires GNU Coreutils Stat and GNU Grep being installed
# This script only compiles if one of the files changed
# You will need to remove your static/scss/.dir-changes file, if you're adding a new scss file
# under GPL v3.0 @ Cobalt <cobalt.rocks> (see pentagame-online LICENSE)

# Constants
CHANGE_FILE="./scss/.dir-changes"
DIST_DIR="./dist"
FILES=(./scss/*.scss ../templates/**/*.html ./purgecss.config.js)
FORCE_REBUILD=0

# Compiling scss and removing unnecessary css
compile_scss() {
    echo "[ASSETS]: Compiling SCSS -> CSS"
    sass --style=compressed --load-path scss/ --load-path node_modules/ scss/main.scss "$DIST_DIR/css/app.css"
    purgecss --config purgecss.config.js --css "$DIST_DIR/css/app.css" --content "../templates/**/*.html" --output "$DIST_DIR/css"
    echo "[ASSETS]: Done"
}

# Builds change file
build_change_file() {
    # Ensure file is present && empty
    if [ -f "$CHANGE_FILE" ]; then
        rm "$CHANGE_FILE"
        touch "$CHANGE_FILE"
    fi

    for f in "${FILES[@]}"; do
        FILE_MODIFY=$(stat "$f" | grep Modify)
        echo "$f:$FILE_MODIFY" >>"$CHANGE_FILE"
    done
}

# loads file and checks stats => 1: recompilation required 0: nothing todo
load_change_file() {
    INDEX=0

    # This just reads it line by line and compares the current stat to the saved stat
    # I need to extend this, when I have too much time, to support checking
    # for Filenames and have graceful handling of new files
    while IFS= read -r line; do
        CURRENT_CHANGE=$(stat "${FILES[INDEX]}" | grep Modify)
        if [ "${FILES[INDEX]}:$CURRENT_CHANGE" != "$line" ]; then
            echo 1
        fi
        INDEX=$((INDEX + 1))
    done <"$CHANGE_FILE"
    echo 0
}

# Check for dist dir
if [ ! -d "$DIST_DIR" ]; then
    echo "$DIST_DIR missing. Creating new one."
    mkdir "$DIST_DIR"
    FORCE_REBUILD=1
    if [ -d "node_modules" ]; then
        echo "[ASSETS]: Copying fonts to $DIST_DIR"
        mkdir -p "$DIST_DIR/css/fonts"
        cp -r ./node_modules/bootstrap-icons/font/fonts/ "$DIST_DIR/css/"
        echo "[ASSETS]: Copying images to $DIST_DIR"
        mkdir -p "$DIST_DIR/images"
        cp -r ./images/ "$DIST_DIR/"
        echo "[ASSETS]: Done copying assets"
    else
        echo "[ASSETS]: Please install all required modules with npm before continuing."
        exit 1
    fi
fi

# Get modified timestamp
if [ -f "$CHANGE_FILE" ]; then
    if [ "$(load_change_file)" != "0" ] || [ "$FORCE_REBUILD" == "1" ]; then
        echo "[ASSETS]: Rebuilding assets"
        compile_scss
        build_change_file
    fi
    echo "[ASSETS]: All assets are ready"
    # When no changes made -> no recompilation needed
else
    # When the file doesn't exists it should be built and the scss should be compiled
    echo "[ASSETS]: Rebuilding assets"
    build_change_file
    compile_scss
fi
