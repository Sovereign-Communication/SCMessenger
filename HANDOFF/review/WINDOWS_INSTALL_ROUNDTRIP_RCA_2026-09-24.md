# Windows install / Android round-trip audit and RCA handoff

**Date:** 2026-09-24
**Scope:** Read-only audit of the approved Windows candidate installation, identity/data preservation, Android relationship, and the Windows -> Android -> Windows receipt path.
**Change boundary:** This documentation-only pass edits this handoff only. It does not change source, tests, build output, installed files, runtime data, Android state, identity, contacts, or deployment.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Canonical handoff-only hard gate (normative)

**Gate ID:** `handoff-only-verified-evidence-v1`

**Controlled status vocabulary (normative):** Status values are closed; do not invent aliases.

<!-- HANDOFF-VOCABULARY-BEGIN -->
| Value | Meaning |
|---|---|
| `HANDOFF_EVIDENCE_VERIFIED` | The evidence handoff satisfies all four preflight requirements. It makes no product-implementation claim. |
| `HANDOFF_UNVERIFIED` | Any required evidence, acceptance, compatibility/rollback, or authorization clause is missing, ambiguous, stale, or unsupported. This is the fail-closed value. |
| `PRODUCT_FIX_UNIMPLEMENTED` | No product fix is implemented or verified by this lane. |
<!-- HANDOFF-VOCABULARY-END -->

**Current status declaration (normative):**

<!-- HANDOFF-STATUS-BEGIN -->
| Field | Value |
|---|---|
| `handoff_evidence` | `HANDOFF_EVIDENCE_VERIFIED` |
| `product_fix` | `PRODUCT_FIX_UNIMPLEMENTED` |
<!-- HANDOFF-STATUS-END -->

This is the canonical rule for this lane. It overrides any wording elsewhere in this document that could be read as permission to implement, operate, or ship a product fix. The lane may investigate and test, but it may not implement product fixes in any repository.

### Non-negotiable boundaries

1. **No product implementation or delivery.** Except for the explicitly bounded disposable test-only changes in item 2, this lane must not edit source, tests, product configuration, migrations, build inputs, generated product artifacts, runtime data, or deployment state in SCMessenger or any receiving/product repository. It must not create or apply a product patch, commit, release, install, deployment, or production migration.
2. **Disposable local test-only changes permitted.** The lane may make temporary, disposable changes in this lane's own isolated local SCMessenger environment, including test-only patches, fixtures, or test configuration, solely to execute verification. It must label them local and non-deliverable, never copy, transfer, commit, or push them into any receiving repository, and discard or quarantine them after verification. They are not product fixes or implementation authorization. If isolation or cleanup cannot be proved, stop and report the result as unverified.
3. **Handoff-only delivery.** Every repository-specific finding, proposed fix, or verification result must be delivered as a handoff to the repository that owns the change. The receiving repository's owner is responsible for implementation and review. This lane stops at the handoff and does not implement, merge, install, deploy, or publish it.
4. **No operational side effects.** Do not send messages, change identity or contacts, install or deploy software, alter production data, commit, or push while producing or updating a handoff. Historical evidence and protected artifacts are not authorization to repeat an operation.
5. **No status inflation.** A healthy service, a transport acknowledgement, a history record, an external completion event, a passing existing test, or a proposed contract is not proof that a product fix was implemented or delivered. A handoff must distinguish evidence, proposal, implementation, and independent verification.

### Handoff preflight (normative; fail closed)

This preflight validates the evidence handoff, not product acceptance. The status block above is the only place where a status may be assigned; standalone `VERIFIED`, `verified`, or legacy status labels are invalid. A status containing `VERIFIED` is permitted only when all four checklist items are present, checked, and backed by the cited sections.

<!-- HANDOFF-PREFLIGHT-BEGIN -->
- [x] `REDACTED_EVIDENCE` — `## Evidence boundary and safety rules`, `## Protected evidence ledger`, and `## Artifact and claim ledger` provide bounded, redacted evidence and explicitly exclude bodies, keys, credentials, and private identifiers.
- [x] `DETERMINISTIC_ACCEPTANCE` — `### Required new tests`, `### Discovery assertion that cannot pass with zero matches`, `delivery_state_transition_test`, and `real_text_roundtrip_test` define deterministic acceptance and reject zero-match filters.
- [x] `COMPATIBILITY_ROLLBACK` — `### Compatibility mapping` and `## Rollback, migration, and acceptance boundaries` define persistence/FFI/client compatibility, migration, a `downgraded binary`, recovery, and rollback boundaries.
- [x] `NO_IMPLEMENTATION_AUTHORIZATION` — `**This handoff is not implementation authorization.**` and the owning repository stop rule prohibit implementation authorization.
<!-- HANDOFF-PREFLIGHT-END -->

The following command is normative. Run it from the repository root. It runs the positive path, the prior negative cases, the generated grammar matrix, and generated placement cases; every negative case must be rejected and the canonical in-block placement cases must remain accepted. It exits non-zero for a missing or contradictory checklist item, an unknown or duplicate status, a forbidden verification claim, a controlled token or alias outside its parsed status-field or canonical vocabulary-table location, or a verified evidence status without all four anchors. It does not modify product source. The executable validator is explicitly delimited as a non-document fixture; all other fenced content is scanned.

The validator builds one normalized document model. It removes Markdown prefixes, case, Unicode punctuation variants, hyphenation, whitespace differences, emphasis, strikethrough, links, inline HTML, and code wrappers before parsing table cells and status ownership. Its closed claim grammar recognizes subject/value assignments, finite predicates with modifiers, passive and completed predicates, reporting/marked predicates, and code-shaped unknown status values; it is not a phrase blacklist. Controlled aliases are compared through the same normalized identity and are accepted only in the exact parsed vocabulary or status-field cells. The vocabulary rows have an explicit owner block so a copied row cannot become canonical by relocation.

<!-- HANDOFF-VALIDATOR-FIXTURE-BEGIN -->
```bash
python - <<'PY'
from pathlib import Path
import html
import re
import unicodedata
from functools import lru_cache

path = Path("HANDOFF/review/WINDOWS_INSTALL_ROUNDTRIP_RCA_2026-09-24.md")
if not path.is_file():
    raise SystemExit(f"missing handoff: {path}")
text = path.read_text(encoding="utf-8")
CONTROLLED_TOKENS = {
    "HANDOFF_EVIDENCE_VERIFIED",
    "HANDOFF_UNVERIFIED",
    "PRODUCT_FIX_UNIMPLEMENTED",
}
STATUS_VALUES = {
    "handoff_evidence": {"HANDOFF_EVIDENCE_VERIFIED", "HANDOFF_UNVERIFIED"},
    "product_fix": {"PRODUCT_FIX_UNIMPLEMENTED"},
}
CHECKLIST_ANCHORS = {
    "REDACTED_EVIDENCE": (
        "## Evidence boundary and safety rules",
        "## Protected evidence ledger",
        "## Artifact and claim ledger",
    ),
    "DETERMINISTIC_ACCEPTANCE": (
        "### Required new tests",
        "### Discovery assertion that cannot pass with zero matches",
        "delivery_state_transition_test",
        "real_text_roundtrip_test",
    ),
    "COMPATIBILITY_ROLLBACK": (
        "### Compatibility mapping",
        "## Rollback, migration, and acceptance boundaries",
        "downgraded binary",
    ),
    "NO_IMPLEMENTATION_AUTHORIZATION": (
        "**This handoff is not implementation authorization.**",
        "owning repository",
    ),
}
def block_span(name, document):
    begin = f"<!-- {name}-BEGIN -->"
    end = f"<!-- {name}-END -->"
    lines = document.splitlines()
    if lines.count(begin) != 1 or lines.count(end) != 1:
        return None
    start = lines.index(begin) + 1
    stop = lines.index(end)
    if start > stop:
        return None
    return start, stop


def document_without_fixture(candidate):
    begin = "<!-- HANDOFF-" + "VALIDATOR-FIXTURE-BEGIN -->"
    end = "<!-- HANDOFF-" + "VALIDATOR-FIXTURE-END -->"
    if candidate.count(begin) != 1 or candidate.count(end) != 1:
        return None
    match = re.search(
        rf"{re.escape(begin)}.*?{re.escape(end)}",
        candidate,
        re.S,
    )
    return candidate[:match.start()] + candidate[match.end():] if match else None


def checklist_entries(candidate):
    return re.findall(
        r"\[([ xX])\]\s*`([^`]+)`\s*([^\n]*)",
        candidate,
    )


DASH_CHARS = set("\u2010\u2011\u2012\u2013\u2014\u2015\u2212\ufe58\ufe63\uff0d")
SOFT_DASH_CHARS = set("\u2010\u2011\u2012")
HARD_DASH_CHARS = DASH_CHARS - SOFT_DASH_CHARS
COLON_CHARS = set("\uff1a\ufe55\ua789\u2236")
EQUAL_CHARS = set("\uff1d\ufe66")
SOFT_HYPHEN = "\ue001"

# These are lexical classes for one closed grammar, not a list of claim phrases.
CLAIM_SUBJECT_HEADS = frozenset({
    "status", "state", "verification", "result", "outcome", "label", "release",
    "handoff", "evidence", "delivery", "implementation", "acceptance", "fix",
    "change", "product", "build", "deployment", "candidate", "proof", "claim",
    "contract", "gate", "migration", "compatibility", "installation", "test", "suite",
    "run", "artifact", "message",
})
CLAIM_SUBJECT_FILLERS = frozenset({
    "status", "state", "verification", "result", "outcome", "release", "handoff",
    "evidence", "delivery", "implementation", "acceptance", "fix", "change", "product",
    "current", "live", "new", "actual", "proposed", "user", "test", "suite", "run",
    "artifact", "message",
})
CLAIM_AUXILIARIES = frozenset({
    "is", "was", "are", "were", "be", "been", "being", "has", "have", "had",
    "will", "would", "can", "could", "may", "might", "must", "shall", "should",
    "ought", "remains", "remain", "remained", "appears", "appeared", "seems",
    "seemed", "shows", "showed", "becomes", "became", "gets", "get", "got",
    "gotten", "proves", "said", "says", "stated", "states", "reported", "reports",
    "declared", "declares", "claimed", "claims", "asserted", "asserts", "noted", "notes",
})
CLAIM_MODIFIERS = frozenset({
    "not", "never", "no", "none", "neither", "nor", "already", "successfully",
    "currently", "now", "fully", "safely", "independently", "actually", "recently",
    "quietly", "reportedly", "been", "being", "be", "as", "to", "yet", "going",
    "getting", "rather", "also", "only", "still", "ever", "here", "there", "longer",
    "fact", "in", "merely", "simply", "positively", "formally", "ultimately",
})
CLAIM_MARKERS = frozenset({
    "marked", "marks", "labeled", "labelled", "reported", "reports", "declared",
    "declares", "confirmed", "confirms", "proven", "proved", "validated", "validates",
    "claimed", "asserted", "represented", "described", "certified", "attested", "announced",
})
CLAIM_COMPLETION_VERBS = frozenset({
    "completed", "completes", "finished", "finishes", "passed", "passes", "proven",
    "proved", "accepted", "accepts", "confirmed", "confirms", "validated", "certified", "attested",
})
CLAIM_VALUES = frozenset({
    "verified", "unverified", "confirmed", "complete", "completed", "passed",
    "accepted", "proven", "proved", "unimplemented", "approved", "rejected", "authorized",
})
CLAIM_STATUS_ONLY_VALUES = frozenset({
    "delivered", "pending", "failed", "unknown", "invalid", "ready", "healthy",
    "active", "inactive", "successful", "success", "closed", "open",
})
NEGATIVE_POLICY_VALUES = frozenset({
    "unverified", "unimplemented", "pending", "failed", "unknown", "invalid",
})
CLAIM_ASSIGNMENT_VALUES = CLAIM_VALUES | CLAIM_STATUS_ONLY_VALUES
BARE_CLAIM_VALUES = frozenset({
    "verified", "unverified", "confirmed", "complete", "completed", "passed",
    "accepted", "proven", "proved", "approved", "rejected", "authorized",
})
CLAIM_BOUNDARIES = frozenset({
    "if", "when", "unless", "while", "after", "before", "because", "so", "and",
    "or", "then", "until", "once", "where", "although", "though", "but", "that",
    "which", "who", "whose",
})
CLAIM_ASSIGNMENT_PUNCTUATION = frozenset({":", "="})
CLAIM_STRICT_ASSIGNMENT_HEADS = frozenset({
    "status", "state", "verification", "release", "handoff", "evidence", "delivery",
    "implementation", "acceptance", "fix", "change", "product",
})
CLAIM_DIRECT_HEADS = frozenset({
    "status", "state", "verification", "result", "outcome", "label", "release",
    "handoff", "evidence", "delivery", "implementation", "acceptance", "change",
    "build", "deployment", "candidate", "proof", "claim", "contract", "gate",
    "migration", "compatibility", "installation", "test", "suite", "run", "artifact", "message",
})
CLAIM_SOFT_STATUS_HEADS = frozenset({
    "status", "state", "verification", "release", "result", "outcome", "label",
    "acceptance", "change", "fix", "handoff",
})
CLAIM_UNKNOWN_VALUE_HEADS = frozenset({
    "unknown", "invalid", "pending", "failed", "unimplemented", "unverified",
    "verified", "confirmed", "complete", "completed", "passed", "accepted",
    "approved", "rejected", "authorized", "ready", "healthy", "active", "inactive",
})
CLAIM_ALIAS_GROUPS = {
    "handoffevidence": {"unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven", "unimplemented"},
    "productfix": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven", "unimplemented"},
    "handoff": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "evidence": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "release": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "status": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "verification": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "delivery": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "acceptance": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "implementation": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "change": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "result": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "outcome": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
    "label": {"verified", "unverified", "confirmed", "complete", "completed", "passed", "accepted", "proven"},
}
TOKEN_RE = re.compile(r"[^\W_]+|[^\w\s]", re.UNICODE)


def split_markdown_prefix(raw):
    value = raw.strip()
    while True:
        previous = value
        value = re.sub(r"^(?:>\s*)+", "", value)
        value = re.sub(r"^#{1,6}\s+", "", value)
        value = re.sub(r"^(?:[-+*]|\d+[.)])\s+", "", value)
        if value == previous:
            return value.strip()


@lru_cache(maxsize=32768)
def normalize_markdown(value):
    value = unicodedata.normalize("NFKC", html.unescape(str(value))).casefold()
    value = re.sub(r"<!--(.*?)-->", r" \1 ", value, flags=re.S)
    value = re.sub(r"!\[([^\]]*)\]\([^)]*\)", r" \1 ", value)
    value = re.sub(r"\[([^\]]*)\]\([^)]*\)", r" \1 ", value)
    value = re.sub(r"\[([^\]]*)\]\[[^\]]*\]", r" \1 ", value)
    value = re.sub(r"<[^>\n]*>", " ", value)
    value = value.replace(chr(92), " ")
    value = re.sub(r"[`*_~]+", " ", value)
    for dash in SOFT_DASH_CHARS:
        value = value.replace(dash, SOFT_HYPHEN)
    for dash in HARD_DASH_CHARS:
        value = value.replace(dash, ":")
    value = re.sub(r"\s+-\s+", " : ", value)
    value = re.sub(r"(?<=[^\W_])-(?=[^\W_])", SOFT_HYPHEN, value)
    value = value.replace("-", " ")
    value = "".join(
        ch if ch in {SOFT_HYPHEN, ":", "="} else
        " " if unicodedata.category(ch)[0] in {"P", "S", "Z"} else ch
        for ch in value
    )
    return re.sub(r"\s+", " ", value).strip()


@lru_cache(maxsize=32768)
def collapse_normalized(value):
    return "".join(ch for ch in str(value) if ch.isalnum())


def collapse(value):
    return collapse_normalized(normalize_markdown(value))


def normalize_status(value):
    return normalize_markdown(value)


def table_cells(content):
    content = content.strip()
    if not content.startswith("|"):
        return None
    parts = []
    buffer = []
    escaped = False
    for char in content:
        if char == "|" and not escaped:
            parts.append("".join(buffer))
            buffer = []
        else:
            buffer.append(char)
        if char == chr(92) and not escaped:
            escaped = True
        else:
            escaped = False
    parts.append("".join(buffer))
    if parts and not parts[0].strip():
        parts = parts[1:]
    if parts and not parts[-1].strip():
        parts = parts[:-1]
    return [cell.strip().replace(chr(92) + "|", "|") for cell in parts]


def exact_code_cell(cell):
    match = re.fullmatch(r"`([^`]+)`", cell.strip())
    return match.group(1) if match else None


def _items(value):
    return [match.group(0) for match in TOKEN_RE.finditer(value)]


def _is_word(item):
    return bool(item) and item[0].isalnum()


def code_only_cell(cell):
    remainder = re.sub(r"`[^`]*`", "", cell)
    return not re.sub(r"[\s,;|]", "", remainder)


def grammar_signal(value):
    return any(
        (_is_word(item) and (
            item in CLAIM_AUXILIARIES
            or item in CLAIM_MARKERS
            or item in CLAIM_COMPLETION_VERBS
        ))
        or (not _is_word(item) and item in CLAIM_ASSIGNMENT_PUNCTUATION)
        for item in _items(value)
    )


@lru_cache(maxsize=32768)
def raw_code_value(raw):
    text = str(raw)
    subject = (
        r"(?:product\s+fix|status|state|verification|release|result|outcome|label|"
        r"delivery|implementation|acceptance|handoff|evidence|fix|change)"
    )
    assignment = re.search(
        rf"\b{subject}\b\s*(?::|=|—|–|−|-)\s*"
        r"(?P<value>[A-Za-z][A-Za-z0-9_-]*)",
        text,
        flags=re.I,
    )
    predicate = re.search(
        rf"\b{subject}\b(?:\s+[A-Za-z][A-Za-z0-9_-]*){{0,4}}\s+"
        r"(?:is|was|are|were|has|have|had|remains?|becomes?|gets?)\s+"
        r"(?P<value>[A-Za-z][A-Za-z0-9_-]*)",
        text,
        flags=re.I,
    )
    match = assignment or predicate
    if not match:
        return False
    value = match.group("value")
    return "_" in value or (value.isupper() and len(value) >= 2)


def _force_unknown_claim(normalized, raw):
    items = _items(normalized)
    words = [item for item in items if _is_word(item)]
    code_hint = raw_code_value(raw)
    raw_soft_dash_hint = any(dash in str(raw) for dash in SOFT_DASH_CHARS)
    for index, item in enumerate(items):
        if not _is_word(item) or not _is_subject_word(item):
            continue
        next_index = _skip_subject_context(items, index)
        if next_index >= len(items):
            continue
        effective_subject = item
        for word in items[index + 1:next_index]:
            if _is_word(word) and word not in {"label", "value", "state", "gate", "kind", "field"} and (
                word in CLAIM_SUBJECT_FILLERS or _is_subject_word(word)
            ):
                effective_subject = word
        if effective_subject not in CLAIM_STRICT_ASSIGNMENT_HEADS:
            continue
        context_items = items[index + 1:next_index]
        compound_soft_hint = (
            SOFT_HYPHEN in context_items
            and any(word in CLAIM_AUXILIARIES for word in context_items)
            and any(
                word in CLAIM_SUBJECT_FILLERS
                or _is_subject_word(word)
                or word in {"label", "value", "state", "gate", "kind", "field"}
                for word in context_items
            )
        )
        if items[next_index] in CLAIM_ASSIGNMENT_PUNCTUATION or items[next_index] == "-":
            rhs = [word for word in items[next_index + 1:] if _is_word(word)]
            if rhs and all(token.isalpha() for token in rhs[:2]) and (len(rhs) <= 2 or code_hint):
                return True
            continue
        if items[next_index] == SOFT_HYPHEN and next_index + 1 < len(items):
            rhs = [word for word in items[next_index + 1:] if _is_word(word)]
            if (
                (
                    (raw_soft_dash_hint and effective_subject in CLAIM_SOFT_STATUS_HEADS)
                    or (compound_soft_hint and effective_subject in CLAIM_STRICT_ASSIGNMENT_HEADS)
                )
                and rhs
                and all(token.isalpha() for token in rhs[:2])
                and (len(rhs) <= 2 or code_hint)
            ):
                return True
            continue
        if items[next_index] not in CLAIM_AUXILIARIES:
            continue
        cursor = next_index
        while cursor < len(items):
            if items[cursor] == SOFT_HYPHEN or items[cursor] in {",", ";"}:
                cursor += 1
                continue
            if not _is_word(items[cursor]):
                break
            if items[cursor] in CLAIM_AUXILIARIES or _is_modifier(items[cursor]):
                cursor += 1
                continue
            break
        if cursor < len(items) and _is_word(items[cursor]):
            value = items[cursor]
            if value not in CLAIM_VALUES and value not in CLAIM_COMPLETION_VERBS and value not in CLAIM_MARKERS:
                if code_hint or (len(words) <= 6 and value.isalpha()):
                    return True
    return False


def cell_claim(normalized, raw):
    if claim_text_normalized(normalized):
        if not code_only_cell(raw):
            return True
        words = [item for item in _items(normalized) if _is_word(item)]
        return len(words) == 1 or grammar_signal(normalized)
    return _force_unknown_claim(normalized, raw)


def table_prefix_claim(cells, width):
    selected = cells[:width]
    normalized = normalize_markdown(" ".join(selected))
    if width == 2 and code_only_cell(selected[0]) and not code_only_cell(selected[1]):
        return False
    if len(selected) >= 2:
        first_words = [item for item in _items(normalize_markdown(selected[0])) if _is_word(item)]
        second_words = [item for item in _items(normalize_markdown(selected[1])) if _is_word(item)]
        if first_words and first_words[-1] in CLAIM_SUBJECT_HEADS and len(second_words) <= 2:
            return True
    if width == 3 and code_only_cell(selected[0]) and not code_only_cell(selected[1]):
        second_normalized = normalize_markdown(selected[1])
        if second_normalized in CLAIM_AUXILIARIES and claim_text_normalized(normalize_markdown(selected[2])):
            return True
        return False
    if not claim_text_normalized(normalized):
        return False
    if all(code_only_cell(cell) for cell in selected) and not grammar_signal(normalized):
        words = [item for item in _items(normalized) if _is_word(item)]
        return len(words) == 1
    if (
        width == 3
        and all(code_only_cell(cell) for cell in selected[:2])
        and not code_only_cell(selected[2])
    ):
        return grammar_signal(normalize_markdown(" ".join(selected[:2]))) and claim_text_normalized(
            normalize_markdown(selected[2])
        )
    return True


def _is_subject_word(word):
    return word in CLAIM_SUBJECT_HEADS


def _is_modifier(word):
    return word in CLAIM_MODIFIERS or word.endswith("ly")


def _skip_soft_hyphen(items, index):
    while index < len(items) and items[index] == SOFT_HYPHEN:
        index += 1
    return index


def _skip_subject_context(items, index):
    index += 1
    while index < len(items):
        if _is_word(items[index]) and (
            items[index] in {"of", "for", "the", "a", "an", "this", "that", "its", "their"}
            or items[index] in CLAIM_SUBJECT_FILLERS
            or _is_subject_word(items[index])
        ):
            index += 1
            continue
        if (
            _is_word(items[index])
            and items[index] in {"label", "value", "state", "gate", "kind", "field"}
            and index + 1 < len(items)
        ):
            lookahead = _skip_soft_hyphen(items, index + 1)
            if lookahead < len(items) and (
                items[lookahead] in CLAIM_ASSIGNMENT_PUNCTUATION
                or items[lookahead] in CLAIM_AUXILIARIES
            ):
                index += 1
                continue
        if items[index] == SOFT_HYPHEN and index + 1 < len(items) and _is_word(items[index + 1]):
            next_word = items[index + 1]
            if next_word in {"label", "value", "state", "gate", "kind", "field"}:
                index += 1
                continue
            if (
                next_word in CLAIM_SUBJECT_FILLERS
                or _is_subject_word(next_word)
                or next_word in CLAIM_AUXILIARIES
            ):
                index += 2
                continue
        break
    return index


def _is_clause_break(item):
    return item in {".", ",", ";", "!", "?"} or item in CLAIM_BOUNDARIES


def _assignment_value(items, index, strict):
    index += 1
    while index < len(items) and _is_word(items[index]) and (
        _is_modifier(items[index]) or items[index] in {"the", "a", "an", "this", "that", "its", "their"}
    ):
        index += 1
    if index >= len(items) or not _is_word(items[index]):
        return False
    value = items[index]
    if _is_subject_word(value) or value in CLAIM_BOUNDARIES:
        return False
    if value in CLAIM_ASSIGNMENT_VALUES:
        return True
    if value in CLAIM_UNKNOWN_VALUE_HEADS:
        return True
    if strict and index + 1 < len(items):
        next_word = items[index + 1]
        if _is_word(next_word) and next_word in {"status", "state", "value"}:
            return True
    return False


def _predicate_claim(items, index, subject=None):
    index = _skip_soft_hyphen(items, index)
    saw_predicate = False
    while index < len(items):
        item = items[index]
        if item == SOFT_HYPHEN:
            index += 1
            continue
        if not _is_word(item):
            if item in CLAIM_ASSIGNMENT_PUNCTUATION or item == "-":
                return _assignment_value(items, index, True)
            if saw_predicate and item in {",", ";"}:
                index += 1
                continue
            if _is_clause_break(item):
                return False
            index += 1
            continue
        word = item
        if word in CLAIM_BOUNDARIES:
            if saw_predicate and word == "and" and index + 1 < len(items):
                following = items[index + 1]
                if _is_word(following) and (
                    _is_modifier(following)
                    or following in CLAIM_AUXILIARIES
                    or following in CLAIM_VALUES
                ):
                    index += 1
                    continue
            return False
        if not saw_predicate:
            if word in CLAIM_AUXILIARIES:
                saw_predicate = True
                index += 1
                continue
            if _is_modifier(word) and (index == 0 or items[index - 1] != SOFT_HYPHEN):
                saw_predicate = True
                index += 1
                continue
            if word in CLAIM_COMPLETION_VERBS or word in CLAIM_MARKERS:
                return True
            return False
        if _is_modifier(word) or word in CLAIM_AUXILIARIES:
            index += 1
            continue
        if word in CLAIM_VALUES or (
            word in CLAIM_STATUS_ONLY_VALUES
            and subject in CLAIM_STRICT_ASSIGNMENT_HEADS
        ):
            return True
        if word in CLAIM_COMPLETION_VERBS or word in CLAIM_MARKERS:
            return True
        return False
    return False


def _claim_from_items(items):
    words = [item for item in items if _is_word(item)]
    if len(words) == 1 and words[0] in BARE_CLAIM_VALUES:
        return True
    for index, item in enumerate(items):
        if not _is_word(item) or not _is_subject_word(item):
            continue
        next_index = _skip_subject_context(items, index)
        if next_index >= len(items):
            continue
        if (
            item in {"result", "outcome"}
            and next_index + 1 < len(items)
            and items[next_index] == "as"
            and items[next_index + 1] in NEGATIVE_POLICY_VALUES
        ):
            continue
        if items[next_index] in CLAIM_ASSIGNMENT_PUNCTUATION or items[next_index] == "-":
            if _assignment_value(
                items,
                next_index,
                item in CLAIM_STRICT_ASSIGNMENT_HEADS,
            ):
                return True
            continue
        if (
            _is_word(items[next_index])
            and items[next_index] in CLAIM_VALUES
            and "of" not in items[index + 1:next_index]
        ):
            if item in CLAIM_DIRECT_HEADS or item == "fix":
                return True
        if items[next_index] == SOFT_HYPHEN and next_index + 1 < len(items):
            soft_value = items[next_index + 1]
            if (
                _is_word(soft_value)
                and soft_value in CLAIM_ASSIGNMENT_VALUES
                and item in CLAIM_SOFT_STATUS_HEADS
            ):
                return True
        effective_subject = item
        for word in items[index + 1:next_index]:
            if _is_word(word) and word not in {"label", "value", "state", "gate", "kind", "field"} and (
                word in CLAIM_SUBJECT_FILLERS or _is_subject_word(word)
            ):
                effective_subject = word
        context_words = {
            word for word in items[index + 1:next_index] if _is_word(word)
        }
        if "field" in context_words:
            value_index = _skip_soft_hyphen(items, next_index)
            while value_index < len(items) and (
                _is_word(items[value_index])
                and (
                    items[value_index] in CLAIM_AUXILIARIES
                    or _is_modifier(items[value_index])
                )
            ):
                value_index += 1
            if value_index < len(items) and items[value_index] in NEGATIVE_POLICY_VALUES:
                continue
        if _predicate_claim(items, next_index, effective_subject):
            return True
    for index, item in enumerate(items[:-1]):
        if (
            _is_word(item)
            and item in CLAIM_VALUES
            and items[index + 1] in CLAIM_ASSIGNMENT_PUNCTUATION
        ):
            return True
    return False


def status_assignment_present(value):
    return _claim_from_items(_items(value))


def claim_text_normalized(normalized):
    return _claim_from_items(_items(normalized))


def claim_text(value):
    return claim_text_normalized(normalize_markdown(value))


class DocumentModel:
    def __init__(self, document):
        self.document = document
        self.raw_lines = document.splitlines()
        self.nodes = []
        for index, raw in enumerate(self.raw_lines):
            content = split_markdown_prefix(raw)
            cells = table_cells(content)
            normalized = normalize_markdown(content)
            normalized_cells = [normalize_markdown(cell) for cell in cells] if cells is not None else []
            self.nodes.append({
                "index": index,
                "raw": raw,
                "content": content,
                "cells": cells,
                "normalized": normalized,
                "normalized_cells": normalized_cells,
                "collapsed": collapse_normalized(normalized),
                "collapsed_cells": [collapse_normalized(cell) for cell in normalized_cells],
            })
        self.status_span = block_span("HANDOFF-STATUS", document)
        self.vocabulary_span = block_span("HANDOFF-VOCABULARY", document)
        self.preflight_span = block_span("HANDOFF-PREFLIGHT", document)
        self.status_rows = []
        self.vocabulary_rows = {}
        self.allowed_controlled = set()
        self.allowed_cells = set()
        self.canonical_row_indices = set()
        self.controlled_collapsed = {
            collapse(token): token for token in CONTROLLED_TOKENS
        }
        self.status_structure_error = None
        self.vocabulary_error = None
        self._parse_status_rows()
        self._parse_vocabulary()
        self._build_scan_text()
        self.checklist_entries = []
        self.block_checklist_entries = []
        self._parse_checklist()

    def _parse_status_rows(self):
        if self.status_span is None:
            return
        start, stop = self.status_span
        body = self.nodes[start:stop]
        expected_header = "| Field | Value |"
        expected_separator = "|---|---|"
        if (
            len(body) != 4
            or body[0]["raw"] != expected_header
            or body[1]["raw"] != expected_separator
        ):
            self.status_structure_error = "canonical status block must have exactly its header and two rows"
            return
        expected_fields = ("handoff_evidence", "product_fix")
        for offset, expected_field in enumerate(expected_fields, start=2):
            node = body[offset]
            cells = node["cells"]
            if cells is None or len(cells) != 2:
                self.status_structure_error = "canonical status row must have exactly two cells"
                continue
            field = exact_code_cell(cells[0])
            value = exact_code_cell(cells[1])
            if field != expected_field:
                self.status_structure_error = "canonical status fields must be in exact order"
            if field is None or value is None:
                self.status_structure_error = "canonical status cells must be exact code cells"
                continue
            self.status_rows.append({"line": node["index"], "field": field, "value": value})
            if field in STATUS_VALUES and value in STATUS_VALUES[field]:
                self.allowed_cells.update({(node["index"], 0), (node["index"], 1)})
                self.allowed_controlled.add((node["index"], 1, collapse(value)))
                self.canonical_row_indices.add(node["index"])

    def _parse_vocabulary(self):
        if self.vocabulary_span is None:
            self.vocabulary_error = "canonical vocabulary block is required"
            return
        start, stop = self.vocabulary_span
        body = self.nodes[start:stop]
        expected_lines = (
            "| Value | Meaning |",
            "|---|---|",
            "| `HANDOFF_EVIDENCE_VERIFIED` | The evidence handoff satisfies all four preflight requirements. It makes no product-implementation claim. |",
            "| `HANDOFF_UNVERIFIED` | Any required evidence, acceptance, compatibility/rollback, or authorization clause is missing, ambiguous, stale, or unsupported. This is the fail-closed value. |",
            "| `PRODUCT_FIX_UNIMPLEMENTED` | No product fix is implemented or verified by this lane. |",
        )
        if len(body) != len(expected_lines) or any(
            node["raw"] != expected for node, expected in zip(body, expected_lines)
        ):
            self.vocabulary_error = "canonical vocabulary block must contain the exact three controlled rows"
            return
        rows = {}
        for node, expected in zip(body[2:], expected_lines[2:]):
            cells = node["cells"]
            token = exact_code_cell(cells[0]) if cells is not None and len(cells) == 2 else None
            if token not in CONTROLLED_TOKENS:
                self.vocabulary_error = "canonical vocabulary row is not an exact controlled token"
                continue
            rows[token] = {"line": node["index"], "cell": 0}
            self.allowed_cells.add((node["index"], 0))
            self.allowed_controlled.add((node["index"], 0, collapse(token)))
            self.canonical_row_indices.add(node["index"])
        if not self.vocabulary_error and set(rows) != CONTROLLED_TOKENS:
            self.vocabulary_error = "canonical vocabulary table does not contain exactly the controlled tokens"
        self.vocabulary_rows = rows

    def _build_scan_text(self):
        scan_parts = []
        for node in self.nodes:
            if node["index"] in self.canonical_row_indices:
                scan_parts.append(" ")
            else:
                scan_parts.append(node["normalized"])
        self.scan_normalized = " ".join(scan_parts)
        self.scan_collapsed = collapse_normalized(self.scan_normalized)

    def _parse_checklist(self):
        for node in self.nodes:
            self.checklist_entries.extend(checklist_entries(node["content"]))
        if self.preflight_span is not None:
            start, stop = self.preflight_span
            for index in range(start, stop):
                self.block_checklist_entries.extend(checklist_entries(self.nodes[index]["content"]))

    def _controlled_hits(self, collapsed):
        hits = {
            token for key, token in self.controlled_collapsed.items()
            if key in collapsed
        }
        for entity, suffixes in CLAIM_ALIAS_GROUPS.items():
            for suffix in suffixes:
                key = collapse_normalized(entity + suffix)
                if key in collapsed:
                    hits.add(key)
        return hits

    def controlled_token_errors(self):
        if self._controlled_hits(self.scan_collapsed):
            return ["controlled status token or alias outside its parsed field or vocabulary row"]
        return []

    def status_assignment_errors(self):
        found = []
        for node in self.nodes:
            if node["index"] in self.canonical_row_indices:
                continue
            values = node["normalized_cells"] if node["cells"] is not None else [node["normalized"]]
            for cell_index, normalized in enumerate(values):
                if node["cells"] is not None and (node["index"], cell_index) in self.allowed_cells:
                    continue
                if cell_claim(normalized, node["cells"][cell_index] if node["cells"] is not None else node["raw"]):
                    found.append("status assignment outside the controlled machine fields")
            if node["cells"] is not None and len(node["cells"]) >= 2:
                if any(
                    table_prefix_claim(node["cells"], width)
                    for width in (2, 3)
                    if len(node["cells"]) >= width
                ):
                    found.append("status assignment outside the controlled machine fields")
        return found

    def claim_errors(self):
        found = []
        for node in self.nodes:
            if node["index"] in self.canonical_row_indices:
                continue
            if node["cells"] is None:
                if cell_claim(node["normalized"], node["raw"]):
                    found.append("verification status claim")
                continue
            for cell_index, normalized in enumerate(node["normalized_cells"]):
                if (node["index"], cell_index) in self.allowed_cells:
                    continue
                if cell_claim(normalized, node["cells"][cell_index]):
                    found.append("verification claim in table")
            if any(
                table_prefix_claim(node["cells"], width)
                for width in (2, 3)
                if len(node["cells"]) >= width
            ):
                found.append("verification claim in table")
        return found


def validate(candidate):
    document = document_without_fixture(candidate)
    if document is None:
        errors = ["exactly one HANDOFF-VALIDATOR-FIXTURE block is required"]
        document = candidate
    else:
        errors = []
    model = DocumentModel(document)
    if model.status_span is None:
        errors.append("exactly one HANDOFF-STATUS block is required")
    else:
        rows = model.status_rows
        fields = [row["field"] for row in rows]
        if len(rows) != 2:
            errors.append("status block must contain exactly two rows")
        if len(fields) != len(set(fields)):
            errors.append("duplicate status field")
        if set(fields) != set(STATUS_VALUES):
            errors.append("status fields must be exactly handoff_evidence and product_fix")
        for row in rows:
            if row["field"] not in STATUS_VALUES:
                errors.append(f"unknown status field: {row['field']}")
            elif row["value"] not in STATUS_VALUES[row["field"]]:
                errors.append(f"value {row['value']} is not allowed for {row['field']}")

    if model.preflight_span is None:
        errors.append("exactly one HANDOFF-PREFLIGHT block is required")
    else:
        all_entries = model.checklist_entries
        block_entries = model.block_checklist_entries
        if len(all_entries) != len(CHECKLIST_ANCHORS):
            errors.append("document must contain exactly four checklist marker entries")
        if block_entries != all_entries:
            errors.append("checklist markers must exist only in the bounded preflight block")
        for marker in CHECKLIST_ANCHORS:
            marker_entries = [entry for entry in all_entries if entry[1] == marker]
            if len(marker_entries) != 1:
                errors.append(f"checklist marker must appear exactly once: {marker}")
            for state, _, description in marker_entries:
                if state.lower() != "x":
                    errors.append(f"checklist marker is not checked: {marker}")
                for anchor in CHECKLIST_ANCHORS[marker]:
                    if anchor not in description:
                        errors.append(f"checklist anchor missing: {marker}:{anchor}")

    status = {row["field"]: row["value"] for row in model.status_rows}
    if status.get("handoff_evidence") == "HANDOFF_EVIDENCE_VERIFIED" and errors:
        errors.append("verified evidence status requires a complete preflight")
    if model.status_structure_error:
        errors.append(model.status_structure_error)
    if model.vocabulary_error:
        errors.append(model.vocabulary_error)
    errors.extend(model.status_assignment_errors())
    errors.extend(model.claim_errors())
    errors.extend(model.controlled_token_errors())
    return errors


current_errors = validate(text)
if current_errors:
    for error in current_errors:
        print(f"handoff preflight: FAIL: {error}")
    raise SystemExit(1)
print("handoff preflight: PASS")

positive = "VER" + "IFIED"
negative = "UN" + "VERIFIED"
status_begin_marker = "<!-- HANDOFF-" + "STATUS-BEGIN -->"
status_end_marker = "<!-- HANDOFF-" + "STATUS-END -->"
preflight_begin_marker = "<!-- HANDOFF-" + "PREFLIGHT-BEGIN -->"
preflight_end_marker = "<!-- HANDOFF-" + "PREFLIGHT-END -->"
fixture_begin_marker = "<!-- HANDOFF-" + "VALIDATOR-FIXTURE-BEGIN -->"
fixture_end_marker = "<!-- HANDOFF-" + "VALIDATOR-FIXTURE-END -->"
status_row_evidence = "| `handoff_evidence` | `HANDOFF_EVIDENCE_VERIFIED` |"
status_row_product = "| `product_fix` | `PRODUCT_FIX_UNIMPLEMENTED` |"
checked_marker = "- [" + "x" + "]"
unchecked_marker = "- [" + " " + "]"
marker_line = checked_marker + " `REDACTED_EVIDENCE` " + chr(0x2014) + " `## Evidence boundary and safety rules`, `## Protected evidence ledger`, and `## Artifact and claim ledger` provide bounded, redacted evidence and explicitly exclude bodies, keys, credentials, and private identifiers."
adversarial = [
    (
        "status-field-swap",
        text.replace(status_row_evidence, "| `handoff_evidence` | `PRODUCT_FIX_UNIMPLEMENTED` |", 1).replace(
            status_row_product, "| `product_fix` | `HANDOFF_EVIDENCE_VERIFIED` |", 1
        ),
    ),
    (
        "duplicate-conflicting-status-row",
        text.replace(
            status_row_product + "\n" + status_end_marker,
            status_row_product + "\n| `product_fix` | `HANDOFF_EVIDENCE_VERIFIED` |\n" + status_end_marker,
            1,
        ),
    ),
    (
        "unknown-status-value",
        text.replace(status_row_product, "| `product_fix` | `UNKNOWN_STATUS` |", 1),
    ),
    ("controlled-token-in-prose", text + "\nThe handoff evidence is HANDOFF_EVIDENCE_VERIFIED.\n"),
    ("controlled-token-in-table", text + "\n| status | HANDOFF_EVIDENCE_VERIFIED |\n"),
    ("controlled-token-in-product-field", text + "\n- Product fix: PRODUCT_FIX_UNIMPLEMENTED\n"),
    (
        "duplicate-status-block",
        text + "\n" + status_begin_marker + "\n| Field | Value |\n|---|---|\n| `handoff_evidence` | `PRODUCT_FIX_UNIMPLEMENTED` |\n| `product_fix` | `HANDOFF_EVIDENCE_VERIFIED` |\n" + status_end_marker + "\n",
    ),
    (
        "duplicate-preflight-block",
        text + "\n" + preflight_begin_marker + "\n" + unchecked_marker + " `REDACTED_EVIDENCE`\n" + preflight_end_marker + "\n",
    ),
    ("out-of-block-checklist-marker", text + "\n" + unchecked_marker + " `REDACTED_EVIDENCE`\n"),
    ("table-bare-label", text + f"\n| verification | {positive} |\n"),
    ("status-bare-label", text + f"\nStatus: {positive}\n"),
    ("blockquote-status-label", text + f"\n> Status: {positive}\n"),
    ("blockquote-prose-label", text + f"\n> The product fix is {positive}.\n"),
    ("ordered-list-status-label", text + f"\n1. Status: {positive}\n"),
    ("bullet-bare-label", text + f"\n- Verification: {positive}\n"),
    ("prose-bare-label", text + f"\nThe product fix is {positive}.\n"),
    ("product-fix-bare-label", text + f"\nProduct fix: {positive}\n"),
    ("lowercase-status-label", text + f"\nVerification status: {negative.lower()}\n"),
    ("passive-prose-label", text + "\nThe product fix has been verified.\n"),
    ("already-passive-prose-label", text + "\nThe product fix has already been verified.\n"),
    ("successful-passive-prose-label", text + "\nThe product fix has been successfully verified.\n"),
    ("successful-past-prose-label", text + "\nThe product fix was successfully verified.\n"),
    ("adverb-prose-label", text + "\nThis change is now verified.\n"),
    ("completed-prose-label", text + "\nVerification completed: verified\n"),
    ("completed-verb-prose-label", text + "\nThe product fix has completed: verified\n"),
    ("completed-subject-prose-label", text + "\nVerification has completed: verified\n"),
    ("nonfixture-code-bare-label", text + "\n```text\nStatus: " + positive + "\n```\n"),
    ("nonfixture-code-controlled-token", text + "\n```text\nHANDOFF_EVIDENCE_VERIFIED\n```\n"),
    (
        "duplicate-validator-fixture",
        text + "\n" + fixture_begin_marker + "\n" + fixture_end_marker + "\n",
    ),
    (
        "unchecked-checklist-marker",
        text.replace(checked_marker + " `REDACTED_EVIDENCE`", unchecked_marker + " `REDACTED_EVIDENCE`", 1),
    ),
    ("duplicate-checklist-marker", text.replace(marker_line, marker_line + "\n" + marker_line, 1)),
    (
        "contradictory-checklist-marker",
        text.replace(marker_line, marker_line + "\n" + marker_line.replace(checked_marker, unchecked_marker, 1), 1),
    ),
    ("missing-checklist-marker", text.replace(marker_line + "\n", "", 1)),
    (
        "missing-checklist-anchor",
        text.replace("`## Evidence boundary and safety rules`", "redacted boundary", 1),
    ),
]
for name, candidate in adversarial:
    if not validate(candidate):
        print(f"adversarial case: FAIL (accepted: {name})")
        raise SystemExit(1)
    print(f"adversarial case: PASS (rejected: {name})")

prefixes = ["", "> ", ">> ", "- ", "1. ", "> - ", "1) > ", "### "]
emphasis_marks = ["", "**", "__", "*", "_"]
claim_tokens = [positive, negative]
claim_templates = [
    "Status: {token}",
    "Verification status: {token}",
    "Product fix: {token}",
    "The product fix has already been {token}.",
    "The product fix has been successfully {token}.",
]
generated_cases = []
for prefix in prefixes:
    for mark in emphasis_marks:
        for token in claim_tokens:
            for template in claim_templates:
                claim = template.replace("{token}", token)
                rendered = prefix + (mark + claim + mark if mark else claim)
                generated_cases.append((f"generated-claim-{len(generated_cases):04d}", text + "\n" + rendered + "\n"))

auxiliaries = ["is", "was", "has been", "has already been", "has been successfully", "was successfully"]
adverbs = ["already", "successfully", "currently", "now"]
for auxiliary in auxiliaries:
    for adverb in adverbs:
        for token in claim_tokens:
            for prefix in ["", "> "]:
                for mark in ["", "**"]:
                    claim = f"The product fix {auxiliary} {adverb} {token}."
                    rendered = prefix + (mark + claim + mark if mark else claim)
                    generated_cases.append((f"generated-aux-{len(generated_cases):04d}", text + "\n" + rendered + "\n"))

for prefix in ["", "> ", "- "]:
    for mark in ["", "**", "__"]:
        for token in claim_tokens:
            rendered = prefix + "| status | " + mark + token + mark + " |"
            generated_cases.append((f"generated-table-{len(generated_cases):04d}", text + "\n" + rendered + "\n"))

for name, candidate in generated_cases:
    if not validate(candidate):
        print(f"generated claim matrix: FAIL (accepted: {name})")
        raise SystemExit(1)
print(f"generated claim matrix: {len(generated_cases)}/{len(generated_cases)} rejected")

# Coverage matrix for the normalized grammar, not a phrase blacklist.
grammar_subjects = [
    "product fix", "fix", "change", "release", "status", "verification", "result",
    "outcome", "label", "delivery", "implementation", "acceptance", "handoff", "evidence",
]
grammar_predicates = [
    "is {}", "was {}", "has been {}", "has recently been {}", "has been successfully {}",
    "must be {}", "will be {}", "could be {}", "is currently being {}", "was marked as {}",
    "is reported as {}", "has completed: {}", "was independently {}",
]
grammar_prefixes = ["", "> ", ">> ", "- ", "1. ", "### ", "> - ", "1) > "]
grammar_wrappers = ["", "**", "__", "*", "_", "~~", "<strong>", "<em>", "[link]", "<div>", "<!--"]
grammar_labels = [
    positive, negative, "~~" + positive + "~~", "<strong>" + positive + "</strong>",
    "[" + positive + "](#status)", "__" + positive + "__", "*" + positive + "*",
]


def wrap_claim(claim, wrapper):
    if wrapper == "[link]":
        return "[" + claim + "](#status)"
    if wrapper == "<!--":
        return "<!-- " + claim + " -->"
    if wrapper == "<div>":
        return "<div>" + claim + "</div>"
    if wrapper in {"<strong>", "<em>"}:
        return wrapper + claim + "</" + wrapper[1:] + ">"
    if wrapper == "~~":
        return "~~" + claim + "~~"
    return wrapper + claim + wrapper if wrapper else claim


generated_grammar_cases = []
for subject_index, subject in enumerate(grammar_subjects):
    for predicate_index, predicate in enumerate(grammar_predicates):
        label = grammar_labels[(subject_index + predicate_index) % len(grammar_labels)]
        claim = subject + " " + predicate.format(label) + "."
        rendered = grammar_prefixes[(subject_index + predicate_index) % len(grammar_prefixes)]
        rendered += wrap_claim(claim, grammar_wrappers[(subject_index + 2 * predicate_index) % len(grammar_wrappers)])
        generated_grammar_cases.append((
            f"generated-grammar-claim-{len(generated_grammar_cases):04d}",
            text + chr(10) + rendered + chr(10),
        ))

for subject_index, subject in enumerate(grammar_subjects):
    for separator_index, separator in enumerate([":", "=", "-", chr(0x2013), chr(0x2014), chr(0x2212), chr(0xFF1A), " : "]):
        value = [positive, negative, "confirmed", "complete", "unimplemented", "delivered"][separator_index % 6]
        rendered = subject + separator + value
        rendered = grammar_prefixes[(subject_index + separator_index) % len(grammar_prefixes)] + wrap_claim(
            rendered, grammar_wrappers[(subject_index + separator_index) % len(grammar_wrappers)]
        )
        generated_grammar_cases.append((
            f"generated-grammar-assignment-{len(generated_grammar_cases):04d}",
            text + chr(10) + rendered + chr(10),
        ))

controlled_alias_forms = [
    "handoff_evidence_verified", "HANDOFF-EVIDENCE-VERIFIED", "HANDOFF EVIDENCE VERIFIED",
    "HANDOFF_EVIDENCE_**VERIFIED**", "handoff_evidence_unverified", "HANDOFF-UNVERIFIED",
    "product_fix_unimplemented", "PRODUCT-FIX-UNIMPLEMENTED", "PRODUCT FIX UNIMPLEMENTED",
    "PRODUCT_FIX_**UNIMPLEMENTED**", "HANDOFF_EVIDENCE_CONFIRMED", "PRODUCT_FIX_ACCEPTED",
    "HANDOFF_EVIDENCE_" + chr(0x200B) + "VERIFIED", "RELEASE_VERIFIED", "STATUS-VERIFIED",
    "DELIVERY-UNVERIFIED", "ACCEPTANCE_CONFIRMED", "EVIDENCE_APPROVED", "RESULT_COMPLETE",
    "OUTCOME_PASSED", "IMPLEMENTATION_UNVERIFIED", "CHANGE-ACCEPTED", "status_value-is-verified",
]
for alias_index, alias in enumerate(controlled_alias_forms):
    for wrapper_index, wrapper in enumerate(["", "**", "<code>", "[link]"]):
        rendered = grammar_prefixes[(alias_index + wrapper_index) % len(grammar_prefixes)] + wrap_claim(alias, wrapper)
        generated_grammar_cases.append((
            f"generated-grammar-alias-{len(generated_grammar_cases):04d}",
            text + chr(10) + rendered + chr(10),
        ))

for dash_index, dash in enumerate(["-", chr(0x2013), chr(0x2014), chr(0x2212)]):
    for space_index, spacing in enumerate(["", "\u00a0", "\u2003"]):
        rendered = "Status" + spacing + dash + spacing + positive
        generated_grammar_cases.append((
            f"generated-grammar-unicode-{len(generated_grammar_cases):04d}",
            text + chr(10) + rendered + chr(10),
        ))

for subject_index, subject in enumerate(grammar_subjects):
    for label_index, label in enumerate([positive, negative, "<strong>" + positive + "</strong>", "[" + positive + "](#status)"]):
        rendered = "| " + subject + " | " + label + " |"
        generated_grammar_cases.append((
            f"generated-grammar-table-{len(generated_grammar_cases):04d}",
            text + chr(10) + rendered + chr(10),
        ))

passive_subjects = [
    "product fix", "release", "status", "verification", "result", "acceptance", "handoff", "evidence",
]
passive_predicates = [
    "is {}", "was {}", "has been {}", "has already been {}", "has been successfully {}",
    "was independently {}", "is currently being {}", "was marked as {}", "is reported as {}",
    "has completed: {}",
]
passive_values = ["confirmed", "complete", "completed", "passed", "accepted", "proven", "unimplemented"]
for subject_index, subject in enumerate(passive_subjects):
    for predicate_index, predicate in enumerate(passive_predicates):
        for value_index, value in enumerate(passive_values):
            claim = subject + " " + predicate.format(value) + "."
            rendered = grammar_prefixes[(subject_index + predicate_index + value_index) % len(grammar_prefixes)]
            rendered += wrap_claim(
                claim,
                grammar_wrappers[(subject_index + 2 * predicate_index + value_index) % len(grammar_wrappers)],
            )
            generated_grammar_cases.append((
                f"generated-grammar-passive-{len(generated_grammar_cases):04d}",
                text + chr(10) + rendered + chr(10),
            ))

unknown_assignment_subjects = ["status", "release", "verification", "product fix", "handoff", "result"]
for subject_index, subject in enumerate(unknown_assignment_subjects):
    for separator_index, separator in enumerate([":", "=", "-", chr(0x2013), chr(0x2014), chr(0xFF1A)]):
        value = ["UNKNOWN_STATUS", "APPROVED", "REJECTED"][separator_index % 3]
        rendered = grammar_prefixes[(subject_index + separator_index) % len(grammar_prefixes)] + subject + separator + value
        generated_grammar_cases.append((
            f"generated-grammar-unknown-{len(generated_grammar_cases):04d}",
            text + chr(10) + rendered + chr(10),
        ))

for name, candidate in generated_grammar_cases:
    if not validate(candidate):
        print(f"generated grammar matrix: FAIL (accepted: {name})")
        raise SystemExit(1)
print(f"generated grammar matrix: {len(generated_grammar_cases)}/{len(generated_grammar_cases)} rejected")
vocabulary_evidence_row = next(
        line for line in text.splitlines()
        if line.startswith("| `HANDOFF_EVIDENCE_VERIFIED` |")
    )
vocabulary_product_row = next(
        line for line in text.splitlines()
        if line.startswith("| `PRODUCT_FIX_UNIMPLEMENTED` |")
    )
placement_cases = [
    ("canonical-status-row-inside", text, False),
    ("canonical-vocabulary-row-inside", text, False),
    ("canonical-status-row-outside", text + "\n" + status_row_evidence + "\n", True),
    ("canonical-vocabulary-row-outside", text + "\n" + vocabulary_evidence_row + "\n", True),
    ("canonical-product-row-outside", text + "\n" + vocabulary_product_row + "\n", True),
    ("fabricated-vocabulary-row-outside", text + "\n| `HANDOFF_EVIDENCE_VERIFIED` | fabricated |\n", True),
    ("controlled-token-outside-table", text + "\n| field | `HANDOFF_EVIDENCE_VERIFIED` |\n", True),
]

for name, candidate, should_reject in placement_cases:
    case_errors = validate(candidate)
    if should_reject and not case_errors:
        print(f"generated placement matrix: FAIL (accepted: {name})")
        raise SystemExit(1)
    if not should_reject and case_errors:
        print(f"generated placement matrix: FAIL (rejected canonical: {name})")
        raise SystemExit(1)
print(f"generated placement matrix: {len(placement_cases)}/{len(placement_cases)} expected")
fixture_label = text.replace(
    fixture_end_marker,
    "\n| fixture | " + positive + "\nHANDOFF_EVIDENCE_VERIFIED\n" + fixture_end_marker,
    1,
)
if not validate(fixture_label):
    print("fixture exemption: PASS (confined labels ignored)")
else:
    print("fixture exemption: FAIL (confined labels were rejected)")
    raise SystemExit(1)
print(f"adversarial cases: {len(adversarial)}/{len(adversarial)} rejected")
PY
```
<!-- HANDOFF-VALIDATOR-FIXTURE-END -->

**Adversarial cases and expected results (normative):** The command mutates only an in-memory copy. The first 34 rows are negative cases and must be rejected with a non-zero preflight result. The two canonical in-block placement rows are positive controls and must remain accepted; the remaining placement rows are negative and must be rejected. The generated claim, grammar, and placement matrices exercise prefixes, emphasis, wrappers, case and punctuation normalization, aliases, subjects, predicates, auxiliaries, adverbs, connectors, passive/completed forms, unknown code-shaped values, table forms, and parsed field/table locations; these fixtures are not status assignments.

| Case | In-memory mutation | Expected result |
|---|---|---|
| `status-field-swap` | Exchange the two controlled field values. | Reject: invalid field-specific status. |
| `duplicate-conflicting-status-row` | Add a second `product_fix` row with a different value. | Reject: duplicate field and row cardinality failure. |
| `unknown-status-value` | Replace the product status with an unknown token. | Reject: value is outside the field vocabulary. |
| `controlled-token-in-prose` | Put a controlled status token in ordinary prose. | Reject: token is outside its defined field. |
| `controlled-token-in-table` | Put a controlled status token in an ordinary table cell. | Reject: token is outside its defined field. |
| `controlled-token-in-product-field` | Put a controlled product token in an ordinary bullet. | Reject: token is outside its defined field. |
| `duplicate-status-block` | Append a second contradictory status block. | Reject: more than one status block. |
| `duplicate-preflight-block` | Append a second contradictory checklist block. | Reject: more than one preflight block. |
| `out-of-block-checklist-marker` | Add an unchecked checklist marker outside the bounded block. | Reject: checklist marker exists outside its owner block. |
| `nonfixture-code-bare-label` | Put a bare label in a non-fixture fenced block. | Reject: only the explicit validator fixture is exempt. |
| `nonfixture-code-controlled-token` | Put a controlled token in a non-fixture fenced block. | Reject: only the explicit validator fixture is exempt. |
| `duplicate-validator-fixture` | Add a second validator fixture delimiter pair. | Reject: fixture region is not unique. |
| `table-bare-label` | Put a bare positive token in a table value cell. | Reject: uncontrolled table label. |
| `status-bare-label` | Add a standalone status assignment with a bare positive token. | Reject: uncontrolled status label. |
| `bullet-bare-label` | Put a bare positive token after a bullet label. | Reject: uncontrolled bullet label. |
| `prose-bare-label` | State that the product fix is a bare positive token. | Reject: uncontrolled prose label. |
| `product-fix-bare-label` | Add a product-fix assignment with a bare positive token. | Reject: uncontrolled product-fix label. |
| `lowercase-status-label` | Add a lowercase negative status assignment. | Reject: uncontrolled lowercase label. |
| `passive-prose-label` | State a passive product-fix claim with the positive token. | Reject: passive prose status claim. |
| `adverb-prose-label` | State an adverb-qualified change claim with the positive token. | Reject: adverb-qualified prose status claim. |
| `completed-prose-label` | Describe a completed-form claim with a bare token. | Reject: completed-form status claim. |
| `already-passive-prose-label` | State a passive claim with an already connector. | Reject: connector-qualified status claim. |
| `successful-passive-prose-label` | State a passive claim with a success adverb. | Reject: adverb-qualified status claim. |
| `successful-past-prose-label` | State a past claim with a success adverb. | Reject: adverb-qualified status claim. |
| `completed-verb-prose-label` | Describe a completed-form claim about the product fix using a bare token. | Reject: completed-verb status claim. |
| `completed-subject-prose-label` | Describe a completed-form verification claim using a bare token. | Reject: completed-subject status claim. |
| `blockquote-status-label` | Put a status assignment behind a blockquote prefix. | Reject: normalized Markdown status label. |
| `blockquote-prose-label` | Put a prose claim behind a blockquote prefix. | Reject: normalized Markdown prose label. |
| `ordered-list-status-label` | Put a status assignment behind an ordered-list prefix. | Reject: normalized Markdown status label. |
| `unchecked-checklist-marker` | Change one required marker to unchecked. | Reject: marker is not checked. |
| `duplicate-checklist-marker` | Add a second checked copy of a required marker. | Reject: marker count is not exactly one. |
| `contradictory-checklist-marker` | Add an unchecked copy beside a checked required marker. | Reject: contradictory checklist state. |
| `missing-checklist-marker` | Remove one required marker. | Reject: checklist is incomplete. |
| `missing-checklist-anchor` | Remove an anchor from one marker description. | Reject: required evidence anchor is absent. |
| `generated-claim-matrix` | Generate prefix, emphasis, token, auxiliary, adverb, connector, and table variants. | Reject: 610/610 generated variants. |
| `generated-grammar-matrix` | Generate normalized claims over subjects, predicates, separators, wrappers, aliases, Unicode punctuation, passive/completed forms, unknown values, and table cells. | Reject: 1050/1050 generated grammar cases. |
| `canonical-status-row-inside` | Keep the exact status field/value rows in the single HANDOFF-STATUS block. | Accept: rows are in their parsed field locations. |
| `canonical-vocabulary-row-inside` | Keep the exact controlled-token rows in the canonical vocabulary table. | Accept: rows are in their parsed vocabulary locations. |
| `canonical-status-row-outside` | Append an exact status-shaped row outside HANDOFF-STATUS. | Reject: controlled token is outside its parsed field location. |
| `canonical-vocabulary-row-outside` | Append an exact vocabulary row outside the canonical table. | Reject: controlled token is outside its parsed vocabulary location. |
| `canonical-product-row-outside` | Append the exact product vocabulary row outside the canonical table. | Reject: controlled token is outside its parsed vocabulary location. |
| `fabricated-vocabulary-row-outside` | Append a vocabulary-shaped row with fabricated meaning outside the canonical table. | Reject: controlled token is outside its parsed vocabulary location. |
| `controlled-token-outside-table` | Put a controlled token in an ordinary table value cell. | Reject: controlled token is outside its parsed field or vocabulary location. |

Expected output includes one `handoff preflight: PASS` line, one adversarial-case PASS line for each of the 34 prior negative cases, the generated claim/grammar/placement count lines, a fixture-exemption PASS line, and a final aggregate negative-case count. No fixture is copied to disk.

If any item is missing, ambiguous, stale, or unsupported, set the evidence field to its fail-closed value and do not use any bare verification status. A handoff evidence status is not a product-fix status; the product-fix field remains unimplemented until the owning repository implements and independently verifies the work.

### Required stop-and-handoff sequence

`read-only investigation` → `isolated local test/verification` (if needed) → `redacted evidence and deterministic criteria recorded` → `handoff written to the owning repository` → **stop**. The owning repository performs implementation, review, compatibility/rollback checks, and independent verification. No step in that downstream sequence grants this lane authority to edit product code or operate a live system.

**This handoff is not implementation authorization.** It is a bounded request for the owning repository to implement and verify the listed work. A recipient must not infer permission to skip review, tests, compatibility checks, rollback planning, or deployment gates.

## Executive result: bounded eventual live round trip

The strongest supported conclusion is a **bounded eventual live round trip**, not proof of the complete delivery contract:

- The approved Windows candidate is installed at `C:\Users\SCM\.local\bin\scmessenger-cli.exe`.
- Candidate and installed binary SHA-256 values were observed as `454CC346811F30A51E4A42BFB7F51F7DAF458313F82AD7A5D2A8477DE1FAB061`.
- The installed node reported healthy `0.4.0`, Git `1bc78c8`, with the original Windows start arguments and preserved configuration.
- The Windows config was recorded as 492 bytes with baseline SHA-256 `BAECF6D152F39E8223CC1BD1B253A34AD7E0C094E7CBE7A609F6AA5EBFB27134`.
- Identity, contacts, storage, and the protected stopped-state backup were preserved. The backup manifest records 1,428 files / 532,961,353 bytes, zero hash errors, zero mismatches, and an empty error list.
- Android remained installed as `com.scmessenger.android`, version `0.4.0`, version code `15`; no reinstall or signer change occurred.
- One explicitly authorized correction was eventually delivered to the user-confirmed live Android peer. A later Android-originating `kind=text` record was observed in Windows history with `delivered=true`; its body was not printed or recorded.
- The exact route, receipt timing, retry history, and causal correlation for that later text record remain unproven. The retained redacted receipt analysis is a repeated `history_sync` observation, not proof of the later text record.

This result does **not** prove that transport acknowledgement, application receipt, history reconciliation, retry, terminal failure, or all Windows entry points obey one delivery-state contract. Those are the implementation work described below.

## Evidence boundary and safety rules

A healthy service response, a history record, a transport acknowledgement, an application delivery receipt, an external completion event, and a passing contract test are different results. This handoff keeps those distinctions explicit.

The historical runtime pass used read-only API, process, package, log, and protected-copy collection. No additional message was sent during this documentation pass. Historical commands in session evidence are not new actions and must not be rerun as a health check.

The following are prohibited in this handoff and in its evidence projections:

- message bodies or plaintext content;
- private keys, credentials, tokens, or full provider configuration;
- raw peer IDs, device IDs, message IDs, or other private identifiers;
- unredacted command lines or raw logs.

Only hashes, counts, timestamps, state/kind labels, lengths, and explicitly redacted metadata are admissible. A path to a protected raw artifact is not permission to print its contents.

## RCA facts preserved from the audit

1. The installation and preservation checks were successful for the approved candidate, but they do not validate the delivery-state design.
2. The current Android peer was alive enough to continue sending machine envelopes. That is evidence of peer activity, not proof that a particular text was delivered.
3. The first Windows send used a preserved historical identity target and remained pending after its single status check. The currently live peer was different; the raw identifiers were intentionally not printed.
4. The explicitly authorized correction to the live peer was eventually observed as delivered. A later Android-originating `kind=text` record was also eventually observed as delivered in Windows history. The exact route and timing were not reconstructed.
5. Repeated `history_sync` and `identity_sync` records are machine envelopes. They must never satisfy a real-text acceptance assertion.
6. Android `stored` is not proof of delivery. In the observed source regions it can mean transport acknowledgement while waiting for a receipt, or a pending outbox item in retry backoff.
7. The current Windows API comments and call sites treat swarm `Ok` as a “true delivery” signal and mark the outbox entry sent. That is the semantic mismatch to correct; it is not a runtime proof of recipient delivery.
8. `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924` is a separate AWS/Windows rollout evidence set. Its `t0` capture is `2026-09-24T02:12:21Z`, its final capture is `2026-09-24T02:17:36Z`, and stability samples include `2026-09-24T02:14:12Z` and `2026-09-24T02:15:29Z`. It must not be conflated with the Android receipt evidence.

## Canonical delivery-state decision (implementation required)

### Owner and model boundary

**Decision:** `scmessenger-core` is the sole owner of outbound delivery state. The persisted owner is the message/history record in `core/src/store/history.rs:17-30`, with transition logic exposed through one core service (implemented in the history/store layer and called by `IronCore`). `core/src/iron_core.rs` orchestrates the service; it must not contain a second policy.

The current models explain why this decision is needed:

- `core/src/store/history.rs:17-30` persists `MessageRecord` with only `delivered` and no canonical status. `new_sent` at `:42-55` starts an outbound record with `delivered=false`.
- `core/src/mobile_bridge.rs:3117-3138` defines a separate `MessageStatus { Queued, InCustody, Sent, Delivered }` and a separate `MessageRecord` projection.
- `core/src/api.udl:317-332` mirrors that four-value status and record shape for UniFFI.
- These are duplicated projections today; none is currently a complete authoritative transition owner.

Implement one core-owned canonical type, for example `DeliveryState`, and one event reducer/service. The exact names below are the contract for the implementation pass; they are not claims that the code already exists.

```text
DeliveryState = Queued | InCustody | TransportAcked | ReceiptAcked | Delivered | Failed
DeliveryEvent = Prepared | CustodyEstablished | TransportAcked
              | ReceiptAcked | Reconciled | TerminalFailure
```

Semantics:

- `Queued`: the encrypted message and outbound record are durable, but no transport custody is proven.
- `InCustody`: a relay/transport has accepted durable custody. A buffer enqueue alone is not custody.
- `TransportAcked`: the transport operation returned success. This is not an application delivery receipt.
- `ReceiptAcked`: a valid receipt for the exact message has been authenticated and correlated, but history/outbox reconciliation has not yet committed.
- `Delivered`: canonical receipt application and history/outbox reconciliation have committed successfully.
- `Failed`: an explicit terminal policy has rejected the message. It is not a synonym for “retry later” or “transport failed once.”

The legacy `delivered` boolean remains readable during migration, but it is derived from canonical state (`true` only for `Delivered`) and is never an independent transition authority. The existing `mark_message_sent` outbox/drift removal operation must be split or renamed: transport acknowledgement may record state but must not clear the pending delivery record; only receipt reconciliation may clear it.

### Exact transition table

The reducer must be monotonic for delivery progress and idempotent for repeated events. A transition is committed only after its durable state and required side effects succeed.

| Event and guard | Legal from | Legal to | Required effect and failure behavior |
|---|---|---|---|
| `Prepared`: persist a new outbound record | no prior record | `Queued` | Persist the canonical state before dispatch. A later failed prepare creates no history record. |
| `CustodyEstablished`: durable relay/transport custody is recorded | `Queued` | `InCustody` | Record route/custody metadata. A local queue write or BLE notification completion alone stays `Queued`. |
| `TransportAcked`: transport returns success | `Queued` or `InCustody` | `TransportAcked` | Record the transport event and keep the outbox/pending entry. Never set `delivered=true` and never call the receipt-clearing operation. |
| `ReceiptAcked`: authenticated application receipt matches the message ID and expected sender | `TransportAcked` | `ReceiptAcked` | Correlate the receipt before changing state. Uncorrelated, malformed, wrong-peer, or machine-envelope traffic leaves the current state unchanged and pending. A legacy path that lacks a transport event must first record `TransportAcked`, not skip the event. |
| `Reconciled`: history update and outbox/pending cleanup commit | `ReceiptAcked` | `Delivered` | Atomically or idempotently mark history delivered and clear the matching pending ownership. If reconciliation fails, remain `ReceiptAcked` and retry reconciliation. |
| `TerminalFailure`: explicit terminal policy rejects the send | `Queued`, `InCustody`, or `TransportAcked` | `Failed` | Record a redacted terminal reason/code and stop automatic retry. A timeout, one transport error, or age counter alone must not trigger this transition. `ReceiptAcked` and `Delivered` cannot transition to `Failed`. |
| transient transport/retry failure | any nonterminal pre-receipt state | same state | Keep the record pending and apply the documented backoff. This is not a downgrade. |
| duplicate receipt/retry event | any state | same state | Return an idempotent result; do not clear a different message, regress progress, or emit a second delivery. |
| inbound message initialization | no prior record | `Delivered` | Incoming records are delivered from the receiver’s perspective; this is initialization, not an outbound shortcut. |

The following are explicitly illegal: `Queued -> Delivered` without a correlated receipt; `TransportAcked -> Delivered` on transport `Ok` alone; `ReceiptAcked -> Failed`; any `Delivered ->` non-delivered state; and accepting a receipt whose message ID, sender identity, or message kind does not match the pending record. A duplicate receipt after `Delivered` is a no-op, not a regression.

### Compatibility mapping

Keep the existing four-value `MessageStatus` in `core/src/mobile_bridge.rs` and `core/src/api.udl` as a compatibility projection. Add an additive canonical `delivery_state` field to the persisted/FFI record (and an additive terminal-failure field/code); regenerate bindings with the implementation change. Do not silently reinterpret old `Sent` as `Delivered`.

| Canonical `DeliveryState` | Legacy mobile/UDL `MessageStatus` projection | Legacy `delivered` projection | Meaning for old clients |
|---|---|---|---|
| `Queued` | `Queued` | `false` | Durable pending; may still be dispatched. |
| `InCustody` | `InCustody` | `false` | Custody is known, receipt is not. |
| `TransportAcked` | `Sent` | `false` | Transport success only; **not** delivery. |
| `ReceiptAcked` | `Sent` | `false` | Lossy four-value projection; new clients must read `delivery_state`. |
| `Delivered` | `Delivered` | `true` | Canonical receipt and reconciliation committed. |
| `Failed` | `Queued` plus terminal-failure metadata | `false` | Conservative non-delivered projection; it must not be interpreted as permission to retry. New clients must read the failure field. |

Reverse/migration rules are additive and conservative:

- A legacy received record with `delivered=true`, or an outbound record with durable receipt provenance and `delivered=true`, maps to `Delivered`.
- An outbound legacy `delivered=true` with no receipt provenance is not allowed to manufacture delivery from the current swarm-`Ok` path; quarantine it as `TransportAcked`/pending until a receipt or an explicit operator reconciliation exists.
- `delivered=false` with no status maps to `Queued`; use `InCustody` only when a durable custody record proves it.
- Legacy `MessageStatus::Queued` maps to `Queued`, `InCustody` to `InCustody`, `Sent` to `TransportAcked` (not `Delivered`), and `Delivered` to `Delivered`.
- Add serde defaults for the new field and retain the old field. Migration must be idempotent, preserve message IDs and history, and must not delete or rewrite unrelated records.
- An uncorrelated receipt remains pending and must not clear an outbox, pending JSON, or drift-custody entry.

## Windows caller inventory and decisions

### Router ownership

The live control API is the Axum server in `cli/src/api.rs`, started by `cli/src/main.rs:2642` and `:4027` through `api::start_api_server` at `cli/src/api.rs:1599`. Its routes are registered at `cli/src/api.rs:1643-1644`:

- `POST /api/send` -> `handle_send_message` (`:849-921`);
- `GET /api/send/:message_id` -> `handle_get_send_status` (`:957-983`).

`cli/src/api_axum.rs:253-318` contains a second handler and `cli/src/api_axum.rs:723` registers a second `/api/send` route, but the current module graph does not include `api_axum` (`cli/src/lib.rs` exports only `api`; `cli/src/main.rs` declares only `mod api`). It is a duplicate/latent surface, not a second live router in this checkout. Do not infer that both routes were exercised.

### Caller table

| Caller/source | Current behavior | Decision | Required convergence |
|---|---|---|---|
| `cli/src/api.rs:849-921`, live `POST /api/send` | Persists a sent record, calls swarm, marks on `Ok`, returns `accepted`. | **Retain as the sole public compatibility adapter**, but remove direct delivery policy. | Resolve recipient, prepare once, call the core state service, return canonical state plus legacy fields. `Ok` means `TransportAcked`, never `Delivered`. |
| `cli/src/api.rs:957-983`, live status route at `:1644` | Status is derived only from `record.delivered`. | **Retain and upgrade**. | Read `delivery_state`; derive legacy `status`/`delivered` through the mapping table. Never infer delivery from a pending outbox or transport result. |
| `cli/src/api.rs:279-318 send_message_via_api`, called by `cli/src/main.rs:4438` | CLI client posts to `/api/send` when the node is available. | **Retain as a thin client**. | Return/preserve the message ID and query canonical status; do not create a second state machine or label transport success “delivered.” |
| `cli/src/main.rs:3320-3370`, `UiCommand::Send` (direct mark at `:3355`) | BLE-or-swarm dispatch; swarm `Ok` calls `mark_message_sent`; UI emits `sent`. | **Deprecate duplicate UI policy; route through the shared service/API adapter**. | Use the same preflight, message ID, transition reducer, and receipt policy as the HTTP path. |
| `cli/src/main.rs:3515-3575`, daemon `ClientIntent::SendMessage` (direct mark at `:3553`) | Swarm `Ok` clears the outbox and emits `sent` over JSON-RPC. | **Deprecate duplicate JSON-RPC policy; route through the shared service/API adapter**. | Return `queued`/`transport_acked`/pending as appropriate; do not expose “sent” as product-level delivery. |
| `cli/src/main.rs:4435-4600`, `cmd_send_offline` (direct mark at `:4572`) | Uses the HTTP API when available, otherwise starts a temporary swarm and retries. | **Retain the user-facing command but deprecate direct mark-on-`Ok`**. | Use the core transition service in the fallback and preserve the same queue/retry semantics. The API and fallback must not diverge. |
| `cli/src/main.rs:3780-3800`, `flush_outbox_for_peer` | Sends queued messages and re-enqueues only on immediate error; no canonical status update. | **Retain as dispatcher only**. | Emit `TransportAcked` on success, retain pending ownership, and let receipt reconciliation clear it. |
| `core/src/iron_core.rs:3271-3290` and the surrounding outbox flush | Core egress comments distinguish dispatch from receipt, but legacy `mark_message_sent` remains the convergence name. | **Retain core ownership and refactor the API**. | All outbox/custody paths use the canonical reducer; no transport path invokes receipt cleanup. |
| `cli/src/main.rs:3244` and `:4395`, `MessageType::Receipt` handlers | Directly call `history.mark_delivered` after decoding a receipt. | **Retain as receipt adapters, not independent owners**. | Authenticate/correlate first, call `ReceiptAcked` then `Reconciled`; keep pending on failure or mismatch. |
| `cli/src/server.rs:1686-1700`, `ClientIntent::MarkMessageDelivered` | Lets a client directly set history delivered. | **Deprecate unrestricted mutation; route to a receipt-aware/admin-only operation**. | Do not allow a UI/operator assertion to satisfy the real-text contract without a validated receipt. |
| `cli/src/main.rs:4860-4875`, `cmd_mark_sent` | Directly removes an outbox/drift entry. | **Remove from normal delivery workflow; deprecate/rename as an operator repair tool**. | It must not be called “delivered” and cannot be used by send callers to clear pending state. |
| `cli/src/main.rs:5113-5128`, `cmd_history_mark_delivered` | Directly sets the legacy delivered flag. | **Restrict to explicit audited repair/test tooling or remove from the normal CLI**. | Require a validated receipt/reconciliation event; manual overrides are not acceptance evidence. |
| `cli/src/api_axum.rs:253-318`, route `:723` | Duplicate send handler has the same swarm-`Ok`/`accepted` conflation and returns no message ID. | **Remove from the production module graph; quarantine only as migration reference**. | Add a discovery/build assertion that no second `/api/send` router is registered. |

The transport-only calls in `cli/src/main.rs:3033`, `:3129`, `:3200`, `:4317`, `:4360`, and `:4380` are relay, receipt, or auto-reply transport operations, not additional user-send policies. They still must not call the legacy outbox-clearing operation or create a parallel status model. Their route/receipt events are inputs to the same core reducer.

## Protected evidence ledger

All paths below are evidence references, not instructions to reopen or publish raw contents. `capture_utc` is the collection time recorded by the artifact; it is not necessarily the time of the underlying event. A historical producer is marked **exact invocation not retained** when the session record preserved only a command family or script fragment. That limitation is deliberate; do not invent an exact command.

### Artifact and claim ledger

| Runtime claim | Protected artifact path(s) | UTC collection / event time | Producer command or provenance | Admissible conclusion |
|---|---|---|---|---|
| Candidate hash and staged binary identity | `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\t0\staged-binary-sha256.txt`; `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\pre-running-exe-sha256.txt` | `2026-09-24T02:12:21Z` (`t0/captured-at.txt`) | `Get-FileHash -Algorithm SHA256 -LiteralPath <candidate-exe>`; exact historical projection invocation not retained | Candidate/staged hash is recorded. This is separate Windows rollout evidence, not Android receipt evidence. |
| Health/version/process metadata | `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\t0\local-health.json`; `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\t0\local-version.json`; `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\t0\process.json`; `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\final\local-health.json`; `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\final\local-version.json`; `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\final\process.json` | t0 `2026-09-24T02:12:21Z`; final `2026-09-24T02:17:36Z` | `Invoke-RestMethod -Uri 'http://127.0.0.1:9876/health'`; `Invoke-RestMethod -Uri 'http://127.0.0.1:9876/version'`; `Get-CimInstance Win32_Process` projected to process name/path hash only | Supports the bounded health/version/process observation. It does not prove a text delivery contract. |
| Protected stopped-state backup integrity | `C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\full-backup\manifest.json` | manifest created `2026-09-24T07:05:35.991694+00:00` | Historical session seq 191 Python/PowerShell copy-and-hash script; retained fragment: `BACKUP_ROOT=<protected-backup-root> python - <<'PY'` with `shutil.copytree` and SHA-256 manifest generation. Full argv/script is not retained. | Supports 1,428 files / 532,961,353 bytes, zero hash errors, zero mismatches, and `errors=[]`; it does not prove runtime delivery. |
| Receipt evidence inventory v1 | `C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\manifest.json` | captured `2026-09-24T07:02:29.420410+00:00` | Historical session seq 191 Python copy/hash collector for Windows logs, Android logs, and protected copies; exact invocation not retained | Supports the protected inventory of 768 files / 562,946,802 bytes. |
| Installed binary path/hash comparison | `C:\Users\SCM\.local\bin\scmessenger-cli.exe`; candidate hash artifact `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\t0\staged-binary-sha256.txt` | Exact installed-path collection timestamp is not retained; candidate capture is `2026-09-24T02:12:21Z` | `Get-FileHash -Algorithm SHA256 -LiteralPath <installed-exe>` and the corresponding candidate hash command; exact historical comparison invocation is not retained | The matching installed/candidate hash is a preserved historical claim, but its installed-path timestamp is an evidence gap. |
| Receipt evidence inventory v2 | `C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\manifest-v2.json` | captured `2026-09-24T07:03:39.474883+00:00` | Same historical collector plus Android DB pulls and redacted analyses; exact invocation not retained | Supports the later protected inventory of 775 files / 565,441,822 bytes. |
| Redacted API/history metadata analysis | `C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\receipt-analysis-redacted.json` | captured `2026-09-24T07:02:29.420410+00:00`; latest record metadata `2026-09-24T07:01:20+00:00` | Historical Python read of `/api/history?limit=100`, envelope-kind parsing, body hashing, and redacted JSON emission; exact full invocation not retained | The retained latest record is a repeated `history_sync` with `delivered=true` and an 885-byte content length. It is **not** evidence of the later Android `kind=text` record. |
| Redacted log correlation | `C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\receipt-correlation-redacted.json` | captured `2026-09-24T07:03:39.474883+00:00` | Historical Python marker/line-hash scan over protected Windows and Android logs and DB byte scans; exact full invocation not retained | Supports aggregate log correlation: Windows log 20,420 lines; Android logcat 1,512,020 lines / 232,769,860 bytes; machine-envelope evidence present. It does not establish a route for the text reply. |
| Android logcat marker aggregate | `C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\android-logcat-receipt-correlation-redacted.json` | protected capture represented by manifest-v2 at `2026-09-24T07:03:39.474883+00:00` | `adb exec-out logcat -d` followed by the historical Python line-hash/marker/context scan; exact scan invocation not retained | Supports only the redacted aggregate: 13,596 bytes, 61,051 interesting lines, `history_sync=456`, `delivery=2400`, `received=32834`, `response=25142`. No body is admissible. |
| Android package/process/DB observations | `C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\manifest-v2.json` and the protected Android copies below | capture `2026-09-24T07:03:39.474883+00:00` (collection time) | `adb shell dumpsys package com.scmessenger.android`; `adb shell pidof com.scmessenger.android`; `adb exec-out run-as com.scmessenger.android cat <protected-db-or-preference-path>`; exact historical argv and DB path are not retained | Supports the package/version/process observation only where represented by the manifest. Do not infer delivery from package or process state. |

Protected raw evidence paths (referenced, never printed) are:

```text
C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\android\logcat-full.txt
C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\android\scmessenger-mesh.log
C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\android\mesh_diagnostics.log*
C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\windows\node-stdout.log
C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\windows\node-stderr.log
C:\Users\SCM\Documents\GitHub\SCMessenger-Backups\windows-1bc78c85-20260924T065506Z\receipt-evidence-20260924T065644Z\windows\data-logs\*
```

The separate rollout timestamps are recorded in `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\t0\captured-at.txt`, `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\final\captured-at.txt`, and `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\windows-rollout-20260924\stability\sample-*`. They are not substitutes for the receipt-evidence timestamps above.

### Historical command family (documentation only; do not rerun as a health check)

These are the safe command families recovered from the historical pass. The original output was reduced to hashes/counts/kinds; the exact full invocations for the Python collectors are not recoverable from the retained session record.

```powershell
# API reads used by the historical pass; project only status/count/kind/hash fields.
Invoke-RestMethod -Uri 'http://127.0.0.1:9876/health'
Invoke-RestMethod -Uri 'http://127.0.0.1:9876/version'
Invoke-RestMethod -Uri 'http://127.0.0.1:9876/api/identity'
Invoke-RestMethod -Uri 'http://127.0.0.1:9876/api/contacts'
Invoke-RestMethod -Uri 'http://127.0.0.1:9876/api/peers'
Invoke-RestMethod -Uri 'http://127.0.0.1:9876/api/history?limit=100'
Invoke-RestMethod -Uri 'http://127.0.0.1:9876/api/discovery/status'

Get-FileHash -Algorithm SHA256 -LiteralPath <candidate-or-installed-exe>
Get-FileHash -Algorithm SHA256 -LiteralPath <preserved-config-file>
Get-CimInstance Win32_Process | Select-Object Name,ExecutablePath
adb shell dumpsys package com.scmessenger.android
adb shell pidof com.scmessenger.android
adb exec-out run-as com.scmessenger.android cat <protected-db-or-preference-path>
adb exec-out logcat -d
adb exec-out uiautomator dump /dev/tty
```

For any future reproduction, hash or discard message content and identifiers before writing an artifact. Never save raw API JSON, command lines, or UI dumps as a substitute for a redacted projection.

## Test inventory and fail-if-missing gate

### Existing relevant tests (coverage inventory, not a green claim)

These files/tests exist in the current checkout and are the closest existing coverage. They do not collectively satisfy the five required new contract tests.

| File | Exact relevant symbols / scope | Gap to close |
|---|---|---|
| `core/tests/integration_ironcore_roundtrip.rs` | `test_two_node_message_roundtrip`, `test_wrong_recipient_cannot_decrypt`, `test_envelope_signature_verification`, `test_duplicate_delivery_rejected`, `test_multiple_messages_roundtrip`, `test_self_message_roundtrip`, `test_empty_payload_roundtrip`, `test_receipt_roundtrip_flips_state`, `test_history_conversation_coalesces_pubkey_and_identity_flavors` | In-process crypto/receipt coverage; no explicit six-state reducer, stale live-peer preflight, or text-vs-machine acceptance gate. |
| `core/tests/integration_retry_lifecycle.rs` | `retry_attempt_state_persists_across_restart`, `undelivered_message_does_not_transition_to_terminal_drop`, `test_custody_ownership_mutual_exclusion`, `outbox_stops_retrying_when_route_is_store_and_carry`, `restored_route_sends_new_messages_via_outbox_without_disturbing_existing_custody`, `property_message_never_owned_by_both_outbox_and_drift` | Outbox/custody ownership is covered; the canonical history state and receipt reconciliation contract are not. |
| `core/src/iron_core.rs` | `queued_transport_send_remains_in_outbox_until_receipt`, `swarm_peer_registration_drives_is_connected_and_flush`, `swarm_peer_registration_is_multi_peer_safe`, `reconnect_flush_egresses_over_live_link` | Useful transport/outbox regressions; not a complete cross-platform state-machine test. |
| `core/tests/integration_receipt_convergence.rs` | `multi_forwarder_convergence_stops_duplicate_retry_and_purges_pending`, `convergence_purges_dispatching_duplicate_attempts` | Covers convergence cleanup, not the full status vocabulary or compatibility migration. |
| `core/tests/integration_core_ffi_node1.rs` | `test_layer1_and_4_state_machine_transitions` and related FFI boundary tests | Does not establish the new `DeliveryState` owner or Windows caller convergence. |
| `cli/src/api.rs` unit tests | `api_send_resolves_contact_aliases_and_peer_id`, `api_send_refuses_blocked_contact_alias`, `api_send_ignores_stale_contact_public_key` | Exercises recipient resolution and stale public-key shape; does not prove current live-peer triad preflight before persistence. |
| `cli/tests/integration_message_requests.rs` | `message_request_lifecycle_accept`, `message_request_lifecycle_reject`, `existing_cli_contact_message_is_not_a_pending_request`, `accept_unknown_request_id_fails` | Request/contact behavior only; no delivery-state contract. |
| `android/app/src/test/java/com/scmessenger/android/test/ReceiptUnificationTest.kt` | Receipt struct/status round trips, inbound dedup, delivered receipt processing, duplicate receipt handling, concurrent receipt coalescing, encode-failure retry | Strong Android receipt coverage; package path and the separate `data/ReceiptUnificationTest.kt` copy must be consolidated/deduplicated. |
| `android/app/src/test/java/com/scmessenger/android/data/ReceiptUnificationTest.kt` | Duplicate `ReceiptUnificationTest` class in the `data` package | Same receipt theme in a second location; do not count it twice as independent coverage. |
| `android/app/src/test/java/com/scmessenger/android/test/ReceiptWindowTest.kt` | `testReceiptAckTimeoutConstant`, `testReceiptTimeoutDoesNotDowngradeToFailed`, `testNoDowngradeRuleProtectsAckedMessages`, adaptive receipt waits, age ceiling, and terminal-failure cases | Useful no-downgrade behavior; must be mapped to the canonical reducer rather than a parallel state model. |
| `android/app/src/test/java/com/scmessenger/android/test/DeliveryStateMapperTest.kt` | `maps delivered flag to delivered state`, `maps missing pending snapshot to pending state`, future retry, due retry, exhausted, and delivered-precedence cases | Tests the UI projection, not persisted canonical transitions. |
| `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt` | Mesh participation, Wi-Fi/BLE fallback, acked-without-receipt age policy, transport parsing, and related repository tests | Does not prove a matching application receipt or a Windows text record. |

### Required new tests

The implementation must add these exact, stable test names. A name in a comment or a zero-match Cargo filter is not a test.

| Required name | Target owner | Required assertion |
|---|---|---|
| `delivery_state_transition_test` | `scmessenger-core` reducer/history tests | Exercise every legal transition, every illegal regression, transient failure behavior, terminal-failure boundaries, and duplicate receipt idempotence. |
| `stale_recipient_preflight_test` | Core plus `cli/src/api.rs` recipient adapter | A resolvable historical identity that does not match the currently verified live peer triad fails before history persistence or dispatch; no fallback peer is selected. |
| `machine_envelope_rejection_test` | Core/Android message-kind filter | `history_sync`, `identity_sync`, and other machine kinds cannot satisfy a real-text acceptance assertion or create a text delivery receipt. |
| `real_text_roundtrip_test` | Approved cross-platform test transport/fixture | Send one unique text token, require the Android-side text send, a matching application receipt, and a Windows-side `kind=text` record with canonical `Delivered`; machine envelopes alone fail the test. |
| `transport_ack_without_receipt_test` | Core plus Android outbox/receipt-window tests | Transport success without a matching receipt remains non-delivered, retains pending ownership, and follows the documented retry/terminal policy without downgrade. |

Suggested target files for the implementation pass are `core/tests/delivery_state_contract.rs`, `core/tests/recipient_preflight.rs`, `core/tests/text_vs_machine_envelope.rs`, and a focused Android transport-ack test. These names are a plan; no test files were changed in this pass.

### Discovery assertion that cannot pass with zero matches

Run this after the implementation adds tests. It searches only source/test roots, requires a Rust `#[test]`/`#[tokio::test]` or Kotlin `@Test` declaration, and exits non-zero if any required name has zero matches. The current checkout is expected to fail this gate because the five names are not yet present; that failure is intentional and must not be converted into a pass.

```python
from pathlib import Path
import re
import sys

required = [
    "delivery_state_transition_test",
    "stale_recipient_preflight_test",
    "machine_envelope_rejection_test",
    "real_text_roundtrip_test",
    "transport_ack_without_receipt_test",
]
files = [
    p
    for root in (Path("core"), Path("cli"), Path("android"))
    if root.exists()
    for p in root.rglob("*")
    if p.is_file() and p.suffix in {".rs", ".kt"}
]
missing = []
for name in required:
    hits = []
    escaped = re.escape(name)
    declaration = re.compile(
        rf"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+{escaped}\s*\(|"
        rf"^\s*fun\s+`?{escaped}`?\s*\("
    )
    for path in files:
        text = path.read_text(encoding="utf-8", errors="replace")
        for match in declaration.finditer(text):
            context = text[max(0, match.start() - 300):match.start()]
            if re.search(r"#\[(?:tokio::)?test\b", context) or re.search(r"@Test\b", context):
                hits.append(str(path))
    print(f"{name}: {len(hits)} test declaration(s)")
    if not hits:
        missing.append(name)
if missing:
    print("missing required tests: " + ", ".join(missing), file=sys.stderr)
    sys.exit(1)
```

Do not use a bare `cargo test <filter>` or Gradle filter as the discovery gate: those commands can report success after matching zero tests. The exact target commands to run **after** discovery passes are:

```text
cargo test -p scmessenger-core --test integration_ironcore_roundtrip test_receipt_roundtrip_flips_state -- --exact
cargo test -p scmessenger-core --test integration_retry_lifecycle restored_route_sends_new_messages_via_outbox_without_disturbing_existing_custody -- --exact
cargo test -p scmessenger-core --lib queued_transport_send_remains_in_outbox_until_receipt -- --exact
cargo test -p scmessenger-cli --lib api_send_ignores_stale_contact_public_key -- --exact
cd android && .\\gradlew.bat :app:testDebugUnitTest --tests com.scmessenger.android.test.ReceiptUnificationTest --tests com.scmessenger.android.test.ReceiptWindowTest --tests com.scmessenger.android.test.DeliveryStateMapperTest
```

`scmessenger-cli` has both a library and a binary, and its `Cargo.toml` sets the binary target's `test = false`. Therefore `cargo test -p scmessenger-cli --lib` does **not** run bin-only tests in `cli/src/main.rs`; the three `main.rs` caller paths require an extracted library/integration test seam or an explicitly verified binary test target. Do not claim library coverage for them without that target.

No product acceptance test, live send, install, or device test was run for this documentation pass.

## Rollback, migration, and acceptance boundaries

- Make the canonical state addition additive at the persistence and FFI boundaries. Retain old records and old enum values; do not delete history, outbox, drift custody, ledger, or contacts during migration.
- Make migration idempotent and preserve the original message ID. A record without the new field must deserialize with a conservative state; unknown or malformed states must remain pending and be reported, never promoted to `Delivered`.
- If a receipt cannot be correlated by message ID and verified sender, keep the pending entry and report a redacted `receipt_pending`/equivalent condition.
- A failed recipient preflight must return a stable non-secret code such as `recipient_identity_ambiguous` or `recipient_not_current` before creating an outbound history record; it must not silently select another contact.
- Rollback is a code rollback plus forward-compatible reads. A downgraded binary must not reinterpret `TransportAcked` or `ReceiptAcked` as `Delivered`.
- Require a read-only migration rehearsal and backup verification before any production migration. Do not use the dirty checkout as an install/build source without an isolated boundary.
- Real-device acceptance, if later authorized, must use a user-operated approved fixture. Emit only hashes/counts/kinds/timestamps; never print bodies, keys, credentials, or private IDs.

## Remaining implementation work and exit criteria

This handoff is evidence-ready for the owning repository, not product-complete. Its evidence field uses the controlled evidence state; its product-fix field remains unimplemented. The unresolved work is:

1. Add the canonical state/event reducer and durable migration in `scmessenger-core`; make `MessageRecord` the persisted owner and project it through mobile/UDL compatibility fields.
2. Replace every direct `mark_message_sent`/`mark_delivered` policy in the caller inventory with the reducer, including outbox flush and receipt ingress. Keep outbox/pending ownership until `Reconciled`.
3. Remove the duplicate `api_axum` route from the production module graph and add a route-ownership regression check.
4. Add verified current-peer triad preflight and the five exact contract tests, with the fail-if-missing discovery gate above.
5. Regenerate and test UniFFI/Android compatibility projections; prove legacy `Sent` never becomes `Delivered` without a receipt.
6. On a separately authorized fixture, run a real text round trip and retain only redacted evidence. Do not promote machine-envelope traffic or Android `stored` state to delivery proof.

## Audit conclusion

The Windows installation/preservation outcome and a bounded eventual live round trip are supported by the historical evidence. The remaining product risk is the ambiguous delivery-state ownership and stale-recipient preflight, not the installer. Until the canonical reducer, caller convergence, compatibility mapping, and exact regression tests are implemented and independently verified, this path must not be described as production-grade or as proof of the full delivery contract.

**Provenance correction (2026-09-24):** The prior footer claiming pre-edit SHA-256 `B834AE52E46502AA577103BADA039FF1E3A188328786888DBDC13DAF3A525408` (19,974 bytes, 200 lines) was stale and is not used as evidence. Immediately before this repair, the handoff was rechecked at SHA-256 `da847cb1805acd4b71796c672621fa88fca3e2eec15f025d7c061338a64632d1` (44,148 bytes, 363 lines). The final hash is reported outside this file because embedding a file's hash would change the bytes being hashed.
