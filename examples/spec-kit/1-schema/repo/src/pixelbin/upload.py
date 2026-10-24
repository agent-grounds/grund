"""Store uploaded photos, as §SPEC-001-photo-upload specifies."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from uuid import uuid4

ACCEPTED = ("image/jpeg", "image/png")
MAX_BYTES = 20 * 1024 * 1024


class Refused(Exception):
    """A file the library will not store; its message names the limit it broke."""


@dataclass
class Photo:
    """One stored image (§SPEC-001-photo-upload.requirements.entities)."""

    media_type: str
    data: bytes
    tags: dict
    id: str = field(default_factory=lambda: uuid4().hex)
    uploaded: datetime = field(default_factory=lambda: datetime.now(timezone.utc))


def strip_gps(tags: dict) -> dict:
    """Drop every GPS tag (§SPEC-001-photo-upload.requirements.functional.fr-003)."""
    return {name: value for name, value in tags.items() if not name.startswith("GPS")}


def upload(library: dict, media_type: str, data: bytes, tags: dict) -> str:
    """Store one photo and return its new ID.

    Implements §SPEC-001-photo-upload.requirements.functional.fr-001.
    """
    # §SPEC-001-photo-upload.requirements.functional.fr-002
    if media_type not in ACCEPTED:
        raise Refused(f"only JPEG and PNG are accepted, not {media_type}")
    if len(data) > MAX_BYTES:
        raise Refused("a photo may be at most 20 MB")
    photo = Photo(media_type, data, strip_gps(tags))
    library[photo.id] = photo
    return photo.id
