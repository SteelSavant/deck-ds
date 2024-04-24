import { useEffect, useState } from "react";
import { getToplevel } from "../backend";
import { ToplevelInfo } from "../types/backend_api";
import { Loading } from "../util/loading";

const useToplevel = (): Loading<Array<ToplevelInfo>> => {
    const [result, setResult] = useState<Loading<Array<ToplevelInfo>>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getToplevel();

                if (!active) {
                    return;
                }

                setResult(res.map((v) => {
                    return v.toplevel;
                }));
            })();
        }

        return () => { active = false; };
    });

    return result;
}

export default useToplevel;