# Building Block View

## Whitebox Overall System

| Building block | Responsibility |
|---|---|
| Booking API | books, moves and cancels seats; answers occupancy |
| Release job | releases seats not claimed in time |
| Door gateway | turns a door reader's tap into a claim |
| Booking store | the PostgreSQL database every instance shares |

The code for the first two is `src/carrel/`.
