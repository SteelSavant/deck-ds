import { routerHook } from '@decky/api';
import { ShortAppDetailsState } from '../context/appContext';
import patchContextMenu, { LibraryContextMenu } from './patchContextMenu';
import patchLibraryApp from './patchLibraryApp';

interface Patch {
    route: any;
    unpatch: any;
}

export class PatchHandler {
    private static instance: PatchHandler | null;

    private isEnabled = false;
    private readonly patches: Array<{
        unpatch: () => void;
    }> = [];
    private readonly patchFns: Array<{
        route: any;
        patch: (route: any) => any;
        unpatch: (patch: Patch) => () => void;
    }>;

    private constructor(appDetailsState: ShortAppDetailsState) {
        this.patchFns = [
            {
                route: '/library/app/:appid',
                patch: (route: string) =>
                    patchLibraryApp(route, appDetailsState),
                unpatch: (patch: Patch) => () =>
                    routerHook.removePatch(patch.route, patch.unpatch),
            },
            {
                route: LibraryContextMenu,
                patch: patchContextMenu,
                unpatch: (patch: Patch) => patch.unpatch.unpatch,
            },
        ];
    }

    public setPatchEnabled(patchEnabled: boolean) {
        if (patchEnabled === this.isEnabled) {
            return;
        }

        this.isEnabled = patchEnabled;

        if (!patchEnabled) {
            for (const patch of this.patches) {
                patch.unpatch();
            }
            this.patches.length = 0;
        } else {
            for (const patch of this.patchFns) {
                this.patches.push({
                    unpatch: patch.unpatch({
                        route: patch.route,
                        unpatch: patch.patch(patch.route),
                    }),
                });
            }
        }
    }

    public static init(appDetailsState: ShortAppDetailsState) {
        if (this.instance) {
            return;
        }

        PatchHandler.instance = new PatchHandler(appDetailsState);
    }

    public static dispose() {
        this.instance?.setPatchEnabled(false);
    }

    public static getInstance(): PatchHandler {
        if (!PatchHandler.instance) {
            throw Error('PatchHandler Not Set');
        }
        return PatchHandler.instance;
    }
}
