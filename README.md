# DeckDS 
[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/S6S7U6S4P)

DeckDS is a Steam Deck plugin for easily configuring and launching multi-window applications from Gamemode.
The `DS` in the name is of dual significance, and stands for both Dual-Screen, and Desktop-Settings.

## What Does it Do?

This plugin allows applications to be launched in Desktop mode, from Game mode. Additionally, DeckDS has specialized support for emulators for systems with multiple screens, configured as best as possible to use an external monitor as a second screen for a near-native dual-screen experience.

### App Support

DeckDS allows launching Steam applications (both Steam and non-Steam titles are supported) in desktop mode. Additionally, the plugin supports launching a secondary application (currently only flatpaks are supported) simultaneously, allowing the user to play a game while interacting with an application like YouTube or Discord on the second display.

### Emulator Support

Configuration and display/window management for a small selection of emulators are currently supported. The following table lists the support status of supported emulators by install source:

- ✅ : Supported
- ☑️  : If it exists, should work if the emulator settings location is configured
- 🚧 : Planned/In-Progress
- ❌ : Not Supported/Not Planned

|              | Flatpak  | AppImage | Emudeck (Proton) | Other Portable/Binary | RetroArch 
|--------------|----------|----------|------------------|-----------------------|-----------
| Dolphin/mGBA | 🚧       | 🚧       | ➖               |  🚧                  | ❌         
| Cemu         | ✅       | ✅       | ✅               | ☑️                    | ❌         
| Citra        | ✅       | ☑️        | ➖               | ☑️                    | ❌   
| Citra forks*  | ➖       |      ➖  | ➖               |  ☑️                   | ❌   
| MelonDS      | ✅       | ☑️        | ➖               | ☑️                    | ❌        


*Citra forks include, but may not be limited to, [Lemonade](https://github.com/Lemonade-emu/Lemonade) and [Lime3DS](https://lime3ds.github.io/).


### Configuration 

Configuration profiles can be made from existing templates, and then customized as needed. Currently, only the emulator install source and layout options are configurable. 

Profiles can be applied to Steam categories, and can be overridden per-game to provide the best experience.

Configurations each have (up to) two launch targets: Desktop, and Gamemode. This allows customizing settings, such as display layout, per-profile and per-game; useful for Nintendo DS and 3DS emulators specifically.

### Desktop Display Settings

DeckDS also has basic support for changing display settings when swapping to desktop mode normally, in addition to when launching games. Configurable settings include:
- Resolution
- Refresh Rate
- Enabling/disabling the Deck's internal display
- Setting the location of the Deck's internal display relative to the primary output

## Decky Loader

This plugin requires [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader). DeckDS is (not yet) available on the store.

## Supported Platforms

This plugin is only supported on Steam Deck. I do not currently have the resources to develop for and test against other hardware or software platforms. That being said, if you're wont to tinker, tinker away; all I ask is that you don't file issues for problems encountered on other platforms. 

Desktop mode functionality makes use of:

- the `steamos-session-select` command for mode switching
- KDE's autostart functionality
- the KWin window manager
- `xrandr` (x11)

It will not work without them.

## License

DeckDS is licensed under GNU GPLv3.
