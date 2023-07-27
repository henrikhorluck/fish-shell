use crate::expand::ExpandResultCode;
use crate::{
    ast::{Ast, Node},
    complete::CompletionList,
    expand::{expand_string, ExpandFlags},
    operation_context::OperationContext,
    parse_constants::ParseTreeFlags,
    wchar::wstr,
};

pub struct Parser;

impl Parser {
    /// Evaluate line as a list of parameters, i.e. tokenize it and perform parameter expansion and
    /// cmdsubst execution on the tokens. Errors are ignored. If a parser is provided, it is used
    /// for command substitution expansion.
    pub fn expand_argument_list(
        arg_list_src: &wstr,
        flags: ExpandFlags,
        ctx: &OperationContext<'_>,
    ) -> CompletionList {
        // Parse the string as an argument list.
        let ast = Ast::parse_argument_list(arg_list_src, ParseTreeFlags::default(), None);
        if ast.errored() {
            // Failed to parse. Here we expect to have reported any errors in test_args.
            return vec![];
        }

        // Get the root argument list and extract arguments from it.
        let mut result = vec![];
        let list = ast.top().as_freestanding_argument_list().unwrap();
        for arg in list.arguments.iter() {
            let arg_src = arg.source(arg_list_src);
            if expand_string(arg_src.to_owned(), &mut result, flags, ctx, None)
                == ExpandResultCode::error
            {
                break; // failed to expand a string
            }
        }
        result
    }
}
