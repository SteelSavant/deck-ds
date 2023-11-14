import { VFC, useEffect, useState } from "react";
import { getProfiles, GetProfilesResponse, Profile } from "../../backend";

export const ProfilesPage: VFC = () => {
    const [loading, setLoading] = useState(false);
    const [profiles, setProfiles] = useState(Array<Profile>);

    useEffect(() => {
        const loadProfiles = async () => {
            setLoading(true);

            const response = await getProfiles();

            if (response.ok) {
                setProfiles(response.data.profiles)
            } else {
                console.log(response.err);
                setTimeout(() => {
                    loadProfiles();
                }, 5000);
            }
        }
    })



    return <div>
        <div> Profiles</div>
        loading
        ? <div> Loading...</div>
        : <div> Got {profiles.length} Profiles!</div>
    </div>
}