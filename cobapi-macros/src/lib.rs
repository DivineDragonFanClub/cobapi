use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::quote;
use syn::FnArg;
use syn::ItemTrait;
use syn::Pat;
use syn::ReturnType;
use syn::TraitItem;
use syn::TraitItemFn;
use syn::Type;
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_def = parse_macro_input!(item as ItemTrait);

    match expand_service(&trait_def) {
        Ok(out) => out.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_service(trait_def: &ItemTrait) -> syn::Result<TokenStream2> {
    let trait_name = &trait_def.ident;
    let vis = &trait_def.vis;
    let handle_name = format_ident!("{}Handle", trait_name);

    let methods: Vec<&TraitItemFn> = trait_def
        .items
        .iter()
        .filter_map(|i| if let TraitItem::Fn(f) = i { Some(f) } else { None })
        .collect();

    for m in &methods {
        validate_method(m)?;
    }

    let id_str_lit = {
        let trait_name_str = trait_name.to_string();

        quote! {
            ::core::concat!(
                ::core::module_path!(),
                "::",
                #trait_name_str,
                "@",
                ::core::env!("CARGO_PKG_VERSION_MAJOR"),
            )
        }
    };

    let handle_field_decls = methods.iter().map(|m| {
        let n = &m.sig.ident;
        quote! { #n: *const ::core::ffi::c_void, }
    });

    let handle_method_impls = methods.iter().map(|m| make_handle_method(m));

    let trampoline_defs = methods.iter().map(|m| make_trampoline(m, trait_name));

    let entries_init = methods.iter().map(|m| make_entry_init(m, trait_name));

    let lookup_field_inits = methods.iter().map(|m| make_lookup_field_init(m));

    let drop_trampoline = quote! {
        unsafe extern "C" fn __cobapi_trampoline_drop<__T>(this: *mut ::core::ffi::c_void) {
            ::core::mem::drop(::std::boxed::Box::from_raw(this as *mut __T));
        }
    };

    let method_count = methods.len() as u32;

    Ok(quote! {
        #trait_def

        #[allow(non_camel_case_types)]
        #vis struct #handle_name {
            __cobapi_this: *mut ::core::ffi::c_void,
            #(#handle_field_decls)*
        }

        unsafe impl ::core::marker::Send for #handle_name {}
        unsafe impl ::core::marker::Sync for #handle_name {}

        impl #handle_name {
            #(#handle_method_impls)*
        }

        #[allow(non_snake_case, dead_code)]
        const _: () = {
            const __COBAPI_SERVICE_ID: &'static str = #id_str_lit;

            #vis fn __cobapi_id() -> &'static str {
                #id_str_lit
            }
        };

        #vis fn install<__T: #trait_name + ::core::marker::Send + ::core::marker::Sync + 'static>(
            impl_: __T,
        ) -> ::core::result::Result<(), ::cobapi::RegisterServiceError> {
            #(#trampoline_defs)*
            #drop_trampoline

            let entries: ::std::vec::Vec<::cobapi::MethodEntry> = ::std::vec![
                #(#entries_init),*
            ];
            let entries_static: &'static [::cobapi::MethodEntry] =
                ::std::boxed::Box::leak(entries.into_boxed_slice());

            let header = ::cobapi::VTableHeader {
                abi_version: ::cobapi::VTABLE_ABI_VERSION,
                _reserved: 0,
                method_count: #method_count,
                _pad: 0,
                methods: entries_static.as_ptr(),
                drop_fn: __cobapi_trampoline_drop::<__T>,
            };
            let header_static: &'static ::cobapi::VTableHeader =
                ::std::boxed::Box::leak(::std::boxed::Box::new(header));

            let this_ptr: *mut ::core::ffi::c_void =
                ::std::boxed::Box::into_raw(::std::boxed::Box::new(impl_)) as *mut ::core::ffi::c_void;

            ::cobapi::register_service(#id_str_lit, this_ptr, header_static)
        }

        #vis fn lookup() -> ::core::option::Option<#handle_name> {
            let (this, vtable) = ::cobapi::lookup_service(#id_str_lit)?;
            unsafe {
                let header: &::cobapi::VTableHeader = &*vtable;
                if header.abi_version != ::cobapi::VTABLE_ABI_VERSION {
                    return ::core::option::Option::None;
                }
                let methods: &[::cobapi::MethodEntry] =
                    ::core::slice::from_raw_parts(header.methods, header.method_count as usize);

                ::core::option::Option::Some(#handle_name {
                    __cobapi_this: this,
                    #(#lookup_field_inits)*
                })
            }
        }

        #vis fn unregister() {
            ::cobapi::unregister_service(#id_str_lit);
        }
    })
}

fn validate_method(m: &TraitItemFn) -> syn::Result<()> {
    if m.sig.generics.params.iter().any(|p| !matches!(p, syn::GenericParam::Lifetime(_))) {
        return Err(syn::Error::new(
            m.sig.generics.span(),
            "service methods cannot be generic",
        ));
    }

    if m.sig.asyncness.is_some() {
        return Err(syn::Error::new(
            m.sig.span(),
            "service methods cannot be async",
        ));
    }

    let mut saw_self = false;
    for arg in &m.sig.inputs {
        if let FnArg::Receiver(r) = arg {
            if r.mutability.is_some() {
                return Err(syn::Error::new(
                    r.span(),
                    "service methods take &self only, not &mut self",
                ));
            }
            if r.reference.is_none() {
                return Err(syn::Error::new(
                    r.span(),
                    "service methods take &self, not self by value",
                ));
            }
            saw_self = true;
        }
    }

    if !saw_self {
        return Err(syn::Error::new(
            m.sig.span(),
            "service methods must take &self",
        ));
    }

    Ok(())
}

fn make_handle_method(m: &TraitItemFn) -> TokenStream2 {
    let name = &m.sig.ident;
    let other_args = non_self_args(&m.sig.inputs);

    let arg_names: Vec<_> = other_args.iter().map(|(p, _)| p.clone()).collect();
    let arg_types: Vec<_> = other_args.iter().map(|(_, t)| t.clone()).collect();
    let ret = &m.sig.output;

    let sig_ty = quote! {
        unsafe extern "C" fn(*mut ::core::ffi::c_void, #(#arg_types),*) #ret
    };

    quote! {
        pub fn #name(&self, #(#arg_names: #arg_types),*) #ret {
            let f: #sig_ty = unsafe { ::core::mem::transmute(self.#name) };
            unsafe { f(self.__cobapi_this, #(#arg_names),*) }
        }
    }
}

fn make_trampoline(m: &TraitItemFn, trait_name: &syn::Ident) -> TokenStream2 {
    let name = &m.sig.ident;
    let other_args = non_self_args(&m.sig.inputs);
    let arg_names: Vec<_> = other_args.iter().map(|(p, _)| p.clone()).collect();
    let arg_types: Vec<_> = other_args.iter().map(|(_, t)| t.clone()).collect();
    let ret = &m.sig.output;

    let trampoline_name = format_ident!("__cobapi_trampoline_{}", name);

    quote! {
        unsafe extern "C" fn #trampoline_name<__T: #trait_name>(
            this: *mut ::core::ffi::c_void,
            #(#arg_names: #arg_types),*
        ) #ret {
            let this_ref: &__T = &*(this as *const __T);
            <__T as #trait_name>::#name(this_ref, #(#arg_names),*)
        }
    }
}

fn make_entry_init(m: &TraitItemFn, _trait_name: &syn::Ident) -> TokenStream2 {
    let name = &m.sig.ident;
    let name_str = name.to_string();
    let sig_str = signature_string(m);
    let trampoline_name = format_ident!("__cobapi_trampoline_{}", name);

    let name_hash = const_fnv1a_64(name_str.as_bytes());
    let sig_hash = const_fnv1a_64(sig_str.as_bytes());

    quote! {
        ::cobapi::MethodEntry {
            name_hash: #name_hash,
            sig_hash: #sig_hash,
            fn_ptr: #trampoline_name::<__T> as *const ::core::ffi::c_void,
        }
    }
}

fn make_lookup_field_init(m: &TraitItemFn) -> TokenStream2 {
    let name = &m.sig.ident;
    let name_str = name.to_string();
    let sig_str = signature_string(m);

    let name_hash = const_fnv1a_64(name_str.as_bytes());
    let sig_hash = const_fnv1a_64(sig_str.as_bytes());

    quote! {
        #name: match methods.iter().find(|e| e.name_hash == #name_hash && e.sig_hash == #sig_hash) {
            ::core::option::Option::Some(e) => e.fn_ptr,
            ::core::option::Option::None => return ::core::option::Option::None,
        },
    }
}

fn non_self_args(inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>) -> Vec<(syn::Ident, Type)> {
    let mut out = Vec::new();

    for (i, arg) in inputs.iter().enumerate() {
        if let FnArg::Typed(pt) = arg {
            let name = match pt.pat.as_ref() {
                Pat::Ident(pi) => pi.ident.clone(),
                _ => format_ident!("__arg_{}", i),
            };
            out.push((name, (*pt.ty).clone()));
        }
    }

    out
}

fn signature_string(m: &TraitItemFn) -> String {
    let mut s = String::new();
    s.push('(');

    let mut first = true;
    for (_, ty) in non_self_args(&m.sig.inputs) {
        if !first {
            s.push(',');
        }
        first = false;
        s.push_str(&type_to_canonical_string(&ty));
    }

    s.push(')');

    if let ReturnType::Type(_, t) = &m.sig.output {
        s.push_str("->");
        s.push_str(&type_to_canonical_string(t));
    }

    s
}

fn type_to_canonical_string(ty: &Type) -> String {
    quote!(#ty).to_string().split_whitespace().collect()
}

fn const_fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;

    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}
