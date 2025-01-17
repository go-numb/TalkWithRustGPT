// 定数を設定し、公開する
export const prompts = {
   self_analysis: "From all of our interactions what is one thing that you can tell me about myself that I may not know about myself. 日本語で出力してください。",
   matome: '会話履歴を復習・知識定着のためにまとめて出力し、事後検索のための#キーワードでタグを付与してください。',
   save: `会話履歴の要約と引き継ぎプロンプト作成指示：

1. 目的：
   - 現在のユーザーの状況と学習進捗を正確に把握し、後任者が効率的に継続サポートできるようにする

2. 要約内容：
   a) ユーザーのバックグラウンド（職業、興味、目標など）
   b) 討議されたトピックと主要な学習ポイント
   c) ユーザーの理解度と習熟レベル
   d) 未解決の質問や懸念事項

3. 引き継ぎ情報の形式：
   - 簡潔で構造化された箇条書き形式
   - 重要なキーワードや概念を強調

4. 後任者への指示：
   a) 重複を避けるべき内容
   b) 深掘りや拡張が必要な領域
   c) 推奨される次のステップや学習方向性

5. 出力形式：
   「前任者から後任者への引き継ぎ：[日付]
   
   1. ユーザープロファイル：
   2. 学習進捗：
   3. 重要ポイント：
   4. 次のステップ：
   5. 注意事項：」

この形式に沿って、現在の会話履歴を分析し、効果的な引き継ぎプロンプトを作成してください。`,
}

interface Value {
   label: string;
   value: string;
};

export const prompts_list: Value[] = [
   { label: "None", value: "0" },
   { label: "[system] 厳格で正確な", value: "1" },
   { label: "[system] 肯定的な", value: "3" },
   { label: "[system] 批判的な", value: "4" },
   { label: "[user] 自己分析", value: prompts.self_analysis },
   { label: "[user] まとめ", value: prompts.matome },
   { label: "[user] 引き継ぎ", value: prompts.save },
];