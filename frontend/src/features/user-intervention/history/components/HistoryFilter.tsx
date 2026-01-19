import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

export interface HistoryFilterValue {
  eventTypes: string[]
  actor: '' | 'system' | 'user'
  iterationMin: string
  iterationMax: string
  timeStart: string
  timeEnd: string
}

interface HistoryFilterProps {
  value: HistoryFilterValue
  onChange: (value: HistoryFilterValue) => void
  disableEventFilters?: boolean
  disableActorFilter?: boolean
}

const EVENT_TYPES = [
  { value: 'iteration_started', label: '迭代开始' },
  { value: 'iteration_completed', label: '迭代完成' },
  { value: 'evaluation_completed', label: '评估完成' },
  { value: 'user_pause', label: '用户暂停' },
  { value: 'user_resume', label: '用户恢复' },
  { value: 'user_edit', label: '用户编辑' },
  { value: 'user_guidance', label: '对话引导' },
  { value: 'rollback', label: '回滚' },
  { value: 'checkpoint_saved', label: 'Checkpoint 保存' },
  { value: 'error_occurred', label: '异常' },
  { value: 'config_changed', label: '配置变更' },
  { value: 'task_terminated', label: '任务终止' },
  { value: 'checkpoint_recovered', label: '恢复' },
]

export function HistoryFilter({
  value,
  onChange,
  disableEventFilters = false,
  disableActorFilter = false,
}: HistoryFilterProps) {
  const toggleEventType = (eventType: string) => {
    const exists = value.eventTypes.includes(eventType)
    const next = exists
      ? value.eventTypes.filter((item) => item !== eventType)
      : [...value.eventTypes, eventType]
    onChange({ ...value, eventTypes: next })
  }

  const handleReset = () => {
    onChange({
      eventTypes: [],
      actor: '',
      iterationMin: '',
      iterationMax: '',
      timeStart: '',
      timeEnd: '',
    })
  }

  return (
    <div className="rounded-md border p-4 bg-background">
      <div className="flex items-center justify-between gap-2">
        <h4 className="text-sm font-semibold">筛选条件</h4>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleReset}
          className="min-w-[44px] min-h-[32px]"
        >
          清空
        </Button>
      </div>

      <div className="mt-3">
        <Label className="text-xs text-muted-foreground">操作类型</Label>
        <div className="mt-2 flex flex-wrap gap-2">
          {EVENT_TYPES.map((item) => {
            const active = value.eventTypes.includes(item.value)
            return (
              <Button
                key={item.value}
                variant={active ? 'default' : 'outline'}
                size="sm"
                onClick={() => toggleEventType(item.value)}
                disabled={disableEventFilters}
                className="min-h-[32px]"
              >
                {item.label}
              </Button>
            )
          })}
        </div>
      </div>

      <div className="mt-4">
        <Label className="text-xs text-muted-foreground">操作者</Label>
        <div className="mt-2 flex gap-2">
          <Button
            variant={value.actor === 'system' ? 'default' : 'outline'}
            size="sm"
            onClick={() =>
              onChange({
                ...value,
                actor: value.actor === 'system' ? '' : 'system',
              })
            }
            disabled={disableActorFilter}
            className="min-h-[32px]"
          >
            系统
          </Button>
          <Button
            variant={value.actor === 'user' ? 'default' : 'outline'}
            size="sm"
            onClick={() =>
              onChange({
                ...value,
                actor: value.actor === 'user' ? '' : 'user',
              })
            }
            disabled={disableActorFilter}
            className="min-h-[32px]"
          >
            用户
          </Button>
        </div>
      </div>

      <div className="mt-4 grid gap-3 md:grid-cols-2">
        <div>
          <Label className="text-xs text-muted-foreground">迭代轮次范围</Label>
          <div className="mt-2 flex items-center gap-2">
            <Input
              type="number"
              value={value.iterationMin}
              onChange={(event) =>
                onChange({ ...value, iterationMin: event.target.value })
              }
              placeholder="最小"
              className="h-9"
            />
            <span className="text-xs text-muted-foreground">-</span>
            <Input
              type="number"
              value={value.iterationMax}
              onChange={(event) =>
                onChange({ ...value, iterationMax: event.target.value })
              }
              placeholder="最大"
              className="h-9"
            />
          </div>
        </div>
        <div>
          <Label className="text-xs text-muted-foreground">时间范围</Label>
          <div className="mt-2 grid gap-2">
            <Input
              type="datetime-local"
              value={value.timeStart}
              onChange={(event) =>
                onChange({ ...value, timeStart: event.target.value })
              }
              className="h-9"
            />
            <Input
              type="datetime-local"
              value={value.timeEnd}
              onChange={(event) =>
                onChange({ ...value, timeEnd: event.target.value })
              }
              className="h-9"
            />
          </div>
        </div>
      </div>
    </div>
  )
}
