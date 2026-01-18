import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach, vi } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { OptimizationTaskConfigView } from './OptimizationTaskConfigView'
import { useAuthStore } from '@/stores/useAuthStore'
import type { UserInfo } from '@/types/generated/api/UserInfo'
import type { OptimizationTaskResponse } from '@/types/generated/api/OptimizationTaskResponse'
import type { UpdateOptimizationTaskConfigRequest } from '@/types/generated/api/UpdateOptimizationTaskConfigRequest'

const API_BASE = 'http://localhost:3000/api/v1'

let task: OptimizationTaskResponse | null = null
let putCallCount = 0
let lastPutBody: UpdateOptimizationTaskConfigRequest | null = null

const server = setupServer(
  http.get(`${API_BASE}/auth/generic-llm/models`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    return HttpResponse.json({ data: { models: ['gpt-4', 'gpt-3.5-turbo'] } })
  }),

  http.get(`${API_BASE}/workspaces/:workspaceId/optimization-tasks/:taskId`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const workspaceId = String(params.workspaceId)
    const taskId = String(params.taskId)

    if (!task || task.workspace_id !== workspaceId || task.id !== taskId) {
      return HttpResponse.json(
        { error: { code: 'OPTIMIZATION_TASK_NOT_FOUND', message: '优化任务不存在' } },
        { status: 404 }
      )
    }

    return HttpResponse.json({ data: task })
  }),

  http.put(
    `${API_BASE}/workspaces/:workspaceId/optimization-tasks/:taskId/config`,
    async ({ request }) => {
      putCallCount += 1
      lastPutBody = null
      const auth = request.headers.get('authorization')
      if (auth !== 'Bearer test-token') {
        return HttpResponse.json(
          { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
          { status: 401 }
        )
      }

      const body = (await request.json()) as UpdateOptimizationTaskConfigRequest
      lastPutBody = body

      const normalizedInitialPrompt =
        body.initial_prompt && body.initial_prompt.trim() ? body.initial_prompt.trim() : null

      const now = 1700000001234
	      if (task) {
	        task = {
	          ...task,
	          config: {
	            ...task.config,
	            schema_version: 1,
	            initial_prompt: normalizedInitialPrompt,
	            max_iterations: body.max_iterations,
	            pass_threshold_percent: body.pass_threshold_percent,
	            candidate_prompt_count: body.candidate_prompt_count,
	            diversity_injection_threshold: body.diversity_injection_threshold,
	            execution_mode: body.execution_mode,
	            max_concurrency: body.max_concurrency,
	            data_split: {
	              train_percent: body.train_percent,
	              validation_percent: body.validation_percent,
	              holdout_percent: 0,
            },
            output_config: body.output_config,
            evaluator_config: body.evaluator_config,
            teacher_llm: body.teacher_llm,
            advanced_data_split: body.advanced_data_split,
          },
          updated_at: now,
        }
      }

      return HttpResponse.json({ data: task })
    }
  )
)

function renderPage(initialEntry: string) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[initialEntry]}>
        <Routes>
          <Route path="/workspaces/:id/tasks/:taskId" element={<OptimizationTaskConfigView />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('OptimizationTaskConfigView', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  beforeEach(() => {
    putCallCount = 0
    lastPutBody = null
    const currentUser: UserInfo = { id: 'u1', username: 'user1' }
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser,
      requiresRegistration: null,
    })

    task = {
      id: 'task-1',
      workspace_id: 'ws-1',
      name: '任务 1',
      description: null,
      goal: 'g',
      execution_target_type: 'dify',
      task_mode: 'fixed',
      status: 'draft',
      test_set_ids: ['ts-1'],
      config: {
        schema_version: 1,
        initial_prompt: null,
        max_iterations: 10,
        pass_threshold_percent: 95,
        candidate_prompt_count: 5,
        diversity_injection_threshold: 3,
        execution_mode: 'serial',
        max_concurrency: 4,
        data_split: { train_percent: 80, validation_percent: 20, holdout_percent: 0 },
        output_config: { strategy: 'single', conflict_alert_threshold: 3, auto_recommend: true },
        evaluator_config: {
          evaluator_type: 'auto',
          exact_match: { case_sensitive: false },
          semantic_similarity: { threshold_percent: 85 },
          constraint_check: { strict: true },
          teacher_model: { llm_judge_samples: 1 },
        },
        teacher_llm: { model_id: null },
        advanced_data_split: { strategy: 'percent', k_fold_folds: 5, sampling_strategy: 'random' },
      },
      final_prompt: null,
      terminated_at: null,
      selected_iteration_id: null,
      created_at: 1700000000000,
      updated_at: 1700000000001,
    }
  })

	  it('应渲染默认值并在 initial_prompt 为 null 时展示留空提示（含高级配置默认值）', async () => {
	    renderPage('/workspaces/ws-1/tasks/task-1')

    expect(await screen.findByText('任务配置：任务 1')).toBeInTheDocument()
    expect(
      await screen.findByText('留空时，系统将在首次迭代中基于优化目标和测试集自动生成初始 Prompt')
    ).toBeInTheDocument()

    expect(screen.getByLabelText('最大迭代轮数')).toHaveValue(10)
    expect(screen.getByLabelText('通过率阈值（%）')).toHaveValue(95)
	    expect(screen.getByLabelText('候选 Prompt 生成数量')).toHaveValue(5)
	    expect(screen.getByLabelText('多样性注入阈值（连续失败次数）')).toHaveValue(3)
	    expect(screen.getByLabelText('执行模式')).toHaveValue('serial')
	    expect(screen.queryByLabelText('并发数（max_concurrency）')).not.toBeInTheDocument()
	    expect(screen.getByLabelText('Train%')).toHaveValue(80)
	    expect(screen.getByLabelText('Validation%')).toHaveValue(20)

    expect(screen.getByLabelText('输出策略')).toHaveValue('single')
    expect(screen.getByLabelText('冲突告警阈值')).toHaveValue(3)
    expect(screen.getByLabelText('自动推荐输出')).toBeChecked()
    expect(screen.getByLabelText('评估器类型')).toHaveValue('auto')
    expect(screen.getByLabelText('高级数据划分策略')).toHaveValue('percent')
    expect(screen.queryByLabelText('交叉验证折数')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('采样策略')).not.toBeInTheDocument()
  })

	  it('保存成功后应提示成功并回显后端归一化配置（空 prompt → null）', async () => {
	    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.change(screen.getByLabelText('候选 Prompt 生成数量'), { target: { value: '4' } })
    fireEvent.change(screen.getByLabelText('多样性注入阈值（连续失败次数）'), { target: { value: '2' } })

    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    await waitFor(() => {
      expect(screen.getByText('保存成功')).toBeInTheDocument()
    })

    expect(
      screen.getByText('留空时，系统将在首次迭代中基于优化目标和测试集自动生成初始 Prompt')
    ).toBeInTheDocument()

	    expect(screen.getByLabelText('候选 Prompt 生成数量')).toHaveValue(4)
	    expect(screen.getByLabelText('多样性注入阈值（连续失败次数）')).toHaveValue(2)
	  })

	  it('选择 example 评估器时应随请求发送 evaluator_type=example 并回显', async () => {
	    renderPage('/workspaces/ws-1/tasks/task-1')

	    await screen.findByText('任务配置：任务 1')

	    fireEvent.change(screen.getByLabelText('评估器类型'), { target: { value: 'example' } })
	    expect(screen.getByLabelText('评估器类型')).toHaveValue('example')

	    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

	    await waitFor(() => {
	      expect(screen.getByText('保存成功')).toBeInTheDocument()
	    })

	    expect(lastPutBody?.evaluator_config.evaluator_type).toBe('example')
	    expect(screen.getByLabelText('评估器类型')).toHaveValue('example')
	  })

	  it('切换到并行模式时应显示并发数输入，并发数越界应本地拦截且不发送请求', async () => {
	    renderPage('/workspaces/ws-1/tasks/task-1')

	    await screen.findByText('任务配置：任务 1')

	    expect(screen.queryByLabelText('并发数（max_concurrency）')).not.toBeInTheDocument()

	    fireEvent.change(screen.getByLabelText('执行模式'), { target: { value: 'parallel' } })

	    const concurrencyInput = await screen.findByLabelText('并发数（max_concurrency）')
	    expect(concurrencyInput).toHaveValue(4)

	    fireEvent.change(concurrencyInput, { target: { value: '0' } })
	    await waitFor(() => {
	      expect(concurrencyInput).toHaveValue(0)
	    })

	    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))
	    expect(await screen.findByText('并发数仅允许 1-64')).toBeInTheDocument()
	    expect(putCallCount).toBe(0)
	  })

	  it('并行模式保存成功时应携带 execution_mode/max_concurrency 并回显', async () => {
	    renderPage('/workspaces/ws-1/tasks/task-1')

	    await screen.findByText('任务配置：任务 1')

	    fireEvent.change(screen.getByLabelText('执行模式'), { target: { value: 'parallel' } })
	    const concurrencyInput = await screen.findByLabelText('并发数（max_concurrency）')
	    fireEvent.change(concurrencyInput, { target: { value: '8' } })
	    await waitFor(() => {
	      expect(concurrencyInput).toHaveValue(8)
	    })

	    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

	    await waitFor(() => {
	      expect(screen.getByText('保存成功')).toBeInTheDocument()
	    })

	    expect(lastPutBody?.execution_mode).toBe('parallel')
	    expect(lastPutBody?.max_concurrency).toBe(8)

	    expect(screen.getByLabelText('执行模式')).toHaveValue('parallel')
	    expect(screen.getByLabelText('并发数（max_concurrency）')).toHaveValue(8)
	  })

  it('切换 evaluator 类型时仅渲染对应字段，保存成功后回显', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.change(screen.getByLabelText('评估器类型'), { target: { value: 'semantic_similarity' } })
    expect(screen.queryByLabelText('大小写敏感')).not.toBeInTheDocument()
    expect(screen.getByLabelText('语义相似度阈值（%）')).toHaveValue(85)

    fireEvent.change(screen.getByLabelText('语义相似度阈值（%）'), { target: { value: '90' } })
    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    await waitFor(() => {
      expect(screen.getByText('保存成功')).toBeInTheDocument()
    })

    expect(screen.getByLabelText('评估器类型')).toHaveValue('semantic_similarity')
    expect(screen.getByLabelText('语义相似度阈值（%）')).toHaveValue(90)
  })

  it('默认应选中“系统默认（不覆盖）”', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    expect(screen.getByLabelText('老师模型')).toHaveValue('')
    expect(screen.getByRole('option', { name: '系统默认（不覆盖）' })).toBeInTheDocument()
  })

  it('模型列表加载后可选择老师模型并在保存 payload 中包含 teacher_llm', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    await waitFor(() => {
      expect(screen.getByRole('option', { name: 'gpt-4' })).toBeInTheDocument()
    })

    fireEvent.change(screen.getByLabelText('老师模型'), { target: { value: 'gpt-4' } })
    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    await waitFor(() => {
      expect(screen.getByText('保存成功')).toBeInTheDocument()
    })

    expect(lastPutBody?.teacher_llm?.model_id).toBe('gpt-4')
    expect(screen.getByLabelText('老师模型')).toHaveValue('gpt-4')
  })

  it('当模型列表为空时应显示引导到 /settings/api', async () => {
    server.use(
      http.get(`${API_BASE}/auth/generic-llm/models`, () => {
        return HttpResponse.json({ data: { models: [] } })
      })
    )

    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    await waitFor(() => {
      expect(screen.getByText(/未检测到通用大模型配置或无法获取模型列表/)).toBeInTheDocument()
    })
    expect(screen.getByRole('link', { name: '/settings/api' })).toBeInTheDocument()
  })

  it('当模型列表加载失败时应只显示 message，并提供 /settings/api 入口', async () => {
    server.use(
      http.get(`${API_BASE}/auth/generic-llm/models`, () => {
        return HttpResponse.json(
          {
            error: {
              code: 'AUTH_UPSTREAM_ERROR',
              message: '上游服务不可用',
              details: { leaked: 'sk-should-not-appear' },
            },
          },
          { status: 502 }
        )
      })
    )

    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    await waitFor(() => {
      expect(screen.getByText(/加载模型列表失败：上游服务不可用/)).toBeInTheDocument()
    })
    expect(screen.queryByText('sk-should-not-appear')).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: '/settings/api' })).toBeInTheDocument()
  })

  it('点击“重置为默认值”确认后应持久化并回显默认高级配置（不影响基础字段）', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true)
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.change(screen.getByLabelText('输出策略'), { target: { value: 'multi' } })
    fireEvent.change(screen.getByLabelText('冲突告警阈值'), { target: { value: '10' } })
    fireEvent.click(screen.getByLabelText('自动推荐输出'))

    fireEvent.change(screen.getByLabelText('评估器类型'), { target: { value: 'teacher_model' } })
    fireEvent.change(screen.getByLabelText('老师模型采样数'), { target: { value: '5' } })

    fireEvent.change(screen.getByLabelText('高级数据划分策略'), { target: { value: 'k_fold' } })
    fireEvent.change(screen.getByLabelText('交叉验证折数'), { target: { value: '10' } })
    fireEvent.change(screen.getByLabelText('采样策略'), { target: { value: 'stratified' } })

    fireEvent.change(screen.getByLabelText('最大迭代轮数'), { target: { value: '20' } })

    fireEvent.click(screen.getByRole('button', { name: '重置为默认值' }))

    await waitFor(() => {
      expect(confirmSpy).toHaveBeenCalledTimes(1)
    })

    await waitFor(() => {
      expect(screen.getByText('保存成功')).toBeInTheDocument()
    })

    expect(screen.getByLabelText('输出策略')).toHaveValue('single')
    expect(screen.getByLabelText('冲突告警阈值')).toHaveValue(3)
    expect(screen.getByLabelText('自动推荐输出')).toBeChecked()
    expect(screen.getByLabelText('高级数据划分策略')).toHaveValue('percent')
    expect(screen.queryByLabelText('交叉验证折数')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('采样策略')).not.toBeInTheDocument()

    expect(screen.getByLabelText('评估器类型')).toHaveValue('auto')

    expect(screen.getByLabelText('最大迭代轮数')).toHaveValue(20)
  })

  it('取消“重置为默认值”确认时不应发送请求', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false)
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.click(screen.getByRole('button', { name: '重置为默认值' }))

    await waitFor(() => {
      expect(confirmSpy).toHaveBeenCalledTimes(1)
    })

    expect(putCallCount).toBe(0)
  })

  it('保存失败时仅展示 message（不展示 details）', async () => {
    server.use(
      http.put(
        `${API_BASE}/workspaces/:workspaceId/optimization-tasks/:taskId/config`,
        async () => {
          return HttpResponse.json(
            { error: { code: 'VALIDATION_ERROR', message: '最大迭代轮数仅允许 1-100' } },
            { status: 400 }
          )
        }
      )
    )

    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')
    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('保存失败：最大迭代轮数仅允许 1-100')).toBeInTheDocument()
  })

  it('候选 Prompt 生成数量为小数时应本地拦截且不发送请求', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    const candidateInput = screen.getByLabelText('候选 Prompt 生成数量')
    fireEvent.change(candidateInput, { target: { value: '4.5' } })
    await waitFor(() => {
      expect(candidateInput).toHaveValue(4.5)
    })
    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('候选 Prompt 生成数量必须为整数')).toBeInTheDocument()
    expect(putCallCount).toBe(0)
  })

  it('多样性注入阈值越界时应本地拦截且不发送请求', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    const thresholdInput = screen.getByLabelText('多样性注入阈值（连续失败次数）')
    fireEvent.change(thresholdInput, { target: { value: '11' } })
    await waitFor(() => {
      expect(thresholdInput).toHaveValue(11)
    })
    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('多样性注入阈值仅允许 1-10')).toBeInTheDocument()
    expect(putCallCount).toBe(0)
  })

  it('冲突告警阈值越界时应本地拦截且不发送请求', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    const conflictInput = screen.getByLabelText('冲突告警阈值')
    fireEvent.change(conflictInput, { target: { value: '0' } })
    await waitFor(() => {
      expect(conflictInput).toHaveValue(0)
    })
    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('冲突告警阈值仅允许 1-10')).toBeInTheDocument()
    expect(putCallCount).toBe(0)
  })

  it('语义相似度阈值越界时应本地拦截且不发送请求', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.change(screen.getByLabelText('评估器类型'), { target: { value: 'semantic_similarity' } })

    const thresholdInput = screen.getByLabelText('语义相似度阈值（%）')
    fireEvent.change(thresholdInput, { target: { value: '101' } })
    await waitFor(() => {
      expect(thresholdInput).toHaveValue(101)
    })

    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('语义相似度阈值（%）仅允许 1-100')).toBeInTheDocument()
    expect(putCallCount).toBe(0)
  })

  it('老师模型采样数为 0 时应本地拦截且不发送请求', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.change(screen.getByLabelText('评估器类型'), { target: { value: 'teacher_model' } })
    const samplesInput = screen.getByLabelText('老师模型采样数')
    fireEvent.change(samplesInput, { target: { value: '0' } })
    await waitFor(() => {
      expect(samplesInput).toHaveValue(0)
    })

    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('老师模型采样数仅允许 1-5')).toBeInTheDocument()
    expect(putCallCount).toBe(0)
  })

  it('交叉验证折数越界时应本地拦截且不发送请求', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.change(screen.getByLabelText('高级数据划分策略'), { target: { value: 'k_fold' } })
    const foldsInput = screen.getByLabelText('交叉验证折数')
    fireEvent.change(foldsInput, { target: { value: '1' } })
    await waitFor(() => {
      expect(foldsInput).toHaveValue(1)
    })

    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('交叉验证折数仅允许 2-10')).toBeInTheDocument()
    expect(putCallCount).toBe(0)
  })
})
