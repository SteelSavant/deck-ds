import { ReactElement } from 'react';
import { ApiError } from '../backend';
import { Loading } from '../util/loading';

export default function HandleLoading<T>({
    value,
    onOk,
    onErr,
    onLoading,
}: {
    value: Loading<T>;
    onOk: (value: T) => ReactElement;
    onErr?: (err: ApiError) => ReactElement;
    onLoading?: () => ReactElement;
}): ReactElement | null {
    const definiteOnLoading = onLoading ?? (() => null);
    const definiteOnErr =
        onErr ??
        ((err) => (
            <div>
                Loading error! Server returned {err.code}: {err.err}
            </div>
        ));

    return !value
        ? definiteOnLoading()
        : value.isOk
        ? onOk(value.data)
        : definiteOnErr(value.err);
}
