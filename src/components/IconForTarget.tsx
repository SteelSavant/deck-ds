import { ReactElement } from "react";
import { FaDesktop, FaGamepad } from "react-icons/fa6";
import { PipelineTarget } from "../backend";

export function IconForTarget({ target }: { target: PipelineTarget }): ReactElement {
    switch (target) {
        case 'Desktop':
            return <FaDesktop />;
        case 'Gamemode':
            return <FaGamepad />; // TODO::proper handheld icon
        default:
            const typecheck: never = target;
            throw `icon for target ${typecheck} failed to typecheck`
    }
}