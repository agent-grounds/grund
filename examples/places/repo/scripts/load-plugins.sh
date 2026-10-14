#!/bin/sh
# Loads every plugin in ./plugins (§FS-plugin-load).
for plugin in plugins/*; do . "$plugin"; done
