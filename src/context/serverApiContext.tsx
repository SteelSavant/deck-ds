import { ServerAPI } from 'decky-frontend-lib';
import * as React from 'react';

type ModifiablePipelineContextProviderProps = {
    children: React.ReactNode;
    serverApi: ServerAPI;
};

const ServerApiContext = React.createContext<ServerAPI | null>(null);

function ServerApiProvider({
    children,
    serverApi,
}: ModifiablePipelineContextProviderProps) {
    return (
        <ServerApiContext.Provider value={serverApi}>
            {children}
        </ServerApiContext.Provider>
    );
}

function useServerApi() {
    const context = React.useContext(ServerApiContext);
    if (context === null) {
        throw new Error('useServerApi must be used within a ServerApiProvider');
    }
    return context;
}

export { ServerApiProvider, useServerApi };
