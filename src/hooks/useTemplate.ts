import { Template } from '../backend';
import { Loading } from '../util/loading';
import useTemplates from './useTemplates';

const useTemplate = (templateId: string): Loading<Template | null> => {
    const templates = useTemplates();

    return templates?.map((t) => t.find((d) => d.id == templateId) ?? null);
};

export default useTemplate;
