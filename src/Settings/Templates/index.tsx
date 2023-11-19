// reference: https://github.com/SteamDeckHomebrew/decky-loader/blob/main/frontend/src/components/settings/index.tsx

import { ControlsList, DialogControlsSection, Spinner } from 'decky-frontend-lib';
import { VFC } from 'react';
import useTemplates from '../../hooks/useTemplates';
import TemplateMenuItem from './TemplateMenuItem';


export const TemplatesPage: VFC = () => {
    const templates = useTemplates();

    return <div>
        {
            templates === null
                ? <div> <Spinner /> </div>
                : templates.isOk
                    ? <DialogControlsSection>
                        <ControlsList>
                            {templates.data.map((t) => <TemplateMenuItem template={t} />)}
                        </ControlsList>
                    </DialogControlsSection>
                    : <div> Error loading templates! {templates.err} </div>
        }
    </div>
}