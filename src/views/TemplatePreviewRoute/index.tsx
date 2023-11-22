import { Focusable, Tabs, useParams } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import HandleLoading from "../../components/HandleLoading";
import useTemplate from "../../hooks/useTemplate";
import TemplateInfo from "./TemplateInfo";


export default function TemplatePreviewRoute(): ReactElement {
    const [currentTabRoute, setCurrentTabRoute] = useState<string>("info")
    const { templateid } = useParams<{ templateid: string }>()

    const template = useTemplate(templateid);

    return <HandleLoading
        value={template}
        onOk={
            (template) => {
                if (template === undefined) {
                    return <div> Template {templateid} does not exist!</div>;
                } else {
                    const targets = template.targets;

                    console.log("Targets: ", targets);

                    return <div style={{
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'center',
                    }}>
                        {template.name}
                        <Focusable style={{ minWidth: "100%", minHeight: "100%" }}>
                            <div
                                style={{
                                    marginTop: "40px",
                                    height: "calc(100% - 40px)",
                                }}>
                                <Tabs
                                    activeTab={currentTabRoute}
                                    onShowTab={(tabID: string) => {
                                        setCurrentTabRoute(tabID);
                                    }}
                                    tabs={[
                                        {
                                            title: "Info",
                                            content: <TemplateInfo template={template} />,
                                            id: "info",
                                        },
                                    ]}
                                />
                            </div>
                        </Focusable>
                    </div>
                }
            }
        }
    />;
}