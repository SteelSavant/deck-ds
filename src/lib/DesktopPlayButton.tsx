import { DialogButton } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaDesktop } from "react-icons/fa";


interface DesktopPlayButtonProps {
    gameId: String
}

export default function DesktopPlayButton({
    gameId
}: DesktopPlayButtonProps): ReactElement {
    const onLaunch = () => {
        console.log('desktop', gameId);
        // TODO::this
    }
    const vPadding = 14;
    const wPadding = 17;
    return (
        <DialogButton
            // I would be thrilled if this matched the other buttons exactly, but alas...
            style={{
                minWidth: 0,
                paddingLeft: wPadding,
                paddingRight: wPadding,
                paddingTop: vPadding,
                paddingBottom: vPadding,
            }}
            onClick={onLaunch}
            onOKButton={onLaunch}
        >
            <FaDesktop />
        </DialogButton>
    )

}