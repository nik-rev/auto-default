use proc_macro::{Delimiter, Group, Ident, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn autodefault(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        panic!("No arguments expected")
    }

    let mut input = input.into_iter().peekable();
    let mut output = TokenStream::new();

    // Remove visibility if it is present
    //
    // pub(in crate) struct
    // ^^^^^^^^^^^^^
    if let Some(TokenTree::Ident(vis)) = input.peek()
        && vis.to_string() == "pub"
    {
        // pub(in crate) struct
        // ^^^
        let vis = input.next().unwrap();
        output.extend([vis]);

        if let Some(TokenTree::Group(group)) = input.peek()
            && let Delimiter::Parenthesis = group.delimiter()
        {
            // pub(in crate) struct
            //    ^^^^^^^^^^
            let restriction = input.next().unwrap();
            output.extend([restriction]);
        }
    };

    if let Some(TokenTree::Ident(kw)) = input.next()
        && kw.to_string() == "struct"
    {
        output.extend([kw]);
    } else {
        panic!("expected a `struct`")
    };

    if let Some(TokenTree::Ident(struct_ident)) = input.next() {
        output.extend([struct_ident]);
    } else {
        panic!("expected a `struct`")
    };

    loop {
        match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let mut fields = TokenStream::new();

                let mut fields_stream = group.stream().into_iter().peekable();

                let mut g = Group::new(Delimiter::Brace, fields);
                g.set_span(g.span());
                output.extend([g]);
                break;
            }
            Some(tt) => output.extend([tt]),
            // reached end of input
            None => panic!("expected struct with named fields"),
        }
    }

    output
}
