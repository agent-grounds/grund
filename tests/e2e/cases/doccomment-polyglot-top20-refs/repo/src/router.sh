#!/bin/sh
# AR-shell: Shell router
#
# ## 1. Contract
#
# Routes requests from shell.
route() {
  # Implements §FS-entry.1.
  # See §FS-gone-shell.
  one="data §api/FS-str-shell"
  many="first line
data §api/FS-multi-shell
"
  echo "$one$many"
}
