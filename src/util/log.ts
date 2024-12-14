import { toaster } from '@decky/api';
import { log as backend_log } from '../backend';

export enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
}

class Logger {
    /// please don't set this outside of the init function (in the backend)
    public minLevel: LogLevel;

    public constructor() {
        this.minLevel = LogLevel.Info;
    }

    public trace(...args: any[]) {
        this.log(LogLevel.Trace, true, ...args);
    }

    public debug(...args: any[]) {
        this.log(LogLevel.Debug, true, ...args);
    }

    public info(...args: any[]) {
        this.log(LogLevel.Info, true, ...args);
    }

    public warn(...args: any[]) {
        this.log(LogLevel.Warn, true, ...args);
    }

    public error(...args: any[]) {
        this.log(LogLevel.Error, true, ...args);
    }

    public trace_nobackend(...args: any[]) {
        this.log(LogLevel.Trace, false, ...args);
    }

    public debug_nobackend(...args: any[]) {
        this.log(LogLevel.Debug, false, ...args);
    }

    public info_nobackend(...args: any[]) {
        this.log(LogLevel.Info, false, ...args);
    }

    public warn_nobackend(...args: any[]) {
        this.log(LogLevel.Warn, false, ...args);
    }

    public error_nobackend(...args: any[]) {
        this.log(LogLevel.Error, false, ...args);
    }

    public log(level: LogLevel, sendToBackend: boolean, ...args: any[]) {
        if (level >= this.minLevel) {
            // TODO::would be nice if these formatted like normal console.log.
            console.log(`DeckDS::${this.getStringForLevel(level)}:`, ...args);
            if (sendToBackend) {
                try {
                    backend_log(level, args.toString());
                } catch (ex) {
                    this.warn_nobackend('failed to log', ...args, ':', ex);
                }
            }
        }
    }

    public toastWarn(...args: any[]) {
        this.warn(...args);
        if (toaster) {
            toaster.toast({
                title: 'Error', // don't differentiate between warning and error to the user, since either way, something broke.
                body: args.join(' '),
            });
        }
    }

    public toastError(...args: any[]) {
        this.error(...args);
        if (toaster) {
            toaster.toast({
                title: 'Error',
                body: args.join(' '),
            });
        }
    }

    private getStringForLevel(level: LogLevel) {
        switch (level) {
            case LogLevel.Trace:
                return 'TRACE';
            case LogLevel.Debug:
                return 'DEBUG';
            case LogLevel.Info:
                return 'INFO';
            case LogLevel.Warn:
                return 'WARN';
            case LogLevel.Error:
                return 'ERROR';
        }
    }
}

export const logger = new Logger();
