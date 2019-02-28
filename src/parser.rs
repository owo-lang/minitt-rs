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
/// ```ignore
/// entry = { expression }
/// ```
pub fn parse_str(input: &str) -> Result<Expression, String> {
    Ok(expression_to_expression(
        MiniParser::parse(Rule::expression, input)
            .map_err(|err| format!("Parse failed at:{}", err))?
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

/// ```ignore
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

/// ```ignore
/// first = { atom ~ ".1" }
/// ```
fn first_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let pair = next_atom!(inner);
    end_of_rule!(inner);
    Expression::First(Box::new(pair))
}

/// ```ignore
/// second = { atom ~ ".2" }
/// ```
fn second_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let pair = next_atom!(inner);
    end_of_rule!(inner);
    Expression::Second(Box::new(pair))
}

/// ```ignore
/// pair = { atom ~ "," ~ expression }
/// ```
fn pair_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let first = next_atom!(inner);
    let second = next_expression!(inner);
    end_of_rule!(inner);
    Expression::Pair(Box::new(first), Box::new(second))
}

/// ```ignore
/// application = { atom ~ expression }
/// ```
fn application_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let function = next_atom!(inner);
    let argument = next_expression!(inner);
    end_of_rule!(inner);
    Expression::Application(Box::new(function), Box::new(argument))
}

/// ```ignore
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

/// ```ignore
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
        Rule::pi_type => pi_type_to_expression(the_rule),
        Rule::sigma_type => sigma_type_to_expression(the_rule),
        Rule::lambda_expression => lambda_expression_to_expression(the_rule),
        Rule::universe => Expression::Type,
        Rule::expression => expression_to_expression(the_rule),
        _ => panic!("{}", the_rule),
    }
}

/// ```ignore
/// pi = _{ Pi unicode | "\\Pi" }
/// pi_type = { pi ~ typed_abstraction }
/// ```
fn pi_type_to_expression(the_rule: Tok) -> Expression {
    let (first_name, first_type, second) = typed_abstraction_to_tuple(the_rule);
    Expression::Pi(first_name, Box::new(first_type), Box::new(second))
}

/// ```ignore
/// pi = _{ Pi unicode | "\\Pi" }
/// pi_type = { pi ~ typed_abstraction }
/// ```
fn sigma_type_to_expression(the_rule: Tok) -> Expression {
    let (input_name, input_type, output) = typed_abstraction_to_tuple(the_rule);
    Expression::Sigma(input_name, Box::new(input_type), Box::new(output))
}

/// ```ignore
/// typed_abstraction = _{ identifier ~ ":" ~ expression ~ "." ~ expression }
/// ```
fn typed_abstraction_to_tuple(the_rule: Tok) -> (Pattern, Expression, Expression) {
    let mut inner = the_rule.into_inner();
    // TODO: parse as pattern
    let input_name = next_identifier!(inner);
    let input_type = next_expression!(inner);
    let output = next_expression!(inner);
    end_of_rule!(inner);
    (Pattern::Var(input_name), input_type, output)
}

/// ```ignore
/// lambda = _{ lambda unicode | "\\lambda" }
/// lambda_expression = { lambda ~ identifier ~ "." ~ expression }
/// ```
fn lambda_expression_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    // TODO: parse as pattern
    let parameter = next_identifier!(inner);
    let body = next_expression!(inner);
    end_of_rule!(inner);
    Expression::Lambda(Pattern::Var(parameter), Box::new(body))
}

/// Constructor as an expression
fn constructor_to_expression(the_rule: Tok) -> Expression {
    let (constructor, argument) = constructor_to_tuple(the_rule);
    Expression::Constructor(constructor, Box::new(argument))
}

/// ```ignore
/// constructor_name = @{ ASCII_ALPHA_UPPER ~ identifier? }
/// constructor = { constructor_name ~ expression }
/// ```
fn constructor_to_tuple(the_rule: Tok) -> (String, Expression) {
    let mut inner = the_rule.into_inner();
    let constructor = next_rule!(inner, constructor_name, identifier_to_name);
    let argument = next_expression!(inner);
    end_of_rule!(inner);
    (constructor, argument)
}

/// ```ignore
/// variable = { identifier }
/// ```
fn variable_to_expression(the_rule: Tok) -> Expression {
    let mut inner = the_rule.into_inner();
    let name = next_identifier!(inner);
    end_of_rule!(inner);
    Expression::Var(name)
}

/// ```ignore
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
        let expr = parse_str(code).map_err(|err| println!("{}", err)).unwrap();
        print!("{}", expr);
        let code = format!("{}", expr);
        println!("========= double ===========");
        print!(
            "{}",
            parse_str(code.as_str())
                .map_err(|err| println!("{}", err))
                .unwrap()
        );
        println!("========= finish ===========\n");
    }

    #[test]
    fn simple_parse() {
        successful_test_case("let unit_one : 1 = 0;\nlet type_one : U = unit_one;");
        successful_test_case("let application : k = f e;");
        successful_test_case("let pair_first_second : k = ((x, y).1).2;");
        successful_test_case("let sigma_type : \\Sigma x : x_type . y = x, y;");
        successful_test_case("let constructor : C k = C e;");
        successful_test_case("let pi_lambda : \\Pi a : b . c = \\lambda a . expr;");
    }
}
