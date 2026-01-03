import path from 'node:path'

export function parseCoreJourneysYaml(yamlText) {
  const journeys = []

  let currentJourney = null
  let inJourneys = false
  let inCoveredBy = false

  for (const rawLine of yamlText.split(/\r?\n/)) {
    const line = rawLine.replace(/\s+#.*$/, '').trimEnd()
    if (!line.trim()) continue

    if (line.trim() === 'journeys:') {
      inJourneys = true
      currentJourney = null
      inCoveredBy = false
      continue
    }

    if (!inJourneys) continue

    const idMatch = line.match(/^\s*-\s*id:\s*(.+)\s*$/)
    if (idMatch) {
      const id = unquote(idMatch[1].trim())
      currentJourney = { id, title: '', covered_by: [] }
      journeys.push(currentJourney)
      inCoveredBy = false
      continue
    }

    if (!currentJourney) continue

    const titleMatch = line.match(/^\s*title:\s*(.+)\s*$/)
    if (titleMatch) {
      currentJourney.title = unquote(titleMatch[1].trim())
      continue
    }

    if (line.match(/^\s*covered_by:\s*$/)) {
      inCoveredBy = true
      continue
    }

    if (inCoveredBy) {
      const fileMatch = line.match(/^\s*-\s*file:\s*(.+)\s*$/)
      if (fileMatch) {
        currentJourney.covered_by.push({ file: unquote(fileMatch[1].trim()) })
      }
    }
  }

  if (journeys.length === 0) {
    throw new Error('core-journeys.yml 解析失败：未找到 journeys 列表')
  }

  for (const journey of journeys) {
    if (!journey.id) throw new Error('core-journeys.yml 校验失败：存在空的 journey.id')
    if (!Array.isArray(journey.covered_by) || journey.covered_by.length === 0) {
      throw new Error(`core-journeys.yml 校验失败：journey "${journey.id}" 缺少 covered_by`)
    }
    for (const item of journey.covered_by) {
      if (!item?.file) throw new Error(`core-journeys.yml 校验失败：journey "${journey.id}" 存在空的 covered_by.file`)
    }
  }

  return { journeys }
}

export function evaluateCoreJourneysCoverage({ journeys, repoRoot, fileExists }) {
  const evaluatedJourneys = journeys.map((journey) => {
    const missing_files = []
    const existing_files = []

    for (const { file } of journey.covered_by) {
      const abs = resolveRepoPath(repoRoot, file)
      if (fileExists(abs)) existing_files.push(file)
      else missing_files.push(file)
    }

    const covered = existing_files.length > 0
    return { ...journey, covered, existing_files, missing_files }
  })

  const total = evaluatedJourneys.length
  const covered = evaluatedJourneys.filter((j) => j.covered).length
  const percentage = total === 0 ? 0 : covered / total
  const uncovered = evaluatedJourneys.filter((j) => !j.covered).map((j) => ({ id: j.id, title: j.title }))

  return {
    total,
    covered,
    percentage,
    uncovered,
    journeys: evaluatedJourneys,
  }
}

function resolveRepoPath(repoRoot, repoRelativePath) {
  const normalized = repoRelativePath.replace(/^[./]+/, '')
  return path.resolve(repoRoot, normalized)
}

function unquote(value) {
  if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
    return value.slice(1, -1)
  }
  return value
}
