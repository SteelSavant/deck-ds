import { createContext } from "react";
import { DependencyError } from "../backend";

type ConfigErrors = {
    [k: string]: DependencyError[];
};
export const ConfigErrorContext = createContext<ConfigErrors>({});