import { ReactElement } from "react";
import { ApiError } from "../backend";
import { Loading } from "../util/loading";

export default function HandleLoading<T>({ value, onOk, onErr, onLoading }: {
    value: Loading<T>,
    onOk: (value: T) => ReactElement,
    onErr?: (err: ApiError) => ReactElement,
    onLoading?: () => ReactElement,
}): ReactElement {
    const definiteOnLoading = onLoading ?? (() => <div />);
    const definiteOnErr = onErr ?? ((err) => <div>Loading error! Server returned {err.code}: {err.err}</div>)

    console.log("Handling loading for", value, "with ", onOk, definiteOnErr, definiteOnLoading);

    return !value
        ? definiteOnLoading() : value.isOk
            ? onOk(value.data)
            : definiteOnErr(value.err);
}
