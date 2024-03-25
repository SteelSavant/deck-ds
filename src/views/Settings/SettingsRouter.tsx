import {
    SidebarNavigation
} from "decky-frontend-lib";
import { VFC } from "react";
import { FaGears } from "react-icons/fa6";
import { HiFolder } from "react-icons/hi";
import { GlobalSettingsPage } from "./GlobalSettings";
import { ProfilesPage } from "./Profiles";

const SettingsRouter: VFC = () => {
    const pages = [
        {
            title: 'Profiles',
            content: <div > <ProfilesPage /> </div>,
            icon: <HiFolder />,
            hideTitle: false,
            identifier: 'profiles',
        },
        {
            title: 'Settings',
            content: <div> <GlobalSettingsPage /> </div>,
            icon: <FaGears />,
            hideTitle: true,
            identifier: 'global',
        },
    ].map((p) => {
        return {
            route: `/deck-ds/settings/${p.identifier}`,
            ...p,
        }
    });

    return (
        <SidebarNavigation pages={pages} />
    )
};

export default SettingsRouter