# Validation Report

**Document:** /Users/jingshun/Desktop/prompt 自动优化（规范开发）/docs/implementation-artifacts/7-1-checkpoint-auto-save.md
**Checklist:** /Users/jingshun/Desktop/prompt 自动优化（规范开发）/_bmad/bmm/workflows/4-implementation/create-story/checklist.md
**Date:** 2026-01-18 14:16:54

## Summary
- Overall: 30/69 passed (43.5%)
- Critical Issues: 13

## Section Results

### Process & Framework Requirements (Meta)
Pass Rate: N/A (process instructions, not story content)

➖ N/A CRITICAL MISSION: Outperform and fix original create-story LLM
Evidence: Process instruction for validator; not expected in story content.

➖ N/A Purpose is NOT just to validate - fix and prevent LLM mistakes
Evidence: Process instruction for validator; not expected in story content.

➖ N/A EXHAUSTIVE ANALYSIS REQUIRED
Evidence: Process instruction for validator; not expected in story content.

➖ N/A UTILIZE SUBPROCESSES AND SUBAGENTS
Evidence: Process instruction for validator; not expected in story content.

➖ N/A COMPETITIVE EXCELLENCE
Evidence: Process instruction for validator; not expected in story content.

### Critical Mistakes to Prevent
Pass Rate: 4/8 (50%)

✓ PASS Reinventing wheels
Evidence: “复用 `PauseController` 获取用户介入数据” and reuse notes in Project Structure Notes. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L238-L241, #L518-L523)

✓ PASS Wrong libraries
Evidence: Library/Framework requirements list explicit stacks. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L480-L486)

✓ PASS Wrong file locations
Evidence: File Structure Requirements list explicit paths. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L488-L502)

⚠ PARTIAL Breaking regressions
Evidence: Scope boundaries and pause_state compatibility noted, but no explicit regression test plan for existing iterations/history behavior. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L220-L225, #L238-L241)
Impact: Risk of unintended changes to existing iteration history or pause_state behavior.

⚠ PARTIAL Ignoring UX
Evidence: Only minimal UX mention (frontend optional + error.details not shown). (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L160-L165, #L467)
Impact: UX reliability requirements like “断点恢复” context summary/time-line are not surfaced. (docs/project-planning-artifacts/ux-design-specification.md#L86-L93)

✓ PASS Vague implementations
Evidence: Detailed tasks, schema, and code-level guidance are specified. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L98-L175, #L243-L431)

⚠ PARTIAL Lying about completion
Evidence: Hard gate checklist exists but Dev Agent Record and File List are empty placeholders. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L177-L184, #L560-L571)
Impact: Completion evidence could be claimed without traceable artifacts.

⚠ PARTIAL Not learning from past work
Evidence: References prior story learnings and Epic 6 baseline but does not summarize actionable learnings. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L196-L205, #L534)
Impact: Prior pitfalls may be repeated.

### Step 1: Load and Understand the Target
Pass Rate: 2/3 (66.7%)

➖ N/A Load workflow configuration
Evidence: Process instruction; not applicable to story content.

➖ N/A Load the story file
Evidence: Process instruction; not applicable to story content.

➖ N/A Load validation framework
Evidence: Process instruction; not applicable to story content.

✓ PASS Extract metadata (epic_num, story_num, story_key, story_title)
Evidence: Story title and key explicitly present. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L1-L7)

⚠ PARTIAL Resolve workflow variables (story_dir, output_folder, epics_file, architecture_file, etc.)
Evidence: References to source docs exist but variables are not resolved in-place. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L525-L533)
Impact: Developer may need to re-derive locations during implementation.

✓ PASS Understand current status
Evidence: Status fields provided. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L3, #L555-L558)

### Step 2.1: Epics and Stories Analysis
Pass Rate: 5/5 (100%)

✓ PASS Epic objectives and business value
Evidence: Business value explicitly described. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L212-L213)

✓ PASS ALL stories in this epic
Evidence: Epic 7 story list included. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L206-L210)

✓ PASS Our specific story requirements and acceptance criteria
Evidence: Acceptance Criteria listed. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L52-L95)

✓ PASS Technical requirements and constraints
Evidence: Technical requirements + architecture compliance listed. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L460-L479)

✓ PASS Cross-story dependencies and prerequisites
Evidence: Epic 7 gates + dependency list included. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L9-L32, #L214-L218)

### Step 2.2: Architecture Deep-Dive
Pass Rate: 4/9 (44.4%)

✓ PASS Technical stack with versions
Evidence: Version snapshot list. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L480-L486)

✓ PASS Code structure and organization patterns
Evidence: File structure requirements + project structure notes. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L488-L523)

⚠ PARTIAL API design patterns and contracts
Evidence: Endpoints listed and ApiResponse requirement noted, but error schema and pagination/limits not fully specified. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L433-L446, #L465)
Impact: Risk of inconsistent API behaviors across endpoints.

⚠ PARTIAL Database schemas and relationships
Evidence: Schema defined, but missing PRD-required fields such as `workspace_id`, `step`, `parent_id`, and `lineage_type`. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L248-L260; docs/project-planning-artifacts/prd.md#L636-L649)
Impact: Potential mismatch with PRD data model and future rollback/lineage requirements.

⚠ PARTIAL Security requirements and patterns
Evidence: Permissions mentioned for checkpoint APIs, but broader data isolation/encryption requirements not referenced. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L151, #L457; docs/project-planning-artifacts/prd.md#L1043-L1051)
Impact: Security expectations may be inconsistently applied.

✓ PASS Performance requirements and optimization strategies
Evidence: WAL + FULL sync, LRU cache, memory thresholds, idle save timer. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L36-L43, #L65-L69, #L131-L133)

✓ PASS Testing standards and frameworks
Evidence: Detailed testing matrix and regression commands. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L504-L516)

✗ FAIL Deployment and environment patterns
Evidence: No deployment/environment guidance found in story file.
Impact: Risk of implementation choices that break local deployment/runtime assumptions.

⚠ PARTIAL Integration patterns and external services
Evidence: Integration with optimization engine and pause_state noted, but no WS event or historical-source integration guidance. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L135-L140, #L238-L241)
Impact: Cross-module data flow may be implemented inconsistently.

### Step 2.3: Previous Story Intelligence
Pass Rate: 0/6 (0%)

⚠ PARTIAL Dev notes and learnings
Evidence: References Epic 6 baseline and prior story learnings, but no extracted learnings. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L196-L205, #L534)
Impact: Risks repeating known pitfalls.

✗ FAIL Review feedback and corrections needed
Evidence: No prior review findings summarized in story.
Impact: Known defects may reappear.

✗ FAIL Files created/modified and their patterns
Evidence: No prior file list or patterns included.
Impact: Developers may miss established conventions.

✗ FAIL Testing approaches that worked/didn't work
Evidence: No previous testing insights referenced.
Impact: Repeated test gaps likely.

✗ FAIL Problems encountered and solutions found
Evidence: No prior problem/solution summary provided.
Impact: Known failure modes may recur.

⚠ PARTIAL Code patterns and conventions established
Evidence: Naming/serialization conventions present but not tied to previous story learnings. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L469-L479)
Impact: Conventions may be inconsistently applied.

### Step 2.4: Git History Analysis
Pass Rate: 0/5 (0%)

⚠ PARTIAL Files created/modified in previous work
Evidence: Git Intelligence Summary lists key components but not files/paths. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L536-L542)
Impact: Hard to reuse existing modules precisely.

✗ FAIL Code patterns and conventions used
Evidence: No git-derived patterns summarized.
Impact: Divergent implementations likely.

✗ FAIL Library dependencies added/changed
Evidence: No git-derived dependency changes noted.
Impact: Potential version or feature mismatches.

⚠ PARTIAL Architecture decisions implemented
Evidence: Summary mentions RunControlState/PauseController, but no detailed decisions. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L536-L542)
Impact: Architectural intent may be missed.

✗ FAIL Testing approaches used
Evidence: No git-derived testing approach summary.
Impact: Tests may be misaligned with existing patterns.

### Step 2.5: Latest Technical Research
Pass Rate: 1/3 (33.3%)

✓ PASS Identify libraries/frameworks mentioned
Evidence: Library/Framework list included. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L480-L486)

✗ FAIL Breaking changes or security updates
Evidence: No mention of latest versions, breaking changes, or security updates in story.
Impact: Implementation may rely on outdated behaviors.

✗ FAIL Performance improvements or deprecations / Best practices for current versions
Evidence: No best-practice or deprecation notes included.
Impact: Risk of suboptimal or deprecated usage.

### Step 3.1: Reinvention Prevention Gaps
Pass Rate: 3/3 (100%)

✓ PASS Wheel reinvention prevention
Evidence: Reuse PauseController and existing structures. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L238-L241, #L520-L523)

✓ PASS Code reuse opportunities identified
Evidence: Explicit reuse of iteration_engine structure and pause_state integration. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L518-L523, #L238-L241)

✓ PASS Existing solutions mentioned
Evidence: Existing Checkpoint struct and pause_state noted. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L203-L205, #L238-L241)

### Step 3.2: Technical Specification DISASTERS
Pass Rate: 2/5 (40%)

✓ PASS Wrong libraries/frameworks prevention
Evidence: Version snapshot defined. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L480-L486)

⚠ PARTIAL API contract violations prevention
Evidence: Endpoints and ApiResponse noted, but full response fields and error codes not fully specified. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L433-L446, #L465)
Impact: Clients may rely on inconsistent response payloads.

✗ FAIL Database schema conflicts prevention
Evidence: Story schema omits PRD checkpoint lineage fields (`parent_id`, `lineage_type`) and uses `task_id` instead of `workspace_id/step` fields defined in PRD. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L248-L260; docs/project-planning-artifacts/prd.md#L636-L649)
Impact: Incompatible data model with rollback/branching requirements.

⚠ PARTIAL Security vulnerabilities prevention
Evidence: Permission check exists but no mention of data isolation/encryption policies for checkpoint data. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L151, #L457; docs/project-planning-artifacts/prd.md#L1043-L1051)
Impact: Security posture may be inconsistently enforced.

✓ PASS Performance disasters prevention
Evidence: WAL + FULL sync, memory thresholds and LRU caching specified. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L36-L43, #L131-L133)

### Step 3.3: File Structure DISASTERS
Pass Rate: 2/4 (50%)

✓ PASS Wrong file locations prevention
Evidence: Explicit backend/frontend path requirements. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L488-L502)

✓ PASS Coding standard violations prevention
Evidence: Naming conventions and serde rename rules specified. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L469-L479)

⚠ PARTIAL Integration pattern breaks
Evidence: Integration points listed but no explicit cross-system data flow contracts (e.g., WS events, history unification). (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L135-L140, #L523)
Impact: Data flow fragmentation risk.

✗ FAIL Deployment failures prevention
Evidence: No deployment/runtime guidance in story.
Impact: Implementation may break environment assumptions.

### Step 3.4: Regression DISASTERS
Pass Rate: 1/4 (25%)

⚠ PARTIAL Breaking changes prevention
Evidence: Scope boundaries and compatibility notes exist but no explicit regression checklist. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L220-L225, #L238-L241)
Impact: Existing pause_state/iteration behavior could regress.

✓ PASS Test failures prevention
Evidence: Comprehensive test matrix + regression commands included. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L504-L516)

⚠ PARTIAL UX violations prevention
Evidence: Only minimal UX constraints included; broader reliability UX requirements omitted. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L160-L165, #L467; docs/project-planning-artifacts/ux-design-specification.md#L86-L93)
Impact: UX expectations for recovery may be missed.

⚠ PARTIAL Learning failures prevention
Evidence: Prior learnings referenced but not captured in story. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L534)
Impact: Repeated issues likely.

### Step 3.5: Implementation DISASTERS
Pass Rate: 3/4 (75%)

✓ PASS Vague implementations prevention
Evidence: Detailed tasks, schemas, and code guidance. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L98-L175, #L243-L431)

⚠ PARTIAL Completion lies prevention
Evidence: Hard Gate Checklist exists but no concrete completion evidence fields filled. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L177-L184, #L560-L571)
Impact: Definition-of-done may be weakly enforced.

✓ PASS Scope creep prevention
Evidence: Scope boundaries explicitly defined. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L220-L225)

✓ PASS Quality failures prevention
Evidence: Testing requirements and regression commands defined. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L504-L516)

### Step 4: LLM-Dev-Agent Optimization Issues
Pass Rate: 1/5 (20%)

⚠ PARTIAL Verbosity problems
Evidence: Extensive inline code and long lists may add token overhead. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L243-L431)
Impact: Higher token usage may reduce model focus.

⚠ PARTIAL Ambiguity issues
Evidence: Requirements like “内存使用监控与告警阈值配置” lack implementation detail. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L69)
Impact: Multiple interpretations possible.

⚠ PARTIAL Context overload
Evidence: Story includes large code blocks and long sections without prioritization. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L243-L431)
Impact: Important signals may be diluted.

✗ FAIL Missing critical signals
Evidence: PRD mandates checkpoint lineage (`parent_id`, `lineage_type`) and step granularity not reflected in story. (docs/project-planning-artifacts/prd.md#L636-L649; docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L248-L260)
Impact: LLM dev may implement incomplete data model.

✓ PASS Poor structure
Evidence: Clear headings, sections, and checklists present. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L9-L588)

### Step 4: LLM Optimization Principles
Pass Rate: 2/5 (40%)

⚠ PARTIAL Clarity over verbosity
Evidence: Some sections verbose without concise summaries. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L243-L431)
Impact: Clarity could suffer under token budget.

✓ PASS Actionable instructions
Evidence: Tasks and acceptance criteria are action-oriented. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L52-L175)

✓ PASS Scannable structure
Evidence: Consistent headings, bullet lists, tables. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L9-L588)

⚠ PARTIAL Token efficiency
Evidence: Long code snippets and repeated constraints increase token usage. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L243-L431)
Impact: Less efficient LLM consumption.

⚠ PARTIAL Unambiguous language
Evidence: Some requirements (e.g., monitoring/alert thresholds) lack specifics. (docs/implementation-artifacts/7-1-checkpoint-auto-save.md#L69)
Impact: Multiple valid implementations possible.

### Step 5: Improvement Recommendations
Pass Rate: N/A (process output, not story content)

➖ N/A Missing essential technical requirements
Evidence: Instruction for reviewer; not story content.

➖ N/A Missing previous story context
Evidence: Instruction for reviewer; not story content.

➖ N/A Missing anti-pattern prevention
Evidence: Instruction for reviewer; not story content.

➖ N/A Missing security or performance requirements
Evidence: Instruction for reviewer; not story content.

➖ N/A Additional architectural guidance
Evidence: Instruction for reviewer; not story content.

➖ N/A More detailed technical specifications
Evidence: Instruction for reviewer; not story content.

➖ N/A Better code reuse opportunities
Evidence: Instruction for reviewer; not story content.

➖ N/A Enhanced testing guidance
Evidence: Instruction for reviewer; not story content.

➖ N/A Performance optimization hints
Evidence: Instruction for reviewer; not story content.

➖ N/A Additional context for complex scenarios
Evidence: Instruction for reviewer; not story content.

➖ N/A Enhanced debugging or development tips
Evidence: Instruction for reviewer; not story content.

➖ N/A Token-efficient phrasing of existing content
Evidence: Instruction for reviewer; not story content.

➖ N/A Clearer structure for LLM processing
Evidence: Instruction for reviewer; not story content.

➖ N/A More actionable and direct instructions
Evidence: Instruction for reviewer; not story content.

➖ N/A Reduced verbosity while maintaining completeness
Evidence: Instruction for reviewer; not story content.

### Competition Success Metrics
Pass Rate: N/A (evaluation rubric, not story content)

➖ N/A Essential technical requirements missing
Evidence: Evaluation rubric for reviewer.

➖ N/A Previous story learnings missing
Evidence: Evaluation rubric for reviewer.

➖ N/A Anti-pattern prevention missing
Evidence: Evaluation rubric for reviewer.

➖ N/A Security/performance requirements missing
Evidence: Evaluation rubric for reviewer.

➖ N/A Architecture guidance needed
Evidence: Evaluation rubric for reviewer.

➖ N/A Technical specification detail needed
Evidence: Evaluation rubric for reviewer.

➖ N/A Code reuse opportunities needed
Evidence: Evaluation rubric for reviewer.

➖ N/A Testing guidance needed
Evidence: Evaluation rubric for reviewer.

➖ N/A Performance/efficiency improvements
Evidence: Evaluation rubric for reviewer.

➖ N/A Development workflow optimizations
Evidence: Evaluation rubric for reviewer.

➖ N/A Additional context for complex scenarios
Evidence: Evaluation rubric for reviewer.

### Interactive Improvement Process
Pass Rate: N/A (process steps, not story content)

➖ N/A Present improvement suggestions template
Evidence: Process instruction for reviewer.

➖ N/A Interactive user selection prompt
Evidence: Process instruction for reviewer.

➖ N/A Apply selected improvements (load story file)
Evidence: Process instruction for reviewer.

➖ N/A Apply selected improvements (no mention of review process)
Evidence: Process instruction for reviewer.

➖ N/A Apply selected improvements (ensure clean final story)
Evidence: Process instruction for reviewer.

➖ N/A Confirmation template after applying changes
Evidence: Process instruction for reviewer.

### Competitive Excellence Mindset
Pass Rate: N/A (meta success criteria, not story content)

➖ N/A Clear technical requirements
Evidence: Meta success criteria for reviewer.

➖ N/A Previous work context
Evidence: Meta success criteria for reviewer.

➖ N/A Anti-pattern prevention
Evidence: Meta success criteria for reviewer.

➖ N/A Comprehensive guidance
Evidence: Meta success criteria for reviewer.

➖ N/A Optimized content structure
Evidence: Meta success criteria for reviewer.

➖ N/A Actionable instructions with no ambiguity
Evidence: Meta success criteria for reviewer.

➖ N/A Efficient information density
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to reinvent existing solutions
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to use wrong approaches/libraries
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to create duplicate functionality
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to miss critical requirements
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to make implementation errors
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to misinterpret requirements (LLM)
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to waste tokens on verbosity (LLM)
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to miss buried info (LLM)
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to get confused by structure (LLM)
Evidence: Meta success criteria for reviewer.

➖ N/A Make it impossible to miss key signals (LLM)
Evidence: Meta success criteria for reviewer.

## Failed Items

1. Deployment and environment patterns missing in story.
2. Previous story intelligence (review feedback, file patterns, testing learnings, problems/solutions) missing.
3. Git history analysis (patterns, dependencies, testing approaches) missing.
4. Latest technical research (breaking changes/best practices) missing.
5. Database schema conflicts with PRD (lineage/parent_id/workspace_id/step).
6. Deployment failures prevention missing.
7. Missing critical signals for LLM (lineage/step granularity).

## Partial Items

- Breaking regression prevention lacks explicit regression checklist.
- UX requirements for reliability/recovery not captured.
- Completion evidence placeholders not filled.
- Workflow variables not resolved in story.
- API contract details incomplete (error/pagination).
- Security requirements beyond permissions not captured.
- Integration patterns for WS/history unification incomplete.
- Previous story learnings not summarized.
- Git Intelligence summary lacks file-level details.
- LLM optimization issues: verbosity, ambiguity, context overload.

## Recommendations

1. Must Fix: Align checkpoints schema with PRD (add `parent_id`, `lineage_type`, step granularity; reconcile `workspace_id` vs `task_id`).
2. Should Improve: Summarize prior story review learnings and git-derived file patterns; add explicit deployment/env assumptions.
3. Consider: Add concise UX recovery expectations and tighten LLM token efficiency by trimming redundant code blocks.

## Adopted Recommendations (Post-Review)

### Must Fix (will be applied to story)
1. Add missing dependency: include `sha2` in `backend/Cargo.toml` to avoid checksum compile failure.
2. Register new modules:
   - `backend/src/domain/models/mod.rs`: `pub mod checkpoint;` and re-export as needed.
   - `backend/src/core/iteration_engine/mod.rs`: `pub mod checkpoint;`.
   - `backend/src/infra/db/repositories/mod.rs`: `pub mod checkpoint_repo;` + `pub use`.
3. Make routing explicit: update `backend/src/api/routes/mod.rs` to export `checkpoints`, and mount routes in `backend/src/main.rs` for:
   - `GET /api/v1/tasks/{task_id}/checkpoints`
   - `GET /api/v1/checkpoints/{checkpoint_id}`
4. Align schema with existing model + PRD lineage: ensure `branch_id`, `parent_id`, `lineage_type` (and optional `branch_description`) are present in Checkpoint DB entity and migration; clarify iteration step mapping.
5. Checksum coverage: include `run_control_state`, `user_guidance`, and lineage fields in checksum calculation.
6. A2 logging completeness: log `correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp`.
7. Clarify AC4 “idle auto-save”: define “no running task” via `RunControlState` and timer start/reset conditions.

### Should Add (recommended, not scope-expanding)
1. Clarify model relationship: specify `CheckpointEntity` (DB) vs `Checkpoint` (domain) mapping and conversion.
2. History source boundary: state that 7.1 saves checkpoint data for recovery; history UI remains `iterations` until Story 7.4 unifies.
3. P1 design reference: add a note that P1 convergence design doc is the source of truth once complete.

### Not Adopted (out of scope / overreach)
1. Persist full `OptimizationContext` (execution_target_config/test_cases/config) into checkpoints — not required by PRD and would duplicate data.
2. Recovery/kill-restart test matrix for NFR5 — belongs to Story 7.2 recovery scope.
3. “WAL test missing” — already included in current test matrix.
4. Add UI-centric fields (pass_rate, patterns_count) — not required for FR52 and would expand scope.
