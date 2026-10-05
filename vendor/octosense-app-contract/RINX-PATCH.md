# Pending shared contract release

Source from companion App Hub commit `9854614` (merged in App Hub PR #72),
based on App Hub 2bcb898, with the Palpo consent descriptions, explicit signup
navigation and failed-allocation recovery grants from
`3862c366e1af147707afed80ba2f9d1083e27e18` (App Hub PR #82).
Adds the ADR 0010 exact service grants and their consent language as version 1.2.0.
Signup navigation requires its own `palpo-account-navigation-v1` host feature;
older grants are not widened. No admission or integrity rule is relaxed.
Failed-allocation recovery requires the exact `palpo.inbox.recover` grant;
an existing project-decision grant does not acquire recovery authority.
Remove the root, standalone miniapp-catalog, system-apps and miniapp-package
Cargo patches when the same shared contract is published. All four consumers
require at least version 1.2 so build, admission and packaging agree.
