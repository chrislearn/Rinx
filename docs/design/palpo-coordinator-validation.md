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

Projects and Agents use the Rust service's role-scoped, paginated read models.
Approved agents appear while allocation is still pending. Missing consumption is
explicitly unreported; reported consumption is a lower bound with freshness,
never an invented remaining balance. Stale readiness becomes unknown. Matrix
administrators do not inherit access to other users' agent lists.

The agent owner can request additional tokens from a current allocation. Rust
binds the form to that agent, project and resource grant; the coordinator reviews
the exact increase in Inbox. Hagency execution is still required before the
displayed allocation increases. A retry returns the original request after the
allocation has changed. Stale provider observations hide the top-up action.

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

The extended run `532acbbcffc4484288443239c3d1f04e` passed eleven checks,
adding Projects/Agents navigation, pending allocation/unknown usage, current
lower-bound usage, and stale observations. Provider updates are explicit fixtures
sent through the real authenticated machine HTTP route. The pending, current and
stale agent cards were visually inspected. Four Rust Palpo host tests also pass.

Run `ef8d10a573c5479a868109c266b2d5b2` passed all fourteen checks. It adds
native owner top-up submission, native coordinator approval, and replay after
an authenticated provider fixture reports execution. The form, approval and
120,000-token result captures were inspected. This proves the UI/HTTP path;
the provider fixture does not reserve resources or execute a real Hagency agent.

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
still needs the remaining lifecycle/admin routes and migration acceptance.
Resource browsing and project/agent creation now use the paired Rust routes with
explicit allocation IDs; unsupported entries remain hidden.
The real Hagency runtime has separate tests; this UI run does not establish
combined provisioning, live chat, usage metering or notification delivery.
Android, OpenHarmony and hosted OctoSense also require their own device evidence.

Companion changes: [Palpo #508](https://github.com/palpo-im/palpo/pull/508) and
[Hagency #29](https://github.com/hagency-org/hagency-rs/pull/29).

## Continued implementation, 2026-10-04

The manager's forms now distinguish the engagement allocation from its parent
resource. Rust Palpo prepares owner-authorized Matrix rooms with durable recovery
and freezes the definition before coordinator review. Its HTTP tests cover a
lost createRoom reply, a lost agent event response followed by a new Matrix login,
conflicting retries, funding/role checks and automatic command queueing. The
paired Rust notification worker adds private My Actions delivery, reminders,
quiet hours, privacy revalidation and a pinned Inbox summary.

The desktop window host from commit `9927786e` is integrated here. Real Makepad
window input verified opening, OS close, Back, reopening, theme reapplication and
preservation of the main chat draft/display context. The light catalog and dark
import screenshots were inspected. Evidence:
`target/miniapp-window-validation/51e566f3bcdc4f78a7a3a8a8e128a3e0/report.json`.

The coordinator UI scenario also passed against the rebuilt Rust process,
including the original fourteen checks. Its evidence is
`target/palpo-coordinator-validation/5e9b1bdbeb974031a58b7f039b051f68/report.json`.
The coordinator decision screenshot was inspected. These runs use a Matrix HTTP
fixture; they do not claim combined live Hagency or mobile acceptance.

## Owner association and native connection probe

The paired Rust services now implement owner initiation, one designated Matrix
admin decision, resumable appservice installation, explicitly scoped profile
retrieval and generation-bound connection verification. The Rinx association
action exposes setup retry and authorized export; Engagements separates the last
proof time from current heartbeat connectivity.

`native_palpo_association.py` runs actual Rinx, Rust Palpo and native Hagency
processes against an isolated Matrix HTTP fixture. Hagency's CLI persists and
retries the association, the real OctoScript admin form approves it, native
Hagency imports the matching profile, and the owner's Rinx Verify action sends
the probe. Hagency consumes both delivery lanes and publishes the authenticated
receipt. All eight checks passed; both screenshots were inspected at
`target/palpo-association-validation/93b07861d5bb4ca5b0bc4ddfa0a950b9/report.json`.
This covers native connection proof, not agent execution, live Matrix or a native
save-dialog interaction: the harness retrieves the authorized profile through
the API and writes its private import file.

## Delivered refusals and current delegation

The agent list now retains the coordinator's approval and shows Hagency's
terminal allocation refusal separately, with a readable reason and no pending
allocation claim. Association-only buttons are guarded on non-association
records. The native scenario drives a second approval and its authenticated
capacity refusal, then scrolls the narrow owner list to inspect that result.
All fifteen checks passed and the refusal screenshot was reviewed:
`target/palpo-coordinator-validation/7c195bb18a92451a9b0385a806c090cb/report.json`.

The paired Hagency console now edits the single engagement resource ledger
from both Resources and Server engagements, and displays delivered agent
approvals before Matrix admission. It also records owner delegation revisions,
with suspension/revocation, explicit export recipients and durable publication.
Palpo applies those revisions to current approval/export authority and notices.
These validations remain isolated fixture evidence, not production cutover or
device acceptance.
