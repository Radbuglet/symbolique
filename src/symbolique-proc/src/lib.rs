use std::num::NonZeroU32;

use proc_macro::{Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};
use sha2::{Digest, Sha256};

use litrs::Literal as PLiteral;

#[proc_macro]
pub fn __handle_magic(tokens: TokenStream) -> TokenStream {
    process_stream(
        tokens,
        &mut StreamProcessState {
            rng: fastrand::u128(..),
            nz_id_gen: NonZeroU32::new(1).unwrap(),
        },
    )
}

#[derive(Debug)]
struct StreamProcessState {
    rng: u128,
    nz_id_gen: NonZeroU32,
}

fn process_stream(tokens: TokenStream, state: &mut StreamProcessState) -> TokenStream {
    let mut out = TokenStream::new();
    let mut tokens = tokens.into_iter();

    while let Some(token) = tokens.next() {
        // Parse token
        match token {
            TokenTree::Group(group) => {
                let mut new_group =
                    Group::new(group.delimiter(), process_stream(group.stream(), state));
                new_group.set_span(group.span());
                out.extend([TokenTree::Group(new_group)]);
            }
            TokenTree::Ident(ident)
                if matches!(
                    ident.to_string().as_str(),
                    "__SYMBOLIQUE_MAGIC_HASH" | "__SYMBOLIQUE_MAGIC_HASH_RAND"
                ) =>
            {
                let names = tokens
                    .next()
                    .expect("expected token after `__SYMBOLIQUE_MAGIC_HASH` directive");
                let span = names.span();

                let mut sha = Sha256::new();
                if ident.to_string() == "__SYMBOLIQUE_MAGIC_HASH_RAND" {
                    sha.update(u128::to_be_bytes(state.rng));
                }
                hash_tokens(names, &mut sha);

                out.extend([TokenTree::Ident(Ident::new(
                    &format!("symbolique_{:x}", sha.finalize()),
                    span,
                ))]);
            }
            TokenTree::Ident(ident)
                if ident.to_string() == "__SYMBOLIQUE_GEN_NZ_INCREMENTAL_ID" =>
            {
                out.extend([TokenTree::Literal(Literal::u32_suffixed(
                    state.nz_id_gen.get(),
                ))]);

                state.nz_id_gen = state
                    .nz_id_gen
                    .checked_add(1)
                    .expect("generated too many symbols");
            }
            TokenTree::Ident(ident) if ident.to_string() == "__SYMBOLIQUE_DOLLAR" => {
                out.extend([TokenTree::Punct(Punct::new('$', Spacing::Alone))]);
            }
            TokenTree::Ident(ident) if ident.to_string() == "__SYMBOLIQUE_SKIP" => {
                if let Some(next) = tokens.next() {
                    out.extend([next]);
                }
            }
            token => out.extend([token]),
        }
    }

    out
}

fn hash_tokens(t: TokenTree, sha: &mut Sha256) {
    match t {
        TokenTree::Group(group) => {
            for t in group.stream() {
                hash_tokens(t, sha);
            }
        }
        TokenTree::Ident(ident) => {
            sha.update(ident.to_string().as_bytes());
        }
        TokenTree::Literal(lit) => {
            let lit = PLiteral::parse(lit.to_string()).unwrap();
            match lit {
                PLiteral::Bool(lit) => {
                    if lit.value() {
                        sha.update(b"true");
                    } else {
                        sha.update(b"false");
                    }
                }
                PLiteral::Integer(lit) => {
                    sha.update(lit.raw_input());
                }
                PLiteral::Float(lit) => {
                    sha.update(lit.raw_input());
                }
                PLiteral::Char(lit) => {
                    sha.update(lit.value().to_string().as_bytes());
                }
                PLiteral::String(lit) => {
                    sha.update(lit.value().as_bytes());
                }
                PLiteral::Byte(lit) => {
                    sha.update([lit.value()]);
                }
                PLiteral::ByteString(lit) => {
                    sha.update(lit.value());
                }
            }
        }
        TokenTree::Punct(_punct) => { /* ignored */ }
    }
}
