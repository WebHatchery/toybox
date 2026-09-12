# Toybox After Hours: Closing Shift

A Rust + Macroquad first-person 3D cleanup game for Web Hatchery Games.

You are the night-shift worker in a magical toy store after closing. Pick up
scattered toys, read their visual clues, and return every plushie, block set,
action figure, and board game to its matching display before opening.

## A Shift

Two ways to start, chosen on the title screen:

- **Closing Shift** — 30 minutes until the doors open. The HUD clock counts
  down, ambering under five minutes and reddening under one. When it runs out
  the shift ends where it stands and the score screen reports the damage. Its
  fixed layout keeps record attempts directly comparable.
- **Relaxed Run** — the same store rules with no deadline and a fresh seeded
  scatter on every new start. The clock still counts, but never ends the shift.
  Pause or finish to see the exact layout code; `F5` replays it.

The deterministic closer completes the whole job in about 21.4 minutes: it
starts empty-handed, earns all five tools, repairs all 28 broken toys, and
shelves all 240. That leaves 8.6 modelled minutes inside the 30-minute deadline;
the margin is deliberately larger than the replay needs because a person has to
recognise toys, search, turn, and backtrack. Shelving the store perfectly ends
the run early with a "Store Restored" score screen.

Each mode keeps its own best run — most toys shelved, then fewest wrong
shelves, then fastest — and the score screen shows it so there is something to
beat. Closing records share one competitive layout; Relaxed records are a
personal history across varied layouts. Tools do not carry between shifts; the
record is the only thread from one run to the next.

A first-shift guide teaches movement, pickup, category shelving, repairs,
display credits, and trolley cycling as each becomes relevant. Tap **SKIP
GUIDE** to hide it; **Controls & How to Play** in Settings can replay it later.

## The Store

- A multi-zone toy store (34×22 m): Plush Corner, Checkout, Dragon Alcove,
  Block Pit, Robot Lab, Backroom, and the Board Game Wall, connected by
  shelving-lined aisles with real player collision
- **240 toys** scattered deterministically from the shift seed — fixed for
  Closing Shift and reshuffled for each new Relaxed Run
- 50 distinct procedural toy designs (10 per category) built entirely from
  primitives: no image assets, everything drawn in code
- 20 themed displays (walls, pegboards, bins, shelves, tables) to shelve toys
  onto, four per category, twelve slots each
- Hanging zone signs and a live minimap keep the store navigable
- A warm wood-and-brass HUD keeps the clock, whole-store progress, current
  aisle, trolley, prompts, and store directory readable without hiding the room
- Two repair benches; roughly one toy in eight starts broken, split into a head
  and a body scattered into *different* zones, to be rejoined at one bench
  before either half can be shelved
- A one-toy carry limit to start with, a wrong-shelf penalty worth about one
  toy's work, and a score screen grading the run
- Tool credits from completed displays, spent on five tools and a bounded
  late-shift search service
- Procedurally generated footsteps, action cues, closing warnings, and a quiet
  eight-second shop ambience loop; no sampled audio assets are required
- Toolkit save/load slot support, including the exact layout seed
- 60+ FPS native via spatial-grid culling and distance LOD (F3 shows the debug
  overlay when enabled)

## Tools

Each restored display earns one credit. Tools unlock as displays are completed
and are bought from the shop screen (`T`); they last the shift, not beyond it.

| Tool | Unlocks at | Cost | Effect |
|---|---|---|---|
| Toy Scanner | 1 display | 1 | Recommends the nearest matching display with room, while still marking the alternatives, and pins the exact spot of a carried part's other half |
| Sorting Trolley | 2 displays | 2 | Carry three toys instead of one |
| Grippy Sneakers | 3 displays | 2 | A third faster across the floor |
| Long-Handled Grabber | 4 displays | 3 | Reach further into a pile |
| Manager's Nod | 5 displays | 3 | Stops the next twenty-five wrong placements before they leave your hands; they still count as mistakes |

After all five tools are owned, each spare credit can call a **Stockroom
Spotlight** for 60 seconds. It marks the nearest loose toy in the room and on
the minimap, stacking up to three minutes; it never moves or sorts the toy for
you.

Without the scanner a carried repair part still names the aisle its other half
landed in — enough to make the errand a search rather than a sweep of the whole
store.

## Controls

The browser HUD always shows touch buttons for looking, walking, ACT, CARRY,
DROP, TOOLS, and PAUSE. Drag the view or tap the arrow buttons to look; tap
the movement buttons to walk. ACT picks up, loads another toy onto the
trolley, shelves, places repair parts on the bench, or puts the active toy on
the floor. No mouse capture or pointer lock is required.

Keyboard and mouse remain optional shortcuts:

- `WASD`: move relative to the first-person view
- Mouse or arrow keys: look around, including up and down
- `E` or `Space`: activate ACT
- `Q`: cycle which carried toy is active
- `G`: quick-drop the active toy
- `T`: open or close the shop tools screen
- `H`: hide the first-shift guide (replay it from Settings)
- `Esc`: pause and open Settings
- `Ctrl+S` / `Ctrl+L`: save / load
- `R`: start a fresh shift in the current mode (a fresh Relaxed scatter)
- `F5`: replay the current layout seed exactly

Settings persist fullscreen, field of view, look sensitivity, UI text size,
high-contrast mode, and separate master/effects/ambience levels separately from
the current cleanup save. Any channel can be lowered to 0% for true mute.

## Validation

```powershell
cargo test                                    # session, interaction and replay suites
cargo clippy --all-targets --all-features -- -D warnings
.\publish.ps1                                 # Windows + WebGL build and deploy
```

Balance numbers come from deterministic replays in
`tests/state_tests/replay.rs`, which drive the real `GameSession` API rather than
a model of it. The normal suite requires the earned-tool route to finish all
240 toys with every repair complete and at least 15% deadline headroom. Two
diagnostic reports are `#[ignore]`d because they are only wanted when retuning:

```powershell
cargo test --release shop_scale -- --ignored --nocapture   # run length vs shop size
cargo test --release full_shift -- --ignored --nocapture   # whole shop, start to finish
```

The integration tests live in `tests/`, with five-case coverage treated as the
default for a major feature. Smaller suites are intentionally narrow for
single schemas, render math, and persistence adapters; the larger state and
replay suites cover the gameplay branches.
