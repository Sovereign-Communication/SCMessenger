"""Unit + end-to-end tests. Run: python -m unittest discover -s scripts/trinode/tests -t scripts -v
(or: python scripts/trinode/tests/test_trinode.py)."""
from __future__ import annotations

import io
import json
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(os.path.dirname(HERE)))

from trinode import cli, collectors, correlate, markers, parse, report, scenario  # noqa: E402
from trinode.tests import synth  # noqa: E402

REAL = os.path.join(HERE, "fixtures", "real")


def events_for(lines, naive=0):
    out = []
    for node, ls in lines.items():
        out += parse.parse_text(node, f"{node}.log", "\n".join(ls) + "\n", 2026, naive)
    return out


def analyse(lines, tol=2.0, ids=None):
    ev = events_for(lines)
    ids = ids or synth.IDS
    skew = correlate.estimate_skew(ev, ids, ["aws", "windows", "android"])
    msgs = correlate.classify_messages(ev, skew, ids, tol_s=tol)
    return ev, skew, msgs, {m["msg_id"]: m for m in msgs}


class TestMarkerTable(unittest.TestCase):
    def test_every_marker_has_source_pointer_and_compiles(self):
        names = set()
        for m in markers.MARKERS:
            self.assertTrue(m.src, m.name)
            self.assertNotIn(m.name, names)
            names.add(m.name)
        not_in_code = [m.name for m in markers.MARKERS if m.src.startswith("NOT-IN-CODE")]
        self.assertEqual(sorted(not_in_code), ["legacy_msg_rx"])

    def test_transport_ack_markers_never_score(self):
        for m in markers.MARKERS:
            if m.event == "transport_ack":
                self.assertFalse(m.evidence)


class TestNormalization(unittest.TestCase):
    def test_docker_double_timestamp_uses_app_clock(self):
        ev = parse.parse_text("aws", "d.log",
                              "2026-09-29T20:20:03.985707722Z 2026-09-29T20:20:03.985541Z  INFO scmessenger_core::transport::swarm: "
                              f"Identified peer {synth.WIN_ID} - agent: scmessenger/0.4.0/full/relay/{synth.WIN_ID}, protocols: 16, discoverable_addrs: 14\n")
        self.assertEqual(len(ev), 1)
        e = ev[0]
        self.assertEqual(e["event"], "peer_identified")
        self.assertEqual(e["peer"], synth.WIN_ID)
        self.assertEqual(e["ts_utc"], "2026-09-29T20:20:03.985541Z")
        self.assertEqual(e["src_line"], "d.log:1")
        for k in ("node", "ts_utc", "ts_raw", "event", "msg_id", "peer", "detail", "src_line"):
            self.assertIn(k, e)

    def test_json_tracing_line(self):
        ev = parse.parse_text("android", "a.log", synth.andj(5, f"Ledger exchange response from {synth.WIN_ID}: they learned 1 new peers, sent 3 back") + "\n")
        self.assertEqual(ev[0]["event"], "ledger_exchange_response")
        self.assertEqual(ev[0]["detail"]["learned"], "1")
        self.assertEqual(ev[0]["detail"]["n"], "3")

    def test_logcat_year_utc_and_delivery_state(self):
        ev = parse.parse_text("android", "l.txt",
                              "2026-09-29 21:02:29.617 +0000  1415  1544 I MeshRepository: delivery_state msg=abc-1 state=pending detail=message_prepared_local_history_written\n")
        self.assertEqual(ev[0]["msg_id"], "abc-1")
        self.assertEqual(ev[0]["detail"]["state"], "pending")
        self.assertEqual(ev[0]["ts_utc"], "2026-09-29T21:02:29.617000Z")

    def test_timber_naive_time_uses_offset(self):
        line = "2026-09-29 11:10:44.782 I/Mesh: delivery_state msg=x1 state=pending detail=d\n"
        ev = parse.parse_text("android", "t.log", line, naive_offset_min=-600)
        self.assertEqual(ev[0]["ts_utc"], "2026-09-29T21:10:44.782000Z")
        self.assertTrue(ev[0]["detail"]["naive_ts"])

    def test_threadtime_needs_year(self):
        ev = parse.parse_text("android", "t.log",
                              "09-29 21:02:29.617  1415  1544 I Tag: delivery_state msg=x1 state=pending detail=d\n", year=2026)
        self.assertEqual(ev[0]["ts_utc"], "2026-09-29T21:02:29.617000Z")

    def test_ack_sent_derives_decrypt_and_receipt(self):
        ev = parse.parse_text("windows", "w.log", synth.win(1, f"Sending delivery ACK for m1 to {synth.AND_ID}") + "\n")
        self.assertEqual(sorted(e["event"] for e in ev), ["receipt_sent", "rx_decrypted"])
        self.assertTrue(all(e["detail"]["inferred"] for e in ev))

    def test_connection_established_prefix_hex(self):
        ev = parse.parse_text("aws", "a.log", "2026-09-29T20:00:00.000000Z DEBUG scmessenger_core::transport::manager: Connection established to [39, 60, 19, d1, d0, 43, e2, 6] via Internet\n")
        self.assertEqual(ev[0]["peer"], "396019d1d043e206")

    def test_unrelated_lines_ignored(self):
        self.assertEqual(parse.parse_text("aws", "x", "garbage\n2026-09-29T20:00:00.000000Z DEBUG x: nothing here\n"), [])

    def test_real_fixture_parses(self):
        for node, fn in (("aws", "docker.log.fixture"), ("windows", "scm.log.2026-09-29-21.fixture"), ("android", "scmessenger-mesh.log.fixture")):
            ev = parse.parse_files(node, [os.path.join(REAL, node, fn)])
            self.assertGreater(len(ev), 10, node)
            self.assertTrue(all(e["ts_utc"] for e in ev))


class TestSkew(unittest.TestCase):
    def test_synthetic_skew_recovered(self):
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        ev, skew, _, _ = analyse(lines)
        self.assertEqual(skew["reference"], "aws")
        w = skew["nodes"]["windows"]["offset_s"]
        a = skew["nodes"]["android"]["offset_s"]
        self.assertAlmostEqual(w, synth.WINDOWS_SKEW, delta=0.35)
        self.assertAlmostEqual(a, synth.ANDROID_SKEW, delta=0.35)
        self.assertIn("identify", skew["nodes"]["windows"]["method"])

    def test_android_via_windows_when_no_direct_pair(self):
        lines = synth.connect_lines()
        lines["aws"] = [l for l in lines["aws"] if synth.AND_ID not in l]       # aws never sees android
        lines["android"] = [l for l in lines["android"] if synth.AWS_ID not in l]
        ev = events_for(lines)
        skew = correlate.estimate_skew(ev, synth.IDS, ["aws", "windows", "android"])
        self.assertIn("via windows", skew["nodes"]["android"]["method"])
        self.assertAlmostEqual(skew["nodes"]["android"]["offset_s"], synth.ANDROID_SKEW, delta=0.5)

    def test_msg_rtt_method_when_no_identify(self):
        lines = synth.msg_lines()
        ev = events_for(lines)
        r = correlate._msg_rtt_pair_offset(ev, "windows", "android")
        self.assertIsNotNone(r)
        self.assertEqual(r["method"], "msg_rtt_ntp_style")

    def test_real_fixture_skew(self):
        ev = []
        for node, fn in (("aws", "docker.log.fixture"), ("windows", "scm.log.2026-09-29-21.fixture"), ("android", "scmessenger-mesh.log.fixture")):
            ev += parse.parse_files(node, [os.path.join(REAL, node, fn)])
        ids = synth.IDS
        skew = correlate.estimate_skew(ev, ids, ["aws", "windows", "android"])
        # real capture: android clock ~1.8 s ahead of windows at 21:02:50-52
        pair = [p for p in skew["pairs"] if p["a"] == "android" and p["b"] == "windows"][0]
        self.assertTrue(-3.0 < pair["offset_s"] < -0.5, pair)    # windows - android
        self.assertLess(abs(skew["nodes"]["windows"]["offset_s"]), 3.0)

    def test_tz_suggestion(self):
        lines = synth.connect_lines()
        ev = events_for(lines)
        shifted = []
        for e in ev:
            if e["node"] == "android":
                e = dict(e, ts_utc=(parse.ts(e) + __import__("datetime").timedelta(hours=-10)).strftime("%Y-%m-%dT%H:%M:%S.%fZ"))
            shifted.append(e)
        sug = correlate.suggest_tz_offset(shifted, synth.IDS, "windows", "android")
        self.assertEqual(sug, 600)


class TestClassification(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ev, cls.skew, cls.msgs, cls.by = analyse(synth.merge(synth.connect_lines(), synth.msg_lines()))

    def test_verified_full_chain_android_to_windows(self):
        m = self.by["m-ok-aw"]
        self.assertEqual(m["status"], "VERIFIED", m)
        self.assertEqual(m["direction"], "android->windows")
        self.assertEqual(m["missing"], [])

    def test_verified_via_inferred_legs_is_labelled(self):
        m = self.by["m-ok-wa"]
        self.assertEqual(m["status"], "VERIFIED", m)
        self.assertEqual(m["direction"], "windows->android")
        self.assertTrue(any("inferred" in n for n in m["notes"]))

    def test_partial_lists_missing_legs(self):
        m = self.by["m-partial"]
        self.assertEqual(m["status"], "PARTIAL")
        self.assertEqual(sorted(m["missing"]), ["history", "receipt"])

    def test_receipt_without_decrypt_is_contradicted(self):
        m = self.by["m-contra"]
        self.assertEqual(m["status"], "CONTRADICTED")
        self.assertTrue(any("without receiver decrypt" in c for c in m["contradictions"]))

    def test_state_regression_is_contradicted(self):
        m = self.by["m-regress"]
        self.assertEqual(m["status"], "CONTRADICTED")
        self.assertTrue(any("delivered -> pending" in c for c in m["contradictions"]))

    def test_transport_ack_alone_never_verifies(self):
        m = self.by["m-transport"]
        self.assertEqual(m["status"], "PARTIAL")
        self.assertEqual(sorted(m["missing"]), ["decrypt", "history", "receipt"])

    def test_receipt_without_receiver_capture_is_partial_not_contradicted(self):
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        lines["windows"] = [l for l in lines["windows"] if "m-contra" not in l]
        # shrink windows coverage so it does not span t=90
        lines["windows"] = [l for l in lines["windows"] if not any(f"21:0{m}:" in l for m in (1, 2)) or "Dialing" in l]
        _, _, _, by = analyse(lines)
        self.assertEqual(by["m-contra"]["status"], "PARTIAL")

    def test_causality_violation_beyond_skew_is_contradicted(self):
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        # receipt arrives 30 s BEFORE the receiver decrypted: impossible for any skew we measured
        lines["android"] = [l for l in lines["android"] if "m-ok-aw status=delivered" not in l]
        lines["android"].append(synth.andlog(30.0, "[RECEIPT-RX] Received from core: msg=m-ok-aw status=delivered"))
        _, _, _, by = analyse(lines)
        self.assertEqual(by["m-ok-aw"]["status"], "CONTRADICTED")
        self.assertTrue(any("causality" in c for c in by["m-ok-aw"]["contradictions"]))

    def test_small_apparent_reordering_inside_tolerance_is_ok(self):
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        lines["android"] = [l for l in lines["android"] if "m-ok-aw status=delivered" not in l]
        lines["android"].append(synth.andlog(60.3, "[RECEIPT-RX] Received from core: msg=m-ok-aw status=delivered"))
        _, _, _, by = analyse(lines)
        self.assertEqual(by["m-ok-aw"]["status"], "VERIFIED", by["m-ok-aw"])

    def test_metadata_kind_is_not_a_chat_message(self):
        line = synth.andlog(5, f"UNIFICATION onMessageReceived pairing: sender={synth.WIN_ID} canonical=x routePeerId=null messageId=sync-1 kind=identity_sync transport=INTERNET isKnownContact=true")
        ev = events_for({"android": [line], "aws": [], "windows": []})
        skew = correlate.estimate_skew(ev, synth.IDS, ["android"])
        self.assertEqual(correlate.classify_messages(ev, skew, synth.IDS), [])

    def test_unknown_msg_id_ignored(self):
        ev = parse.parse_text("android", "x", synth.andlog(1, "delivery_state msg=unknown state=pending detail=d") + "\n")
        skew = correlate.estimate_skew(ev, {}, ["android"])
        self.assertEqual(correlate.classify_messages(ev, skew, {}), [])


class TestLedger(unittest.TestCase):
    def test_pairs_and_inferred_learning(self):
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        ev, skew, _, _ = analyse(lines)
        rep = correlate.ledger_report(ev, synth.IDS, skew)
        pr = rep["pairs"]["android<->windows"]
        self.assertTrue(pr["exchanged"])
        self.assertEqual(pr["directions"]["android_view_of_windows"]["entries_in_responses"], 2)
        self.assertFalse(rep["pairs"]["aws<->windows"]["exchanged"])
        learned = [(x["learner"], x["via"], x["learned"], x["evidence"]) for x in rep["address_learning"]]
        self.assertIn(("android", "windows", "aws", "inferred"), learned)

    def test_direct_bootstrap_disqualifies_inference(self):
        lines = synth.connect_lines()
        lines["android"].insert(0, synth.andj(5, f"[OK]  Dialing bootstrap: {synth.AWS_ADDR}"))
        ev = events_for(lines)
        skew = correlate.estimate_skew(ev, synth.IDS, ["aws", "windows", "android"])
        learned = correlate.ledger_learning(ev, synth.IDS, skew)
        self.assertFalse([x for x in learned if x["learner"] == "android" and x["learned"] == "aws"])

    def test_explicit_marker_wins(self):
        lines = synth.connect_lines()
        lines["android"].append(synth.andj(35, f"ledger_address_learned peer={synth.AWS_ID} via={synth.WIN_ID} addr={synth.AWS_ADDR}"))
        ev = events_for(lines)
        skew = correlate.estimate_skew(ev, synth.IDS, ["aws", "windows", "android"])
        learned = correlate.ledger_learning(ev, synth.IDS, skew)
        self.assertTrue([x for x in learned if x["evidence"] == "explicit" and x["learned"] == "aws"])

    def test_real_fixture_android_windows_exchange(self):
        ev = []
        for node, fn in (("aws", "docker.log.fixture"), ("windows", "scm.log.2026-09-29-21.fixture"), ("android", "scmessenger-mesh.log.fixture")):
            ev += parse.parse_files(node, [os.path.join(REAL, node, fn)])
        skew = correlate.estimate_skew(ev, synth.IDS, ["aws", "windows", "android"])
        rep = correlate.ledger_report(ev, synth.IDS, skew)
        self.assertTrue(rep["pairs"]["android<->windows"]["exchanged"])
        self.assertIn("android<->aws", rep["pairs"])


class TestScenario(unittest.TestCase):
    def _run(self, lines, strict=False):
        ev, skew, msgs, _ = analyse(lines)
        return scenario.run_scenario("invite-aws-windows-android", ev, synth.IDS, skew, msgs, ["203.0.113.10"], strict)

    def test_connection_steps_pass_and_message_step_fails_with_bad_messages(self):
        steps = self._run(synth.merge(synth.connect_lines(), synth.msg_lines()))
        st = {s["step"]: s["status"] for s in steps}
        self.assertEqual([st[1], st[2], st[3], st[4]], ["PASS"] * 4, steps)
        self.assertEqual(st[5], "FAIL")
        self.assertTrue(steps[0]["evidence"])

    def test_all_pass_with_clean_messages(self):
        good = synth.msg_lines()
        keep = ("m-ok-aw", "m-ok-wa", "Relay custody", "tick")
        good = {k: [l for l in v if any(t in l for t in keep)] for k, v in good.items()}
        # add the reverse direction already present (m-ok-wa) ; both directions now exist
        steps = self._run(synth.merge(synth.connect_lines(), good))
        self.assertTrue(all(s["status"] == "PASS" for s in steps), steps)

    def test_strict_rejects_inferred_ledger(self):
        steps = self._run(synth.merge(synth.connect_lines(), synth.msg_lines()), strict=True)
        self.assertEqual({s["step"]: s["status"] for s in steps}[4], "FAIL")

    def test_missing_bootstrap_naming_fails_step1(self):
        lines = synth.connect_lines()
        lines["windows"] = [l for l in lines["windows"] if "Dialing bootstrap" not in l]
        steps = self._run(synth.merge(lines, synth.msg_lines()))
        self.assertEqual(steps[0]["status"], "FAIL")

    def test_missing_aws_logs_is_no_data(self):
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        lines["aws"] = []
        steps = self._run(lines)
        self.assertIn("NO_DATA", [s["status"] for s in steps])

    def test_one_sided_identify_fails(self):
        lines = synth.connect_lines()
        lines["aws"] = [l for l in lines["aws"] if synth.WIN_ID not in l]
        steps = self._run(synth.merge(lines, synth.msg_lines()))
        self.assertEqual(steps[1]["status"], "FAIL")


class TestDecideAndExit(unittest.TestCase):
    def test_pass_only_when_all_verified(self):
        v = report.decide([{"msg_id": "a", "status": "VERIFIED", "missing": [], "contradictions": []}], [],
                          ["aws"], ["aws"])
        self.assertEqual(v["exit_code"], 0)

    def test_partial_is_fail(self):
        v = report.decide([{"msg_id": "a", "status": "PARTIAL", "missing": ["history"], "contradictions": []}], [],
                          ["aws"], ["aws"])
        self.assertEqual(v["exit_code"], 1)

    def test_no_messages_is_insufficient(self):
        self.assertEqual(report.decide([], [], ["aws"], ["aws"])["exit_code"], 2)

    def test_missing_node_is_insufficient(self):
        v = report.decide([{"msg_id": "a", "status": "VERIFIED", "missing": [], "contradictions": []}], [],
                          ["aws"], ["aws", "windows"])
        self.assertEqual(v["exit_code"], 2)

    def test_contradiction_beats_insufficient(self):
        v = report.decide([{"msg_id": "a", "status": "CONTRADICTED", "missing": [], "contradictions": ["x"]}], [],
                          ["aws"], ["aws", "windows"])
        self.assertEqual(v["exit_code"], 1)

    def test_required_message_absent_is_insufficient(self):
        v = report.decide([{"msg_id": "a", "status": "VERIFIED", "missing": [], "contradictions": []}], [],
                          ["aws"], ["aws"], required_msg_ids=["zzz"])
        self.assertEqual(v["exit_code"], 2)


class TestObservabilityMarkers(unittest.TestCase):
    def _ev(self, node, text):
        return parse.parse_text(node, f"{node}.log", synth.win(1, text) + "\n", 2026, 0)

    def test_transport_marker_with_and_without_peer_count(self):
        e = self._ev("windows", "[TRANSPORT] kind=ble state=unavailable detail=no_adapter")
        self.assertEqual(e[0]["event"], "transport_status")
        self.assertEqual(e[0]["detail"]["tkind"], "ble")
        self.assertEqual(e[0]["detail"]["state"], "unavailable")
        self.assertEqual(e[0]["detail"]["reason"], "no_adapter")
        self.assertNotIn("peers", e[0]["detail"])
        e = self._ev("aws", "[TRANSPORT] kind=quic state=connected peers=3 detail=periodic_total_3")
        self.assertEqual(e[0]["detail"]["peers"], "3")

    def test_routing_marker(self):
        e = self._ev("android", "[ROUTING] peer_seen peer=abcdef0123456789 source=ble")
        self.assertEqual(e[0]["event"], "routing_peer_seen")
        self.assertEqual(e[0]["peer"], "abcdef0123456789")
        self.assertEqual(e[0]["detail"]["source"], "ble")

    def test_rx_markers_feed_receiver_legs(self):
        e = self._ev("windows", "rx_decrypt msg=m1 from=abcdef0123456789 type=text result=ok")
        self.assertEqual((e[0]["event"], e[0]["msg_id"], e[0]["detail"]["kind"]), ("rx_decrypted", "m1", "text"))
        e = self._ev("windows", "rx_history msg=m1 from=abcdef0123456789 result=ok dup=false hidden=false")
        self.assertEqual(e[0]["event"], "rx_history")
        e = self._ev("windows", "rx_history msg=m1 from=abcdef0123456789 result=failed dup=false hidden=false")
        self.assertEqual(e[0]["event"], "rx_history_failed")

    def test_rx_drop_suppressed_and_stall(self):
        e = self._ev("android", "[RX-DROP] suppressed=7 stage=decode")
        self.assertEqual(e[0]["event"], "rx_drop_suppressed")
        self.assertEqual(e[0]["detail"]["count"], "7")
        e = self._ev("android", "[RX-STALL] idle_ms=30000")
        self.assertEqual(e[0]["event"], "rx_stall")

    def test_rx_drop_and_mesh_stop(self):
        e = self._ev("android", "[RX-DROP] msg=m9 stage=decode reason=bad_sig")
        self.assertEqual((e[0]["event"], e[0]["msg_id"], e[0]["detail"]["stage"], e[0]["detail"]["reason"]),
                         ("rx_drop", "m9", "decode", "bad_sig"))
        e = self._ev("android", "[MESH-STOP] swarm_shutdown timeout ms=5000")
        self.assertEqual((e[0]["event"], e[0]["detail"]["phase"], e[0]["detail"]["result"], e[0]["detail"]["ms"]),
                         ("mesh_stop", "swarm_shutdown", "timeout", "5000"))
        e = self._ev("android", "[MESH-STOP] complete")
        self.assertEqual(e[0]["detail"]["phase"], "complete")

    def test_custody_and_ledger_markers_are_in_code_now(self):
        e = self._ev("aws", "custody_accept msg=m5 from=aaaa1111bbbb2222 dest=cccc3333dddd4444")
        self.assertEqual((e[0]["event"], e[0]["msg_id"]), ("relay_custody_accept", "m5"))
        e = self._ev("windows", "ledger_address_learned peer=aaaa1111bbbb2222 via=ledger_exchange:cccc3333dddd4444 "
                                "addr=/ip4/1.2.3.4/tcp/9001")
        self.assertEqual(e[0]["detail"]["via"], "ledger_exchange:cccc3333dddd4444")

    def test_explicit_ledger_via_prefix_resolves_to_node(self):
        lines = synth.connect_lines()
        lines["android"].append(synth.andj(
            35, f"ledger_address_learned peer={synth.AWS_ID[:16]} via=ledger_exchange:{synth.WIN_ID[:16]} "
                f"addr={synth.AWS_ADDR}"))
        ev = events_for(lines)
        skew = correlate.estimate_skew(ev, synth.IDS, ["aws", "windows", "android"])
        learned = correlate.ledger_learning(ev, synth.IDS, skew)
        hit = [x for x in learned if x["evidence"] == "explicit"]
        self.assertTrue(hit and hit[0]["via"] == "windows" and hit[0]["learned"] == "aws", hit)

    def test_transport_table_marks_silent_kinds_and_tracks_changes(self):
        lines = synth.connect_lines()
        lines["windows"] += [
            synth.win(2, "[TRANSPORT] kind=ble state=unavailable detail=no_adapter"),
            synth.win(3, "[TRANSPORT] kind=tcp4 state=listening detail=listen_port_9001"),
            synth.win(4, "[TRANSPORT] kind=tcp4 state=connected peers=2 detail=periodic_total_2_unclassified_0"),
            synth.win(5, "[TRANSPORT] kind=tcp4 state=error detail=listener_failed"),
        ]
        ev = events_for(lines)
        tab = correlate.transport_availability(ev, ["aws", "windows", "android"])
        self.assertEqual(tab["windows"]["ble"]["state"], "unavailable")
        self.assertEqual(tab["windows"]["ble"]["detail"], "no adapter")
        self.assertEqual(tab["windows"]["tcp4"]["state"], "error")
        self.assertEqual(tab["windows"]["tcp4"]["max_peers"], 2)
        self.assertEqual(tab["windows"]["tcp4"]["changes"], 2)
        self.assertEqual(tab["aws"]["ble"]["state"], "no-marker")
        self.assertEqual(tab["android"]["cellular"]["state"], "no-marker")

    def test_rx_drop_becomes_note_not_status(self):
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        lines["windows"].append(synth.win(80.5, "[RX-DROP] msg=m-partial stage=decode reason=bad_sig"))
        _, _, _, by = analyse(lines)
        self.assertEqual(by["m-partial"]["status"], "PARTIAL")
        self.assertTrue(any("rx_drop on windows stage=decode" in n for n in by["m-partial"]["notes"]))

    def test_explicit_rx_legs_verify_a_message(self):
        lines = synth.connect_lines()
        lines["android"].append(synth.andlog(
            60.0, "delivery_state msg=m-x state=pending detail=message_prepared_local_history_written"))
        lines["windows"] += [
            synth.win(60.3, f"rx_decrypt msg=m-x from={synth.AND_ID[:16]} type=text result=ok"),
            synth.win(60.4, f"rx_history msg=m-x from={synth.AND_ID[:16]} result=ok dup=false hidden=false"),
        ]
        lines["android"].append(synth.andlog(60.9, "[RECEIPT-RX] Received from core: msg=m-x status=delivered"))
        _, _, _, by = analyse(lines)
        self.assertEqual(by["m-x"]["status"], "VERIFIED", by["m-x"])

    def test_failed_history_write_is_not_the_history_leg(self):
        lines = synth.connect_lines()
        lines["android"].append(synth.andlog(
            60.0, "delivery_state msg=m-y state=pending detail=message_prepared_local_history_written"))
        lines["windows"] += [
            synth.win(60.3, f"rx_decrypt msg=m-y from={synth.AND_ID[:16]} type=text result=ok"),
            synth.win(60.4, f"rx_history msg=m-y from={synth.AND_ID[:16]} result=failed dup=false hidden=false"),
        ]
        lines["android"].append(synth.andlog(60.9, "[RECEIPT-RX] Received from core: msg=m-y status=delivered"))
        _, _, _, by = analyse(lines)
        self.assertEqual(by["m-y"]["status"], "PARTIAL")
        self.assertIn("history", by["m-y"]["missing"])

    def test_mesh_stop_summary_flags_timeout(self):
        lines = synth.connect_lines()
        lines["android"] += [
            synth.andlog(50, "[MESH-STOP] requested"),
            synth.andlog(51, "[MESH-STOP] swarm_shutdown ok ms=40"),
            synth.andlog(52, "[MESH-STOP] rust_stop timeout ms=5000"),
            synth.andlog(53, "[MESH-STOP] complete"),
        ]
        ev = events_for(lines)
        ds = correlate.drop_and_stop_summary(ev, ["aws", "windows", "android"])
        self.assertFalse(ds["mesh_stop"]["android"]["clean"])
        self.assertEqual(len(ds["mesh_stop"]["android"]["sequence"]), 4)
        self.assertIsNone(ds["mesh_stop"]["windows"]["clean"])

    def test_verdict_md_has_transport_table_and_default_ports(self):
        self.assertIn(":9876/", cli.build_parser().get_default("win_diag_url"))
        lines = synth.merge(synth.connect_lines(), synth.msg_lines())
        lines["windows"].append(synth.win(2, "[TRANSPORT] kind=ble state=unavailable detail=no_D-Bus"))
        lines["android"].append(synth.andlog(2, "[TRANSPORT] kind=cellular state=connected detail=validated_internet"))
        import shutil
        src, root = tempfile.mkdtemp(), tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, src, True)
        self.addCleanup(shutil.rmtree, root, True)
        synth.build(src, lines)
        cli.run(["--from-dir", src, "--repo-root", root, "--run-id", "tx1"], out=io.StringIO())
        md = None
        for dp, dn, fn in os.walk(os.path.join(root, "tmp", "evidence")):
            if "verdict.md" in fn:
                with open(os.path.join(dp, "verdict.md"), encoding="utf-8") as fh:
                    md = fh.read()
        self.assertIn("## Transport availability per node", md)
        self.assertIn("unavailable (no D-Bus)", md)
        self.assertIn("connected (validated internet)", md)
        self.assertIn("NO-MARKER", md)


class FakeRunner:
    def __init__(self):
        self.calls = []

    def __call__(self, argv, timeout):
        self.calls.append(list(argv))
        joined = " ".join(argv)
        if "logcat" in joined:
            return collectors.CmdResult(0, (synth.andlog(1, "delivery_state msg=r1 state=pending detail=message_prepared_local_history_written") + "\n").encode())
        if "ls files/logs" in joined:
            return collectors.CmdResult(0, b"scmessenger-mesh.log\n")
        if "cat files/logs" in joined or "cat files/mesh" in joined:
            return collectors.CmdResult(0, (synth.andj(2, "tick") + "\n").encode())
        if "dumpsys package" in joined:
            return collectors.CmdResult(0, b"    versionName=0.4.0\n")
        if "docker logs" in joined:
            return collectors.CmdResult(0, (synth.aws(1, "Relay custody audit log count: 9") + "\n").encode())
        if "curl" in joined:
            return collectors.CmdResult(0, b'{"custody_audit_count": 9, "peers": []}')
        return collectors.CmdResult(0, b"image=testbotz/scmessenger:sha-x started=t status=running\n2026-10-01T21:00:00Z\n")


class TestCollectorsOffline(unittest.TestCase):
    def test_android_collector_commands_and_files(self):
        r = FakeRunner()
        with tempfile.TemporaryDirectory() as d:
            res = collectors.AndroidCollector("10.0.0.5:5555", runner=r).collect(d)
            self.assertIn("raw/android/logcat.txt", res.files)
            self.assertEqual(res.meta["version_name"], "0.4.0")
        self.assertTrue(all(c[:3] == ["adb", "-s", "10.0.0.5:5555"] for c in r.calls))
        flat = [" ".join(c) for c in r.calls]
        self.assertTrue(any("-v threadtime -v year -v UTC" in f for f in flat))
        self.assertTrue(all(not any(w in f for w in ("install", "uninstall", "clear", "rm ")) for f in flat))

    def test_aws_collector_needs_host_and_never_stores_it(self):
        old = os.environ.pop("SCM_AWS_HOST", None)
        try:
            with tempfile.TemporaryDirectory() as d:
                res = collectors.AwsCollector(runner=FakeRunner()).collect(d)
                self.assertTrue(res.errors and not res.files)
                r = FakeRunner()
                res = collectors.AwsCollector(host="192.0.2.77", user="u", key="/k", runner=r).collect(d)
                self.assertIn("raw/aws/docker.log", res.files)
                self.assertNotIn("192.0.2.77", json.dumps(res.meta))
                self.assertTrue(any("docker logs -t --since 2h scm-node" in " ".join(c) for c in r.calls))
        finally:
            if old is not None:
                os.environ["SCM_AWS_HOST"] = old

    def test_windows_collector_reads_hourly_files(self):
        with tempfile.TemporaryDirectory() as src, tempfile.TemporaryDirectory() as run:
            for h in ("20", "21"):
                with open(os.path.join(src, f"scm.log.2026-10-01-{h}"), "w") as fh:
                    fh.write(synth.win(1, "x") + "\n")
            res = collectors.WindowsCollector(src, fetch=lambda u, t: b'{"custody_audit_count": 3, "peers": ["p"]}').collect(run)
            self.assertEqual(len([f for f in res.files if "scm.log" in f]), 2)
            self.assertEqual(res.meta["diagnostics"]["custody_audit_count"], 3)

    def test_diag_self_peer_id_from_circuit_listener(self):
        d = collectors._diag_summary(json.dumps({"listeners": [f"/ip4/1.2.3.4/tcp/9001/p2p/{synth.AWS_ID}/p2p-circuit/p2p/{synth.WIN_ID}"]}).encode())
        self.assertEqual(d["self_peer_id"], synth.WIN_ID)


class TestEndToEnd(unittest.TestCase):
    def _run(self, lines, extra=()):
        import shutil
        src, root = tempfile.mkdtemp(), tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, src, True)
        self.addCleanup(shutil.rmtree, root, True)
        synth.build(src, lines)
        buf = io.StringIO()
        args = ["--from-dir", src, "--repo-root", root, "--run-id", "t1",
                "--peer-id", f"aws={synth.AWS_ID}", "--peer-id", f"windows={synth.WIN_ID}",
                "--peer-id", f"android={synth.AND_ID}", *extra]
        code = cli.run(args, out=buf)
        run_dirs = []
        for dp, dn, fn in os.walk(os.path.join(root, "tmp", "evidence")):
            if "manifest.json" in fn:
                run_dirs.append(dp)
        self.assertEqual(len(run_dirs), 1)
        rd = run_dirs[0]

        def read(n):
            with open(os.path.join(rd, n), encoding="utf-8") as fh:
                return fh.read()
        return code, rd, json.loads(read("manifest.json")), json.loads(read("verdict.json")), read("verdict.md"), os.listdir(rd)

    def test_failing_run_outputs_everything(self):
        code, rd, man, ver, md, listing = self._run(synth.merge(synth.connect_lines(), synth.msg_lines()),
                                                    ["--scenario", "invite-aws-windows-android"])
        self.assertEqual(code, 1)
        self.assertEqual(ver["overall"], "FAIL")
        for f in ("events.jsonl", "manifest.json", "verdict.md", "verdict.json", "raw"):
            self.assertIn(f, listing)
        self.assertEqual(man["schema_version"], 1)
        self.assertEqual(sorted(man["nodes"]), ["android", "aws", "windows"])
        for n in man["nodes"].values():
            for f in n["files"]:
                self.assertEqual(len(f["sha256"]), 64)
                self.assertTrue(os.path.isfile(os.path.join(rd, f["path"])))
        self.assertIn("clock_skew", man)
        self.assertIn("VERIFIED", md)
        self.assertIn("m-ok-aw", md)
        # date/run-id path convention
        self.assertRegex(rd.replace("\\", "/"), r"tmp/evidence/\d{4}-\d{2}-\d{2}/t1$")

    def test_passing_run_exit_zero(self):
        keep = ("m-ok-aw", "m-ok-wa", "Relay custody", "tick")
        good = {k: [l for l in v if any(t in l for t in keep)] for k, v in synth.msg_lines().items()}
        code, _, _, ver, _, _ = self._run(synth.merge(synth.connect_lines(), good),
                                          ["--scenario", "invite-aws-windows-android"])
        self.assertEqual(code, 0, ver["failed"] + ver["insufficient"])
        self.assertEqual(ver["overall"], "PASS")

    def test_events_jsonl_is_normalized_schema(self):
        _, rd, _, _, _, _ = self._run(synth.merge(synth.connect_lines(), synth.msg_lines()))
        with open(os.path.join(rd, "events.jsonl"), encoding="utf-8") as fh:
            rows = [json.loads(l) for l in fh]
        self.assertGreater(len(rows), 20)
        for r in rows:
            self.assertEqual(sorted(r), ["detail", "event", "msg_id", "node", "peer", "src_line", "ts_raw", "ts_utc"])

    def test_no_sources_is_insufficient(self):
        self.assertEqual(cli.run([], out=io.StringIO()), 2)

    def test_empty_logs_exit_2(self):
        code, *_ = self._run({"aws": [synth.aws(1, "Relay custody audit log count: 1")], "windows": [synth.win(1, "x")],
                              "android": [synth.andj(1, "x")]})
        self.assertEqual(code, 2)

    def test_from_dir_flat_map(self):
        with tempfile.TemporaryDirectory() as src, tempfile.TemporaryDirectory() as root:
            with open(os.path.join(src, "w.log"), "w") as fh:
                fh.write(synth.win(1, f"Delivery ACK received from {synth.AND_ID}: msg_id=zz") + "\n")
            code = cli.run(["--from-dir", src, "--map", "windows=w.log", "--repo-root", root, "--run-id", "f1"],
                           out=io.StringIO())
            self.assertIn(code, (1, 2))


if __name__ == "__main__":
    unittest.main(verbosity=2)
