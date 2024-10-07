import { routerHook } from '@decky/api';
import { EventEmitter } from 'events';
import { ShortAppDetailsState } from '../context/appContext';
import patchLibraryApp from './patchLibraryApp';

export class PatchEmitter extends EventEmitter {
    private static instance: PatchEmitter | null;

    private patches: Array<{
        route: string;
        patch: any;
    }> = [];
    private patchFns: Array<{
        route: string;
        fn: Function;
    }>;

    private constructor(
        patchEnabled: boolean,
        appDetailsState: ShortAppDetailsState,
    ) {
        super();
        this.patchFns = [
            {
                route: '/library/app/:appid',
                fn: (route: string) => patchLibraryApp(route, appDetailsState),
            },
        ];
        this.setPatchEnabled(patchEnabled);
        this.on('statusChange', (isEnabled) => {
            this.setPatchEnabled(isEnabled);
        });
    }

    private setPatchEnabled(patchEnabled: boolean) {
        if (!patchEnabled) {
            for (const patch of this.patches) {
                routerHook.removePatch(patch.route, patch.patch);
            }
        } else {
            for (const patch of this.patchFns) {
                this.patches.push({
                    route: patch.route,
                    patch: patch.fn(patch.route),
                });
            }
        }
    }

    public static init(
        patchEnabled: boolean,
        appDetailsState: ShortAppDetailsState,
    ) {
        if (this.instance) {
            return;
        }

        PatchEmitter.instance = new PatchEmitter(patchEnabled, appDetailsState);
    }

    public static dispose() {
        this.instance?.setPatchEnabled(false);
        PatchEmitter.instance = null;
    }

    public static getInstance(): PatchEmitter {
        if (!PatchEmitter.instance) {
            throw Error('PatchEmitter Not Set');
        }
        return PatchEmitter.instance;
    }
}
