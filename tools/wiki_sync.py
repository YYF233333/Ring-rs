from __future__ import annotations

import argparse
import posixpath
import re
import shutil
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DOCS_ROOT = REPO_ROOT / "docs"
MARKDOWN_EXTENSIONS = {".md", ".mdc"}
LINK_PATTERN = re.compile(r"(!?\[[^\]]*])\(([^)]+)\)")
INLINE_CODE_PATTERN = re.compile(r"`[^`]*`")

EXTRA_PAGES = {
    "README.md": "project-overview.md",
    "ARCH.md": "architecture.md",
    "CONTRIBUTING.md": "contributing.md",
    "RFCs/README.md": "rfc-index.md",
}


def normalize_repo_path(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def repo_path(repo_rel: str) -> Path:
    return REPO_ROOT / Path(repo_rel)


def resolve_relative(current_rel: str, target: str) -> str | None:
    current_dir = posixpath.dirname(current_rel)
    resolved = posixpath.normpath(posixpath.join(current_dir, target))
    if resolved.startswith("../") or resolved == "..":
        return None
    return resolved


def resolve_local_target(
    current_rel: str,
    path_part: str,
    known_paths: set[str] | None = None,
) -> str | None:
    candidates: list[str] = []

    relative_candidate = resolve_relative(current_rel, path_part)
    if relative_candidate is not None:
        candidates.append(relative_candidate)

    if not path_part.startswith("."):
        repo_root_candidate = posixpath.normpath(path_part)
        if repo_root_candidate not in candidates:
            candidates.append(repo_root_candidate)

    for candidate in candidates:
        if known_paths and candidate in known_paths:
            return candidate
        if repo_path(candidate).exists():
            return candidate

    return candidates[0] if candidates else None


def is_external_target(target: str) -> bool:
    return (
        not target
        or target.startswith("#")
        or target.startswith(("http://", "https://", "mailto:", "data:", "file://"))
        or (target.startswith("<") and target.endswith(">"))
    )


def split_target(target: str) -> tuple[str, str]:
    if "#" not in target:
        return target, ""
    path_part, anchor = target.split("#", 1)
    return path_part, anchor


def overlap_inline_code(spans: list[tuple[int, int]], match_span: tuple[int, int]) -> bool:
    start, end = match_span
    return any(start < span_end and end > span_start for span_start, span_end in spans)


def collect_publish_map() -> dict[str, str]:
    publish_map: dict[str, str] = {}

    for source in DOCS_ROOT.rglob("*.md"):
        source_rel = normalize_repo_path(source)
        if source_rel == "docs/README.md":
            publish_map[source_rel] = "Home.md"
        else:
            publish_map[source_rel] = source.relative_to(DOCS_ROOT).as_posix()

    publish_map.update(EXTRA_PAGES)
    return publish_map


def rewrite_markdown_links(
    text: str,
    current_source_rel: str,
    current_output_rel: str,
    publish_map: dict[str, str],
    repo_url_base: str,
) -> str:
    known_paths = set(publish_map)
    inside_fence = False
    rewritten_lines: list[str] = []

    for line in text.splitlines(keepends=True):
        stripped = line.lstrip()
        if stripped.startswith("```"):
            rewritten_lines.append(line)
            inside_fence = not inside_fence
            continue

        if inside_fence:
            rewritten_lines.append(line)
            continue

        inline_code_spans = [match.span() for match in INLINE_CODE_PATTERN.finditer(line)]

        def replacer(match: re.Match[str]) -> str:
            if overlap_inline_code(inline_code_spans, match.span()):
                return match.group(0)

            label, raw_target = match.groups()
            target = raw_target.strip()
            if is_external_target(target):
                return match.group(0)

            path_part, anchor = split_target(target)
            if not path_part:
                return match.group(0)

            resolved = resolve_local_target(
                current_rel=current_source_rel,
                path_part=path_part,
                known_paths=known_paths,
            )
            if resolved is None:
                return match.group(0)

            if resolved in publish_map:
                new_target = posixpath.relpath(
                    publish_map[resolved], posixpath.dirname(current_output_rel)
                )
            else:
                candidate = repo_path(resolved)
                if not candidate.exists():
                    return match.group(0)
                new_target = f"{repo_url_base}/{resolved}" if repo_url_base else target

            if anchor:
                new_target = f"{new_target}#{anchor}"
            return f"{label}({new_target})"

        rewritten_lines.append(LINK_PATTERN.sub(replacer, line))

    return "".join(rewritten_lines)


def write_sidebar(output_dir: Path) -> None:
    sidebar = """# Ring-rs Wiki

- [文档首页](Home.md)
- [项目概览](project-overview.md)
- [架构约束](architecture.md)
- [贡献指南](contributing.md)
- [RFC 索引](rfc-index.md)

## 文档分类

- [内容作者](authoring/README.md)
- [引擎开发](engine/README.md)
- [测试与调试](testing/README.md)
- [维护文档](maintenance/README.md)

## 高频入口

- [Getting Started](authoring/getting-started.md)
- [脚本语法规范](authoring/script-syntax.md)
- [仓库导航地图](engine/architecture/navigation-map.md)
- [模块摘要入口](engine/architecture/module-summaries/README.md)
- [Headless 指南](testing/headless-guide.md)
"""
    (output_dir / "_Sidebar.md").write_text(sidebar, encoding="utf-8")


def build_wiki(output_dir: Path, repo_url_base: str) -> None:
    publish_map = collect_publish_map()
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    for source_rel, output_rel in publish_map.items():
        source_path = repo_path(source_rel)
        output_path = output_dir / output_rel
        text = source_path.read_text(encoding="utf-8")
        rewritten = rewrite_markdown_links(
            text=text,
            current_source_rel=source_rel,
            current_output_rel=output_rel,
            publish_map=publish_map,
            repo_url_base=repo_url_base.rstrip("/"),
        )
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(rewritten, encoding="utf-8")

    write_sidebar(output_dir)


def collect_link_check_files() -> list[Path]:
    files: set[Path] = set()

    for path in REPO_ROOT.glob("*.md"):
        files.add(path)

    for base in [
        REPO_ROOT / "docs",
        REPO_ROOT / "RFCs",
        REPO_ROOT / ".cursor",
    ]:
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if path.is_file() and path.suffix.lower() in MARKDOWN_EXTENSIONS:
                files.add(path)

    for path in (REPO_ROOT / "assets").rglob("README.md"):
        files.add(path)

    return sorted(files)


def validate_links() -> list[str]:
    errors: list[str] = []

    for path in collect_link_check_files():
        source_rel = normalize_repo_path(path)
        text = path.read_text(encoding="utf-8")
        inside_fence = False
        for line_no, line in enumerate(text.splitlines(), start=1):
            stripped = line.lstrip()
            if stripped.startswith("```"):
                inside_fence = not inside_fence
                continue
            if inside_fence:
                continue

            inline_code_spans = [match.span() for match in INLINE_CODE_PATTERN.finditer(line)]

            for match in LINK_PATTERN.finditer(line):
                if overlap_inline_code(inline_code_spans, match.span()):
                    continue
                target = match.group(2).strip()
                if is_external_target(target):
                    continue

                path_part, _anchor = split_target(target)
                if not path_part:
                    continue

                resolved = resolve_local_target(source_rel, path_part)
                if resolved is None:
                    errors.append(f"{source_rel}:{line_no} -> {target} (escapes repo root)")
                    continue

                candidate = repo_path(resolved)
                if candidate.exists():
                    continue
                if candidate.is_dir() and (candidate / "README.md").exists():
                    continue

                errors.append(f"{source_rel}:{line_no} -> {target} (missing target)")

    return errors


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build Ring-rs GitHub Wiki content and validate markdown links."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    build_parser = subparsers.add_parser("build", help="Build wiki content")
    build_parser.add_argument(
        "--output-dir",
        required=True,
        help="Directory to write generated wiki files into.",
    )
    build_parser.add_argument(
        "--repo-url-base",
        default="",
        help="GitHub blob URL prefix for non-wiki files, e.g. https://github.com/org/repo/blob/main",
    )

    subparsers.add_parser("check-links", help="Validate markdown links in tracked docs")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.command == "build":
        build_wiki(Path(args.output_dir), args.repo_url_base)
        return 0

    if args.command == "check-links":
        errors = validate_links()
        if errors:
            print("Broken markdown links detected:")
            for error in errors:
                print(f"- {error}")
            return 1
        print("All markdown links resolved successfully.")
        return 0

    raise ValueError(f"Unsupported command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
