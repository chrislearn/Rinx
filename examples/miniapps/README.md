# Octoscript mini apps in Rinx

The `matrix-octos-script` example uses the current OctoSense `main.splash` bundle format and calls `host.request` directly. It reads a Matrix profile, opens and hydrates its own session on the shared core, and sends a model turn.

The `matrix-octos` example is an ordinary L0 bundle: `page.card`, `page.data.json`, a kit, the OctoSense `manifest.json`, and declarative `bindings.json`. Rinx uses the shared Octoscript checker/lowerer for L0 and the shared App Hub entry resolver for `main.splash`, with native Makepad Splash widgets for both. Service results are live; the example does not substitute fixtures for Matrix or Octos.

## Browse and install

Open **Discover → Mini apps** (or **Mini apps** in the desktop sidebar).
**My apps** includes the built-in Article editor and installed Hub apps; **Recent**
lists apps opened with this Matrix account. **Browse** reads OctoSense's signed
App Hub catalog. Open a listing, review its publisher and permissions, and choose
**Add**. Installed apps show **Open**; a new published version shows **Update**.
Choose a conversation by name before opening if the app needs room access.
Closing the app revokes its session. Removing it keeps its saved documents.

The production catalog may be empty. Rinx never fills it with fabricated apps.
Unsupported host services, withdrawn versions and stale installation information
are shown explicitly. A previously verified installed app can reopen offline.

## Try a development bundle

1. Log in to Matrix in Rinx. On desktop, choose **Mini apps** in the sidebar. On mobile, choose **Discover → Mini apps**.
2. Choose **Developer**. In OctoSense, open AppCard once to connect its Octos core. Rinx captures that same connection when a mini app starts. Standalone Rinx offers an explicit Octos URL/profile/token form here.
3. Enter the full path of the `matrix-octos-script` or `matrix-octos` folder, choose **Review bundle**, and review its declared services. Leave the room empty for this example.
4. Choose **Run**, then **Read my Matrix profile**. Edit the prompt and choose **Ask Octos**. Back revokes the app session and interrupts its active turn.

For `main.splash`, service callbacks receive `{is_ok, data, error}`. The source and its `{{assets}}` URLs follow the same entry contract as App Hub. Script apps own their state and callbacks; `bindings.json` is only for L0.

Developer import accepts local unsigned bundles after explicit review. Signed
packages use the verified Hub catalog path. A2App `.splashapp` files remain
unsupported. Room grants apply only to the conversation selected during review.
Missing providers and failed calls are shown as errors. Services such as Mail
account management and Hub `agent` profiles are not exposed by this Rinx adapter.
Octos sessions narrow filesystem access to the app's data directory and disable
tool network access. A remote provider must resolve that directory on its host.
Bundled `os.*` apps do not gain implicit system trust when imported.

## Publish an app for Rinx and OctoSense

Use the [App Hub publication contract](https://github.com/OctoSense-org/OctoSense-App-Hub/blob/59004274ef0334b4fc25cd1ebac0caf80547c64a/docs/PUBLISHING.md):

1. Package `manifest.json`, `listing.json`, `main.splash` (or `page.card` and its
   kit), icon and screenshots. Declare the services the app uses and platforms
   on which it was tested.
2. Run the actual app, capture screenshots, then run `hub stamp`, `hub check`
   and `hub scan` on the final bundle. Sign with `hub sign-manifest` and check
   again with the publisher public key.
3. Commit and tag the source. Submit a Hub issue with the repository, full commit,
   bundle path, publisher public key and review results. A Hub maintainer admits
   and publishes the artifact and signed catalog. No automated publishing Action
   is available today.
4. Publish changes as a new version. Rinx verifies the same catalog, publisher
   signature and artifact digest, and asks the user to review the update.

Rinx's shared policy pin supports `matrix.*` and `octos.*`; production Hub main
does not yet contain those additions. They must reach the Hub gate before it can
admit apps declaring those services. Ordinary supported bundles already use the
same distribution format. See [ADR 0006](../../docs/adr/0006-shared-app-hub-miniapps.md)
for this upstream dependency and the exact division of responsibility.

## Bind an event to a service

```json
{
  "events": {
    "refresh": {"service": "matrix.profile", "args": {}, "target": "profile"},
    "ask": {"service": "octos.turn.start", "args": {"text": {"$state": "prompt"}}, "target": "answer"}
  }
}
```

Declare each service in `manifest.json` capabilities. Responses appear in the corresponding data field as `{"is_ok":true,"data":...}` or `{"is_ok":false,"error":...}`. Arguments can reference `{"$state":"field"}`, `{"$data":"/json/pointer"}` or `{"$value":true}`. A binding cannot provide account credentials, a core session ID, or an approval decision. Octos tool approvals use native host controls.

After editing bundle bytes, regenerate the digest:

```sh
cargo build --offline --manifest-path tools/miniapp-package/Cargo.toml
tools/miniapp-package/target/debug/rinx-miniapp-package examples/miniapps/matrix-octos
```

Each admitted instance runs from a verified snapshot. Editing the original package requires reviewing it again. Storage is confined to the current Matrix account and app ID. With a local shared core it lives beneath that core's configured data root, so session access can narrow the profile allowlist without widening it. Matrix-only / remote configurations use Rinx's data root. A custom core profile can still refuse the directory; Rinx does not relax that profile.

The kit files are from Octoscript revision `68f6a9df55692b5d8ef8873a12721e279a3f40d6`; their MIT license is included in `kit/LICENSE`. The Matrix contract and adapters derive from A2App revision `d4d39612fdee574a0f6a33480a19868f1ec85644`, under the license retained in `crates/miniapp-core/LICENSE-MIT`.
