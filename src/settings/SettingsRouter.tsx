import {
    SidebarNavigation
} from "decky-frontend-lib"
import { VFC } from "react"
import { HiOutlineArchive, HiOutlineTemplate } from "react-icons/hi"

const SettingsRouter: VFC = () => {
    const pages = [
        {
            title: 'Profiles',
            content: <div > Profiles </div>,
            icon: <HiOutlineArchive />,
            hideTitle: false
        },
        {
            title: 'Templates',
            content: <div> Templates </div>,
            icon: <HiOutlineTemplate />,
            hideTitle: false
        },
    ]

    return (
        <SidebarNavigation pages={pages} />
    )
};

export default SettingsRouter