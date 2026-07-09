#!/usr/bin/env python3
"""Generate the playground's level data from the Code.org CSA corpus.

Dev tool (not part of the build). Requires a local checkout of the corpus:
  - artifacts/ (git-ignored): per-level start/, validation/, solution/, properties.json
  - a code-dot-org checkout for scripts/scripts_json/levels metadata

Emits:
  - src/csa-index.ts         COMMITTED — light unit/level names for the pickers.
  - src/csa-units/unit-N.ts  COMMITTED — one code-split chunk per unit with that
    unit's start sources, JUnit validators (Test button), and data files. Loaded
    on demand so the initial bundle stays small.
  - src/csa-solutions.local.ts  GIT-IGNORED — the complete teacher solutions
    (Solve button). Never committed and never shipped: the hosted build runs with
    this file absent. We never read Code.org's encrypted_validation cipher.

Paths are configurable via env vars: ARTIFACTS_DIR, CODE_DOT_ORG_DIR.
"""
import json, os, glob, re, subprocess, tempfile
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ART = os.environ.get("ARTIFACTS_DIR", os.path.join(REPO, "artifacts"))
CDO = os.environ.get("CODE_DOT_ORG_DIR", os.path.join(REPO, "..", "code-dot-org"))
SCR = os.path.join(CDO, "dashboard/config/scripts")
LVL = os.path.join(CDO, "dashboard/config/levels/custom/javalab")
SJ = os.path.join(CDO, "dashboard/config/scripts_json")
SRC = os.path.join(REPO, "apps/playground/src")
UNITS = [
    ("csa1-2025", "CSA 2025 Unit 1"),
    ("csa2-2025", "CSA 2025 Unit 2"),
    ("csa3-2025", "CSA 2025 Unit 3"),
    ("csa4-2025", "CSA 2025 Unit 4"),
    ("csa5-2025", "CSA 2025 Unit 5"),
    ("csa6-2025", "CSA 2025 Unit 6"),
    ("csa7-2025", "CSA 2025 Unit 7"),
    ("csa8-2025", "CSA 2025 Unit 8"),
]


def verifier_bin():
    """Build (if needed) and locate the nbhcheck verifier example."""
    subprocess.run(
        ["cargo", "build", "-q", "-p", "caturra-vm", "--example", "nbhcheck"],
        cwd=REPO, check=True,
    )
    bins = [b for b in glob.glob(os.path.join(REPO, "target/debug/examples/nbhcheck-*"))
            if not b.endswith(".d")]
    if not bins:
        raise SystemExit("nbhcheck verifier not found after build")
    return max(bins, key=os.path.getmtime)


BIN = verifier_bin()


def slug(k):
    return re.sub(r"[^a-z0-9]+", "_", k.lower()).strip("_")


def bc_subs(k):
    p = os.path.join(SCR, slug(k) + ".bubble_choice")
    if not os.path.isfile(p):
        return None
    dn = None; subs = []; ins = False
    for line in open(p):
        line = line.strip()
        m = re.match(r"display_name '(.*)'$", line); dn = m.group(1) if m else dn
        if line == "sublevels":
            ins = True
        m = re.match(r"level '(.*)'$", line)
        if m and ins:
            subs.append(m.group(1))
    return dn, subs


def template_of(key):
    f = os.path.join(LVL, key + ".level")
    if not os.path.isfile(f):
        return None
    m = re.search(r'project_template_level_name"\s*:\s*"([^"]+)"', open(f, errors="ignore").read())
    return m.group(1) if m else None


def _has_main(path):
    try:
        return "static void main" in open(path, errors="ignore").read()
    except Exception:
        return False


def start_javas(key):
    # Merge template + own start files; own overrides by name. Skip a
    # differently-named template runner that would add a second main().
    own = {os.path.basename(j): j for j in sorted(glob.glob(os.path.join(ART, key, "start", "*.java")))}
    t = template_of(key)
    tmpl = {os.path.basename(j): j for j in sorted(glob.glob(os.path.join(ART, t, "start", "*.java")))} if t else {}
    own_has_main = any(_has_main(j) for j in own.values())
    merged = {}
    for name, path in tmpl.items():
        if own_has_main and name not in own and _has_main(path):
            continue
        merged[name] = path
    for name, path in own.items():
        merged[name] = path
    return [merged[k] for k in sorted(merged)]


DATA_EXTS = ("*.txt", "*.csv")


def start_datafiles(key):
    def gather(base):
        d = {}
        for pat in DATA_EXTS:
            for j in glob.glob(os.path.join(ART, base, "start", pat)):
                d[os.path.basename(j)] = j
        return d
    merged = gather(template_of(key)) if template_of(key) else {}
    merged.update(gather(key))
    return [(k, open(merged[k], errors="ignore").read()) for k in sorted(merged)]


def validation_files(key):
    # JUnit validators from the level's (and its template's) validation/ dir —
    # the plaintext of what Code.org ships as encrypted_validation.
    def gather(base):
        return {os.path.basename(j): j for j in sorted(glob.glob(os.path.join(ART, base, "validation", "*.java")))}
    merged = gather(template_of(key)) if template_of(key) else {}
    merged.update(gather(key))
    return [(k, open(merged[k], errors="ignore").read()) for k in sorted(merged)]


def solution_files(key):
    # The complete (teacher) solution — not committed; loaded by the Solve
    # button to exercise validators. Built as the start sources with the
    # student's answer files (those carrying a "TO DO" marker) replaced by the
    # solution versions. Some solution/ dirs strip untouched helper files
    # (e.g. ConstructorsHelper), so starting from start/ keeps them intact.
    def read(base):
        out = {}
        for j in sorted(glob.glob(os.path.join(ART, key, base, "*.java"))):
            out[os.path.basename(j)] = open(j, errors="ignore").read()
        t = template_of(key)
        if t:
            for j in sorted(glob.glob(os.path.join(ART, t, base, "*.java"))):
                out.setdefault(os.path.basename(j), open(j, errors="ignore").read())
        return out

    start = read("start")
    solution = read("solution")
    if not solution:
        return []
    # Prefer the solution (the completed answers), but if the solution's copy
    # of a file has FEWER method declarations than start's, it's a stripped
    # helper (some solution/ dirs drop untouched helpers like ConstructorsHelper
    # down to a stub) — keep start's full version. Bug-fix/syntax levels change
    # bodies, not method counts, so those still take the solution.
    def method_count(text):
        return len(re.findall(r"\b(?:public|private|protected)\b[^;=\n{]*\(", text))

    merged = dict(solution)
    for name, text in start.items():
        # `*Helper` files are teacher-provided (students don't edit them), so
        # start/ is authoritative — some solution/ copies are stale/buggy.
        if (
            name not in solution
            or name.endswith("Helper.java")
            or method_count(text) > method_count(solution[name])
        ):
            merged[name] = text
    return list(merged.items())


def grid_of(pj):
    m = pj.get("serialized_maze")
    if not m:
        return None
    return "\n".join(
        " ".join(f"{c.get('tileType',1)},{c.get('value',0)}" for c in row) for row in m
    ) + "\n"


def is_validator(text):
    return "org.junit" in text


# Reflection caturra cannot run yet: field get/set, method/constructor invocation.
UNSUPPORTED_REFLECT = (
    ".invoke(", ".newInstance(", "getDeclaredMethod", "getMethod(",
    "getMethods(", "getConstructor", "getDeclaredConstructor",
    "setAccessible", ".getDeclaredField(", ".getField(",
)


def split_files(files):
    validators = [(n, t) for n, t in files if is_validator(t)]
    rest = [(n, t) for n, t in files if not is_validator(t)]
    keep = []
    for name, text in rest:
        if "java.lang.reflect" in text and any(m in text for m in UNSUPPORTED_REFLECT):
            cls = name[:-5]
            if any(cls in ot for on, ot in rest if on != name):
                keep.append((name, text))
        else:
            keep.append((name, text))
    return keep, validators


def load_level(key):
    p = os.path.join(ART, key)
    if not os.path.isdir(p):
        return None
    try:
        pj = json.load(open(os.path.join(p, "properties.json")))
    except Exception:
        return None
    vm = pj.get("csa_view_mode")
    # The `playground` view mode isn't curriculum: its 68 artifacts are demos,
    # sandboxes, and project templates, and not one is referenced by a csa1-9
    # unit script. Nothing to recover there.
    if vm not in ("neighborhood", "console", "theater"):
        return None
    grid = grid_of(pj)
    if vm == "neighborhood" and not grid:
        return None
    javas = start_javas(key)
    if not javas:
        return None
    files = [(os.path.basename(j), open(j, errors="ignore").read()) for j in javas]
    student, validators = split_files(files)
    # Teacher-authored validators from the plaintext validation/ dir, merged
    # with any plaintext org.junit files in start/ (dedup by name; the
    # validation/ dir wins so we never compile the same test twice).
    vd = {}
    for n, t in validators:
        vd[n] = t
    for n, t in validation_files(key):
        vd[n] = t
    validators = list(vd.items())
    if not student:
        return None
    return vm, (grid or ""), student, validators, start_datafiles(key), solution_files(key)


def rename_main(files):
    out = []
    for name, text in files:
        if "static void main" in text:
            mm = re.search(r"class\s+(\w+)", text[: text.index("static void main")])
            if mm:
                text = re.sub(r"(\bclass\s+)" + re.escape(mm.group(1)) + r"\b", r"\1Main", text, count=1)
            out.append(("Main.java", text))
        else:
            out.append((name, text))
    out.sort(key=lambda f: f[0] != "Main.java")
    return out


# A caturra limitation (exclude) vs a real javac error the student fixes (include).
CATURRA_LIMITS = ("caturra", "is not supported", "are not supported", "not yet supported")


def is_real_error(msg):
    return not any(x in msg for x in CATURRA_LIMITS)


def verify(files, grid, validators=(), data=()):
    rec = (
        ["F:" + n + "\x1f" + t for n, t in list(files) + list(validators)]
        + ["D:" + n + "\x1f" + t for n, t in data]
        + (["G:" + grid] if grid else [])
    )
    with tempfile.NamedTemporaryFile("w", suffix=".rec", delete=False) as f:
        f.write("\x1e".join(rec)); path = f.name
    try:
        # Generous: an infinite-loop debugging level runs until the VM's
        # instruction budget trips (~16s), and we want that deterministic
        # "BUDGET" rather than a load-dependent "TIMEOUT" that silently drops it.
        return subprocess.run([BIN, path], capture_output=True, text=True, timeout=90).stdout.strip()
    except subprocess.TimeoutExpired:
        return "TIMEOUT"
    finally:
        os.unlink(path)


def letter_of(k):
    m = re.search(r"-L\d+([a-z])", k)
    return m.group(1) if m else None


def build_unit(script, unit_name):
    d = json.load(open(os.path.join(SJ, script + ".script_json")))
    lessons = {l["key"]: l["name"] for l in d["lessons"]}
    levels = []; seen = set()
    for sl in d["script_levels"]:
        lk = sl.get("seeding_key", {}).get("lesson.key", "")
        prog = sl.get("properties", {}).get("progression", "")
        for key in (sl.get("level_keys") or []):
            if key in seen:
                continue
            seen.add(key)
            bc = bc_subs(key)
            targets = [(letter_of(s), s, bc[0]) for s in bc[1]] if bc else [(None, key, None)]
            for let, tkey, dn in targets:
                got = load_level(tkey)
                if not got:
                    continue
                vm, grid, student, validators, data, solution = got
                if not any("static void main" in t for _, t in student):
                    continue
                student = rename_main(student)
                solution = rename_main(solution) if solution else []
                # Gate on the STUDENT program (validators are for the Test button,
                # not for inclusion). Include intentional errors the student fixes.
                st = verify(student, grid, data=data)
                if st.startswith("LOAD") or st == "TIMEOUT":
                    continue
                if st.startswith("RUN:") and "UncaughtException" not in st:
                    continue
                if st.startswith("CE:") and not is_real_error(st[3:].strip()):
                    continue
                nm = (dn or prog or tkey) + (f" ({let})" if let else "")
                levels.append({
                    "lesson": lessons.get(lk, lk), "name": nm, "view": vm, "grid": grid,
                    "files": student, "validation": validators, "data": data, "solution": solution,
                })
    return {"name": unit_name, "levels": disambiguate(levels)}


VIEW_LABEL = {"console": "Console", "theater": "Theater", "neighborhood": "Neighborhood"}


def disambiguate(levels):
    """Make (lesson, name) unique. A lesson often has two BubbleChoice sets with
    the same display name — one console, one theater — so name the view rather
    than emitting a meaningless "#1"/"#2". Same-view clashes still fall back to #N.
    """
    def group(ls):
        out = {}
        for l in ls:
            out.setdefault((l["lesson"], l["name"]), []).append(l)
        return out

    for clash in group(levels).values():
        if len(clash) > 1 and len({l["view"] for l in clash}) > 1:
            for l in clash:
                l["name"] = f"{l['name']} — {VIEW_LABEL[l['view']]}"
    for clash in group(levels).values():
        if len(clash) > 1:
            for i, l in enumerate(clash, 1):
                l["name"] = f"{l['name']} #{i}"
    return levels


# ----- AP FRQ unit: student stubs + a JUnit validator, verified to run -----
DEFAULT = {
    "void": "", "boolean": "return false;", "int": "return 0;", "long": "return 0;",
    "double": "return 0;", "float": "return 0;", "short": "return 0;", "byte": "return 0;",
    "char": "return ' ';",
}
STUB = re.compile(
    r"(?P<head>\b(?:public|private|protected)\s+(?:(?:static|final|abstract)\s+)*"
    r"(?P<ret>[A-Za-z_][\w.<>\[\], ]*?)\s+[A-Za-z_]\w*\s*\([^;{}]*\)\s*(?:throws[\w., ]+)?)"
    r"\{\s*(?:/\*.*?\*/\s*|//[^\n]*\n\s*)*\}",
    re.S,
)


def make_compilable(text):
    return STUB.sub(lambda m: m.group("head") + "{ " + DEFAULT.get(m.group("ret").strip(), "return null;") + " }", text)


def build_frq_unit():
    keys = sorted(
        k for k in (os.path.basename(p) for p in glob.glob(os.path.join(ART, "CSA-frq-*")))
        if re.search(r"-Part[A-Z]+$", k)
    )
    levels = []
    for key in keys:
        got = load_level(key)
        if not got:
            continue
        vm, grid, student, validators, data, solution = got
        if not validators:
            continue
        if any("static void main" in t for _, t in student):
            continue
        stubbed = [(n, make_compilable(t)) for n, t in student]
        st = verify(stubbed, "", validators, data)
        vt = int(re.search(r"VT(\d+)", st).group(1)) if "VT" in st else 0
        if vt <= 0:
            continue
        m = re.match(r"CSA-frq-(\d+)-(\w+?)-Part([A-Z]+)$", key)
        year, prob, part = (m.group(1), m.group(2), m.group(3)) if m else ("", "", key)
        prob = re.sub(r"(?<=[a-z])(?=[A-Z])", " ", prob)
        levels.append({
            "lesson": f"{prob} ({year})", "name": f"Part {part} — {vt} test(s)",
            "view": "console", "grid": "", "files": student, "validation": validators,
            "data": data, "solution": rename_main(solution) if solution else [],
        })
    return {"name": "AP FRQ Practice", "levels": levels}


units = [build_unit(s, n) for s, n in UNITS] + [build_frq_unit()]


def prettify(paths):
    """Format the emitted TypeScript with the repo's own prettier.

    Cheaper and far more faithful than hand-matching prettier's style (quoting,
    escaping, wrapping) in the emitter. These files used to be prettier-ignored,
    so nothing checked them and their style drifted with the generator; now they
    are ordinary lint-checked source and re-running this script is a no-op.
    """
    subprocess.run(
        ["pnpm", "exec", "prettier", "--write", "--log-level", "warn", *paths],
        cwd=REPO, check=True,
    )


def js(s):
    return "'" + s.replace("\\", "\\\\").replace("'", "\\'") + "'"


def emit_files(files, indent):
    lines = []
    for n, t in files:
        lines.append(f"{indent}{{ path: {js(n)}, text: {json.dumps(t)} }},")
    return lines


# csa-index.ts — COMMITTED, light. Unit/level names for the pickers only. Heavy
# per-level content (start sources, validators, data) lives in per-unit chunks
# under csa-units/, code-split and loaded on demand (see levels.ts). Solutions
# are dev-only (csa-solutions.local.ts) and never committed/shipped.
idx = [
    "// AUTO-GENERATED by apps/playground/scripts/generate-csa-levels.py — do not edit.",
    "// Lightweight unit/level index for the pickers. Heavy per-level content (start",
    "// sources, validators, data) lives in per-unit chunks under csa-units/, loaded",
    "// on demand; solutions are dev-only (csa-solutions.local.ts).",
    "",
    "export interface CsaLevelFile {", "  path: string;", "  text: string;", "}",
    "export interface CsaLevelData {", "  name: string;", "  lesson: string;",
    "  view: 'neighborhood' | 'console' | 'theater';", "  grid: string;", "  files: CsaLevelFile[];",
    "  validationFiles: CsaLevelFile[];", "  dataFiles: CsaLevelFile[];", "}",
    "export interface CsaLevelMeta {", "  name: string;", "  lesson: string;", "}",
    "export interface CsaUnitMeta {", "  name: string;", "  levels: CsaLevelMeta[];", "}",
    "export const CSA_UNITS: CsaUnitMeta[] = [",
]
for u in units:
    idx += ["  {", f"    name: {js(u['name'])},", "    levels: ["]
    for l in u["levels"]:
        idx += [f"      {{ name: {js(l['name'])}, lesson: {js(l['lesson'])} }},"]
    idx += ["    ],", "  },"]
idx += ["];", ""]
index_path = os.path.join(SRC, "csa-index.ts")
open(index_path, "w").write("\n".join(idx))
emitted = [index_path]

# csa-units/unit-N.ts — COMMITTED. One code-split chunk per unit with the full
# level content (start sources, validators, data) for that unit's levels.
os.makedirs(os.path.join(SRC, "csa-units"), exist_ok=True)
for ui, u in enumerate(units):
    chunk = [
        "// AUTO-GENERATED by apps/playground/scripts/generate-csa-levels.py — do not edit.",
        "// Per-unit level content (start sources, validators, data), code-split and",
        "// loaded on demand by levels.ts.",
        "",
        "import type { CsaLevelData } from '../csa-index.js';",
        "",
        "export const LEVELS: CsaLevelData[] = [",
    ]
    for l in u["levels"]:
        chunk += [
            "  {", f"    name: {js(l['name'])},", f"    lesson: {js(l['lesson'])},",
            f"    view: {js(l['view'])},", f"    grid: {json.dumps(l['grid'])},", "    files: [",
        ]
        chunk += emit_files(l["files"], "      ")
        chunk += ["    ],", "    validationFiles: ["]
        chunk += emit_files(l.get("validation", []), "      ")
        chunk += ["    ],", "    dataFiles: ["]
        chunk += emit_files(l.get("data", []), "      ")
        chunk += ["    ],", "  },"]
    chunk += ["];", ""]
    chunk_path = os.path.join(SRC, "csa-units", f"unit-{ui}.ts")
    open(chunk_path, "w").write("\n".join(chunk))
    emitted.append(chunk_path)

# Stale chunks from a previous run with more units would otherwise linger and
# still be lint-checked (and imported by a stale index).
for stale in sorted(glob.glob(os.path.join(SRC, "csa-units", "unit-*.ts"))):
    if stale not in emitted:
        os.remove(stale)
        print(f"removed stale chunk {os.path.basename(stale)}")

prettify(emitted)


def emit_map(name, key):
    lines = [f"export const {name}: Record<string, CsaLevelFile[]> = {{"]
    for ui, u in enumerate(units):
        for li, l in enumerate(u["levels"]):
            if not l.get(key):
                continue
            lines += [f"  '{ui}:{li}': ["]
            lines += emit_files(l[key], "    ")
            lines += ["  ],"]
    lines += ["};", ""]
    return lines


# csa-solutions.local.ts — GIT-IGNORED. Complete teacher solutions (Solve
# button). Never committed and never shipped to the hosted build: the deploy
# builds with this file absent, so solutions stay out of the public site.
sol = [
    "// AUTO-GENERATED, GIT-IGNORED — complete teacher solutions (Solve button),",
    "// generated locally from the artifacts. Never commit this file; the hosted",
    "// build omits it so solutions are never published.",
    "",
    "import type { CsaLevelFile } from './csa-index.js';",
    "",
]
sol += emit_map("SOLUTIONS", "solution")
open(os.path.join(SRC, "csa-solutions.local.ts"), "w").write("\n".join(sol))

for u in units:
    print(f"{u['name']}: {len(u['levels'])} levels, {sum(1 for l in u['levels'] if l['validation'])} with validators")
print("total", sum(len(u["levels"]) for u in units))
