"""Black-box cost regression for §FS-rules.5.3, ported from grund.53.

Run with an optimized grund executable as the sole argument. The Python gate
driver builds it; compilation and corpus generation are never timed. Fixtures
are removed on success or failure and always live under ~/ag/tmp.
"""

import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path


CONDITIONS = """restricted-class trusted-invoke private-member-access type-confusion
memory-corruption arbitrary-deserialization code-execution file-read file-write
network-connect info-disclosure denial-of-service auth-bypass cert-bypass
crypto-weakness timing-leak sandbox-escape privilege-escalation resource-exhaustion
injection path-traversal xxe ssrf race integer-overflow log-forging""".split()
BOUNDARIES = ["untrusted-code", "untrusted-data", "network-peer", "local-host"]
RULES = {
    "boundary": "The boundary chapter of each CVE must cite exactly one BOUNDARY.",
    "condition": "Each CVE must have exactly one condition chapter.",
    "establishes": "The establishes chapter of each CVE must cite at least one COND.",
    "no-cve-edges": "Each CVE must not cite any CVE.",
    "repro": "Each CVE must have at most 1 repro chapter.",
    "requires": "Each CVE must have at most 1 requires chapter.",
    "site": "Each CVE must have exactly one site chapter.",
}
CONFIG = '''project_name = "scaling-fixture"
[id]
named_sections = true
[scan]
include = ["cves", "boundaries.md", "conditions.md"{rule_scan}]
[[kinds]]
kind = "CVE"
folder = "cves"
index = false
format = "{{kind}}-{{slug}}"
value_chapter = "site"
[[kinds]]
kind = "COND"
file = "conditions.md"
format = "{{kind}}-{{slug}}"
[[kinds]]
kind = "BOUNDARY"
file = "boundaries.md"
format = "{{kind}}-{{slug}}"
'''
RULE_CONFIG = '''[[kinds]]
kind = "RULE"
folder = "rules"
index = false
format = "{kind}-{slug}"
rules = true
'''
CHECK_TIMEOUT = 120


def make_tree(root, n, rules=True):
    root.mkdir()
    (root / "cves").mkdir()
    (root / "grund.toml").write_text(
        CONFIG.format(rule_scan=', "rules"' if rules else "")
        + (RULE_CONFIG if rules else ""), encoding="utf-8"
    )
    for kind, names, filename in [
        ("COND", CONDITIONS, "conditions.md"),
        ("BOUNDARY", BOUNDARIES, "boundaries.md"),
    ]:
        (root / filename).write_text(
            "# Shared vocabulary\n\n" + "".join(
                f"## {kind}-{name}: {name}\n\nOne paragraph describing the capability.\n\n"
                for name in names
            ), encoding="utf-8"
        )
    if rules:
        (root / "rules").mkdir()
        for name, sentence in RULES.items():
            (root / "rules" / f"RULE-{name}.md").write_text(
                f"# RULE-{name}: {sentence}\n\nSchema rule for the fixture corpus.\n",
                encoding="utf-8"
            )
    # The original corpus generator: same chapters, section depth, varying
    # targets, physical-site counts and seven rule sentences as grund.53.
    for i in range(n):
        rid = f"CVE-9{i // 10000:03d}-{i % 10000:04d}"
        requires = (f"\n## requires: Requires\n\n§COND-{CONDITIONS[(i + 7) % 26]}\n"
                    if i % 4 == 0 else "")
        body = f'''# {rid}: synthetic record {i}

A one-paragraph summary of synthetic record {i}: what the flaw is, where it
lives, and what the fix changed.

## site: Site
### site.method: Culprit method
#### site.method.1: com.example.p{i % 50}.Class{i}#method{i}(java.lang.String)
### site.fixed-in: First fixed release
#### site.fixed-in.1: {8 + i % 17}u{i % 300}

## boundary: Boundary

§BOUNDARY-{BOUNDARIES[i % 4]}

## condition: Condition

```text
attacker(input{i}) ∧ the parser reaches sink{i} ∧ ¬guard{i}(input{i})
```

## establishes: Establishes

§COND-{CONDITIONS[i % 26]}
{requires}
## repro: Reproducer

### repro.status: Status

#### repro.status.1: unrunnable-here: legacy-jdk-only — synthetic record
'''
        (root / "cves" / f"{rid}.md").write_text(body, encoding="utf-8")


def check(binary, root):
    start = time.perf_counter()
    result = subprocess.run(
        [str(binary), "check", "--ignore", "unused"], cwd=root,
        capture_output=True, text=True, timeout=CHECK_TIMEOUT
    )
    return time.perf_counter() - start, result


def measure(binary, root, n, rules):
    make_tree(root, n, rules)
    elapsed, result = check(binary, root)
    print(f"rules={rules!s:<5} N={n:<4} {elapsed:.6f}s exit={result.returncode}", flush=True)
    assert result.returncode == 0, result.stdout + result.stderr
    # Discard controls only after they have also exercised the complete check.
    shutil.rmtree(root)
    return elapsed


def regression(binary):
    scratch = Path.home() / "ag" / "tmp"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="grund-423-", dir=scratch) as temporary:
        root = Path(temporary)
        sanity = root / "live-rule"
        make_tree(sanity, 1)
        record = sanity / "cves" / "CVE-9000-0000.md"
        text = record.read_text(encoding="utf-8")
        start, end = text.index("## condition:"), text.index("## establishes:")
        record.write_text(text[:start] + text[end:], encoding="utf-8")
        _, broken = check(binary, sanity)
        assert broken.returncode == 1, broken.stdout + broken.stderr
        assert "has 0 condition chapters; RULE-condition requires exactly one" in broken.stdout
        print("live-rule: removed condition chapter -> exit=1, chapter-cardinality", flush=True)
        with_rules = {n: measure(binary, root / f"rules-{n}", n, True)
                      for n in (200, 400, 800)}
        for n in (200, 400, 800):
            measure(binary, root / f"control-{n}", n, False)
        ratio = with_rules[800] / with_rules[400]
        print(f"400->800 growth={ratio:.3f}x; 800={with_rules[800]:.6f}s", flush=True)
        assert not (ratio > 3.0 and with_rules[800] > 2.0), (
            "chapter-rule evaluation rescans independent records: "
            f"400->800 growth {ratio:.3f}x exceeds 3x AND "
            f"800-record check {with_rules[800]:.6f}s exceeds 2s"
        )


if __name__ == "__main__":
    regression(Path(sys.argv[1]).resolve())
