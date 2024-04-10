// Adapted from https://github.com/jurassicplayer/decky-autosuspend/blob/main/main.py, BSD-3 Clause License


import { DialogBody, DialogControlsSection, Field } from "decky-frontend-lib";
import { VFC } from "react";
import { useServerApi } from "../../context/serverApiContext";
import usePluginInfo from "../../hooks/usePluginInfo";


const About: VFC = () => {
    const serverApi = useServerApi();
    let pluginInfo = usePluginInfo(serverApi);


    let fields = [
        pluginInfo ? {
            label: `${pluginInfo.name} v${pluginInfo.version}`,
            description: "Thanks to: AAGaming00, Beebles, EMERALD0874, NGenius, and all the other plugin devs",
            onClick: () => SteamClient.URL.ExecuteSteamURL("steam://openurl/https://github.com/SteelSavant/deck-ds")
        } : null,
        {
            label: "Support",
            description: "Support the project on Ko-Fi: https://ko-fi.com/steelsavant",
            onClick: () => SteamClient.URL.ExecuteSteamURL("steam://openurl/https://ko-fi.com/steelsavant"),
            onOKActionDescription: "Support!"
        },
        {
            label: "BSD 3-Clause License",
            description: (<div style={{ whiteSpace: "pre-wrap" }}>
                Copyright (c) 2023-2024, SteelSavant<br />
                Original Copyright (c) 2022, Steam Deck Homebrew<br />
                <br />
                All rights reserved.<br />
                <br />
                Redistribution and use in source and binary forms, with or without
                modification, are permitted provided that the following conditions are met:<br />
                <br />
                1. Redistributions of source code must retain the above copyright notice, this
                list of conditions and the following disclaimer.<br />
                <br />
                2. Redistributions in binary form must reproduce the above copyright notice,
                this list of conditions and the following disclaimer in the documentation
                and/or other materials provided with the distribution.<br />
                <br />
                3. Neither the name of the copyright holder nor the names of its
                contributors may be used to endorse or promote products derived from
                this software without specific prior written permission.<br />
                <br />
                THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
                AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
                IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
                DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
                FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
                DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
                SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
                CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
                OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
                OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
            </div>)
        },
        {
            label: "Github",
            description: "https://github.com/SteelSavant",
            onClick: () => SteamClient.URL.ExecuteSteamURL("steam://openurl/https://github.com/SteelSavant")
        },
    ].filter((v) => v);

    return (
        <DialogBody>
            <DialogControlsSection>
                {fields.map(field => <Field focusable={true} {...field} />)}
            </DialogControlsSection>
        </DialogBody>
    )
}

export default About


