use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

/// Adds a default field value of `Default::default()` to fields that don't have one
///
/// # Example
///
/// Turns this:
///
/// ```rust
/// # #![feature(default_field_values)]
/// # #![feature(const_trait_impl)]
/// # #![feature(const_default)]
/// #[auto_default]
/// struct User {
///     age: u8,
///     is_admin: bool = false
/// }
/// # use auto_default::auto_default;
/// ```
///
/// Into this:
///
/// ```rust
/// # #![feature(default_field_values)]
/// # #![feature(const_trait_impl)]
/// # #![feature(const_default)]
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

    // This loop parses generics, where clause, and fields of the struct
    loop {
        match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let mut fields = TokenStream::new();

                let mut input = group.stream().into_iter().peekable();

                // This loop parses all fields. Each iteration parses
                // a single field
                'parse_field: loop {
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
                            None => break 'parse_field,
                        }
                    };

                    // Field has visibility
                    //
                    // pub(in crate) field: Type
                    // ^^^^^^^^^^^^^
                    let field_ident_span = if let TokenTree::Ident(ref ident) = tt_after_attributes
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
                        let field_ident = input.next().expect("field identifier");
                        let span = field_ident.span();

                        // pub(in crate) field: Type
                        //               ^^^^^
                        fields.extend([field_ident]);
                        span
                    }
                    // No visibility
                    else {
                        let field_ident = tt_after_attributes;
                        let span = field_ident.span();
                        // field: Type
                        // ^^^^^
                        fields.extend([field_ident]);
                        span
                    };

                    // field: Type
                    //      ^
                    fields.extend(input.next());

                    // Everything after the `:` in the field
                    //
                    // Involves:
                    //
                    // - Adding default value of `= Default::default()` if one is not present
                    // - Continue to next iteration of the loop
                    loop {
                        match input.peek() {
                            // This field has a custom default field value
                            //
                            // field: Type = default
                            //             ^
                            Some(TokenTree::Punct(p)) if p.as_char() == '=' => loop {
                                match input.next() {
                                    Some(TokenTree::Punct(p)) if p == ',' => {
                                        fields.extend([p]);
                                        // Comma after field. Field is finished.
                                        continue 'parse_field;
                                    }
                                    Some(tt) => fields.extend([tt]),
                                    // End of input. Field is finished. This is the last field
                                    None => break 'parse_field,
                                }
                            },
                            // Reached end of field, has comma at the end, no custom default value
                            //
                            // field: Type,
                            //            ^
                            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                                // Insert default value before the comma
                                //
                                // field: Type = Default::default(),
                                //             ^^^^^^^^^^^^^^^^^^^^
                                fields.extend(default(field_ident_span));
                                // field: Type = Default::default(),
                                //                                 ^
                                fields.extend(input.next());
                                // Next iteration handles the next field
                                continue 'parse_field;
                            }
                            // This token is part of the field's type
                            //
                            // field: some::Type
                            //              ^^^^
                            Some(_) => fields.extend(input.next()),
                            // Reached end of input, and it has no comma.
                            // This is the last field.
                            //
                            // struct Foo {
                            //     field: Type
                            //                ^
                            // }
                            None => {
                                fields.extend(default(field_ident_span));
                                // No more fields
                                break 'parse_field;
                            }
                        }
                    }
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
fn default(_span: Span) -> [TokenTree; 14] {
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
