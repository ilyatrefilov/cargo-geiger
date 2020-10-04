pub mod traversal;

use crate::format::print::{Prefix, PrintConfig};
use crate::format::Charset;

use cargo::core::dependency::DepKind;
use cargo::core::PackageId;

/// A step towards decoupling some parts of the table-tree printing from the
/// dependency graph traversal.
pub enum TextTreeLine {
    /// A text line for a package
    Package { id: PackageId, tree_vines: String },
    /// There are extra dependencies coming and we should print a group header,
    /// eg. "[build-dependencies]".
    ExtraDepsGroup { kind: DepKind, tree_vines: String },
}

#[derive(Debug, PartialEq)]
pub struct TreeSymbols {
    pub down: &'static str,
    pub tee: &'static str,
    pub ell: &'static str,
    pub right: &'static str,
}

fn construct_tree_vines_string(
    levels_continue: &mut Vec<bool>,
    print_config: &PrintConfig,
) -> String {
    let tree_symbols = get_tree_symbols(print_config.charset);

    match print_config.prefix {
        Prefix::Depth => format!("{} ", levels_continue.len()),
        Prefix::Indent => {
            let mut buffer = String::new();
            if let Some((&last_continues, rest)) = levels_continue.split_last()
            {
                for &continues in rest {
                    let c = if continues { tree_symbols.down } else { " " };
                    buffer.push_str(&format!("{}   ", c));
                }
                let c = if last_continues {
                    tree_symbols.tee
                } else {
                    tree_symbols.ell
                };
                buffer.push_str(&format!("{0}{1}{1} ", c, tree_symbols.right));
            }
            buffer
        }
        Prefix::None => "".into(),
    }
}

pub fn get_tree_symbols(charset: Charset) -> TreeSymbols {
    match charset {
        Charset::Utf8 => UTF8_TREE_SYMBOLS,
        Charset::Ascii => ASCII_TREE_SYMBOLS,
    }
}

const ASCII_TREE_SYMBOLS: TreeSymbols = TreeSymbols {
    down: "|",
    tee: "|",
    ell: "`",
    right: "-",
};

const UTF8_TREE_SYMBOLS: TreeSymbols = TreeSymbols {
    down: "│",
    tee: "├",
    ell: "└",
    right: "─",
};

#[cfg(test)]
mod tree_tests {
    use super::*;

    use crate::format::pattern::Pattern;
    use crate::format::Charset;

    use cargo::core::shell::Verbosity;
    use geiger::IncludeTests;
    use petgraph::EdgeDirection;

    #[test]
    fn construct_tree_vines_string_test() {
        let mut levels_continue = vec![true, false, true];

        let print_config = construct_print_config(Prefix::Depth);
        let tree_vines_string =
            construct_tree_vines_string(&mut levels_continue, &print_config);

        assert_eq!(tree_vines_string, "3 ");

        let print_config = construct_print_config(Prefix::Indent);
        let tree_vines_string =
            construct_tree_vines_string(&mut levels_continue, &print_config);

        assert_eq!(tree_vines_string, "|       |-- ");

        let print_config = construct_print_config(Prefix::None);
        let tree_vines_string =
            construct_tree_vines_string(&mut levels_continue, &print_config);

        assert_eq!(tree_vines_string, "");
    }

    #[test]
    fn get_tree_symbols_test() {
        assert_eq!(get_tree_symbols(Charset::Utf8), UTF8_TREE_SYMBOLS);
        assert_eq!(get_tree_symbols(Charset::Ascii), ASCII_TREE_SYMBOLS)
    }

    fn construct_print_config(prefix: Prefix) -> PrintConfig {
        let pattern = Pattern::try_build("{p}").unwrap();
        PrintConfig {
            all: false,
            verbosity: Verbosity::Verbose,
            direction: EdgeDirection::Outgoing,
            prefix,
            format: pattern,
            charset: Charset::Ascii,
            allow_partial_results: false,
            include_tests: IncludeTests::Yes,
            output_format: None,
        }
    }
}
