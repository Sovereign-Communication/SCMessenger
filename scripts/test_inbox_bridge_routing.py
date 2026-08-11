#!/usr/bin/env python3
"""Standalone tests for the /handoff vs ordinary-chat vs housekeeping
routing decision in scripts/inbox_bridge.py.

No live node is required: these exercise the pure functions
(parse_command, classify_content, route_message, _normalize_peer_ids) and
the durable-write helper (append_chat_log) directly, with no network calls.
Run with:

    python scripts/test_inbox_bridge_routing.py
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import inbox_bridge as ib  # noqa: E402


class ParseCommandTests(unittest.TestCase):
    def test_handoff_with_body(self):
        is_handoff, body = ib.parse_command("/handoff foo")
        self.assertTrue(is_handoff)
        self.assertEqual(body, "foo")

    def test_handoff_uppercase_case_insensitive(self):
        is_handoff, body = ib.parse_command("/HANDOFF foo")
        self.assertTrue(is_handoff)
        self.assertEqual(body, "foo")

    def test_handoff_mixed_case(self):
        is_handoff, body = ib.parse_command("/HandOff do the thing")
        self.assertTrue(is_handoff)
        self.assertEqual(body, "do the thing")

    def test_handoff_alone_no_body(self):
        is_handoff, body = ib.parse_command("/handoff")
        self.assertTrue(is_handoff)
        self.assertEqual(body, "")

    def test_handoff_alone_trailing_whitespace_only(self):
        is_handoff, body = ib.parse_command("/handoff   ")
        self.assertTrue(is_handoff)
        self.assertEqual(body, "")

    def test_prefix_mid_message_is_ordinary_chat(self):
        # The command only counts at the start of the message, not anywhere
        # inside it.
        is_handoff, body = ib.parse_command("foo /handoff bar")
        self.assertFalse(is_handoff)
        self.assertEqual(body, "foo /handoff bar")

    def test_empty_content_is_ordinary_chat(self):
        is_handoff, body = ib.parse_command("")
        self.assertFalse(is_handoff)
        self.assertEqual(body, "")

    def test_leading_whitespace_before_command_still_matches(self):
        is_handoff, body = ib.parse_command("   /handoff foo")
        self.assertTrue(is_handoff)
        self.assertEqual(body, "foo")

    def test_lookalike_token_not_a_command(self):
        # "/handoffoo" is not "/handoff" followed by whitespace or end of
        # string, so it must not be treated as the command.
        is_handoff, body = ib.parse_command("/handoffoo bar")
        self.assertFalse(is_handoff)
        self.assertEqual(body, "/handoffoo bar")

    def test_ordinary_chat_content_untouched(self):
        is_handoff, body = ib.parse_command("just saying hi")
        self.assertFalse(is_handoff)
        self.assertEqual(body, "just saying hi")


def _envelope(kind=None, text="", **extra):
    """Build a scm.message.* envelope JSON string the way the Android/iOS
    apps actually emit it (encodeMeshMessagePayload() always sets `schema`,
    `kind`, and `text`; `sender` is included here too since every real
    envelope carries one, though classify_content() does not depend on it).
    """
    body = {"schema": "scm.message.identity.v1"}
    if kind is not None:
        body["kind"] = kind
    body["text"] = text
    body["sender"] = extra.pop(
        "sender",
        {
            "identity_id": "1111111111111111111111111111111111111111111111111111111111111111",
            "public_key": "2222222222222222222222222222222222222222222222222222222222222222",
            "device_id": "00000000-0000-4000-8000-000000000000",
            "nickname": "Lucaso",
        },
    )
    body.update(extra)
    return json.dumps(body)


class ClassifyContentTests(unittest.TestCase):
    """Exercises classify_content() directly against the observed shapes:
    real history_sync/identity_sync/history_sync_data traffic pulled from
    this node's own /api/history (see HANDOFF/logs/chat/2026-08-11.jsonl and
    HANDOFF/review/WINDOWS_ANDROID_PROBE_2026-08-10.md), plus the real human
    message (e7a8b366, "confirmed delivery probe.") that proved `schema`
    alone cannot be used to detect housekeeping -- it is identical on both.
    """

    # -- observed housekeeping shapes -------------------------------------

    def test_identity_sync_envelope_no_text_key_is_housekeeping(self):
        # Shape 1 from the task: {"schema":"scm.message.identity.v1","kind":...}
        # -- kind present, no separate text key at all.
        content = json.dumps(
            {"schema": "scm.message.identity.v1", "kind": "identity_sync"}
        )
        category, text = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")
        self.assertEqual(text, "")

    def test_history_sync_empty_text_is_housekeeping(self):
        # Shape 2: {"text":"","kind":"history_sync","sender":...}
        content = _envelope(kind="history_sync", text="")
        category, text = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")
        self.assertEqual(text, "")

    def test_empty_text_with_schema_no_explicit_kind_is_housekeeping(self):
        # Shape 3: {"text":"","schema":"scm.message.identity.v1",...} -- no
        # `kind` key at all, just schema + blank text.
        content = json.dumps(
            {
                "text": "",
                "schema": "scm.message.identity.v1",
                "sender": {"identity_id": "phone"},
            }
        )
        category, text = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")

    def test_history_sync_data_nonempty_text_is_housekeeping_by_kind(self):
        # history_sync_data carries a non-empty `text` (a JSON array of the
        # sender's own history rows) -- an empty-text-only rule would miss
        # this; it must be caught by `kind` specifically.
        content = _envelope(
            kind="history_sync_data",
            text='[{"id":"m1","dir":"sent","pid":"x","txt":"hi","ts":1,"sts":1,"del":true}]',
        )
        category, _ = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")

    def test_real_recorded_history_sync_envelope_is_housekeeping(self):
        # The literal envelope actually logged by this bridge run (see
        # HANDOFF/logs/chat/2026-08-11.jsonl, message 8662c8cf).
        content = (
            '{"schema":"scm.message.identity.v1","kind":"history_sync","text":"",'
            '"sender":{"identity_id":"1111111111111111111111111111111111111111111111111111111111111111",'
            '"public_key":"2222222222222222222222222222222222222222222222222222222222222222",'
            '"device_id":"00000000-0000-4000-8000-000000000000","nickname":"Lucaso"}}'
        )
        category, text = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")
        self.assertEqual(text, "")

    def test_empty_content_is_housekeeping(self):
        category, _ = ib.classify_content("")
        self.assertEqual(category, "housekeeping")

    def test_whitespace_only_content_is_housekeeping(self):
        category, _ = ib.classify_content("   \n  ")
        self.assertEqual(category, "housekeeping")

    def test_text_kind_with_empty_text_is_housekeeping(self):
        content = _envelope(kind="text", text="   ")
        category, _ = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")

    # -- real human messages must never be swallowed -----------------------

    def test_enveloped_human_text_is_content_despite_identical_schema(self):
        # The exact regression this predicate exists to prevent: this
        # envelope has the SAME schema string as every housekeeping example
        # above. Only `kind`/`text` may be used to tell them apart.
        content = _envelope(kind="text", text="confirmed delivery probe.")
        category, text = ib.classify_content(content)
        self.assertEqual(category, "content")
        self.assertEqual(text, "confirmed delivery probe.")

    def test_real_recorded_human_message_envelope_is_content(self):
        # The literal envelope from message e7a8b366 (see
        # HANDOFF/todo/INBOX_2026-08-11T043333Z_e7a8b366ff27.md) -- a
        # verified human reply, carrying the identical `schema` field as the
        # history_sync ping logged right next to it.
        content = (
            '{"schema":"scm.message.identity.v1","kind":"text",'
            '"text":"confirmed delivery probe.","sender":{"identity_id":'
            '"1111111111111111111111111111111111111111111111111111111111111111"}}'
        )
        category, text = ib.classify_content(content)
        self.assertEqual(category, "content")
        self.assertEqual(text, "confirmed delivery probe.")

    def test_plain_string_content_is_content(self):
        category, text = ib.classify_content("hey, you there?")
        self.assertEqual(category, "content")
        self.assertEqual(text, "hey, you there?")

    def test_json_looking_but_not_envelope_is_content(self):
        # Has no `kind` and no `schema`+`text` pair -- not our envelope
        # shape, so it must not be swallowed as housekeeping.
        content = json.dumps({"foo": "bar", "note": "not a control message"})
        category, text = ib.classify_content(content)
        self.assertEqual(category, "content")
        self.assertEqual(text, content)

    def test_prose_mentioning_json_is_content(self):
        content = 'hey check this out: {"kind": "history_sync"} weird right?'
        category, text = ib.classify_content(content)
        self.assertEqual(category, "content")
        self.assertEqual(text, content)

    def test_literal_string_history_sync_is_content_not_housekeeping(self):
        # A human typing the word "history_sync" is plain text, not a JSON
        # control envelope -- must not be misclassified just because it
        # matches a control-kind name.
        category, text = ib.classify_content("history_sync")
        self.assertEqual(category, "content")
        self.assertEqual(text, "history_sync")

    def test_unrecognized_kind_with_text_falls_through_to_content(self):
        # Conservative default: an unknown future `kind` with real text in
        # it is NOT assumed to be housekeeping -- only the known control set
        # is treated that way.
        content = _envelope(kind="some_future_kind", text="surprise me")
        category, text = ib.classify_content(content)
        self.assertEqual(category, "content")
        self.assertEqual(text, "surprise me")

    def test_enveloped_handoff_text_is_content_with_command_preserved(self):
        content = _envelope(kind="text", text="/handoff do the thing")
        category, text = ib.classify_content(content)
        self.assertEqual(category, "content")
        self.assertEqual(text, "/handoff do the thing")

    # -- bare delivery receipts (ACK-storm regression) ---------------------
    #
    # 2026-08-11: allow-listing a second peer that emits bare receipts caused
    # an unbounded ACK loop -- the bridge ACKed each receipt, the far node
    # emitted a receipt for that ACK, and round it went (~1400 messages in
    # minutes). Receipts must classify as housekeeping so no ACK is sent.

    def test_bare_delivery_receipt_is_housekeeping(self):
        content = json.dumps(
            {
                "message_id": "ae31b5af-0ebc-4eb0-b09a-f26f684e24ab",
                "status": "Delivered",
                "timestamp": 1786472561,
            }
        )
        category, _ = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")

    def test_bare_delivery_receipt_without_timestamp_is_housekeeping(self):
        content = json.dumps({"message_id": "abc", "status": "Delivered"})
        category, _ = ib.classify_content(content)
        self.assertEqual(category, "housekeeping")

    def test_receipt_shape_carrying_text_is_content(self):
        # A text-bearing field disqualifies the receipt match, so a human
        # message can never be swallowed by this rule.
        content = json.dumps(
            {"message_id": "abc", "status": "Delivered", "text": "read this"}
        )
        category, _ = ib.classify_content(content)
        self.assertEqual(category, "content")

    def test_partial_receipt_shape_is_content(self):
        content = json.dumps({"message_id": "abc"})
        category, _ = ib.classify_content(content)
        self.assertEqual(category, "content")

    def test_receipt_with_unknown_extra_key_is_content(self):
        content = json.dumps(
            {"message_id": "abc", "status": "Delivered", "payload": "surprise"}
        )
        category, _ = ib.classify_content(content)
        self.assertEqual(category, "content")

    def test_human_message_mentioning_a_receipt_is_content(self):
        content = "[PR139 RECEIPT ACK] chunks 1/87 through 87/87 verified"
        category, text = ib.classify_content(content)
        self.assertEqual(category, "content")
        self.assertEqual(text, content)


class RouteMessageTests(unittest.TestCase):
    ALLOWED = ["good-peer-1", "good-peer-2"]

    def test_allowlisted_handoff_routes_to_handoff(self):
        self.assertEqual(
            ib.route_message("good-peer-1", "/handoff foo", self.ALLOWED), "handoff"
        )

    def test_allowlisted_uppercase_handoff_routes_to_handoff(self):
        self.assertEqual(
            ib.route_message("good-peer-2", "/HANDOFF foo", self.ALLOWED), "handoff"
        )

    def test_allowlisted_bare_handoff_routes_to_handoff(self):
        self.assertEqual(ib.route_message("good-peer-1", "/handoff", self.ALLOWED), "handoff")

    def test_allowlisted_midmessage_prefix_routes_to_chat(self):
        self.assertEqual(
            ib.route_message("good-peer-1", "foo /handoff bar", self.ALLOWED), "chat"
        )

    def test_allowlisted_empty_content_routes_to_housekeeping(self):
        # Behaviour change from the pre-housekeeping-route design: there is
        # no text here to ticket or log either way, so empty/whitespace
        # content is now swept into "housekeeping" (seen-only, no reply)
        # rather than "chat" (which would fire a [SEEN] ack for nothing).
        self.assertEqual(
            ib.route_message("good-peer-1", "", self.ALLOWED), "housekeeping"
        )
        self.assertEqual(
            ib.route_message("good-peer-1", "   \n ", self.ALLOWED), "housekeeping"
        )

    def test_allowlisted_ordinary_text_routes_to_chat(self):
        self.assertEqual(ib.route_message("good-peer-1", "hey there", self.ALLOWED), "chat")

    def test_non_allowlisted_sender_is_ignored_even_with_handoff(self):
        # Silence for non-allow-listed senders is a security property: it
        # must hold regardless of message content.
        self.assertEqual(
            ib.route_message("stranger", "/handoff foo", self.ALLOWED), "ignored"
        )

    def test_non_allowlisted_sender_is_ignored_for_ordinary_chat(self):
        self.assertEqual(
            ib.route_message("stranger", "hello", self.ALLOWED), "ignored"
        )

    def test_second_allowlisted_peer_also_routes(self):
        # allowed_peer_id may be a list; every entry is a valid sender.
        self.assertEqual(
            ib.route_message("good-peer-2", "/handoff foo", self.ALLOWED), "handoff"
        )

    # -- housekeeping (identity_sync/history_sync/history_sync_data/blank) --

    def test_allowlisted_history_sync_envelope_is_housekeeping(self):
        content = _envelope(kind="history_sync", text="")
        self.assertEqual(
            ib.route_message("good-peer-1", content, self.ALLOWED), "housekeeping"
        )

    def test_allowlisted_identity_sync_envelope_is_housekeeping(self):
        content = json.dumps(
            {"schema": "scm.message.identity.v1", "kind": "identity_sync"}
        )
        self.assertEqual(
            ib.route_message("good-peer-1", content, self.ALLOWED), "housekeeping"
        )

    def test_allowlisted_history_sync_data_envelope_is_housekeeping(self):
        content = _envelope(kind="history_sync_data", text='[{"id":"m1"}]')
        self.assertEqual(
            ib.route_message("good-peer-1", content, self.ALLOWED), "housekeeping"
        )

    def test_allowlisted_blank_text_schema_only_envelope_is_housekeeping(self):
        content = json.dumps(
            {"text": "", "schema": "scm.message.identity.v1", "sender": {}}
        )
        self.assertEqual(
            ib.route_message("good-peer-1", content, self.ALLOWED), "housekeeping"
        )

    def test_housekeeping_from_non_allowlisted_sender_is_still_ignored(self):
        # Allow-list gate always wins, regardless of content shape.
        content = _envelope(kind="history_sync", text="")
        self.assertEqual(
            ib.route_message("stranger", content, self.ALLOWED), "ignored"
        )

    # -- real envelope traffic (kind:"text") still routes correctly --------

    def test_allowlisted_enveloped_human_text_routes_to_chat(self):
        content = _envelope(kind="text", text="hey there")
        self.assertEqual(
            ib.route_message("good-peer-1", content, self.ALLOWED), "chat"
        )

    def test_allowlisted_enveloped_handoff_routes_to_handoff(self):
        # Regression check: the /handoff prefix must be tested against the
        # unwrapped text, not the raw envelope string (which always starts
        # with '{' and could never match "/handoff").
        content = _envelope(kind="text", text="/handoff do the thing")
        self.assertEqual(
            ib.route_message("good-peer-1", content, self.ALLOWED), "handoff"
        )

    def test_allowlisted_json_looking_non_envelope_routes_to_chat(self):
        content = json.dumps({"foo": "bar"})
        self.assertEqual(
            ib.route_message("good-peer-1", content, self.ALLOWED), "chat"
        )

    def test_allowlisted_literal_history_sync_string_routes_to_chat(self):
        self.assertEqual(
            ib.route_message("good-peer-1", "history_sync", self.ALLOWED), "chat"
        )


class NormalizePeerIdsTests(unittest.TestCase):
    def test_single_string_still_works(self):
        self.assertEqual(ib._normalize_peer_ids("solo-peer"), ["solo-peer"])

    def test_list_of_strings(self):
        self.assertEqual(
            ib._normalize_peer_ids(["peer-a", "peer-b"]), ["peer-a", "peer-b"]
        )

    def test_list_dedupes_preserving_order(self):
        self.assertEqual(
            ib._normalize_peer_ids(["peer-a", "peer-b", "peer-a"]), ["peer-a", "peer-b"]
        )

    def test_blank_and_empty_entries_dropped(self):
        self.assertEqual(ib._normalize_peer_ids(["peer-a", "  ", ""]), ["peer-a"])

    def test_empty_string_yields_empty_list(self):
        self.assertEqual(ib._normalize_peer_ids(""), [])

    def test_whitespace_trimmed(self):
        self.assertEqual(ib._normalize_peer_ids("  peer-a  "), ["peer-a"])


class AppendChatLogTests(unittest.TestCase):
    def setUp(self):
        self._tmpdir = tempfile.TemporaryDirectory()
        self._orig_dir = ib.HANDOFF_CHAT_LOG_DIR
        ib.HANDOFF_CHAT_LOG_DIR = Path(self._tmpdir.name) / "chat"

    def tearDown(self):
        ib.HANDOFF_CHAT_LOG_DIR = self._orig_dir
        self._tmpdir.cleanup()

    def test_appends_one_json_object_per_line(self):
        message = {"id": "m1", "timestamp": 1700000000000, "content": "hi there"}
        path1 = ib.append_chat_log(message, "good-peer-1")
        message2 = {"id": "m2", "timestamp": 1700000005000, "content": "second"}
        path2 = ib.append_chat_log(message2, "good-peer-1")

        self.assertEqual(path1, path2)  # same UTC day -> same file
        self.assertTrue(path1.is_file())

        lines = path1.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), 2)

        record1 = json.loads(lines[0])
        self.assertEqual(record1["message_id"], "m1")
        self.assertEqual(record1["peer_id"], "good-peer-1")
        self.assertEqual(record1["content"], "hi there")
        self.assertIn("timestamp", record1)

        record2 = json.loads(lines[1])
        self.assertEqual(record2["message_id"], "m2")

    def test_enveloped_content_is_unwrapped_before_logging(self):
        # Regression: the log used to store the raw scm.message.* envelope
        # verbatim (an unreadable wall of escaped JSON per line -- see the
        # pre-fix HANDOFF/logs/chat/2026-08-11.jsonl entry). It must now
        # store just the human text.
        message = {
            "id": "m3",
            "timestamp": 1700000010000,
            "content": _envelope(kind="text", text="hi from the envelope"),
        }
        path = ib.append_chat_log(message, "good-peer-1")
        record = json.loads(path.read_text(encoding="utf-8").splitlines()[-1])
        self.assertEqual(record["content"], "hi from the envelope")


if __name__ == "__main__":
    unittest.main(verbosity=2)
