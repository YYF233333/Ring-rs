from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path


DEFAULT_EXCLUDE_DIRS = {
    ".git",
    ".idea",
    ".vscode",
    "assets",
    "debug",
    "dist",
    "node_modules",
    "saves",
    "target",
    "test",
    "tests",
    "benches",
}


_RE_CFG_TEST_LINE = re.compile(r"^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*$")
_RE_ATTR_LINE = re.compile(r"^\s*#\s*\[[^\]]+\]\s*$")
_RE_MOD_TESTS = re.compile(r"\bmod\s+tests\b")


@dataclass(frozen=True)
class FileCount:
    path: Path
    loc: int


def _iter_rs_files(root: Path, exclude_dirs: set[str]) -> list[Path]:
    files: list[Path] = []
    root = root.resolve()

    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [
            d
            for d in dirnames
            if d not in exclude_dirs and not d.startswith(".") and not d.startswith("mutants.out")
        ]

        for name in filenames:
            if not name.endswith(".rs"):
                continue
            if name.endswith(".rs.bk"):
                continue
            if name == "tests.rs" or name == "test.rs":
                continue
            if name.endswith("_test.rs") or name.endswith("_tests.rs"):
                continue
            files.append(Path(dirpath) / name)

    return sorted(files)


def _scan_raw_string_prefix(s: str, i: int) -> tuple[int, int] | None:
    n = len(s)
    if i >= n:
        return None

    j = i
    if s[j] == "b" and j + 1 < n and s[j + 1] == "r":
        j += 2
    elif s[j] == "r":
        j += 1
    else:
        return None

    hashes = 0
    while j < n and s[j] == "#":
        hashes += 1
        j += 1
    if j < n and s[j] == '"':
        return (j - i + 1, hashes)
    return None


def _strip_comments_preserve_newlines(text: str) -> str:
    out: list[str] = []
    i = 0
    n = len(text)

    in_line_comment = False
    block_depth = 0

    in_string = False
    string_escape = False

    in_raw_string = False
    raw_hashes = 0

    in_char = False
    char_escape = False

    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""

        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
                out.append(ch)
            i += 1
            continue

        if block_depth > 0:
            if ch == "\n":
                out.append(ch)
                i += 1
                continue
            if ch == "/" and nxt == "*":
                block_depth += 1
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                i += 2
                continue
            i += 1
            continue

        if in_raw_string:
            if ch == '"':
                ok = True
                for k in range(raw_hashes):
                    if i + 1 + k >= n or text[i + 1 + k] != "#":
                        ok = False
                        break
                if ok:
                    out.append('"')
                    out.append("#" * raw_hashes)
                    i += 1 + raw_hashes
                    in_raw_string = False
                    raw_hashes = 0
                    continue
            out.append(ch)
            i += 1
            continue

        if in_string:
            out.append(ch)
            if string_escape:
                string_escape = False
            elif ch == "\\":
                string_escape = True
            elif ch == '"':
                in_string = False
            i += 1
            continue

        if in_char:
            out.append(ch)
            if char_escape:
                char_escape = False
            elif ch == "\\":
                char_escape = True
            elif ch == "'":
                in_char = False
            i += 1
            continue

        raw = _scan_raw_string_prefix(text, i)
        if raw is not None:
            prefix_len, hashes = raw
            out.append(text[i : i + prefix_len])
            i += prefix_len
            in_raw_string = True
            raw_hashes = hashes
            continue

        if ch == "/" and nxt == "/":
            in_line_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "*":
            block_depth = 1
            i += 2
            continue

        if ch == '"':
            in_string = True
            string_escape = False
            out.append(ch)
            i += 1
            continue

        if ch == "'":
            # Heuristic: treat as char literal only if there's a closing quote
            # within a short window; otherwise it's likely a lifetime.
            window = text[i : min(n, i + 12)]
            closing = window.find("'", 1)
            if closing != -1:
                in_char = True
                char_escape = False
            out.append(ch)
            i += 1
            continue

        out.append(ch)
        i += 1

    return "".join(out)


def _find_item_terminator(text: str, start: int) -> tuple[str, int] | None:
    i = start
    n = len(text)

    in_line_comment = False
    block_depth = 0
    in_string = False
    string_escape = False
    in_raw_string = False
    raw_hashes = 0
    in_char = False
    char_escape = False

    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""

        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
            i += 1
            continue

        if block_depth > 0:
            if ch == "/" and nxt == "*":
                block_depth += 1
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                i += 2
                continue
            i += 1
            continue

        if in_raw_string:
            if ch == '"':
                ok = True
                for k in range(raw_hashes):
                    if i + 1 + k >= n or text[i + 1 + k] != "#":
                        ok = False
                        break
                if ok:
                    i += 1 + raw_hashes
                    in_raw_string = False
                    raw_hashes = 0
                    continue
            i += 1
            continue

        if in_string:
            if string_escape:
                string_escape = False
            elif ch == "\\":
                string_escape = True
            elif ch == '"':
                in_string = False
            i += 1
            continue

        if in_char:
            if char_escape:
                char_escape = False
            elif ch == "\\":
                char_escape = True
            elif ch == "'":
                in_char = False
            i += 1
            continue

        raw = _scan_raw_string_prefix(text, i)
        if raw is not None:
            prefix_len, hashes = raw
            i += prefix_len
            in_raw_string = True
            raw_hashes = hashes
            continue

        if ch == "/" and nxt == "/":
            in_line_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "*":
            block_depth = 1
            i += 2
            continue

        if ch == '"':
            in_string = True
            string_escape = False
            i += 1
            continue

        if ch == "'":
            window = text[i : min(n, i + 12)]
            closing = window.find("'", 1)
            if closing != -1:
                in_char = True
                char_escape = False
            i += 1
            continue

        if ch == "{":
            return ("{", i)
        if ch == ";":
            return (";", i)

        i += 1

    return None


def _find_matching_brace(text: str, open_brace_index: int) -> int | None:
    i = open_brace_index
    n = len(text)
    depth = 0

    in_line_comment = False
    block_depth = 0
    in_string = False
    string_escape = False
    in_raw_string = False
    raw_hashes = 0
    in_char = False
    char_escape = False

    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""

        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
            i += 1
            continue

        if block_depth > 0:
            if ch == "/" and nxt == "*":
                block_depth += 1
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                i += 2
                continue
            i += 1
            continue

        if in_raw_string:
            if ch == '"':
                ok = True
                for k in range(raw_hashes):
                    if i + 1 + k >= n or text[i + 1 + k] != "#":
                        ok = False
                        break
                if ok:
                    i += 1 + raw_hashes
                    in_raw_string = False
                    raw_hashes = 0
                    continue
            i += 1
            continue

        if in_string:
            if string_escape:
                string_escape = False
            elif ch == "\\":
                string_escape = True
            elif ch == '"':
                in_string = False
            i += 1
            continue

        if in_char:
            if char_escape:
                char_escape = False
            elif ch == "\\":
                char_escape = True
            elif ch == "'":
                in_char = False
            i += 1
            continue

        raw = _scan_raw_string_prefix(text, i)
        if raw is not None:
            prefix_len, hashes = raw
            i += prefix_len
            in_raw_string = True
            raw_hashes = hashes
            continue

        if ch == "/" and nxt == "/":
            in_line_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "*":
            block_depth = 1
            i += 2
            continue

        if ch == '"':
            in_string = True
            string_escape = False
            i += 1
            continue

        if ch == "'":
            window = text[i : min(n, i + 12)]
            closing = window.find("'", 1)
            if closing != -1:
                in_char = True
                char_escape = False
            i += 1
            continue

        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0 and i > open_brace_index:
                return i

        i += 1

    return None


def _remove_cfg_test_mod_tests(text: str) -> str:
    # Remove `#[cfg(test)] mod tests { ... }` (and stacked attrs in between).
    # Integration tests are excluded by path; this only targets unit-test modules inside src.
    lines = text.splitlines(keepends=True)
    if not lines:
        return text

    line_starts: list[int] = [0]
    for ln in lines[:-1]:
        line_starts.append(line_starts[-1] + len(ln))

    spans: list[tuple[int, int, int]] = []

    i = 0
    while i < len(lines):
        line = lines[i]
        if "cfg(test)" not in line:
            i += 1
            continue

        cfg_here = _RE_CFG_TEST_LINE.match(line) is not None or "cfg(test)" in line
        if not cfg_here:
            i += 1
            continue

        j = i
        mod_line_index = None
        mod_match = _RE_MOD_TESTS.search(lines[i])
        if mod_match:
            mod_line_index = i
        else:
            j = i + 1
            while j < len(lines) and (_RE_ATTR_LINE.match(lines[j]) or lines[j].strip() == ""):
                j += 1
            if j < len(lines) and _RE_MOD_TESTS.search(lines[j]):
                mod_line_index = j

        if mod_line_index is None:
            i += 1
            continue

        mod_match = _RE_MOD_TESTS.search(lines[mod_line_index])
        assert mod_match is not None
        start = line_starts[i]
        mod_kw_pos = line_starts[mod_line_index] + mod_match.start()

        term = _find_item_terminator(text, mod_kw_pos)
        if term is None:
            i += 1
            continue

        kind, idx = term
        if kind == ";":
            end = idx + 1
        else:
            close = _find_matching_brace(text, idx)
            if close is None:
                i += 1
                continue
            end = close + 1

        removed = text[start:end]
        spans.append((start, end, removed.count("\n")))

        # Advance i roughly to the end line.
        end_line_guess = text[:end].count("\n")
        i = min(len(lines), end_line_guess + 1)

    if not spans:
        return text

    new_text = text
    for start, end, nl in sorted(spans, key=lambda x: x[0], reverse=True):
        new_text = new_text[:start] + ("\n" * nl) + new_text[end:]
    return new_text


def _count_effective_loc(text: str, include_inline_test: bool, include_comments: bool) -> int:
    if not include_inline_test:
        text = _remove_cfg_test_mod_tests(text)
    if not include_comments:
        text = _strip_comments_preserve_newlines(text)
    return sum(1 for line in text.splitlines() if line.strip())


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Count non-test Rust LOC (effective lines: non-empty after stripping comments)."
    )
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="Repository root (default: cwd)")
    parser.add_argument(
        "--exclude-dir",
        action="append",
        default=[],
        help="Extra directory name to exclude (can be repeated)",
    )
    parser.add_argument("--include-inline-test", action="store_true", help="Include inline test code")
    parser.add_argument("--include-comments", action="store_true", help="Include comments")
    parser.add_argument("--verbose", action="store_true", help="Print per-file LOC breakdown")
    args = parser.parse_args(argv)

    root: Path = args.root.resolve()
    exclude_dirs = set(DEFAULT_EXCLUDE_DIRS) | set(args.exclude_dir)

    rs_files = _iter_rs_files(root, exclude_dirs)
    counts: list[FileCount] = []

    for path in rs_files:
        try:
            raw = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            raw = path.read_text(encoding="utf-8", errors="replace")
        counts.append(FileCount(path=path, loc=_count_effective_loc(raw, args.include_inline_test, args.include_comments)))

    total = sum(c.loc for c in counts)

    if args.verbose:
        for c in sorted(counts, key=lambda x: (-x.loc, str(x.path))):
            rel = c.path.relative_to(root)
            print(f"{c.loc:7d}  {rel.as_posix()}")

    print(f"Total non-test Rust LOC: {total} (files: {len(counts)})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

