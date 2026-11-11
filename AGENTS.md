# Repository Guidelines

## 项目结构

- `core/`：Rust 协议层 `mhub-core`，不依赖 Tauri，可在无窗口、无外设的机器上测试。
- `src-tauri/`：Tauri 2 外壳，命令、窗口与应用状态。
- `src/`：Svelte 5 前端；`app.css` 是设计系统，`routes/` 是五个页面，`lib/` 是状态与 API。
- `docs/`：协议与设计说明；`screenshots/`：README 用截图。

## 提交信息

- 使用英文 Conventional Commits：`feat(dpi): ...`、`fix(protocol): ...`、`docs: ...`。
- 标题用祈使句、简短、描述清楚改了什么；正文说明原因与验证方式。

## 代码约定

- Rust：`cargo fmt`；`cargo clippy -p mhub-core --all-targets -- -D warnings`。
- 前端：`pnpm check`；提交前跑 `pnpm build`。
- 用户可见文案用中文；错误码和诊断信息保持英文。
- 组件不要新增第二种「选中态」；需要更明确时用 `--glass-state-armed`。
- 不引入 Tailwind、shadcn、组件库或第二套图标集。

## 安全边界（重要）

- **没有实机验证的型号不得打开写入。** 未知 PID 只列出识别信息。
- 所有写入参数在 Rust 侧再次校验，不信任前端。
- 保持界面未开放字段（`sensorFlags` / `highspeedMode` / `sensorAngle`）原样读写。
- 新增型号时同步更新 `docs/protocol.md`、README 支持表和测试。

## 验证

```bash
cargo test -p mhub-core
cargo clippy -p mhub-core --all-targets -- -D warnings
pnpm check
pnpm build
pnpm tauri dev          # 有硬件时
MHUB_DEMO=1 pnpm tauri dev
```
