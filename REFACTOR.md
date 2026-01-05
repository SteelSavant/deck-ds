# Backend/UI Refactor

No business logic should exist in the UI layer (preferably at all, in general for pipelines). This means:

-   No custom logic for filling dropdowns
-   No custom logic for reifying/patching
-   No custom render/component logic per-item

High Priority:

-   No custom patching will be done at any point in the UI layer
-   Remove logic and weirdness around `patchPipeline.ts`
-   Remove logic and weirdness in `modifiablePipelineContext.tsx`
-   Remove logic and weirdness in `appContext.tsx -> dispatchUpdate`

Medium Priority:

-   Avoid duplicating descriptions in components; ALL values should come from the pipeline definitions

Low Priority:

-   Dropdowns and other "selection" logic will have a predefined method of querying the server for their values
-   Actions should have a way to define/return some sort of UI schema that can be automatically rendered by the client UI
