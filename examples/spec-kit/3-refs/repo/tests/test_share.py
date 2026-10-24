"""Tests for §SPEC-002-album-share, written before share.py."""

from datetime import datetime, timedelta

import pytest

from pixelbin.share import share, serves

NOW = datetime(2026, 10, 12, 9, 0)


def test_only_the_owner_can_share():
    # §SPEC-002-album-share.requirements.functional.fr-001
    with pytest.raises(PermissionError):
        share("holiday", owner="mira", requester="tomas", now=NOW)


def test_a_link_expires_after_seven_days():
    # §SPEC-002-album-share.requirements.functional.fr-002
    link = share("holiday", owner="mira", requester="mira", now=NOW)
    assert serves(link, NOW + timedelta(days=6))
    assert not serves(link, NOW + timedelta(days=7))


def test_a_revoked_link_serves_nothing():
    # §SPEC-002-album-share.requirements.functional.fr-003
    link = share("holiday", owner="mira", requester="mira", now=NOW)
    link.revoked = True
    assert not serves(link, NOW)
