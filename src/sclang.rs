mod sclcelldata;

use std::fmt::Write;

use std::{collections::HashMap, str};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

use sclcelldata::{create_cell_with_links, create_cell_with_text_only, enable_feature, is_debug_enabled, SCLCursor};

pub type SCLDataMap = HashMap<String, SCLCursor>;

#[derive(Parser)]
#[grammar_inline = r#"
WHITESPACE = _{ " " }
lparen = _{ "(" }
rparen = _{ ")" }
store_data_command = { "store-data" }
show_data_command = { "show-data" }
update_data_command = { "update-data" }
command_name = { "store-data"
  | "show-data"
  | "update-data"
  | "drop-symbol"
  | "enable-feature"
}
symbol_name = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "-")* }
// XXX TODO BACKSLASH-ESCAPED CHARS INCLUDING DOUBLE-QUOTE
string_char = { !("\"" | "\\") ~ ANY }
string_value_contents = @{ string_char* }
string_value = @{ "\"" ~ string_value_contents ~ "\"" }
command_arg = @{ symbol_name }
// XXX TODO XXX DATA IN CELL VALUE ETC
cell_symbol_refs = { lparen ~ symbol_name ~ symbol_name ~ rparen }
cell_value = { lparen ~ string_value ~ string_value ~ cell_symbol_refs? ~ rparen }
command_line = { lparen ~ command_name ~ command_arg ~ cell_value? ~ rparen }
"#]
struct SCLParser;

fn get_cell_symbol_refs(cell_symbol_refs: Pair<Rule>) -> (Pair<Rule>, Pair<Rule>) {
    let mut refs = cell_symbol_refs.into_inner();
    (refs.next().unwrap(), refs.next().unwrap())
}

fn handle_command_line(m: &mut SCLDataMap, p: Pairs<Rule>) -> String {
    let inner_pairs = p.clone().next().unwrap();
    let mut inner_cl_iter = inner_pairs.into_inner();
    let c1 = inner_cl_iter.next();
    match c1.clone().unwrap().as_rule() {
        Rule::command_name => {
            let command_name = c1.unwrap().as_span().as_str();
            let c2 = inner_cl_iter.next();
            match c2.clone().unwrap().as_rule() {
                Rule::command_arg => {
                    let symbol_name = c2.unwrap().as_span().as_str();
                    let c3 = inner_cl_iter.next();
                    if c3.is_some() {
                        match c3.clone().unwrap().as_rule() {
                            Rule::cell_value => {
                                let mut value_inner_iter = c3.clone().into_iter();
                                let x1 = value_inner_iter.next();
                                let mut x2 = x1.unwrap().into_inner();
                                let t1 = x2.next();
                                let t2 = x2.next();
                                let tt1 = t1.unwrap().as_str();
                                let tt2 = t2.unwrap().as_str();
                                let optional_cell_symbol_refs = x2.next();
                                let symbol_refs = match optional_cell_symbol_refs {
                                    Some(r) => Some(get_cell_symbol_refs(r)),
                                    None => None,
                                };
                                match command_name {
                                    "store-data" => {
                                        m.insert(
                                            String::from(symbol_name),
                                            match symbol_refs {
                                                // XXX TODO GRACEFUL HANDLING IN CASE OF NON-EXISTING SYMBOL NAME
                                                Some(ref r) => {
                                                    create_cell_with_links(tt1, tt2, m.get(r.0.as_str()).unwrap().clone(), m.get(r.1.as_str()).unwrap().clone())
                                                }
                                                None => create_cell_with_text_only(tt1, tt2),
                                            },
                                        );
                                        let mut r = String::new();
                                        writeln!(r, "STORED DATA FOR SYMBOL - {}", symbol_name);
                                        if is_debug_enabled() {
                                            write!(r, "{}", m.get(&String::from(symbol_name)).unwrap().get_dump());
                                        }
                                        return r;
                                    }
                                    "update-data" => {
                                        let x = m.get(&String::from(symbol_name)).clone();
                                        if x.is_none() {
                                            // XXX TODO INDICATE ERROR MORE FORMALLY HERE
                                            let mut r = String::new();
                                            writeln!(r, "UPDATE - SYMBOL NOT FOUND: {}", symbol_name);
                                            return r;
                                        } else {
                                            x.unwrap().update_data(
                                                tt1,
                                                tt2,
                                                match symbol_refs {
                                                    Some(ref r) => Some(m.get(r.0.as_str()).unwrap().clone()),
                                                    None => None,
                                                },
                                                match symbol_refs {
                                                    Some(ref r) => Some(m.get(r.1.as_str()).unwrap().clone()),
                                                    None => None,
                                                },
                                            );
                                            let mut r = String::new();
                                            writeln!(r, "UPDATED DATA FOR SYMBOL - {}", symbol_name);
                                            if is_debug_enabled() {
                                                write!(r, "{}", m.get(&String::from(symbol_name)).unwrap().get_dump());
                                            }
                                            return r;
                                        }
                                    }
                                    "show-data" => {
                                        println!("EXTRA ARGUMENT PRESENT FOR COMMAND: {}", command_name);
                                        panic!("FATAL ERROR: BAD INPUT - MISSING GRACEFUL ERROR HANDLER");
                                    }
                                    "drop-symbol" => {
                                        println!("EXTRA ARGUMENT PRESENT FOR COMMAND: {}", command_name);
                                        panic!("FATAL ERROR: BAD INPUT - MISSING GRACEFUL ERROR HANDLER");
                                    }
                                    "enable-feature" => {
                                        println!("EXTRA ARGUMENT PRESENT FOR COMMAND: {}", command_name);
                                        panic!("FATAL ERROR: BAD INPUT - MISSING GRACEFUL ERROR HANDLER");
                                    }
                                    _ => unreachable!("INTERNAL ERROR - XXX"),
                                }
                            }
                            _ => unreachable!("INTERNAL ERROR - XXX"),
                        }
                    } else {
                        match command_name {
                            "show-data" => {
                                let x = m.get(&String::from(symbol_name));
                                // NOTE: DEBUG OUTPUT OF RAW GET RESULT OVERFLOWS IN CASE OF CIRCULAR DATA !!!
                                if x.is_none() {
                                    // XXX TODO INDICATE ERROR MORE FORMALLY HERE
                                    let mut r = String::new();
                                    writeln!(r, "SYMBOL NOT FOUND: {}", symbol_name);
                                    return r;
                                } else {
                                    let mut r = String::new();
                                    writeln!(r, "DATA FOR SYMBOL - {}", symbol_name);
                                    write!(r, "{}", m.get(&String::from(symbol_name)).unwrap().get_dump());
                                    return r;
                                }
                            }
                            "drop-symbol" => {
                                let x = m.remove(&String::from(symbol_name));
                                let mut r = String::new();
                                if x.is_none() {
                                    writeln!(r, "DROP FAILURE - SYMBOL NOT FOUND: {}", symbol_name);
                                } else {
                                    writeln!(r, "DROPPED SYMBOL: {}", symbol_name);
                                }
                                return r;
                            }
                            "store-data" => {
                                println!("MISSING DATA ARGUMENT FOR COMMAND: {}", command_name);
                                panic!("FATAL ERROR: BAD INPUT - MISSING GRACEFUL ERROR HANDLER");
                            }
                            "update-data" => {
                                println!("MISSING DATA ARGUMENT FOR COMMAND: {}", command_name);
                                panic!("FATAL ERROR: BAD INPUT - MISSING GRACEFUL ERROR HANDLER");
                            }
                            "enable-feature" => {
                                let mut r = String::new();
                                writeln!(r, "ENABLE FEATURE: {}", symbol_name);
                                enable_feature(symbol_name);
                                return r;
                            }
                            _ => unreachable!("INTERNAL ERROR - XXX"),
                        }
                    }
                }
                _ => unreachable!("INTERNAL ERROR - XXX"),
            }
        }
        _ => unreachable!("INTERNAL ERROR - XXX"),
    }
}

pub fn execute_command(m: &mut SCLDataMap, command_line: &str) -> String {
    let cl = SCLParser::parse(Rule::command_line, command_line);
    if cl.is_ok() {
        handle_command_line(m, cl.unwrap())
    } else {
        // XXX TODO IMPROVE PARSE ERROR REPORTING & HANDLING
        String::from("COMMAND PARSE ERROR")
    }
}

#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_circular_2_records() {
    use expect_test::expect;

    let mut map: SCLDataMap = HashMap::new();
    let m = &mut map;

    let mut cl;
    let mut x;

    sclcelldata::reset_drop_cell_count();

    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(store-data data-1 ("first text" "second text"))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-1
        - text 1: "first text"
        - text 2: "second text"
        - link 1 - empty
        - link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-1)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-1
        - text 1: "first text"
        - text 2: "second text"
        - link 1 - empty
        - link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(store-data data-2 ("text 1" "text 2" (data-1 data-1)))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-2
        - text 1: "text 1"
        - text 2: "text 2"
        - link 1 info:
          link 1 info - text 1: "first text"
          link 1 info - text 2: "second text"
          - link 1 -> link 1 - empty
          - link 1 -> link 2 - empty
        - link 2 info:
          link 2 info - text 1: "first text"
          link 2 info - text 2: "second text"
          - link 2 -> link 1 - empty
          - link 2 -> link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-2)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-2
        - text 1: "text 1"
        - text 2: "text 2"
        - link 1 info:
          link 1 info - text 1: "first text"
          link 1 info - text 2: "second text"
          - link 1 -> link 1 - empty
          - link 1 -> link 2 - empty
        - link 2 info:
          link 2 info - text 1: "first text"
          link 2 info - text 2: "second text"
          - link 2 -> link 1 - empty
          - link 2 -> link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(update-data data-1 ("first text - updated" "second text - updated" (data-2 data-2)))"#;
    x = expect![[r#"
        UPDATED DATA FOR SYMBOL - data-1
        - text 1: "first text - updated"
        - text 2: "second text - updated"
        - link 1 info:
          link 1 info - text 1: "text 1"
          link 1 info - text 2: "text 2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "first text - updated"
            link 1 -> link 1 info - text 2: "second text - updated"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "first text - updated"
            link 1 -> link 2 info - text 2: "second text - updated"
        - link 2 info:
          link 2 info - text 1: "text 1"
          link 2 info - text 2: "text 2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "first text - updated"
            link 2 -> link 1 info - text 2: "second text - updated"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "first text - updated"
            link 2 -> link 2 info - text 2: "second text - updated"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-1)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-1
        - text 1: "first text - updated"
        - text 2: "second text - updated"
        - link 1 info:
          link 1 info - text 1: "text 1"
          link 1 info - text 2: "text 2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "first text - updated"
            link 1 -> link 1 info - text 2: "second text - updated"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "first text - updated"
            link 1 -> link 2 info - text 2: "second text - updated"
        - link 2 info:
          link 2 info - text 1: "text 1"
          link 2 info - text 2: "text 2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "first text - updated"
            link 2 -> link 1 info - text 2: "second text - updated"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "first text - updated"
            link 2 -> link 2 info - text 2: "second text - updated"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-2)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-2
        - text 1: "text 1"
        - text 2: "text 2"
        - link 1 info:
          link 1 info - text 1: "first text - updated"
          link 1 info - text 2: "second text - updated"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "text 1"
            link 1 -> link 1 info - text 2: "text 2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "text 1"
            link 1 -> link 2 info - text 2: "text 2"
        - link 2 info:
          link 2 info - text 1: "first text - updated"
          link 2 info - text 2: "second text - updated"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "text 1"
            link 2 -> link 1 info - text 2: "text 2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "text 1"
            link 2 -> link 2 info - text 2: "text 2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(drop-symbol data-2)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-2
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(show-data data-1)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-1
        - text 1: "first text - updated"
        - text 2: "second text - updated"
        - link 1 info:
          link 1 info - text 1: "text 1"
          link 1 info - text 2: "text 2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "first text - updated"
            link 1 -> link 1 info - text 2: "second text - updated"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "first text - updated"
            link 1 -> link 2 info - text 2: "second text - updated"
        - link 2 info:
          link 2 info - text 1: "text 1"
          link 2 info - text 2: "text 2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "first text - updated"
            link 2 -> link 1 info - text 2: "second text - updated"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "first text - updated"
            link 2 -> link 2 info - text 2: "second text - updated"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-2)"#;
    x = expect![[r#"
        SYMBOL NOT FOUND: data-2
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(drop-symbol data-1)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-1
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-2)"#;
    x = expect![[r#"
        SYMBOL NOT FOUND: data-2
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    x = expect![[r#"
        2
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);
}

#[test]
#[serial_test::serial]
fn test_circular_5_records() {
    use expect_test::expect;

    sclcelldata::reset_drop_cell_count();

    let mut map: SCLDataMap = HashMap::new();
    let m = &mut map;

    let mut cl;
    let mut x;

    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(store-data data-node-a ("a-text-1" "a-text-2"))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-node-a
        - text 1: "a-text-1"
        - text 2: "a-text-2"
        - link 1 - empty
        - link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(store-data data-node-b ("b-text-1" "b-text-2" (data-node-a data-node-a)))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-node-b
        - text 1: "b-text-1"
        - text 2: "b-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 - empty
          - link 1 -> link 2 - empty
        - link 2 info:
          link 2 info - text 1: "a-text-1"
          link 2 info - text 2: "a-text-2"
          - link 2 -> link 1 - empty
          - link 2 -> link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(update-data data-node-a ("a-text-1" "a-text-2" (data-node-b data-node-b)))"#;
    x = expect![[r#"
        UPDATED DATA FOR SYMBOL - data-node-a
        - text 1: "a-text-1"
        - text 2: "a-text-2"
        - link 1 info:
          link 1 info - text 1: "b-text-1"
          link 1 info - text 2: "b-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "a-text-1"
            link 1 -> link 1 info - text 2: "a-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "a-text-1"
            link 1 -> link 2 info - text 2: "a-text-2"
        - link 2 info:
          link 2 info - text 1: "b-text-1"
          link 2 info - text 2: "b-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "a-text-1"
            link 2 -> link 2 info - text 2: "a-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(store-data data-node-c ("c-text-1" "c-text-2" (data-node-a data-node-b)))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-node-c
        - text 1: "c-text-1"
        - text 2: "c-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "b-text-1"
            link 1 -> link 2 info - text 2: "b-text-2"
        - link 2 info:
          link 2 info - text 1: "b-text-1"
          link 2 info - text 2: "b-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "a-text-1"
            link 2 -> link 2 info - text 2: "a-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(update-data data-node-b ("b-text-1" "b-text-2" (data-node-a data-node-c)))"#;
    x = expect![[r#"
        UPDATED DATA FOR SYMBOL - data-node-b
        - text 1: "b-text-1"
        - text 2: "b-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "b-text-1"
            link 1 -> link 2 info - text 2: "b-text-2"
        - link 2 info:
          link 2 info - text 1: "c-text-1"
          link 2 info - text 2: "c-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "b-text-1"
            link 2 -> link 2 info - text 2: "b-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(store-data data-node-d ("d-text-1" "d-text-2" (data-node-a data-node-c)))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-node-d
        - text 1: "d-text-1"
        - text 2: "d-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "b-text-1"
            link 1 -> link 2 info - text 2: "b-text-2"
        - link 2 info:
          link 2 info - text 1: "c-text-1"
          link 2 info - text 2: "c-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "b-text-1"
            link 2 -> link 2 info - text 2: "b-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(store-data data-node-e ("e-text-1" "e-text-2" (data-node-a data-node-d)))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-node-e
        - text 1: "e-text-1"
        - text 2: "e-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "b-text-1"
            link 1 -> link 2 info - text 2: "b-text-2"
        - link 2 info:
          link 2 info - text 1: "d-text-1"
          link 2 info - text 2: "d-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "c-text-1"
            link 2 -> link 2 info - text 2: "c-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(update-data data-node-a ("a-text-1" "a-text-2" (data-node-b data-node-e)))"#;
    x = expect![[r#"
        UPDATED DATA FOR SYMBOL - data-node-a
        - text 1: "a-text-1"
        - text 2: "a-text-2"
        - link 1 info:
          link 1 info - text 1: "b-text-1"
          link 1 info - text 2: "b-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "a-text-1"
            link 1 -> link 1 info - text 2: "a-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "c-text-1"
            link 1 -> link 2 info - text 2: "c-text-2"
        - link 2 info:
          link 2 info - text 1: "e-text-1"
          link 2 info - text 2: "e-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "d-text-1"
            link 2 -> link 2 info - text 2: "d-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(drop-symbol data-node-b)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-node-b
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-node-a)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-node-a
        - text 1: "a-text-1"
        - text 2: "a-text-2"
        - link 1 info:
          link 1 info - text 1: "b-text-1"
          link 1 info - text 2: "b-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "a-text-1"
            link 1 -> link 1 info - text 2: "a-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "c-text-1"
            link 1 -> link 2 info - text 2: "c-text-2"
        - link 2 info:
          link 2 info - text 1: "e-text-1"
          link 2 info - text 2: "e-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "d-text-1"
            link 2 -> link 2 info - text 2: "d-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(show-data data-node-c)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-node-c
        - text 1: "c-text-1"
        - text 2: "c-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "e-text-1"
            link 1 -> link 2 info - text 2: "e-text-2"
        - link 2 info:
          link 2 info - text 1: "b-text-1"
          link 2 info - text 2: "b-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "c-text-1"
            link 2 -> link 2 info - text 2: "c-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-node-e)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-node-e
        - text 1: "e-text-1"
        - text 2: "e-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "e-text-1"
            link 1 -> link 2 info - text 2: "e-text-2"
        - link 2 info:
          link 2 info - text 1: "d-text-1"
          link 2 info - text 2: "d-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "c-text-1"
            link 2 -> link 2 info - text 2: "c-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(drop-symbol data-node-a)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-node-a
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    // XXX TODO ENSURE & TEST THAT OUTGOING LINKS FROM NODE A ARE NOT BROKEN AT THIS POINT

    cl = r#"(show-data data-node-d)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-node-d
        - text 1: "d-text-1"
        - text 2: "d-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "e-text-1"
            link 1 -> link 2 info - text 2: "e-text-2"
        - link 2 info:
          link 2 info - text 1: "c-text-1"
          link 2 info - text 2: "c-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "b-text-1"
            link 2 -> link 2 info - text 2: "b-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    // DROP FAILURE MESSAGE EXPECTED - ALREADY DROPPED THIS SYMBOL BEFORE
    cl = r#"(drop-symbol data-node-b)"#;
    x = expect![[r#"
        DROP FAILURE - SYMBOL NOT FOUND: data-node-b
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(drop-symbol data-node-c)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-node-c
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-node-d)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-node-d
        - text 1: "d-text-1"
        - text 2: "d-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "e-text-1"
            link 1 -> link 2 info - text 2: "e-text-2"
        - link 2 info:
          link 2 info - text 1: "c-text-1"
          link 2 info - text 2: "c-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "b-text-1"
            link 2 -> link 2 info - text 2: "b-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(drop-symbol data-node-d)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-node-d
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(show-data data-node-e)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-node-e
        - text 1: "e-text-1"
        - text 2: "e-text-2"
        - link 1 info:
          link 1 info - text 1: "a-text-1"
          link 1 info - text 2: "a-text-2"
          - link 1 -> link 1 info - text only:
            link 1 -> link 1 info - text 1: "b-text-1"
            link 1 -> link 1 info - text 2: "b-text-2"
          - link 1 -> link 2 info - text only:
            link 1 -> link 2 info - text 1: "e-text-1"
            link 1 -> link 2 info - text 2: "e-text-2"
        - link 2 info:
          link 2 info - text 1: "d-text-1"
          link 2 info - text 2: "d-text-2"
          - link 2 -> link 1 info - text only:
            link 2 -> link 1 info - text 1: "a-text-1"
            link 2 -> link 1 info - text 2: "a-text-2"
          - link 2 -> link 2 info - text only:
            link 2 -> link 2 info - text 1: "c-text-1"
            link 2 -> link 2 info - text 2: "c-text-2"
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(drop-symbol data-node-e)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-node-e
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    x = expect![[r#"
        5
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);
}

#[test]
#[serial]
fn test_non_circular_2_records() {
    use expect_test::expect;

    let mut map: SCLDataMap = HashMap::new();
    let m = &mut map;

    let mut cl;
    let mut x;

    sclcelldata::reset_drop_cell_count();

    cl = r#"(store-data data-1 ("first text" "second text"))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-1
        - text 1: "first text"
        - text 2: "second text"
        - link 1 - empty
        - link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-1)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-1
        - text 1: "first text"
        - text 2: "second text"
        - link 1 - empty
        - link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(store-data data-2 ("text 1" "text 2" (data-1 data-1)))"#;
    x = expect![[r#"
        STORED DATA FOR SYMBOL - data-2
        - text 1: "text 1"
        - text 2: "text 2"
        - link 1 info:
          link 1 info - text 1: "first text"
          link 1 info - text 2: "second text"
          - link 1 -> link 1 - empty
          - link 1 -> link 2 - empty
        - link 2 info:
          link 2 info - text 1: "first text"
          link 2 info - text 2: "second text"
          - link 2 -> link 1 - empty
          - link 2 -> link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-2)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-2
        - text 1: "text 1"
        - text 2: "text 2"
        - link 1 info:
          link 1 info - text 1: "first text"
          link 1 info - text 2: "second text"
          - link 1 -> link 1 - empty
          - link 1 -> link 2 - empty
        - link 2 info:
          link 2 info - text 1: "first text"
          link 2 info - text 2: "second text"
          - link 2 -> link 1 - empty
          - link 2 -> link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-1)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-1
        - text 1: "first text"
        - text 2: "second text"
        - link 1 - empty
        - link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    cl = r#"(show-data data-2)"#;
    x = expect![[r#"
        DATA FOR SYMBOL - data-2
        - text 1: "text 1"
        - text 2: "text 2"
        - link 1 info:
          link 1 info - text 1: "first text"
          link 1 info - text 2: "second text"
          - link 1 -> link 1 - empty
          - link 1 -> link 2 - empty
        - link 2 info:
          link 2 info - text 1: "first text"
          link 2 info - text 2: "second text"
          - link 2 -> link 1 - empty
          - link 2 -> link 2 - empty
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(drop-symbol data-1)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-1
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    // NOTE: EXPECT ZERO (0) drop cell count - data-1 cell is still referenced by data-2 cell
    x = expect![[r#"
        0
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);

    cl = r#"(drop-symbol data-2)"#;
    x = expect![[r#"
        DROPPED SYMBOL: data-2
    "#]];
    x.assert_eq(execute_command(m, cl).as_str());

    // EXPECT BOTH CELLS TO BE DROPPED AT THIS POINT
    x = expect![[r#"
        2
    "#]];
    let drop_cell_count = sclcelldata::get_drop_cell_count();
    x.assert_debug_eq(&drop_cell_count);
}
