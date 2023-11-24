use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use generic::Generic;
use serde::Deserialize;
use sylvia::contract;
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx, SvCustomMsg};

#[contract(module = crate::contract)]
#[messages(generic as Generic)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
        FieldT,
    > Generic
    for crate::contract::GenericsForwardedContract<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
        FieldT,
    >
where
    for<'msg_de> InstantiateT: cosmwasm_std::CustomMsg + Deserialize<'msg_de> + 'msg_de,
    Exec1T: CustomMsg + 'static,
    Exec2T: CustomMsg + 'static,
    Exec3T: CustomMsg + 'static,
    Query1T: CustomMsg + 'static,
    Query2T: CustomMsg + 'static,
    Query3T: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
    CustomMsgT: CustomMsg + 'static,
    CustomQueryT: CustomQuery + 'static,
    FieldT: 'static,
{
    type Error = StdError;
    type Exec1T = Exec1T;
    type Exec2T = Exec2T;
    type Exec3T = Exec3T;
    type Query1T = Query1T;
    type Query2T = Query2T;
    type Query3T = Query3T;
    type RetT = SvCustomMsg;

    #[msg(exec)]
    fn generic_exec_one(
        &self,
        _ctx: ExecCtx,
        _msgs1: Vec<CosmosMsg<Self::Exec1T>>,
        _msgs2: Vec<CosmosMsg<Self::Exec2T>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn generic_exec_two(
        &self,
        _ctx: ExecCtx,
        _msgs2: Vec<CosmosMsg<Self::Exec2T>>,
        _msgs3: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn generic_query_one(
        &self,
        _ctx: QueryCtx,
        _msg1: Self::Query1T,
        _msg2: Self::Query2T,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }

    #[msg(query)]
    fn generic_query_two(
        &self,
        _ctx: QueryCtx,
        _msg1: Self::Query2T,
        _msg2: Self::Query3T,
    ) -> StdResult<SvCustomMsg> {
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
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomQuery,
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
            .generic_exec_one(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(owner)
            .unwrap();

        contract
            .generic_exec_two(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(owner)
            .unwrap();
        contract
            .generic_query_one(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .generic_query_two(SvCustomMsg, SvCustomMsg)
            .unwrap();
    }
}