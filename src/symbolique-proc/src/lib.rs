use std::num::NonZeroU32;

use proc_macro::{Delimiter, Group, Ident, Literal, TokenStream, TokenTree};
use sha2::{Digest, Sha256};

#[proc_macro]
pub fn __handle_magic(tokens: TokenStream) -> TokenStream {
    process_stream(
        tokens,
        &mut StreamProcessState {
            nz_id_gen: NonZeroU32::new(1).unwrap(),
        },
    )
}

#[derive(Debug)]
struct StreamProcessState {
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
            TokenTree::Ident(ident) if ident.to_string() == "__SYMBOLIQUE_MAGIC_HASH" => {
                let Some(TokenTree::Literal(lit)) = tokens.next().map(strip_empty_groups) else {
                    panic!("expected literal after `__SYMBOLIQUE_MAGIC_HASH` directive");
                };

                let mut sha = Sha256::new();
                sha.update(&lit.to_string());
                out.extend([TokenTree::Ident(Ident::new(
                    &format!("sym_{:x}", sha.finalize()),
                    lit.span(),
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
            token => out.extend([token]),
        }
    }

    out
}

fn strip_empty_groups(t: TokenTree) -> TokenTree {
    match t {
        TokenTree::Group(group)
            if group.delimiter() == Delimiter::None && {
                let mut stream = group.stream().into_iter();
                stream.next().is_some() && stream.next().is_none()
            } =>
        {
            strip_empty_groups(group.stream().into_iter().next().unwrap())
        }
        t => t,
    }
}
