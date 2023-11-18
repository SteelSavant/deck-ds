import { VFC } from 'react';
import useTemplates from '../../hooks/useTemplates';


export const TemplatesPage: VFC = () => {
    const templates = useTemplates();

    return <div>
        <div> Templates</div>
        {
            templates.isNone
                ? <div> Loading...</div>
                : templates.data.isOk
                    ? <div> Got {length} Templates!</div>
                    : <div> Error loading templates! {templates.data.err} </div>
        }
    </div>
}