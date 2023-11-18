
module Impl {
    export class Result<T, E, O extends boolean> {
        readonly ok: O;
        readonly data: T;
        readonly err: E;

        constructor(data: T, err: E, ok: O) {
            this.ok = ok;
            this.data = data;
            this.err = err;
        }
    }
}

export module Result {
    export interface Ok<T> {
        ok: true,
        data: T,
    }

    export interface Err<E> {
        ok: false,
        err: E,
    }
}


export type Result<T, E> = Result.Ok<T> | Result.Err<E>

export function Ok<T>(data: T): Result.Ok<T> {
    return new Impl.Result(data, undefined, true);
}

export function Err<E>(err: E): Result.Err<E> {
    return new Impl.Result(undefined, err, false);
}