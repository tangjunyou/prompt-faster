# CORE Workflows

## Available Workflows in core

**brainstorming-session**
- Path: `_bmad/core/workflows/brainstorming/workflow.md`
- Facilitate interactive brainstorming sessions using diverse creative techniques and ideation methods

**party-mode**
- Path: `_bmad/core/workflows/party-mode/workflow.md`
- Orchestrates group discussions between all installed BMAD agents, enabling natural multi-agent conversations

**brainstorming**
- Path: `_bmad/core/workflows/brainstorming/workflow.md`
- Facilitate interactive brainstorming sessions using diverse creative techniques and ideation methods


## Execution

When running a workflow, the execution method depends on the file type:

For `workflow.yaml`:
1. LOAD {project-root}/_bmad/core/tasks/workflow.xml
2. Pass the workflow path as 'workflow-config' parameter
3. Follow workflow.xml instructions EXACTLY
4. Save outputs after EACH section

For `workflow.md`:
1. LOAD the FULL `workflow.md`
2. READ its entire contents and follow its directions exactly
3. DO NOT run workflow.xml for markdown-based workflows

## Modes
- Normal: Full interaction
- #yolo: Skip optional steps
