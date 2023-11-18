
module Impl {
    export class Option<T, O extends boolean, NO extends boolean> {
        readonly isSome: O;
        readonly data: T;

        get isNone(): NO { return !this.isSome as NO; };

        constructor(data: T, ok: O) {
            this.isSome = ok;
            this.data = data;
        }
    }
}

export module Option {
    export interface Some<T> {
        isSome: true,
        isNone: false,
        data: T,
    }

    export interface None<T> {
        isSome: false,
        isNone: true,
    }
}


export type Option<T> = Option.Some<T> | Option.None<T>;

export function Some<T>(data: T): Option.Some<T> {
    return new Impl.Option(data, true);
}

export function None<T>(): Option.None<T> {
    return new Impl.Option(undefined as T, false);
}