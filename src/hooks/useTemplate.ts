import { Template } from "../backend";
import { Loading } from "../util/loading";
import useTemplates from "./useTemplates";

const useTemplate = (templateId: string): Loading<Template | undefined> => {
    const templates = useTemplates();

    return templates?.map((t) => t.find((d) => d.id == templateId));
}

export default useTemplate;