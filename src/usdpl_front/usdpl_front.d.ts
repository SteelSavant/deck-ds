/* tslint:disable */
/* eslint-disable */
/**
* Initialize the front-end library
* @param {number} port
*/
export function init_usdpl(port: number): void;
/**
* Get the targeted plugin framework, or "any" if unknown
* @returns {string}
*/
export function target_usdpl(): string;
/**
* Get the UDSPL front-end version
* @returns {string}
*/
export function version_usdpl(): string;
/**
* Get the targeted plugin framework, or "any" if unknown
* @param {string} key
* @param {any} value
* @returns {any}
*/
export function set_value(key: string, value: any): any;
/**
* Get the targeted plugin framework, or "any" if unknown
* @param {string} key
* @returns {any}
*/
export function get_value(key: string): any;
/**
* Call a function on the back-end.
* Returns null (None) if this fails for any reason.
* @param {string} name
* @param {any[]} parameters
* @returns {Promise<any>}
*/
export function call_backend(name: string, parameters: any[]): Promise<any>;
/**
* Initialize translation strings for the front-end
* @param {string} locale
* @returns {Promise<void>}
*/
export function init_tr(locale: string): Promise<void>;
/**
* Translate a phrase, equivalent to tr_n(msg_id, 0)
* @param {string} msg_id
* @returns {string}
*/
export function tr(msg_id: string): string;
/**
* Translate a phrase, retrieving the plural form for `n` items
* @param {string} msg_id
* @param {number} n
* @returns {string}
*/
export function tr_n(msg_id: string, n: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly init_usdpl: (a: number) => void;
  readonly target_usdpl: (a: number) => void;
  readonly version_usdpl: (a: number) => void;
  readonly set_value: (a: number, b: number, c: number) => number;
  readonly get_value: (a: number, b: number) => number;
  readonly call_backend: (a: number, b: number, c: number, d: number) => number;
  readonly init_tr: (a: number, b: number) => number;
  readonly tr: (a: number, b: number, c: number) => void;
  readonly tr_n: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_export_0: (a: number) => number;
  readonly __wbindgen_export_1: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_export_3: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_4: (a: number, b: number) => void;
  readonly __wbindgen_export_5: (a: number) => void;
  readonly __wbindgen_export_6: (a: number, b: number, c: number, d: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;


// USDPL customization
export function init_embedded();
