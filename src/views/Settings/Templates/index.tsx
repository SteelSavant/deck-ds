// reference: https://github.com/SteamDeckHomebrew/decky-loader/blob/main/frontend/src/components/settings/index.tsx

import { DialogBody, DialogControlsSection } from 'decky-frontend-lib';
import { VFC } from 'react';
import HandleLoading from '../../../components/HandleLoading';
import useTemplates from '../../../hooks/useTemplates';
import TemplateMenuItem from './TemplateMenuItem';


export const TemplatesPage: VFC = () => {
    const templates = useTemplates();

    return <HandleLoading
        value={templates}
        onOk={
            (templates) => <DialogBody>
                <DialogControlsSection>
                    {templates.map((t) => <TemplateMenuItem template={t} />)}
                </DialogControlsSection>
            </DialogBody>
        }
    />;
}