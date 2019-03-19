use crate::ast::*;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
/// The name stands for "Mini-TT's Parser"
struct MiniParser;

// Tik♂Tok on the clock but the party don't stop!
type Tok<'a> = Pair<'a, Rule>;
type Tik<'a> = Pairs<'a, Rule>;

/// Parse a string into an optional expression based on `file` rule:
/// ```ignore
/// file = { WHITESPACE* ~ expression }
/// ```
pub fn parse_str(input: &str) -> Result<Expression, String> {
    Ok(expression_to_expression(
        MiniParser::parse(Rule::file, input)
            .map_err(|err| format!("Parse failed at:{}", err))?
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap(),
    ))
}

/// Parse a string into an optional expression and print error to stderr.
#[inline]
pub fn parse_str_err_printed(code: &str) -> Result<Expression, ()> {
    parse_str(code).map_err(|err| eprintln!("{}", err))
}

macro_rules! next_rule {
    ($inner:expr, $rule_name:ident, $function:ident) => {{
        let token = $inner.next().unwrap();
        debug_assert_eq!(token.as_rule(), Rule::$rule_name);
        $function(token)
    }};
}

#[inline]
fn next_expression(inner: &mut Tik) -> Expression {
    next_rule!(inner, expression, expression_to_expression)
}

#[inline]
fn next_atom(inner: &mut Tik) -> Expression {
    next_rule!(inner, atom, atom_to_expression)
}

#[inline]
fn next_pattern(inner: &mut Tik) -> Pattern {
    next_rule!(inner, pattern, pattern_to_pattern)
}

#[inline]
fn next_constructor_name(inner: &mut Tik) -> String {
    next_rule!(inner, constructor_name, identifier_to_name)
}

#[inline]
fn end_of_rule(inner: &mut Tik) {
    debug_assert_eq!(inner.next(), None)
}

/// ```ignore
/// expression =
///  { declaration
///  | const_declaration
///  | merge_sum
///  | application
///  | function_type
///  | pair_type
///  | first
///  | second
///  | pair
///  | atom
///  }
/// ```
pub fn expression_to_expression(rules: Tok) -> Expression {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::declaration => declaration_to_expression(the_rule),
        Rule::const_declaration => const_declaration_to_expression(the_rule),
        Rule::merge_sum => merge_sum_to_expression(the_rule),
        Rule::application => application_to_expression(the_rule),
        Rule::function_type => function_type_to_expression(the_rule),
        Rule::pair_type => pair_type_to_expression(the_rule),
        Rule::first => first_to_expression(the_rule),
        Rule::second => second_to_expression(the_rule),
        Rule::pair => pair_to_expression(the_rule),
        Rule::atom => atom_to_expression(the_rule),
        _ => unreachable!(),
    }
}

/// ```ignore
/// first = { atom ~ ".1" }
/// ```
pub fn first_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let pair = next_atom(&mut inner);
    end_of_rule(&mut inner);
    Expression::First(Box::new(pair))
}

/// ```ignore
/// function_type = { atom ~ "->" ~ expression }
/// ```
pub fn function_type_to_expression(the_rule: Tok) -> Expression {
    let (input, output) = atom_and_expression_to_tuple(the_rule);
    Expression::Pi((Pattern::Unit, Box::new(input)), Box::new(output))
}

/// ```ignore
/// multiplication = _{ "*" | "\\times" | "×" }
/// pair_type = { atom ~ multiplication ~ expression }
/// ```
pub fn pair_type_to_expression(the_rule: Tok) -> Expression {
    let (first, second) = atom_and_expression_to_tuple(the_rule);
    Expression::Sigma((Pattern::Unit, Box::new(first)), Box::new(second))
}

/// Helper, extracted.
pub fn atom_and_expression_to_tuple(the_rule: Tok) -> (Expression, Expression) {
    let mut inner: Tik = the_rule.into_inner();
    let input = next_atom(&mut inner);
    let output = next_expression(&mut inner);
    end_of_rule(&mut inner);
    (input, output)
}

/// ```ignore
/// second = { atom ~ ".2" }
/// ```
pub fn second_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let pair = next_atom(&mut inner);
    end_of_rule(&mut inner);
    Expression::Second(Box::new(pair))
}

/// ```ignore
/// pair = { atom ~ "," ~ expression }
/// ```
pub fn pair_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let first = next_atom(&mut inner);
    let second = next_expression(&mut inner);
    end_of_rule(&mut inner);
    Expression::Pair(Box::new(first), Box::new(second))
}

/// ```ignore
/// application = { atom ~ expression }
/// ```
pub fn application_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let function = next_atom(&mut inner);
    let argument = next_expression(&mut inner);
    end_of_rule(&mut inner);
    Expression::Application(Box::new(function), Box::new(argument))
}

/// ```ignore
/// prefix_parameter = { "(" ~ typed_pattern ~ ")" }
/// prefix_parameters = { prefix_parameter* }
/// ```
pub fn prefix_parameters_to_vec(the_rule: Tok) -> Vec<Typed> {
    let mut map: Vec<Typed> = Default::default();
    for prefix_parameter in the_rule.into_inner() {
        let mut inner: Tik = prefix_parameter.into_inner();
        let pattern = next_pattern(&mut inner);
        let parameter_type = next_expression(&mut inner);
        map.push((pattern, Box::new(parameter_type)));
    }
    map
}

/// ```ignore
/// declaration =
///  { let_or_rec?
///  ~ pattern
///  ~ prefix_parameters
///  ~ ":" ~ expression
///  ~ "=" ~ expression
///  ~ ";" ~ expression?
///  }
/// ```
pub fn declaration_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let let_or_rec_rule = inner.next().unwrap();
    let rec = match let_or_rec_rule.as_str() {
        "let" => false,
        "rec" => true,
        _ => unreachable!(),
    };
    let name = next_pattern(&mut inner);
    let prefix_parameters = next_rule!(inner, prefix_parameters, prefix_parameters_to_vec);
    let signature = next_expression(&mut inner);
    let body = next_expression(&mut inner);
    let rest = inner
        .next()
        .map(expression_to_expression)
        .unwrap_or(Expression::Void);
    end_of_rule(&mut inner);
    let declaration_type = if rec {
        DeclarationType::Recursive
    } else {
        DeclarationType::Simple
    };
    let declaration = Declaration::new(name, prefix_parameters, signature, body, declaration_type);
    Expression::Declaration(Box::new(declaration), Box::new(rest))
}

/// ```ignore
/// const_declaration =
///  { "const"
///  ~ pattern
///  ~ "=" ~ expression
///  ~ ";" ~ expression?
///  }
/// ```
pub fn const_declaration_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let name = next_pattern(&mut inner);
    let body = next_expression(&mut inner);
    let rest = inner
        .next()
        .map(expression_to_expression)
        .unwrap_or(Expression::Void);
    end_of_rule(&mut inner);
    Expression::Constant(name, Box::new(body), Box::new(rest))
}

/// ```ignore
/// merge_sum = { atom ~ "++" ~ expression }
/// ```
pub fn merge_sum_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let lhs = next_atom(&mut inner);
    let rhs = next_expression(&mut inner);
    end_of_rule(&mut inner);
    Expression::Merge(Box::new(lhs), Box::new(rhs))
}

/// ```ignore
/// atom =
///   { universe
///   | constructor
///   | variable
///   | split
///   | sum
///   | one
///   | unit
///   | pi_type
///   | sigma_type
///   | lambda_expression
///   | "(" ~ expression ~ ")"
///   }
/// ```
pub fn atom_to_expression(rules: Tok) -> Expression {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::universe => universe_to_expression(the_rule),
        Rule::constructor => constructor_to_expression(the_rule),
        Rule::variable => variable_to_expression(the_rule),
        Rule::split => Expression::Split(choices_to_tree_map(the_rule)),
        Rule::sum => Expression::Sum(branches_to_tree_map(the_rule)),
        Rule::one => Expression::One,
        Rule::unit => Expression::Unit,
        Rule::pi_type => pi_type_to_expression(the_rule),
        Rule::sigma_type => sigma_type_to_expression(the_rule),
        Rule::lambda_expression => lambda_expression_to_expression(the_rule),
        Rule::expression => expression_to_expression(the_rule),
        _ => unreachable!(),
    }
}

/// ```ignore
/// branches = _{ "{" ~ (constructor ~ ("|" ~ constructor)*)? ~ "}" }
/// constructor = { constructor_name ~ expression }
/// ```
pub fn branches_to_tree_map(the_rule: Tok) -> Branch {
    let mut map: Branch = Default::default();
    for constructor in the_rule.into_inner() {
        let mut inner: Tik = constructor.into_inner();
        let constructor_name = next_constructor_name(&mut inner);
        let expression = inner
            .next()
            .map(expression_to_expression)
            .unwrap_or(Expression::One);
        map.insert(constructor_name, Box::new(expression));
        end_of_rule(&mut inner);
    }
    map
}

/// ```ignore
/// choices = _{ "{" ~ (pattern_match ~ ("|" ~ pattern_match)*)? ~ "}" }
/// pattern_match = { constructor_name ~ maybe_pattern ~ "=>" ~ expression }
/// ```
pub fn choices_to_tree_map(the_rule: Tok) -> Branch {
    let mut map: Branch = Default::default();
    for pattern_match in the_rule.into_inner() {
        let mut inner: Tik = pattern_match.into_inner();
        let constructor_name = next_constructor_name(&mut inner);
        let pattern = next_rule!(inner, maybe_pattern, maybe_pattern_to_pattern);
        let expression = next_expression(&mut inner);
        map.insert(
            constructor_name,
            Box::new(Expression::Lambda(pattern, None, Box::new(expression))),
        );
    }
    map
}

/// ```ignore
/// pi = _{ Pi unicode | "\\Pi" }
/// pi_type = { pi ~ typed_abstraction }
/// ```
pub fn pi_type_to_expression(the_rule: Tok) -> Expression {
    let (first_name, first_type, second) = typed_abstraction_to_tuple(the_rule);
    Expression::Pi((first_name, Box::new(first_type)), Box::new(second))
}

/// ```ignore
/// pi = _{ Pi unicode | "\\Pi" }
/// pi_type = { pi ~ typed_abstraction }
/// ```
pub fn sigma_type_to_expression(the_rule: Tok) -> Expression {
    let (input_name, input_type, output) = typed_abstraction_to_tuple(the_rule);
    Expression::Sigma((input_name, Box::new(input_type)), Box::new(output))
}

/// ```ignore
/// typed_abstraction = _{ pattern ~ ":" ~ expression ~ "." ~ expression }
/// ```
pub fn typed_abstraction_to_tuple(the_rule: Tok) -> (Pattern, Expression, Expression) {
    let mut inner: Tik = the_rule.into_inner();
    let input_name = next_pattern(&mut inner);
    let input_type = next_expression(&mut inner);
    let output = next_expression(&mut inner);
    end_of_rule(&mut inner);
    (input_name, input_type, output)
}

/// ```ignore
/// atom_pattern = { identifier | meta_var | "(" ~ pattern ~ ")" }
/// pattern = { pair_pattern | atom_pattern }
/// ```
pub fn atom_pattern_to_pattern(the_rule: Tok) -> Pattern {
    let rule: Tok = the_rule.into_inner().next().unwrap();
    match rule.as_rule() {
        Rule::identifier => Pattern::Var(identifier_to_name(rule)),
        Rule::meta_var => Pattern::Unit,
        Rule::pattern => pattern_to_pattern(rule),
        _ => unreachable!(),
    }
}

/// ```ignore
/// pair_pattern = { atom_pattern ~ "," ~ pattern }
/// pattern = { pair_pattern | atom_pattern }
/// ```
pub fn pattern_to_pattern(the_rule: Tok) -> Pattern {
    let rule: Tok = the_rule.into_inner().next().unwrap();
    match rule.as_rule() {
        Rule::pair_pattern => {
            let mut inner: Tik = rule.into_inner();
            let first = next_rule!(inner, atom_pattern, atom_pattern_to_pattern);
            let second = next_pattern(&mut inner);
            end_of_rule(&mut inner);
            Pattern::Pair(Box::new(first), Box::new(second))
        }
        Rule::atom_pattern => atom_pattern_to_pattern(rule),
        _ => unreachable!(),
    }
}

/// ```ignore
/// lambda = _{ lambda unicode | "\\lambda" }
/// lambda_expression = { lambda ~ pattern ~ "." ~ expression }
/// ```
pub fn lambda_expression_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let parameter = next_pattern(&mut inner);
    let body = next_expression(&mut inner);
    end_of_rule(&mut inner);
    Expression::Lambda(parameter, None, Box::new(body))
}

/// Constructor as an expression
pub fn constructor_to_expression(the_rule: Tok) -> Expression {
    let (constructor, argument) = constructor_to_tuple(the_rule);
    Expression::Constructor(constructor, Box::new(argument))
}

/// ```ignore
/// level = { ASCII_DIGIT* }
/// universe = @{ "Type" ~ level }
/// ```
pub fn universe_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let level = inner.next().unwrap().as_str();
    Expression::Type(level.parse().unwrap_or(0))
}

/// ```ignore
/// constructor_name = @{ ASCII_ALPHA_UPPER ~ identifier? }
/// constructor = { constructor_name ~ expression }
/// ```
pub fn constructor_to_tuple(the_rule: Tok) -> (String, Expression) {
    let mut inner: Tik = the_rule.into_inner();
    let constructor = next_constructor_name(&mut inner);
    let argument = inner
        .next()
        .map(expression_to_expression)
        .unwrap_or(Expression::Unit);
    end_of_rule(&mut inner);
    (constructor, argument)
}

/// ```ignore
/// maybe_pattern = { pattern? }
/// ```
pub fn maybe_pattern_to_pattern(the_rule: Tok) -> Pattern {
    let mut inner: Tik = the_rule.into_inner();
    let pattern = inner
        .next()
        .map(pattern_to_pattern)
        .unwrap_or(Pattern::Unit);
    end_of_rule(&mut inner);
    pattern
}

/// ```ignore
/// variable = { identifier }
/// ```
pub fn variable_to_expression(the_rule: Tok) -> Expression {
    let mut inner: Tik = the_rule.into_inner();
    let name = next_rule!(inner, identifier, identifier_to_name);
    end_of_rule(&mut inner);
    Expression::Var(name)
}

/// ```ignore
/// identifier = @{ !"let" ~ !"rec" ~ !"0" ~ !"1" ~ character+ }
/// ```
pub fn identifier_to_name(rule: Tok) -> String {
    rule.as_span().as_str().to_string()
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_str_err_printed;

    #[cfg(not(feature = "pretty"))]
    fn successful_test_case(code: &str) {
        let expr = parse_str_err_printed(code).unwrap();
        println!("{:?}", expr);
    }

    #[cfg(feature = "pretty")]
    fn successful_test_case(code: &str) {
        println!("========= source ===========");
        println!("{}", code);
        println!("========= result ===========");
        let expr = parse_str_err_printed(code).unwrap();
        print!("{}", expr);
        let code = format!("{}", expr);
        println!("========= double ===========");
        print!("{}", parse_str_err_printed(code.as_str()).unwrap());
        println!("========= finish ===========\n");
    }

    #[test]
    fn simple_parse() {
        successful_test_case("let unit_one : 1 = 0;\nlet type_one : Type0 = unit_one;");
        successful_test_case("let application : k = f e;");
        successful_test_case("let pair_first_second : k = ((x, y).1).2;");
        successful_test_case("let sigma_type : \\Sigma x : x_type . y = x, y;");
        successful_test_case("let constructor : C k = C e;");
        successful_test_case("let pi_lambda : \\Pi a : b . c = \\lambda a . expr;");
        successful_test_case("let pat, pat2 : \\Pi _ : b . c = \\lambda _ . expr;");
    }

    #[test]
    fn no_reparse() {
        successful_no_reparse("let function : Sum {C e} = split {C _ => e};");
        successful_no_reparse("let function (x : a) : bla = rua;");
    }

    fn successful_no_reparse(code: &str) {
        println!("{}", parse_str_err_printed(code).unwrap());
    }
}
