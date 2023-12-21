// import {
//     ServerAPI,
//     afterPatch,
//     appDetailsClasses,
//     basicAppDetailsSectionStylerClasses,
//     findInReactTree,
//     wrapReactType
// } from 'decky-frontend-lib';
// import { ReactElement } from 'react';
// import { FaEye } from 'react-icons/fa';


// function patchLibraryApp(serverAPI: ServerAPI) {
//     return serverAPI.routerHook.addPatch(
//         '/library/app/:appid',
//         (props?: { path?: string; children?: ReactElement }) => {
//             if (!props?.children?.props?.renderFunc) {
//                 return props
//             }

//             console.log('initial props with renderfunc', props.children);


//             afterPatch(
//                 props.children.props,
//                 'renderFunc',
//                 (_: Record<string, unknown>[], ret?: ReactElement) => {
//                     if (!ret?.props?.children?.type?.type) {
//                         return ret
//                     }

//                     console.log('props with type', ret.props.children);


//                     wrapReactType(ret.props.children)
//                     afterPatch(
//                         ret.props.children.type,
//                         'type',
//                         (_2: Record<string, unknown>[], ret2?: ReactElement) => {
//                             const container = findInReactTree(
//                                 ret2,
//                                 (x: ReactElement) =>
//                                     Array.isArray(x?.props?.children) &&
//                                     x?.props?.className?.includes(
//                                         appDetailsClasses.InnerContainer
//                                     )
//                             )

//                             for (const item of [basicAppDetailsSectionStylerClasses.PlaySection, basicAppDetailsSectionStylerClasses.ActionRow, basicAppDetailsSectionStylerClasses.AppActionButton, basicAppDetailsSectionStylerClasses.AppButtons
//                             ]) {
//                                 const found = findInReactTree(
//                                     ret2,
//                                     (x: ReactElement) =>
//                                         Array.isArray(x?.props?.children) &&
//                                         x?.props?.className?.includes(
//                                             item
//                                         )
//                                 );

//                                 console.log(item, found);
//                             }


//                             if (typeof container !== 'object') {
//                                 return ret2
//                             }

//                             console.log('InnerContainer props.children', container.props.children);

//                             container.props.children.splice(
//                                 1,
//                                 0,
//                                 <div style={{ backgroundColor: 'red' }} >
//                                     <FaEye />
//                                 </div>
//                             )

//                             wrapReactType(ret2.props.children)

//                             return ret2
//                         }
//                     )
//                     return ret
//                 }
//             )
//             return props
//         }
//     )
// }

// export default patchLibraryApp