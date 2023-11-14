
class Ok<T> {
    ok: true;
    data: T;

    constructor(data: T) {
        this.ok = true;
        this.data = data;
    }
}

class Err<E> {
    ok: false;
    err: E;

    constructor(err: E) {
        this.ok = false;
        this.err = err;
    }
}

type Result<T, E> = Ok<T> | Err<E>