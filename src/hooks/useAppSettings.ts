import { useEffect, useState } from "react";
import { getAppProfile } from "../backend";
import { ShortAppDetails } from "../context/shortAppDetailsContext";
import { AppProfile } from "../types/backend_api";
import { Loading } from "../util/loading";

const useAppProfile = (appDetails: ShortAppDetails | null): Loading<AppProfile | null> => {
    const [result, setResult] = useState<Loading<AppProfile | null>>(null);

    useEffect(() => {
        let active = true;

        if (result === null && appDetails) {
            (async function load() {
                const res = await getAppProfile({
                    app_id: appDetails.appId.toString()
                })

                if (!active) {
                    return;
                }

                setResult(res.map((v) => {
                    return v.app ?? null;
                }));
            })();
        }

        return () => { active = false; };
    });



    return result;
}

export default useAppProfile;