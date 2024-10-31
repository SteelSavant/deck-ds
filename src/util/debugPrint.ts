import {
    achievementListClasses,
    achievementPageClasses,
    appActionButtonClasses,
    appDetailsClasses,
    appDetailsHeaderClasses,
    basicAppDetailsSectionStylerClasses,
    focusRingClasses,
    footerClasses,
    gamepadContextMenuClasses,
    gamepadDialogClasses,
    gamepadLibraryClasses,
    gamepadSliderClasses,
    gamepadTabbedPageClasses,
    gamepadUIClasses,
    libraryAssetImageClasses,
    mainBrowserClasses,
    mainMenuAppRunningClasses,
    playSectionClasses,
    quickAccessControlsClasses,
    quickAccessMenuClasses,
    scrollPanelClasses,
    searchBarClasses,
    steamSpinnerClasses,
    updaterFieldClasses,
} from '@decky/ui';

export function debugPrintStyles() {
    const classes = {
        quickAccessMenuClasses: quickAccessMenuClasses,
        scrollPanelClasses: scrollPanelClasses,
        gamepadDialogClasses: gamepadDialogClasses,
        quickAccessControlsClasses: quickAccessControlsClasses,
        updaterFieldClasses: updaterFieldClasses,
        playSectionClasses: playSectionClasses,
        gamepadSliderClasses: gamepadSliderClasses,
        appDetailsHeaderClasses: appDetailsHeaderClasses,
        appDetailsClasses: appDetailsClasses,
        gamepadUIClasses: gamepadUIClasses,
        gamepadTabbedPageClasses: gamepadTabbedPageClasses,
        gamepadContextMenuClasses: gamepadContextMenuClasses,
        achievementListClasses: achievementListClasses,
        achievementPageClasses: achievementPageClasses,
        mainMenuAppRunningClasses: mainMenuAppRunningClasses,
        basicAppDetailsSectionStylerClasses:
            basicAppDetailsSectionStylerClasses,
        steamSpinnerClasses: steamSpinnerClasses,
        footerClasses: footerClasses,
        appActionButtonClasses: appActionButtonClasses,
        libraryAssetImageClasses: libraryAssetImageClasses,
        gamepadLibraryClasses: gamepadLibraryClasses,
        focusRingClasses: focusRingClasses,
        searchBarClasses: searchBarClasses,
        mainBrowserClasses: mainBrowserClasses,
        staticClasses: quickAccessMenuClasses,
        scrollClasses: scrollPanelClasses,
        achievementClasses: achievementListClasses,
    };

    for (const cls in classes) {
        console.log(cls, ':', (classes as any)[cls]);
    }
}
