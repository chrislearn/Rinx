# Pending shared contract release

Source from companion App Hub commit `9854614` (merged in App Hub PR #72),
based on App Hub 2bcb898, with the two Palpo consent descriptions from
`94ba0f4a7df18174f67da980ba2686223611a894`. Adds the ADR 0010 exact service grants and their
consent language as version 1.2.0. No admission or integrity rule is relaxed.
Remove the root, standalone miniapp-catalog, system-apps and miniapp-package
Cargo patches when the same shared contract is published. All four consumers
require at least version 1.2 so build, admission and packaging agree.
