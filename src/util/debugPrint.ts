import {
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    playSectionClasses,
} from '@decky/ui';

export function debugPrintStyles() {
    const playSectionClassesOptions = [
        'AchievementCountLabel',
        'AchievementProgressRow',
        'ActionSection',
        'AppButtonsContainer',
        'Arrow',
        'AvatarAndPersona',
        'BreakNarrow',
        'BreakShort',
        'BreakTall',
        'BreakUltraWide',
        'BreakWide',
        'ClickablePlayBarItem',
        'CloudStatusIcon',
        'CloudStatusLabel',
        'CloudStatusRow',
        'CloudSyncProblem',
        'CloudSynching',
        'ComingSoon',
        'Container',
        'DetailsProgressBar',
        'DetailsProgressContainer',
        'DetailsSection',
        'DetailsSectionExtra',
        'DetailsSectionStatus',
        'DotDotDot',
        'DownloadPaused',
        'DownloadProgressBar',
        'Downloading',
        'FavoriteButton',
        'Favorited',
        'GameInfoButton',
        'GameStat',
        'GameStatIcon',
        'GameStatIconForced',
        'GameStatRight',
        'GameStatsSection',
        'GamepadUIBreakNarrow',
        'GamepadUIBreakShort',
        'GamepadUIBreakWide',
        'Glassy',
        'HideWhenNarrow',
        'Icon',
        'Icons',
        'InPage',
        'InnerContainer',
        'InvalidPlatform',
        'ItemFocusAnim-darkGrey',
        'ItemFocusAnim-darkerGrey',
        'ItemFocusAnim-darkerGrey-nocolor',
        'ItemFocusAnim-green',
        'ItemFocusAnim-grey',
        'ItemFocusAnimBorder-darkGrey',
        'Label',
        'LastPlayed',
        'LastPlayedInfo',
        'MenuActive',
        'MenuButton',
        'MiniAchievements',
        'OfflineMode',
        'OnlyDownloadBar',
        'PermanentlyUnavailable',
        'PlayBar',
        'PlayBarCloudStatusContainer',
        'PlayBarDetailLabel',
        'PlayBarGameIcon',
        'PlayBarGameName',
        'PlayBarIconAndGame',
        'PlayBarLabel',
        'Playtime',
        'PlaytimeIcon',
        'PlaytimeIconForced',
        'PortraitBar',
        'Presale',
        'RecentlyUpdated',
        'RecentlyUpdatedIcon',
        'RecentlyUpdatedLink',
        'RecentlyUpdatedText',
        'RightBreakNarrow',
        'RightBreakUltraNarrow',
        'RightBreakUltraWide',
        'RightBreakWide',
        'RightControls',
        'Row',
        'SharedLibrary',
        'StatusAndStats',
        'StatusNameContainer',
        'StickyHeader',
        'StickyHeaderShadow',
        'SuperimposedGridItems',
        'SyncAnim',
        'Visible',
        'duration-app-launch',
        'favorited',
        'focusAnimation',
        'hoverAnimation',
    ].map((v) => [v, (playSectionClasses as any)[v] as string] as const);

    const appDetailsClassesOptions = [
        'BreakNarrow',
        'BreakShort',
        'BreakTall',
        'BreakUltraWide',
        'BreakWide',
        'Container',
        'GamepadUIBreakNarrow',
        'GamepadUIBreakShort',
        'GamepadUIBreakWide',
        'Glassy',
        'Header',
        'HeaderLoaded',
        'InnerContainer',
        'ItemFocusAnim-darkGrey',
        'ItemFocusAnim-darkerGrey',
        'ItemFocusAnim-darkerGrey-nocolor',
        'ItemFocusAnim-green',
        'ItemFocusAnim-grey',
        'ItemFocusAnimBorder-darkGrey',
        'PlayBar',
        'PreventScrolling',
        'RightBreakNarrow',
        'RightBreakUltraNarrow',
        'RightBreakUltraWide',
        'RightBreakWide',
        'ScrollContainer',
        'ShowPlayBar',
        'Throbber',
        'duration-app-launch',
        'fadein',
        'focusAnimation',
        'hoverAnimation',
    ].map((v) => [v, (appDetailsClasses as any)[v] as string] as const);

    const basicAppDetailsSectionStylerClassesOptions = [
        'duration-app-launch',
        'headerPadding',
        'Header',
        'AppDetailsContent',
        'AppDetailsContainer',
        'AppDetailsRoot',
        'GameInfoContainer',
        'GameInfoQuickLinks',
        'GameInfoCollections',
        'CollectionsHeader',
        'PlaySection',
        'ActionRow',
        'AppDetailSectionList',
        'AppActionButton',
        'ActionButtonAndStatusPanel',
        'AppButtons',
        'InvertFocusedIcon',
        'DeckVerifiedFeedbackContainer',
        'DeckVerifiedFeedbackConfirmationContainer',
        'DeckVerifiedFeedbackButton',
        'DeckVerifiedFeedbackQuestion',
        'DeckVerifiedFeedbackConfirmation',
    ].map(
        (v) =>
            [
                v,
                (basicAppDetailsSectionStylerClasses as any)[v] as string,
            ] as const,
    );

    console.log('play section classes:', playSectionClassesOptions);
    console.log('app details classes:', appDetailsClassesOptions);
    console.log(
        'basic app details section  classes:',
        basicAppDetailsSectionStylerClassesOptions,
    );
}