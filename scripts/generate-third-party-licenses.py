#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
APP_DIR = ROOT / "app"
DIST_DIR = ROOT / "dist"


@dataclass(frozen=True)
class RustPkg:
    name: str
    version: str
    license_expr: str | None
    repository: str | None
    source: str | None


@dataclass(frozen=True)
class NodePkg:
    name: str
    version: str | None
    license_expr: str | None
    repository: str | None


def run(cmd: list[str], cwd: Path) -> str:
    p = subprocess.run(cmd, cwd=str(cwd), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(cmd)}\n{p.stderr.strip()}")
    return p.stdout or ""


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def normalize_repo(repo: Any) -> str | None:
    if repo is None:
        return None
    if isinstance(repo, str):
        return repo
    if isinstance(repo, dict):
        url = repo.get("url")
        if isinstance(url, str):
            return url
    return None


def load_rust_packages() -> list[RustPkg]:
    out = run(["cargo", "metadata", "--format-version=1"], cwd=APP_DIR)
    if not out.strip():
        return []
    meta = json.loads(out)

    workspace_members = set(meta.get("workspace_members") or [])
    pkgs: list[RustPkg] = []
    for p in meta.get("packages") or []:
        if p.get("id") in workspace_members:
            continue
        pkgs.append(
            RustPkg(
                name=p.get("name") or "",
                version=p.get("version") or "",
                license_expr=p.get("license"),
                repository=p.get("repository"),
                source=p.get("source"),
            )
        )

    pkgs.sort(key=lambda x: (x.name.lower(), x.version))
    return pkgs


def load_node_packages() -> list[NodePkg]:
    pkg_json = read_json(ROOT / "package.json")
    deps: dict[str, str] = {}
    deps.update(pkg_json.get("dependencies") or {})
    deps.update(pkg_json.get("devDependencies") or {})

    pkgs: list[NodePkg] = []
    for name in sorted(deps.keys(), key=lambda s: s.lower()):
        p = ROOT / "node_modules" / name / "package.json"
        if not p.exists():
            pkgs.append(NodePkg(name=name, version=None, license_expr=None, repository=None))
            continue
        j = read_json(p)
        pkgs.append(
            NodePkg(
                name=name,
                version=j.get("version"),
                license_expr=j.get("license") if isinstance(j.get("license"), str) else None,
                repository=normalize_repo(j.get("repository")),
            )
        )

    return pkgs


def fmt_table(rows: Iterable[list[str]]) -> str:
    rows = list(rows)
    if not rows:
        return ""

    widths = [0] * len(rows[0])
    for r in rows:
        for i, c in enumerate(r):
            widths[i] = max(widths[i], len(c))

    out_lines: list[str] = []
    for r in rows:
        out_lines.append(" | ".join(c.ljust(widths[i]) for i, c in enumerate(r)))
    return "\n".join(out_lines)


def generate() -> str:
    lines: list[str] = []
    lines.append("THIRD_PARTY_LICENSES")
    lines.append("")
    lines.append("本文件用于列出构建/分发二进制时用到的第三方依赖及其许可证信息（便于合规）。")
    lines.append("许可证全文与更完整信息请以各项目仓库/发布页为准。")
    lines.append("")

    lines.append("=== Rust (Cargo) ===")
    rust_error: str | None = None
    try:
        rust = load_rust_packages()
    except Exception as e:  # noqa: BLE001
        rust = []
        rust_error = str(e)
    rust_rows = [["Name", "Version", "License", "Repository/Source"]]
    for p in rust:
        repo = p.repository or p.source or ""
        rust_rows.append([p.name, p.version, p.license_expr or "", repo])
    lines.append(fmt_table(rust_rows))
    if rust_error:
        lines.append("")
        lines.append(f"[warn] Rust 依赖清单生成失败：{rust_error}")
    lines.append("")

    lines.append("=== Node (TS/Bun) - direct dependencies ===")
    node_error: str | None = None
    try:
        node = load_node_packages()
    except Exception as e:  # noqa: BLE001
        node = []
        node_error = str(e)
    node_rows = [["Name", "Version", "License", "Repository"]]
    for p in node:
        node_rows.append([p.name, p.version or "", p.license_expr or "", p.repository or ""])
    lines.append(fmt_table(node_rows))
    if node_error:
        lines.append("")
        lines.append(f"[warn] Node 依赖清单生成失败：{node_error}")
    lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def main() -> None:
    try:
        out = generate()
    except Exception as e:  # noqa: BLE001
        out = (
            "THIRD_PARTY_LICENSES\n\n"
            "生成第三方许可证清单时发生错误（已降级生成占位文件，避免打包失败）。\n\n"
            f"[error] {e}\n"
        )
    DIST_DIR.mkdir(parents=True, exist_ok=True)
    out_path = DIST_DIR / "THIRD_PARTY_LICENSES.txt"
    out_path.write_text(out, encoding="utf-8")
    print("OK:", str(out_path))


if __name__ == "__main__":
    main()
