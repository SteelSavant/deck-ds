
```mermaid
erDiagram
  PipelineDefinition {
    string name
    string description

  }

  PipelineActionDefinition {
    string name
    string description
    Selection selection
  }

  OneOfSetting {
    id selection
  }

  ActionSetting {
    Action action
  }

  EnabledSetting {
    bool isEnabled
  }

  OverrideSetting {
    id profile
  }

  PipelineDefinition }|--|{ PipelineActionDefinition : uses
  PipelineDefinition ||--|{ OneOfSetting : contains
  PipelineDefinition ||--|{ ActionSetting : contains
  PipelineDefinition ||--|{ EnabledSetting : contains
  PipelineDefinition ||--|{ OverrideSetting : contains
```