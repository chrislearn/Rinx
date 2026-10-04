#!/usr/bin/env python3
"""Production Palpo Splash/host against delayed, failed and stale HTTP fixtures."""
import argparse, collections, json, pathlib, threading, time, uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from native_palpo import launch, port


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--binary', type=pathlib.Path, required=True)
    args = parser.parse_args(); root = pathlib.Path('target/palpo-connection-validation') / uuid.uuid4().hex
    root.mkdir(parents=True); counts = collections.Counter(); state = {'mode': 'pending', 'reads': 0, 'probing': False}
    identity = {'version': 1, 'userId': '@owner:example.test', 'isAdmin': False, 'serverName': 'example.test'}
    def fleet():
        ready = state['mode'] == 'ready'
        return {'id': 'fixture_fleet', 'name': 'Delayed proof fleet', 'state': 'ready' if ready else 'pending_connection',
                'ownerMxid': identity['userId'], 'readiness': {'eventDelivery': 'verified' if ready else 'unverified', 'verifiedAt': '2026-10-03T00:00:00Z' if ready else None},
                'transport': {'mode': 'outbound', 'online': True}}
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args): pass
        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers.get('Content-Length', 0))))
            status = 200
            if self.path.endswith('/session'): result = dict(identity, sessionToken='fixture-app-session')
            elif self.path.endswith('/disconnect'): result = {'disconnected': True}
            else:
                service = body['service']; counts[service] += 1
                if service == 'palpo.session.open': result = identity
                elif service == 'palpo.inbox.list': result = {'actions': [], 'pendingCount': 0, 'total': 0}
                elif service == 'palpo.fleets.list':
                    if state['probing']:
                        state['reads'] += 1
                        if state['mode'] == 'pending' and state['reads'] >= 2: state['mode'] = 'ready'
                    result = {'fleets': [fleet()]}
                elif service == 'palpo.fleets.connect':
                    state['reads'] = 0; state['probing'] = True
                    if state['mode'] == 'error': status = 409; result = {'code': 'event_delivery_unverified', 'error': 'Hagency has not verified this connection.'}
                    else: result = fleet()
                else: status = 400; result = {'code': 'unexpected_service', 'error': service}
            data = json.dumps(result).encode(); self.send_response(status); self.send_header('Content-Type','application/json'); self.send_header('Content-Length', str(len(data))); self.end_headers(); self.wfile.write(data)
    server = ThreadingHTTPServer(('127.0.0.1', port()), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start(); app = None
    report = {'passed': False, 'checks': [], 'evidence': str(root.resolve())}
    try:
        app = launch(root/'owner', args.binary, f'http://127.0.0.1:{server.server_port}')
        app.wait_text('Fleets'); app.click_id('fleets'); app.wait_text('Connection not yet verified'); app.click_id('verify')
        app.wait_text('Waiting for Hagency to verify')
        assert not any(w.get('t','').startswith('Connection verified') for w in app.snap())
        app.capture('pending-proof'); app.wait_text('Connection verified', timeout=15)
        assert counts['palpo.fleets.connect'] == 1
        assert not any(w.get('i') == 'verify' for w in app.snap())
        app.click_id('inbox'); app.wait_text('No requests in this view'); app.click_id('fleets'); app.wait_text('Connection verified')
        app.capture('verified-proof'); report['checks'].append('pending proof polls read-only, completes without another Verify and persists on reopen')
        state.update(mode='stuck', reads=0, probing=False)
        app.click_id('refresh'); app.wait_text('Connection not yet verified'); app.click_id('verify'); app.wait_text('Waiting for Hagency')
        app.click_id('inbox'); app.wait_text('No requests in this view'); before = counts['palpo.fleets.list']; time.sleep(2.5)
        assert counts['palpo.fleets.list'] == before, counts
        assert not any('Verification' in w.get('t','') for w in app.snap())
        report['checks'].append('navigation cancels pending polling and prevents stale status replacing the next page')
        state.update(mode='error', reads=0, probing=False)
        app.wait_text('Fleets'); app.click_id('fleets'); app.wait_text('Connection not yet verified'); app.click_id('verify'); app.wait_text('event_delivery_unverified:')
        assert not any(w.get('t','').startswith('Connection verified') for w in app.snap())
        report['checks'].append('server refusal remains an error, never a verified result')
        errors = [line for line in (app.output/'native.log').read_text().splitlines() if '[E]' in line or 'on_render closure failed' in line]
        assert not errors, errors
        report['passed'] = True
    finally:
        if app: app.stop()
        server.shutdown(); server.server_close()
        report['calls'] = dict(counts); (root/'report.json').write_text(json.dumps(report, indent=2)+'\n'); print(json.dumps(report,indent=2))

if __name__ == '__main__': main()
