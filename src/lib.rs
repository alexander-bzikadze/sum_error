//! Here we present a library for convinient summing already defined unique Errors.
//! We provide a derive macros that can be applied towards a
//! [enum](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html) of unnamed
//! variants each containing single type that implements
//! [Error](https://doc.rust-lang.org/std/error/trait.Error.html) trait.
//!
//! To provide more details, deriving from SumError will implement
//! [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html),
//! [Display](https://doc.rust-lang.org/std/fmt/trait.Display.html),
//! [Error](https://doc.rust-lang.org/std/error/trait.Error.html) and
//! [From](https://doc.rust-lang.org/std/convert/trait.From.html)
//! traits using contained errors.
//!
//! # Examples
//! The macros was designed to work in pair with Try paradigm.
//! It is often tiring when different called methods return
//! [Results](https://doc.rust-lang.org/std/result/enum.Result.html)
//! with different Error types. In order to easily combinate and store them
//! one may use SumError derive macros to create a summing error type and
//! automatically convert all the possible errors to it and write all the
//! Try calls without spending spacetime on converting errors.
//!
//! Here goes a simple example.
//! ```rust
//! use std::fs::File;
//! use std::rc::Rc;
//! use sprite::Sprite;
//! use piston_window::Texture;
//! use image::gif::Decoder;
//! use image::AnimationDecoder;
//! use piston_window::{TextureContext, TextureSettings};
//! use sum_error::*;
//!
//! /// Load a gif sprite.
//! pub fn load<F: gfx::Factory<R>,
//!             R: gfx::Resources,
//!             C: gfx::CommandBuffer<R>>(ctx: &mut TextureContext<F, R, C>)
//!                 -> Result<Vec<Sprite<Texture<R>>>, CombineError> {
//!     let file = File::open("file.gif")?;
//!     let decoder = Decoder::new(file)?;
//!     let frames = decoder.into_frames().collect_frames()?;
//!     frames.iter()
//!         .map(|frame| {
//!             Texture::from_image(ctx, frame.buffer(), &TextureSettings::new())
//!                 .map(Rc::new)
//!                 .map(Sprite::from_texture)
//!                 .map_err(|e| e.into())
//!         }).collect()
//! }
//!
//! #[derive(SumError)]
//! pub enum CombineError {
//!     FileError(std::io::Error),
//!     ImageError(image::ImageError),
//!     GfxError(gfx_texture::Error),
//! }
//! ```
//!
//! # Generated Code
//! To provide simplest illustration of the derive macros work let us discuss
//! the following peace of code.
//! ```rust
//! #[derive(SumError)]
//! enum A {
//!     A(std::io::Error)
//! }
//! ```
//! In order to check in the compile time that contained types implemet
//! [Error](https://doc.rust-lang.org/std/error/trait.Error.html) trait we
//! generate this code.
//! ```rust
//! struct ___AssertNameA
//! where
//!    std::io::Error: std::error::Error;
//! ```
//! After that given types auto implements
//! [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html) and
//! [Display](https://doc.rust-lang.org/std/fmt/trait.Display.html)
//! traits.
//! ```rust
//! impl std::fmt::Debug for A {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             A::A(error) => write!("{:?}", error),
//!         }
//!     }
//! }
//!
//! impl std::fmt::Display for A {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             A::A(error) => write!("{}", error),
//!         }
//!     }
//! }
//! ```
//! The following generated code is an example of default
//! [Error](https://doc.rust-lang.org/std/error/trait.Error.html) trait
//! implementation.
//! ```rust
//! impl std::error::Error for A {
//!     fn description(&self) -> &str {
//!         match self {
//!             A::A(error) => error.description(),
//!         }
//!     }
//!     fn cause(&self) -> Option<&dyn std::error::Error> {
//!         match self {
//!             A::A(error) => Some(error),
//!         }
//!     }
//!     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//!         match self {
//!             A::A(error) => Some(error),
//!         }
//!     }
//! }
//! ```
//! The last generated auto trait is [From](https://doc.rust-lang.org/std/convert/trait.From.html).
//! ```rust
//! impl From<std::io::Error> for A {
//!     fn from(error: std::io::Error) -> Self {
//!         A::A(error)
//!     }
//! }
//! ```
//!

extern crate proc_macro;

use proc_macro::TokenStream as TS1;
use proc_macro2::TokenStream as TS2;
use syn::Data::Enum;
use syn::{DeriveInput, Fields, parse_macro_input};
use syn::spanned::Spanned;
use quote::{quote_spanned, quote, format_ident};

/// The whole point. Refer to the whole crate desription for a guide.
#[proc_macro_derive(SumError)]
pub fn derive_sum_error(stream: TS1) -> TS1 {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = input.ident;
    match input.data {
        Enum(enum_body) => enum_body
            .variants
            .iter()
            .fold(Ok(StreamHolder::new()), |holder_r, var| { holder_r.and_then(|mut holder| {
                match &var.fields {
                    Fields::Unnamed(fields) =>
                        Ok(&fields.unnamed)
                            .check(|u| { u.len() == 1 }, |_| {
                                "Invalid number of contained errors! Only one error allowed in any enum variant!"
                            }).and_then(|u| {
                                let var_name = &var.ident;

                                let assert_error = {
                                    let ty = &u[0].ty;
                                    let ty_span = ty.span();
                                    quote_spanned!(ty_span=> #ty: std::error::Error,)
                                };
                                holder.check_streams.push(assert_error);

                                let match_stream = {
                                    let name_local = name.clone();
                                    let var_name_local = var_name.clone();
                                    Box::new(move |process| {
                                        quote!(#name_local::#var_name_local(error) => #process,)
                                    })
                                };
                                holder.match_streams.push(match_stream);

                                let into_stream = {
                                    let name_local = name.clone();
                                    let var_name_local = var_name.clone();
                                    let ty = &u[0].ty;
                                    let ty_span = ty.span();
                                    quote_spanned! {ty_span=>
                                        impl From<#ty> for #name_local {
                                            fn from(error: #ty) -> Self {
                                                #name_local::#var_name_local(error)
                                            }
                                        }
                                    }
                                };
                                holder.into_streams.push(into_stream);

                                Ok(holder)
                            }),
                    _ => Err("Contained in variants errors should be unnamed!")
                }
            })}).map(|it| {
                let mut result = TS2::new();

                {
                    let mut local = TS2::new();
                    local.extend(it.check_streams);
                    let assert_name = format_ident!("___{}{}", "AssertName", name);
                    result.extend(vec![quote!{
                        struct #assert_name where #local;
                    }])
                }

                let local_debug = create_stream(&it.match_streams, quote!{ write!(f, "{:?}", error) });
                let local_display = create_stream(&it.match_streams, quote!{ write!(f, "{}", error) });
                let local_description = create_stream(&it.match_streams, quote!{ error.description() });
                let local_cause = create_stream(&it.match_streams, quote!{ Some(error) });
                let local_source = create_stream(&it.match_streams, quote!{ Some(error) });

                result.extend(vec![quote! {
                    impl std::fmt::Debug for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            match self {
                                #local_debug
                            }
                        }
                    }

                    impl std::fmt::Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            match self {
                                #local_display
                            }
                        }
                    }

                    impl std::error::Error for #name {
                        fn description(&self) -> &str {
                            match self {
                                #local_description
                            }
                        }

                        fn cause(&self) -> Option<&dyn std::error::Error> {
                            match self {
                                #local_cause
                            }
                        }

                        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                            match self {
                                #local_source
                            }
                        }
                    }
                }]);

                result.extend(it.into_streams);

                result
            }),
        _ => Err("Deriving from SumError is only avaliable for enums of errors!")
    }.map(|it| TS1::from(it))
    .unwrap()
}

type TemplateStreams = Vec<Box<dyn Fn(TS2) -> TS2>>;
struct StreamHolder {
    check_streams: Vec<TS2>,
    match_streams: TemplateStreams,
    into_streams: Vec<TS2>,
}

impl StreamHolder {
    fn new() -> Self {
        StreamHolder {
            check_streams: Vec::new(),
            match_streams: Vec::new(),
            into_streams: Vec::new(),
        }
    }
}

fn create_stream(streams: &TemplateStreams, t: TS2) -> TS2 {
    let mut local = TS2::new();
    local.extend(streams.iter().map(|mat| {mat(t.clone())}));
    local
}

trait ResultExtender<V, E, R> where Self: std::marker::Sized {
    fn check<C: FnOnce(&V) -> bool, T: FnOnce(V) -> E>(self, _: C, _: T) -> Result<R, E>;
}

impl<V, E> ResultExtender<V, E, V> for Result<V, E> {
    fn check<C: FnOnce(&V) -> bool, T: FnOnce(V) -> E>(self, check_block: C, throw_block: T) -> Result<V, E> {
        self.and_then(|v| {
            if check_block(&v) { Ok(v) } else { Err(throw_block(v)) }
        })
    }

}
