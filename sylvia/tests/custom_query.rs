use cosmwasm_std::{CustomQuery, Response, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::contract;
use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyQuery;

impl CustomQuery for MyQuery {}

pub struct MyContract;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct SomeResponse;

mod some_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia::{contract, interface};

    use crate::{MyQuery, SomeResponse};

    #[interface]
    #[sv::custom(query=MyQuery)]
    pub trait SomeInterface {
        type Error: From<StdError>;

        #[cfg(not(tarpaulin_include))]
        #[msg(query)]
        fn interface_query(&self, ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse>;

        #[cfg(not(tarpaulin_include))]
        #[msg(exec)]
        fn interface_exec(&self, ctx: ExecCtx<MyQuery>) -> StdResult<Response>;
    }

    #[contract(module=super)]
    #[sv::custom(query=MyQuery)]
    impl SomeInterface for crate::MyContract {
        type Error = StdError;

        #[msg(query)]
        fn interface_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[msg(exec)]
        fn interface_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

#[contract]
#[messages(some_interface as SomeInterface)]
#[sv::custom(query=MyQuery)]
impl MyContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[msg(exec)]
    pub fn some_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[msg(query)]
    pub fn some_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
        Ok(SomeResponse)
    }

    #[cfg(not(tarpaulin_include))]
    #[msg(migrate)]
    pub fn some_migrate(&self, _ctx: MigrateCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }
}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use crate::{MyContract, MyQuery};
    use cosmwasm_std::Empty;
    use sylvia::multitest::App;

    use crate::some_interface::test_utils::SomeInterface;

    #[test]
    fn test_custom() {
        let _ = MyContract::new();
        let app = App::<cw_multi_test::BasicApp<Empty, MyQuery>>::custom(|_, _, _| {});
        let code_id = crate::multitest_utils::CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("MyContract")
            .call(owner)
            .unwrap();

        contract.some_exec().call(owner).unwrap();
        contract.some_query().unwrap();

        // Interface messsages
        contract.some_interface_proxy().interface_query().unwrap();
        contract
            .some_interface_proxy()
            .interface_exec()
            .call(owner)
            .unwrap();
    }
}
