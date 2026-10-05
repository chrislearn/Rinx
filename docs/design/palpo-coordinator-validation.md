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

## Scoped agent lifecycle

The owner can pause, resume, remove and retry definitively failed cleanup through
native forms when the runtime advertises `coordinatorAgentControlV1`. A pending
command, accepted retirement and verified cleanup are separate projections.
Retired history retains its allocation and usage; unknown consumption is not
refunded by retirement. Final accounted usage is labelled separately from runtime
lower-bound observations.

The eighteen-check instrument run passed at
`target/palpo-coordinator-validation/bba39c472bba48da8e45cc259d889739/report.json`.
It drives real native forms and Rust Palpo with authenticated provider fixtures;
the retired-history screenshot was inspected. An earlier hidden-window run
missed the initial Waiting input before reaching the lifecycle flow; failed runs
remain recorded. This evidence does not claim live executor termination.

## Account notification preferences

Notifications now exposes account-scoped delivery/reminder switches, reminder
times and quiet hours, with the native device time zone as an optional suggestion.
The Rust service retains revisioned settings and checks daylight-saving wall
time. Disabled delivery does not decide a pending action; overdue reminders are
combined into one delivery when service resumes.

The native run passed twenty checks at
`target/palpo-coordinator-validation/a66cfbd69adb479aa3dfa58e65d9f7ac/report.json`.
The settings screenshot was inspected. This run saves the coordinator's settings,
reopens them and verifies that the owner's settings remain independent. The
startup harness now waits for the initial Inbox HTTP request to settle before
clicking Waiting, fixing the previously recorded missed input while busy.

## Verified room board and native navigation

My Actions mounts the installed bundle only after exact-bundle consent and a
current private-room/account verification. The native host rechecks the binding
on each service call and on refresh; account changes revoke it and restore chat
history. Agent chat navigation accepts only the service-derived room after both
the user and agent are joined. Signup navigation retains the original approval
event through an invitation join, with account and expiry checks.

The combined native run passed all 24 checks at
`target/palpo-coordinator-validation/cf7023ce7a614e1d802911d2ae987f15/report.json`.
The actual embedded board, theme switch and account-switch screenshots were
inspected. The host suite passed 25 checks and shell navigation passed six.
The instrument uses real Rinx and Rust Palpo processes with isolated Matrix and
provider fixtures; it does not establish live Hagency execution or mobile
acceptance. The scroll helper now requires a visible hit target before clicking.

## Rust signup approval worker

The Signups screen now opens the original verified Matrix approval event through
its explicit native navigation grant. A navigation call sends no verdict. The
Rust worker consumes the bound Matrix decision and projects registration only
after a confirmed ordinary account or original-device reconciliation.

All five native checks passed at
`target/palpo-signup-validation/ffd93453b2d64a028adf42d8cfe0aec5/report.json`;
the pending handoff and terminal registration screenshots were inspected. This
uses the real Rust worker and native Palpo adapter with an isolated Matrix HTTP
fixture. It verifies the source-event handoff, not a live Matrix SDK room join.
The Rust suite separately covers eight signup scenarios, including lost replies,
restart, private-room changes, stale/forged verdicts and revocation during UIAA.

## Credential controls and independent profiles

Administrators can pause, resume, revoke and renew an engagement's Matrix or
transport credentials through native forms. Pending changes retain their exact
intent after a lost reply. Owners see the resulting association state and
notices; a resumed or rotated connection requires a new authenticated probe.
Credential revocation does not assert that an offline runtime stopped.

The combined native run passed all 16 checks at
`target/palpo-association-validation/7a2dc86faa514493888fba9425eefc35/report.json`.
The paused and two-engagement screenshots were inspected. The run pins all three
executables before and after acceptance. One native Hagency process imports two
profiles for the same homeserver, retains both proofs after restart, rejects an
old credential generation, and preserves the first profile when adding the second.
An earlier run completed its functional checks but failed the executable-hash
gate during an overlapping build; it is not counted as passing evidence.

This uses actual Rinx, Rust Palpo and Hagency processes with a Matrix HTTP
fixture. Profile retrieval uses the authorized API, so native save-dialog and
real Matrix/agent/device acceptance remain separate gates.

### Scoped Matrix display-name changes (2026-10-05)

Native run `db6e9f0cfe45422c801846e432a3bdda` passed 21 checks. The actual
Rinx form submits a frozen rename command, shows Matrix verification pending,
and displays the confirmed name after an authenticated observation. Chat remains
usable while only a label update is pending. Captures `owner-rename-pending.png`
and `owner-rename-verified.png` were inspected. This run uses the Rust Palpo
process and a provider fixture; Hagency store and HTTPS-client tests separately
cover real rename execution and read-back. It is not live-provider acceptance.

### Approved project setup recovery (2026-10-05)

Run `0a97a9126f214590a91825d332085946` passed 22 checks with binary hashes
held constant through acceptance. The Projects page shows a failed private-room
join, sends a frozen retry through the native form, and enables Request agent
only after the approved project becomes ready. The original decision and grant
are unchanged. Failure and ready screenshots were inspected. The first run
caught missing optional fields for older project projections; the Rust DTO now
supplies explicit null/false defaults and association views do not read a
project-only property. Evidence uses Rust Palpo plus provider/Matrix fixtures,
not a live provider deployment.
