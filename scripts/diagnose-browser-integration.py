"""Read local integration state without visiting stores or modifying the library."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import sys


def read_report(root, limit, store):
    report = {"checked_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "data_dir": str(root)}
    db = root / "launcherg_sqlite.db3"
    if db.exists():
        with sqlite3.connect(db.as_uri() + "?mode=ro", uri=True) as connection:
            connection.execute("PRAGMA query_only=ON")
            tables = {row[0] for row in connection.execute("SELECT name FROM sqlite_master WHERE type='table'")}
            report["library_counts"] = {name: connection.execute(f"SELECT COUNT(*) FROM {name}").fetchone()[0]
                                        for name in ("works", "dmm_works", "dlsite_works") if name in tables}
            if "native_messaging_host_logs" in tables:
                rows = connection.execute("SELECT type,created_at,message FROM native_messaging_host_logs ORDER BY id DESC LIMIT 12").fetchall()
                report["host_activity"] = [{"type": typ, "at_utc": at,
                    "request_case": match.group(1) if (match := re.search(r"request: (\w+)", message)) else None,
                    "success": "success: true" in message if "response:" in message else None}
                    for typ, at, message in rows]
    else:
        report["database"] = "not found"
    events = []
    for name in ("browser-sync.previous.jsonl", "browser-sync.jsonl"):
        path = root / "diagnostics" / name
        if path.exists():
            for line in path.read_text(encoding="utf-8").splitlines():
                try:
                    event = json.loads(line)
                    if not store or event.get("store") in (store, "extension"):
                        events.append(event)
                except (ValueError, TypeError):
                    continue
    report["diagnostics"] = events[-limit:]
    report["last_automatic_completion"] = next((event for event in reversed(events)
        if event.get("trigger") == "auto" and event.get("stage") == "sync_finished"), None)
    report["registration"] = []
    if os.name == "nt":
        import winreg
        for browser in ("Google\\Chrome", "Microsoft\\Edge"):
            entry = {"browser": browser}
            try:
                key_path = rf"Software\{browser}\NativeMessagingHosts\moe.ryoha.launcherg.extension_host"
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path) as key:
                    manifest_path = Path(winreg.QueryValue(key, None))
                manifest = json.loads(manifest_path.read_text(encoding="utf-8-sig"))
                host = Path(manifest["path"])
                entry.update(manifest=str(manifest_path), host=str(host), host_exists=host.is_file())
                if host.is_file():
                    entry["host_sha256"] = hashlib.sha256(host.read_bytes()).hexdigest()
            except (OSError, ValueError, KeyError) as error:
                entry["error"] = str(error)
            report["registration"].append(entry)
    extension = root / "browser-extension" / "dev-extension"
    if (extension / "manifest.json").exists():
        manifest = json.loads((extension / "manifest.json").read_text(encoding="utf-8-sig"))
        report["extension"] = {"path": str(extension), "version": manifest.get("version"),
            "main_world_hooks": [item["js"] for item in manifest.get("content_scripts", []) if item.get("world") == "MAIN"]}
    return report


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-dir", type=Path, default=Path(os.environ.get("LAUNCHERG_DATA_DIR") or Path(os.environ["APPDATA"]) / "ryoha.moe" / "launcherg"))
    parser.add_argument("--limit", type=int, default=60)
    parser.add_argument("--store", choices=("dmm", "dlsite"))
    args = parser.parse_args()
    print(json.dumps(read_report(args.data_dir.resolve(), max(1, min(args.limit, 200)), args.store), ensure_ascii=False, indent=2))
