"""Tests for §SPEC-001-photo-upload, written before upload.py."""

import pytest

from pixelbin.upload import MAX_BYTES, Refused, upload

JPEG = b"\xff\xd8\xff\xe0" + b"\x00" * 1024


def test_a_jpeg_is_stored_and_its_id_returned():
    # §SPEC-001-photo-upload.requirements.functional.fr-001
    library = {}
    photo_id = upload(library, "image/jpeg", JPEG, {})
    assert library[photo_id].data == JPEG


def test_gps_tags_are_removed():
    # §SPEC-001-photo-upload.requirements.functional.fr-003
    library = {}
    photo_id = upload(library, "image/jpeg", JPEG, {"GPSLatitude": 45.3, "Model": "X100"})
    assert library[photo_id].tags == {"Model": "X100"}


@pytest.mark.parametrize("media_type, size", [("image/gif", 10), ("image/jpeg", MAX_BYTES + 1)])
def test_the_wrong_file_is_refused(media_type, size):
    # §SPEC-001-photo-upload.requirements.functional.fr-002
    library = {}
    with pytest.raises(Refused):
        upload(library, media_type, b"\x00" * size, {})
    assert library == {}
