"""The module follows §FS-a and names §REQ-1x."""

PAYLOAD = """stored §REQ-2x"""


def sites():
    # A comment cites §REQ-3x.
    return "string §REQ-4x", "escaped \u00a7REQ-5x"
