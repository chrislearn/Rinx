#!/usr/bin/env python3
"""Exercise offline Rinx chat widgets using native pointer/keyboard events."""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import time
import uuid
from native_probe import NativeApp

PLAIN = "Alpha 中文 👩‍💻 bravo & <literal>\nSecond line has selectable words.\nThird line ends here."


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path("target/debug/examples/chat_text_selection"))
    args = parser.parse_args()
    root = Path("target/chat-selection-regressions") / uuid.uuid4().hex
    root.mkdir(parents=True)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    app = NativeApp(root, port, auto_login=False)
    app.output.mkdir(parents=True)
    app.log = (app.output / "native.log").open("w")
    env = dict(os.environ, MAKEPAD_REMOTE=str(port), MAKEPAD_HIDE_WINDOWS="1", MAKEPAD_NO_FOCUS="1",
               RINX_DATA_DIR=str((root / "profile").resolve()), RAYON_NUM_THREADS="1")
    app.process = subprocess.Popen([str(args.binary.resolve())], env=env, stdin=subprocess.DEVNULL,
                                   stdout=app.log, stderr=subprocess.STDOUT)
    report = {"passed": False, "checks": []}
    try:
        for _ in range(80):
            if app.process.poll() is not None:
                raise RuntimeError("Native selection fixture exited")
            try:
                if app.request("/s")["pid"] == app.process.pid:
                    break
            except OSError:
                pass
            time.sleep(.25)
        app.wait_text("Ready")

        def state():
            app.click_id("inspect")
            return json.loads(next(w["t"] for w in app.snap() if w["i"] == "result"))

        def drag(points):
            app.request("/m", k="down", x=points[0][0], y=points[0][1], wait=1)
            for x, y in points[1:]:
                app.request("/m", k="move", x=x, y=y, wait=1)
            app.request("/m", k="up", x=points[-1][0], y=points[-1][1], wait=1)

        plain = next(w["r"] for w in app.snap() if w["i"] == "plain")
        rich = next(w["r"] for w in app.snap() if w["i"] == "rich")
        px, py = plain[:2]
        rx, ry = rich[:2]
        line = py + 11
        link = ry + 41

        drag([(px, line), (px + 38, line)])
        s = state()
        assert s["plain"] == s["copy"] == "Alpha", s
        app.capture("plain-partial-selection")
        drag([(px + 38, line), (px, line)])
        assert state()["copy"] == "Alpha"
        report["checks"].append("forward_reverse_partial_plaintext_and_native_copy")

        drag([(px + 42, line), (px + 96, line)])
        s = state()
        assert "中文" in s["copy"] and "👩‍💻" in s["copy"] and "Alpha" not in s["copy"], s
        report["checks"].append("chinese_and_emoji_graphemes")

        app.request("/k", c="A", cmd=1, wait=1)
        s = state()
        assert s["plain"] == s["copy"] == PLAIN, s
        report["checks"].append("select_all_preserves_newlines_literal_markup_and_unicode")

        drag([(px, line), (px + 300, py + plain[3] - 10)])
        assert state()["copy"] == PLAIN
        report["checks"].append("multiline_drag")

        drag([(rx, ry + 14), (rx + 60, ry + 14), (rx + 240, ry + 14)])
        s = state()
        assert s["plain"] == "" and "bold 中文" in s["rich"] and "italic words" in s["copy"], s
        report["checks"].append("rich_text_cross_style_selection_and_focus_transfer")

        drag([(rx + 1, link), (rx + 50, link), (rx + 120, link), (rx + 240, link)])
        s = state()
        assert s["copy"] == "Selectable link text after the link." and s["link_clicks"] == 0, s
        app.capture("rich-link-drag-selection")
        report["checks"].append("link_start_multiple_move_drag_without_activation")

        drag([(rx + 1, link), (rx + 9, link)])
        s = state()
        assert s["copy"] and len(s["copy"]) < 4 and s["link_clicks"] == 0, s
        drag([(rx + 1, link), (rx + 60, link), (rx + 1, link)])
        assert state()["link_clicks"] == 0
        app.click(rx + 20, link)
        assert state()["link_clicks"] == 1
        report["checks"].append("short_and_returning_drag_suppress_links_normal_click_preserved")

        drag([(px, line), (px + 38, line)])
        app.click_id("menu_button")
        snapshot = app.snap()
        (root / "menu-snapshot.json").write_text(json.dumps(snapshot, ensure_ascii=False))
        app.capture("menu-open")
        ids = {w["i"] for w in snapshot}
        assert {"copy_selection_button", "copy_text_button"} <= ids
        app.capture("copy-selection-context-menu")
        app.click_id("copy_selection_button")
        s = json.loads(next(w["t"] for w in app.snap() if w["i"] == "result"))
        assert s["menu_copy"] == "Alpha", s
        report["checks"].append("selection_snapshot_survives_menu_focus_whole_copy_retained")

        drag([(px, line), (px + 38, line)])
        app.click_id("reset")
        s = state()
        assert not s["plain"] and s["copy"] is None, s
        app.click_id("menu_button")
        assert "copy_selection_button" not in {w["i"] for w in app.snap()}
        report["checks"].append("replacement_clears_selection_and_hides_partial_copy")
        report["passed"] = True
    finally:
        app.process.terminate()
        app.process.wait(timeout=10)
        app.log.close()
        (root / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2))
        (root / "trace.json").write_text(json.dumps(app.trace, ensure_ascii=False, indent=2))
        print(json.dumps({"report": str(root / "report.json"), **report}, ensure_ascii=False))


if __name__ == "__main__":
    main()
