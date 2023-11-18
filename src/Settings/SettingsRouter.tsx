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
            hideTitle: false
        },
        {
            title: 'Templates',
            content: <div> <TemplatesPage /> </div>,
            icon: <HiOutlineTemplate />,
            hideTitle: false
        },
    ]

    return (
        <SidebarNavigation pages={pages} />
    )
};

export default SettingsRouter