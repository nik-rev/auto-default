//! Rust's [default field values](https://github.com/rust-lang/rust/issues/132162) allow
//! the shorthand `Struct { field, .. }` instead of the lengthy `Struct { field, ..Default::default() }`
//!
//! For this syntax to work (`..` instead of `..Default::default()`),
//! your `Struct` needs **all** fields to have a default value.
//!
//! This often means lots of `= Default::default()` boilerplate on every field, because it is
//! very common to want field defaults to be the value of their `Default` implementation
//!
//! This crate provides an attribute macro `#[auto_default]`, which adds `= Default::default()` to every
//! field that does not have a default value.
//!
//! <table>
//! <tr>
//! <th>Before</th>
//! <th>After</th>
//! </tr>
//! <tr>
//! <td>
//!
//! ```rust
//! #[derive(Default)]
//! pub struct Layout {
//!     order: u32 = Default::default(),
//!     location: Point<f32> = Default::default(),
//!     size: Size<f32> = Default::default(),
//!     content_size: Size<f32> = Default::default(),
//!     scrollbar_size: Size<f32> = Default::default(),
//!     border: Rect<f32> = Default::default(),
//!     padding: Rect<f32> = Default::default(),
//!     margin: Rect<f32> = Default::default(),
//! }
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust
//! #[auto_default]
//! pub struct Layout {
//!     order: u32,
//!     location: Point<f32>,
//!     size: Size<f32>,
//!     content_size: Size<f32>,
//!     scrollbar_size: Size<f32>,
//!     border: Rect<f32>,
//!     padding: Rect<f32>,
//!     margin: Rect<f32>,
//! }
//! ```
//!
//! </td>
//! </tr>
//! </table>

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
    let mut source = input.into_iter().peekable();

    // We collect all tokens into here and then return this
    let mut sink = TokenStream::new();

    stream_attrs(&mut source, &mut sink);
    stream_vis(&mut source, &mut sink);

    // pub(in crate) struct Foo
    //               ^^^^^^
    let item_kind = match source.next() {
        Some(TokenTree::Ident(kw)) if kw.to_string() == "struct" => {
            sink.extend([kw]);
            ItemKind::Struct
        }
        Some(TokenTree::Ident(kw)) if kw.to_string() == "enum" => {
            sink.extend([kw]);
            ItemKind::Enum
        }
        tt => {
            compile_errors.extend(create_compile_error!(
                tt,
                "expected a `struct` or an `enum`"
            ));
            return compile_errors;
        }
    };

    // struct Foo
    //        ^^^
    let item_ident_span = stream_ident(&mut source, &mut sink)
        .expect("`struct` or `enum` keyword is always followed by an identifier");

    // Generics
    //
    // struct Foo<Bar, Baz: Trait> where Baz: Quux { ... }
    //           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    let source_item_fields = loop {
        match source.next() {
            // Fields of the struct
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => break group,
            // This token is part of the generics of the struct
            Some(tt) => sink.extend([tt]),
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
            sink.extend([add_default_field_values(source_item_fields)]);
        }
        ItemKind::Enum => {
            let mut source_variants = source_item_fields.stream().into_iter().peekable();
            let mut sink_variants = TokenStream::new();

            loop {
                stream_attrs(&mut source_variants, &mut sink_variants);
                stream_vis(&mut source_variants, &mut sink_variants);
                let _ = stream_ident(&mut source_variants, &mut sink_variants);
                match source_variants.peek() {
                    // Enum variant with named fields. Add default field values.
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                        let Some(TokenTree::Group(named_variant_fields)) = source_variants.next()
                        else {
                            unreachable!()
                        };
                        sink_variants.extend([add_default_field_values(named_variant_fields)]);

                        stream_enum_variant_discriminant_and_comma(
                            &mut source_variants,
                            &mut sink_variants,
                        );
                    }
                    // Enum variant with unnamed fields.
                    Some(TokenTree::Group(group))
                        if group.delimiter() == Delimiter::Parenthesis =>
                    {
                        let Some(TokenTree::Group(unnamed_variant_fields)) = source_variants.next()
                        else {
                            unreachable!()
                        };
                        sink_variants.extend([unnamed_variant_fields]);

                        stream_enum_variant_discriminant_and_comma(
                            &mut source_variants,
                            &mut sink_variants,
                        );
                    }
                    // This was a unit variant. Next variant may exist,
                    // if it does it is parsed on next iteration
                    Some(TokenTree::Punct(punct))
                        if punct.as_char() == ',' || punct.as_char() == '=' =>
                    {
                        stream_enum_variant_discriminant_and_comma(
                            &mut source_variants,
                            &mut sink_variants,
                        );
                    }
                    // Unit variant, with no comma at the end. This is the last variant
                    None => break,
                    Some(_) => unreachable!(),
                }
            }

            let mut sink_variants = Group::new(source_item_fields.delimiter(), sink_variants);
            sink_variants.set_span(source_item_fields.span());
            sink.extend([sink_variants]);
        }
    }

    sink.extend(compile_errors);

    sink
}

/// Streams enum variant discriminant + comma at the end from `source` into `sink`
///
/// enum Example {
///     Three,
///          ^
///     Two(u32) = 2,
///             ^^^^^
///     Four { hello: u32 } = 4,
///                        ^^^^^
/// }
fn stream_enum_variant_discriminant_and_comma(source: &mut Source, sink: &mut Sink) {
    match source.next() {
        // No discriminant, there may be another variant after this
        Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
            sink.extend([punct]);
        }
        // No discriminant, this is the final enum variant
        None => {}
        // Enum variant has a discriminant
        Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
            sink.extend([punct]);

            // Stream discriminant expression from `source` into `sink`
            loop {
                match source.next() {
                    // End of discriminant, there may be a variant after this
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                        sink.extend([punct]);
                        break;
                    }
                    // This token is part of the variant's expression
                    Some(tt) => {
                        sink.extend([tt]);
                    }
                    // End of discriminant, this is the last variant
                    None => break,
                }
            }
        }
        Some(_) => unreachable!(),
    }
}

type Source = Peekable<proc_macro::token_stream::IntoIter>;
type Sink = TokenStream;

/// Streams the identifier from `input` into `output`, returning its span, if the identifier exists
fn stream_ident(source: &mut Source, sink: &mut Sink) -> Option<Span> {
    let ident = source.next()?;
    let span = ident.span();
    sink.extend([ident]);
    Some(span)
}

// Parses attributes
//
// #[attr] #[attr] pub field: Type
// #[attr] #[attr] struct Foo
// #[attr] #[attr] enum Foo
fn stream_attrs(source: &mut Source, sink: &mut Sink) {
    loop {
        if !matches!(source.peek(), Some(TokenTree::Punct(hash)) if *hash == '#') {
            break;
        };
        // #[attr]
        // ^
        sink.extend(source.next());
        // #[attr]
        //  ^^^^^^
        sink.extend(source.next());
    }
}

fn stream_vis(source: &mut Source, sink: &mut Sink) {
    // Remove visibility if it is present
    //
    // pub(in crate) struct
    // ^^^^^^^^^^^^^
    if let Some(TokenTree::Ident(vis)) = source.peek()
        && vis.to_string() == "pub"
    {
        // pub(in crate) struct
        // ^^^
        sink.extend(source.next());

        if let Some(TokenTree::Group(group)) = source.peek()
            && let Delimiter::Parenthesis = group.delimiter()
        {
            // pub(in crate) struct
            //    ^^^^^^^^^^
            sink.extend(source.next());
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
        stream_attrs(&mut input_fields, &mut output_fields);
        stream_vis(&mut input_fields, &mut output_fields);
        let Some(field_ident_span) = stream_ident(&mut input_fields, &mut output_fields) else {
            // No fields. e.g.: `struct Struct {}`
            break;
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
