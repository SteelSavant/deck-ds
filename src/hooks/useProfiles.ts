import { useEffect, useState } from "react";
import { Profile, getProfiles, } from "../backend";
import { Loading } from "../util/loading";

const useProfiles = (): Loading<Array<Profile>> => {
    const [result, setResult] = useState<Loading<Array<Profile>>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getProfiles();

                if (!active) {
                    return;
                }

                setResult(res.map((v) => {
                    v.profiles.sort((a, b) =>
                        a.pipeline.name < b.pipeline.name ? -1
                            : a.pipeline.name > b.pipeline.name ? 1
                                : a.id < b.id ? -1
                                    : 1);
                    return v.profiles;
                }));
            })();
        }

        return () => { active = false; };
    });

    return result;
}

export default useProfiles;