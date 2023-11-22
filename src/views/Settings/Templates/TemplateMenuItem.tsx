import { DialogButton, Field, Focusable, Navigation } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaEye, FaPlus } from "react-icons/fa";
import { PipelineDefinition } from "../../../backend";

export default function TemplateMenuItem({ template }: { template: PipelineDefinition }): ReactElement {

    function previewTemplate(templateId: string) {
        const route = `/deck-ds/settings/templates/${templateId}`;
        console.log("Navigating to", route);
        Navigation.Navigate(route);
    }

    return (
        <Field focusable={false} label={template.name} description={template.description} children={
            <Focusable style={{ display: 'flex', width: '100%', position: 'relative' }}>
                <DialogButton
                    style={{ height: '40px', minWidth: '60px', marginRight: '10px' }}
                    onClick={() => previewTemplate(template.id)}
                    onOKButton={() => previewTemplate(template.id)}
                >

                    <div style={{ display: 'flex', minWidth: '180px', justifyContent: 'space-between', alignItems: 'center' }}>
                        Preview
                        <FaEye style={{ paddingLeft: '1rem' }} />
                    </div>
                </DialogButton>
                <DialogButton
                    style={{
                        height: '40px',
                        width: '40px',
                        padding: '10px 12px',
                        minWidth: '40px',
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'center',
                    }}
                    onClick={() => console.log("pressed", template.name)}
                    onOKButton={() => console.log("ok", template.name)}
                >
                    <FaPlus />
                </DialogButton>
            </Focusable>
        } />
    );
}