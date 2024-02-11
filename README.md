# DeckDS 
[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/S6S7U6S4P)

DeckDS is a Steam Deck plugin for easily configuring and launching desktop and dual-screen applications from Game mode.
The 'DS' in the name is of dual significance, and stands for both Dual-Screen, and Desktop-Start.

## What Does it Do?

This plugin allows applications to be launched in Desktop mode, from Game mode. DeckDS has specialized support for emulators for systems with multiple screens, configured as best as possible to use an external monitor as a second screen for a near-native dual-screen experience.

### Emulator Support

A small selection of emulators are currently supported. The following table lists the support status of supported emulators by install source:


|              | Flatpak  | AppImage | Emudeck (Proton) | Other Portable/Binary | RetroArch 
|--------------|----------|----------|------------------|-----------------------|-----------
| Dolphin/mGBA | 🚧       | 🚧       | ➖               |  🚧                  | ❌         
| Cemu         | ✅       | ✅       | ✅               | ☑️                    | ❌         
| Citra        | ✅       | ☑️        | ➖               | ☑️                    | ❌        
| MelonDS      | ✅       | ☑️        | ➖               | ☑️                    | ❌        

- ✅ : Supported
- ☑️  : If it exists, should work if the emulator settings location is configured
- 🚧 : Planned/In-Progress
- ❌ : Not Supported/Not Planned

### Configuration 

Configuration profiles can be made from existing emulator templates, and then customized as needed. Currently, only the emulator install source and layout options are configurable. 

Profiles can be applied to Steam categories, and can be overridden per-game to provide the best experience.

Configurations each have two launch targets: Desktop, and Gamemode. This allows customizing settings, such as display layout, per-profile and per-game; useful for Nintendo DS and 3DS emulators specifically.

## Decky Loader

This plugin requires [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader). DeckDS is (not yet) available on the store.

## Supported Platforms

This plugin is only supported on Steam Deck. Desktop mode functionality makes use of:

- the `steamos-session-select` command for mode switching
- KDE's autostart functionality
- the KWin window manager
- `xrandr` (x11)

It will not work without them.

## License

DeckDS is licensed under GNU GPLv3.
