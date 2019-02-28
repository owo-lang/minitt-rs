use crate::syntax::*;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
/// The name stands for "Mini-TT's Parser"
struct MiniParser;

type Tok<'a> = Pair<'a, Rule>;

/// Parse a string into an optional expression
/// ```
/// entry = { expression }
/// ```
pub fn parse_str(input: &str) -> Result<Expression, String> {
    Ok(rule_to_expression(
        MiniParser::parse(Rule::expression, input)
            .map_err(|err| format!("Parse failed at: {}", err).to_string())?
            .next()
            .unwrap(),
    ))
}

/// ```
/// expression =
///  { declaration
///  | application
///  | first
///  | second
///  | pair
///  | atom
///  | void
///  }
/// ```
fn rule_to_expression(rules: Tok) -> Expression {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::declaration => {
            let mut inner = the_rule.into_inner();
            let let_or_rec_rule = inner.next().unwrap();
            assert_eq!(let_or_rec_rule.as_rule(), Rule::let_or_rec);
            let rec = match let_or_rec_rule.as_str() {
                "let" => false,
                "rec" => true,
                _ => unreachable!(),
            };
            let name_rule = inner.next().unwrap();
            assert_eq!(name_rule.as_rule(), Rule::identifier);
            // TODO parse pattern
            let name = identifier_to_name(name_rule);
            let signature_rule = inner.next().unwrap();
            assert_eq!(signature_rule.as_rule(), Rule::expression);
            let signature = expression!();
            let body_rule = inner.next().unwrap();
            assert_eq!(body_rule.as_rule(), Rule::expression);
            let body = rule_to_expression(body_rule);
            let rest_rule = inner.next().unwrap();
            assert_eq!(rest_rule.as_rule(), Rule::expression);
            let rest = rule_to_expression(rest_rule);
            let declaration = if rec {
                Declaration::Recursive(Pattern::Var(name), signature, body)
            } else {
                Declaration::Simple(Pattern::Var(name), signature, body)
            };
            assert_eq!(inner.next(), None);
            Expression::Declaration(Box::new(declaration), Box::new(rest))
        }
        Rule::variable => {
            let mut inner = the_rule.into_inner();
            let name_rule = inner.next().unwrap();
            assert_eq!(name_rule.as_rule(), Rule::identifier);
            let name = identifier_to_name(name_rule);
            assert_eq!(inner.next(), None);
            Expression::Var(name)
        }
        // Rule::application => {}
        // Rule::first => {}
        // Rule::second => {}
        // Rule::pair => {}
        // Rule::atom => {}
        Rule::void => Expression::Void,
        _ => panic!("{}", the_rule),
    }
}

fn identifier_to_name(rule: Tok) -> String {
    rule.as_span().as_str().to_string()
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_str;

    #[test]
    fn simple_parse() {
        println!("{}", parse_str("let f : k = e;\n").unwrap());
    }
}
