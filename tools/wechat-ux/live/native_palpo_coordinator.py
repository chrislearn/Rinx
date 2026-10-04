#!/usr/bin/env python3
"""Production Rinx Splash/host + Rust Palpo process; explicit loopback Matrix fixture.

No live accounts, JavaScript backend, deployment state or existing app instances.
This proves the mini-app decision boundary, not native Hagency provisioning/chat.
"""
import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import threading
import time
from urllib.request import Request, build_opener, ProxyHandler
import uuid

from native_palpo import PalpoApp, port, fill, inspect

SERVICES = ["palpo.session.open", "palpo.inbox.submit", "palpo.inbox.get", "palpo.inbox.list"]
HTTP = build_opener(ProxyHandler({}))


class Matrix(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        users = {f"Bearer {name}-secret": f"@{name}:example.test" for name in ("owner", "coordinator", "admin")}
        user = users.get(self.headers.get("Authorization"))
        status = 200 if user else 401
        if self.path == "/_matrix/client/v3/account/whoami" and user:
            value = {"user_id": user}
        elif self.path == "/_palpo/admin/v1/appservices" and user == "@admin:example.test":
            value = {"appservices": []}
        else:
            status = 403 if user else 401
            value = {"errcode": "M_FORBIDDEN"}
        raw = json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)


def post(endpoint, operation, token, body):
    req = Request(endpoint + "/_palpo/miniapp/v1/" + operation,
                  data=json.dumps(body).encode(), headers={"Content-Type": "application/json", "Authorization": "Bearer " + token})
    with HTTP.open(req, timeout=10) as response:
        return json.load(response)


def call(endpoint, token, service, args):
    return post(endpoint, "call", token, {"service": service, "args": args})


def launch(root, binary, endpoint, role):
    profile = root / "profile"
    (profile / "app").mkdir(parents=True)
    app = PalpoApp(root, port=port(), auto_login=False)
    app.output.mkdir(parents=True)
    app.log = (app.output / "native.log").open("w")
    env = dict(os.environ, RINX_DATA_DIR=str(profile.resolve()), MAKEPAD_HIDE_WINDOWS="1",
               MAKEPAD_NO_FOCUS="1", MAKEPAD_REMOTE=str(app.port), PALPO_FIXTURE_URL=endpoint)
    env.pop("PALPO_LIVE_SESSION_FILE", None)
    env.pop("MAKEPAD_FOCUS", None)
    args = [str(binary.resolve())] + (["--narrow"] if role == "owner" else ["--" + role])
    app.process = subprocess.Popen(args, env=env, stdout=app.log, stderr=subprocess.STDOUT)
    try:
        for _ in range(150):
            if app.process.poll() is not None:
                raise RuntimeError("Native fixture exited; inspect native.log")
            try:
                if app.request("/s")["w"]:
                    break
            except OSError:
                pass
            time.sleep(.1)
        app.wait_text("Pending actions stay here", timeout=45)
        return app
    except Exception:
        app.stop()
        raise


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--backend", type=Path, required=True)
    parser.add_argument("--binary", type=Path, default=Path("target/fast/examples/palpo_miniapp"))
    args = parser.parse_args()
    root = Path("target/palpo-coordinator-validation") / uuid.uuid4().hex
    root.mkdir(parents=True)
    matrix = ThreadingHTTPServer(("127.0.0.1", 0), Matrix)
    threading.Thread(target=matrix.serve_forever, daemon=True).start()
    endpoint = f"http://127.0.0.1:{port()}"
    env = {k: v for k, v in os.environ.items() if not k.startswith("PALPO_")}
    env.update(PALPO_SERVER_NAME="example.test", PALPO_URL=f"http://127.0.0.1:{matrix.server_port}",
               PALPO_ADMIN_DATABASE=str((root / "admin.sqlite").resolve()), PUBLIC_ORIGIN=endpoint,
               PALPO_OPERATIONS_LISTEN=endpoint.removeprefix("http://"), PALPO_TRANSPORT_ORIGIN=endpoint,
               PALPO_RELAY_ORIGIN=endpoint)
    fleet = "hf_" + "a" * 32
    now = int(time.time() * 1000)
    snapshot = {"engagements": {fleet: {"id": fleet, "server": "example.test", "owner": "@provider:example.test",
        "coordinator": "@coordinator:example.test", "registrationGeneration": 1, "delegationRevision": 1,
        "delegationExpiresAtMs": now + 3600000, "state": "verified", "allowSelfApproval": False, "coordinatorApprovalV1": True}},
        "resources": {"grant_a": {"id": "grant_a", "serverEngagementId": fleet, "revision": 1, "allocatedTokens": 1000000,
            "eligibleManagers": ["@owner:example.test"]}},
        "projects": {"project_one": {"projectId": "project_one", "serverEngagementId": fleet, "revision": 1,
            "owner": "@owner:example.test", "resourceAllocations": ["grant_a"], "state": "ready"}}}
    authority = root / "authority.json"
    authority.write_text(json.dumps(snapshot))
    subprocess.run([str(args.backend.resolve()), "import-authority", str(authority.resolve())], env=env, check=True,
                   stdout=subprocess.DEVNULL)
    # Seed an installed transport fixture while no process owns this fixture DB.
    with sqlite3.connect(root / "admin.sqlite") as db:
        state = json.loads(db.execute("SELECT body FROM state WHERE id=1").fetchone()[0])
        state["fleets"][fleet] = {"id": fleet, "state": "ready", "installation": "installed",
            "representativeMxid": f"@{fleet}_representative:example.test", "registrationGeneration": 1,
            "transport": {"mode": "outbound", "generation": 1, "sequence": 0, "token": "fixture-machine"},
            "capabilities": {"coordinatorApprovalV1": True}}
        db.execute("UPDATE state SET body=? WHERE id=1", [json.dumps(state)])
    log = (root / "backend.log").open("w")
    server = subprocess.Popen([str(args.backend.resolve())], env=env, stdout=log, stderr=subprocess.STDOUT)
    apps = []
    report = {"passed": False, "evidence": str(root.resolve()), "checks": [], "scope": "Rinx decision UI and Rust Palpo; Matrix fixture; no live Hagency",
              "binary_sha256": hashlib.sha256(args.binary.read_bytes()).hexdigest(),
              "backend_sha256": hashlib.sha256(args.backend.read_bytes()).hexdigest()}
    try:
        for _ in range(100):
            if server.poll() is not None:
                raise RuntimeError("Rust service exited; inspect backend.log")
            try:
                with HTTP.open(endpoint + "/healthz", timeout=1):
                    break
            except OSError:
                time.sleep(.1)
        opened = post(endpoint, "session", "owner-secret", {"appId": "im.palpo.operations", "bundleDigest": "c" * 64, "services": SERVICES})
        token = opened["sessionToken"]
        definition = {"v": 1, "fleetId": fleet, "requestId": "a" * 40, "targetProjectId": "project_one",
            "targetRoomId": "!project:example.test", "sourceRoomId": "!reception:example.test", "sourceEventId": "$fixture-request",
            "ownerMxid": "@owner:example.test", "requesterMxid": "@owner:example.test", "ownerDmRoomId": "!private:example.test",
            "role": "developer", "requestedTokens": 100000, "agentDefinition": {"name": "Littlewhite", "instructions": "Help with the project"}}
        digest = hashlib.sha256(json.dumps(definition, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        request = {"kind": "agent", "request": {"id": "a" * 40, "revision": 1, "serverEngagementId": fleet,
            "projectId": "project_one", "projectRevision": 1, "resourceAllocationId": "grant_a", "projectOwner": "@owner:example.test",
            "requester": "@owner:example.test", "definitionDigest": digest, "requestedTokens": 100000}, "definition": definition}
        action = call(endpoint, token, "palpo.inbox.submit", request)["action"]
        owner = launch(root / "owner", args.binary, endpoint, "owner"); apps.append(owner)
        owner.click_id("waiting"); owner.wait_text("Littlewhite")
        owner.click_id("review"); owner.wait_text("Requested by @owner:example.test")
        assert not any(w.get("t") == "Approve" for w in owner.snap())
        owner.capture("owner-waiting")
        admin = launch(root / "admin", args.binary, endpoint, "admin"); apps.append(admin)
        admin.wait_text("No requests in this view")
        assert not any(w.get("t") == "Contribute resources" for w in admin.snap())
        admin.capture("admin-no-agent-authority")
        coordinator = launch(root / "coordinator", args.binary, endpoint, "coordinator"); apps.append(coordinator)
        coordinator.wait_text("Littlewhite"); coordinator.click_id("review"); coordinator.wait_text("Approve")
        coordinator.click_id("approve"); coordinator.wait_text("Decision reason")
        fill(coordinator, "Decision reason", "Approved within the engagement quota")
        before = inspect(coordinator)
        for theme in ("dark", "violet", "light"):
            coordinator.request("/event", data="palpo:" + theme, wait=1)
            coordinator.wait_text("Approved within the engagement quota")
            after = inspect(coordinator)
            assert (before["heap"], before["calls"]) == (after["heap"], after["calls"])
            coordinator.capture("coordinator-decision-" + theme)
        draft = json.loads((coordinator.root / "profile/app/draft.json").read_text())
        coordinator.click_id("submit"); coordinator.wait_text("No requests in this view")
        latest = call(endpoint, token, "palpo.inbox.get", {"id": action["id"]})["action"]
        assert latest["state"] == "approved" and latest["execution"] == "pending", latest
        coordinator_session = post(endpoint, "session", "coordinator-secret", {"appId": "im.palpo.operations", "bundleDigest": "d" * 64, "services": ["palpo.inbox.decide"]})
        replay = call(endpoint, coordinator_session["sessionToken"], "palpo.inbox.decide", draft["payload"])["action"]
        assert replay["id"] == action["id"] and replay["state"] == "approved"
        owner.click_id("latest"); owner.wait_text("agent · approved"); owner.wait_text("Execution · pending")
        owner.capture("owner-approved-awaiting-hagency")
        with sqlite3.connect(root / "admin.sqlite") as db:
            state = json.loads(db.execute("SELECT body FROM state WHERE id=1").fetchone()[0])
            assert len(state["rustWorkflows"]["outbox"]) == 1
            command = next(iter(state["rustWorkflows"]["outbox"].values()))["command"]
            assert command["context"]["actor"] == "@coordinator:example.test"
            assert db.execute("SELECT COUNT(*) FROM fleet_delivery WHERE lane='work'").fetchone()[0] == 1
        report["checks"] = ["manager cannot approve own agent", "Matrix admin has no implicit agent approval",
            "coordinator approves from the actual OctoScript form", "theme changes preserve draft and request count",
            "same command retry queues exactly one Hagency delivery", "owner sees approved and pending execution separately",
            "resource contribution is absent from the mini app"]
        report["passed"] = True
    finally:
        for app in apps:
            app.stop()
        server.terminate()
        try:
            server.wait(timeout=15)
        except subprocess.TimeoutExpired:
            server.kill(); server.wait(timeout=5)
        log.close()
        matrix.shutdown(); matrix.server_close()
        (root / "report.json").write_text(json.dumps(report, indent=2))
        print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
