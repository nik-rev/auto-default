//! [![crates.io](https://img.shields.io/crates/v/auto-default?style=flat-square&logo=rust)](https://crates.io/crates/auto-default)
//! [![docs.rs](https://img.shields.io/badge/docs.rs-auto-default-blue?style=flat-square&logo=docs.rs)](https://docs.rs/auto-default)
//! ![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)
//! ![msrv](https://img.shields.io/badge/msrv-nightly-blue?style=flat-square&logo=rust)
//! [![github](https://img.shields.io/github/stars/nik-rev/auto-default)](https://github.com/nik-rev/auto-default)
//!
//! This crate provides an attribute macro `#[auto_default]`, which adds a default field value of
//! `Default::default()` to fields that do not have one.
//!
//! ```toml
//! [dependencies]
//! auto-default = "0.1"
//! ```
//!
//! Note: `auto-default` has *zero* dependencies. Not even `syn`! The compile times are very fast.
//!
//! ## Showcase
//!
//! Rust's [default field values](https://github.com/rust-lang/rust/issues/132162) allow
//! the shorthand `Struct { field, .. }` instead of the lengthy `Struct { field, ..Default::default() }`
//!
//! For `..` instead of `..Default::default()` to work,
//! your `Struct` needs **all** fields to have a default value.
//!
//! This often means `= Default::default()` boilerplate on every field, because it is
//! very common to want field defaults to be the value of their `Default` implementation
//!
//! ### Before
//!
//! ```rust
//! # #![feature(default_field_values)]
//! # #![feature(const_trait_impl)]
//! # #![feature(const_default)]
//! # #![feature(derive_const)]
//! # use auto_default::auto_default;
//! # #[derive_const(Default)]
//! # struct Rect { value: f32 }
//! # #[derive_const(Default)]
//! # struct Size { value: f32 }
//! # #[derive_const(Default)]
//! # struct Point { value: f32 }
//! #[derive(Default)]
//! pub struct Layout {
//!     order: u32 = Default::default(),
//!     location: Point = Default::default(),
//!     size: Size = Default::default(),
//!     content_size: Size = Default::default(),
//!     scrollbar_size: Size = Default::default(),
//!     border: Rect = Default::default(),
//!     padding: Rect = Default::default(),
//!     margin: Rect = Default::default(),
//! }
//! ```
//!
//! ### With `#[auto_default]`
//!
//! ```rust
//! # #![feature(default_field_values)]
//! # #![feature(const_trait_impl)]
//! # #![feature(const_default)]
//! # #![feature(derive_const)]
//! # use auto_default::auto_default;
//! # #[derive_const(Default)]
//! # struct Rect { value: f32 }
//! # #[derive_const(Default)]
//! # struct Size { value: f32 }
//! # #[derive_const(Default)]
//! # struct Point { value: f32 }
//! #[auto_default]
//! #[derive(Default)]
//! pub struct Layout {
//!     order: u32,
//!     location: Point,
//!     size: Size,
//!     content_size: Size,
//!     scrollbar_size: Size,
//!     border: Rect,
//!     padding: Rect,
//!     margin: Rect,
//! }
//! ```
//!
//! You can apply the [`#[auto_default]`](macro@auto_default) macro to `struct`s with named fields, and `enum`s.
//!
//! If any field or variant has the `#[auto_default(skip)]` attribute, a default field value of `Default::default()`
//! will not be added
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
///
/// This macro applies to `struct`s with named fields, and enums.
///
/// # Do not add `= Default::default()` field value to select fields
///
/// If you do not want a specific field to have a default, you can opt-out
/// with `#[auto_default(skip)]`:
///
/// ```rust
/// # #![feature(default_field_values)]
/// # #![feature(const_trait_impl)]
/// # #![feature(const_default)]
/// #[auto_default]
/// struct User {
///     #[auto_default(skip)]
///     age: u8,
///     is_admin: bool
/// }
/// # use auto_default::auto_default;
/// ```
///
/// The above is transformed into this:
///
/// ```rust
/// # #![feature(default_field_values)]
/// # #![feature(const_trait_impl)]
/// # #![feature(const_default)]
/// struct User {
///     age: u8,
///     is_admin: bool = Default::default()
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

    stream_attrs(
        &mut source,
        &mut sink,
        &mut compile_errors,
        // no skip allowed on the container, would make no sense
        // (just don't use the `#[auto_default]` at all at that point!)
        IsSkipAllowed(false),
    );
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
            sink.extend([add_default_field_values(
                source_item_fields,
                &mut compile_errors,
                // none of the fields are considered to be skipped initially
                IsSkip(false),
            )]);
        }
        ItemKind::Enum => {
            let mut source_variants = source_item_fields.stream().into_iter().peekable();
            let mut sink_variants = TokenStream::new();

            loop {
                // if this variant is marked #[auto_default(skip)]
                let is_skip = stream_attrs(
                    &mut source_variants,
                    &mut sink_variants,
                    &mut compile_errors,
                    // can skip the variant, which removes auto-default for all
                    // fields
                    IsSkipAllowed(true),
                );

                // variants technically can have visibility, at least on a syntactic level
                //
                // pub Variant {  }
                // ^^^
                stream_vis(&mut source_variants, &mut sink_variants);

                // Variant {  }
                // ^^^^^^^
                let Some(variant_ident_span) =
                    stream_ident(&mut source_variants, &mut sink_variants)
                else {
                    // that means we have an enum with no variants, e.g.:
                    //
                    // enum Never {}
                    //
                    // When we parse the variants, there won't be an identifier
                    break;
                };

                // only variants with named fields can be marked `#[auto_default(skip)]`
                let mut disallow_skip = || {
                    if is_skip.0 {
                        compile_errors.extend(CompileError::new(
                            variant_ident_span,
                            concat!(
                                "`#[auto_default(skip)]` is",
                                " only allowed on variants with named fields"
                            ),
                        ));
                    }
                };

                match source_variants.peek() {
                    // Enum variant with named fields. Add default field values.
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                        let Some(TokenTree::Group(named_variant_fields)) = source_variants.next()
                        else {
                            unreachable!()
                        };
                        sink_variants.extend([add_default_field_values(
                            named_variant_fields,
                            &mut compile_errors,
                            is_skip,
                        )]);

                        stream_enum_variant_discriminant_and_comma(
                            &mut source_variants,
                            &mut sink_variants,
                        );
                    }
                    // Enum variant with unnamed fields.
                    Some(TokenTree::Group(group))
                        if group.delimiter() == Delimiter::Parenthesis =>
                    {
                        disallow_skip();
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
                        disallow_skip();
                        stream_enum_variant_discriminant_and_comma(
                            &mut source_variants,
                            &mut sink_variants,
                        );
                    }
                    // Unit variant, with no comma at the end. This is the last variant
                    None => {
                        disallow_skip();
                        break;
                    }
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

struct IsSkip(bool);
struct IsSkipAllowed(bool);

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
//
// Returns `true` if `#[auto_default(skip)]` was encountered
fn stream_attrs(
    source: &mut Source,
    sink: &mut Sink,
    errors: &mut TokenStream,
    is_skip_allowed: IsSkipAllowed,
) -> IsSkip {
    let mut is_skip = None;

    let is_skip = loop {
        if !matches!(source.peek(), Some(TokenTree::Punct(hash)) if *hash == '#') {
            break is_skip;
        };

        // #[some_attr]
        // ^
        let pound = source.next();

        // #[some_attr]
        //  ^^^^^^^^^^^
        let Some(TokenTree::Group(attr)) = source.next() else {
            unreachable!()
        };

        // #[some_attr = hello]
        //   ^^^^^^^^^^^^^^^^^
        let mut attr_tokens = attr.stream().into_iter().peekable();

        // Check if this attribute is `#[auto_default(skip)]`
        if let Some(skip_span) = is_skip_attribute(&mut attr_tokens, errors) {
            if is_skip.is_some() {
                // Disallow 2 attributes on a single field:
                //
                // #[auto_default(skip)]
                // #[auto_default(skip)]
                errors.extend(CompileError::new(
                    skip_span,
                    "duplicate `#[auto_default(skip)]`",
                ));
            } else {
                is_skip = Some(skip_span);
            }
            continue;
        }

        // #[attr]
        // ^
        sink.extend(pound);

        // Re-construct the `[..]` for the attribute
        //
        // #[attr]
        //  ^^^^^^
        let mut group = Group::new(attr.delimiter(), attr_tokens.collect());
        group.set_span(attr.span());

        // #[attr]
        //  ^^^^^^
        sink.extend([group]);
    };

    if let Some(skip_span) = is_skip
        && !is_skip_allowed.0
    {
        errors.extend(CompileError::new(
            skip_span,
            "`#[auto_default(skip)]` is not allowed on container",
        ));
    }

    IsSkip(is_skip.is_some())
}

/// if `source` is exactly `auto_default(skip)`, returns `Some(span)`
/// with `span` being the `Span` of the `skip` identifier
fn is_skip_attribute(source: &mut Source, errors: &mut TokenStream) -> Option<Span> {
    let Some(TokenTree::Ident(ident)) = source.peek() else {
        return None;
    };

    if ident.to_string() != "auto_default" {
        return None;
    };

    // #[auto_default(skip)]
    //   ^^^^^^^^^^^^
    let ident = source.next().unwrap();

    // We know it is `#[auto_default ???]`, we need to validate that `???`
    // is exactly `(skip)` now

    // #[auto_default(skip)]
    //   ^^^^^^^^^^^^
    let auto_default_span = ident.span();

    // #[auto_default(skip)]
    //               ^^^^^^
    let group = match source.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => group,
        Some(tt) => {
            errors.extend(CompileError::new(tt.span(), "expected `(skip)`"));
            return None;
        }
        None => {
            errors.extend(CompileError::new(
                auto_default_span,
                "expected `(skip)` after this",
            ));
            return None;
        }
    };

    // #[auto_default(skip)]
    //                ^^^^
    let mut inside = group.stream().into_iter();

    // #[auto_default(skip)]
    //                ^^^^
    let ident_skip = match inside.next() {
        Some(TokenTree::Ident(ident)) => ident,
        Some(tt) => {
            errors.extend(CompileError::new(tt.span(), "expected `skip`"));
            return None;
        }
        None => {
            errors.extend(CompileError::new(
                group.span(),
                "expected `(skip)`, found `()`",
            ));
            return None;
        }
    };

    if ident_skip.to_string() != "skip" {
        errors.extend(CompileError::new(ident_skip.span(), "expected `skip`"));
        return None;
    }

    // Validate that there's nothing after `skip`
    //
    // #[auto_default(skip    )]
    //                    ^^^^
    if let Some(tt) = inside.next() {
        errors.extend(CompileError::new(tt.span(), "unexpected token"));
        return None;
    }

    Some(ident_skip.span())
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
fn add_default_field_values(
    fields: Group,
    compile_errors: &mut TokenStream,
    is_skip_variant: IsSkip,
) -> Group {
    // All the tokens corresponding to the struct's field, passed by the user
    // These tokens will eventually all be sent to `output_fields`,
    // plus a few extra for any `Default::default()` that we output
    let mut input_fields = fields.stream().into_iter().peekable();

    // The tokens corresponding to the fields of the output struct
    let mut output_fields = TokenStream::new();

    // Parses all fields.
    // Each iteration parses a single field
    'parse_field: loop {
        let is_skip_field = stream_attrs(
            &mut input_fields,
            &mut output_fields,
            compile_errors,
            IsSkipAllowed(true),
        );
        let is_skip = is_skip_field.0 || is_skip_variant.0;
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
                    if !is_skip {
                        output_fields.extend(default(field_ident_span));
                    }
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
                    if !is_skip {
                        output_fields.extend(default(field_ident_span));
                    }
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
