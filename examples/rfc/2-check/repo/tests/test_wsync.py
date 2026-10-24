"""Tests of the Widget Sync server, one class per section of the draft."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent / "src"))

from wsync import Collection, etag  # noqa: E402


class ListingChanges(unittest.TestCase):
    """§draft-ietf-wsync-widget-sync.4.1"""

    def test_lists_only_what_changed_since_the_token(self):
        widgets = Collection()
        widgets.add("w1", "Red widget")
        _, _, first = widgets.list_changes()
        widgets.add("w2", "Green widget")
        _, _, second = widgets.list_changes(first["token"])
        self.assertEqual([w["id"] for w in second["changes"]], ["w2"])

    def test_a_token_older_than_the_history_is_gone(self):
        widgets = Collection(history_limit=1)
        widgets.add("w1", "Red widget")
        widgets.add("w2", "Green widget")
        self.assertEqual(widgets.list_changes("0")[0], 410)


class ConditionalUpdate(unittest.TestCase):
    """§draft-ietf-wsync-widget-sync.4.2 and §draft-ietf-wsync-widget-sync.4.3"""

    def setUp(self):
        self.widgets = Collection()
        self.widget = self.widgets.add("w1", "Red widget")
        self.seen = etag(self.widget)

    def test_a_put_without_if_match_is_refused(self):
        self.assertEqual(self.widgets.update("w1", None, "Blue widget")[0], 428)

    def test_a_stale_entity_tag_reports_the_current_widget(self):
        self.widgets.update("w1", self.seen, "Teal widget")
        status, headers, problem = self.widgets.update("w1", self.seen, "Blue widget")
        self.assertEqual(status, 412)
        self.assertEqual(problem["current"]["name"], "Teal widget")
        self.assertEqual(headers["ETag"], '"w1-2"')


if __name__ == "__main__":
    unittest.main()
