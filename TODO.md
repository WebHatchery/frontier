# TODO — Frontier Kingdom

All tracked TODO items are complete. The title flow, tap-first persistence controls, mission redesign, combat feedback, deterministic evaluators, campaign fixtures, and verification captures are now part of the checked-in game.

## Shell and navigation

- [x] Add a title/menu state before the base screen.
  - [x] Show Continue only when a save exists and load that save.
  - [x] Start New Game with a fresh kingdom and starter roster.
  - [x] Add a visible Exit control with the correct browser/native behavior.
- [x] Replace the base screen's `Shortcuts:` footer with a compact optional-controls hint.
- [x] Add visible Save and Load controls to the base screen so persistence does not require F5/F9.
- [x] Remove the duplicate Roster and Facilities buttons from the base action bar.
  - [x] Keep the top tab bar as the navigation control.
  - [x] Reflow the remaining contextual actions and update their hit rectangles.

## Screens

- [x] Finish the mission-selection redesign before `src/state/mission_select.rs` reaches the 800-line limit.
  - [x] Extract input and transition handling from the screen module.
  - [x] Extract mission-card, briefing, and layout helpers into focused modules.
  - [x] Verify the 1280×720 layout has no clipping or overlap and refresh its verification screenshot.
- [x] Make combat resolution feedback explicit for damage, blocked damage, and status changes.
  - [x] Record the resolved deltas instead of only showing selection and turn-start messages.
  - [x] Render readable feedback for both player and enemy effects, including multi-party combat.

## Testing

- [x] Add gameplay test coverage in separate test modules.
  - [x] Test mission selection and launch for locked and unlocked missions.
  - [x] Test route-node resolution, successful completion, rewards, and failure results.
- [x] Extract recruit eligibility and cost calculation into a pure evaluator and fixture low-resource and roster-capacity cases.
- [x] Extract event outcome application into a deterministic evaluator and fixture each event choice, including combat-triggering choices.
- [x] Extract combat reward and consequence calculation into a pure evaluator and fixture damage, stress, injury, death, and party cases.
- [x] Add campaign fixtures for base upgrades, mission chains and unlocks, kingdom events, and result-screen progression.
- [x] Refresh the affected `docs/verification/` screenshots so they match the current tap-first controls.
