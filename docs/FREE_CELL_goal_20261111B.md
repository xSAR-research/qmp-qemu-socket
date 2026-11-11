# Free Cell visible five-packet diagnostic — 3.1.11

Status: completed and host-qualified. xSAR is 0.14.3. Operator Windows acceptance remains separate. All 51 fresh paired host gates passed: 1,882 application, 353 library and 80 console tests, zero failed/ignored/filtered.

Charlie approved application 3.1.11 and the necessary xSAR patch on 11 November 2026. The previous fast trial sent the correct five input requests but retained frames showed the Ace still at source. Host timestamps measured about 0.22 ms from DOWN flush to UP flush; Windows pickup and held-button state were unmeasured. Timing is the next diagnostic variable, not an established cause.

Keep the five ABS SINGLE STEP input requests and insert one second between each consecutive pair: source, DOWN, destination, identical destination, UP. Preserve source acknowledgement, fresh routing, authority checks, release-only recovery and semantic effect verification. Leave normal route drag, REL probe, recognition, cached UNDO ALL retry, recorded AUTO sequencing, public signatures, dependencies and schemas unchanged.

Accept host qualification only after exact-wire/timing/STOP regressions, the required paired locked gates and actual release-artifact binding. Preserve the original failed trial, previous sealed capsules, unrelated edits and all four Git histories/indices. No agent guest input, application relaunch, install or promotion is authorised by this build.

Operator acceptance is separate: inspect pickup, the first destination movement, its repeated packet and release on a supported restored original board. Determine whether the duplicate destination changes visible behaviour without claiming its necessity from this one configuration.

See [contract](FreeCell-Pacing-3.1.11.md) and [verification](Verification-3.1.11.md).
