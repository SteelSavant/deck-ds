import {
    ServerAPI,
    afterPatch,
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    findInReactTree,
    wrapReactType
} from 'decky-frontend-lib';
import { ReactElement } from 'react';

// TODO::don't patch if appid doesn't have pipeline
// TODO::patch in real button

function patchLibraryApp(serverAPI: ServerAPI) {
    return serverAPI.routerHook.addPatch(
        '/library/app/:appid',
        (props?: { path?: string; children?: ReactElement }) => {
            console.log("props", props);

            if (!props?.children?.props?.renderFunc) {
                return props;
            }

            console.log("props passed");

            afterPatch(
                props.children.props,
                'renderFunc',
                (_: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret;
                    }

                    wrapReactType(ret.props.children)
                    afterPatch(
                        ret.props.children.type,
                        'type',
                        (_2: Record<string, unknown>[], ret2?: ReactElement) => {
                            const container = findInReactTree(
                                ret2,
                                (x: ReactElement) =>
                                    Array.isArray(x?.props?.children) &&
                                    x?.props?.className?.includes(
                                        appDetailsClasses.InnerContainer
                                    )
                            )


                            if (typeof container !== 'object') {
                                return ret2;
                            }


                            const children = container.props.children;
                            const child = children.find((c: any) => c?.type?.render);

                            console.log('AppDetails child:', child);

                            wrapReactType(child.type);
                            afterPatch(child.type, 'render', (_3: Record<string, unknown>[], ret3?: ReactElement) => {
                                const container = findInReactTree(
                                    ret3,
                                    (x: ReactElement) =>
                                        Array.isArray(x?.props?.children) &&
                                        x?.props?.className?.includes(
                                            basicAppDetailsSectionStylerClasses.AppButtons
                                        )
                                );

                                if (typeof container !== 'object') {
                                    return ret3;
                                }

                                const children = container?.props?.children;

                                console.log('children', children.toString());
                                if (children?.length && children?.length < 3) {
                                    console.log('less than 3 children; adding:', children.toString());
                                    children?.splice(0, 0, <p>HERE!</p>)
                                }

                                console.log('ret3 container', container);


                                return ret3;
                            });

                            console.log('returning 2');


                            return ret2;
                        }
                    )

                    return ret;
                }
            )

            return props;
        }
    )
}

export default patchLibraryApp;