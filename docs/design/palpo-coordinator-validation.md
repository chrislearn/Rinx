# Coordinator workflow integration

This implements the decision-screen portion of [ADR 0011](../adr/0011-hagency-server-engagements.md).
It replaces the earlier resource-contribution action: resources are contributed
by their Hagency owner, while Rinx handles requests and coordinator decisions.

The UI labels Matrix administrator and engagement coordinator separately. It
renders only navigation granted by the current server session. Rust Palpo accepts
the reviewed manifest but grants only services already implemented, so the app
can connect during migration without advertising unsupported operations.
Approval state and execution state are displayed separately. Contribution drafts
from the earlier workflow are retained locally but cannot be resumed.

The production OctoScript form sends a decision intent and stable command ID.
Rust Palpo constructs the authority envelope from its authenticated Matrix
session and frozen request. Retrying cannot create another Hagency delivery.

## Validated locally

On 2026-10-04, the actual Makepad instrument host, production Splash bundle and
Rinx HTTP adapter ran against the Rust `palpo-operations` executable. Matrix
authentication and existing engagement/project authority were explicit fixtures.
Three hidden windows used isolated manager, coordinator and administrator data
directories. No existing user session or deployment was opened.

Run `6d7e0802975f493e96e167b94de1b01c` passed all seven checks:

- The project manager can read the agent request but cannot approve it.
- The Matrix administrator has no implicit agent-approval authority.
- The assigned coordinator approves through the real OctoScript form.
- Light, dark and custom themes preserve draft contents, script heap and call count.
- Repeating the command creates exactly one outbound Hagency work item.
- The manager sees `approved` alongside `Execution · pending`, not a live agent.
- Resource contribution is absent from the mini app.

Real captures were inspected: the narrow owner result, desktop coordinator dark
form and administrator Inbox. Runtime logs had no Splash evaluation/callback
errors. Evidence remains under `target/palpo-coordinator-validation/<run>/`,
including the report, binary hashes, input traces, widget captures and screenshots.

```sh
# Build the paired Palpo Rust branch first.
cargo build --profile fast --locked --example palpo_miniapp
python3 tools/wechat-ux/live/native_palpo.py \
  --backend /path/to/palpo/target/debug/palpo-operations \
  --binary target/fast/examples/palpo_miniapp
```

`tools/octo check` restamped the manifest and resolved its declared capabilities.
Its store-publication check refused the missing `listing.json`, as expected for
this built-in app with no publisher/platform submission. No listing or identity
was fabricated. Rinx's own bundle packaging/build validation passed.

## Remaining acceptance

This is a draft integration, not the full ADR implementation. The Rust backend
still needs association/profile issuance, room preparation and the remaining
resource/project/agent navigation services. Those unsupported entries are hidden.
The real Hagency runtime has separate tests; this UI run does not establish
combined provisioning, live chat, usage metering or notification delivery.
Android, OpenHarmony and hosted OctoSense also require their own device evidence.

Companion changes: [Palpo #508](https://github.com/palpo-im/palpo/pull/508) and
[Hagency #29](https://github.com/hagency-org/hagency-rs/pull/29).
