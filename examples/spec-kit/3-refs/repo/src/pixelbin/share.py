"""Share links for albums, as §SPEC-002-album-share specifies."""

from dataclasses import dataclass
from datetime import datetime, timedelta
from secrets import token_urlsafe

LIFETIME = timedelta(days=7)


@dataclass
class ShareLink:
    """An unguessable token for one album (§SPEC-002-album-share.requirements.entities)."""

    album: str
    created: datetime
    token: str
    revoked: bool = False


def share(album: str, owner: str, requester: str, now: datetime) -> ShareLink:
    """Create a link for an album its owner names.

    Implements §SPEC-002-album-share.requirements.functional.fr-001.
    """
    if requester != owner:
        raise PermissionError(f"only the owner of {album} can share it")
    return ShareLink(album, now, token_urlsafe(16))


def serves(link: ShareLink, now: datetime) -> bool:
    """Whether a link still shows its album."""
    # §SPEC-002-album-share.requirements.functional.fr-002
    if now - link.created >= LIFETIME:
        return False
    # §SPEC-002-album-share.requirements.functional.fr-003
    return not link.revoked
