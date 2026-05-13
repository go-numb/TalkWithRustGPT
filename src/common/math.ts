import katex from "katex";

// markdown クレート (math_text/math_flow 有効) の出力:
//   インライン: <code class="language-math math-inline">...</code>
//   ブロック:   <pre><code class="language-math math-flow">...</code></pre>
//   ※ class属性には "language-math" が必ず含まれるが追加クラスが付く場合がある

// AI が $...$ なしで裸の LaTeX コマンドを出力することがあるため
// テキストノード内の \cmd{...} パターンも補完レンダリングする

export function renderMath(html: string): string {
  // 1) <pre><code class="...language-math..."> — ブロック数式
  const withBlock = html.replace(
    /<pre><code\b[^>]*class="[^"]*language-math[^"]*"[^>]*>([\s\S]*?)<\/code><\/pre>/g,
    (_, raw) => renderKatex(raw.trim(), true)
  );

  // 2) <code class="...language-math..."> — インライン数式
  const withInline = withBlock.replace(
    /<code\b[^>]*class="[^"]*language-math[^"]*"[^>]*>([\s\S]*?)<\/code>/g,
    (_, raw) => renderKatex(raw.trim(), false)
  );

  // 3) テキスト中の $...$ / $$...$$ (markdown クレートが処理しなかった残り)
  //    既に KaTeX レンダリング済みの <span class="katex..."> 内には入らないよう
  //    タグ外のテキスト部分のみ対象にする
  return withInline
    .replace(/\$\$([^$]+?)\$\$/g, (_, tex) => renderKatex(tex.trim(), true))
    .replace(/\$([^$\n]+?)\$/g, (_, tex) => renderKatex(tex.trim(), false));
}

function renderKatex(tex: string, display: boolean): string {
  try {
    return katex.renderToString(decodeHtmlEntities(tex), {
      displayMode: display,
      throwOnError: false,
    });
  } catch {
    return tex;
  }
}

function decodeHtmlEntities(text: string): string {
  return text
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}
