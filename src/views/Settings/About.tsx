// Adapted from https://github.com/jurassicplayer/decky-autosuspend/blob/main/main.py, BSD-3 Clause License

import { DialogBody, DialogControlsSection, Field } from '@decky/ui';
import { VFC } from 'react';
import usePluginInfo from '../../hooks/usePluginInfo';

const About: VFC = () => {
    const pluginInfo = usePluginInfo();

    const fields = [
        pluginInfo
            ? {
                  label: `${pluginInfo.name} v${pluginInfo.version}`,
                  description:
                      'Thanks to: AAGaming00, Beebles, EMERALD0874, NGenius, and all the other plugin devs.',
                  onClick: () =>
                      SteamClient.URL.ExecuteSteamURL(
                          'steam://openurl/https://github.com/SteelSavant/deck-ds',
                      ),
              }
            : null,
        {
            label: 'Support',
            description:
                'Support the project on Ko-Fi: https://ko-fi.com/steelsavant',
            onClick: () =>
                SteamClient.URL.ExecuteSteamURL(
                    'steam://openurl/https://ko-fi.com/steelsavant',
                ),
            onOKActionDescription: 'Support!',
        },
        {
            label: 'Github',
            description: 'https://github.com/SteelSavant',
            onClick: () =>
                SteamClient.URL.ExecuteSteamURL(
                    'steam://openurl/https://github.com/SteelSavant',
                ),
        },
        {
            label: 'GNU GPLv3 License',
            description: (
                <div style={{ whiteSpace: 'pre-wrap' }}>
                    DeckDS <br />
                    Copyright (c) 2023-2024 SteelSavant <br />
                    <br />
                    This program is free software: you can redistribute it
                    and/or modify <br />
                    it under the terms of the GNU General Public License as
                    published by <br />
                    the Free Software Foundation, either version 3 of the
                    License, or <br />
                    (at your option) any later version. <br />
                    <br />
                    This program is distributed in the hope that it will be
                    useful, <br />
                    but WITHOUT ANY WARRANTY; without even the implied warranty
                    of <br />
                    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the{' '}
                    <br />
                    GNU General Public License for more details. <br />
                    <br />
                    You should have received a copy of the GNU General Public
                    License <br />
                    along with this program. If not, see
                    https://www.gnu.org/licenses. <br />
                </div>
            ),
        },
    ].filter((v) => v);

    return (
        <DialogBody>
            <DialogControlsSection>
                {fields.map((field) => (
                    <Field focusable={true} {...field} />
                ))}
            </DialogControlsSection>
        </DialogBody>
    );
};

export default About;
