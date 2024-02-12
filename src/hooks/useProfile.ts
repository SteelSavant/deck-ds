import { useEffect, useState } from "react";
import { CategoryProfile, getProfile, } from "../backend";
import { Loading } from "../util/loading";

const useProfile = (profileId: string | null): Loading<CategoryProfile | null | undefined> => {
    const [result, setResult] = useState<Loading<CategoryProfile | null | undefined>>(null);

    useEffect(() => {
        let active = true;

        if (result === null && profileId != null) {
            (async function load() {
                const res = await getProfile({ profile_id: profileId });

                if (!active) {
                    return;
                }

                setResult(res.map((p) => p.profile));
            })();
        }

        return () => { active = false; };
    }, [profileId]);

    return result;
}

export default useProfile;