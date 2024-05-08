import { useEffect, useState } from "react";
import { getDisplayInfo } from "../backend";
import { DisplayValues } from "../types/backend_api";
import { Loading } from "../util/loading";

const useDisplayInfo = (): Loading<Array<DisplayValues>> => {
    const [result, setResult] = useState<Loading<Array<DisplayValues>>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getDisplayInfo();

                if (!active) {
                    return;
                }

                setResult(res.map((v) => {
                    return v.available_values;
                }));
            })();
        }

        return () => { active = false; };
    });

    return result;
}

export default useDisplayInfo;