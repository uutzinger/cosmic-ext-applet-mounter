"""Archive completed task-list and completion-note history at a release break.

Moves everything before a chosen heading to docs/archive/ unchanged (relative
links repaired), keeps the rest active, and copies every still-open checklist
item into the task list's "Archive and open items carried forward" section.
"""

import argparse
import os
from pathlib import Path
import re
import sys
from urllib.parse import unquote

sys.path.insert(0, str(Path(__file__).resolve().parent))
from task_output import TaskReport

TASKS = Path("Task List.md")
NOTES = Path("Task List Completion Notes.md")
ARCHIVE_DIR = Path("docs/archive")
CARRY = "## Archive and open items carried forward"
LINK = re.compile(r"(\[[^\]]*\]\()([^)\s]+)(\))")
OPEN = re.compile(r"^(\s*)- \[ \]")


def first_heading(lines):
    return next((i for i, line in enumerate(lines) if i and line.startswith("#")), len(lines))


def find_break(lines, marker, start):
    hits = [i for i, line in enumerate(lines) if line.startswith(marker) and i >= start]
    if len(hits) != 1:
        raise ValueError(f"Expected exactly one heading starting with {marker!r}, found {len(hits)}")
    return hits[0]


def repair_links(text, archive_path):
    def fix(match):
        target = match.group(2)
        if re.match(r"[a-z][a-z0-9+.-]*:", target) or target.startswith("#"):
            return match.group(0)
        name = unquote(target.split("#")[0])
        if (archive_path.parent / name).exists() or not Path(name).exists():
            return match.group(0)
        relative = os.path.relpath(Path(name), archive_path.parent)
        return match.group(1) + relative.replace(" ", "%20") + target[len(target.split("#")[0]):] + match.group(3)
    return LINK.sub(fix, text)


def open_items(lines):
    """Yield (section, subsection, block) for each outermost open checkbox item."""
    section = sub = None
    taken = -1
    for i, line in enumerate(lines):
        heading = re.match(r"^(#{2,3}) (.*)", line)
        if heading:
            if heading.group(1) == "##":
                section, sub = heading.group(2), None
            else:
                sub = heading.group(2)
        match = OPEN.match(line)
        if not match or i <= taken:
            continue
        indent, end = len(match.group(1)), i + 1
        while end < len(lines):
            nxt = lines[end]
            if not nxt.strip():
                probe = end + 1
                while probe < len(lines) and not lines[probe].strip():
                    probe += 1
                if (probe < len(lines) and not lines[probe].startswith("#")
                        and len(lines[probe]) - len(lines[probe].lstrip()) > indent):
                    end = probe
                    continue
                break
            if len(nxt) - len(nxt.lstrip()) <= indent:
                break
            end += 1
        taken = end - 1
        yield section, sub, lines[i:end]


def archive_note(title, version, other):
    return (f"# {title} — archive through v{version}\n\n"
            f"**Archived** after the v{version} release. Frozen historical record; checkboxes here are\n"
            f"not maintained. Open items were carried forward to the active\n"
            f"[Task List](../../Task%20List.md) under \"Open items carried forward\"; that copy is\n"
            f"authoritative. {other}\n\n---\n\n")


def carry_groups(items):
    out, last_section, last_sub = [], None, None
    for section, sub, block in items:
        if section != last_section:
            out += [f"### From: {section}", ""]
            last_section, last_sub = section, None
        if sub and sub != last_sub:
            out += [f"_Subsection: {sub}_", ""]
            last_sub = sub
        out += block + [""]
    return out


def task_pointers():
    names = sorted(p.name for p in ARCHIVE_DIR.glob("task-list-through-v*.md"))
    lines = ["Completed history is archived, unchanged, under `docs/archive/`:", ""]
    for name in names:
        version = name[len("task-list-through-"):-3]
        lines.append(f"- {version}: [task list]({ARCHIVE_DIR / name}), "
                     f"[completion notes]({ARCHIVE_DIR / ('task-list-completion-notes-through-' + version + '.md')})")
    return lines + ["", "Items below were still `[ ]` when archived and are copied verbatim, grouped by",
                    "original section. The archived copy is historical; triage each as `[x]`, `[o]`",
                    "(with a reference to its authoritative task) or keep it open.", ""]


def plan(args):
    tasks, notes = TASKS.read_text().split("\n"), NOTES.read_text().split("\n")
    t_head = first_heading(tasks)
    t_start, carry = t_head, None
    if tasks[t_head].startswith(CARRY):
        carry = t_head
        t_start = next((i for i in range(t_head + 1, len(tasks)) if tasks[i].startswith("## ")), len(tasks))
    t_break = find_break(tasks, args.tasks_from, t_start)
    n_start = first_heading(notes)
    n_break = find_break(notes, args.notes_from, n_start)
    return tasks, notes, t_head, carry, t_start, t_break, n_start, n_break


def apply(args, report):
    tasks, notes, t_head, carry, t_start, t_break, n_start, n_break = plan(args)
    version = args.version
    t_archive, n_archive = tasks[t_start:t_break], notes[n_start:n_break]
    items = list(open_items(t_archive))
    report.update(next_action=f"{len(items)} open items to carry forward; review the result before committing.")
    if args.dry_run:
        print(f"dry run: archive {len(t_archive)} task lines and {len(n_archive)} note lines, "
              f"carry forward {len(items)} open items", file=sys.stderr)
        return
    t_path = ARCHIVE_DIR / f"task-list-through-v{version}.md"
    n_path = ARCHIVE_DIR / f"task-list-completion-notes-through-v{version}.md"
    if t_path.exists() or n_path.exists():
        raise ValueError("Archive for this version already exists; refusing to overwrite")
    ARCHIVE_DIR.mkdir(parents=True, exist_ok=True)
    t_path.write_text(archive_note("Task List", version, f"Evidence: [completion notes]({n_path.name}).")
                      + repair_links("\n".join(t_archive).rstrip("\n"), t_path) + "\n")
    n_path.write_text(archive_note("Task List Completion Notes", version, f"Tasks: [task list]({t_path.name}).")
                      + repair_links("\n".join(n_archive).rstrip("\n"), n_path) + "\n")
    # Active task list: header, refreshed carry section (old groups kept), remainder.
    old_groups = []
    if carry is not None:
        body = tasks[carry + 1:t_start]
        first = next((i for i, line in enumerate(body) if line.startswith("### From")), len(body))
        old_groups = body[first:]
    new_carry = [CARRY, ""] + task_pointers() + old_groups + carry_groups(items)
    TASKS.write_text("\n".join(tasks[:t_head] + new_carry + tasks[t_break:]))
    pointer = [f"Entries through v{version}: [archive]({n_path})."]
    n_head = "\n".join(notes[:n_start]).rstrip("\n").split("\n")
    NOTES.write_text("\n".join(n_head + [""] + pointer + [""] + notes[n_break:]))


def main():
    parser = argparse.ArgumentParser(prog="scripts/archive-release.sh", description=__doc__)
    parser.add_argument("version", help="Released version, for example 0.5.3")
    parser.add_argument("--tasks-from", required=True,
                        help="Heading prefix of the first task-list section that stays active")
    parser.add_argument("--notes-from", required=True,
                        help="Heading prefix of the first completion-note entry that stays active")
    parser.add_argument("--dry-run", action="store_true", help="Report what would move; write nothing")
    parser.add_argument("--allow-dirty", action="store_true", help="Allow uncommitted edits to the two files")
    parser.add_argument("--log-dir")
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", args.version):
        parser.error("version must look like 1.2.3")
    os.chdir(Path(os.environ.get("ARCHIVE_ROOT") or Path(__file__).resolve().parents[2]))
    with TaskReport("archive-release", args.log_dir) as report:
        report.update(ref=f"v{args.version}")
        if not args.allow_dirty and not args.dry_run and report.capture(
                "git", "status", "--porcelain", "--", str(TASKS), str(NOTES)):
            raise ValueError("Commit or stash edits to the task list and notes first (or --allow-dirty)")
        apply(args, report)
        report.phase("archive-complete" if not args.dry_run else "dry-run-complete")


if __name__ == "__main__":
    main()
