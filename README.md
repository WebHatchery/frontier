# Frontier Kingdom

Frontier Kingdom is a dark expedition card RPG built in Rust with Macroquad. You manage a fragile settlement, recruit adventurers, build facilities, choose branching mission routes, and survive turn-based card combat while stress and injuries persist between expeditions.

## Current Features

- Party formation with soldiers, scouts, healers, and mystics.
- Gendered adventurer portraits and class-specific card pools.
- Candlelit command-table base UI with top-level tabs, facility cards, action bar, and contextual detail panels.
- Title menu with fresh-campaign, save continuation, and platform-safe exit controls.
- Embark preparation screen with party risk, mission briefing, readable locked requirements, and rewards.
- Branching mission maps with combat, event, rest, and boss nodes.
- Expedition route panel with anchored party status and node legend.
- Region-aware enemy spawning from JSON data.
- Turn-based card combat with central enemy intent, battle report preview, energy, block, status effects, and hover tooltips.
- Stress, resolve checks, trauma, heart attacks, injuries, death, and a graveyard.
- Kingdom facilities for healing, stress relief, recruitment, card learning, and the citadel win condition.
- Threat scaling, economy rewards, quest log, random kingdom events, save/load, and deck viewer.

## Content

- 33 cards, including Knowledge-unlockable advanced cards.
- 10 enemies across 6 AI patterns.
- 3 regions with region art and mission unlock requirements.
- Data-driven cards, enemies, missions, regions, and prompt metadata under `assets/`.

## Controls

- `Tab`: cycle base tabs.
- `1-9`: select roster, building, mission, path, or combat card depending on screen.
- `M`: form a party from the selected adventurer.
- `D`: view the selected adventurer's deck.
- `H`: heal at the Infirmary when built.
- `T`: reduce stress at the Chapel/Tavern when built.
- `U`: learn an advanced card at the Foundry when built.
- `R`: recruit from the Guild Hall.
- `Enter`: confirm selection, construct, embark, choose event, or play selected card.
- `Space`: advance missions or confirm paths.
- `A/D` or `Left/Right`: choose between available mission paths.
- `E`: end combat turn.
- `Esc`: close overlays, cancel, retreat, or return.
- `F5` / `F9`: optional save and load shortcuts from the base; visible Save and Load buttons are always available.

Mouse selection is supported for roster rows, facility cards, action buttons, mission cards, event choices, mission path nodes, combat cards, and the end-turn button.

## Build And Run

```powershell
cargo run
```

Useful checks:

```powershell
cargo fmt --check
cargo check
cargo test
```

## Publishing

Use the project wrapper to call the shared RustGames publisher:

```powershell
.\publish.ps1
```

Generated build outputs belong in ignored directories such as `target/` and `dist/`. Runtime logs and temporary generated image batches are also ignored and can be regenerated when needed.

## Art Direction

The fantasy is a candlelit command table: the player manages a doomed frontier
charter from a desk covered in maps, blood, debt, and bad weather — never a
navigable town. Normal text is parchment, grey, and dull brass; bright colour is
reserved for selection (candle gold), readiness (muted moss), danger (blood
red), scouting information (cold steel), and trauma (occult violet). Card
information is rendered by the UI, never baked into artwork.

## Project Layout

- `src/main.rs`, `src/game.rs`: Macroquad entry point and the top-level state switch.
- `src/state/`: one module per screen — `base/`, `mission_select`, `mission`, `event`, `combat/`, `recruit`, `results`. Only one state is active; transitions are explicit.
- `src/combat/`: units, cards, effects, and the resolver. Cards emit effects; the resolver applies them, so nothing mutates a unit directly.
- `src/kingdom/`, `src/missions/`: roster, party, buildings, unlocks, regions, and mission templates.
- `src/ui/`, `src/data/`, `src/save/`: immediate-mode drawing helpers, JSON loaders, and the single versioned `SaveData` struct.
- `assets/`: cards, enemies, missions, regions, prompt metadata, and runtime images. Balance and content live here, not in Rust.
- `gdd.md`: game design notes.
- `TODO.md`: checked completion record for the current TODO pass.
- `generate_assets.ps1`, `comfyui-*.ps1`, `COMFYUI-curl-examples.md`: optional local asset-generation tooling.
