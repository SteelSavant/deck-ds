import { DialogButton, Field, Focusable, Navigation } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { FaEye, FaPlus } from "react-icons/fa";
import { Template, createProfile } from "../../../backend";
import { useServerApi } from "../../../context/serverApiContext";

export default function TemplateMenuItem({ template }: { template: Template }): ReactElement {
    const serverApi = useServerApi();
    const [waiting, setWaiting] = useState(false);

    function previewTemplate(templateId: string) {
        const route = `/deck-ds/settings/templates/${templateId}`;
        console.log("Navigating to", route);
        Navigation.Navigate(route);
    }

    const onCreateTemplate = async () => {
        if (!waiting) {
            setWaiting(true);
            const response = await createProfile({ pipeline: template.pipeline });

            if (response.isOk) {
                const route = `/deck-ds/settings/profiles/${response.data.profile_id}`;
                console.log("Navigating to", route);
                Navigation.Navigate(route);
            } else {
                serverApi.toaster.toast({
                    title: 'Error',
                    body: 'Failed to save profile from template.'
                })
            }

            setWaiting(false);
        }
    };

    return (
        <Field focusable={false} label={template.pipeline.name} description={template.pipeline.description} children={
            <Focusable style={{ display: 'flex', width: '100%', position: 'relative' }}>
                <DialogButton
                    style={{ height: '40px', minWidth: '60px', marginRight: '10px' }}
                    onClick={() => previewTemplate(template.id)}
                    onOKButton={() => previewTemplate(template.id)}
                >
                    <div style={{ display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center' }}>
                        <FaEye style={{ paddingRight: '1rem' }} />
                        Preview
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
                    onClick={onCreateTemplate}
                    onOKButton={onCreateTemplate}
                >
                    <FaPlus />
                </DialogButton>
            </Focusable>
        } />
    );
}