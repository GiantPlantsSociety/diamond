pub mod ast;
mod parser;

pub use ast::*;
pub use std::convert::TryFrom;
pub use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let _ex: ast::Expression = "template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false,none=none)|aliasByNode(1)|movingAverage(\"5min\")".parse().unwrap();
    }

    #[test]
    fn pathword_to_regex() {
        assert_eq!(
            PathExpression::from_str("just_a_metric").unwrap().0[0]
                .to_regex_pattern()
                .unwrap(),
            "^(just_a_metric)$"
        );
        assert_eq!(
            PathExpression::from_str("host*.cpu").unwrap().0[0]
                .to_regex_pattern()
                .unwrap(),
            "^(host)(.*?)$"
        );
        assert_eq!(
            PathExpression::from_str("east-1[abcd].app.swap_free")
                .unwrap()
                .0[0]
                .to_regex_pattern()
                .unwrap(),
            "^(east\\-1)(a|b|c|d)$"
        );
        assert_eq!(
            PathExpression::from_str("east-[1-3][a-d].app.swap_free")
                .unwrap()
                .0[0]
                .to_regex_pattern()
                .unwrap(),
            "^(east\\-)(1|2|3)(a|b|c|d)$"
        );
        assert_eq!(
            PathExpression::from_str("region-{ohio,virginia}.app.swap_free")
                .unwrap()
                .0[0]
                .to_regex_pattern()
                .unwrap(),
            "^(region\\-)(ohio|virginia)$"
        );
    }
}
