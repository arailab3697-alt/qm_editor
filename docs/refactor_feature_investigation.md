# リファクタ・機能追加に向けた調査メモ（QM Editor）

## 1. 調査目的
- 現行実装の把握（Frontend/Backend/AI連携）。
- リファクタ候補の抽出（保守性・安全性・拡張性）。
- 次の機能追加に向けた実装ポイント整理。

## 2. 現状アーキテクチャ要約

### 2.1 全体
- React + Zustand のフロントエンドが Tauri `invoke` で Rust バックエンドを呼び出す構成。
- 状態更新は `Command` を backend reducer に渡して反映する一方向モデル。

### 2.2 フロントエンド
- 状態管理は `state`, `past`（Undo履歴）、`dispatchCommand`, `applyCommands` を中心に構成。
- UI は `App.tsx` に主要機能（Viewer, Import, Calculationフォーム, Geometry編集, AIパネル）が集約されており、単一ファイルが大きい。
- Gaussianプレビュー・バリデーションは state 変更時に毎回 backend invoke を行っている。

### 2.3 バックエンド
- `lib.rs` に Tauri command が集約され、`reduce`, `parse`, `validation`, `ai_commands` などへ委譲。
- `reducer.rs` は多機能（計算条件更新・分子編集・テンプレート配置・フラグメント置換・選択状態更新）を一つの `reduce` に集約。
- `apply_commands` は command 配列を逐次 fold で適用。

### 2.4 AIコマンド生成
- ルールベース（`propose_commands_by_rules`）を先に試し、失敗時にLLM連携する設計。
- ルールベースは「`set` で始まる」厳格条件、トークン解析、重複排除、選択原子依存の幾何コマンド推論を実装。

## 3. 調査で見えた改善余地（リファクタ候補）

### 3.1 型定義の重複管理
- TypeScript 側 `Command` と Rust 側 `Command` は実質ミラー管理。
- 追加コマンド時に両者修正が必要で差分漏れリスクが高い。

**提案**
- スキーマ駆動（例: JSON Schema / OpenAPI-like / codegen）導入、または最低限 shared contract ドキュメントと整合テストを追加。

### 3.1.1 「スキーマ駆動」の具体像（本プロジェクト想定）
- 単一の契約ファイル（例: `contracts/command.schema.json`）を真実源（SoT）として管理する。
- `Command`, `AiResult`, `AppState` のうち、まず変更頻度が高い `Command` から適用する。

**想定フロー**
1. schema に command discriminator（`type`）と payload を定義。
2. schema から TypeScript 型を自動生成（`src/generated/commands.ts`）。
3. 同じ schema から Rust 側の型を自動生成、または serde で厳密バリデーション。
4. frontend/backend ともに hand-written 型を generated 型へ段階置換。
5. CI で「schema変更時に generated 差分が未コミットなら fail」を追加。

**最小構成（段階導入）**
- Phase 1: `Command` のみ schema 化（既存 `chemicalSpec` は手動維持）。
- Phase 2: `CalculationSpec` / `ValidationMessage` へ拡張。
- Phase 3: `AppState` 全体へ拡張（必要なら UI state は別schema）。

**メリット**
- TS/Rust の型ズレを実装時ではなく生成時・CI時に検出できる。
- 新コマンド追加時の修正点が schema 中心に集約され、漏れを減らせる。
- AI JSON 出力の受け口を schema 検証でき、異常系の扱いを統一しやすい。

**注意点**
- いきなり `AppState` 全量を schema 化すると運用コストが高い。
- 先に `Command` へ限定し、生成物のレビュー負荷とビルド時間を計測してから拡張する。


### 3.2 `App.tsx` の肥大化
- UIコンポーネント群が 1 ファイル集中で、責務分離と再利用性が低い。

**提案**
- `src/features/*` へ分割（viewer, calculation, molecule-editor, geometry-editor, ai-assistant）。
- Tauri invoke ラッパーを `src/services/tauriApi.ts` に分離して副作用を集約。

### 3.3 Reducerの責務過多
- `reduce` が多機能化し、個別ロジックの見通しとテスト粒度が悪化。

**提案**
- command ハンドラをモジュール分割（`calc_handlers`, `molecule_handlers`, `selection_handlers`, `fragment_handlers`）。
- `match` から関数ディスパッチへ段階移行し、ユニットテストを機能単位化。

### 3.4 AIルールベースの入力耐性
- 先頭 `set` 強制は誤入力耐性が低い（日本語/自然文で失敗しやすい）。
- `parse_kv_by_rules` が token 単位のため、`charge = -1` など表記揺れ対応に限界。

**提案**
- 前処理で `set` 不在時も intent 判定を試す。
- 正規化レイヤ追加（全角空白、記号揺れ、日本語キーワード対応）。
- ルール失敗理由を structured error 化して UI へ返す。

### 3.5 バリデーションの拡張性
- 現在は主に原子数、Multiplicity、電荷/スピンのパリティ整合性チェック。

**提案**
- ルールID付きメッセージ（`VAL001` 形式）を導入。
- Molecule構造チェック（孤立原子、異常結合距離、結合次数と価数整合）を段階追加。

## 4. 優先順位に基づくリファクタ方針（更新版）

ユーザー要件に基づき、優先順位は次の通りで進める。

1. **バックエンド機能の拡充（Command中心）**
2. **リファクタ（保守性改善）**
3. **バリデーション強化**
4. **UI改善**

また、ルールベース入力はデバッグ用途として扱い、**日本語拡張を含む機能拡充は優先対象外**とする。

### 4.1 バックエンド機能拡充（最優先）
- `Command` の表現力を先に拡張し、フロントより先にドメイン能力を上げる。
- 既存 reducer/geometry/fragment 系の資産を活かし、追加 command を段階導入する。

**具体案**
- Fragment/Template 操作 command の仕様明確化（前提条件・失敗時挙動・戻り値）。
- 複合操作 command（例: 複数原子への一括適用）を検討。
- command 追加時に reducer テストを同時追加（仕様凍結）。

### 4.2 リファクタ（2nd）
- backend の command dispatch を機能別ハンドラへ分割し、`reduce` の責務を縮小。
- schema-driven は **Command契約の安定化** を主目的として継続する（UI先行ではなく backend 先行）。

**具体案**
- `reducer.rs` の handler 分割（calculation / molecule / selection / fragment）。
- `Command` 契約の schema 化 + 生成物整合 CI。
- `apply_commands` のエラーハンドリング方針（中断/継続）を明文化。

### 4.3 バリデーション強化（3rd）
- 既存 parity チェックに加え、backend 側のドメイン妥当性検証を拡張。

**具体案**
- ルールID付きメッセージ（`VALxxx`）導入。
- 結合距離異常、価数整合、孤立原子などを段階導入。
- command 適用前後での検証ポイントを定義（将来的に strict mode を検討）。

### 4.4 UI改善（4th）
- UIは backend の機能追加・契約安定化後に追従実装する。

**具体案**
- `App.tsx` 分割、`tauriApi` ラッパー化。
- 追加 command を操作する最小UIを順次追加。
- AI/ルールベースのUX強化は現時点では優先しない。

## 5. 推奨実行計画（優先順位反映）

### Sprint 1: Backend Command 拡充
- Command 仕様の棚卸しと拡張項目確定。
- reducer/geometry/fragment の不足機能を command 単位で実装。
- command ごとのユニットテスト追加。

### Sprint 2: Backend リファクタ
- `reduce` の handler 分割。
- `Command` schema 導入（Phase 1: Command のみ）。
- CI に生成物差分チェック追加。

### Sprint 3: Validation 強化
- `VALxxx` 形式のメッセージ体系導入。
- 構造検証ルール追加（距離・価数・孤立原子）。
- 検証失敗時の扱い（warning/error）基準整理。

### Sprint 4: UI追従
- `App.tsx` 分割と API 呼び出し整理。
- 新規 command を利用する編集UIを段階追加。
- 表示改善は backend 安定後に最小限から実施。


## 6. 実装時の注意点
- 状態の唯一ソースは `AppState` を維持し、Gaussian文字列の直接編集を避ける。
- AIは command 提案に限定し、直接 mutate させない原則を維持する。
- Frontend/Backend の型ズレを CI で検知する仕組みを優先導入する。

## 7. 調査対象ファイル
- `README.md`
- `specs/specs.md`
- `src/domain/chemicalSpec.ts`
- `src/domain/commands.ts`
- `src/app/store.ts`
- `src/App.tsx`
- `src-tauri/src/lib.rs`
- `src-tauri/src/reducer.rs`
- `src-tauri/src/ai_commands.rs`
- `src-tauri/src/validation.rs`
- `src-tauri/src/parser.rs`
