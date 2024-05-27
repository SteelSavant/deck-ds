import { ReactElement, Ref } from 'react';

interface PipelineHeaderProps {
    containerRef: Ref<HTMLDivElement>;
    children: ReactElement;
}

const PipelineHeader: React.FC<PipelineHeaderProps> = ({
    children,
    containerRef,
}: PipelineHeaderProps): ReactElement => {
    const margin = '30px';
    return (
        <div
            ref={containerRef}
            style={{
                marginLeft: margin,
                marginRight: margin,
            }}
        >
            {children}
        </div>
    );
};

export default PipelineHeader;
