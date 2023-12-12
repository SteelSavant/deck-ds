import { useEffect, useState } from "react";
import { Profile, getProfile, } from "../backend";
import { Loading } from "../util/loading";

const useProfile = (profileId: string): Loading<Profile | null | undefined> => {
    const [result, setResult] = useState<Loading<Profile | null | undefined>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
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