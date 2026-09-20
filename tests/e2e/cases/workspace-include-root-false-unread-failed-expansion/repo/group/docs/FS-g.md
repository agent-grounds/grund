# FS-g: G

Nothing scans this file, so on a run that expanded the workspace the block
above would earn `no project scans`. This run does not expand it: the sibling
member's config does not parse, so the answer that warning depends on was
never produced, and the run exits 2 on the config error instead.
