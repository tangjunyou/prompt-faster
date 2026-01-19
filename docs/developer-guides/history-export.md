# 历史记录导出接口

本文档描述历史记录导出接口的用法、响应结构与注意事项。

## 1) 接口定义

- 方法: `GET`
- 路径: `/api/v1/tasks/{task_id}/history/export`
- 权限: 仅任务所有者可访问
- 响应: `ApiResponse<HistoryExportData>`（JSON 文件下载）

响应头:
- `Content-Type: application/json`
- `Content-Disposition: attachment; filename="{task_name}_history_{timestamp}.json"`

## 2) 请求参数

- `task_id` (Path, string): 任务 ID
- 无 query 参数

## 3) 响应结构

> 字段命名为 `camelCase`。时间字段均为 ISO 8601 字符串。

```
ApiResponse<HistoryExportData> {
  data: HistoryExportData
}
```

`HistoryExportData`:
- `task`: 任务元信息
- `iterations`: 迭代摘要列表
- `checkpoints`: Checkpoint 摘要列表
- `events`: 历史事件列表
- `branches`: 分支摘要
- `truncated`: 是否被截断（当前实现为全量导出，固定为 `false`）
- `eventTotal`: 历史事件总数
- `checkpointTotal`: Checkpoint 总数（含归档）
- `exportLimit`: 导出上限（当前为 `0`，表示不限制）
- `exportedAt`: 导出时间

`TaskExportMeta`:
- `id`, `name`, `status`, `createdAt`, `updatedAt`

`IterationExportEntry`:
- `iteration`, `prompt`, `ruleSystem`, `passRate`, `status`, `createdAt`

`BranchInfo`:
- `branchId`, `parentBranchId`, `createdAt`, `checkpointCount`

## 4) 注意事项

- **全量导出**: 当前实现为全量导出，数据量可能较大。
- **敏感信息**: 返回中可能包含 prompt 片段或其他业务信息，前端展示需遵循脱敏规范。
- **权限校验**: 仅任务所有者可导出，非所有者请求返回 403。

## 5) 参考实现

- 路由实现: `backend/src/api/routes/history.rs`
- 数据结构: `backend/src/domain/models/history.rs`
- 事件结构: `backend/src/domain/models/history_event.rs`
