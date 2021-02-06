#!/usr/bin/env bash
# -*- coding: utf-8 -*-

# Script for packaging pentagame-online for different distributions
#

if ! command -v cargo &>/dev/null; then
    echo "You need to install cargo and cargo deb"
    exit 1
fi

cargo deb
