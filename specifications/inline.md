# Inline Layout Module

CSS2: https://www.w3.org/TR/CSS21/box.html#box-model
CSS3: https://drafts.csswg.org/css-inline-3/

## Positioning

- `containing block`のtopから初めて水平方向に次々に配置される
- 水平方向のmargin, border, paddingが適用される
- 垂直方向の整列も可能
- lineを含む矩形は`line box`と呼ばれる
- `line box`のwidthは`containing block` or `float`の存在によって決まる
- `line box`の高さは[line-height](#line-height-calculation)によって決まる
- `line box`は含まれる全てのboxにとって十分な高さになる。(中身によって高さが変わる)
- When the height of a box B is less than the height of the line box containing it, the vertical alignment of B within the line box is determined by the 'vertical-align' property.(中身のコンテンツが`containing block`よりも小さい場合は`vertical-align`によって配置が決まる)
- 複数の`inline-level box`が一つの`line box`内に水平に配置できない場合、2つ以上の垂直に積み上げられた(`vertically-stacked`)`line box`に配分される
- 段落は垂直方向に積み上げられる。他の場所で指定されていない限り、重なったり、分離することはない。
- 一般的に`inline box`の`width`は同じですが、`float`により`width`が減少している場合、`width`が異なる場合がある
- `inline box`の`height`は一般的に異なります(例えば、一つのラインに`img`が含まれている場合)
- `inline-level box`の`width`の合計が`containing box`の`width`よりも小さい場合、`text-align`によって水平方向の配置が決まる
- もしpropertyが`justify`を持っていれば、スペースと文字を引き伸ばす(`inline-block`と`inline-table`はのぞく)
- `inline box`が`line box`の幅を超える時は、いくつかの`box`に区切って、それらの`box`を複数行に渡って`line box`を配置する
  - もし`inline box`が単一の単語を含んでいたり、言語の特定の改行ルールが無効になっていたり、`white-space`の値が`nowrap`または`pre`になっている場合は、`line box`をはみ出す
  - `line box`が分割される時、見た目に影響を与えません
  - `bidirectional text processing`のため、`inline box`は同じ`line box`内でいくつかの`box`に分割されるかもしれない
    - `bidirectional text processing` ... 双方向テキスト処理。ヘブライ語などの右から左に読むような言語をサポートする場合に必要。
- `line box`
  - `line box`は`inline formatting context`の内部で`inline-level content`を包含するために必要に応じて作られる
  - `line box`は`text, white-space, inline-element, non-zero margin, padding, border, img, inline-block`を含まない
        - 内部の要素の`positioning`の目的で使うため、zero-heightとして扱い、さらにその他の目的では存在していないものとして扱う

```html
<!-- line box example -->
<P>Several <EM>emphasized words</EM> appear
<STRONG>in this</STRONG> sentence, dear.</P>
```

上記の例では、`P`は5つの`inline box`を含む`block box`を生成している

- `Anonymous` ... "Several"
- `EM` ... "emphasized words"
- `Anonymous` ... "appear"
- `STRONG` ... "in this"
- `Anonymous` ... "sentence, dear."

これらをformatするために、`line box`を生成する。
今回の例では、P要素用に生成されたboxが`line box`のための`contain block`を確立する。もし`containing block`が十分に広い場合、全ての`inline box`は一つの`*line* box`に納る。

*Several emphasized words appear in this sentence, dear.*

もしそうでないなら、`inline box`は分割され、複数行にわたって配置される。これらの一行一行が`line box`であり、今回のケースでは2つの`line box`に分割されていることになる。

*Several emphasized words appear*
*in this sentence, dear.*

または次のようになる。

*Several emphasized*
*words appear in this*
*sentence, dear.*

上記の例では、`EM box`が2つに分割されている(これらを`split1`, `split2`と呼ぶ)。
`margins`, `borders`, `padding`, または`text decorations`は`split1`の後または`split2`の前で視覚的な変化を起こさない。

つまり、
- `split1`は`margin`の`top`、`left`、にスタイルがあたり、`split2`は`margin`の`right`、`bottom`にスペースが当たる
- `split1`は`padding`と`border`の`top`、`left`、`bottom`にスタイルがあたり、`split2`は`margin`の`top`、`right`、`bottom`にスペースが当たる

## line height calculation

1. `replaced element`、`inline-block element`、`inline-table element`の場合、高さはmargin boxによって決まる。`inline box`の場合は、`line-height`によって決まる
2. `inline-level box`は`vertical-align`によって垂直方向に整列される。
3. `line box`の高さは、boxのtopとbottomの距離

#### Leading and Half Leading

- CSSは全てのfontは特徴的なbaselineの上の高さ(`Ascendent`)とbaselineの下の深さ(`Descendent`)を指定するfont metricsを持つと過程する
- Aを高さとし、Dを深さとした場合、`AD = A + D`と定義する。この距離はtopからbottomへの距離である。
- leadingを求めるには、`L = line-height - AD`とする
- さらに`A' = A - L / 2`, `D' = D - L / 2`とすることで合計の高さと深さを求められる。これらを足し合わせることで高さがもとまる。
- `A`と`D`は`OpenType`または`TrueType`のfontから`Ascendent`・`Descendent`を取得することで実装することができる。
  - `Ascendent`は文字の上半分、`Descendent`は文字の下半分のこと。[Java の Font 周りの比較的ディープな話(前編)](https://www.cresco.co.jp/blog/entry/91/)がわかりやすかった。
  - icedでは文字の左上を起点に座標を決めているっぽいのでこれらは必要なさそう？(単純に文字の高さを求めたい)
- `line-height`によって指定された高さが`containing box`よりも小さい場合、`background-color`などがはみ出る(これはまだよくわかってない)
> Although margins, borders, and padding of non-replaced elements do not enter into the line box calculation, they are still rendered around inline boxes. This means that if the height specified by 'line-height' is less than the content height of contained boxes, backgrounds and colors of padding and borders may "bleed" into adjoining line boxes. User agents should render the boxes in document order. This will cause the borders on subsequent lines to paint over the borders and text of previous lines.
- `line-height`の値には、line-boxの計算値が指定される

### Leadingの実装

- `servo/font-kit`の[Font::metrics(&self)](https://docs.rs/font-kit/0.10.0/font_kit/loaders/freetype/struct.Font.html#method.metrics)を使えば、`ascendent`と`descendent`を求められそう。
  - `iced`の実装例(https://github.com/hecrj/iced/blob/master/graphics/src/font/source.rs)
- またここで取得したfontデータは直接`iced_native::Text`に渡したい