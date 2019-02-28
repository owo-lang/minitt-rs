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
    Ok(expression_to_expression(
        MiniParser::parse(Rule::expression, input)
            .map_err(|err| format!("Parse failed at: {}", err).to_string())?
            .next()
            .unwrap(),
    ))
}

macro_rules! next_rule {
    ($inner:expr, $rule_name:ident, $function:ident) => {{
        let token = $inner.next().unwrap();
        assert_eq!(token.as_rule(), Rule::$rule_name);
        $function(token)
    }};
}

macro_rules! next_expression {
    ($inner:expr) => {
        next_rule!($inner, expression, expression_to_expression)
    };
}

macro_rules! next_atom {
    ($inner:expr) => {
        next_rule!($inner, atom, atom_to_expression)
    };
}

macro_rules! next_identifier {
    ($inner:expr) => {{
        next_rule!($inner, identifier, identifier_to_name)
    }};
}

macro_rules! end_of_rule {
    ($inner:expr) => {
        assert_eq!($inner.next(), None)
    };
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
fn expression_to_expression(rules: Tok) -> Expression {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::declaration => declaration_to_expression(the_rule),
        Rule::application => application_to_expression(the_rule),
        Rule::first => first_to_expression(the_rule),
        Rule::second => second_to_expression(the_rule),
        Rule::pair => pair_to_expression(the_rule),
        Rule::atom => atom_to_expression(the_rule),
        Rule::void => Expression::Void,
        _ => unreachable!(),
    }
}

/// ```
/// first = { atom ~ ".1" }
/// ```
fn first_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let pair = next_atom!(inner);
    end_of_rule!(inner);
    Expression::First(Box::new(pair))
}

/// ```
/// second = { atom ~ ".2" }
/// ```
fn second_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let pair = next_atom!(inner);
    end_of_rule!(inner);
    Expression::Second(Box::new(pair))
}

/// ```
/// pair = { atom ~ "," ~ expression }
/// ```
fn pair_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let first = next_atom!(inner);
    let second = next_expression!(inner);
    end_of_rule!(inner);
    Expression::Pair(Box::new(first), Box::new(second))
}

/// ```
/// application = { atom ~ expression }
/// ```
fn application_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let function = next_atom!(inner);
    let argument = next_expression!(inner);
    end_of_rule!(inner);
    Expression::Application(Box::new(function), Box::new(argument))
}

/// ```
/// declaration =
///  { let_or_rec?
///  ~ identifier
///  ~ ":" ~ expression
///  ~ "=" ~ expression
///  ~ ";" ~ expression
///  }
/// ```
fn declaration_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let let_or_rec_rule = inner.next().unwrap();
    let rec = match let_or_rec_rule.as_str() {
        "let" => false,
        "rec" => true,
        _ => unreachable!(),
    };
    // TODO: parse as pattern
    let name = next_identifier!(inner);
    let signature = next_expression!(inner);
    let body = next_expression!(inner);
    let rest = next_expression!(inner);
    let declaration = if rec {
        Declaration::Recursive(Pattern::Var(name), signature, body)
    } else {
        Declaration::Simple(Pattern::Var(name), signature, body)
    };
    end_of_rule!(inner);
    Expression::Declaration(Box::new(declaration), Box::new(rest))
}

/// ```
/// atom =
///   { constructor
///   | variable
///   | function
///   | sum
///   | one
///   | unit
///   | pi_type
///   | sigma_type
///   | lambda_expression
///   | universe
///   | "(" ~ expression ~ ")"
///   }
/// ```
fn atom_to_expression(rules: Tok) -> Expression {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::constructor => constructor_to_expression(the_rule),
        Rule::variable => variable_to_expression(the_rule),
        Rule::one => Expression::One,
        Rule::unit => Expression::Unit,
        Rule::universe => Expression::Type,
        Rule::expression => expression_to_expression(the_rule),
        _ => panic!("{}", the_rule),
    }
}

/// ```
/// constructor_name = @{ ASCII_ALPHA_UPPER ~ identifier? }
/// constructor = { constructor_name ~ expression }
/// ```
fn constructor_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let constructor = next_rule!(inner, constructor_name, identifier_to_name);
    let argument = next_expression!(inner);
    end_of_rule!(inner);
    Expression::Constructor(constructor, Box::new(argument))
}

/// ```
/// variable = { identifier }
/// ```
fn variable_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let name = next_identifier!(inner);
    end_of_rule!(inner);
    Expression::Var(name)
}

/// ```
/// identifier = @{ !"let" ~ !"rec" ~ !"0" ~ !"1" ~ character+ }
/// ```
fn identifier_to_name(rule: Tok) -> String {
    rule.as_span().as_str().to_string()
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_str;

    fn successful_test_case(code: &str) {
        println!("========= source ===========");
        println!("{}", code);
        println!("========= result ===========");
        print!("{}", parse_str(code).unwrap());
        println!("========= finish ===========\n");
    }

    #[test]
    fn simple_parse() {
        successful_test_case("let f : 1 = 0;");
        successful_test_case("let f : k = f e;");
        successful_test_case("let f : k = ((x, y).1).2;");
        successful_test_case("let f : C k = C e;");
    }
}
