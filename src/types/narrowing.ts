import { EnabledFor_String, PipelineAction, Selection } from "./backend_api";

export type ActionSelection = {
    Action: PipelineAction;
};

export type OneOfSelection = {
    OneOf: {
        actions: string[];
        selection: string;
    }
}

export type AllOfSelection = {
    AllOf: EnabledFor_String[];
}

export function isAction(p: Selection): p is ActionSelection { return !!(p as any).Action; };
export function isOneOf(p: Selection): p is OneOfSelection { return !!(p as any).OneOf; };
export function isAllOf(p: Selection): p is AllOfSelection { return !!(p as any).AllOf; };
