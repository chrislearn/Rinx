#!/usr/bin/env python3
"""Actual production Splash + Rinx host adapter + local Palpo backend.

No live account or deployment is used. Matrix is the Palpo test fixture;
SQLite workflows, HTTP sessions, Rinx transport, widgets and inputs are real.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import time
import uuid
import urllib.error
from PIL import Image
from native_probe import NativeApp

class PalpoApp(NativeApp):
    def snap(self):
        return [w for w in super().snap() if w.get('ty') != 'Splash']

    def click_id(self, widget_id):
        # Long forms retain their real scroll container. A zero rectangle is
        # offscreen, not a coordinate at which an input can be delivered.
        height = self.request('/s')['w'][0]['sz'][1]
        self.request('/m', k='scroll', x=300, y=650, dy=-2000, wait=1)
        for _ in range(16):
            widgets = [w for w in self.request('/snap', all=1)['s']
                       if w.get('i') == widget_id and w['r'][2] > 0 and w['r'][3] > 0
                       and 30 < w['r'][1] and w['r'][1] + w['r'][3] <= height - 20]
            if widgets:
                x, y, width, height = widgets[0]['r']
                self.click(x + width / 2, y + height / 2)
                return
            self.request('/m', k='scroll', x=300, y=650, dy=220, wait=1)
        raise AssertionError(f'Could not scroll widget into view: {widget_id}')

    def request(self, route, **params):
        # SDK 1f3b1de can fail wait=1 after already applying the input when a
        # hidden Metal frame cannot immediately submit. Never replay that input.
        # A separate read-only grab is the frame barrier instead.
        barrier = route in {'/click', '/m', '/k', '/t'} and params.get('wait') == 1
        if barrier:
            params['wait'] = 0
        try:
            result = super().request(route, **params)
            if barrier:
                time.sleep(.05)
                super().request('/g')
            return result
        except urllib.error.HTTPError as error:
            detail = error.read().decode('utf-8', errors='replace')
            self.trace.append({'bridge_error': route, 'status': error.code, 'detail': detail})
            raise NativeBridgeError(f'{route}: HTTP {error.code}: {detail}') from error


class NativeBridgeError(RuntimeError):
    pass


def native_errors(app):
    return [line for line in (app.output / 'native.log').read_text().splitlines()
            if any(marker in line for marker in ('[E]', 'on_render closure failed', 'callback error',
                                                  'script time budget exceeded', 'spent its instruction budget',
                                                  'script body evaluated to nothing'))]


def port():
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]


def launch(root, binary, endpoint, admin=False, narrow=False, session_file=None, project_admin=False):
    profile = root / 'profile'
    (profile / 'app').mkdir(parents=True, exist_ok=True)
    app = PalpoApp(root, port=port(), auto_login=False)
    app.output.mkdir(parents=True)
    app.log = (app.output / 'native.log').open('w')
    env = dict(os.environ, RINX_DATA_DIR=str(profile.resolve()), MAKEPAD_HIDE_WINDOWS='1',
               MAKEPAD_NO_FOCUS='1', MAKEPAD_REMOTE=str(app.port), PALPO_FIXTURE_URL=endpoint)
    env.pop('MAKEPAD_FOCUS', None)
    env.pop('PALPO_LIVE_SESSION_FILE', None)
    if session_file:
        env['PALPO_LIVE_SESSION_FILE'] = str(session_file.resolve())
    args = [str(binary.resolve())] + (['--admin'] if admin else ['--project-admin'] if project_admin else []) + (['--narrow'] if narrow else [])
    app.process = subprocess.Popen(args, env=env, stdout=app.log, stderr=subprocess.STDOUT)
    try:
        for _ in range(120):
            if app.process.poll() is not None:
                raise RuntimeError(f'Native fixture exited; see {app.output}')
            errors = native_errors(app)
            if errors:
                raise RuntimeError('\n'.join(errors))
            try:
                status = app.request('/s')
                if status['pid'] != app.process.pid:
                    app._interrupt('Bridge belongs to a different process; refusing input and cleanup')
                if status['w']:
                    app.wait_text('Palpo administrator' if admin else 'Project administrator' if project_admin else 'Project manager', timeout=10)
                    return app
            except (OSError, NativeBridgeError):
                pass
            time.sleep(.1)
        raise RuntimeError(f'Native bridge did not draw: {app.output}')
    except BaseException:
        # The caller cannot track an app that never returned from launch.
        # Preserve startup evidence and use the same activity-guarded cleanup.
        try:
            app.capture('startup-failure')
            (app.root / 'final-tree.json').write_text(json.dumps(app.request('/snap', all=1), indent=2))
        except Exception:
            pass
        finally:
            app.stop()
        raise


def fill(app, label, value):
    # Start at the top so offscreen labels (zero rects) cannot select a visible
    # neighboring input. Walk down using the input following the exact label.
    app.request('/m', k='scroll', x=300, y=650, dy=-2000, wait=1)
    for _ in range(16):
        tree = app.request('/snap', all=1)['s']
        (app.root / 'last-fields.json').write_text(json.dumps(tree, indent=2))
        labels = [i for i, w in enumerate(tree) if w.get('t') == label and w.get('ty') != 'TextInput']
        assert labels, (label, app.snap())
        target = next(w for w in tree[labels[-1] + 1:] if w.get('ty') == 'TextInput')
        if target['r'][2] > 0 and 160 < target['r'][1] < 720:
            break
        app.request('/m', k='scroll', x=300, y=650, dy=220, wait=1)
    else:
        raise AssertionError(f'Could not scroll field into view: {label}')
    x, y, width, height = target['r']
    app.click(x + width / 2, y + height / 2)
    app.request('/k', c='KeyA', cmd=1, wait=1)
    app.request('/t', t=value, wait=1)


def inspect(app):
    app.request('/event', data='palpo:inspect', wait=1)
    return json.loads((app.root / 'profile/inspection.json').read_text())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--palpo', type=Path, required=True)
    parser.add_argument('--node', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/fast/examples/palpo_miniapp'))
    parser.add_argument('--smoke', action='store_true')
    args = parser.parse_args()
    root = Path('target/palpo-validation') / uuid.uuid4().hex
    root.mkdir(parents=True)
    service_port = port()
    log = (root / 'server.log').open('w')
    server = subprocess.Popen([str(args.node), str(args.palpo / 'web-admin/test/miniapp-native-fixture.mjs'), str(service_port), str(root.resolve())], stdout=log, stderr=subprocess.STDOUT)
    apps = []
    report = {'passed': False, 'evidence': str(root.resolve()), 'checks': [],
              'binary_sha256': hashlib.sha256(args.binary.read_bytes()).hexdigest(),
              'bundle_sha256': {name: hashlib.sha256((Path('apps/palpo/bundle') / name).read_bytes()).hexdigest()
                                for name in ('main.splash', 'manifest.json')}}
    try:
        for _ in range(80):
            if 'ready' in (root / 'server.log').read_text(): break
            if server.poll() is not None: raise RuntimeError((root / 'server.log').read_text())
            time.sleep(.1)
        endpoint = f'http://127.0.0.1:{service_port}'
        # A draft saved by the previous bundle has no administrator-policy
        # properties. It must remain readable without a Splash missing-key error.
        legacy_dir = root / 'legacy-admin/profile/app'
        legacy_dir.mkdir(parents=True)
        legacy_draft = {'title': 'Approve request', 'service': 'palpo.inbox.decide',
                        'payload': {'id': 'action_' + 'a' * 32, 'expectedRevision': 1,
                                    'commandId': 'legacy_decision', 'decision': 'approve', 'reason': 'Saved review'},
                        'fields': [{'key': 'reason', 'label': 'Decision reason', 'value': 'Saved review'}]}
        (legacy_dir / 'draft.json').write_text(json.dumps(legacy_draft))
        legacy = launch(root / 'legacy-admin', args.binary, endpoint, admin=True); apps.append(legacy)
        legacy.click_id('resume'); legacy.wait_text('Saved review')
        legacy.capture('legacy-decision-draft')
        report['checks'].append('legacy approval draft remains readable without inventing administrator assignments')
        owner = launch(root / 'owner', args.binary, endpoint, narrow=True); apps.append(owner)
        owner.capture('owner-inbox-light')
        assert not any(w.get('i') in {'contribute', 'fleets', 'approve', 'register'} for w in owner.snap())
        report['checks'].append('manager has no contribution, fleet administration or project approval controls')
        owner.click_id('resources'); owner.wait_text('Request project using this resource'); owner.click_id('choose')
        owner.wait_text('Project name')
        fill(owner, 'Project name', 'Native test project')
        fill(owner, 'What will your project do?', 'Shared coding capacity')
        fill(owner, 'Project token budget', '400000')
        fill(owner, 'Maximum concurrent agents', '4')
        fill(owner, 'Combined tokens per day', '40000')
        fill(owner, 'Allocation duration (hours)', '24')
        fill(owner, 'What will your project do?', 'Shared coding capacity')
        owner.capture('project-draft')
        owner.request('/k', c='ArrowLeft', shift=1, wait=1)
        owner.request('/k', c='ArrowLeft', shift=1, wait=1)
        before = inspect(owner)
        for theme in ('dark', 'violet', 'light'):
            owner.request('/event', data='palpo:' + theme, wait=1)
            time.sleep(.25)
            owner.wait_text('Shared coding capacity')
            after = inspect(owner)
            assert (before['heap'], before['calls']) == (after['heap'], after['calls']), (before, after)
            capture = owner.capture('project-' + theme)
            im = Image.open(capture).convert('RGB')
            bright = sum(1 for rgb in im.getdata() if sum(rgb) > 540) / (im.width * im.height)
            assert (bright < .2 if theme == 'dark' else bright > .65), (theme, bright)

        owner.request('/t', t='XY', wait=1)
        edited = json.loads((owner.root / 'profile/app/draft.json').read_text())
        assert edited['payload']['reason'] == 'Shared coding capaciXY', edited
        owner.request('/k', c='KeyZ', cmd=1, wait=1)
        restored = json.loads((owner.root / 'profile/app/draft.json').read_text())
        assert restored['payload']['reason'] == 'Shared coding capacity', restored
        report['checks'].append('live theme changes preserve draft, focus, selection, undo, isolate and request count; dark pixels verified')
        if not args.smoke:
            # A process restart restores the same draft and trusted request ID.
            draft_before = json.loads((owner.root / 'profile/app/draft.json').read_text())
            owner.stop(); apps.remove(owner)
            owner = launch(root / 'owner', args.binary, endpoint, narrow=True); apps.append(owner)
            owner.click_id('resume'); owner.wait_text('Native test project')
            assert json.loads((owner.root / 'profile/app/draft.json').read_text()) == draft_before
            report['checks'].append('draft and idempotency key survive process restart')
            owner.click_id('submit'); owner.wait_text('Open latest result')
            admin = launch(root / 'admin', args.binary, endpoint, admin=True); apps.append(admin)
            admin.wait_text('Native test project'); admin.click_id('review'); admin.wait_text('Approve')
            admin.click_id('approve'); admin.wait_text('Decision reason')
            fill(admin, 'Project administrators (Matrix IDs, comma separated)', '@other:example.test')
            assert any(w.get('t') == 'Self-approval: forbidden' for w in admin.snap())
            fill(admin, 'Decision reason', 'Approved for research')
            admin.capture('project-budget-administrator-policy')
            admin.click_id('submit'); admin.wait_text('No requests in this view')
            admin.capture('designated-admin-project-approvals')
            assert any('Project approvals' in w.get('t', '') for w in admin.snap())
            assert not any(w.get('i') in {'contribute', 'register'} for w in admin.snap())
            owner.click_id('waiting'); owner.wait_text('Native test project'); owner.click_id('review')
            assert not any(w.get('i') in {'approve', 'reject'} for w in owner.snap())
            owner.wait_text('waiting for Hagency to reserve')
            owner.capture('project-awaiting-reservation')
            report['checks'].append('designated admin approves the managers project; ownership remains with requester')
            (root / 'release-reservations').touch()
            for _ in range(20):
                owner.click_id('latest')
                if any('Project allocated' in w.get('t', '') for w in owner.snap()):
                    break
                time.sleep(1)
            owner.wait_text('Project allocated')
            owner.capture('project-reservation-confirmed')
            owner.click_id('projects'); owner.wait_text('Native test project'); owner.click_id('agent')
            owner.wait_text('Use this resource'); owner.click_id('choose'); owner.wait_text('Agent name')
            fill(owner, 'Agent name', 'ResearchBot')
            # Scroll the form's last fields and submit into the viewport.
            owner.request('/m', k='scroll', x=300, y=650, dy=350, wait=1)
            owner.click_id('submit'); owner.wait_text('Initial request:')
            owner.capture('agent-request-pending')
            backend = json.loads((root / 'backend.json').read_text())
            assert backend['projects'] == 1 and backend['requests'] == 1, backend
            report['checks'].append('finite project budget, explicit administrator policy, pending reservation and applied receipt through native forms')
            owner.click_id('review'); owner.wait_text('Waiting for the assigned project administrator')
            assert not any(w.get('i') in {'approve', 'reject'} for w in owner.snap())
            assigned = launch(root / 'project-admin', args.binary, endpoint, project_admin=True); apps.append(assigned)
            assigned.wait_text('ResearchBot'); assigned.click_id('review'); assigned.wait_text('Approve')
            assigned.capture('assigned-admin-agent-review')
            assigned.click_id('approve'); assigned.wait_text('Approved tokens')
            fill(assigned, 'Approved tokens', '80000')
            fill(assigned, 'Decision reason', 'Approved within the research budget')
            assigned.capture('assigned-admin-agent-allocation')
            assigned.click_id('submit'); assigned.wait_text('No requests in this view')
            owner.click_id('latest'); owner.wait_text('Decision recorded')
            owner.capture('agent-awaiting-hagency')
            (root / 'release-agent-decisions').touch()
            for _ in range(20):
                owner.click_id('latest')
                if any('Approval applied' in w.get('t', '') for w in owner.snap()): break
                time.sleep(1)
            owner.wait_text('Approval applied'); owner.capture('agent-decision-applied')
            backend = json.loads((root / 'backend.json').read_text())
            assert backend['requestStates'] == [{'state': 'provisioning', 'usable': False}], backend
            report['checks'].append('assigned project administrator decides in native Inbox; owner cannot approve; applied fixture receipt does not claim a ready agent')
            (root / 'release-agent-decisions').unlink()
            owner.click_id('agent_status'); owner.wait_text('Confirmed allocation: 80000')
            assert not any(w.get('i') == 'open_agent_chat' for w in owner.snap())
            (root / 'publish-ready-agent').touch()
            for _ in range(20):
                owner.click_id('requests'); owner.wait_text('Confirmed allocation: 80000')
                if any(w.get('i') == 'open_agent_chat' for w in owner.snap()): break
                time.sleep(1)
            owner.capture('ready-agent-chat')
            before = json.loads((root / 'backend.json').read_text())['matrixMutations']
            owner.click_id('open_agent_chat'); owner.wait_text("Opening the agent's project room in Rinx")
            navigation = json.loads((owner.root / 'profile/agent-navigation.json').read_text())
            time.sleep(0.2)
            backend = json.loads((root / 'backend.json').read_text())
            assert navigation == backend['agentChats'][0], (navigation, backend['agentChats'])
            assert backend['matrixMutations'] == before, backend
            report['checks'].append('explicit ready-agent fixture opens its account-bound project room through the native adapter without a Matrix mutation; no live timeline claimed')
            (root / 'publish-ready-agent').unlink()
            owner.click_id('top_up'); owner.wait_text('Additional tokens')
            fill(owner, 'Additional tokens', '50000')
            fill(owner, 'Why do you need more tokens?', 'Continue research on the same agent')
            owner.click_id('submit'); owner.wait_text('Token increase')
            owner.click_id('review'); owner.wait_text('Token increase')
            assigned.click_id('needs'); assigned.wait_text('Continue research on the same agent')
            assigned.click_id('review'); assigned.click_id('approve'); assigned.wait_text('Approved additional tokens')
            fill(assigned, 'Approved additional tokens', '40000')
            fill(assigned, 'Decision reason', 'Approve the smaller increase')
            assigned.capture('assigned-admin-token-increase')
            assigned.click_id('submit'); assigned.wait_text('No requests in this view')
            owner.click_id('latest'); owner.wait_text('Token increase approved')
            owner.capture('token-increase-awaiting-hagency')
            (root / 'release-agent-decisions').touch()
            for _ in range(20):
                owner.click_id('latest')
                if any('Token increase applied' in w.get('t', '') for w in owner.snap()): break
                time.sleep(1)
            owner.wait_text('Token increase applied'); owner.click_id('agent_status')
            owner.wait_text('Confirmed allocation: 120000'); owner.capture('same-agent-increased-allocation')
            backend = json.loads((root / 'backend.json').read_text())
            assert backend['requests'] == 1 and backend['allocations'] == [{'tokens': 120000, 'pendingTokens': 0}], backend
            report['checks'].append('owner requests more tokens; assigned administrator approves a smaller increase; one agent receives one applied increase')
            owner.click_id('remove'); owner.wait_text('Reason for removal')
            owner.wait_text('Chat history stays available')
            fill(owner, 'Reason for removal', 'Research complete')
            owner.capture('confirm-agent-removal'); owner.click_id('submit')
            owner.click_id('review'); owner.wait_text('Removal requested')
            assert not any(w.get('i') in {'approve', 'reject'} for w in owner.snap())
            (root / 'release-agent-removals').touch()
            for _ in range(20):
                owner.click_id('latest')
                if any('waiting for runtime and Matrix cleanup' in w.get('t', '') for w in owner.snap()): break
                time.sleep(1)
            owner.wait_text('waiting for runtime and Matrix cleanup'); owner.capture('removal-cleanup-pending')
            (root / 'fail-agent-cleanup').touch()
            for _ in range(20):
                owner.click_id('latest')
                if any('Cleanup failed' in w.get('t', '') for w in owner.snap()): break
                time.sleep(1)
            owner.wait_text('Cleanup failed'); owner.capture('removal-cleanup-failed')
            owner.click_id('retry_remove'); fill(owner, 'Reason for removal', 'Retry the failed cleanup')
            owner.click_id('submit'); owner.click_id('review')
            (root / 'release-agent-cleanup').touch()
            for _ in range(20):
                owner.click_id('latest')
                if any('Agent removed' in w.get('t', '') for w in owner.snap()): break
                time.sleep(1)
            owner.wait_text('Agent removed'); owner.capture('agent-removal-verified')
            owner.click_id('agent_status'); owner.wait_text('Token usage: unknown')
            assert not any(w.get('i') in {'top_up', 'remove'} for w in owner.snap())
            backend = json.loads((root / 'backend.json').read_text())
            assert backend['requests'] == 1 and backend['removalCommands'] == 2, backend
            assert backend['requestStates'] == [{'state': 'removed', 'usable': False}], backend
            report['checks'].append('owner removal needs no second approval; partial cleanup stays pending; definitive failure retries once and runtime plus Matrix proof complete removal')
            admin.click_id('accounts'); admin.wait_text('Alice 张')
            admin.capture('signup-request-list')
            admin.click_id('open_signup'); admin.wait_text('Opening the original approval request in Rinx')
            navigation = json.loads((admin.root / 'profile/signup-navigation.json').read_text())
            signup = json.loads((root / 'backend.json').read_text())['signup']
            assert navigation == {'account': '@admin:example.test', 'roomId': signup['roomId'], 'eventId': signup['eventId']}, (navigation, signup)
            assert signup['status'] == 'pending' and signup['registrations'] == 0, signup
            report['checks'].append('signup page resolves the original account-bound Matrix event through the native adapter; opening sends no verdict')
            for mode in ('retry', 'release'):
                (root / f'seed-{mode}-project').touch()
                admin.click_id('inbox'); admin.click_id('needs')
                for _ in range(20):
                    admin.click_id('refresh')
                    if any(f'Recovery {mode} project' in w.get('t', '') for w in admin.snap()): break
                    time.sleep(1)
                admin.wait_text(f'Recovery {mode} project'); admin.click_id('review')
                admin.wait_text('Budget reservation refused')
                admin.wait_text('Recovery analysis resource')
                admin.wait_text('The contribution cannot currently cover this budget.')
                admin.capture(f'partial-reservation-{mode}')
                admin.click_id('retry_reservation' if mode == 'retry' else 'release_reservation')
                fill(admin, 'Recovery reason', 'Recover the partial allocation')
                admin.capture(f'confirm-project-{mode}'); admin.click_id('submit')
                admin.click_id('review')
                admin.wait_text('waiting for Hagency' if mode == 'retry' else 'Releasing unused reservations')
                admin.capture(f'project-{mode}-awaiting-receipt')
                (root / ('apply-project-recovery' if mode == 'retry' else 'apply-unused-releases')).touch()
                for _ in range(20):
                    admin.click_id('latest')
                    wanted = 'Project allocated' if mode == 'retry' else 'unused reservations released'
                    if any(wanted in w.get('t', '') for w in admin.snap()): break
                    time.sleep(1)
                admin.wait_text(wanted); admin.capture(f'project-{mode}-confirmed')
                data = json.loads((root / 'backend.json').read_text())
                recovery = next(r for r in data['recoveries'] if r['name'] == f'Recovery {mode} project')
                assert recovery['owner'] == '@owner:example.test', recovery
                assert recovery['allocated'] == (mode == 'retry'), recovery
                assert recovery['commands'] == ['reserve_project', 'reserve_project', 'reserve_project' if mode == 'retry' else 'release_unused_project'], recovery
                assert recovery['execution'] == ('done' if mode == 'retry' else 'released'), recovery
                report['checks'].append(f'partial project {mode} uses native confirmation, waits for its exact fixture receipt and preserves the owner')
            owner.click_id('inbox'); owner.click_id('history')
            owner.wait_text('Recovery release project'); owner.click_id('review')
            owner.wait_text('unused reservations released')
            assert not any(w.get('i') in {'retry_reservation', 'release_reservation'} for w in owner.snap())
            owner.click_id('new_project'); owner.wait_text('Hagency contributes finite budgets')
            owner.capture('released-project-owner-returns-to-resources')
            owner.click_id('disconnect'); owner.wait_text('Rinx remains signed in')
            assert json.loads((root / 'backend.json').read_text())['logouts'] == 0
            report['checks'].append('mini-app disconnect preserves Matrix login')

        for app in apps:
            errors = native_errors(app)
            assert not errors, errors
        report['passed'] = True
    finally:
        for app in apps:
            try:
                app.capture('final-state')
                (app.root / 'final-tree.json').write_text(json.dumps(app.request('/snap', all=1), indent=2))
            except Exception: pass
            finally: app.stop()
        server.terminate()
        try: server.wait(timeout=8)
        except subprocess.TimeoutExpired: server.kill(); server.wait()
        log.close()
        (root / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))

if __name__ == '__main__':
    main()
