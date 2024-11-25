namespace Impl {
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

        public mapErr<R>(fn: (err: E) => R): Result<T, R> {
            if (this.isOk) {
                return Ok(this.data);
            } else {
                return Err(fn(this.err));
            }
        }

        public andThen<R>(fn: (res: T) => Result<R, E>): Result<R, E> {
            if (this.isOk) {
                return fn(this.data);
            } else {
                return Err(this.err);
            }
        }

        public andThenAsync<R>(
            fn: (res: T) => Promise<Result<R, E>>,
        ): Promise<Result<R, E>> {
            if (this.isOk) {
                return fn(this.data);
            } else {
                return new Promise((resolve) => resolve(Err(this.err)));
            }
        }

        public expect(msg: string): T {
            if (!this.isOk) {
                throw new Error(`${msg}: ${this.err}`);
            }

            return this.data;
        }

        public unwrap(): T {
            if (!this.isOk) {
                throw new Error(`${this.err}`);
            }

            return this.data;
        }
    }
}

export namespace Result {
    export interface Ok<T, E> {
        isOk: true;
        data: T;
        map<R>(fn: (data: T) => R): Result<R, E>;
        mapErr<R>(fn: (data: E) => R): Result<T, R>;
        andThen<R>(fn: (res: T) => Result<R, E>): Result<R, E>;
        andThenAsync<R>(
            fn: (res: T) => Promise<Result<R, E>>,
        ): Promise<Result<R, E>>;
        expect(msg: string): T;
        unwrap(): T;
    }

    export interface Err<T, E> {
        isOk: false;
        err: E;
        map<R>(fn: (data: T) => R): Result<R, E>;
        mapErr<R>(fn: (data: E) => R): Result<T, R>;
        andThen<R>(fn: (res: T) => Result<R, E>): Result<R, E>;
        andThenAsync<R>(
            fn: (res: T) => Promise<Result<R, E>>,
        ): Promise<Result<R, E>>;
        expect(msg: string): T;
        unwrap(): T;
    }
}

export type Result<T, E> = Result.Ok<T, E> | Result.Err<T, E>;

export function Ok<T, E>(data: T): Result.Ok<T, E> {
    return new Impl.ResultImpl(data, undefined as E, true);
}

export function Err<T, E>(err: E): Result.Err<T, E> {
    return new Impl.ResultImpl(undefined as T, err, false);
}
