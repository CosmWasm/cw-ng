use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use generic::Generic;
use serde::Deserialize;
use sylvia::contract;
use sylvia::types::{CustomMsg, ExecCtx, QueryCtx, SudoCtx, SvCustomMsg};

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
        Sudo1T,
        Sudo2T,
        Sudo3T,
        MigrateT,
        FieldT,
    > Generic
    for crate::contract::GenericContract<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        Sudo1T,
        Sudo2T,
        Sudo3T,
        MigrateT,
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
    Sudo1T: CustomMsg + 'static,
    Sudo2T: CustomMsg + 'static,
    Sudo3T: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
    FieldT: 'static,
{
    type Error = StdError;
    type Exec1T = SvCustomMsg;
    type Exec2T = SvCustomMsg;
    type Exec3T = SvCustomMsg;
    type Query1T = SvCustomMsg;
    type Query2T = SvCustomMsg;
    type Query3T = SvCustomMsg;
    type Sudo1T = SvCustomMsg;
    type Sudo2T = SvCustomMsg;
    type Sudo3T = SvCustomMsg;
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
        _msgs1: Vec<CosmosMsg<Self::Exec2T>>,
        _msgs2: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn generic_query_one(
        &self,
        _ctx: QueryCtx,
        _msg1: Self::Query1T,
        _msg2: Self::Query2T,
    ) -> StdResult<Self::RetT> {
        Ok(SvCustomMsg {})
    }

    #[msg(query)]
    fn generic_query_two(
        &self,
        _ctx: QueryCtx,
        _msg1: Self::Query2T,
        _msg2: Self::Query3T,
    ) -> StdResult<Self::RetT> {
        Ok(SvCustomMsg {})
    }

    #[msg(sudo)]
    fn generic_sudo_one(
        &self,
        _ctx: SudoCtx,
        _msgs1: CosmosMsg<Self::Sudo1T>,
        _msgs2: CosmosMsg<Self::Sudo2T>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(sudo)]
    fn generic_sudo_two(
        &self,
        _ctx: SudoCtx,
        _msgs1: CosmosMsg<Self::Sudo2T>,
        _msgs2: CosmosMsg<Self::Sudo3T>,
    ) -> StdResult<Response> {
        Ok(Response::new())
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
        #[allow(clippy::type_complexity)]
        let code_id: CodeId<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
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

        contract
            .generic_sudo_one(
                CosmosMsg::Custom(SvCustomMsg),
                CosmosMsg::Custom(SvCustomMsg),
            )
            .unwrap();
        contract
            .generic_sudo_two(
                CosmosMsg::Custom(SvCustomMsg),
                CosmosMsg::Custom(SvCustomMsg),
            )
            .unwrap();
    }
}
