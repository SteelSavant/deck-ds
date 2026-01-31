## UI Refactor

No business logic should exist in the UI layer (preferably at all, in general for pipelines). This means:

- No custom logic for filling dropdowns
- No custom logic for reifying/patching
- No custom render/component logic per-item

Low Priority:

- Dropdowns and other "selection" logic will have a predefined method of querying the server for their values
- Actions should have a way to define/return some sort of UI schema that can be automatically rendered by the client UI

## Displays

### Global

This should probably be integrated into the Desktop Session Handler

- Display Id
- Relative Location
- Resolution (Default)
- Deck is Primary Display - this gets weird; potentially want per-orientation
- Display Selection (primary + rest [ordered])

### Per Config

Per-config needs an easy shortcut to global config to change active display(s)

- Resolution - this gets weird; potentially need resolution per-display
    - map of display id to resolution (global if missing, otherwise override)
- Deck is Primary Display - this gets weird; potentially want per-orientation
    - map of display id primary (global if missing, otherwise override)

## Emulator

Configure emulators (config file location, etc.) separately from profiles, to allow all profiles for an emulator instance to share the same configs
