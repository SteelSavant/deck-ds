import { ReactElement } from 'react';
import { FaDesktop, FaPlay } from 'react-icons/fa6';
import { PipelineTargetOrNative } from '../patch/hooks/useActionButtonProps';

export function IconForTarget({
    target,
}: {
    target: PipelineTargetOrNative;
}): ReactElement {
    // TODO::figure out how to make the FaIcons match the style, since currently it is ignored.
    const className = 'SVGIcon_Button SVGIcon_BigPicture';
    switch (target) {
        case 'Desktop':
            return <FaDesktop className={className} />;
        case 'Gamemode':
            // gamepad icon from Deck UI
            return (
                <svg
                    className={className}
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 36 36"
                    fill="none"
                    height="1.1em"
                >
                    <path
                        fill="currentColor"
                        fill-rule="evenodd"
                        clip-rule="evenodd"
                        d="M0 11C0 9.89543 0.895431 9 2 9H34C35.1046 9 36 9.89543 36 11V24C36 25.6569 34.6569 27 33 27H3C1.34315 27 0 25.6569 0 24V11ZM33 16C33 16.5523 32.5523 17 32 17C31.4477 17 31 16.5523 31 16C31 15.4477 31.4477 15 32 15C32.5523 15 33 15.4477 33 16ZM32 13C32.5523 13 33 12.5523 33 12C33 11.4477 32.5523 11 32 11C31.4477 11 31 11.4477 31 12C31 12.5523 31.4477 13 32 13ZM35 14C35 14.5523 34.5523 15 34 15C33.4477 15 33 14.5523 33 14C33 13.4477 33.4477 13 34 13C34.5523 13 35 13.4477 35 14ZM30 15C30.5523 15 31 14.5523 31 14C31 13.4477 30.5523 13 30 13C29.4477 13 29 13.4477 29 14C29 14.5523 29.4477 15 30 15ZM6 14C6 15.1046 5.10457 16 4 16C2.89543 16 2 15.1046 2 14C2 12.8954 2.89543 12 4 12C5.10457 12 6 12.8954 6 14ZM2.5 21C2.22386 21 2 21.2239 2 21.5V24.5C2 24.7761 2.22386 25 2.5 25H5.5C5.77614 25 6 24.7761 6 24.5V21.5C6 21.2239 5.77614 21 5.5 21H2.5ZM30 21.5C30 21.2239 30.2239 21 30.5 21H33.5C33.7761 21 34 21.2239 34 21.5V24.5C34 24.7761 33.7761 25 33.5 25H30.5C30.2239 25 30 24.7761 30 24.5V21.5ZM28 11H8V25H28V11Z"
                    ></path>
                </svg>
            );
        case 'Native':
            return <FaPlay className={className} />;
        default:
            const typecheck: never = target;
            throw `icon for target ${typecheck} failed to typecheck`;
    }
}
