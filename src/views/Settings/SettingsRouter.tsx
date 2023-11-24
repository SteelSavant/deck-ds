import {
    SidebarNavigation
} from "decky-frontend-lib";
import { VFC } from "react";
import { HiOutlineArchive, HiOutlineTemplate } from "react-icons/hi";
import { ProfilesPage } from "./Profiles";
import { TemplatesPage } from "./Templates";

const SettingsRouter: VFC = () => {
    const pages = [
        {
            title: 'Profiles',
            content: <div > <ProfilesPage /> </div>,
            icon: <HiOutlineArchive />,
            hideTitle: false,
            identifier: 'profiles',
        },
        {
            title: 'Templates',
            content: <div> <TemplatesPage /> </div>,
            icon: <HiOutlineTemplate />,
            hideTitle: false,
            identifier: 'templates',
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