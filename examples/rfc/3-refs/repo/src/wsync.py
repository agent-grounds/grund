"""A Widget Sync server, §draft-ietf-wsync-widget-sync, kept in memory.

A response is a (status, headers, body) triple, so the protocol logic runs
without an HTTP stack.
"""

STALE_WIDGET = "https://iana.org/assignments/http-problem-types#stale-widget"


def etag(widget: dict) -> str:
    """The strong entity tag of a widget, §draft-ietf-wsync-widget-sync.3.2.

    It names the widget and its revision, so it changes with every change.
    """
    return f'"{widget["id"]}-{widget["revision"]}"'


class Collection:
    """One owner's widgets, §draft-ietf-wsync-widget-sync.3.1, and their change history."""

    def __init__(self, history_limit: int = 1000) -> None:
        self.widgets: dict[str, dict] = {}
        self.log: list[str] = []  # widget id per change; the token is an index
        self.first = 0  # the oldest change still held
        self.history_limit = history_limit

    def add(self, widget_id: str, name: str) -> dict:
        widget = {"id": widget_id, "name": name, "revision": 1}
        self.widgets[widget_id] = widget
        self._record(widget_id)
        return widget

    def _record(self, widget_id: str) -> None:
        self.log.append(widget_id)
        if len(self.log) - self.first > self.history_limit:
            self.first = len(self.log) - self.history_limit

    def list_changes(self, since: str | None = None):
        """GET on the collection, §draft-ietf-wsync-widget-sync.4.1."""
        start = 0 if since is None else int(since)
        if start < self.first:
            return 410, {}, None  # §RFC9110.15.5.11: the client relists
        changed = dict.fromkeys(self.log[start:])
        body = {
            "changes": [self.widgets[i] for i in changed],
            "token": str(len(self.log)),
        }
        return 200, {}, body

    def update(self, widget_id: str, if_match: str | None, name: str):
        """PUT on a widget, §draft-ietf-wsync-widget-sync.4.2."""
        current = self.widgets[widget_id]
        if if_match is None:
            return 428, {}, None  # §RFC6585
        if if_match != etag(current):
            return 412, {"ETag": etag(current)}, conflict(current)
        current["name"] = name
        current["revision"] += 1
        self._record(widget_id)
        return 200, {"ETag": etag(current)}, current


def conflict(current: dict) -> dict:
    """The problem details of a 412, §draft-ietf-wsync-widget-sync.4.3."""
    return {
        "type": STALE_WIDGET,
        "title": "Stale Widget",
        "status": 412,
        "current": current,  # an extension member, §RFC9457.3.2
    }
