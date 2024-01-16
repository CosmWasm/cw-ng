use crate::associated_types::AssociatedTypes;
use crate::check_generics::{CheckGenerics, GetPath};
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::parser::{
    parse_associated_custom_type, parse_struct_message, ContractErrorAttr, ContractMessageAttr,
    Custom, EntryPointArgs, MsgAttr, MsgType, OverrideEntryPoints,
};
use crate::strip_generics::StripGenerics;
use crate::strip_self_path::{AddSelfPath, StripSelfPath};
use crate::utils::{
    as_where_clause, emit_bracketed_generics, extract_return_type, filter_generics, filter_wheres,
    process_fields,
};
use crate::variant_descs::{AsVariantDescs, VariantDescs};
use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::fold::Fold;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    parse_quote, Attribute, GenericArgument, GenericParam, Ident, ItemImpl, ItemTrait, Pat,
    PatType, ReturnType, Signature, Token, Type, WhereClause, WherePredicate,
};

/// Representation of single struct message
pub struct StructMessage<'a> {
    contract_type: &'a Type,
    fields: Vec<MsgField<'a>>,
    function_name: &'a Ident,
    generics: Vec<&'a GenericParam>,
    unused_generics: Vec<&'a GenericParam>,
    wheres: Vec<&'a WherePredicate>,
    full_where: Option<&'a WhereClause>,
    result: &'a ReturnType,
    msg_attr: MsgAttr,
    custom: &'a Custom<'a>,
}

impl<'a> StructMessage<'a> {
    /// Creates new struct message of given type from impl block
    pub fn new(
        source: &'a ItemImpl,
        ty: MsgType,
        generics: &'a [&'a GenericParam],
        custom: &'a Custom,
    ) -> Option<StructMessage<'a>> {
        let mut generics_checker = CheckGenerics::new(generics);

        let contract_type = &source.self_ty;

        let parsed = parse_struct_message(source, ty);
        let Some((method, msg_attr)) = parsed else {
            return None;
        };

        let function_name = &method.sig.ident;
        let fields = process_fields(&method.sig, &mut generics_checker);
        let (used_generics, unused_generics) = generics_checker.used_unused();
        let wheres = filter_wheres(&source.generics.where_clause, generics, &used_generics);

        Some(Self {
            contract_type,
            fields,
            function_name,
            generics: used_generics,
            unused_generics,
            wheres,
            full_where: source.generics.where_clause.as_ref(),
            result: &method.sig.output,
            msg_attr,
            custom,
        })
    }

    pub fn emit(&self) -> TokenStream {
        use MsgAttr::*;

        match &self.msg_attr {
            Instantiate { name } => self.emit_struct(name),
            Migrate { name } => self.emit_struct(name),
            _ => {
                emit_error!(Span::mixed_site(), "Invalid message type");
                quote! {}
            }
        }
    }

    pub fn emit_struct(&self, name: &Ident) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            contract_type,
            fields,
            function_name,
            generics,
            unused_generics,
            wheres,
            full_where,
            result,
            msg_attr,
            custom,
        } = self;

        let ctx_type = msg_attr
            .msg_type()
            .emit_ctx_type(&custom.query_or_default());
        let fields_names: Vec<_> = fields.iter().map(MsgField::name).collect();
        let parameters = fields.iter().map(|field| {
            let name = field.name;
            let ty = &field.ty;
            quote! { #name : #ty}
        });
        let fields = fields.iter().map(MsgField::emit);

        let where_clause = as_where_clause(wheres);
        let generics = emit_bracketed_generics(generics);
        let unused_generics = emit_bracketed_generics(unused_generics);

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #[allow(clippy::derive_partial_eq_without_eq)]
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
                #[serde(rename_all="snake_case")]
                pub struct #name #generics {
                    #(pub #fields,)*
                }

                impl #generics #name #generics #where_clause {
                    pub fn new(#(#parameters,)*) -> Self {
                        Self { #(#fields_names,)* }
                    }

                    pub fn dispatch #unused_generics(self, contract: &#contract_type, ctx: #ctx_type)
                        #result #full_where
                    {
                        let Self { #(#fields_names,)* } = self;
                        contract.#function_name(Into::into(ctx), #(#fields_names,)*).map_err(Into::into)
                    }
                }
            }
        }
    }
}

/// Representation of single enum message
pub struct EnumMessage<'a> {
    source: &'a ItemTrait,
    variants: MsgVariants<'a, Ident>,
    associated_types: &'a AssociatedTypes<'a>,
    msg_ty: MsgType,
    resp_type: Type,
    query_type: Type,
}

impl<'a> EnumMessage<'a> {
    pub fn new(
        source: &'a ItemTrait,
        msg_ty: MsgType,
        custom: &'a Custom,
        variants: MsgVariants<'a, Ident>,
        associated_types: &'a AssociatedTypes<'a>,
    ) -> Self {
        let associated_exec = parse_associated_custom_type(source, "ExecC");
        let associated_query = parse_associated_custom_type(source, "QueryC");

        let resp_type = custom
            .msg()
            .or(associated_exec)
            .unwrap_or_else(Custom::default_type);

        let query_type = custom
            .query()
            .or(associated_query)
            .unwrap_or_else(Custom::default_type);

        Self {
            source,
            variants,
            associated_types,
            msg_ty,
            resp_type,
            query_type,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            source,
            variants,
            associated_types,
            msg_ty,
            resp_type,
            query_type,
        } = self;

        let trait_name = &source.ident;
        let enum_name = msg_ty.emit_msg_name(false);
        let unique_enum_name =
            Ident::new(&format!("{}{}", trait_name, enum_name), enum_name.span());

        let match_arms = variants.emit_dispatch_legs();
        let mut msgs = variants.as_names_snake_cased();
        msgs.sort();
        let msgs_cnt = msgs.len();
        let variants_constructors = variants.emit_constructors();
        let msg_variants = variants.emit();

        let ctx_type = msg_ty.emit_ctx_type(query_type);
        let dispatch_type = msg_ty.emit_result_type(resp_type, &parse_quote!(ContractT::Error));

        let used_generics = variants.used_generics();
        let unused_generics = variants.unused_generics();
        let where_predicates = associated_types.as_where_predicates();
        let where_clause = variants.where_clause();
        let contract_predicate = associated_types.emit_contract_predicate(trait_name);

        let phantom_variant = variants.emit_phantom_variant();
        let phatom_match_arm = variants.emit_phantom_match_arm();
        let bracketed_used_generics = emit_bracketed_generics(used_generics);

        let ep_name = msg_ty.emit_ep_name();
        let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), enum_name.span());
        let derive_call = msg_ty.emit_derive_call();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #[allow(clippy::derive_partial_eq_without_eq)]
                #derive_call
                #[serde(rename_all="snake_case")]
                pub enum #unique_enum_name #bracketed_used_generics {
                    #(#msg_variants,)*
                    #phantom_variant
                }
                pub type #enum_name #bracketed_used_generics = #unique_enum_name #bracketed_used_generics;

                impl #bracketed_used_generics #unique_enum_name #bracketed_used_generics #where_clause {
                    pub fn dispatch<ContractT, #(#unused_generics,)*>(self, contract: &ContractT, ctx: #ctx_type)
                        -> #dispatch_type
                    where
                        #(#where_predicates,)*
                        #contract_predicate
                    {
                        use #unique_enum_name::*;

                        match self {
                            #(#match_arms,)*
                            #phatom_match_arm
                        }
                    }
                    #(#variants_constructors)*
                }

                pub const fn #messages_fn_name () -> [&'static str; #msgs_cnt] {
                    [#(#msgs,)*]
                }
            }
        }
    }
}

/// Representation of single enum message
pub struct ContractEnumMessage<'a> {
    variants: MsgVariants<'a, GenericParam>,
    msg_ty: MsgType,
    contract: &'a Type,
    error: &'a ContractErrorAttr,
    custom: &'a Custom<'a>,
    where_clause: &'a Option<WhereClause>,
}

impl<'a> ContractEnumMessage<'a> {
    pub fn new(
        source: &'a ItemImpl,
        msg_ty: MsgType,
        generics: &'a [&'a GenericParam],
        error: &'a ContractErrorAttr,
        custom: &'a Custom,
    ) -> Self {
        let where_clause = &source.generics.where_clause;
        let variants = MsgVariants::new(source.as_variants(), msg_ty, generics, where_clause);

        Self {
            variants,
            msg_ty,
            contract: &source.self_ty,
            error,
            custom,
            where_clause,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            variants,
            msg_ty,
            contract,
            error,
            custom,
            where_clause,
            ..
        } = self;

        let enum_name = msg_ty.emit_msg_name(false);
        let match_arms = variants.emit_dispatch_legs();
        let unused_generics = variants.unused_generics();
        let bracketed_unused_generics = emit_bracketed_generics(unused_generics);
        let used_generics = variants.used_generics();
        let bracketed_used_generics = emit_bracketed_generics(used_generics);

        let mut variant_names = variants.as_names_snake_cased();
        variant_names.sort();
        let variants_cnt = variant_names.len();
        let variants_constructors = variants.emit_constructors();
        let variants = variants.emit();

        let ctx_type = msg_ty.emit_ctx_type(&custom.query_or_default());
        let ret_type = msg_ty.emit_result_type(&custom.msg_or_default(), &error.error);

        let derive_query = match msg_ty {
            MsgType::Query => quote! { #sylvia ::cw_schema::QueryResponses },
            _ => quote! {},
        };

        let ep_name = msg_ty.emit_ep_name();
        let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), contract.span());

        let phantom_variant = msg_ty.emit_phantom_variant(used_generics);
        let phantom_match_arm = match !used_generics.is_empty() {
            true => quote! {
                _Phantom(_) => Err(#sylvia ::cw_std::StdError::generic_err("Phantom message should not be constructed.")).map_err(Into::into),
            },
            false => quote! {},
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #[allow(clippy::derive_partial_eq_without_eq)]
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema, #derive_query )]
                #[serde(rename_all="snake_case")]
                pub enum #enum_name #bracketed_used_generics {
                    #(#variants,)*
                    #phantom_variant
                }

                impl #bracketed_used_generics #enum_name #bracketed_used_generics {
                    pub fn dispatch #bracketed_unused_generics (self, contract: &#contract, ctx: #ctx_type) -> #ret_type #where_clause {
                        use #enum_name::*;

                        match self {
                            #(#match_arms,)*
                            #phantom_match_arm
                        }
                    }

                    #(#variants_constructors)*
                }

                pub const fn #messages_fn_name () -> [&'static str; #variants_cnt] {
                    [#(#variant_names,)*]
                }
            }
        }
    }
}

/// Representation of whole message variant
#[derive(Debug)]
pub struct MsgVariant<'a> {
    name: Ident,
    function_name: &'a Ident,
    // With https://github.com/rust-lang/rust/issues/63063 this could be just an iterator over
    // `MsgField<'a>`
    fields: Vec<MsgField<'a>>,
    return_type: TokenStream,
    stripped_return_type: TokenStream,
    msg_type: MsgType,
}

impl<'a> MsgVariant<'a> {
    /// Creates new message variant from trait method
    pub fn new<Generic>(
        sig: &'a Signature,
        generics_checker: &mut CheckGenerics<Generic>,
        msg_attr: MsgAttr,
    ) -> MsgVariant<'a>
    where
        Generic: GetPath + PartialEq,
    {
        let function_name = &sig.ident;

        let name = Ident::new(
            &function_name.to_string().to_case(Case::UpperCamel),
            function_name.span(),
        );
        let fields = process_fields(sig, generics_checker);
        let msg_type = msg_attr.msg_type();

        let (return_type, stripped_return_type) = if let MsgAttr::Query { resp_type } = msg_attr {
            match resp_type {
                Some(resp_type) => {
                    let resp_type = parse_quote! { #resp_type };
                    generics_checker.visit_path(&resp_type);
                    (quote! { #resp_type }, quote! { #resp_type })
                }
                None => {
                    let return_type = extract_return_type(&sig.output);
                    let stripped_return_type = StripSelfPath.fold_path(return_type.clone());
                    generics_checker.visit_path(&stripped_return_type);
                    (quote! { #return_type }, quote! { #stripped_return_type })
                }
            }
        } else {
            (quote! {}, quote! {})
        };

        Self {
            name,
            function_name,
            fields,
            return_type,
            stripped_return_type,
            msg_type,
        }
    }

    /// Emits message variant
    pub fn emit(&self) -> TokenStream {
        let Self {
            name,
            fields,
            msg_type,
            stripped_return_type,
            ..
        } = self;
        let fields = fields.iter().map(MsgField::emit);
        let returns_attribute = match msg_type {
            MsgType::Query => quote! { #[returns(#stripped_return_type)] },
            _ => quote! {},
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #returns_attribute
                #name {
                    #(#fields,)*
                }
            }
        }
    }

    /// Emits match leg dispatching against this variant. Assumes enum variants are imported into the
    /// scope. Dispatching is performed by calling the function this variant is build from on the
    /// `contract` variable, with `ctx` as its first argument - both of them should be in scope.
    pub fn emit_dispatch_leg(&self) -> TokenStream {
        use MsgType::*;

        let Self {
            name,
            fields,
            function_name,
            msg_type,
            ..
        } = self;

        let sylvia = crate_module();

        let args = fields
            .iter()
            .zip(1..)
            .map(|(field, num)| Ident::new(&format!("field{}", num), field.name.span()));

        let fields = fields
            .iter()
            .map(|field| field.name)
            .zip(args.clone())
            .map(|(field, num_field)| quote!(#field : #num_field));

        #[cfg(not(tarpaulin_include))]
        match msg_type {
            Exec => quote! {
                #name {
                    #(#fields,)*
                } => contract.#function_name(Into::into(ctx), #(#args),*).map_err(Into::into)
            },
            Query => quote! {
                #name {
                    #(#fields,)*
                } => #sylvia ::cw_std::to_json_binary(&contract.#function_name(Into::into(ctx), #(#args),*)?).map_err(Into::into)
            },
            Instantiate | Migrate | Reply | Sudo => {
                emit_error!(name.span(), "Instantiation, Reply, Migrate and Sudo messages not supported on traits, they should be defined on contracts directly");
                quote! {}
            }
        }
    }

    /// Emits variants constructors. Constructors names are variants names in snake_case.
    pub fn emit_variants_constructors(&self) -> TokenStream {
        let Self { name, fields, .. } = self;

        let method_name = name.to_string().to_case(Case::Snake);
        let method_name = Ident::new(&method_name, name.span());
        let parameters = fields.iter().map(MsgField::emit_method_field);
        let arguments = fields.iter().map(MsgField::name);

        quote! {
            pub fn #method_name( #(#parameters),*) -> Self {
                Self :: #name { #(#arguments),* }
            }
        }
    }

    pub fn emit_trait_querier_impl(&self, associated_name: &[&Ident]) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            name,
            fields,
            return_type,
            ..
        } = self;

        let parameters = fields.iter().map(MsgField::emit_method_field);
        let fields_names = fields.iter().map(MsgField::name);
        let variant_name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());
        let bracketed_generics = emit_bracketed_generics(associated_name);

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                fn #variant_name(&self, #(#parameters),*) -> Result< #return_type, #sylvia:: cw_std::StdError> {
                    let query = <Api #bracketed_generics as sylvia::types::InterfaceApi>::Query:: #variant_name (#(#fields_names),*);
                    self.querier().query_wasm_smart(self.contract(), &query)
                }
            }
        }
    }

    pub fn emit_querier_impl<Generic>(
        &self,
        api_path: &TokenStream,
        generics: &[&Generic],
    ) -> TokenStream
    where
        Generic: ToTokens + GetPath,
    {
        let sylvia = crate_module();
        let Self {
            name,
            fields,
            return_type,
            ..
        } = self;

        let parameters = fields
            .iter()
            .map(|field| field.emit_method_field_with_self(Some(AddSelfPath::new(generics))));
        let fields_names = fields.iter().map(MsgField::name);
        let variant_name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                fn #variant_name(&self, #(#parameters),*) -> Result< #return_type, #sylvia:: cw_std::StdError> {
                    let query = #api_path :: #variant_name (#(#fields_names),*);
                    self.querier().query_wasm_smart(self.contract(), &query)
                }
            }
        }
    }

    pub fn emit_querier_declaration<Generic>(&self, generics: &[&Generic]) -> TokenStream
    where
        Generic: ToTokens + GetPath,
    {
        let sylvia = crate_module();
        let Self {
            name,
            fields,
            return_type,
            ..
        } = self;

        let parameters = fields
            .iter()
            .map(|field| field.emit_method_field_with_self(Some(AddSelfPath::new(generics))));
        let variant_name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                fn #variant_name(&self, #(#parameters),*) -> Result< #return_type, #sylvia:: cw_std::StdError>;
            }
        }
    }

    pub fn emit_multitest_proxy_methods<Generic>(
        &self,
        msg_ty: &MsgType,
        custom_msg: &Type,
        mt_app: &Type,
        error_type: &Type,
        generics: &[&Generic],
    ) -> TokenStream
    where
        Generic: ToTokens,
    {
        let sylvia = crate_module();
        let Self {
            name,
            fields,
            stripped_return_type,
            ..
        } = self;

        let params = fields.iter().map(|field| field.emit_method_field());
        let arguments = fields.iter().map(MsgField::name);
        let name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());
        let enum_name = msg_ty.emit_msg_name(false);
        let enum_name: Type = if !generics.is_empty() {
            parse_quote! { #enum_name ::< #(#generics,)* > }
        } else {
            parse_quote! { #enum_name }
        };

        match msg_ty {
            MsgType::Exec => quote! {
                #[track_caller]
                pub fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, #enum_name, #mt_app, #custom_msg> {
                    let msg = #enum_name :: #name ( #(#arguments),* );

                    #sylvia ::multitest::ExecProxy::new(&self.contract_addr, msg, &self.app)
                }
            },
            MsgType::Migrate => quote! {
                #[track_caller]
                pub fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::MigrateProxy::<#error_type, #enum_name, #mt_app, #custom_msg> {
                    let msg = #enum_name ::new( #(#arguments),* );

                    #sylvia ::multitest::MigrateProxy::new(&self.contract_addr, msg, &self.app)
                }
            },
            MsgType::Query => quote! {
                pub fn #name (&self, #(#params,)* ) -> Result<#stripped_return_type, #error_type> {
                    let msg = #enum_name :: #name ( #(#arguments),* );

                    (*self.app)
                        .app()
                        .wrap()
                        .query_wasm_smart(self.contract_addr.clone(), &msg)
                        .map_err(Into::into)
                }
            },
            _ => quote! {},
        }
    }

    pub fn emit_interface_multitest_proxy_methods<Generics>(
        &self,
        msg_ty: &MsgType,
        custom_msg: &Type,
        mt_app: &Type,
        error_type: &Type,
        generics: &[&Generics],
        interface_api: &TokenStream,
    ) -> TokenStream
    where
        Generics: ToTokens + GetPath,
    {
        let sylvia = crate_module();
        let Self {
            name,
            fields,
            return_type,
            ..
        } = self;

        let params = fields
            .iter()
            .map(|field| field.emit_method_field_with_self(Some(AddSelfPath::new(generics))));
        let arguments = fields.iter().map(MsgField::name);
        let type_name = msg_ty.as_accessor_name(false);
        let name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());

        match msg_ty {
            MsgType::Exec => quote! {
                #[track_caller]
                fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, #interface_api :: #type_name, #mt_app, #custom_msg> {
                    let msg = #interface_api :: #type_name :: #name ( #(#arguments),* );

                    #sylvia ::multitest::ExecProxy::new(&self.contract_addr, msg, &self.app)
                }
            },
            MsgType::Query => quote! {
                fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type> {
                    let msg = #interface_api :: #type_name :: #name ( #(#arguments),* );

                    (*self.app)
                        .app()
                        .wrap()
                        .query_wasm_smart(self.contract_addr.clone(), &msg)
                        .map_err(Into::into)
                }
            },
            _ => quote! {},
        }
    }

    pub fn emit_proxy_methods_declarations<Generic>(
        &self,
        msg_ty: &MsgType,
        custom_msg: &Type,
        error_type: &Type,
        interface_enum: &TokenStream,
        generics: &[&Generic],
    ) -> TokenStream
    where
        Generic: GetPath + ToTokens,
    {
        let sylvia = crate_module();
        let Self {
            name,
            fields,
            return_type,
            ..
        } = self;

        let params = fields
            .iter()
            .map(|field| field.emit_method_field_with_self(Some(AddSelfPath::new(generics))));
        let type_name = msg_ty.as_accessor_name(false);
        let name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());

        match msg_ty {
            MsgType::Exec => quote! {
                fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, #interface_enum :: #type_name, MtApp, #custom_msg>;
            },
            MsgType::Query => quote! {
                fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type>;
            },
            _ => quote! {},
        }
    }

    pub fn as_fields_names(&self) -> Vec<&Ident> {
        self.fields.iter().map(MsgField::name).collect()
    }

    pub fn emit_fields(&self) -> Vec<TokenStream> {
        self.fields.iter().map(MsgField::emit).collect()
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}

#[derive(Debug)]
pub struct MsgVariants<'a, Generic> {
    variants: Vec<MsgVariant<'a>>,
    used_generics: Vec<&'a Generic>,
    unused_generics: Vec<&'a Generic>,
    where_predicates: Vec<&'a WherePredicate>,
    msg_ty: MsgType,
}

impl<'a, Generic> MsgVariants<'a, Generic>
where
    Generic: GetPath + PartialEq + ToTokens,
{
    pub fn new(
        source: VariantDescs<'a>,
        msg_ty: MsgType,
        all_generics: &'a [&'a Generic],
        unfiltered_where_clause: &'a Option<WhereClause>,
    ) -> Self {
        let mut generics_checker = CheckGenerics::new(all_generics);
        let variants: Vec<_> = source
            .filter_map(|variant_desc| {
                let msg_attr = variant_desc.attr_msg()?;
                let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                    Ok(attr) => attr,
                    Err(err) => {
                        emit_error!(variant_desc.span(), err);
                        return None;
                    }
                };

                if attr.msg_type() != msg_ty {
                    return None;
                }

                Some(MsgVariant::new(
                    variant_desc.into_sig(),
                    &mut generics_checker,
                    attr,
                ))
            })
            .collect();

        let (used_generics, unused_generics) = generics_checker.used_unused();
        let where_predicates = filter_wheres(unfiltered_where_clause, all_generics, &used_generics);

        Self {
            variants,
            used_generics,
            unused_generics,
            where_predicates,
            msg_ty,
        }
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        let where_predicates = &self.where_predicates;
        if !where_predicates.is_empty() {
            Some(parse_quote! { where #(#where_predicates),* })
        } else {
            None
        }
    }

    pub fn variants(&self) -> &Vec<MsgVariant<'a>> {
        &self.variants
    }

    pub fn used_generics(&self) -> &Vec<&'a Generic> {
        &self.used_generics
    }

    pub fn unused_generics(&self) -> &Vec<&'a Generic> {
        &self.unused_generics
    }

    pub fn emit_default_entry_point(
        &self,
        custom_msg: &Type,
        custom_query: &Type,
        name: &Type,
        error: &Type,
        contract_generics: &Option<Punctuated<GenericArgument, Token![,]>>,
    ) -> TokenStream {
        let Self { msg_ty, .. } = self;
        let sylvia = crate_module();

        let resp_type = match msg_ty {
            MsgType::Query => quote! { #sylvia ::cw_std::Binary },
            _ => quote! { #sylvia ::cw_std::Response < #custom_msg > },
        };
        let params = msg_ty.emit_ctx_params(custom_query);
        let values = msg_ty.emit_ctx_values();
        let ep_name = msg_ty.emit_ep_name();
        let bracketed_generics = match &contract_generics {
            Some(generics) => quote! { ::< #generics > },
            None => quote! {},
        };
        let associated_name = msg_ty.as_accessor_name(true);

        quote! {
            #[#sylvia ::cw_std::entry_point]
            pub fn #ep_name (
                #params ,
                msg: < #name < #contract_generics > as #sylvia ::types::ContractApi> :: #associated_name,
            ) -> Result<#resp_type, #error> {
                msg.dispatch(&#name #bracketed_generics ::new() , ( #values )).map_err(Into::into)
            }
        }
    }
    pub fn emit_multitest_proxy_methods(
        &self,
        custom_msg: &Type,
        mt_app: &Type,
        error_type: &Type,
    ) -> Vec<TokenStream> {
        self.variants
            .iter()
            .map(|variant| {
                variant.emit_multitest_proxy_methods(
                    &self.msg_ty,
                    custom_msg,
                    mt_app,
                    error_type,
                    &self.used_generics,
                )
            })
            .collect()
    }

    pub fn emit_interface_multitest_proxy_methods<Generics>(
        &self,
        custom_msg: &Type,
        mt_app: &Type,
        error_type: &Type,
        generics: &[&Generics],
        interface_api: &TokenStream,
    ) -> Vec<TokenStream>
    where
        Generics: ToTokens + GetPath,
    {
        self.variants
            .iter()
            .map(|variant| {
                variant.emit_interface_multitest_proxy_methods(
                    &self.msg_ty,
                    custom_msg,
                    mt_app,
                    error_type,
                    generics,
                    interface_api,
                )
            })
            .collect()
    }

    pub fn emit_proxy_methods_declarations(
        &self,
        custom_msg: &Type,
        error_type: &Type,
        interface_enum: &TokenStream,
        generics: &[&Generic],
    ) -> Vec<TokenStream> {
        self.variants
            .iter()
            .map(|variant| {
                variant.emit_proxy_methods_declarations(
                    &self.msg_ty,
                    custom_msg,
                    error_type,
                    interface_enum,
                    generics,
                )
            })
            .collect()
    }

    pub fn emit_phantom_match_arm(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self { used_generics, .. } = self;
        if used_generics.is_empty() {
            return quote! {};
        }
        quote! {
            _Phantom(_) => Err(#sylvia ::cw_std::StdError::generic_err("Phantom message should not be constructed.")).map_err(Into::into),
        }
    }

    pub fn emit_dispatch_legs(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants
            .iter()
            .map(|variant| variant.emit_dispatch_leg())
    }

    pub fn as_names_snake_cased(&self) -> Vec<String> {
        self.variants
            .iter()
            .map(|variant| variant.name.to_string().to_case(Case::Snake))
            .collect()
    }

    pub fn emit_constructors(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants
            .iter()
            .map(MsgVariant::emit_variants_constructors)
    }

    pub fn emit(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants.iter().map(MsgVariant::emit)
    }

    pub fn get_only_variant(&self) -> Option<&MsgVariant> {
        self.variants.first()
    }

    pub fn emit_phantom_variant(&self) -> TokenStream {
        let Self {
            msg_ty,
            used_generics,
            ..
        } = self;

        if used_generics.is_empty() {
            return quote! {};
        }

        let return_attr = match msg_ty {
            MsgType::Query => quote! { #[returns((#(#used_generics,)*))] },
            _ => quote! {},
        };

        quote! {
            #[serde(skip)]
            #return_attr
            _Phantom(std::marker::PhantomData<( #(#used_generics,)* )>),
        }
    }
}

/// Representation of single message variant field
#[derive(Debug)]
pub struct MsgField<'a> {
    name: &'a Ident,
    ty: &'a Type,
    stripped_ty: Type,
    attrs: &'a Vec<Attribute>,
}

impl<'a> MsgField<'a> {
    /// Creates new field from trait method argument
    pub fn new<Generic>(
        item: &'a PatType,
        generics_checker: &mut CheckGenerics<Generic>,
    ) -> Option<MsgField<'a>>
    where
        Generic: GetPath + PartialEq,
    {
        let name = match &*item.pat {
            Pat::Ident(p) => Some(&p.ident),
            pat => {
                // TODO: Support pattern arguments, when decorated with argument with item
                // name
                //
                // Eg.
                //
                // ```
                // fn exec_foo(&self, ctx: Ctx, #[msg(name=metadata)] SomeData { addr, sender }: SomeData);
                // ```
                //
                // should expand to enum variant:
                //
                // ```
                // ExecFoo {
                //   metadata: SomeDaa
                // }
                // ```
                emit_error!(pat.span(), "Expected argument name, pattern occurred");
                None
            }
        }?;

        let ty = &item.ty;
        let stripped_ty = StripSelfPath.fold_type((*item.ty).clone());
        let attrs = &item.attrs;
        generics_checker.visit_type(&stripped_ty);

        Some(Self {
            name,
            ty,
            stripped_ty,
            attrs,
        })
    }

    /// Emits message field
    pub fn emit(&self) -> TokenStream {
        let Self {
            name,
            stripped_ty,
            attrs,
            ..
        } = self;

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #(#attrs)*
                #name: #stripped_ty
            }
        }
    }

    /// Emits method field
    pub fn emit_method_field(&self) -> TokenStream {
        let Self {
            name, stripped_ty, ..
        } = self;

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #name: #stripped_ty
            }
        }
    }

    pub fn emit_method_field_with_self<GenericT: GetPath>(
        &self,
        add_self: Option<AddSelfPath<GenericT>>,
    ) -> TokenStream {
        let Self {
            name, stripped_ty, ..
        } = self;

        let ty = match add_self {
            Some(mut add_self) => add_self.fold_type((*stripped_ty).clone()),
            None => stripped_ty.clone(),
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #name: #ty
            }
        }
    }

    pub fn name(&self) -> &'a Ident {
        self.name
    }
}

/// Glue message is the message composing Exec/Query messages from several traits
#[derive(Debug)]
pub struct GlueMessage<'a> {
    source: &'a ItemImpl,
    contract: &'a Type,
    msg_ty: MsgType,
    error: &'a ContractErrorAttr,
    custom: &'a Custom<'a>,
    interfaces: &'a Interfaces,
    variants: MsgVariants<'a, GenericParam>,
}

impl<'a> GlueMessage<'a> {
    pub fn new(
        source: &'a ItemImpl,
        msg_ty: MsgType,
        error: &'a ContractErrorAttr,
        custom: &'a Custom,
        interfaces: &'a Interfaces,
        variants: MsgVariants<'a, GenericParam>,
    ) -> Self {
        GlueMessage {
            source,
            contract: &source.self_ty,
            msg_ty,
            error,
            custom,
            interfaces,
            variants,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            contract,
            msg_ty,
            error,
            custom,
            interfaces,
            variants,
        } = self;

        let generics: Vec<_> = source.generics.params.iter().collect();
        let interface_generic_args = interfaces.as_generic_args();
        let interface_used_generics = filter_generics(&generics, &interface_generic_args);
        let used_generics = variants.used_generics();

        let wrapper_generics: Vec<_> = interface_used_generics
            .iter()
            .chain(used_generics.iter())
            .unique()
            .copied()
            .collect();

        let unused_generics: Vec<_> = variants
            .unused_generics()
            .iter()
            .filter(|unused| !wrapper_generics.iter().any(|used| used == *unused))
            .collect();

        let full_where_clause = &source.generics.where_clause;
        let wrapper_predicates = filter_wheres(full_where_clause, &generics, &wrapper_generics);
        let wrapper_where_clause = if !wrapper_predicates.is_empty() {
            quote! { where #(#wrapper_predicates,)* }
        } else {
            quote! {}
        };

        let unused_generics = emit_bracketed_generics(&unused_generics);
        let bracketed_used_generics = emit_bracketed_generics(used_generics);
        let bracketed_wrapper_generics = emit_bracketed_generics(&wrapper_generics);

        let contract_enum_name = msg_ty.emit_msg_name(true);
        let enum_name = msg_ty.emit_msg_name(false);
        let contract_name = StripGenerics.fold_type((*contract).clone());

        let variants = interfaces.emit_glue_message_variants(msg_ty);

        let ep_name = msg_ty.emit_ep_name();
        let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), contract.span());
        let contract_variant = quote! { #contract_name ( #enum_name #bracketed_used_generics ) };
        let mut messages_call = interfaces.emit_messages_call(msg_ty);
        let prefixed_used_generics = if !used_generics.is_empty() {
            quote! { :: #bracketed_used_generics }
        } else {
            quote! {}
        };
        messages_call.push(quote! { &#messages_fn_name() });

        let variants_cnt = messages_call.len();

        let dispatch_arms = interfaces.interfaces().iter().map(|interface| {
            let ContractMessageAttr {
                variant,
                customs,
                ..
            } = interface;

            let ctx = match (msg_ty, customs.has_query) {
                (MsgType::Exec, true )=> quote! {
                    ( ctx.0.into_empty(), ctx.1, ctx.2)
                },
                (MsgType::Query, true )=> quote! {
                    ( ctx.0.into_empty(), ctx.1)
                },
                _=> quote! { ctx },
            };

            match (msg_ty, customs.has_msg) {
                (MsgType::Exec, true) => quote! {
                    #contract_enum_name:: #variant(msg) => #sylvia ::into_response::IntoResponse::into_response(msg.dispatch(contract, Into::into( #ctx ))?)
                },
                _ => quote! {
                    #contract_enum_name :: #variant(msg) => msg.dispatch(contract, Into::into( #ctx ))
                },
            }
        });

        let dispatch_arm =
            quote! {#contract_enum_name :: #contract_name (msg) => msg.dispatch(contract, ctx)};

        let interfaces_deserialization_attempts = interfaces.emit_deserialization_attempts(msg_ty);

        #[cfg(not(tarpaulin_include))]
        let contract_deserialization_attempt = quote! {
            let msgs = &#messages_fn_name();
            if msgs.into_iter().any(|msg| msg == &recv_msg_name) {
                match val.deserialize_into() {
                    Ok(msg) => return Ok(Self:: #contract_name (msg)),
                    Err(err) => return Err(D::Error::custom(err)).map(Self:: #contract_name )
                };
            }
        };

        let ctx_type = msg_ty.emit_ctx_type(&custom.query_or_default());
        let ret_type = msg_ty.emit_result_type(&custom.msg_or_default(), &error.error);

        let mut response_schemas_calls = interfaces.emit_response_schemas_calls(msg_ty);
        response_schemas_calls
            .push(quote! {#enum_name #prefixed_used_generics :: response_schemas_impl()});

        let response_schemas = match msg_ty {
            MsgType::Query => {
                #[cfg(not(tarpaulin_include))]
                {
                    quote! {
                        #[cfg(not(target_arch = "wasm32"))]
                        impl #bracketed_wrapper_generics #sylvia ::cw_schema::QueryResponses for #contract_enum_name #bracketed_wrapper_generics #wrapper_where_clause {
                            fn response_schemas_impl() -> std::collections::BTreeMap<String, #sylvia ::schemars::schema::RootSchema> {
                                let responses = [#(#response_schemas_calls),*];
                                responses.into_iter().flatten().collect()
                            }
                        }
                    }
                }
            }
            _ => {
                quote! {}
            }
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #[allow(clippy::derive_partial_eq_without_eq)]
                #[derive(#sylvia ::serde::Serialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
                #[serde(rename_all="snake_case", untagged)]
                pub enum #contract_enum_name #bracketed_wrapper_generics #wrapper_where_clause {
                    #(#variants,)*
                    #contract_variant
                }

                impl #bracketed_wrapper_generics #contract_enum_name #bracketed_wrapper_generics #wrapper_where_clause {
                    pub fn dispatch #unused_generics (
                        self,
                        contract: &#contract,
                        ctx: #ctx_type,
                    ) -> #ret_type #full_where_clause {
                        const _: () = {
                            let msgs: [&[&str]; #variants_cnt] = [#(#messages_call),*];
                            #sylvia ::utils::assert_no_intersection(msgs);
                        };

                        match self {
                            #(#dispatch_arms,)*
                            #dispatch_arm
                        }
                    }
                }

                #response_schemas

                impl<'de, #(#wrapper_generics,)* > serde::Deserialize<'de> for #contract_enum_name #bracketed_wrapper_generics #wrapper_where_clause {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where D: serde::Deserializer<'de>,
                    {
                        use serde::de::Error;

                        let val = #sylvia ::serde_value::Value::deserialize(deserializer)?;
                        let map = match &val {
                            #sylvia ::serde_value::Value::Map(map) => map,
                            _ => return Err(D::Error::custom("Wrong message format!"))
                        };
                        if map.len() != 1 {
                            return Err(D::Error::custom(format!("Expected exactly one message. Received {}", map.len())))
                        }

                        // Due to earlier size check of map this unwrap is safe
                        let recv_msg_name = map.into_iter().next().unwrap();

                        if let #sylvia ::serde_value::Value::String(recv_msg_name) = &recv_msg_name .0 {
                            #(#interfaces_deserialization_attempts)*
                            #contract_deserialization_attempt
                        }

                        let msgs: [&[&str]; #variants_cnt] = [#(#messages_call),*];
                        let mut err_msg = msgs.into_iter().flatten().fold(
                            // It might be better to forward the error or serialization, but we just
                            // deserialized it from JSON, not reason to expect failure here.
                            format!(
                                "Unsupported message received: {}. Messages supported by this contract: ",
                                #sylvia ::serde_json::to_string(&val).unwrap_or_else(|_| String::new())
                            ),
                            |mut acc, message| acc + message + ", ",
                        );
                        err_msg.truncate(err_msg.len() - 2);
                        Err(D::Error::custom(err_msg))
                    }
                }
            }
        }
    }
}

pub struct ContractApi<'a> {
    source: &'a ItemImpl,
    exec_variants: MsgVariants<'a, GenericParam>,
    query_variants: MsgVariants<'a, GenericParam>,
    instantiate_variants: MsgVariants<'a, GenericParam>,
    migrate_variants: MsgVariants<'a, GenericParam>,
    generics: &'a [&'a GenericParam],
    custom: &'a Custom<'a>,
    interfaces: &'a Interfaces,
}

impl<'a> ContractApi<'a> {
    pub fn new(
        source: &'a ItemImpl,
        generics: &'a [&'a GenericParam],
        custom: &'a Custom<'a>,
        interfaces: &'a Interfaces,
    ) -> Self {
        let exec_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Exec,
            generics,
            &source.generics.where_clause,
        );

        let query_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Query,
            generics,
            &source.generics.where_clause,
        );

        let instantiate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Instantiate,
            generics,
            &source.generics.where_clause,
        );

        let migrate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Migrate,
            generics,
            &source.generics.where_clause,
        );

        Self {
            source,
            exec_variants,
            query_variants,
            instantiate_variants,
            migrate_variants,
            generics,
            custom,
            interfaces,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            exec_variants,
            query_variants,
            instantiate_variants,
            migrate_variants,
            generics,
            custom,
            interfaces,
        } = self;

        let where_clause = &source.generics.where_clause;
        let contract_name = &source.self_ty;
        let exec_generics = &exec_variants.used_generics;
        let query_generics = &query_variants.used_generics;
        let instantiate_generics = &instantiate_variants.used_generics;
        let migrate_generics = &migrate_variants.used_generics;

        let interface_generics = interfaces.as_generic_args();
        let interface_generics = filter_generics(generics, &interface_generics);

        let contract_exec_generics: Vec<_> = interface_generics
            .iter()
            .chain(exec_generics.iter())
            .unique()
            .collect();
        let contract_query_generics: Vec<_> = interface_generics
            .iter()
            .chain(query_generics.iter())
            .unique()
            .collect();

        let bracket_generics = emit_bracketed_generics(generics);
        let exec_bracketed_generics = emit_bracketed_generics(exec_generics);
        let query_bracketed_generics = emit_bracketed_generics(query_generics);
        let instantiate_bracketed_generics = emit_bracketed_generics(instantiate_generics);
        let migrate_bracketed_generics = emit_bracketed_generics(migrate_generics);
        let contract_exec_bracketed_generics = emit_bracketed_generics(&contract_exec_generics);
        let contract_query_bracketed_generics = emit_bracketed_generics(&contract_query_generics);

        let migrate_type = match !migrate_variants.variants().is_empty() {
            true => quote! { type Migrate = MigrateMsg #migrate_bracketed_generics; },
            false => quote! { type Migrate = #sylvia ::cw_std::Empty; },
        };
        let custom_query = custom.query_or_default();

        quote! {
            impl #bracket_generics #sylvia ::types::ContractApi for #contract_name #where_clause {
                type ContractExec = ContractExecMsg #contract_exec_bracketed_generics;
                type ContractQuery = ContractQueryMsg #contract_query_bracketed_generics;
                type Exec = ExecMsg #exec_bracketed_generics;
                type Query = QueryMsg #query_bracketed_generics;
                type Instantiate = InstantiateMsg #instantiate_bracketed_generics;
                #migrate_type
                type Remote<'remote> = Remote<'remote, #(#generics,)* >;
                type Querier<'querier> = BoundQuerier<'querier, #custom_query, #(#generics,)* >;
            }
        }
    }
}

pub struct InterfaceApi<'a> {
    source: &'a ItemTrait,
    custom: &'a Custom<'a>,
    associated_types: &'a AssociatedTypes<'a>,
}

impl<'a> InterfaceApi<'a> {
    pub fn new(
        source: &'a ItemTrait,
        associated_types: &'a AssociatedTypes<'a>,
        custom: &'a Custom<'a>,
    ) -> Self {
        Self {
            source,
            custom,
            associated_types,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            custom,
            associated_types,
        } = self;

        let generics = associated_types.as_names();
        let exec_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Exec,
            &generics,
            &source.generics.where_clause,
        );

        let query_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Query,
            &generics,
            &source.generics.where_clause,
        );

        let where_clause = &self.associated_types.as_where_clause();
        let custom_query = custom.query_or_default();
        let exec_generics = &exec_variants.used_generics;
        let query_generics = &query_variants.used_generics;

        let bracket_generics = emit_bracketed_generics(&generics);
        let exec_bracketed_generics = emit_bracketed_generics(exec_generics);
        let query_bracketed_generics = emit_bracketed_generics(query_generics);

        let phantom = if !generics.is_empty() {
            quote! {
                _phantom: std::marker::PhantomData<( #(#generics,)* )>,
            }
        } else {
            quote! {}
        };

        quote! {
            pub struct Api #bracket_generics {
                #phantom
            }

            impl #bracket_generics #sylvia ::types::InterfaceApi for Api #bracket_generics #where_clause {
                type Exec = ExecMsg #exec_bracketed_generics;
                type Query = QueryMsg #query_bracketed_generics;
                type Querier<'querier> = BoundQuerier<'querier, #custom_query, #(#generics,)* >;
            }
        }
    }
}

pub struct EntryPoints<'a> {
    source: &'a ItemImpl,
    name: Type,
    error: Type,
    custom: Custom<'a>,
    override_entry_points: OverrideEntryPoints,
    generics: Vec<&'a GenericParam>,
    where_clause: &'a Option<WhereClause>,
    attrs: EntryPointArgs<'a>,
}

impl<'a> EntryPoints<'a> {
    pub fn new(source: &'a ItemImpl, attrs: EntryPointArgs<'a>) -> Self {
        let sylvia = crate_module();
        let name = StripGenerics.fold_type(*source.self_ty.clone());
        let override_entry_points = OverrideEntryPoints::new(&source.attrs);

        let error = source
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("error"))
            .and_then(
                |attr| match ContractErrorAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(error) => Some(error.error),
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        None
                    }
                },
            )
            .unwrap_or_else(|| parse_quote! { #sylvia ::cw_std::StdError });

        let generics: Vec<_> = source.generics.params.iter().collect();
        let where_clause = &source.generics.where_clause;
        let custom = Custom::new(&source.attrs);

        Self {
            source,
            name,
            error,
            custom,
            override_entry_points,
            generics,
            where_clause,
            attrs,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            source,
            name,
            error,
            custom,
            override_entry_points,
            generics,
            where_clause,
            attrs,
        } = self;
        let sylvia = crate_module();

        let custom = match &attrs.custom {
            Some(custom) => custom,
            None => custom,
        };

        let custom_msg = custom.msg_or_default();
        let custom_query = custom.query_or_default();

        let instantiate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Instantiate,
            generics,
            where_clause,
        );
        let exec_variants =
            MsgVariants::new(source.as_variants(), MsgType::Exec, generics, where_clause);
        let query_variants =
            MsgVariants::new(source.as_variants(), MsgType::Query, generics, where_clause);
        let migrate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Migrate,
            generics,
            where_clause,
        );
        let reply =
            MsgVariants::<GenericParam>::new(source.as_variants(), MsgType::Reply, &[], &None)
                .variants()
                .iter()
                .map(|variant| variant.function_name.clone())
                .next();
        let contract_generics = match &attrs.generics {
            Some(generics) => quote! { ::< #generics > },
            None => quote! {},
        };

        #[cfg(not(tarpaulin_include))]
        {
            let entry_points = [instantiate_variants, exec_variants, query_variants]
                .into_iter()
                .map(
                    |variants| match override_entry_points.get_entry_point(variants.msg_ty) {
                        Some(_) => quote! {},
                        None => variants.emit_default_entry_point(
                            &custom_msg,
                            &custom_query,
                            name,
                            error,
                            &attrs.generics,
                        ),
                    },
                );

            let migrate_not_overridden = override_entry_points
                .get_entry_point(MsgType::Migrate)
                .is_none();

            let migrate = if migrate_not_overridden && migrate_variants.get_only_variant().is_some()
            {
                migrate_variants.emit_default_entry_point(
                    &custom_msg,
                    &custom_query,
                    name,
                    error,
                    &attrs.generics,
                )
            } else {
                quote! {}
            };

            let reply_ep = override_entry_points
                .get_entry_point(MsgType::Reply)
                .map(|_| quote! {})
                .unwrap_or_else(|| match reply {
                    Some(reply) => quote! {
                        #[#sylvia ::cw_std::entry_point]
                        pub fn reply(
                            deps: #sylvia ::cw_std::DepsMut< #custom_query >,
                            env: #sylvia ::cw_std::Env,
                            msg: #sylvia ::cw_std::Reply,
                        ) -> Result<#sylvia ::cw_std::Response < #custom_msg >, #error> {
                            #name #contract_generics ::new(). #reply((deps, env).into(), msg).map_err(Into::into)
                        }
                    },
                    _ => quote! {},
                });

            quote! {
                pub mod entry_points {
                    use super::*;

                    #(#entry_points)*

                    #migrate

                    #reply_ep
                }
            }
        }
    }
}
