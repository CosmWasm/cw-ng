use generic::Generic;
use sylvia::ctx::{ExecCtx, QueryCtx, SudoCtx};
use sylvia::cw_std::{CosmosMsg, Response};
use sylvia::serde::Deserialize;
use sylvia::types::{CustomMsg, CustomQuery};

use crate::contract::{GenericsForwardedContract, SvCustomMsg};
use crate::error::ContractError;

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
        CustomMsgT,
        CustomQueryT,
        FieldT,
    > Generic
    for GenericsForwardedContract<
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
        CustomMsgT,
        CustomQueryT,
        FieldT,
    >
where
    for<'msg_de> InstantiateT: sylvia::cw_std::CustomMsg + Deserialize<'msg_de> + 'msg_de,
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
    CustomMsgT: CustomMsg + 'static,
    CustomQueryT: CustomQuery + 'static,
    FieldT: 'static,
{
    type Error = ContractError;
    type Exec1T = Exec1T;
    type Exec2T = Exec2T;
    type Exec3T = Exec3T;
    type Query1T = Query1T;
    type Query2T = Query2T;
    type Query3T = Query3T;
    type Sudo1T = Sudo1T;
    type Sudo2T = Sudo2T;
    type Sudo3T = Sudo3T;
    type RetT = SvCustomMsg;

    fn generic_exec_one(
        &self,
        _ctx: ExecCtx,
        _msgs1: Vec<CosmosMsg<Self::Exec1T>>,
        _msgs2: Vec<CosmosMsg<Self::Exec2T>>,
    ) -> Result<Response, Self::Error> {
        Ok(Response::new())
    }

    fn generic_exec_two(
        &self,
        _ctx: ExecCtx,
        _msgs2: Vec<CosmosMsg<Self::Exec2T>>,
        _msgs3: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> Result<Response, Self::Error> {
        Ok(Response::new())
    }

    fn generic_query_one(
        &self,
        _ctx: QueryCtx,
        _msg1: Self::Query1T,
        _msg2: Self::Query2T,
    ) -> Result<SvCustomMsg, Self::Error> {
        Ok(SvCustomMsg {})
    }

    fn generic_query_two(
        &self,
        _ctx: QueryCtx,
        _msg1: Self::Query2T,
        _msg2: Self::Query3T,
    ) -> Result<SvCustomMsg, Self::Error> {
        Ok(SvCustomMsg {})
    }

    fn generic_sudo_one(
        &self,
        _ctx: SudoCtx,
        _msgs1: CosmosMsg<Self::Sudo1T>,
        _msgs2: CosmosMsg<Self::Sudo2T>,
    ) -> Result<Response, Self::Error> {
        Ok(Response::new())
    }

    fn generic_sudo_two(
        &self,
        _ctx: SudoCtx,
        _msgs1: CosmosMsg<Self::Sudo2T>,
        _msgs2: CosmosMsg<Self::Sudo3T>,
    ) -> Result<Response, Self::Error> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::sv::mt::CodeId;
    use crate::contract::{GenericsForwardedContract, SvCustomMsg, SvCustomQuery};
    use generic::sv::mt::GenericProxy;
    use sylvia::cw_multi_test::{BasicApp, IntoBech32};
    use sylvia::cw_std::CosmosMsg;
    use sylvia::multitest::App;

    #[test]
    fn proxy_methods() {
        let app = App::<BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        #[allow(clippy::type_complexity)]
        let code_id: CodeId<
            GenericsForwardedContract<
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
                SvCustomMsg,
                SvCustomQuery,
                String,
            >,
            _,
        > = CodeId::store_code(&app);

        let owner = "owner".into_bech32();

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner.as_str())
            .call(&owner)
            .unwrap();

        contract
            .generic_exec_one(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(&owner)
            .unwrap();

        contract
            .generic_exec_two(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(&owner)
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
