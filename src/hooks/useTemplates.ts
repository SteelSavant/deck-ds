import { useEffect, useState } from "react";
import { Template, getTemplates } from "../backend";
import { Loading } from "../util/loading";

const useTemplates = (): Loading<Array<Template>> => {
    const [result, setResult] = useState<Loading<Array<Template>>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getTemplates();

                if (!active) {
                    return;
                }

                setResult(res.map((v) => {
                    v.templates.sort((a, b) => {
                        const aName = a.pipeline.name.toLowerCase();
                        const bName = b.pipeline.name.toLowerCase();

                        return aName < bName ? -1
                            : aName > bName ? 1
                                : a.id < b.id ? -1
                                    : 1
                    });
                    return v.templates;
                }));
            })();
        }

        return () => { active = false; };
    });

    return result;
}

export default useTemplates;