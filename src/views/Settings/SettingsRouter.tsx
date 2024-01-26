import {
    SidebarNavigation
} from "decky-frontend-lib";
import { VFC } from "react";
import { FaGears } from "react-icons/fa6";
import { HiFolder, HiTemplate } from "react-icons/hi";
import { GlobalSettingsPage } from "./GlobalSettings";
import { ProfilesPage } from "./Profiles";
import { TemplatesPage } from "./Templates";

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
            title: 'Templates',
            content: <div> <TemplatesPage /> </div>,
            icon: <HiTemplate />,
            hideTitle: false,
            identifier: 'templates',
        },
        {
            title: 'Settings',
            content: <div> <GlobalSettingsPage /> </div>,
            icon: <FaGears />,
            hideTitle: false,
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