use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

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
    let mut compile_errors = TokenStream::new();

    if !args.is_empty() {
        compile_errors.extend(create_compile_error!(
            args.into_iter().next(),
            "no arguments expected",
        ));
    }

    // Input supplied by the user. All tokens from here will
    // get sent back to `output`
    let mut input = input.into_iter().peekable();

    // We collect all tokens into here and then return this
    let mut output = TokenStream::new();

    parse_attributes(&mut input, &mut output);
    parse_vis(&mut input, &mut output);

    // pub(in crate) struct Foo
    //               ^^^^^^
    let item_kind = match input.next() {
        Some(TokenTree::Ident(kw)) if kw.to_string() == "struct" => {
            output.extend([kw]);
            ItemKind::Struct
        }
        Some(TokenTree::Ident(kw)) if kw.to_string() == "enum" => {
            output.extend([kw]);
            ItemKind::Enum
        }
        tt => {
            compile_errors.extend(create_compile_error!(tt, "expected a `struct`"));
            return compile_errors;
        }
    };

    // struct Foo
    //        ^^^
    let Some(TokenTree::Ident(item_ident)) = input.next() else {
        unreachable!("`struct` keyword is always followed by an identifier")
    };
    let item_ident_span = item_ident.span();
    output.extend([item_ident]);

    // Generics
    //
    // struct Foo<Bar, Baz: Trait> where Baz: Quux { ... }
    //           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    let item_fields = loop {
        match input.next() {
            // Fields of the struct
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => break group,
            // This token is part of the generics of the struct
            Some(tt) => output.extend([tt]),
            // reached end of input
            None => {
                // note: if enum, this is unreachable because `enum Foo` is invalid (requires `{}`),
                // whilst `struct Foo;` is completely valid
                compile_errors.extend(CompileError::new(
                    item_ident_span,
                    "expected struct with named fields",
                ));
                return compile_errors;
            }
        }
    };

    match item_kind {
        ItemKind::Struct => {
            let output_fields = add_default_field_values(item_fields);
            output.extend([output_fields]);
        }
        ItemKind::Enum => {
            todo!()
        }
    }

    output.extend(compile_errors);

    output
}

type Input = Peekable<proc_macro::token_stream::IntoIter>;

// Parses attributes
//
// #[attr] #[attr] pub field: Type
// #[attr] #[attr] struct Foo
// #[attr] #[attr] enum Foo
fn parse_attributes(input: &mut Input, output: &mut TokenStream) {
    loop {
        if !matches!(input.peek(), Some(TokenTree::Punct(hash)) if *hash == '#') {
            break;
        };
        // #[attr]
        // ^
        output.extend(input.next());
        // #[attr]
        //  ^^^^^^
        output.extend(input.next());
    }
}

fn parse_vis(input: &mut Input, output: &mut TokenStream) {
    // Remove visibility if it is present
    //
    // pub(in crate) struct
    // ^^^^^^^^^^^^^
    if let Some(TokenTree::Ident(vis)) = input.peek()
        && vis.to_string() == "pub"
    {
        // pub(in crate) struct
        // ^^^
        output.extend(input.next());

        if let Some(TokenTree::Group(group)) = input.peek()
            && let Delimiter::Parenthesis = group.delimiter()
        {
            // pub(in crate) struct
            //    ^^^^^^^^^^
            output.extend(input.next());
        }
    };
}

#[derive(PartialEq)]
enum ItemKind {
    Struct,
    Enum,
}

/// `fields` is [`StructFields`] in the grammar.
///
/// It is the curly braces, and everything within, for a struct with named fields,
/// or an enum variant with named fields.
///
/// These fields are transformed by adding `= Default::default()` to every
/// field that doesn't already have a default value.
///
/// If a field is marked with `#[auto_default(skip)]`, no default value will be
/// added
///
/// [`StructFields`]: https://doc.rust-lang.org/reference/items/structs.html#grammar-StructFields
fn add_default_field_values(fields: Group) -> Group {
    // All the tokens corresponding to the struct's field, passed by the user
    // These tokens will eventually all be sent to `output_fields`,
    // plus a few extra for any `Default::default()` that we output
    let mut input_fields = fields.stream().into_iter().peekable();

    // The tokens corresponding to the fields of the output struct
    let mut output_fields = TokenStream::new();

    // Parses all fields.
    // Each iteration parses a single field
    'parse_field: loop {
        // Parses attributes on the field
        //
        // #[attr] #[attr] pub field: Type
        //                ^ after the attributes
        let tt_after_attributes = loop {
            match input_fields.next() {
                // this is an attribute: #[attr]
                Some(TokenTree::Punct(hash)) if hash == '#' => {
                    // #[attr]
                    // ^
                    output_fields.extend([hash]);
                    // #[attr]
                    //  ^^^^^^
                    output_fields.extend(input_fields.next());
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
            output_fields.extend([kw_pub]);
            if let Some(TokenTree::Group(group)) = input_fields.next() {
                // pub(in crate)
                //    ^^^^^^^^^^
                output_fields.extend([group]);
            }
            let field_ident = input_fields.next().expect("field identifier");
            let span = field_ident.span();

            // pub(in crate) field: Type
            //               ^^^^^
            output_fields.extend([field_ident]);
            span
        }
        // No visibility
        else {
            let field_ident = tt_after_attributes;
            let span = field_ident.span();
            // field: Type
            // ^^^^^
            output_fields.extend([field_ident]);
            span
        };

        // field: Type
        //      ^
        output_fields.extend(input_fields.next());

        // Everything after the `:` in the field
        //
        // Involves:
        //
        // - Adding default value of `= Default::default()` if one is not present
        // - Continue to next iteration of the loop
        loop {
            match input_fields.peek() {
                // This field has a custom default field value
                //
                // field: Type = default
                //             ^
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => loop {
                    match input_fields.next() {
                        Some(TokenTree::Punct(p)) if p == ',' => {
                            output_fields.extend([p]);
                            // Comma after field. Field is finished.
                            continue 'parse_field;
                        }
                        Some(tt) => output_fields.extend([tt]),
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
                    output_fields.extend(default(field_ident_span));
                    // field: Type = Default::default(),
                    //                                 ^
                    output_fields.extend(input_fields.next());
                    // Next iteration handles the next field
                    continue 'parse_field;
                }
                // This token is part of the field's type
                //
                // field: some::Type
                //              ^^^^
                Some(_) => output_fields.extend(input_fields.next()),
                // Reached end of input, and it has no comma.
                // This is the last field.
                //
                // struct Foo {
                //     field: Type
                //                ^
                // }
                None => {
                    output_fields.extend(default(field_ident_span));
                    // No more fields
                    break 'parse_field;
                }
            }
        }
    }
    let mut g = Group::new(Delimiter::Brace, output_fields);
    g.set_span(fields.span());
    g
}

// = ::core::default::Default::default()
fn default(span: Span) -> [TokenTree; 14] {
    [
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Ident(Ident::new("core", span)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Ident(Ident::new("default", span)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Ident(Ident::new("Default", span)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(span),
        TokenTree::Ident(Ident::new("default", span)),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())).with_span(span),
    ]
}

macro_rules! create_compile_error {
    ($spanned:expr, $($tt:tt)*) => {{
        let span = if let Some(spanned) = $spanned {
            spanned.span()
        } else {
            Span::call_site()
        };
        CompileError::new(span, format!($($tt)*))
    }};
}
use create_compile_error;

/// `.into_iter()` generates `compile_error!($message)` at `$span`
struct CompileError {
    /// Where the compile error is generates
    pub span: Span,
    /// Message of the compile error
    pub message: String,
}

impl CompileError {
    /// Create a new compile error
    pub fn new(span: Span, message: impl AsRef<str>) -> Self {
        Self {
            span,
            message: message.as_ref().to_string(),
        }
    }
}

impl IntoIterator for CompileError {
    type Item = TokenTree;
    type IntoIter = std::array::IntoIter<Self::Item, 8>;

    fn into_iter(self) -> Self::IntoIter {
        [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(self.span),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(self.span),
            TokenTree::Ident(Ident::new("core", self.span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(self.span),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)).with_span(self.span),
            TokenTree::Ident(Ident::new("compile_error", self.span)),
            TokenTree::Punct(Punct::new('!', Spacing::Alone)).with_span(self.span),
            TokenTree::Group(Group::new(Delimiter::Brace, {
                TokenStream::from(
                    TokenTree::Literal(Literal::string(&self.message)).with_span(self.span),
                )
            }))
            .with_span(self.span),
        ]
        .into_iter()
    }
}

trait TokenTreeExt {
    /// Set span of `TokenTree` without needing to create a new binding
    fn with_span(self, span: Span) -> TokenTree;
}

impl TokenTreeExt for TokenTree {
    fn with_span(mut self, span: Span) -> TokenTree {
        self.set_span(span);
        self
    }
}
