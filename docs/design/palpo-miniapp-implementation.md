# ADR 0010 implementation checkpoint

The user-facing main build remains separate from this development branch. This
implementation builds on the shared Rinx theme branch and the latest reviewed
OctoScript App Design Flow (0e59346e). The bundle was created with `octo new`;
the new host services were implemented in their owning repositories before
the bundle used them. The shared contract starts from App Hub main 2bcb898.

## Full implementation ledger — 2026-10-04

ADR 0010 remains in progress. The role correction is implemented locally; the
running server was restored to its original image after an unsuccessful rollout.
No production activation or client restart is claimed by the local checks below.
Historical contribution tests later in this document describe the earlier model;
they are not acceptance evidence for the corrected Hagency-originated flow.

| Requirement | Current evidence | Remaining acceptance |
| --- | --- | --- |
| One designated Palpo administrator approves projects | Palpo role checks, strict-create migration, distinct native manager/admin controls | Activate strict mode and validate on the live deployment |
| Hagency owns resource contributions | Rinx forms removed; native registration/contribution calls refused | Hagency contribution UI, authenticated association and bounded delegation publication |
| Capacity is reserved before a project becomes allocated | Hagency `86099fc`: tested SQLite provider/project/agent reservation accounting | Closed Palpo command/receipt transport, explicit project budget and assigned-admin form |
| Assigned project admins decide agents and top-ups | Hagency `86099fc`: grant scope, explicit self-approval, atomic debit and durable replay checks | Current-role Palpo decisions, Rinx review UI and full receipt/retry flow |
| Revocation/expiry fence provisioning and execution | Hagency `86099fc`: retirement, claim/completion and runtime checks; revocation tests pass | Full runtime/Matrix cleanup and lost-response/restart validation |
| Named agent appears and works in its project room | Existing provisioning and earlier live agent setup | Corrected grant-based end-to-end approval through actual room readiness |
| Usage and lifecycle management | Existing Hagency usage and quota primitives | Scoped fresh/stale/unknown summaries, same-agent top-up and removal UI |
| Durable actions and notifications | Inbox, outbox, private Matrix notices, stale-card/version checks | Cancel/resubmit/expiry parity, Glance board, reminders/preferences/quiet hours and OS entry |
| Identity, permissions and secrets | Native account binding, exact bundle consent, owner-only save boundary | Full logout/account-switch/bundle-update/revocation tests and native save/import acceptance |
| Signup decisions | Requests listed | Approve/reject navigation and role checks |
| Shared theme and desktop windows | Native theme-preservation and separate-window tests | Complete new forms with current Rinx/OctoScript controls; hosted OctoSense acceptance |
| Android and OpenHarmony | No device acceptance claimed | Actual build, touch/back/keyboard, background notification and file-picker tests |
| Shared contract and App Hub release | Companion contract implementation and local provenance | Reviewed contract release, remove vendor patch, authentic publisher/platform evidence and publication |

No catalog item, connection proof, queued command or admin click is sufficient to
claim reserved capacity, running agent readiness, or complete cleanup. Each UI
state must follow its owning service's durable receipt. Existing projects and
agents retain their original owners; migration into grants must be explicit.

## Implemented

- A built-in Palpo Splash app in `apps/palpo`: member/admin navigation, persistent
  drafts, Inbox views and pagination, project requests and designated-admin decisions,
  owner activation, named-agent requests, fleet and Matrix-identity operations.
- Shared Rinx semantic colors, fonts, controls and live theme reapply. Theme
  changes preserve the script heap, forms, focus, selection, undo and request IDs.
- Exact `palpo.*` service grants in App Contract 1.2, consent bound to the active
  account and exact build-owned bundle digest, and revocable instance leases.
- A native adapter bound to the authenticated homeserver origin. Matrix/app
  credentials stay outside Splash. The owner-only configuration result goes
  directly to a native save dialog; the script receives only saved/cancelled.
- Palpo's passwordless app-session adapter, durable SQLite approval records,
  resource grants, audit/outbox, private My Actions notification rooms and reminders.
  Same-origin action links in Rinx open the latest server-authorized action.

Palpo shares the browser's existing backend and serial mutation queue. It checks
current roles and ownership on the server. Strict project approval across all
frontends is an explicit deployment migration, `PALPO_PROJECT_APPROVAL_REQUIRED=1`;
existing projects are grandfathered. See Palpo's `web-admin/MINIAPP.md`.

## Role correction validated on 2026-10-03

Project creation approval now belongs to one configured Palpo administrator.
The mini app presents Project approvals to that account and My Inbox / My projects /
My agents / Request a project to managers. Rinx no longer submits contributions
or registers fleets; older native calls are rejected by Palpo. Historical
contribution records and connected resources are preserved. An older Palpo server
without `canApproveProjects` does not implicitly grant approval UI to a Matrix admin.

Palpo `f895acdd20` adds the role check, including queued decisions and notification
recipients. Deployment requires `PALPO_PROJECT_APPROVAL_REQUIRED=1` so existing
browser/API routes cannot bypass approval for new projects. Existing projects
remain owned by their original requester; this does not transfer littlewhite.

Validation: 85 backend tests (14 mini-app workflow/role cases), 323 Rinx library
tests passed (2 ignored), full native build, and Makepad hidden-window run
`e1e1bc6c37c64f199a81d0917417eb89`. The native run covers distinct manager/admin
controls, project approval and owner activation, named-agent submission, draft
recovery, theme/focus/selection/undo preservation and disconnect. Real captures
were inspected at 430px manager width and desktop admin width. Connection
regression `85f0e3372cbd4630a79851e66f1c7afa` passes pending-proof polling,
navigation cancellation and refusal handling with admin connection maintenance.

These are native/macOS checks, not Android or OpenHarmony device acceptance.
Hagency-originated automatic pairing and assigned-project-admin agent admission
remain the backend gaps recorded in ADR 0010.

## Hagency reservation foundation — 2026-10-04

Companion Hagency commit `86099fc` adds finite contribution/project grants,
explicit project-admin policy and revision changes, atomic agent/top-up debits,
financial replay receipts and grant retirement. It counts provider reservations
once across projects and agents, including resources sharing a provider seat.
Expired/revoked grants fence queued or late provisioning and runtime capability
use. Legacy console approval cannot bypass a grant. Held resources cannot move
to another account implicitly, and lower ceilings still constrain new decisions.

Evidence: 595 store regressions passed (50 ignored), followed by 46 focused tests
on the reviewed changes, including 15 grant scenarios. The native Hagency build
check passes. The expiry test crosses an actual SQLite lock; financial replay is
checked after restart and removal of the rolling display-history receipt.

This is the accounting foundation. The operator contribution UI, authenticated
Palpo command/receipt consumer, role synchronization, Rinx approval/top-up screens
and full live lifecycle still require integration. No workflow support capability
is advertised by this commit, and no production update or Makepad/mobile result
is claimed for these backend tests.

## Validation

The native fixture uses the production Splash file and Rinx HTTP adapter with
real Palpo HTTP handlers and SQLite, plus explicit fake Matrix/Hagency services.
It drives a 430 × 820 owner window and a desktop admin window with Makepad
remote input and real captures. It does not use the signed-in user's profile.

```sh
cargo build --profile fast --locked --example palpo_miniapp
python3 tools/wechat-ux/live/native_palpo.py \
  --palpo /path/to/palpo-rinx-miniapp \
  --node /path/to/node24 \
  --binary target/fast/examples/palpo_miniapp
```

Reports, input traces, widget trees and screenshots are written under
`target/palpo-validation/<run>/`. The native scenario covers manager/admin UI separation, project approval and owner activation,
named-agent submission, draft recovery after process restart, light/dark/custom
themes, selection/undo, and disconnect without Matrix logout. Screenshots were
inspected. A Makepad hidden-window frame-confirmation failure is handled by
separating input from a read-only screenshot barrier, never replaying a click.

Additional checks: Palpo backend regression suite, Rinx origin/action-link unit
tests, catalog permission/admission tests, shared contract/policy/hub tests,
and the full Rinx release build. The validation report is the evidence for a
specific native run; this document is not a mobile or live-server acceptance claim.

## Remaining ADR gates

This is **not the complete ADR**. Existing-operation parity still needs signup
decision navigation, real native configuration saving/import and full account-
switch/revocation integration. Current signup UI lists requests only. Live
administrator/owner contribution approval and Matrix-room notices now pass;
the Hagency connection/lifecycle and OS push-notification entry remain untested.
My Actions currently uses private Matrix notices and links; the Glance-style
board, pinning, quiet hours and direct OS-notification mini-app routing remain.

The subsequent Hagency extension is still required for delegated operator
decisions, top-ups, runtime stats and complete runtime-plus-Matrix revocation.
The UI does not report these as implemented. Android, OpenHarmony and hosted
OctoSense require their own build/device acceptance.

App Hub publication is not attempted. The template listing with invented
publisher/platform data was removed. See `apps/palpo/PUBLISHING.md`. The stock
card-host lacks Rinx controls and Palpo services, so validation uses the actual
Rinx host fixture. Store admission awaits publisher metadata, device evidence
and release of the shared contract; there is no claimed store gate pass.

The temporary `vendor/octosense-app-contract` patch contains the companion
App Hub additive extension and provenance. Replace it with the published 1.2
crate after cross-repository review; no private App Hub git dependency is added.

## Checkpoint evidence (2026-10-03)

- Companion Palpo commit: `291b438` (`feat/rinx-miniapp`), 79 backend tests passed.
- Companion App Hub commit: `9854614` (`feat/palpo-miniapp-contract`), 162
  contract/policy/hub tests including doctests passed.
- Rinx catalog: 17 passed, one existing ignored test; origin/action-link tests:
  two passed. `cargo build --release --locked` completed successfully.
- Native run `d23fe8f8a25147b6970b103fb626ff4a`: all five scenario checks passed;
  binary SHA-256 `496df362b08aafdd82624bf3bf9aeddfd922dba283ecce6445a506c7af790c59`.
- Design Flow stamp: `25e82095e7e9a68b300e1553cda7b53702cdfd531e6fee73cde6b417aab3da0a`.
  `octo check --allow-unsigned` reports the expected missing store listing as
  its sole refusal, plus an unsigned-publisher warning. This is not a gate pass.

## Live member validation on mini1 (2026-10-03)

SSH succeeds with the configured key using macOS `UseKeychain=yes`. The public
Matrix endpoint is `https://crew.ominix.io:19443`; the existing admin web service
is `https://crew.ominix.io:19444`. The admin service and PostgreSQL are Docker
containers. The Docker homeserver service name currently resolves to a socat
forwarder targeting the native Palpo process at host port 18010. Both public
Matrix traffic and the admin service reach that active Palpo instance.

`tools/wechat-ux/live/live_palpo.py` ran the actual production Splash app through
Makepad with the current member's real saved Matrix session. It uploaded an
isolated validation backend to mini1 and reached it over SSH port forwarding.
Matrix identity and role checks went to the real running Palpo. Workflow records
went into a new private SQLite database, not the production admin database;
notification workers stayed off. Existing public services were not reconfigured.

Live run `edd7f85cdf9f4917846d20793c4dc7c0` passed:

- Real Matrix whoami and the existing admin web service authenticated the same
  session; a deliberately nonexistent fleet returned 404 for the member and
  401 for an invalid bearer.
- The new app-session exchange succeeded. A member's administrator operation
  returned 403, and administrator decision buttons were absent.
- Native input submitted a contribution under the real member identity into
  the isolated workflow store. Live dark-theme reapply preserved heap and calls.
- Mini-app disconnect left the original Matrix session valid.

Screenshots were inspected, and the native log contains no script/render errors.
The sidecar was stopped after the test. The same updated native binary also
passed the original fixture administrator/owner scenario in run
`f324de6560f44d3a933a4edcd71a7f98`.

This member run did not cover privileged approvals. The saved server-side
administrator/bot credentials found during deployment inspection belonged to an
older test server and returned M_UNKNOWN_TOKEN. The subsequent operator test
below resolves the validation credential problem without changing existing users.

## Live administrator/owner validation on mini1 (2026-10-03)

`tools/wechat-ux/live/live_palpo_admin.py` provisions temporary admin, owner and
notification-bot identities through Palpo's supported no-server operator CLI.
It overrides auto-join rooms for that one-shot process and does not restart the
running homeserver or reset existing users. Credentials stay in private files
and native memory. The production workflow database and public routing stay
unchanged; the sidecar uses an isolated SQLite store and SSH port forwarding.

```sh
python3 tools/wechat-ux/live/live_palpo_admin.py \
  --palpo /path/to/palpo-rinx-miniapp \
  --binary /path/to/target/fast/examples/palpo_miniapp
```

The live native scenario verifies:

- Real admin/member role checks and refusal of owner self-approval.
- Native contribution submission, administrator approval, actual Matrix App
  Service registration and the owner's configuration handoff screen.
- Export authorization: only the owner receives configuration; even an unrelated
  server administrator gets 404. No credentials enter notification cards.
- Private My Actions rooms receive minimal Matrix notices. Marking a notice
  seen leaves the action pending and a later reminder arrives. Only the test
  runner uses accelerated reminder intervals.
- A stale decision returns 409, while an old action reference reads the latest
  approved state. Native rejection returns to owner history.
- Disconnecting the mini app leaves its underlying Matrix login valid.

Run `admin-144a5d328f504fce9046289f033f9f7f` passed all five report checks and
cleanup with the corrected error handling. The native binary SHA-256 was
`965245bde972eaa0e8b89694dce1c24b9ff00424ed20964bdf3a139e09c4a124`.
Admin review, owner handoff and rejection-history screenshots were inspected.
The native logs contain no script/render errors. Cleanup removed the test App
Service, left/forgot the fixture rooms, deactivated and locked test users, removed
their devices and confirmed all test tokens returned 401. Historical room events
remain on Matrix; cleanup does not claim to erase that history. Both public
Matrix and admin-web health checks returned 200 afterwards.

The test exposed a misleading native error: a valid adapter's object-level 404
was reported as a missing adapter. Rinx now distinguishes typed operation errors
from missing routes while keeping upstream message/header text out of diagnostics.
All four Rinx Palpo adapter tests and all eight Palpo mini-app backend tests pass.
The companion live-server runner and documentation are in Palpo commit `9d640bb`.

Public mini-app routing is still undeployed. Native save-dialog completion,
Hagency import/connection, live project/agent lifecycle and OS push entry are
separate outstanding gates; these results do not claim full ADR completion.


## Desktop window and verification follow-up (2026-10-03)

Desktop standalone Rinx now hosts MiniAppsPanel in a lazily created native
window. OctoSense uses its injected window host; Android, iOS and OpenHarmony
retain the in-app modal. Closing the native window revokes its instance, and
closing the main window or clearing the account closes the mini-app window.
Theme reapply preserves the panel and the main conversation's display context.

Palpo's contribution detail and fleet cards read the server's connection proof
and show **Connection verified** only for a ready fleet with verified event
delivery and a verification timestamp. Verify performs one mutation and polls
only fleet reads, at two-second intervals for up to 30 reads. Navigation cancels
the watch; a failed or pending proof is never reported as complete. Project
cards distinguish waiting for approval, approved/ready to create, and created.

Validation evidence (local `target/` artifacts are intentionally untracked):

- `miniapp-window-validation/5fe7a341ebfb4ff1b11f93f0f6beabd7`: real hidden
  Makepad windows; lazy open, text input, dark-theme reapply, main draft/display
  preservation, OS close, reopen, and panel Back close passed. Captures inspected.
- `palpo-connection-validation/7fb4401c0447441298015591eddf6a06`: production
  Splash/host with explicit HTTP fixtures; delayed proof, read-only polling,
  navigation cancellation, persistent verified label and server refusal passed.
- `palpo-validation/8fbb736c16eb4448a04da9811a2f8378`: existing native
  contribution/project/named-agent submission, draft restart, theme and logout
  isolation scenarios passed against the local Palpo fixture.
- Library tests: 323 passed, two existing ignored. Account cleanup tests passed
  again after wiring main-window close. No-default-features OctoSense module
  compilation passed. Android/OpenHarmony device validation remains outstanding.
- Rebuilt full Rinx and restarted the isolated owner/admin profiles against
  `crew.ominix.io`. Each has its main chat and a separate Palpo window. The actual
  user's contribution displays Connection verified; both mini-app sessions have
  no Splash errors. No additional resource/agent approvals were submitted by
  this window/status validation.

The live `octosense-dev` project exists and its owner is joined. The diagnostic
at this checkpoint found no agent request in Palpo and no Hagency engagement;
project approval alone had not provisioned an agent. The product owner's new
requirement is recorded in the ADR's project-administrator amendment: assigned
project administrators decide agents within accepted budgeted grants, and
Hagency applies those decisions automatically. That grant/role/protocol work
is **not implemented by this UI fix**.
