# TODO — Frontier Kingdom

## Shell and navigation

- Add a title/menu state before the base screen.
  - Show Continue only when a save exists and load that save.
  - Start New Game with a fresh kingdom and starter roster.
  - Add a visible Exit control with the correct browser/native behavior.
- Replace the base screen's `Shortcuts:` footer with a compact optional-controls hint.
- Add visible Save and Load controls to the base screen so persistence does not require F5/F9.
- Remove the duplicate Roster and Facilities buttons from the base action bar.
  - Keep the top tab bar as the navigation control.
  - Reflow the remaining contextual actions and update their hit rectangles.

## Screens

- Finish the mission-selection redesign before `src/state/mission_select.rs` reaches the 800-line limit.
  - Extract input and transition handling from the screen module.
  - Extract mission-card, briefing, and layout helpers into focused modules.
  - Verify the 1280×720 layout has no clipping or overlap and refresh its verification screenshot.
- Make combat resolution feedback explicit for damage, blocked damage, and status changes.
  - Record the resolved deltas instead of only showing selection and turn-start messages.
  - Render readable feedback for both player and enemy effects, including multi-party combat.

## Testing

- Add gameplay test coverage in separate test modules.
  - Test mission selection and launch for locked and unlocked missions.
  - Test route-node resolution, successful completion, rewards, and failure results.
- Extract recruit eligibility and cost calculation into a pure evaluator and fixture low-resource and roster-capacity cases.
- Extract event outcome application into a deterministic evaluator and fixture each event choice, including combat-triggering choices.
- Extract combat reward and consequence calculation into a pure evaluator and fixture damage, stress, injury, death, and party cases.
- Add campaign fixtures for base upgrades, mission chains and unlocks, kingdom events, and result-screen progression.
- Refresh the affected `docs/verification/` screenshots so they match the current tap-first controls.
