# TODO - Toybox After Hours

- [x] Move tests from `src/**/tests.rs` into `tests/`; add `src/lib.rs` and expose only intentional public APIs.
- [ ] Complete touch access and mobile verification through UI-02, UI-03, UI-07, and UI-V1 below. History: visible controls and touch-first page/tutorial copy were implemented; the 2026-09-12 mobile capture was blocked by managed-browser loopback access. Do not reimplement that completed work; finish the remaining layout, recovery, and verification gaps.
- [x] Replace the tool-rack button with a toolkit-backed widget that supports the shared release semantics and touch input.
- [x] Split near-limit modules and refactor `Game::update` to comply with the documented function-size limit.
- [x] Move remaining tunables and player-facing copy into typed JSON under `assets/`.
- [x] Add load-time validation for IDs, names, references, capacities, unlocks, effects, and positive configuration values.
- [x] Remove unused texture-manifest plumbing and correct the source-gate comment.
- [x] Review feature test suites against the five-case target; consolidate related cases or document justified exceptions in `docs/testing.md`.
- [blocked] Align Project Roost slug guidance across the canonical docs, templates, publisher, and project copies. The project copy now documents the publisher's observed canonical `rust_<project_directory>` behavior, but the external canonical/template files are read-only in this session.
- [blocked] Complete browser boot and WebGL performance verification; resolve the remote-browser or loopback access blocker. Windows/WebGL builds and a native 10-second benchmark pass; Preview deployment and loopback browser access are blocked by the managed environment.
- [x] Coordinate cleanup of stale deployed shared-runtime files in `rust_management`; the audit found no divergent bridge files requiring deletion. See `docs/verification/shared-runtime-audit-2026-09-12.md`.

## UI_STYLE review — 2026-09-20

Audit only: no UI implementation or new captures in this pass. Read the updated
`AGENTS.md`, `UI_STYLE.md`, `CODE_STANDARDS.md`, `GAME_DEVELOPMENT_GUIDE.md`, and
`README.md`; no project GDD or `PROJECT_AGENTS.md` was found. Inspected screen
renderers, screen/action flow, input conversion, camera and targeting logic,
tutorial progression, and the capture harness. The README is the design reference.

Visually inspected existing `docs/verification/` images: `ui_gameplay.png`,
`ui_mid_run.png`, `ui_carrying_a_half_scanned.png`, `ui_large_ui.png`,
`ui_tutorial_first_step.png`, `ui_tool_shop_early.png`, `ui_shift_over.png`,
`ui_title.png`, and `ui_settings.png` (1264×681). These are historical evidence:
gameplay images lack the touch panel now drawn unconditionally, show E rather
than ACT, and the result image lacks the current action buttons. The tutorial
image's pointer-lock instruction has already been replaced in JSON. Do not file
those old instructions or absent buttons as current defects. No current native,
browser, minimum-size, or touch-interaction verification is claimed.

Preserve the existing strengths: a full-screen first-person world, contextual
repair/scanner hints, world placement brackets and wrong-placement markers,
separate shop/results phases, tutorial dismissal/replay and relevance gating,
and explicit tool prices/prerequisites. No generic template demo text or debug
dashboard was found in ordinary play; the composition problems below concern
the game's own HUD. Do not replace it with the template's demonstration panels.

### Verified findings and implementation tasks

Order reflects player impact and dependencies. UI-01 defines the composition;
UI-02 establishes its usable layout; later tasks use that foundation. UI-07 can
be addressed independently because it protects progress. References to sizes
below use a proposed test matrix, not a claim of currently supported minima:
1280×720 normal canvas, 960×540 compact desktop, 844×390 phone landscape, and
390×844 portrait. UI-01 must declare the supported contract and how unsupported
orientations are handled. Measure the actual canvas and physical/CSS targets,
not just browser outer dimensions or logical coordinates.

- [ ] **UI-01 — Recompose normal play around the toy and immediate action.**
  **Scope:** README screen briefs; `src/ui.rs::draw_game_ui`,
  `src/ui/hud.rs::{draw_game_hud,draw_status_panel,draw_carried_card,draw_context_prompt}`,
  `src/ui/hud_chrome.rs::draw_hud_panel`, and `src/ui/minimap.rs`.
  **Evidence/problem:** Historical gameplay/mid-run captures show equally framed
  status, empty-hands, prompt, map, and sometimes notice regions. Current code
  retains those and adds controls/tutorial. Carry count repeats inventory state;
  the map repeats the whole-store count; zone count, percentage, and bar report
  the same progress. The empty-hands card spends 360×102 logical pixels on
  “Hands empty / Ready to sort.” These compete with identifying and shelving toys.
  **Change:** First record UI_STYLE §1 briefs for play, tool purchase, title,
  pause/help, and results, including normal/minimum canvas sizes. Plan one world
  focus, one compact action/inventory support area, and quiet utilities. Remove
  the empty-hands panel and redundant prose, map total, and duplicate progress
  encodings. Put carry capacity with inventory and integrate the target prompt
  with its actionable control. Reduce nested borders, pins, and headings on
  support surfaces; keep the warm store identity without framing every fact.
  Keep the timed deadline, actionable capacity, repair constraints, and readable
  whole-store progress; make detailed aisle progress available through the map.
  **Acceptance:** A player immediately sees the target and next action; normal
  play has at most 2–3 strongly emphasized regions. The world remains dominant,
  and removed panels yield useful visible floor/shelf space rather than scattered
  controls. Every persistent fact has a clear purpose and one home.
  **Verify:** At normal and declared minimum sizes, compare empty hands, full
  trolley, carried half, scanner, first-use guidance, and under-one-minute play.
  Check that a touch player can find capacity, destination, and deadline without
  opening unrelated screens; review composition before decorative changes.

- [ ] **UI-02 — Resolve the map/control collision and reflow real touch targets.**
  **Scope:** `src/ui/{space,touch_controls,minimap,title,score,tutorial}.rs`,
  `src/ui.rs::{logical_pointer,draw_game_ui,draw_tool_shop_screen}`.
  **Evidence/problem (code-verified):** The minimap panel is approximately
  x=1046..1272, y=532..712 for the 34×22 store; the later-drawn opaque touch panel
  covers x=1008..1262, y=492..704. It obscures almost the whole map. The 1280×720
  UI stretches to the framebuffer without breakpoint reflow. A 44-logical-pixel
  control is only about 24 pixels high at an 844×390 canvas; title/settings
  controls start at 38 logical pixels. Text scaling does not enlarge hit boxes.
  **Change:** After UI-01, give movement, action, and navigation/map distinct
  non-overlapping placements. Provide a visible MAP disclosure and a readable
  expandable map if a persistent map cannot fit. Reflow menus/shop/help and
  controls at compact sizes rather than shrinking the whole desktop layout;
  use scrolling for long content. Target at least 44×44 CSS-pixel touch areas
  with spacing as a project acceptance goal. Use toolkit viewport/pointer helpers
  consistently for layout and picking; keep any required 3D-specific conversion
  explicit and documented.
  **Acceptance:** Map navigation remains available while walking; no opaque HUD
  covers required information or controls. Labels and touch areas remain usable
  at the declared minimum, including enlarged text and high contrast. Resize
  preserves agreement between visible controls and hit locations.
  **Verify:** Use the full proposed matrix, actual embedded browser canvas,
  fullscreen, and display scaling. Tap map/open/close, every movement/look
  direction, ACT, CARRY, DROP, TOOLS, and PAUSE. Check edges and gaps after resize;
  test dense minimap markers and return to play without keyboard input.

- [ ] **UI-03 — Separate gameplay choices from utilities and emphasize ACT.**
  **Scope:** `src/ui/touch_controls.rs::draw_touch_controls`,
  `src/ui/title.rs::{draw_title_screen,draw_settings_screen}`,
  `src/ui/score.rs::draw_score_actions`, and associated `UiAction` handling.
  **Evidence/problem:** Current controls put TOOLS and PAUSE in one framed grid
  with movement, ACT, CARRY, and DROP, all using the same treatment. Title mode
  choices/Continue share a row with Settings/Quit (also visible historically).
  Results render NEW SHIFT, REPLAY LAYOUT, and MENU as identical adjacent buttons.
  This mixes decisions with navigation and gives the main interaction no priority.
  **Change:** With UI-02 layout, make ACT a prominent contextual control beside
  the target description; visually separate movement and secondary carry/drop
  actions. Keep tool purchasing with gameplay support and PAUSE in quiet utility
  space, outside that group. Separate title start/resume choices from Settings/
  Quit, and results replay/new-run choices from Menu. Remove “TOUCH CONTROLS” as
  a permanent heading; use understandable directional labels or symbols with
  brief onboarding. Hide or explain unavailable CARRY/DROP actions based on load.
  **Acceptance:** The next useful action is clear without comparing equal buttons;
  utilities remain visible but never share a visual group with gameplay choices.
  Equally valid mode/replay choices need no invented preference.
  **Verify:** At normal/minimum sizes, touch through first launch, both modes,
  Continue, pickup/shelve/repair, trolley cycling, pause/resume, and both endings.
  Confirm action labels match the operation and adjacent taps cannot accidentally
  select a utility or destructive restart.

- [ ] **UI-04 — Make tool announcements temporary and keep current guidance contextual.**
  **Scope:** `src/ui/hud.rs::{draw_notice_panel,prompt_for_interaction,draw_carry_pips}`,
  `src/game/actions.rs`, `src/game.rs` notifications, and `src/ui/tutorial.rs`.
  **Evidence/problem:** Upgrade availability/shortage and the Spotlight sales
  invitation are reconstructed every frame as persistent notices; scanner state
  shares that notice area. “Use the view controls to look” persists when nothing
  is nearby. G/Q/T keycaps remain separate from visible DROP/CARRY/TOOLS controls.
  Historical mid-run captures confirm the permanent upgrade strip. Current
  tutorial body uses the full card width while SKIP GUIDE occupies its lower-right
  body area; this is a geometry risk, not a visually verified current overlap.
  **Change:** Announce a newly available tool briefly, then retain a quiet
  availability cue on TOOLS; show exact affordability in the shop. Keep the
  carried half's zone/distance and active Spotlight duration with their relevant
  state, not a permanent sales warning. Remove idle look prose after onboarding.
  Replace duplicate keycap explanations with visible control names and optional
  shortcuts. Reserve separate space for tutorial text and Skip; retain current
  completion/relevance gating and Settings → Help → Replay Guide.
  **Acceptance:** Calm play has no permanent event/sales/tutorial wallpaper.
  Repair destinations and service expiry remain understandable after toasts fade;
  urgent deadline feedback suppresses unrelated announcements. Tutorial text and
  dismissal are both readable and independently tappable.
  **Verify:** At normal/minimum sizes and maximum text size, earn a credit, unlock
  and buy a tool, carry a half with/without scanner, activate/expire Spotlight,
  and trigger closing warnings. Wait for toasts to expire; verify state remains
  clear. Complete, skip, and replay guidance using touch alone.

- [ ] **UI-05 — Protect decision-critical text from truncation.**
  **Scope:** `src/ui/hud.rs::{draw_zone_stat,draw_carried_card,draw_context_prompt}`,
  `src/ui/widgets.rs`, `src/ui/title/help.rs::draw_help_column`, and tutorial layout.
  **Evidence/problem:** `ui_large_ui.png` visibly truncates the aisle repair
  shortfall and target name. Fixed-width HUD text still uses truncation. The
  help renderer uses fixed columns and unwrapped text; `draw_wrapped_text` takes
  only `max_lines`, so additional lines can be omitted without making their
  remaining content available. Current help/tutorial overflow needs fresh capture.
  **Change:** After composition/reflow, reserve space for missing-part kind,
  repair count, category, and meaningful toy identity before optional flavor text.
  Wrap action/constraint text at readable size. Provide tap-accessible full detail
  for shortened names; reflow or scroll help/tool descriptions when necessary
  instead of silently dropping lines. Do not reduce all text to fit old panels.
  **Acceptance:** Players can identify the correct item/half and understand costs,
  shortages, and instructions at the largest text setting without guessing an
  ellipsis. Optional detail is accessible without hover.
  **Verify:** At normal/minimum sizes, use longest authored names, full trolley,
  repair-ready/mismatch/full-bench states, longest help lines, and all tool rows.
  Check large counts/time strings and 100%/maximum text settings for overlap,
  clipping, and reachable full-detail disclosure.

- [ ] **UI-06 — Focus the tool rack on available purchases and disclose the rest.**
  **Scope:** `src/ui.rs::{draw_tool_shop_screen,draw_tool_row,tool_status,draw_stockroom_service}`.
  **Evidence/problem:** All five tools receive 92-pixel rows with full descriptions
  even early in a shift; the historical early-shop image is mostly owned/locked
  content. Every row adds a border, stripe, numbered badge, status, and button;
  Owned/Locked repeats. The late service is already correctly gated to all tools
  owned and should remain so. At zero credits its disabled Call still says
  “Ready - Costs 1 credit,” without a nearby explicit shortage explanation.
  **Change:** Prioritize available tools and their effects/prices; compact owned
  tools and put future tools behind a visible “Upcoming tools” disclosure with
  unlock requirements. Remove redundant status/badge decoration. Keep balance,
  price, duration/charge constraints, and refusal reasons beside the relevant
  purchase. Explain insufficient service credit and retain its three-minute cap.
  Keep Back separate and reachable; scroll expanded lists at compact sizes.
  **Acceptance:** A player can tell what they can buy now and why another action
  is disabled, while still discovering future upgrades. Inspection is possible
  before spending, without all future systems dominating the first purchase.
  **Verify:** Normal/minimum and maximum text size: zero credits, first scanner,
  insufficient trolley credit, multiple purchasable tools, all owned, Spotlight
  empty/active/capped. Tap disclosure, inspect, buy, and return without hover.

- [ ] **UI-07 — Keep save failures recoverable and expose in-shift touch recovery.**
  **Scope:** `src/game.rs::{save_game,load_game}`, `src/game/actions.rs::apply_action`,
  `src/game/update.rs::queue_gameplay_shortcuts`, `src/ui/title.rs::draw_settings_screen`,
  `src/ui/title/help.rs`, and `game_page.json`.
  **Evidence/problem (code-verified):** BackToTitle calls the void-returning
  `save_game` then leaves regardless of success. Save/load failures live only in
  temporary notifications. In-shift R/F5 and Ctrl+L have no equivalent direct
  pause controls; replay buttons only appear after a shift ends. A touch player
  cannot replay a stuck current layout without finishing, and a failed Save &
  Quit can leave only a stale Continue save after its toast expires.
  **Change:** Return an explicit save result and keep the live session reachable
  on failure. Show persistent contextual failure with Retry, Resume, and an
  explicit leave-without-saving choice. Add clearly separated pause recovery
  controls for restarting/replaying the current layout and loading the saved
  shift; explain replacement of current progress and confirm destructive actions.
  Match help/page copy to actual visible paths and keep these utilities out of
  normal gameplay groups.
  **Acceptance:** No failed save is presented as a successful departure, and
  progress/recovery options remain understandable after notifications disappear.
  Touch users can recover or replay during a shift without a keyboard.
  **Verify:** Normal/minimum sizes: inject save failure, missing/corrupt save, and
  stale existing save; wait out toasts and use Retry/Resume/Cancel. Touch through
  restart/replay/load confirmations in both modes and verify mode/seed semantics.
  Add focused integration coverage for failure transitions and preservation of
  the live session, following the five-case feature target.

### Further inspection and verification (not confirmed visual defects)

- [ ] **UI-V1 — Refresh current-build evidence and finish the existing browser/touch audit.**
  **Scope:** `scripts/capture_scene.ps1`, `src/game.rs::begin_capture_scene`,
  `src/capture_scenes.rs`, `docs/verification/`, and the existing browser/performance
  backlog item above. This consolidates its UI portion rather than duplicating it.
  **Gap:** Existing captures cannot validate current touch controls or result
  buttons. The 2026-09-12 access blocker is historical, not retested this pass.
  `update_player_input` consumes mouse delta without a UI-region/drag guard;
  whether touch-to-mouse synthesis rotates the view while pressing a button
  requires live verification rather than an assumed defect.
  **Action:** Capture a current baseline before implementation and repeat affected
  scenes after each change. Use normal/minimum declared canvas sizes plus the
  proposed matrix; record browser, canvas size, text scale, input method, and
  build revision. Exercise movement/look simultaneously, release outside buttons,
  dragging over HUD, and touching controls without unintended view movement.
  If reproduced, route view drag and button gestures exclusively using toolkit
  input helpers and preserve simultaneous movement/look. Inspect title, first
  tutorial, normal/dense play, carrying/scanner, repair bench, shop/service, pause,
  help, both endings, urgent clock, and failure recovery.
  **Acceptance:** Current captures and touch paths demonstrate the UI_STYLE §9
  checklist at both supported size boundaries; any blocked platform is named
  explicitly. Replace equivalent captures directly in `docs/verification/`.
  Run `./publish.ps1` without parameters after meaningful implementation changes
  and report its result; build success alone does not close visual/touch tasks.

- [ ] **UI-V2 — Verify camera framing and target readability before changing zoom.**
  **Scope:** `src/ui/scene3d.rs::{draw_shop_scene,draw_player_presence,camera_basis}`,
  `src/state/interactions/targeting.rs`, `src/preferences.rs`, and camera config.
  **Evidence/gap:** Historical captures show a dominant world, readable nearby
  toys, substantial ceiling in the starting view, and a carried toy offset near
  the center. Current perspective uses actual framebuffer aspect and adjustable
  60–110° FOV, while UI stretches independently. This establishes no proven
  default-zoom or targeting defect on phones.
  **Action:** After UI-01/02, walk from spawn to floor toys, high shelves, both
  benches, and scanner destinations at default and endpoint FOVs. Compare normal,
  minimum, and portrait/landscape handling; test resizing and display scaling.
  If target size, held-toy occlusion, or wasted framing prevents identification,
  adjust default pitch/FOV or held-object offset to the observed case, preserving
  matching render/targeting geometry. Do not shrink the world to accommodate HUD.
  **Acceptance/verify:** Touch users can recognize whole toys and halves, aim at
  floor/high targets and read placement feedback without HUD or held-object
  obstruction. Capture representative views and record successful pickup/shelve/
  repair paths; close as verified-no-change if camera changes are unnecessary.
