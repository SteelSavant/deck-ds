import { useEffect, useState } from "react";
import { CreateProfileRequest, Profile, Response, SetProfileRequest, createProfile, deleteProfile, getProfile, getProfiles, setProfile, } from "../backend";
import { CreateProfileResponse, DeleteProfileRequest } from "../types/backend_api";
import { Loading } from "../util/loading";

interface Profiles {
    profiles: Loading<Array<Profile>>,
    createProfile: (request: CreateProfileRequest) => Response<CreateProfileResponse>,
    updateProfile: (request: SetProfileRequest) => Response<void>
    deleteProfile: (request: DeleteProfileRequest) => Response<void>
}

const useProfiles = (): Profiles => {
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



    return {
        profiles: result,
        // Recieving updates from the server for state changes, rather than requiring they proxy through here would be better,
        // but right now I'm not going to worry about figuring out bidirectional communication with the server.
        createProfile: async (request) => {
            const res = await createProfile(request);
            if (res.isOk) {
                const profileRes = await getProfile({ profile_id: res.data.profile_id });
                if (profileRes.isOk) {
                    const profile = profileRes.data.profile;
                    if (profile) {
                        setResult(result?.map((v) => [...v, profile]))
                    }
                }
            }

            return res;
        },
        updateProfile: async (request) => {
            const res = await setProfile(request);
            if (res.isOk) {
                setResult(result?.map((v: Profile[]) => v.map((e) => e.id == request.profile.id ? request.profile : e)))
            }

            return res;
        },
        deleteProfile: async (request) => {
            const res = await deleteProfile(request);
            if (res.isOk) {
                setResult(result?.map((v: Profile[]) => v.filter((e) => e.id !== request.profile)))
            }

            return res;
        },
    };
}

export default useProfiles;