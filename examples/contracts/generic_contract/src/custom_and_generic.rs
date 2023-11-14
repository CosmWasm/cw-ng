use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use custom_and_generic::CustomAndGeneric;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg, SvCustomQuery};

#[contract(module = crate::contract)]
#[messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<InstantiateT, ExecT, QueryT, MigrateT, FieldT>
    CustomAndGeneric<
        SvCustomMsg,
        SvCustomMsg,
        sylvia::types::SvCustomQuery,
        sylvia::types::SvCustomMsg,
    > for crate::contract::GenericContract<InstantiateT, ExecT, QueryT, MigrateT, FieldT>
{
    type Error = StdError;

    #[msg(exec)]
    fn custom_generic_execute(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msgs: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn custom_generic_query(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg: sylvia::types::SvCustomMsg,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

#[cfg(test)]
mod tests {
    use super::sv::test_utils::CustomAndGeneric;
    use crate::contract::sv::multitest_utils::CodeId;
    use sylvia::{
        multitest::App,
        types::{SvCustomMsg, SvCustomQuery},
    };

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id = CodeId::<
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            String,
            _,
        >::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract.custom_generic_execute(vec![]).call(owner).unwrap();
        contract.custom_generic_query(SvCustomMsg {}).unwrap();
    }
}
