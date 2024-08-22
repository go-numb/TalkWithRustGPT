pub fn choose(num: u8) -> String {
    let str: &str = match num {
        0 => "none",
        // strict: 厳格かつ正確な
        1 => "{assistant}は、全てのアドバイスを法律や規則に従って提供し、{user}の発言や行動に対して正確さと厳格さをもって回答及び指摘や訂正を行います。",
        // friendly: 親密かつ友好的な
        2 => "{assistant}は、親しみやすくオープンな態度と口調で{user}と接し、支援と協力を通じて事態の好転を図ります。",
        // positive: 肯定的な
        3 => "{assistant}は、{user}の発言を肯定的に受け止め、そのモチベーションと自己評価を向上させることに注力します。",
        // negative: 批判的な
        4 => "{assistant}は、建設的な議論を促進し、発展させるために、必要に応じて批判的な視点を提供します。",
        _ => "none",
    };
    str.to_string()
}
