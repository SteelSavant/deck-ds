module Impl {
    export class ResultImpl<T, E, O extends boolean> {
        readonly isOk: O;
        readonly data: T;
        readonly err: E;

        constructor(data: T, err: E, ok: O) {
            this.isOk = ok;
            this.data = data;
            this.err = err;
        }

        public map<R>(fn: (data: T) => R): Result<R, E> {
            if (this.isOk) {
                return Ok(fn(this.data));
            } else {
                return Err(this.err);
            }
        }

        public and_then<R>(fn: (res: T) => Result<R, E>): Result<R, E> {
            if (this.isOk) {
                return fn(this.data);
            } else {
                return Err(this.err)
            }
        }
    }
}

export module Result {
    export interface Ok<T, E> {
        isOk: true,
        data: T,
        map<R>(fn: (data: T) => R): Result<R, E>,
        and_then<R>(fn: (res: T) => Result<R, E>): Result<R, E>
    }

    export interface Err<T, E> {
        isOk: false,
        err: E,
        map<R>(fn: (data: T) => R): Result<R, E>,
        and_then<R>(fn: (res: T) => Result<R, E>): Result<R, E>
    }
}


export type Result<T, E> = Result.Ok<T, E> | Result.Err<T, E>

export function Ok<T, E>(data: T): Result.Ok<T, E> {
    return new Impl.ResultImpl(data, undefined as E, true);
}

export function Err<T, E>(err: E): Result.Err<T, E> {
    return new Impl.ResultImpl(undefined as T, err, false);
}