// Lightly reverse-engineered source for relevant widgets patching the library app

// import React from 'react';

// const be = React.forwardRef((function(e, t) {
//             const {onNav: onNav, ...n} = e;
//             return A.createElement(_.Z, {
//                 className: O().PlaySection,
//                 onFocusWithin: e => e && onNav()
//             }, A.createElement(ye, {
//                 ...n,
//                 ref: t
//             }), A.createElement(Q.BA, {
//                 ...n
//             }))

// function pe(props: any) {
//     const maybeScrollController = (0, C.iE)(),
//         [appDetailsTabIsActive, setAppDetailsTabIsActive] = (0, ae.SP)(
//             'AppDetailsTabsActive',
//             false,
//         ),
//         componentRef1 = React.useRef(),
//         componentRef2 = React.useRef(),
//         shouldAutoScrollRef = React.useRef(true),
//         o = (0, u.q3)(
//             () =>
//                 B.TS.ON_DECK &&
//                 false == oe.rV.storePreferences.provide_deck_feedback,
//         ),
//         l = (0, u.q3)(() =>
//             me.yX.BShouldPromptForDeckCompatibilityFeedback(
//                 props.overview.appid,
//             ),
//         ),
//         scrollCallback = React.useCallback(() => {
//             setAppDetailsTabIsActive(false),
//                 maybeScrollController?.ScrollToTop();
//         }, [maybeScrollController, setAppDetailsTabIsActive]),
//         focusCallback = React.useCallback(() => {
//             componentRef2.current!.FocusActionButton();
//         }, []),
//         setAppDetailsActiveCallback = React.useCallback(
//             (e) => {
//                 e && setAppDetailsTabIsActive(e);
//             },
//             [setAppDetailsTabIsActive],
//         );
//     return (
//         React.useEffect(() => {
//             const e = shouldAutoScrollRef.current;
//             shouldAutoScrollRef.current = false;
//             let n = componentRef1.current;
//             if (!appDetailsTabIsActive || !maybeScrollController || !n) return;
//             const a = function (e) {
//                 let r =
//                     n.getBoundingClientRect().top +
//                     maybeScrollController.scrollTop -
//                     parseInt(O().headerPadding);
//                 maybeScrollController.ScrollTo(r, e);
//             };
//             e ? window.setTimeout(() => a('auto'), 1) : a('smooth');
//         }, [maybeScrollController, componentRef1, appDetailsTabIsActive]),
//         React.createElement(
//             _.Z,
//             {
//                 className: O().AppDetailsRoot,
//             },
//             React.createElement(be, {
//                 // This is the element we're trying to traverse
//                 ...props,
//                 onNav: scrollCallback,
//                 ref: componentRef2,
//             }),
//             React.createElement(Q.sD, {
//                 ...props,
//                 onFocus: scrollCallback,
//             }),
//             React.createElement(
//                 _.Z,
//                 {
//                     onFocusWithin: scrollCallback,
//                 },
//                 o && React.createElement(ge, null),
//                 !o &&
//                     l &&
//                     React.createElement(he, {
//                         ...props,
//                     }),
//             ),
//             React.createElement(
//                 _.Z,
//                 {
//                     ref: componentRef1,
//                     className: O().AppDetailsContainer,
//                     onFocusWithin: setAppDetailsActiveCallback,
//                 },
//                 React.createElement(_e, {
//                     fnOnCancelFromTabHeader: focusCallback,
//                     details: props.details,
//                     overview: props.overview,
//                     setSections: props.setSections,
//                     bSuppressTransition: props.bSuppressTransition,
//                     parentComponent: props.parentComponent,
//                 }),
//             ),
//         )
//     );
// }
