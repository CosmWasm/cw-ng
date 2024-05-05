use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, GenericParam, Ident, Type};

use crate::crate_module;
use crate::parser::attributes::msg::MsgType;
use crate::parser::Customs;

impl MsgType {
    pub fn emit_ctx_type(self, query_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate => quote! {
                (#sylvia ::cw_std::DepsMut< #query_type >, #sylvia ::cw_std::Env, #sylvia ::cw_std::MessageInfo)
            },
            Migrate | Reply | Sudo => quote! {
                (#sylvia ::cw_std::DepsMut< #query_type >, #sylvia ::cw_std::Env)
            },
            Query => quote! {
                (#sylvia ::cw_std::Deps< #query_type >, #sylvia ::cw_std::Env)
            },
        }
    }

    pub fn emit_ctx_dispatch_values(self, customs: &Customs) -> TokenStream {
        use MsgType::*;

        match (self, customs.has_query) {
            (Exec, true) => quote! {
                (ctx.0.into_empty(), ctx.1, ctx.2)
            },
            (Query, true) | (Sudo, true) => quote! {
                (ctx.0.into_empty(), ctx.1)
            },
            _ => quote! { ctx },
        }
    }

    pub fn emit_ctx_params(self, query_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate => quote! {
                deps: #sylvia ::cw_std::DepsMut< #query_type>, env: #sylvia ::cw_std::Env, info: #sylvia ::cw_std::MessageInfo
            },
            Migrate | Reply | Sudo => quote! {
                deps: #sylvia ::cw_std::DepsMut< #query_type>, env: #sylvia ::cw_std::Env
            },
            Query => quote! {
                deps: #sylvia ::cw_std::Deps< #query_type>, env: #sylvia ::cw_std::Env
            },
        }
    }

    pub fn emit_ep_name(self) -> Ident {
        match self {
            Self::Exec => parse_quote! { execute },
            Self::Instantiate => parse_quote! { instantiate },
            Self::Migrate => parse_quote! { migrate },
            Self::Sudo => parse_quote! { sudo },
            Self::Reply => parse_quote! { reply },
            Self::Query => parse_quote! { query },
        }
    }

    pub fn emit_ctx_values(self) -> TokenStream {
        use MsgType::*;

        match self {
            Exec | Instantiate => quote! { deps, env, info },
            Migrate | Reply | Query | Sudo => quote! { deps, env },
        }
    }

    /// Emits type which should be returned by dispatch function for this kind of message
    pub fn emit_result_type(self, msg_type: &Type, err_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate | Migrate | Reply | Sudo => {
                quote! {
                    std::result::Result< #sylvia:: cw_std::Response <#msg_type>, #err_type>
                }
            }
            Query => quote! {
                std::result::Result<#sylvia ::cw_std::Binary, #err_type>
            },
        }
    }

    pub fn emit_msg_wrapper_name(&self) -> Ident {
        match self {
            MsgType::Exec => parse_quote! { ContractExecMsg },
            MsgType::Query => parse_quote! { ContractQueryMsg },
            MsgType::Sudo => parse_quote! { ContractSudoMsg },
            _ => self.emit_msg_name(),
        }
    }

    pub fn emit_msg_name(&self) -> Ident {
        match self {
            MsgType::Exec => parse_quote! { ExecMsg },
            MsgType::Query => parse_quote! { QueryMsg },
            MsgType::Instantiate => parse_quote! { InstantiateMsg },
            MsgType::Migrate => parse_quote! { MigrateMsg },
            MsgType::Reply => parse_quote! { ReplyMsg },
            MsgType::Sudo => parse_quote! { SudoMsg },
        }
    }

    pub fn as_accessor_wrapper_name(&self) -> Type {
        match self {
            MsgType::Exec => parse_quote! { ContractExec },
            MsgType::Query => parse_quote! { ContractQuery },
            MsgType::Sudo => parse_quote! { ContractSudo },
            _ => self.as_accessor_name(),
        }
    }

    pub fn as_accessor_name(&self) -> Type {
        match self {
            MsgType::Instantiate => parse_quote! { Instantiate },
            MsgType::Exec => parse_quote! { Exec },
            MsgType::Query => parse_quote! { Query },
            MsgType::Migrate => parse_quote! { Migrate },
            MsgType::Sudo => parse_quote! { Sudo },
            MsgType::Reply => parse_quote! { Reply },
        }
    }

    pub fn emit_phantom_variant(&self, generics: &[&GenericParam]) -> TokenStream {
        match self {
            _ if generics.is_empty() => quote! {},
            MsgType::Query => quote! {
                #[serde(skip)]
                #[returns(( #(#generics,)* ))]
                _Phantom(std::marker::PhantomData<( #(#generics,)* )>),
            },
            _ => quote! {
                #[serde(skip)]
                _Phantom(std::marker::PhantomData<( #(#generics,)* )>),
            },
        }
    }

    pub fn emit_derive_call(&self) -> TokenStream {
        let sylvia = crate_module();
        match self {
            MsgType::Query => quote! {
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema, #sylvia:: cw_schema::QueryResponses, #sylvia:: cw_orch::QueryFns)]
            },
            MsgType::Exec => quote! {
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema, #sylvia:: cw_orch::ExecuteFns)]
            },
            _ => quote! {
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
            },
        }
    }
}
