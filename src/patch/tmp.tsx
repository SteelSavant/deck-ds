import React from 'react';

function pe(props: any) {
    const maybeScrollController = (0, C.iE)(),
        [appDetailsTabIsActive, setAppDetailsTabIsActive] = (0, ae.SP)(
            'AppDetailsTabsActive',
            false,
        ),
        componentRef1 = React.useRef(),
        componentRef2 = React.useRef(),
        shouldAutoScrollRef = React.useRef(true),
        o = (0, u.q3)(
            () =>
                B.TS.ON_DECK &&
                0 == oe.rV.storePreferences.provide_deck_feedback,
        ),
        l = (0, u.q3)(() =>
            me.yX.BShouldPromptForDeckCompatibilityFeedback(
                props.overview.appid,
            ),
        ),
        scrollCallback = React.useCallback(() => {
            setAppDetailsTabIsActive(false),
                maybeScrollController?.ScrollToTop();
        }, [maybeScrollController, setAppDetailsTabIsActive]),
        foucusCallback = React.useCallback(() => {
            componentRef2.current!.FocusActionButton();
        }, []),
        checkCallback = React.useCallback(
            (e) => {
                e && setAppDetailsTabIsActive(e);
            },
            [setAppDetailsTabIsActive],
        );
    return (
        React.useEffect(() => {
            const e = shouldAutoScrollRef.current;
            shouldAutoScrollRef.current = false;
            let n = componentRef1.current;
            if (!appDetailsTabIsActive || !maybeScrollController || !n) return;
            const a = function (e) {
                let r =
                    n.getBoundingClientRect().top +
                    maybeScrollController.scrollTop -
                    parseInt(O().headerPadding);
                maybeScrollController.ScrollTo(r, e);
            };
            e ? window.setTimeout(() => a('auto'), 1) : a('smooth');
        }, [maybeScrollController, componentRef1, appDetailsTabIsActive]),
        React.createElement(
            _.Z,
            {
                className: O().AppDetailsRoot,
            },
            React.createElement(be, {
                ...props,
                onNav: scrollCallback,
                ref: componentRef2,
            }),
            React.createElement(Q.sD, {
                ...props,
                onFocus: scrollCallback,
            }),
            React.createElement(
                _.Z,
                {
                    onFocusWithin: scrollCallback,
                },
                o && React.createElement(ge, null),
                !o &&
                    l &&
                    React.createElement(he, {
                        ...props,
                    }),
            ),
            React.createElement(
                _.Z,
                {
                    ref: componentRef1,
                    className: O().AppDetailsContainer,
                    onFocusWithin: checkCallback,
                },
                React.createElement(_e, {
                    fnOnCancelFromTabHeader: foucusCallback,
                    details: props.details,
                    overview: props.overview,
                    setSections: props.setSections,
                    bSuppressTransition: props.bSuppressTransition,
                    parentComponent: props.parentComponent,
                }),
            ),
        )
    );
}
