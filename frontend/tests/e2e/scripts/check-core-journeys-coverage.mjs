import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { evaluateCoreJourneysCoverage, parseCoreJourneysYaml } from '../../../src/lib/core-journeys-coverage.js'

const threshold = 0.8

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../')
const inputPath = path.join(repoRoot, 'frontend/tests/e2e/core-journeys.yml')
const outputPath = path.join(repoRoot, 'frontend/test-results/core-journeys-coverage.json')

const yamlText = fs.readFileSync(inputPath, 'utf-8')
const { journeys } = parseCoreJourneysYaml(yamlText)

const result = evaluateCoreJourneysCoverage({
  journeys,
  repoRoot,
  fileExists: (absPath) => fs.existsSync(absPath),
})

const payload = {
  generated_at: new Date().toISOString(),
  threshold,
  ...result,
}

fs.mkdirSync(path.dirname(outputPath), { recursive: true })
fs.writeFileSync(outputPath, JSON.stringify(payload, null, 2) + '\n', 'utf-8')

const percentText = `${Math.round(payload.percentage * 100)}%`
const summaryLines = [
  '## Core Journeys E2E Coverage',
  '',
  `- Covered: ${payload.covered}/${payload.total} (${percentText})`,
  `- Threshold: ${Math.round(threshold * 100)}%`,
]

if (payload.uncovered.length > 0) {
  summaryLines.push('', '### Uncovered journeys')
  for (const j of payload.uncovered) summaryLines.push(`- ${j.id}: ${j.title}`)
} else {
  summaryLines.push('', '### Uncovered journeys', '- (none)')
}

const summary = summaryLines.join('\n') + '\n'
process.stdout.write(summary)

if (process.env.GITHUB_STEP_SUMMARY) {
  fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary, 'utf-8')
}

if (payload.total === 0) {
  console.error('core-journeys.yml journeys 为空，无法计算覆盖率')
  process.exit(1)
}

if (payload.percentage < threshold) {
  console.error(`核心用户旅程 E2E 覆盖率未达标：${percentText} < ${Math.round(threshold * 100)}%`)
  process.exit(1)
}
