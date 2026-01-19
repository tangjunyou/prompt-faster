import type { PassRateSummary } from '@/types/generated/models/PassRateSummary'

export function formatCheckpointTime(isoString: string): string {
  try {
    const date = new Date(isoString)
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return isoString
  }
}

export function formatPassRateSummary(
  summary?: PassRateSummary | null,
): string {
  if (!summary) {
    return '通过率未知'
  }
  const percent = Math.round(summary.passRate * 1000) / 10
  return `通过率 ${percent}% (${summary.passedCases}/${summary.totalCases})`
}
