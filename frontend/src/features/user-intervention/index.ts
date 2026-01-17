/**
 * 用户介入功能模块
 * 包含暂停/继续控制、产物编辑等用户干预组件
 */

export { PauseResumeControl } from './PauseResumeControl'
export type { PauseResumeControlProps } from './PauseResumeControl'

export { ArtifactEditor } from './ArtifactEditor'
export type { ArtifactEditorProps } from './ArtifactEditor'

export { GuidanceInput } from './GuidanceInput'
export type { GuidanceInputProps, UserGuidance, GuidanceStatus } from './GuidanceInput'

export {
  HistoryPanel,
  IterationHistoryItem,
  HistoryDetailView,
  useIterationHistory,
  useIterationDetail,
} from './history'
