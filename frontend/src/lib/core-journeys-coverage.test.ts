/* @vitest-environment node */
import { describe, expect, it } from 'vitest'
import { evaluateCoreJourneysCoverage, parseCoreJourneysYaml } from './core-journeys-coverage.js'

describe('core-journeys-coverage', () => {
  it('parses core-journeys.yml minimal format', () => {
    const yaml = `
journeys:
  - id: auth
    title: 用户登录/认证
    covered_by:
      - file: frontend/tests/e2e/auth.spec.ts
`
    const parsed = parseCoreJourneysYaml(yaml)
    expect(parsed.journeys).toHaveLength(1)
    expect(parsed.journeys[0]?.id).toBe('auth')
    expect(parsed.journeys[0]?.covered_by[0]?.file).toBe('frontend/tests/e2e/auth.spec.ts')
  })

  it('computes covered/total by journey count', () => {
    const yaml = `
journeys:
  - id: a
    title: A
    covered_by:
      - file: frontend/tests/e2e/a.spec.ts
  - id: b
    title: B
    covered_by:
      - file: frontend/tests/e2e/b.spec.ts
`
    const { journeys } = parseCoreJourneysYaml(yaml)
    const result = evaluateCoreJourneysCoverage({
      journeys,
      repoRoot: '/repo',
      fileExists: (absPath: string) => absPath.endsWith('/frontend/tests/e2e/a.spec.ts'),
    })
    expect(result.total).toBe(2)
    expect(result.covered).toBe(1)
    expect(result.percentage).toBeCloseTo(0.5)
    expect(result.uncovered).toEqual([{ id: 'b', title: 'B' }])
  })
})
