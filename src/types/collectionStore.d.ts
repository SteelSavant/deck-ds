// Taken from https://github.com/Tormak9970/TabMaster/blob/bcde232b68caab07e7381713fc690e4f0ac898c0/src/types/stores/collectionStore.d.ts

// Types for the collectionStore global

type AppCollectionType =
    | 'type-games'
    | 'type-software'
    | 'type-music'
    | 'type-videos'
    | 'type-tools';

type CollectionStore = {
    appTypeCollectionMap: Map<AppCollectionType, Collection>;
    userCollections: SteamCollection[];
    allGamesCollection: Collection;
    deckDesktopApps: Collection | null;
    userCollections: Collection[];
    localGamesCollection: Collection;
    allAppsCollection: Collection;
    BIsHidden: (appId: number) => boolean;
    SetAppsAsHidden: (appIds: number[], hide: boolean) => void;
    GetUserCollectionsByName: (name: string) => SteamCollection[];
    GetCollectionListForAppID: (appId: number) => Collection[];
    GetCollection: (id: SteamCollection['id']) => Collection;
};

type SteamCollection = {
    AsDeletableCollection: () => null;
    AsDragDropCollection: () => null;
    AsEditableCollection: () => null;
    GetAppCountWithToolsFilter: (t: any) => any;
    allApps: SteamAppOverview[];
    apps: Map<number, SteamAppOverview>;
    bAllowsDragAndDrop: boolean;
    bIsDeletable: boolean;
    bIsDynamic: boolean;
    bIsEditable: boolean;
    displayName: string;
    id: string;
    visibleApps: SteamAppOverview[];
};

type Collection = {
    AsDeletableCollection: () => null;
    AsDragDropCollection: () => null;
    AsEditableCollection: () => null;
    GetAppCountWithToolsFilter: (t) => any;
    allApps: SteamAppOverview[];
    apps: Set<number>;
    bAllowsDragAndDrop: boolean;
    bIsDeletable: boolean;
    bIsDynamic: boolean;
    bIsEditable: boolean;
    displayName: string;
    id: string;
    visibleApps: SteamAppOverview[];
};
