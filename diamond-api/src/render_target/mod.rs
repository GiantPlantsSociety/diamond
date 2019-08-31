pub mod ast;
mod parser;

pub use ast::*;
pub use std::convert::TryFrom;
pub use std::str::FromStr;

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let _ex: ast::Expression = "template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false,none=none)|aliasByNode(1)|movingAverage(\"5min\")".parse().unwrap();
    }

    #[test]
    fn test_from_bytes() {
        let _ex = ast::Expression::try_from(&b"template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false,none=none)|aliasByNode(1)|movingAverage(\"5min\")"[..]).unwrap();
    }
}
*/
