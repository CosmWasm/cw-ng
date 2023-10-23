use cosmwasm_std::{Reply, Response, StdResult};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::types::{
    CustomMsg, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SvCustomMsg,
};
use sylvia::{contract, schemars};

pub struct GenericContract<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType>(
    std::marker::PhantomData<(
        InstantiateParam,
        ExecParam,
        QueryParam,
        MigrateParam,
        RetType,
    )>,
);

#[contract]
#[messages(cw1 as Cw1: custom(msg))]
#[messages(generic<SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg> as Generic: custom(msg))]
#[messages(custom_and_generic<SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg> as CustomAndGeneric)]
#[sv::custom(msg=SvCustomMsg)]
impl<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType>
    GenericContract<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType>
where
    for<'msg_de> InstantiateParam: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecParam: CustomMsg + DeserializeOwned + 'static,
    QueryParam: CustomMsg + DeserializeOwned + 'static,
    MigrateParam: CustomMsg + DeserializeOwned + 'static,
    RetType: CustomMsg + DeserializeOwned + 'static,
{
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx,
        _msg: InstantiateParam,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn contract_execute(
        &self,
        _ctx: ExecCtx,
        _msg: ExecParam,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn contract_query(
        &self,
        _ctx: QueryCtx,
        _msg: QueryParam,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(migrate)]
    pub fn migrate(
        &self,
        _ctx: MigrateCtx,
        _msg: MigrateParam,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[msg(reply)]
    fn reply(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use super::multitest_utils::CodeId;
    use sylvia::multitest::App;
    use sylvia::types::SvCustomMsg;

    #[test]
    fn generic_contract() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg>>::custom(|_, _, _| {});
        let code_id: CodeId<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            super::SvCustomMsg,
            super::SvCustomMsg,
            _,
        > = CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract.contract_execute(SvCustomMsg).call(owner).unwrap();
        contract.contract_query(SvCustomMsg).unwrap();
        contract
            .migrate(SvCustomMsg)
            .call(owner, code_id.code_id())
            .unwrap();
    }
}
