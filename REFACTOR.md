# Backend/UI Refactor

No business logic should exist in the UI layer (preferably at all, in general for pipelines). This means:

-   No custom logic for filling dropdowns
-   No custom logic for reifying/patching
-   No custom render/component logic per-item

Low Priority:

-   Dropdowns and other "selection" logic will have a predefined method of querying the server for their values
-   Actions should have a way to define/return some sort of UI schema that can be automatically rendered by the client UI
