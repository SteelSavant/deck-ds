import { routerHook } from '@decky/api';
import { ShortAppDetailsState } from '../context/appContext';
import patchLibraryApp from './patchLibraryApp';

export class PatchHandler {
    private static instance: PatchHandler | null;

    private isEnabled = false;
    private patches: Array<{
        route: string;
        patch: any;
    }> = [];
    private patchFns: Array<{
        route: string;
        fn: Function;
    }>;

    private constructor(appDetailsState: ShortAppDetailsState) {
        this.patchFns = [
            {
                route: '/library/app/:appid',
                fn: (route: string) => patchLibraryApp(route, appDetailsState),
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
                routerHook.removePatch(patch.route, patch.patch);
            }
            this.patches.length = 0;
        } else {
            for (const patch of this.patchFns) {
                this.patches.push({
                    route: patch.route,
                    patch: patch.fn(patch.route),
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
