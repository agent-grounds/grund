# Requirements

## FS-image: A native image is built ahead of time

The image starts fast because its heap is snapshotted at build time (§FS-heap),
and the build is tuned by §ee/FS-pgo.

## FS-heap: The image heap is snapshotted at build time

The heap the build saw is the heap §FS-image starts with.
