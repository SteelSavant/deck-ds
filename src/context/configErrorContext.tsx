import { createContext } from "react";
import { DependencyError } from "../types/backend_api";

type ConfigErrors = {
    [k: string]: DependencyError[];
};
export const ConfigErrorContext = createContext<ConfigErrors>({});