# Story 8.5: Prompt 版本对比（Growth）

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 8-5-prompt-version-comparison-growth

## Epic 8 概述

> **Epic 8: 结果输出与元优化** - 用户成果：用户可以查看、导出优化结果，查看诊断报告，并使用元优化功能优化老师模型 Prompt。

**Epic 8 Story 列表**：
- 8.1 结果查看与导出（FR60, FR61，NFR18）- ✅ done
- 8.2 诊断报告（FR63）- ✅ done
- 8.3 元优化基础（FR56, FR57, FR58）- ✅ done
- 8.4 高级用户直接编辑老师模型 Prompt（FR59）- ✅ done
- **8.5 Prompt 版本对比（本 Story，FR62）** - Growth
- 8.6 创意任务多样性检测（FR34）- Growth

## Key Decisions (Growth)

- **对比策略**：使用同一测试集对两个版本分别执行预览，合并结果进行对比。
- **执行复用**：复用 Story 8.4 的 `preview_prompt` 执行逻辑，串行执行两个版本。
- **Diff 视图**：使用 Monaco DiffEditor（架构 UX 规范明确指定）展示 Prompt 文本差异。
- **测试用例限制**：最多 10 条（对比需执行两次，耗时更长），超限时按 **task_ids → test_set_ids → cases 顺序**确定性截断取前 10 条。
- **超时控制**：整体 60 秒；单版本执行沿用 preview 超时（默认 30 秒），任一版本超时则整体返回超时错误。
- **差异说明**：后端自动生成简要差异描述，帮助用户理解为什么某版本更好/更差。
- **Diff 数据来源**：对比响应直接返回两个版本的 Prompt 内容（供 Diff 视图使用，避免额外 API 调用）。
- **入口位置**：在版本列表页添加"版本对比"按钮。
- **AR2 遵循**：所有操作记录 correlationId，支持全链路追踪。

## Story

As a Prompt 优化用户,
I want 对比任意两个 Prompt 版本在同一测试集上的效果差异,
so that 我可以量化评估不同版本的优劣。

## Acceptance Criteria

1. **Given** 用户有多个 Prompt 版本
   **When** 选择"版本对比"功能
   **Then** 显示版本选择器（可选择任意两个版本）

2. **Given** 用户选择两个版本
   **When** 点击"开始对比"
   **Then** 使用同一测试集分别评估两个版本
   **And** 显示对比结果：通过率差异、具体用例差异

3. **Given** 对比结果显示
   **When** 用户查看详情
   **Then** 高亮显示两个版本表现不同的用例
   **And** 对表现不同的用例提供简要差异说明（如失败原因或输出差异），帮助用户理解为什么 A 更好/更差

## Tasks / Subtasks

- 文件落点以 **File Structure Requirements** 为准；本节只描述职责，避免重复写路径。

- [x] 后端：对比 DTO 定义（AC: 1-3）
  - [x] 扩展 `teacher_prompt.rs` 添加对比相关 DTO
    - `PromptCompareRequest`: version_id_a, version_id_b, task_ids（必填）, test_case_ids（可选）
    - `VersionCompareResult`: version_id, version, total_passed, total_failed, pass_rate
    - `CaseComparisonResult`: test_case_id, input, reference, version_a_output, version_a_passed, version_b_output, version_b_passed, is_different, difference_note
    - `CompareSummary`: pass_rate_diff, improved_cases, regressed_cases, unchanged_cases
    - `PromptCompareResponse`: version_a, version_b, version_a_content, version_b_content, case_comparisons, summary
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增类型

- [x] 后端：对比执行服务逻辑（AC: 2, 3）
  - [x] 扩展 `meta_optimization_service/mod.rs`
    - `compare_prompts(pool, api_key_manager, user_id, user_password, request, correlation_id) -> PromptCompareResponse`
      - 复用 `preview_prompt` 的签名参数要求（确保可解密并注入 API Key）
    - 验证两个版本存在且属于当前用户
    - 测试用例来源：复用 8.4 的 task_ids → test_set_ids → cases 逻辑
      - 逻辑位置参考：`backend/src/core/meta_optimization_service/mod.rs` 中 `preview_prompt` 的用例选择段落
    - 串行执行两个版本的预览（复用 preview_prompt 核心逻辑）
    - 合并结果，计算差异统计
    - 生成差异说明（改进/退化/无变化）；当两版本均通过但输出不同，提示“输出存在差异”
    - 测试用例限制：最多 10 条（超限时按 task_ids → test_set_ids → cases 顺序取前 10 条）
    - 超时控制：整体 60 秒；单版本沿用 preview 超时（默认 30 秒），任一版本超时则整体失败

- [x] 后端：对比 API（AC: 1-3）
  - [x] 扩展 `meta_optimization.rs`
    - `POST /api/v1/meta-optimization/prompts/compare` 对比执行
    - 响应包含 `versionAContent` / `versionBContent` 供 Diff 展示
  - [x] 权限校验：需登录（`CurrentUser` 提取 user_id）
  - [x] correlationId：从 headers 提取并写入 tracing 日志
  - [x] 添加 OpenAPI 文档描述
  - [x] 在 `docs.rs` 注册新增 path/schema

- [x] 前端：版本对比入口（AC: 1）
  - [x] 修改 `PromptVersionList.tsx` 添加"版本对比"按钮入口
  - [x] 按钮位置：`CardHeader` 右侧（与标题同一行）
  - [x] 点击后打开对比面板或导航到对比页面（保持与现有详情布局一致）

- [x] 前端：对比面板主组件（AC: 1, 2）
  - [x] 创建 `PromptComparePanel.tsx`
    - 版本选择器（两个下拉框选择 Version A 和 Version B）
    - 测试任务选择（复用 8.4 的任务选择模式）
    - "开始对比"按钮
    - 对比结果展示区域
    - 执行状态指示器（loading/success/error）

- [x] 前端：Prompt Diff 视图组件（AC: 3）
  - [x] 创建 `PromptDiffViewer.tsx`
    - 使用 Monaco DiffEditor（`@monaco-editor/react` 支持 diff 模式）
    - 左右对比视图展示两个版本的 Prompt 内容
    - 动态 import 延迟加载

- [x] 前端：对比结果摘要组件（AC: 2）
  - [x] 创建 `CompareResultSummary.tsx`
    - 通过率对比展示（Version A vs Version B）
    - 改进/退化/无变化用例数统计
    - 通过率差异可视化（正向绿色、负向红色）

- [x] 前端：用例对比列表组件（AC: 3）
  - [x] 创建 `CaseComparisonList.tsx`
    - 高亮显示差异用例（改进用绿色、退化用红色）
    - 每个用例展示：输入、参考答案、A 输出、B 输出、通过状态
    - 差异说明展示
    - 提供"只看差异"过滤选项
    - 按差异程度排序（先显示差异用例）

- [x] 前端：对比服务层封装（AC: 1-3）
  - [x] 扩展 `metaOptimizationService.ts`
    - `comparePrompts(request): Promise<PromptCompareResponse>`
  - [x] 创建 `hooks/usePromptCompare.ts` TanStack Query mutation hook

- [x] 测试与回归（AC: 1-3）
  - [x] 按 **Testing Requirements** 表执行
  - [x] 新增/覆盖测试文件
    - `backend/tests/meta_optimization_test.rs`（扩展对比测试）
    - `frontend/src/features/meta-optimization/components/PromptComparePanel.test.tsx`
    - `frontend/src/features/meta-optimization/components/PromptDiffViewer.test.tsx`
    - `frontend/src/features/meta-optimization/components/CaseComparisonList.test.tsx`

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（correlation_id/user_id/version_id_a/version_id_b/action/timestamp；task_id/iteration_state 标注 N/A）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（对比为只读操作，无状态变更）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review] (placeholder) 将本 Story 的 review 结论沉淀到 `## Review Notes`（含风险/遗留）
- [x] [AI-Review][CRITICAL] 明确 Diff 数据来源：对比响应包含 `versionAContent` / `versionBContent`（避免额外 API）
- [x] [AI-Review][CRITICAL] compare_prompts 签名包含 `pool/api_key_manager/user_password/correlation_id` 并注入 API Key
- [x] [AI-Review][CRITICAL] 超时策略统一：整体 60 秒，单版本沿用 preview 超时（默认 30 秒）
- [x] [AI-Review][CRITICAL] 超过 10 条用例时按 task_ids → test_set_ids → cases 顺序确定性取前 10 条
- [x] [AI-Review][MEDIUM] 两版本都通过但输出不同的差异说明逻辑
- [x] [AI-Review][MEDIUM] 明确“版本对比”入口按钮位置（版本列表 CardHeader 右侧）
- [x] [AI-Review][HIGH] 输出不同但同为通过的用例未被高亮，未满足“表现不同用例高亮”要求（frontend/src/features/meta-optimization/components/CaseComparisonList.tsx:50）
- [x] [AI-Review][MEDIUM] 对比摘要将“输出不同但同为通过”的用例计入 unchanged，导致统计与差异列表不一致（backend/src/core/meta_optimization_service/mod.rs:874）
- [x] [AI-Review][MEDIUM] 缺少失败原因上下文：对比结果未携带 per-case error，差异说明无法解释“为什么退化/失败”（backend/src/core/meta_optimization_service/mod.rs:666）
- [x] [AI-Review][MEDIUM] Story File List 未包含实际变更文件：8-4 复盘文档与最新 validation report（docs/implementation-artifacts/8-4-advanced-user-edit-teacher-model-prompt.md, docs/implementation-artifacts/validation-report-20260121-121630.md）
- [x] [AI-Review][HIGH] 修复 compare 总超时边界（避免外层超时先触发）
- [x] [AI-Review][MEDIUM] 对比请求增加取消机制（AbortController）
- [x] [AI-Review][MEDIUM] 测试补充 output_diff_cases 与双失败场景
- [x] [AI-Review][MEDIUM] useEffect 依赖修正避免闭包陈旧
- [x] [AI-Review][LOW] correlationId 缺失时生成 UUID
- [x] [AI-Review][LOW] 差异用例补充文字标签提升无障碍
- [x] [AI-Review][LOW] compare 端点增加速率限制

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Story 8.3/8.4 已完成）**：
  - `teacher_prompts` 表和版本管理已就绪
  - `meta_optimization_service` 服务层已实现（版本 CRUD + 统计 + 预览）
  - 前端 `meta-optimization/` 模块已建立
  - `PromptVersionList.tsx` 版本列表组件已存在
  - `PromptVersionDetail.tsx` 版本详情组件已存在（含编辑模式）
  - `preview_prompt` 预览执行能力已实现
  - Monaco Editor 已集成（含 lazy import 模式）
  - 测试用例选择与获取链路已打通

- **业务价值（为什么做）**：用户需要量化评估不同 Prompt 版本的优劣，通过对比功能可以直观看到哪个版本在哪些用例上表现更好，辅助决策选择最佳版本。

- **依赖关系**：
  - 依赖 Story 8.3 的版本管理基础设施
  - 依赖 Story 8.4 的预览执行能力
  - 依赖 `TeacherModel` trait 执行预览
  - 依赖 `Evaluator` trait 评估预览结果
  - 依赖优化任务详情与测试集接口获取测试用例
  - 依赖 `ApiKeyManager` + user_password 解密 API Key（对比执行需注入执行目标配置）
  - 复用 TanStack Query 数据获取模式
  - 复用 Monaco Editor（diff 模式）

- **范围边界（必须遵守）**：
  - 本 Story 实现：版本选择器、对比执行、结果展示（含差异高亮和说明）、Prompt 文本 diff 视图
  - 不包含：多样性检测（8.6）
  - 对比执行限制：最多 10 条测试用例，超时 60 秒

### 与其他 Story 的关系

| 功能 | Story 8.3 | Story 8.4 | Story 8.5（本 Story） |
| --- | --- | --- | --- |
| Prompt 版本管理 | ✅ 已实现 | 复用 | 复用 |
| 版本成功率统计 | ✅ 已实现 | 复用 | 复用 |
| 高级编辑 | - | ✅ 已实现 | - |
| 预览执行 | - | ✅ 已实现 | 复用核心逻辑 |
| 版本对比 | - | - | ✅ 新增 |

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/teacher_prompt.rs（扩展）
use std::collections::HashMap;
use crate::domain::models::TaskReference;

/// 对比执行请求
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptCompareRequest {
    /// 版本 A 的 ID
    pub version_id_a: String,
    /// 版本 B 的 ID
    pub version_id_b: String,
    /// 必填：历史任务 ID（用于解析 test_set_ids）
    #[serde(default)]
    pub task_ids: Vec<String>,
    /// 可选：指定测试用例 ID，为空时自动选择最多 10 条
    #[serde(default)]
    pub test_case_ids: Vec<String>,
}

/// 单个版本的对比结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct VersionCompareResult {
    pub version_id: String,
    pub version: i32,
    pub total_passed: i32,
    pub total_failed: i32,
    pub pass_rate: f64,
}

/// 单条测试用例的对比结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CaseComparisonResult {
    pub test_case_id: String,
    pub input: HashMap<String, serde_json::Value>,
    pub reference: TaskReference,
    /// 版本 A 的输出
    pub version_a_output: String,
    /// 版本 A 是否通过
    pub version_a_passed: bool,
    /// 版本 A 的错误信息
    pub version_a_error: Option<String>,
    /// 版本 B 的输出
    pub version_b_output: String,
    /// 版本 B 是否通过
    pub version_b_passed: bool,
    /// 版本 B 的错误信息
    pub version_b_error: Option<String>,
    /// A 与 B 结果是否不同
    pub is_different: bool,
    /// 差异说明（帮助用户理解为什么某版本更好/更差）
    pub difference_note: Option<String>,
}

/// 对比摘要统计
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CompareSummary {
    /// 通过率差异（B - A），正值表示 B 更好
    pub pass_rate_diff: f64,
    /// 改进的用例数（B 通过但 A 失败）
    pub improved_cases: i32,
    /// 退化的用例数（A 通过但 B 失败）
    pub regressed_cases: i32,
    /// 无变化的用例数
    pub unchanged_cases: i32,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: i64,
}

/// 对比执行响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptCompareResponse {
    /// 版本 A 的结果摘要
    pub version_a: VersionCompareResult,
    /// 版本 B 的结果摘要
    pub version_b: VersionCompareResult,
    /// 版本 A 的 Prompt 内容（用于 Diff 视图）
    pub version_a_content: String,
    /// 版本 B 的 Prompt 内容（用于 Diff 视图）
    pub version_b_content: String,
    /// 每条测试用例的对比结果
    pub case_comparisons: Vec<CaseComparisonResult>,
    /// 对比摘要统计
    pub summary: CompareSummary,
}
```

### Difference Note Generation Logic

差异说明生成逻辑（后端实现）：

```rust
fn generate_difference_note(
    a_passed: bool,
    b_passed: bool,
    a_output: &str,
    b_output: &str,
    a_error: &Option<String>,
    b_error: &Option<String>,
) -> Option<String> {
    match (a_passed, b_passed) {
        (true, false) => Some(format!("版本 B 在此用例退化：{}", b_error.as_deref().unwrap_or("未知错误"))),
        (false, true) => Some("版本 B 在此用例改进".to_string()),
        (true, true) => {
            if a_output != b_output {
                Some("两版本均通过，但输出内容存在差异".to_string())
            } else {
                None
            }
        }
        (false, false) => Some("两版本均失败，错误原因可能不同".to_string()),
    }
}
```

### Suggested API Endpoints

```
# 对比执行（新增）
POST /api/v1/meta-optimization/prompts/compare
Request: PromptCompareRequest
Response: ApiResponse<PromptCompareResponse>
权限校验：需登录
限制：task_ids 必填；test_case_ids 最多 10 条，超时 60 秒；响应包含 version_a_content / version_b_content 供 Diff 展示
```

### Frontend Component Notes

**PromptComparePanel.tsx 结构：**
```tsx
import { useState } from 'react';
import { usePromptVersions } from '../hooks/usePromptVersions';
import { usePromptCompare } from '../hooks/usePromptCompare';
import { PromptDiffViewer } from './PromptDiffViewer';
import { CompareResultSummary } from './CompareResultSummary';
import { CaseComparisonList } from './CaseComparisonList';

interface PromptComparePanelProps {
  workspaceId: string;
}

export function PromptComparePanel({ workspaceId }: PromptComparePanelProps) {
  const [versionIdA, setVersionIdA] = useState<string>('');
  const [versionIdB, setVersionIdB] = useState<string>('');
  const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([]);
  
  const { data: versions } = usePromptVersions(workspaceId);
  const { mutate: compare, isPending, data: result } = usePromptCompare();

  const handleCompare = () => {
    compare({
      versionIdA,
      versionIdB,
      taskIds: selectedTaskIds,
      testCaseIds: [],
    });
  };

  return (
    <div className="space-y-6">
      {/* 版本选择器 */}
      <div className="grid grid-cols-2 gap-4">
        <VersionSelector 
          label="版本 A（基准）"
          versions={versions}
          value={versionIdA}
          onChange={setVersionIdA}
          excludeId={versionIdB}
        />
        <VersionSelector 
          label="版本 B（对比）"
          versions={versions}
          value={versionIdB}
          onChange={setVersionIdB}
          excludeId={versionIdA}
        />
      </div>

      {/* 任务选择器 */}
      <TaskSelector 
        workspaceId={workspaceId}
        selected={selectedTaskIds}
        onSelect={setSelectedTaskIds}
      />

      {/* 开始对比按钮 */}
      <Button 
        onClick={handleCompare} 
        disabled={isPending || !versionIdA || !versionIdB || selectedTaskIds.length === 0}
      >
        {isPending ? '对比执行中...' : '开始对比'}
      </Button>

      {/* 对比结果 */}
      {result && (
        <>
          <CompareResultSummary summary={result.summary} versionA={result.versionA} versionB={result.versionB} />
          <PromptDiffViewer
            versionA={{ version: result.versionA.version, content: result.versionAContent }}
            versionB={{ version: result.versionB.version, content: result.versionBContent }}
          />
          <CaseComparisonList comparisons={result.caseComparisons} />
        </>
      )}
    </div>
  );
}
```

**PromptDiffViewer.tsx 结构：**
```tsx
import { lazy, Suspense } from 'react';
const MonacoDiffEditor = lazy(async () => import('@monaco-editor/react').then(m => ({ default: m.DiffEditor })));

interface PromptDiffViewerProps {
  versionA: { version: number; content: string };
  versionB: { version: number; content: string };
}

export function PromptDiffViewer({ versionA, versionB }: PromptDiffViewerProps) {
  return (
    <div className="border rounded-lg overflow-hidden">
      <div className="flex justify-between px-4 py-2 bg-muted text-sm">
        <span>版本 {versionA.version}（基准）</span>
        <span>版本 {versionB.version}（对比）</span>
      </div>
      <Suspense fallback={<div className="h-[300px] flex items-center justify-center text-muted-foreground">加载 Diff 编辑器中...</div>}>
        <MonacoDiffEditor
          height="400px"
          language="markdown"
          theme="vs-light"
          original={versionA.content}
          modified={versionB.content}
          options={{
            readOnly: true,
            renderSideBySide: true,
            minimap: { enabled: false },
            wordWrap: 'on',
          }}
        />
      </Suspense>
    </div>
  );
}
```

**CaseComparisonList.tsx 结构：**
```tsx
import { useState } from 'react';
import { CaseComparisonResult } from '@/types/generated/models';

interface CaseComparisonListProps {
  comparisons: CaseComparisonResult[];
}

export function CaseComparisonList({ comparisons }: CaseComparisonListProps) {
  const [showOnlyDiff, setShowOnlyDiff] = useState(false);

  // 按差异程度排序：先显示差异用例
  const sortedComparisons = [...comparisons].sort((a, b) => {
    if (a.isDifferent && !b.isDifferent) return -1;
    if (!a.isDifferent && b.isDifferent) return 1;
    return 0;
  });

  const filteredComparisons = showOnlyDiff 
    ? sortedComparisons.filter(c => c.isDifferent)
    : sortedComparisons;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-medium">用例对比详情</h3>
        <label className="flex items-center gap-2 text-sm">
          <input 
            type="checkbox" 
            checked={showOnlyDiff} 
            onChange={(e) => setShowOnlyDiff(e.target.checked)} 
          />
          只看差异
        </label>
      </div>

      {filteredComparisons.map((comparison) => (
        <CaseComparisonCard key={comparison.testCaseId} comparison={comparison} />
      ))}
    </div>
  );
}

function CaseComparisonCard({ comparison }: { comparison: CaseComparisonResult }) {
  // 判断是改进还是退化
  const isImproved = !comparison.versionAPassed && comparison.versionBPassed;
  const isRegressed = comparison.versionAPassed && !comparison.versionBPassed;

  return (
    <div className={cn(
      "border rounded-lg p-4",
      isImproved && "border-green-300 bg-green-50",
      isRegressed && "border-red-300 bg-red-50",
    )}>
      {/* 用例信息 */}
      <div className="text-sm text-muted-foreground mb-2">
        用例 ID: {comparison.testCaseId}
      </div>

      {/* 输入 */}
      <div className="mb-2">
        <span className="font-medium">输入：</span>
        <pre className="text-sm bg-muted p-2 rounded mt-1 overflow-auto">
          {JSON.stringify(comparison.input, null, 2)}
        </pre>
      </div>

      {/* A/B 输出对比 */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <span className="font-medium">版本 A 输出：</span>
          <span className={comparison.versionAPassed ? "text-green-600" : "text-red-600"}>
            {comparison.versionAPassed ? "✓ 通过" : "✗ 失败"}
          </span>
          <pre className="text-sm bg-muted p-2 rounded mt-1 overflow-auto max-h-32">
            {comparison.versionAOutput}
          </pre>
        </div>
        <div>
          <span className="font-medium">版本 B 输出：</span>
          <span className={comparison.versionBPassed ? "text-green-600" : "text-red-600"}>
            {comparison.versionBPassed ? "✓ 通过" : "✗ 失败"}
          </span>
          <pre className="text-sm bg-muted p-2 rounded mt-1 overflow-auto max-h-32">
            {comparison.versionBOutput}
          </pre>
        </div>
      </div>

      {/* 差异说明 */}
      {comparison.differenceNote && (
        <div className="mt-2 text-sm text-muted-foreground">
          💡 {comparison.differenceNote}
        </div>
      )}
    </div>
  );
}
```

**UX 对齐**：
- 版本选择器互斥（A 和 B 不能选择相同版本）
- 对比执行显示进度指示器
- 结果按差异程度排序（先显示差异用例）
- 改进用例用绿色高亮，退化用例用红色高亮
- 提供"只看差异"过滤选项
- Diff 视图使用左右对比布局

### Dev Agent Guardrails（避免常见踩坑）

- **Monaco DiffEditor 延迟加载**：使用动态 import 延迟加载，与 Story 8.4 模式一致
- **版本选择互斥**：确保 A 和 B 不能选择相同版本
- **测试用例限制**：最多 10 条，避免长时间阻塞
- **超时处理**：必须处理 60 秒超时，显示友好提示
- **串行执行**：两个版本的预览串行执行，避免并发竞争
- **API Key 注入**：compare_prompts 必须传入 `api_key_manager` + `user_password`，复用 preview 的 ExecutionTargetConfig 构建逻辑
- **日志安全**：日志不得包含 Prompt 完整内容，仅记录 id/version
- **差异说明**：后端自动生成，前端直接展示
- **结果排序**：默认按差异程度排序，先显示差异用例
- **采样规则**：超过 10 条用例时按 task_ids → test_set_ids → cases 顺序取前 10 条

### Technical Requirements（必须满足）

- 时间戳使用 Unix 毫秒存储，API 返回 ISO 8601
- API 响应使用 `ApiResponse<T>` 统一结构
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`
- Monaco DiffEditor 使用动态 import 延迟加载
- 对比执行最多 10 条测试用例，超时 60 秒（单版本沿用 preview 超时，默认 30 秒）
- 超过 10 条时按 task_ids → test_set_ids → cases 顺序确定性取前 10 条
- 对比测试用例来源：历史任务 → `test_set_ids` → `test_sets.cases_json`
- 对比响应包含 `versionAContent` / `versionBContent` 以支持 Diff 视图

### Backward Compatibility / Non-Regressions（必须遵守）

- 复用 Story 8.3/8.4 的 `teacher_prompts` 表，不新增数据库迁移
- 复用 Story 8.4 的 `preview_prompt` 执行逻辑
- 新增 `/api/v1/meta-optimization/prompts/compare` 端点，不修改现有 API
- 扩展现有组件（`PromptVersionList.tsx`），添加对比入口

### Previous Story Learnings (Story 8.3/8.4 复盘/模式/测试)

- **后端路由模式**：使用 `CurrentUser` 提取器进行权限校验
- **DTO 设计模式**：使用 `#[serde(rename_all = "camelCase")]` + `#[ts(export_to = "models/")]`
- **前端模块结构**：采用 `components/` + `hooks/` + `services/` + `index.ts`
- **测试实践**：使用 MSW + `QueryClientProvider`，通过 `useAuthStore` 注入登录态
- **预览执行**：复用 `create_teacher_model` + `create_evaluator_for_task_config` 工厂
- **Monaco Editor**：使用 lazy import + `vs-light` 主题
- **超时控制**：使用 `tokio::time::timeout`

### Latest Technical Notes（基于当前项目版本）

**Breaking Changes / Best Practices**
- Monaco Editor：支持 DiffEditor 组件，通过 `@monaco-editor/react` 的 `DiffEditor` 导出
- TanStack Query v5：mutation 使用 `useMutation` hook
- Axum 0.8：路由路径参数语法 `/{param}`

**Performance / Deprecation Notes**
- Monaco DiffEditor 延迟加载，避免首屏加载过重
- 对比执行串行两个版本，总超时 60 秒

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/domain/models/teacher_prompt.rs`：扩展对比 DTO
  - `backend/src/core/meta_optimization_service/mod.rs`：扩展对比服务
  - `backend/src/api/routes/meta_optimization.rs`：扩展对比 API
  - `frontend/src/features/meta-optimization/components/PromptComparePanel.tsx`：对比面板（新增）
  - `frontend/src/features/meta-optimization/components/PromptDiffViewer.tsx`：Diff 视图（新增）
  - `frontend/src/features/meta-optimization/components/CompareResultSummary.tsx`：结果摘要（新增）
  - `frontend/src/features/meta-optimization/components/CaseComparisonList.tsx`：用例对比列表（新增）
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- tokio：异步运行时
- chrono：时间戳处理
- React：`react@19.x`
- TanStack Query：服务端状态管理
- shadcn/ui：UI 组件库
- **@monaco-editor/react**：代码编辑器 + DiffEditor（已存在依赖）

### Deployment / Environment Notes（部署/环境）

- 本 Story 不新增数据库迁移
- 前端依赖无需新增：`@monaco-editor/react` 已存在（支持 DiffEditor）
- 部署验证：建议执行 `cargo test`、`pnpm vitest run`、`pnpm vite build`

### File Structure Requirements（落点约束）

**后端**：
- 对比 DTO：`backend/src/domain/models/teacher_prompt.rs`（扩展）
- 对比服务：`backend/src/core/meta_optimization_service/mod.rs`（扩展）
- 对比 API：`backend/src/api/routes/meta_optimization.rs`（扩展）
- OpenAPI：`backend/src/api/routes/docs.rs`（扩展）
- 类型生成：`backend/src/bin/gen-types.rs`（扩展）

**前端**：
- 对比面板：`frontend/src/features/meta-optimization/components/PromptComparePanel.tsx`（新增）
- Diff 视图：`frontend/src/features/meta-optimization/components/PromptDiffViewer.tsx`（新增）
- 结果摘要：`frontend/src/features/meta-optimization/components/CompareResultSummary.tsx`（新增）
- 用例对比列表：`frontend/src/features/meta-optimization/components/CaseComparisonList.tsx`（新增）
- 版本列表：`frontend/src/features/meta-optimization/components/PromptVersionList.tsx`（扩展，添加入口）
- 服务层：`frontend/src/features/meta-optimization/services/metaOptimizationService.ts`（扩展）
- 对比 Hook：`frontend/src/features/meta-optimization/hooks/usePromptCompare.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

**测试**：
- 后端测试：`backend/tests/meta_optimization_test.rs`（扩展）
- 对比面板测试：`frontend/src/features/meta-optimization/components/PromptComparePanel.test.tsx`（新增）
- Diff 视图测试：`frontend/src/features/meta-optimization/components/PromptDiffViewer.test.tsx`（新增）
- 用例对比测试：`frontend/src/features/meta-optimization/components/CaseComparisonList.test.tsx`（新增）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 对比执行 | 正确执行两个版本并返回对比结果 |
| 后端单测 | 版本校验 | 版本不存在返回 404 |
| 后端单测 | 权限校验 | 非自己的版本返回 403 |
| 后端单测 | 同版本校验 | A 和 B 相同返回 400 |
| 后端单测 | 限制校验 | 超过 10 条测试用例正确拒绝 |
| 后端单测 | 采样规则 | 超过 10 条时按确定性顺序取前 10 条 |
| 后端单测 | 超时处理 | 超过 60 秒正确返回超时错误 |
| 后端单测 | 差异说明 | 正确生成改进/退化/无变化说明 |
| 前端测试 | 版本选择器 | 正确渲染两个版本选择器，互斥逻辑正确 |
| 前端测试 | 对比执行 | 点击后调用 API 并展示结果 |
| 前端测试 | Diff 视图 | Monaco DiffEditor 正确渲染 |
| 前端测试 | Diff 数据 | 使用 compare 响应内容渲染 Diff（versionAContent/versionBContent） |
| 前端测试 | 差异高亮 | 改进/退化用例正确高亮 |
| 前端测试 | 只看差异 | 过滤功能正确工作 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 参考 `frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx` 现有实现
- 参考 `frontend/src/features/meta-optimization/components/PromptPreviewPanel.tsx` 预览面板模式
- 参考 `backend/src/api/routes/meta_optimization.rs` 路由模式
- 复用 `backend/src/core/meta_optimization_service/mod.rs` 服务层
- Monaco DiffEditor 参考 `@monaco-editor/react` 官方文档

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 8 / Story 8.5）
- PRD 元优化：`docs/project-planning-artifacts/prd.md#能力区域 9: 元优化`
- 架构（元优化）：`docs/project-planning-artifacts/architecture.md#9. 元优化`
- 架构（UX PromptDiff）：`docs/project-planning-artifacts/architecture.md`（PromptDiff 使用 Monaco DiffEditor）
- Story 8.3（前序）：`docs/implementation-artifacts/8-3-meta-optimization-basics.md`
- Story 8.4（前序）：`docs/implementation-artifacts/8-4-advanced-user-edit-teacher-model-prompt.md`
- 元优化服务：`backend/src/core/meta_optimization_service/mod.rs`
- 版本列表组件：`frontend/src/features/meta-optimization/components/PromptVersionList.tsx`
- 预览面板组件：`frontend/src/features/meta-optimization/components/PromptPreviewPanel.tsx`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- `cargo run --bin gen-types`
- `cargo test`
- `pnpm vitest run`（首次因用例选择器测试未匹配 label 失败，修复后通过）
- `pnpm vite build`

### Completion Notes List

- 完成 Prompt 对比 DTO/服务/API：支持版本校验、确定性采样、差异说明、超时控制与 correlationId 日志。
- 前端新增版本对比入口与主面板（Diff/摘要/用例列表），并封装 compare 服务与 hook。
- 补齐后端与前端测试用例并完成全量回归。
- 修复差异高亮与摘要统计，补充 per-case 失败原因并更新类型/测试。
- 修复 compare 超时边界与请求取消，补充 rate limit/无障碍标签与测试。
- 补齐对比面板错误态测试，完善 rate limit 清理与可配置项。

### File List

- backend/src/api/routes/docs.rs
- backend/src/api/routes/meta_optimization.rs
- backend/src/bin/gen-types.rs
- backend/src/core/meta_optimization_service/mod.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/models/teacher_prompt.rs
- backend/src/shared/error_codes.rs
- backend/tests/meta_optimization_test.rs
- frontend/src/features/meta-optimization/components/CaseComparisonList.test.tsx
- frontend/src/features/meta-optimization/components/CaseComparisonList.tsx
- frontend/src/features/meta-optimization/components/CompareResultSummary.tsx
- frontend/src/features/meta-optimization/components/PromptComparePanel.test.tsx
- frontend/src/features/meta-optimization/components/PromptComparePanel.tsx
- frontend/src/features/meta-optimization/components/PromptDiffViewer.test.tsx
- frontend/src/features/meta-optimization/components/PromptDiffViewer.tsx
- frontend/src/features/meta-optimization/components/PromptVersionList.test.tsx
- frontend/src/features/meta-optimization/components/PromptVersionList.tsx
- frontend/src/features/meta-optimization/hooks/usePromptCompare.ts
- frontend/src/features/meta-optimization/index.ts
- frontend/src/features/meta-optimization/services/metaOptimizationService.ts
- frontend/src/pages/MetaOptimizationPage.tsx
- frontend/src/lib/api.ts
- frontend/src/types/generated/models/CaseComparisonResult.ts
- frontend/src/types/generated/models/CompareSummary.ts
- frontend/src/types/generated/models/PromptCompareRequest.ts
- frontend/src/types/generated/models/PromptCompareResponse.ts
- frontend/src/types/generated/models/VersionCompareResult.ts
- docs/implementation-artifacts/8-4-advanced-user-edit-teacher-model-prompt.md
- docs/implementation-artifacts/8-5-prompt-version-comparison-growth.md
- docs/implementation-artifacts/validation-report-20260121-121630.md
- docs/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-01-21: 完成 Prompt 版本对比后端/前端/测试与类型生成，回归验证通过。
- 2026-01-21: 修复对比摘要输出差异计数、用例高亮与失败原因透出，更新类型/测试与文档。
- 2026-01-21: 修复 compare 超时边界与请求取消，补充 rate limit/无障碍标签与测试。
- 2026-01-21: 补齐对比面板错误态测试，完善 rate limit 清理与可配置项。

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 输出不同但同为通过的用例未被高亮，表现不同用例无法快速定位（已修复：差异用例新增 amber 高亮）
- [x] [MEDIUM] 对比摘要统计未区分“输出差异”，导致 unchanged 统计与差异列表不一致（已修复：新增 output_diff_cases）
- [x] [MEDIUM] 缺少失败原因上下文：对比结果未携带 per-case error，差异说明无法解释“为什么退化/失败”（已修复：透出 per-case error 并纳入差异说明）
- [x] [MEDIUM] Story File List 与 git 变更不一致（已修复：补齐实际变更文件）
- [x] [HIGH] compare 总超时边界问题可能触发意外超时（已修复：按剩余时间裁剪每次预览超时）
- [x] [MEDIUM] 对比请求缺少取消机制（已修复：AbortController）
- [x] [MEDIUM] 测试缺少 output_diff_cases 与双失败覆盖（已修复：补充测试用例）
- [x] [MEDIUM] useEffect 依赖缺失导致闭包陈旧（已修复：useCallback + deps）
- [x] [LOW] correlationId 缺失时为 unknown（已修复：生成 UUID）
- [x] [LOW] 差异用例仅颜色区分（已修复：增加文字标签）
- [x] [LOW] compare 端点缺少速率限制（已修复：每用户/分钟 5 次）

### Decisions

- [x] [DECISION] 选择在 compare 响应中直接返回 Prompt 内容，避免额外 API 调用与状态同步复杂度。
- [x] [DECISION] 保持两版本串行执行（符合 Story 约束），通过确定性采样与明确超时策略保证可复现与可预期。
- [x] [DECISION] 复用 preview 的用例获取与 API Key 注入逻辑，降低实现偏差与回归风险。

### Risks / Tech Debt

- [ ] [RISK] compare 响应包含完整 Prompt 内容可能导致响应体偏大（单版本 ≤100KB）；若未来出现更大内容，需考虑按需获取或压缩。
- [ ] [RISK] 若单版本预览仍为 30 秒，复杂任务可能频繁超时；触发条件为单用例执行时间过长或模型响应慢。

### Follow-ups

- [x] 对齐 DTO/响应：`PromptCompareResponse` 增加 `versionAContent` / `versionBContent` 并更新 gen-types。
- [x] 实现 compare_prompts 完整签名与 API Key 注入（复用 preview 构建 ExecutionTargetConfig）。
- [x] 实现确定性采样规则（超 10 条按 task_ids → test_set_ids → cases 取前 10）。
- [x] 实现并测试超时策略（整体 60 秒 + 单版本 30 秒）。
- [x] 实现“都通过但输出不同”差异说明逻辑并补测试。
- [x] 明确并实现“版本对比”入口按钮位置与跳转/面板交互。
