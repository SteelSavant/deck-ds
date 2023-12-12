import { Profile } from "../backend";
import { Loading } from "../util/loading";
import useProfiles from "./useProfiles";

const useProfile = (profileId: string): Loading<Profile | undefined> => {
    const profiles = useProfiles();

    return profiles?.map((t) => t.find((d) => d.id == profileId));
}

export default useProfile;