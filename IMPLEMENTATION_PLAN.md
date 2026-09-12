# Toybox After Hours — Review Follow-through Plan

This plan turns the August 2026 fresh review into committed, verifiable work.
The title screen is the visual benchmark: the implementation should carry its
warm, tactile toy-shop identity into play rather than replacing what already
works.

Each numbered section is a major commit boundary. A section is complete only
when its acceptance criteria pass; incidental cleanup stays with the section
that requires it instead of becoming a broad refactor.

## 1. Prove and tune the complete shift

The existing `Earner` replay reports twenty minutes after shelving the whole
toys but leaves the broken pairs unresolved. The ignored test named
`a_full_shift_completes_the_shop` only prints two partial reports and asserts
nothing. Those numbers cannot establish that a 30-minute shift is comfortable.

- Add a true end-to-end replay that starts without tools, earns and buys them,
  repairs every broken pair, shelves all 240 toys, and reaches
  `GamePhase::Finished` through the public interaction API.
- Keep separate sorting and restoration reports for diagnosis, but stop calling
  either one a complete shift.
- Measure at least sort-then-repair and repair-focused routes, including tool
  purchase timing and credits remaining.
- Tune the deadline only from the complete result. The deterministic closer
  should retain 15–20% headroom because it walks direct lines, never gets lost,
  and spends a synthetic 0.6 seconds per interaction.
- Correct README and TODO balance claims to match what the tests actually prove.

Acceptance: the complete replay asserts `Finished`, 240/240 shelving, all 28
repairs, no loose usable toys, and meaningful deadline headroom. The normal
suite guards the invariant; the ignored report remains available for retuning.

Status: completed on 2026-08-05. The earned-tool route finishes 240/240 with
all 28 repairs, all five tools, no loose usable toys, and 8.6 minutes / 28.7%
headroom inside the unchanged 30-minute deadline. The old twenty-minute claim
was removed because it described only the sortable floor.

## 2. Make every tool and credit tell the truth

The scanner currently names the first matching fixture while highlighting all
four interchangeable matching displays. Manager's Nod waives the timer charge
but still lets an incorrect toy occupy a slot, so it is weak for its late cost.
A restored store also leaves nine credits with no decision attached to them.

- Define the scanner as route guidance: strongly mark the nearest compatible
  display with room, subtly mark the other compatible displays, and name the
  recommended destination consistently in the HUD and shop copy.
- Change Manager's Nod into a visible safety net: a protected wrong placement
  is stopped before the toy leaves the player's hands or blocks a slot. It still
  records the mistake, preserving the score's preference for careful work.
- Add a bounded repeatable late-shift service for surplus credits. It must help
  finish work without replacing the sorting loop, work deterministically, and
  disclose its exact effect in the shop.
- Measure purchase orders and individual tool effects in the replay rather than
  treating “fully equipped” as proof that each choice is equally useful.

Acceptance: scanner text and rendering identify the same recommended display;
protected mistakes leave the toy carried and the slot empty; every earned
credit has a useful outlet; save/load preserves all new economy state.

Status: completed on 2026-08-05. The scanner now selects the nearest valid
display with room through one query used by text and rendering. Manager's Nod
stops twenty-five future wrong placements while retaining the score mistake.
After the five-tool rack is complete, spare credits call a deterministic
60-second Stockroom Spotlight, capped at three minutes and persisted in saves.
The service has dedicated HUD, minimap, world-beacon, shop, and capture states.

## 3. Carry the title screen's craft into the game UI

The current gameplay UI is functionally complete but reads as flat black debug
panels beside a richly illustrated title. Small captions, fitted text, dense
rectangles, and a report-like score screen create the dated feeling identified
in the review.

- Establish a shared warm wood, parchment, brass, and category-colour visual
  language for HUD surfaces, keycaps, meters, buttons, and state accents.
- Replace the tall status slab with a compact clock/progress header and a clear
  current-aisle objective that expands only when it has useful detail.
- Present carried toys as a three-position trolley tray with an unmistakable
  active slot and readable identity, without shrinking names into illegibility.
- Frame the minimap as a store directory with category/zone cues and clearer
  repair/scanner markers.
- Rebuild the tool shop around cards that expose current-to-new values and make
  Locked, Affordable, Owned, and Service states distinct at a glance.
- Give both score outcomes a stronger payoff, visible next actions, and cleaner
  hierarchy than a spreadsheet of equally weighted rows.
- Expand Settings to include mouse sensitivity, audio levels, UI scale, high
  contrast, and a controls/help entry.
- Keep layouts usable at 1280×720, 1366×768, 1920×1080, and 16:10 browser sizes.

Acceptance: every affected capture scene is refreshed and inspected; text does
not overlap or escape its surface; pointer-blocking metadata matches the new
layout; settings persist; title, gameplay, shop, pause, and score screens feel
like one game.

Status: completed on 2026-08-05. Gameplay now uses a compact walnut and
brass shift header, trolley card, matching prompt chrome, and a labelled store
directory. Tool states are distinct cards and both score outcomes use summary
cards plus a grade medallion. Settings and pause now sit on the same constructed
surface as the rest of the UI. Persistent FOV, sensitivity, UI scale, contrast,
and help controls are implemented with the onboarding work below; channel
volume controls are completed with their actual audio paths in Section 5.

## 4. Teach the shift in play

The web page lists controls, but the game itself does not teach movement,
mouse-lock, its sorting rule, or the repair/trolley interactions in sequence.

- Add a dismissible first-shift guidance track that teaches look/move, pickup,
  category shelving, a repair pair, the tool shop, and trolley cycling only
  when each action becomes relevant.
- Keep contextual prompts authoritative: tutorial copy points at the same keys
  and actions the interaction preview will execute.
- Provide a Controls/How to Play panel so guidance remains discoverable after
  dismissal and can be replayed from Settings.
- Persist tutorial completion separately from a resumable shift.

Acceptance: a new profile can learn the loop without README/game-page text;
experienced players can skip it; save migration defaults safely; capture scenes
cover the first guidance step and the controls panel.

Status: completed on 2026-08-05. A six-step contextual guide advances from
navigation through sorting and waits to mention repair, tools, and the trolley
until each is usable. `H` dismisses it, Controls & How to Play keeps the full
loop discoverable, and Replay Guide clears the separately persisted completion
flag. Dedicated tutorial, help, high-contrast, and large-text captures cover the
new states.

## 5. Give the shop an audible response

The game currently has no sound path. A magical shop should acknowledge motion,
pickup, shelving, repair, tools, warnings, and restoration without becoming
noisy during a long shift.

- Check the shared toolkit first and use its audio facilities where suitable.
- Add a restrained ambient bed and short, distinct feedback for pickup/drop,
  correct and wrong shelving, repair, tool purchase, display completion, and
  closing urgency.
- Add separate master/effects/ambience controls with mute support and persisted
  values. Missing or blocked audio must fail softly rather than block the game.
- Keep generated or authored assets local and publishing-safe.

Acceptance: every important action has one intelligible cue, repeated sorting
does not produce fatiguing overlap, zero-volume settings silence their channel,
and Windows/WebGL publishing includes everything required.

Status: completed on 2026-08-05. Ten short procedural cues cover motion,
pickup/drop, shelving outcomes, repair, purchases, display completion, closing,
and restoration; per-cue cooldowns keep repeated sorting restrained. An
eight-second generated ambience bed follows the active shift. Persisted master,
effects, and ambience controls reach true zero, capture/bench runs stay silent,
and decoding failures degrade cue-by-cue without blocking startup.

## 6. Add deterministic variety without invalidating records

The competitive Closing Shift should remain reproducible, while Relaxed Run
should not scatter the identical 240 objects on every new start.

- Introduce an explicit persisted `shift_seed` and isolate seeded scatter behind
  the spawn helpers.
- Keep Closing Shift on the catalog's fixed competitive seed so best runs remain
  comparable.
- Give each new Relaxed Run a different seed, display it on pause/score screens,
  and allow replaying that seed.
- Preserve cross-zone repair pairs, safe spawn positions, display capacity, and
  deterministic save/load for every seed.

Acceptance: equal seeds produce byte-equivalent toy layouts; distinct relaxed
seeds materially change positions; timed records remain fixed-seed comparisons;
captures and replays explicitly set their seeds.

Status: completed on 2026-08-05. Closing Shift explicitly uses the fixed
`C105-1A6F-7EED-0001` seed and retains its pre-seed scatter and balance result.
New Relaxed Runs receive fresh deterministic positions, poses, colours, broken
selection, and cross-zone part scatter. The full seed survives old/current
saves, appears on pause and score screens, and can be replayed with `F5` while
`R` starts a new scatter. Equal-seed byte equivalence, material seed variation,
fixture safety, repair invariants, captures, and fixed-seed replays are tested.

## 7. Final verification and documentation

- Run `cargo test --all-targets` and Clippy with warnings denied.
- Run the full balance reports and the complete 76-image capture drift gate.
- Run `publish.ps1` with no parameters and verify Windows and WebGL output.
- Inspect the refreshed title, first-run, gameplay, tool shop, repair, carried
  trolley, settings/help, closing, and both score captures.
- Update README, game-page controls/details, TODO decisions, and this plan with
  final measured results.

Acceptance: every plan item has direct test, capture, or published-artifact
evidence; source-size standards pass; the worktree is clean after the final
commit; no balance or UI claim relies on a partial replay or stale image.

Status: implementation and local verification completed on 2026-09-12. The
normal suite passes 107 tests with the two diagnostic balance reports
intentionally ignored; all code-standards checks pass and Clippy accepts every
target with warnings denied. The explicit shop-scale and full-shift reports
complete successfully, and the native ten-second benchmark measured 171.3 FPS
with one slow frame after warm-up. `publish.ps1` builds and packages both
Windows and WebGL artifacts, but the managed environment denies the configured
Preview share and loopback browser access, so final mobile captures and remote
deployment remain external follow-up. The working-tree commit is also pending
because this session cannot create `.git/index.lock`.
