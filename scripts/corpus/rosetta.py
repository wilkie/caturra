#!/usr/bin/env python3
"""Fetch Rosetta Code's Java solutions into a local corpus.

Every task in Category:Java is read through the MediaWiki API, its Java
section cut out, and each code block that declares a `main` written as a
program of its own:

    ~/.cache/caturra-corpus/rosetta/<task>__<n>/<MainClass>.java
    ~/.cache/caturra-corpus/rosetta/<task>__<n>/meta.json

The corpus is other people's code (GFDL), so it lives OUTSIDE the repository;
only this script and the sweep that reads it are committed. Pages are cached,
so a second run fetches nothing it already has.

    python3 scripts/corpus/rosetta.py [--limit N] [--refresh]
"""
import argparse
import json
import os
import re
import sys
import time
import urllib.parse
import urllib.request

API = "https://rosettacode.org/w/api.php"
AGENT = "caturra-corpus/0.1 (Java compatibility testing; polite, cached)"
CACHE = os.path.expanduser("~/.cache/caturra-corpus/rosetta")
PAGES = os.path.join(CACHE, "_pages")


def api(params):
    url = API + "?" + urllib.parse.urlencode({**params, "format": "json"})
    request = urllib.request.Request(url, headers={"User-Agent": AGENT})
    for attempt in range(4):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return json.load(response)
        except Exception as error:  # a transient failure is retried, politely
            if attempt == 3:
                raise
            print(f"  retry after {error}", file=sys.stderr)
            time.sleep(5 * (attempt + 1))


def task_titles():
    """Every page in Category:Java (the tasks that have a Java solution)."""
    titles, cont = [], {}
    while True:
        data = api({
            "action": "query", "list": "categorymembers", "cmtitle": "Category:Java",
            "cmlimit": "500", "cmnamespace": "0", **cont,
        })
        titles += [m["title"] for m in data["query"]["categorymembers"]]
        if "continue" not in data:
            return titles
        cont = {"cmcontinue": data["continue"]["cmcontinue"]}
        time.sleep(1)


def slug(title):
    return re.sub(r"[^A-Za-z0-9]+", "_", title).strip("_")[:80]


def fetch_pages(titles, refresh):
    """Wikitext by title, from the cache or the API (50 titles a request)."""
    os.makedirs(PAGES, exist_ok=True)
    path = lambda t: os.path.join(PAGES, slug(t) + ".wiki")
    missing = [t for t in titles if refresh or not os.path.exists(path(t))]
    for start in range(0, len(missing), 50):
        batch = missing[start:start + 50]
        data = api({
            "action": "query", "prop": "revisions", "rvprop": "content",
            "rvslots": "main", "titles": "|".join(batch),
        })
        for page in data["query"]["pages"].values():
            if "revisions" not in page:
                continue
            text = page["revisions"][0]["slots"]["main"]["*"]
            open(path(page["title"]), "w", encoding="utf-8").write(text)
        print(f"  fetched {min(start + 50, len(missing))}/{len(missing)}", file=sys.stderr)
        time.sleep(1)
    return {t: open(path(t), encoding="utf-8").read() for t in titles if os.path.exists(path(t))}


def java_section(text):
    start = text.find("=={{header|Java}}==")
    if start < 0:
        return ""
    end = text.find("=={{header|", start + 10)
    return text[start:end if end > 0 else len(text)]


BLOCK = re.compile(
    r'<syntaxhighlight\s+lang="?java5?"?[^>]*>(.*?)</syntaxhighlight>|<lang java5?>(.*?)</lang>',
    re.S | re.I,
)


def top_level_classes(code):
    """(name, start, end) of each top-level type, found by brace depth with
    strings, chars and comments skipped."""
    out, depth, i, pending = [], 0, 0, None
    n = len(code)
    while i < n:
        c = code[i]
        if code.startswith("//", i):
            i = code.find("\n", i)
            i = n if i < 0 else i
            continue
        if code.startswith("/*", i):
            i = code.find("*/", i + 2)
            i = n if i < 0 else i + 2
            continue
        if c in "\"'":
            if code.startswith('"""', i):
                j = code.find('"""', i + 3)
                i = n if j < 0 else j + 3
                continue
            j = i + 1
            while j < n and code[j] != c:
                j += 2 if code[j] == "\\" else 1
            i = j + 1
            continue
        if depth == 0:
            m = re.match(r"\b(?:class|interface|enum|record)\s+([A-Za-z_]\w*)", code[i:i + 200])
            if m and (i == 0 or not (code[i - 1].isalnum() or code[i - 1] == "_")):
                pending = (m.group(1), i)
                i += m.end()
                continue
        if c == "{":
            if depth == 0 and pending:
                start_name = pending
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0 and pending:
                out.append((pending[0], pending[1], i + 1))
                pending = None
        i += 1
    return out


def programs(section):
    for index, match in enumerate(BLOCK.finditer(section)):
        code = (match.group(1) or match.group(2) or "").strip("\n")
        main = re.search(r"static\s+(?:public\s+)?void\s+main\s*\(", code)
        if not main:
            continue
        classes = top_level_classes(code)
        owner = next((name for name, s, e in classes if s <= main.start() < e), None)
        if owner is None:
            continue
        public = re.search(r"^\s*public\s+(?:final\s+|abstract\s+)*(?:class|interface|enum)\s+(\w+)", code, re.M)
        # javac demands a public top-level type live in a file of its name.
        file_class = public.group(1) if public else owner
        yield index, file_class, owner, code


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=0, help="only the first N tasks")
    parser.add_argument("--refresh", action="store_true", help="refetch cached pages")
    args = parser.parse_args()
    listing = os.path.join(CACHE, "_titles.json")
    os.makedirs(CACHE, exist_ok=True)
    if args.refresh or not os.path.exists(listing):
        titles = task_titles()
        json.dump(titles, open(listing, "w"))
    titles = json.load(open(listing))
    if args.limit:
        titles = titles[: args.limit]
    print(f"{len(titles)} tasks", file=sys.stderr)
    pages = fetch_pages(titles, args.refresh)
    written = 0
    for title, text in pages.items():
        for index, file_class, main_class, code in programs(java_section(text)):
            folder = os.path.join(CACHE, f"{slug(title)}__{index}")
            os.makedirs(folder, exist_ok=True)
            open(os.path.join(folder, file_class + ".java"), "w", encoding="utf-8").write(code + "\n")
            json.dump({
                "title": title, "block": index, "file": file_class + ".java", "main": main_class,
                "url": "https://rosettacode.org/wiki/" + urllib.parse.quote(title.replace(" ", "_")),
            }, open(os.path.join(folder, "meta.json"), "w"), indent=1)
            written += 1
    print(f"{written} programs written under {CACHE}", file=sys.stderr)


if __name__ == "__main__":
    main()
