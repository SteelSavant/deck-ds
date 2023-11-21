// reference: https://github.com/SteamDeckHomebrew/decky-loader/blob/main/frontend/src/components/settings/index.tsx

import { DialogBody, DialogControlsSection } from 'decky-frontend-lib';
import { VFC } from 'react';
import useTemplates from '../../../hooks/useTemplates';
import TemplateMenuItem from './TemplateMenuItem';


export const TemplatesPage: VFC = () => {
    const templates = useTemplates();

    return <div>
        {
            templates === null
                ? <div />
                : templates.isOk
                    ? <DialogBody>
                        <DialogControlsSection>
                            {templates.data.map((t) => <TemplateMenuItem template={t} />)}
                        </DialogControlsSection>
                    </DialogBody>
                    : <div> Error loading templates! {templates.err} </div>
        }
    </div>
}