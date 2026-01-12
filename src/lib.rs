use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

/// Adds a default field value of `Default:default()` to fields that don't have one
///
/// Turns this:
///
/// ```rust
/// #[auto_default]
/// struct User {
///     age: u8,
///     is_admin: bool = false
/// }
/// ```
///
/// Into this:
///
/// ```rust
/// #[auto_default]
/// struct User {
///     age: u8 = Default::default(),
///     is_admin: bool = false
/// }
/// ```
#[proc_macro_attribute]
pub fn auto_default(args: TokenStream, input: TokenStream) -> TokenStream {
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

                loop {
                    // This loop parses attributes on the field
                    //
                    // #[attr] #[attr] pub field: Type
                    //                ^ after the attributes
                    let tt_after_attributes = loop {
                        match input.next() {
                            // this is an attribute: #[attr]
                            Some(TokenTree::Punct(hash)) if hash == '#' => {
                                // #[attr]
                                // ^
                                fields.extend([hash]);
                                // #[attr]
                                //  ^^^^^^
                                fields.extend(input.next());
                            }
                            Some(tt) => break tt,
                            None => panic!("unexpected end of input"),
                        }
                    };

                    // Field has visibility
                    //
                    // pub(in crate) field: Type
                    // ^^^^^^^^^^^^^
                    if let TokenTree::Ident(ref ident) = tt_after_attributes
                        && ident.to_string() == "pub"
                    {
                        let kw_pub = tt_after_attributes;
                        // pub(in crate)
                        // ^^^
                        fields.extend([kw_pub]);
                        if let Some(TokenTree::Group(group)) = input.next() {
                            // pub(in crate)
                            //    ^^^^^^^^^^
                            fields.extend([group]);
                        }

                        // pub(in crate) field: Type
                        //               ^^^^^
                        fields.extend(input.next());
                    }
                    // No visibility
                    else {
                        let field_ident = tt_after_attributes;
                        // field: Type
                        // ^^^^^
                        fields.extend([field_ident]);
                    };

                    // field: Type
                    //      ^
                    fields.extend(input.next());

                    // field: Type
                    //        ^^^^
                    loop {
                        match input.next() {
                            // Part of the type
                            Some(tt) => fields.extend([tt]),
                            // Reached end of input. No comma.
                            //
                            // struct {
                            //     field: Type
                            //                ^
                            // }
                            None => {
                                fields.extend(default());
                                break;
                            }
                        }
                    }

                    todo!()
                }

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

// = ::core::default::Default::default()
fn default() -> [TokenTree; 14] {
    [
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Ident(Ident::new("core", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Ident(Ident::new("default", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Ident(Ident::new("Default", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Ident(Ident::new("default", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
    ]
}
