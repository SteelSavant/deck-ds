/*

Consider https://github.com/alvr-org/settings-schema-rs as a base
Support
- uint
  - input
  - slider
- int
  - input
  - slider
- float
  - input
  - slider (?)
- bool
  - toggle
  - checkbox
- String
  - input
- Path (type=file|dir|both, ext_filter)
- Section (struct)
- Dictionary (?)
- List
  - array (fixed)
  - vec (not fixed)
- Dropdown
  - specified from:
    - Enum
    - array/vec
  - if data is a struct, can be marked editable for OneOf style selection

Any field should be able to be marked "hidden" from the UI.
Allow any UI type to be marked with metadata to specify custom UI handling;
useful for:
- runtime updates for connected hardware
- Special UI (add steam tag)
*/
