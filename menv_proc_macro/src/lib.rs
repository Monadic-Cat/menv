// assert_var_body, any_set_body, help_body, getters

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::str::FromStr;

mod lit_parse;

// struct VarDecl {
//     getter_name: Ident,
//     getter_mark: Option<Punct>,
//     var_name: Literal,
//     getter_type: Vec<TokenTree>,
//     help_message: Literal,
// }

// We *could* parse into the above, but we don't actually have to,
// since all we really need is to divide the input stream into pieces
// our existing require_envs! internals can handle.
struct VagueVarDecl {
    tokens: Vec<TokenTree>,
}

struct Stream {
    krate: Ident,
    decls: Vec<VagueVarDecl>,
    // All errors from parsing should be shoved into this field,
    // and the parser should limp along to the end no matter what.
    errors: Vec<TokenStream>,
}
impl Stream {
    fn parse(input: TokenStream) -> Self {
        // dbg!(&input);
        let mut input = input.into_iter();
        // We can assume the $crate token is always present,
        // because require_envs! always puts it in front.
        let krate = input.next().unwrap();
        let TokenTree::Ident(krate) = krate else { panic!() };
        let mut decls = Vec::new();

        let mut cdecl = Vec::new();
        for tree in input {
            if let TokenTree::Punct(punct) = &tree {
                if punct.as_char() == ';' {
                    decls.push(VagueVarDecl { tokens: cdecl });
                    cdecl = Vec::new();
                    continue;
                }
            }
            cdecl.push(tree);
        }
        if cdecl.len() != 0 {
            decls.push(VagueVarDecl { tokens: cdecl });
        }

        Self {
            krate,
            decls,
            errors: Vec::new(),
        }
    }
}

fn call_require_envs(krate: Ident, method: &str, input: Vec<TokenTree>) -> TokenStream {
    let krate_span = krate.span();
    let mut buf = vec![
        TokenTree::Punct(Punct::new('@', Spacing::Alone)),
        TokenTree::Ident(Ident::new(method, krate_span)),
    ];
    buf.extend(input.into_iter());
    let buf = vec![
        TokenTree::Ident(krate),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("require_envs", krate_span)),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Brace, buf.into_iter().collect())),
    ];
    buf.into_iter().collect()
}

#[proc_macro]
pub fn assert_var_body(input: TokenStream) -> TokenStream {
    let stream = Stream::parse(input);
    stream
        .decls
        .into_iter()
        .map(|decl| call_require_envs(stream.krate.clone(), "assert", decl.tokens))
        .collect()
}

#[proc_macro]
pub fn any_set_body(input: TokenStream) -> TokenStream {
    let stream = Stream::parse(input);
    let stream = stream
        .decls
        .into_iter()
        .map(|decl| call_require_envs(stream.krate.clone(), "get_res", decl.tokens))
        .flat_map(|x| [x, TokenStream::from_str(",").unwrap()])
        .collect();
    TokenTree::Group(Group::new(Delimiter::Bracket, stream)).into()
}

#[proc_macro]
pub fn help_body(input: TokenStream) -> TokenStream {
    let stream = Stream::parse(input);
    let stream = stream
        .decls
        .into_iter()
        .map(|decl| call_require_envs(stream.krate.clone(), "etext", decl.tokens))
        .flat_map(|x| [x, TokenStream::from_str(",").unwrap()])
        .collect();
    TokenTree::Group(Group::new(Delimiter::Bracket, stream)).into()
}

#[proc_macro]
pub fn getters(input: TokenStream) -> TokenStream {
    let stream = Stream::parse(input);
    stream
        .decls
        .into_iter()
        .map(|decl| call_require_envs(stream.krate.clone(), "func", decl.tokens))
        .collect()
}

#[proc_macro]
pub fn errors(input: TokenStream) -> TokenStream {
    let stream = Stream::parse(input);
    stream.errors.into_iter().collect()
}

fn compile_error(text: &str, span: Span) -> TokenStream {
    let toks = TokenStream::from_str(&format!("\"{text}\"")).unwrap();
    let toks = toks
        .into_iter()
        .map(|mut tok| {
            tok.set_span(span);
            tok
        })
        .collect();
    toks
}

#[proc_macro]
pub fn trimmed_help(input: TokenStream) -> TokenStream {
    // let dbg = format!("{input:?}");
    let mut input = input.into_iter();
    // `require_envs!` always deals with making sure this receives
    // a literal token (with some number of invisible groups wrapping it),
    // but we need to handle user input failures beyond that,
    // of which there is exactly one:
    // - passing something other than a string literal
    let mut walk = input.next().unwrap();
    while let TokenTree::Group(group) = walk {
        walk = group.stream().into_iter().next().unwrap();
    }
    let TokenTree::Literal(lit) = walk else {
        // eprintln!("{dbg}");
        unreachable!("internals failed to only pass help text")
    };
    let mut errors = Vec::<TokenStream>::new();
    let mut output = Vec::new();

    'parse: {
        let lit = match lit_parse::Literal::parse(&lit) {
            Ok(x) => x,
            Err(e) => {
                errors.extend(e);
                break 'parse;
            }
        };
        let lit_parse::LiteralData::String(lit_data) = lit.data() else {
            errors.push(compile_error(
                "attempted to pass a non-string literal as help text",
                lit.span(),
            ));
            break 'parse;
        };

        let mut new_lit = Literal::string(lit_data.trim());
        new_lit.set_span(lit.span());
        output.push(TokenStream::from(TokenTree::Literal(new_lit)))
    }

    output.extend(errors);
    output.into_iter().collect()
}
