use cosmwasm_std::{CosmosMsg, CustomMsg, Response, StdError, StdResult};
use generic::Generic;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg};

#[contract(module = crate::contract)]
#[messages(generic as Generic)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<InstantiateT, ExecT, QueryT, MigrateT, FieldT> Generic<SvCustomMsg, SvCustomMsg, SvCustomMsg>
    for crate::contract::GenericContract<InstantiateT, ExecT, QueryT, MigrateT, FieldT>
where
    for<'msg_de> InstantiateT: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecT: CustomMsg + DeserializeOwned + 'static,
    QueryT: CustomMsg + DeserializeOwned + 'static,
    MigrateT: CustomMsg + DeserializeOwned + 'static,
    FieldT: 'static,
{
    type Error = StdError;

    #[msg(exec)]
    fn generic_exec(
        &self,
        _ctx: ExecCtx,
        _msgs: Vec<CosmosMsg<SvCustomMsg>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    // Sylvia will fail if single type is used to match against two different generics
    // It's because we have to map unique generics used as they can be used multiple times.
    // If for some reason like here one type would be used in place of two generics either full
    // path or some alias has to be used.
    //
    // Sylvia will fail to recognize generic used if their path is different.
    // F.e. if we this query would return `SvCustomMsg` and we would pass
    // `sylvia::types::SvCustomMsg` to the `Generic` trait paths would not match.
    #[msg(query)]
    fn generic_query(&self, _ctx: QueryCtx, _msg: SvCustomMsg) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

#[cfg(test)]
mod tests {
    use super::sv::test_utils::Generic;
    use crate::contract::sv::multitest_utils::CodeId;
    use cosmwasm_std::CosmosMsg;
    use sylvia::multitest::App;
    use sylvia::types::{SvCustomMsg, SvCustomQuery};

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id: CodeId<
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            String,
            _,
        > = CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract
            .generic_exec(vec![CosmosMsg::Custom(SvCustomMsg {})])
            .call(owner)
            .unwrap();
        contract.generic_query(SvCustomMsg).unwrap();
    }
}
