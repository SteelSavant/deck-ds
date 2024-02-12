import { PipelineTarget } from "../backend";
import useGlobalSettings from "./useGlobalSettings";
import useProfile from "./useProfile";

export interface AppTargets {
    primaryTarget?: PipelineTarget | null | undefined
    secondaryTarget?: PipelineTarget | null | undefined
}

export default function useAppTarget({ isPrimary, profileId }: { isPrimary: boolean, profileId: string | undefined }): PipelineTarget | undefined {
    const { settings } = useGlobalSettings();
    const profile = useProfile(profileId ?? null);

    if (profile?.isOk && settings?.isOk && settings.data.enable_ui_inject) {
        // TODO::may need to consider logic where only one target exists
        const primaryTarget = profile.data?.pipeline.primary_target_override ?? settings.data.primary_ui_target;
        const secondaryTarget: PipelineTarget = primaryTarget === 'Desktop'
            ? 'Gamemode'
            : 'Desktop';
        return isPrimary
            ? primaryTarget
            : secondaryTarget;
    }

    return undefined;
}