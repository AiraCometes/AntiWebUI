# 04. テキスト

## 役割分担

| 処理 | 担当 |
|---|---|
| フォント列挙・フォールバック | fontique（parley 経由、Linux では fontconfig 設定を参照） |
| シェイピング・双方向テキスト・行分割 | parley（`quantize = true`） |
| カーソル移動の基本操作（視覚順、単語、行） | parley の `Cursor` / `Selection` |
| **選択モデル（ドキュメント横断）** | AntiWebUI（core） |
| **編集モデル・IME・クリップボード連携** | AntiWebUI（core） |
| グリフのラスタライズ・アトラス | AntiWebUI（wgpu、ラスタライズは swash） |

ロケールは環境変数 `LANG` から取得して parley に渡す（日本語環境では CJK フォールバックが日本語字形になる）。

空の文字列は幅 0 とする。parley は空の文字列を空白 1 つとしてレイアウトする（行の高さを得るため）ので、そのままだと空白の幅（12 px の太字で約 3 px）を取り、並ぶ要素を押しやる。高さは行の高さのまま。

### フォールバックの問い合わせ

fontique はフォールバックのフォントを文字種とロケールの組で記憶するが、記憶するのは決まった組だけで、それ以外（ラテン文字はどのロケールでも対象外）は**整形のたびに fontconfig に問い合わせる**。1 回約 170 µs かかり、`LANG` が設定された環境では整形の時間の 9 割を占めていた。

対象外の組は、その文字種だけの記憶があれば、それを使う。問い合わせの内容もロケールなしで同じなので、結果は変わらない。そこで整形の前に、文字列に含まれる文字種ごとに 1 回だけ、文字種だけの問い合わせをする（`TextSystem::prepare_fallbacks`）。文字種を判定するのは非 ASCII の文字列だけ。

短い文字列の行 1 つを作るときの時間（文字列を整形してレイアウトする分、リリースビルド）。

| `LANG` | 文字列 | 修正前 | 修正後 |
|---|---|---|---|
| `ja_JP.UTF-8` | `row 123` | 176 µs | 10 µs |
| `ja_JP.UTF-8` | `項目 123 の文章です` | 353 µs | 13 µs |
| `en_US.UTF-8` | `row 123` | 176 µs | 11 µs |
| `C`（ロケールなし） | `row 123` | 11 µs | 11 µs |

fontique 0.12.0 の時点の動作で、fontique 側で直ればこの処理は要らなくなる。

### フォールバック（Windows）

- ロケールは `LANG` などが無ければ `GetUserDefaultLocaleName`（`ja-JP` など）にする。Windows は `LANG` を設定しないので、そのままでは漢字が中国語の字形のフォントになる。
- fontique の DirectWrite のフォールバックは、文字種ごとに **1 つのファミリ**しか返さない（文字種の見本の文字列で `IDWriteFontFallback::MapCharacters` を引く）。ラテン文字なら Segoe UI だけで、矢印や図形（`▾` など）は豆腐になる。fontconfig は長い一覧を返すので、Linux では起きない。
- そこで、文字種を準備するとき（`prepare_fallback`）に、システムの 1 つを先に入れさせてから、`Segoe UI Symbol`、`Segoe UI Emoji`、`Cambria Math` を後ろに足す。Chromium も Windows ではこれらに落ちる。

## 影と縁取り

CSS の `text-shadow` と `-webkit-text-stroke`、`paint-order` に当たる。どれも継承する。

```rust
text("見出し")
    .text_strokes(vec![
        TextStroke { width: 6.0, color: Color::WHITE },   // 先頭が上
        TextStroke { width: 16.0, color: Color::BLACK },  // 広いものを後ろに置くと、外側の縁になる
    ])
    .paint_order(PaintOrder::Stroke)              // 縁を文字の下に敷く
    .text_shadow(TextShadow { offset_x: 0.0, offset_y: 4.0, blur: 8.0, color: Color::from_rgba(0x00000099) })
```

- 縁取りの幅は CSS と同じく輪郭の中心から両側に半分ずつ。`PaintOrder::Normal`（既定）では縁が文字の上に乗り、太くすると文字が痩せる。`PaintOrder::Stroke` では文字の下に敷かれ、外側の半分だけが見える。動画のテロップの縁取りはこちら。
- CSS の縁取りは 1 本だけだが、ここでは何本でも持てる（先頭が上）。二重の縁取りが書ける。
- 影は描かれた文字そのもの（縁取りと下線を含む）を 1 色にしてずらしたもの。先頭が上で、文字より下に描く。
- 描く順は、影（最後のものから）、縁取りと文字（`paint_order` の順）、下線と取り消し線。
- 色付きのグリフ（絵文字）には縁取りがつかない。輪郭を持たないため。
- 付くのはテキストのノードだけで、入力欄（`TextEdit`）の文字には付かない。

### 方式

- `GlyphRun` に `stroke`（物理ピクセルの幅、0 は塗り）を持たせた。swash の `Render::style` に `zeno::Stroke` を渡してラスタライズする。角は丸め（`Join::Round`）。文字の輪郭は鋭い角が多く、マイターではとげが出るため。アトラスのキーに幅（1/64 px 単位）を含める。
- 縁取りした文字の描画範囲（`damage::glyph_run_bounds`）は幅の半分だけ広げる。
- ぼかしのない影は、ずらした `GlyphRun` を同じシーンに並べるだけ。ぼかしのある影は、影の文字だけを中身とするレイヤーにして、`Image::content().blur(r)` のフィルタを付ける（[16](16-filters.md) のぼかしと同じ経路）。
- そのレイヤーのキーは、ノードのキーに影の番号を混ぜ、最上位ビット（`NOT_A_NODE`）を立てたもの。ノードのキーは世代が 2^31 を超えない限りこのビットを持たない。入れ子のレイヤーを探す `nested_layers` はこのビットのあるキーを飛ばす。
- 影と縁取りの一覧は `Arc<[T]>` で持つ。継承する性質なので、子のスタイルを作るたびに複製される。`Vec` だと空でなければ毎回確保が要る。既定値は共有の空の一覧で、既定のスタイルを作るときにも確保しない。
- `Style` が大きくなると、デバッグビルドのスタックが増える。ビルダーの呼び出し（`div().w(..).h(..)`）ごとに `Style` の複製がスタックに積まれるため。`anti-webui-markdown` の `links_go_to_headings_or_out` は、この変更の前から既定のスタック（2 MiB）の 2000 KiB 近くを使っており、今はぎりぎり収まっている。

## フォントの同梱

アプリに同梱したフォントを登録できる。CSS の `@font-face` に相当する。システムのフォントは登録しなくても名前で使える。

```rust
static FONT: &[u8] = include_bytes!("../fonts/RobotoFlex.ttf");

register_font(FONT)?;                    // ファイルに書かれた名前で使えるようになる
register_font_as(FONT, "Brand")?;        // 名前を決めて登録する

text("保存").font_family("Roboto Flex, sans-serif")
```

- 戻り値は登録されたファミリー名の一覧。フォントとして読めないデータは `NotAFont` になる。
- 登録は fontique の `register_fonts` に渡すだけで、整形とラスタライズはシステムのフォントと同じ経路を通る。可変フォントの太さも `font_weight` で効く。ほかの軸は [17](17-font-axes.md)。
- `register_font_as` で、まだどのテキストも求めておらず、同じ名前のファミリもないフォントを登録したときは、レイアウトをやり直さない。それ以外の登録では、フォールバックしていたテキストが新しいフォントを使えるように全部やり直す。
- 表示中の文字も新しいフォントで整形し直す。整形のキャッシュキーに登録回数を含め、全ノードのレイアウトを無効にする。
- フォントは UI スレッドのテキストシステムに属する。
- 埋め込んだ分だけバイナリが大きくなる（Roboto Flex で約 1.8 MB）。

## 選択モデル

ブラウザと同じく **ドキュメント全体で 1 つ** の選択を持つ。

```rust
struct TextPoint { node: NodeId, offset: usize /* バイト位置（書記素境界） */ }
struct DocSelection { anchor: TextPoint, focus: TextPoint }
```

- 文書順はレイアウトのたびにツリーを前順走査して `doc_order: u32` を振り、`(doc_order, offset)` で比較する。
- 選択可能かどうかはスタイルの `user_select`（既定: テキストは `Text`、ボタンなどは `None`）で決まり、継承する。
- 入力欄（TextEdit）は独自の選択を持つ（ブラウザの `<input>` と同じ）。

### ヒットテスト（最も近いテキスト位置）

ドラッグ中のポインタが要素の外・行間・余白にあっても、ブラウザのように自然に選択できるよう、次の順で最も近い位置を求める。

1. 選択可能なテキストノードのうち、ポインタを含む行ボックスを持つものがあれば、その中で `Cursor::from_point`。
2. なければ、各テキストノードの可視矩形との距離を「縦方向の距離 → 横方向の距離」の辞書式順序で比較し、最も近いノードを選ぶ。ポインタをそのノードの矩形内にクランプしてから `Cursor::from_point`。
3. 選択開始ノードより文書順で前に戻ることも、離れた要素にまたがることも自由。

### ダブル / トリプルクリック

- ダブルクリック: 単語選択（parley の単語境界、日本語は辞書ベース）。ドラッグすると単語単位で拡張。
- トリプルクリック: 段落（ハードラインブレーク区切り）選択。ドラッグすると段落単位で拡張。
- 選択範囲の上を押して動かすと、選択した文字を運ぶ（[23](23-drag-and-drop.md)）。

### ハイライト描画

各テキストノードについて、選択範囲と重なる部分の矩形を `Selection::geometry` で求める。y 方向は行ボックス（整数）にスナップするため、複数行のハイライトは隙間も重なりもなく接する。改行をまたぐ場合は行末に小さな余白を加える（ブラウザ同様）。

### コピー

文書順に各ノードの部分文字列を連結する。ノード間には、次のノードが視覚的に新しい行で始まるなら `\n`、同じ行なら ` ` を挟む。選択が確定したら Wayland の primary selection にも設定する。

## 編集モデル（TextEdit）

```rust
struct EditState {
    text: String,
    selection: (usize, usize),        // anchor, focus（バイト位置）
    preedit: Option<Preedit>,          // IME の未確定文字列
    scroll: (i32, i32),                // 内部スクロール（物理 px）
    undo: UndoStack,
}
```

- 単一行（`text_input`）と複数行（`text_area`）。
- キー操作: 矢印（視覚順・Shift で拡張・Ctrl で単語）、Home/End、上下（水平位置を保持）、Backspace/Delete（書記素・Ctrl で単語）、Ctrl+A/C/X/V/Z/Shift+Z、Enter（複数行では改行、単一行では submit イベント）。
- マウス: クリック・ドラッグ・ダブル/トリプルクリック・Shift+クリック、中クリックで primary selection を貼り付け。
- キャレットが常に見えるよう内部スクロールを調整する。キャレットは 500ms で点滅し、入力のたびにリセットする。
- `Signal<String>` と双方向バインドできる（編集 → signal 書き込み、signal の外部変更 → テキスト更新）。
- `.purpose(ImePurpose::Password)` の入力欄はパスワード用。1 文字（書記素でなく char）を `•` 1 つとしてレイアウトし、未確定文字列も伏せる。キャレット・選択・ヒットテストは `to_display` / `to_model` で元の文字列と `•` の並びを行き来する。ダブルクリックは全体を選ぶ（伏せた文字列には単語がないので）。コピーと切り取り、primary selection、ドラッグでは中身を外に出さない。

## IME（text-input-v3）

- 未確定文字列（preedit）は、キャレット位置に **一時的に挿入した状態でレイアウト** し、下線を引いて描画する。IME が指定する preedit 内カーソル範囲も表示する。
- preedit 開始時に選択範囲があれば削除する。
- `done` イベントで次の順に適用する: (1) 既存 preedit を除去 (2) `delete_surrounding_text` (3) `commit_string` を挿入 (4) 新しい preedit を設定。適用後、周辺テキスト・カーソル矩形を送って `commit`。
- 周辺テキスト（surrounding text）はキャレット周辺の最大 4000 バイト。
- カーソル矩形は論理座標に変換して送る（[01](01-pixel-model.md) R7）。

core は IME 状態を `ImeRequest { enabled, surrounding, cursor_rect, content_type }` として出力し、プラットフォーム層がプロトコルに変換する。

## フォントの代役

`font-family` のリストにある Arial、Helvetica、Times New Roman、Times、Courier New、Courier、Calibri、Cambria には、同じメトリクスのフォント（Liberation、Arimo、Tinos、Cousine、Carlito、Caladea）を直後に足す。fontconfig が Chromium に与える別名と同じ考え方である（[25](25-svg.md)）。
