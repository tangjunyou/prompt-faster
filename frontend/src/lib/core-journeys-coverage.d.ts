export type CoreJourneyCoverageRef = {
  file: string
}

export type CoreJourney = {
  id: string
  title: string
  covered_by: CoreJourneyCoverageRef[]
}

export type EvaluatedCoreJourney = CoreJourney & {
  covered: boolean
  existing_files: string[]
  missing_files: string[]
}

export type CoreJourneysYaml = {
  journeys: CoreJourney[]
}

export type CoreJourneysCoverageResult = {
  total: number
  covered: number
  percentage: number
  uncovered: Array<{ id: string; title: string }>
  journeys: EvaluatedCoreJourney[]
}

export function parseCoreJourneysYaml(yamlText: string): CoreJourneysYaml

export function evaluateCoreJourneysCoverage(args: {
  journeys: CoreJourney[]
  repoRoot: string
  fileExists: (absPath: string) => boolean
}): CoreJourneysCoverageResult

