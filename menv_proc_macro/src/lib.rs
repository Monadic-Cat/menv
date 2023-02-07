// assert_var_body, any_set_body, help_body, getters

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};
use std::str::FromStr;

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
