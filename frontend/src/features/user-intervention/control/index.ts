/**
 * 迭代控制模块导出
 */

export { AddRoundsDialog } from './AddRoundsDialog'
export type { AddRoundsDialogProps } from './AddRoundsDialog'

export { TerminateDialog } from './TerminateDialog'
export type { TerminateDialogProps } from './TerminateDialog'

export { CandidatePromptList } from './CandidatePromptList'
export type { CandidatePromptListProps } from './CandidatePromptList'

export { IterationControlPanel } from './IterationControlPanel'
export type { IterationControlPanelProps } from './IterationControlPanel'

export {
  useCandidates,
  useAddRounds,
  useTerminateTask,
  candidatesQueryKey,
  taskConfigQueryKey,
} from './hooks/useIterationControl'

export type {
  AddRoundsRequest,
  AddRoundsResponse,
  CandidatePromptSummary,
  CandidatePromptListResponse,
  TerminateTaskRequest,
  TerminateTaskResponse,
} from './services/iterationControlService'
