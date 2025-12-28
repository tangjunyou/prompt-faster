# Story 1.8 Implementation Summary

**Date:** 2025-12-26

## Changes Overview

- 集中管理错误码：新增并统一使用 `backend/src/shared/error_codes.rs`。
- OpenAPI/Swagger 集成：添加 utoipa 注解与 `ToSchema` derive，Swagger UI 路由 `/swagger` 可访问。
- 错误响应规范化：补齐 error_codes 使用与 dev-only details 行为。
- 集成测试完善：`backend/tests/error_handling_test.rs` 改为内存测试应用，不再依赖外部服务。

## Files Touched (high-level)

- Backend: `backend/src/api/routes/*.rs`, `backend/src/api/response.rs`, `backend/src/shared/error_codes.rs`, `backend/src/api/routes/docs.rs`
- Tests: `backend/tests/error_handling_test.rs`
- Docs: `docs/implementation-artifacts/1-8-unified-error-response-and-openapi-docs.md`
