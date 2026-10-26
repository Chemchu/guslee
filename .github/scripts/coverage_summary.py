import json
import os
import sys

METRICS = [("Lines", "lines"), ("Functions", "functions"), ("Regions", "regions")]


def percent(summary):
    return summary["covered"] / summary["count"] * 100 if summary["count"] else None


def format_percent(value):
    return "–" if value is None else f"{value:.1f}%"


def main():
    report_path, minimum = sys.argv[1], float(sys.argv[2])
    with open(report_path) as report:
        data = json.load(report)["data"][0]

    totals = data["totals"]
    line_coverage = percent(totals["lines"])
    passed = line_coverage is not None and line_coverage >= minimum

    out = ["## Code coverage", ""]
    covered = f"**{format_percent(line_coverage)}** of lines covered"
    if passed:
        out.append(f"✅ {covered} (minimum: {minimum:g}%)")
    else:
        out.append(f"❌ {covered}, below the minimum of {minimum:g}%")
    out += ["", "| Metric | Covered | Total | Coverage |", "|---|---:|---:|---:|"]
    for label, key in METRICS:
        summary = totals[key]
        out.append(
            f"| {label} | {summary['covered']} | {summary['count']} "
            f"| {format_percent(percent(summary))} |"
        )

    files = sorted(
        data["files"],
        key=lambda f: (percent(f["summary"]["lines"]) or 0, f["filename"]),
    )
    out += [
        "",
        "<details><summary>Coverage by file, lowest first</summary>",
        "",
        "| File | Lines | Functions |",
        "|---|---:|---:|",
    ]
    for file in files:
        lines = file["summary"]["lines"]
        name = os.path.relpath(file["filename"])
        out.append(
            f"| `{name}` | {format_percent(percent(lines))} "
            f"({lines['covered']}/{lines['count']}) "
            f"| {format_percent(percent(file['summary']['functions']))} |"
        )
    out += ["", "</details>"]

    print("\n".join(out))


if __name__ == "__main__":
    main()
