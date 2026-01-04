import type { DataSplit } from '@/types/generated/models/DataSplit'
import type { TestCase } from '@/types/generated/models/TestCase'

export type JsonlParseError = { line: number; message: string }

export type ParseStats = {
  processedLines: number
  totalLines: number
  ok: number
  failed: number
}

export type ParseOptions = {
  maxErrors?: number
  progressEvery?: number
  yieldEvery?: number
  onProgress?: (stats: ParseStats) => void
  shouldYield?: () => Promise<void>
}

export type ParseResult = {
  cases: TestCase[]
  errors: JsonlParseError[]
  stats: ParseStats
  truncatedErrors: boolean
}

const ALLOWED_REFERENCE_KEYS = ['Exact', 'Constrained', 'Hybrid'] as const
const ALLOWED_SPLITS: DataSplit[] = ['unassigned', 'train', 'validation', 'holdout']

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function validateConstraintItem(item: unknown, index: number, path: string): string | null {
  if (!isPlainObject(item)) return `${path}[${index}] 必须是对象`
  if (typeof item.name !== 'string') return `${path}[${index}].name 必须是字符串`
  if (typeof item.description !== 'string') return `${path}[${index}].description 必须是字符串`
  if (!('weight' in item)) return `${path}[${index}].weight 必须存在（number|null）`
  if (item.weight !== null && typeof item.weight !== 'number') return `${path}[${index}].weight 必须是 number|null`
  return null
}

function validateQualityDimensionItem(item: unknown, index: number, path: string): string | null {
  if (!isPlainObject(item)) return `${path}[${index}] 必须是对象`
  if (typeof item.name !== 'string') return `${path}[${index}].name 必须是字符串`
  if (typeof item.description !== 'string') return `${path}[${index}].description 必须是字符串`
  if (typeof item.weight !== 'number') return `${path}[${index}].weight 必须是数字`
  return null
}

function validateStringRecord(record: unknown, path: string): string | null {
  if (!isPlainObject(record)) return `${path} 必须是对象`
  for (const [key, value] of Object.entries(record)) {
    if (value !== undefined && typeof value !== 'string') return `${path}.${key} 必须是字符串`
  }
  return null
}

function validateReference(reference: unknown): string | null {
  if (!isPlainObject(reference)) return 'reference 必须是对象'

  const keys = Object.keys(reference)
  if (keys.length !== 1) return 'reference 必须是单 key 的变体对象'

  const variant = keys[0]
  if (!ALLOWED_REFERENCE_KEYS.includes(variant as (typeof ALLOWED_REFERENCE_KEYS)[number])) {
    return 'reference 必须是 Exact / Constrained / Hybrid 之一'
  }

  const payload = reference[variant]
  if (!isPlainObject(payload)) return `reference.${variant} 必须是对象`

  if (variant === 'Exact') {
    if (typeof payload.expected !== 'string') return 'reference.Exact.expected 必须是字符串'
    return null
  }

  if (variant === 'Constrained') {
    if (!Array.isArray(payload.constraints)) return 'reference.Constrained.constraints 必须是数组'
    if (!Array.isArray(payload.quality_dimensions)) {
      return 'reference.Constrained.quality_dimensions 必须是数组'
    }
    for (const [index, item] of payload.constraints.entries()) {
      const err = validateConstraintItem(item, index, 'reference.Constrained.constraints')
      if (err) return err
    }
    for (const [index, item] of payload.quality_dimensions.entries()) {
      const err = validateQualityDimensionItem(item, index, 'reference.Constrained.quality_dimensions')
      if (err) return err
    }
    return null
  }

  if (variant === 'Hybrid') {
    const exactPartsErr = validateStringRecord(payload.exact_parts, 'reference.Hybrid.exact_parts')
    if (exactPartsErr) return exactPartsErr
    if (!Array.isArray(payload.constraints)) return 'reference.Hybrid.constraints 必须是数组'
    for (const [index, item] of payload.constraints.entries()) {
      const err = validateConstraintItem(item, index, 'reference.Hybrid.constraints')
      if (err) return err
    }
    return null
  }

  return 'reference 不支持的变体'
}

export async function parseTestCasesJsonl(text: string, options: ParseOptions = {}): Promise<ParseResult> {
  const maxErrors = options.maxErrors ?? 50
  const progressEvery = options.progressEvery ?? 100
  const yieldEvery = options.yieldEvery ?? 100
  const onProgress = options.onProgress
  const shouldYield =
    options.shouldYield ??
    (async () => {
      await new Promise<void>((resolve) => setTimeout(resolve, 0))
    })

  const lines = text.split(/\r?\n/)
  const totalLines = lines.length
  const errors: JsonlParseError[] = []
  const cases: TestCase[] = []
  const seenIds = new Set<string>()
  let ok = 0
  let failed = 0
  let truncatedErrors = false

  const reportProgress = (processedLines: number) => {
    if (!onProgress) return
    onProgress({ processedLines, totalLines, ok, failed })
  }

  const maybeProgressAndYield = async (processedLines: number) => {
    if (progressEvery > 0 && processedLines % progressEvery === 0) reportProgress(processedLines)
    if (yieldEvery > 0 && processedLines % yieldEvery === 0) await shouldYield()
  }

  const addError = async (line: number, message: string) => {
    failed += 1
    if (errors.length < maxErrors) {
      errors.push({ line, message })
    } else {
      truncatedErrors = true
    }
    await maybeProgressAndYield(line)
  }

  for (let i = 0; i < totalLines; i += 1) {
    const lineNumber = i + 1
    const raw = lines[i] ?? ''
    const trimmed = raw.trim()

    if (trimmed.length === 0) {
      await maybeProgressAndYield(lineNumber)
      continue
    }

    let parsed: unknown
    try {
      parsed = JSON.parse(trimmed)
    } catch {
      await addError(lineNumber, 'JSON 无法解析')
      continue
    }

    if (!isPlainObject(parsed)) {
      await addError(lineNumber, '每行必须是 JSON 对象')
      continue
    }

    const idRaw = parsed.id
    if (typeof idRaw !== 'string' || idRaw.trim() === '') {
      await addError(lineNumber, '缺少必填字段：id（非空字符串）')
      continue
    }

    const id = idRaw.trim()
    if (seenIds.has(id)) {
      await addError(lineNumber, `id 重复：${id}`)
      continue
    }

    const input = parsed.input
    if (!isPlainObject(input)) {
      await addError(lineNumber, '缺少必填字段：input（对象）')
      continue
    }

    const referenceError = validateReference(parsed.reference)
    if (referenceError) {
      await addError(lineNumber, referenceError)
      continue
    }

    const split = parsed.split
    if (split !== undefined && split !== null) {
      if (typeof split !== 'string' || !ALLOWED_SPLITS.includes(split as DataSplit)) {
        await addError(lineNumber, 'split 只允许 unassigned/train/validation/holdout')
        continue
      }
    }

    const metadata = parsed.metadata
    if (metadata !== undefined && metadata !== null && !isPlainObject(metadata)) {
      await addError(lineNumber, 'metadata 必须是对象')
      continue
    }

    cases.push({
      id,
      input: input as TestCase['input'],
      reference: parsed.reference as TestCase['reference'],
      split: (split === undefined ? null : (split as TestCase['split'])),
      metadata: (metadata === undefined ? null : (metadata as TestCase['metadata'])),
    })
    seenIds.add(id)
    ok += 1

    await maybeProgressAndYield(lineNumber)
  }

  reportProgress(totalLines)

  return {
    cases,
    errors,
    stats: { processedLines: totalLines, totalLines, ok, failed },
    truncatedErrors,
  }
}
