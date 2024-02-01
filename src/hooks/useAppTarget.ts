import { PipelineTarget } from "../backend";
import useGlobalSettings from "./useGlobalSettings";

export interface AppTargets {
    primaryTarget?: PipelineTarget | null | undefined
    secondaryTarget?: PipelineTarget | null | undefined
}

export default function useAppTarget({ isPrimary }: { isPrimary: boolean }): PipelineTarget | undefined {
    const { settings } = useGlobalSettings();

    if (settings?.isOk && settings.data.enable_ui_inject) {
        // TODO::may need to consider logic where only one target exists
        const primaryTarget = settings.data.primary_ui_target;
        const secondaryTarget: PipelineTarget = primaryTarget === 'Desktop'
            ? 'Gamemode'
            : 'Desktop';
        return isPrimary
            ? primaryTarget
            : secondaryTarget;
    }

    return undefined;
}