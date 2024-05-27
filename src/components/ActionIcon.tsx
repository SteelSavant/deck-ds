import { ReactElement } from 'react';
import { FaLink } from 'react-icons/fa6';
import { PipelineAction } from '../backend';

export default function ActionIcon({
    action,
}: {
    action: PipelineAction;
}): ReactElement | null {
    return action.id.split(':').length === 3 &&
        action.selection.type !== 'AllOf' ? (
        <FaLink
            style={{
                paddingLeft: '10px',
                paddingRight: '10px',
            }}
        />
    ) : null;
}
