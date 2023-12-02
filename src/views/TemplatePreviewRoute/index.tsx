import { Focusable, Tabs, useParams } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { ActionSelection } from "../../backend";
import HandleLoading from "../../components/HandleLoading";
import useTemplate from "../../hooks/useTemplate";
import Pipeline from "./Pipeline";
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
                    interface KeyValue {
                        target: string,
                        root: ActionSelection,
                    }

                    const defaultTargets: KeyValue[] = [];
                    const extraTargets: KeyValue[] = [] // no real intention of actually supporting extra targets, but...

                    for (const key in template.pipeline.targets) {
                        const value: KeyValue = {
                            target: key,
                            root: template.pipeline.targets[key],
                        };

                        if (key === 'Gamemode') {
                            defaultTargets.push(value);
                        } else if (key === 'Desktop') {
                            defaultTargets.splice(0, 0, value);
                        } else {
                            extraTargets.push(value)
                        }
                    }


                    const allTargets = defaultTargets.concat(extraTargets);

                    const tabs = [
                        {
                            title: "Info",
                            content: <TemplateInfo template={template} />,
                            id: "info",
                        },
                        ...allTargets.map((kv) => {
                            return {
                                title: kv.target,
                                content: <Pipeline root={kv.root} />,
                                id: kv.target.toLowerCase(),
                            };
                        }),
                    ];

                    console.log(`Creating ${template.pipeline.name} pipeline tags:`, tabs);

                    return <Focusable style={{ minWidth: "100%", minHeight: "100%" }}>
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
                                tabs={tabs}
                            />
                        </div>
                    </Focusable>
                }
            }
        }
    />;
}