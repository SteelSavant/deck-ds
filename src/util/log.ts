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
        this.log(LogLevel.Trace, args);
    }

    public debug(...args: any[]) {
        this.log(LogLevel.Debug, args);
    }

    public info(...args: any[]) {
        this.log(LogLevel.Info, args);
    }

    public warn(...args: any[]) {
        this.log(LogLevel.Warn, args);
    }

    public error(...args: any[]) {
        this.log(LogLevel.Error, args);
    }

    public log(level: LogLevel, ...args: any[]) {
        if (level >= this.minLevel) {
            // TODO::would be nice if these formatted like normal console.log.
            console.log(`DeckDS::${this.getStringForLevel(level)}:`, ...args);
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
