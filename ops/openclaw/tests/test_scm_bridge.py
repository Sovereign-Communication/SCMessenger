import json
import os
import subprocess
import tempfile
import unittest
from unittest import mock

import scm_bridge


class BridgeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.db = scm_bridge.connect_db(os.path.join(self.tmp.name, "bridge.sqlite3"))

    def tearDown(self):
        self.db.close()
        self.tmp.cleanup()

    def test_allowlist_fails_closed(self):
        path = os.path.join(self.tmp.name, "allow.json")
        for content in (None, "{}", "[]", "{bad"):
            if content is None:
                with self.assertRaises(scm_bridge.AllowlistError):
                    scm_bridge.load_allowlist(path)
            else:
                with open(path, "w", encoding="utf-8") as f:
                    f.write(content)
                with self.assertRaises(scm_bridge.AllowlistError):
                    scm_bridge.load_allowlist(path)
        with open(path, "w", encoding="utf-8") as f:
            json.dump({"peer-1": "approved"}, f)
        self.assertEqual(scm_bridge.load_allowlist(path), {"peer-1": "approved"})

    def test_duplicate_ingestion_and_send_retry_do_not_repeat_turn(self):
        messages = [{"id": "m1", "peer_id": "peer-1", "content": "hello", "direction": "inbound"}]
        self.assertEqual(scm_bridge.ingest(self.db, messages, {"peer-1": "ok"}), 1)
        self.assertEqual(scm_bridge.ingest(self.db, messages, {"peer-1": "ok"}), 0)
        calls = []
        self.assertTrue(scm_bridge.process_one(self.db, lambda body: calls.append(body) or "answer"))
        self.assertEqual(calls, ["hello"])
        send = mock.Mock(side_effect=[scm_bridge.RetryableSend("refused"), {"message_id": "remote-1", "status": "accepted"}])
        self.assertTrue(scm_bridge.deliver_one(self.db, {}, send))
        row = self.db.execute("SELECT state, attempts FROM outbox").fetchone()
        self.db.execute("UPDATE outbox SET next_at=0")
        self.db.commit()
        self.assertTrue(scm_bridge.deliver_one(self.db, {}, send))
        self.assertEqual(send.call_count, 2)
        self.assertEqual(calls, ["hello"])
        self.assertEqual(self.db.execute("SELECT state FROM outbox").fetchone()[0], "sent")

    def test_legacy_seen_prevents_replay(self):
        path = os.path.join(self.tmp.name, "seen.json")
        with open(path, "w", encoding="utf-8") as f:
            json.dump(["old-1"], f)
        scm_bridge.migrate_legacy_seen(self.db, path)
        scm_bridge.migrate_legacy_seen(self.db, path)
        self.assertEqual(scm_bridge.ingest(self.db, [{"id": "old-1", "peer_id": "peer-1", "content": "old"}], {"peer-1": "ok"}), 0)
        self.assertEqual(self.db.execute("SELECT state FROM inbox WHERE message_id='old-1'").fetchone()[0], "ignored")

    def test_accepted_lost_response_is_uncertain(self):
        scm_bridge.ingest(self.db, [{"id": "m3", "peer_id": "peer-1", "body": "x"}], {"peer-1": "ok"})
        scm_bridge.process_one(self.db, lambda _: "reply")
        send = mock.Mock(side_effect=TimeoutError("response lost"))
        self.assertTrue(scm_bridge.deliver_one(self.db, {}, send))
        self.assertEqual(self.db.execute("SELECT state FROM outbox").fetchone()[0], "uncertain")
        self.assertFalse(scm_bridge.deliver_one(self.db, {}, send))
        self.assertEqual(send.call_count, 1)

    def test_outbox_receipt_confirms_delivery(self):
        scm_bridge.ingest(self.db, [{"id": "m4", "peer_id": "peer-1", "body": "x"}], {"peer-1": "ok"})
        scm_bridge.process_one(self.db, lambda _: "reply")
        scm_bridge.deliver_one(self.db, {}, lambda *_: {"message_id": "remote-4", "status": "accepted"})
        self.assertEqual(self.db.execute("SELECT state FROM outbox").fetchone()[0], "sent")
        with mock.patch.object(scm_bridge, "http_json", return_value={"message_id": "remote-4", "status": "delivered", "delivered": True}) as http:
            scm_bridge.confirm_outbox(self.db)
        http.assert_called_once_with("/api/send/remote-4")
        self.assertEqual(self.db.execute("SELECT state FROM outbox").fetchone()[0], "delivered")

    def test_agent_timeout_raises(self):
        proc = mock.Mock()
        proc.communicate.side_effect = [subprocess.TimeoutExpired("agent", 1), ("", "")]
        with mock.patch.object(scm_bridge.subprocess, "Popen", return_value=proc), mock.patch.object(scm_bridge.os, "killpg", create=True):
            with self.assertRaises(TimeoutError):
                scm_bridge.agent_turn("x")

    def test_restart_marks_running_turn_uncertain(self):
        scm_bridge.ingest(self.db, [{"id": "m2", "peer_id": "peer-1", "body": "x"}], {"peer-1": "ok"})
        self.db.execute("UPDATE inbox SET state='running' WHERE message_id='m2'")
        self.db.commit()
        self.db.execute("UPDATE inbox SET state='uncertain' WHERE state='running'")
        self.db.commit()
        self.assertEqual(self.db.execute("SELECT state FROM inbox WHERE message_id='m2'").fetchone()[0], "uncertain")
        self.assertFalse(scm_bridge.process_one(self.db, lambda _: self.fail("uncertain turn reran")))


if __name__ == "__main__":
    unittest.main()
