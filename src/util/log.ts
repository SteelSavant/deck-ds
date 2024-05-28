import { Logger } from 'tslog';
import { LogLevel, getSettings, setSettings } from '../backend';

export let logger = new Logger({
    minLevel: LogLevel.Info,
    type: 'pretty',
    hideLogPositionForProduction: true,
});

export async function initLogger() {
    const currentSettings = await getSettings();
    if (currentSettings.isOk) {
        logger.settings.minLevel =
            currentSettings.data.global_settings.log_level;
    } else {
        logger.error(
            'failed to fetch backend settings when initializing logger',
        );
    }
}

export async function setLogLevel(level: LogLevel) {
    logger.settings.minLevel = level;
    const currentSettings = await getSettings();
    if (currentSettings.isOk) {
        const res = await setSettings({
            global_settings: {
                ...currentSettings.data.global_settings,
                log_level: level,
            },
        });

        if (!res.isOk) {
            logger.error('failed to set backend settings with new log level');
        }
    } else {
        logger.error('failed to fetch backend settings when setting log level');
    }
}
