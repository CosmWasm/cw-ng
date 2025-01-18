# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over major releases of `sylvia`. Note that you can also view the [complete CHANGELOG](https://github.com/CosmWasm/sylvia/blob/main/CHANGELOG.md) to understand the differences.

## 1.1.0 -> 1.2.0

### Explicit generic customs in entry_points macro

Sylvia can deduce the types used in place of generic customs.
The `, custom(msg=..., query=...)` part is now not supported and can be safely removed.

```diff
-#[entry_points(generics<Empty, Empty>, custom(msg=Empty, query=Empty))]
+#[entry_points(generics<Empty, Empty>)]
#[contract]
#[sv::custom(msg=E, query=Q)]
impl<E, Q> CounterContract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
}
```

### Removed `InterfaceApi` trait

The `InterfaceApi` trait has been deprecated in favor of the `InterfaceMessagesApi`.
It will be removed in the `2.0.0` release.

```diff
-let _ = <dyn super::sv::Api as sylvia::types::InterfaceApi>::Query::query_something();
+let _ = <dyn super::MyInterface as super::sv::InterfaceMessagesApi>::Query::query_something();
```

## 1.0.2 -> 1.1.0

### Generics in `sv::messages` not required

```diff
-#[contract]
-#[sv::messages(generic<SomeType1, SomeType2, SomeType3> as Generic
-impl Contract {}
+#[contract]
+#[sv::messages(generic as Generic)]
+impl Contract {}
```

This change is optional, since the generics are still accepted by the parser. Though they are
ignored in the further process.

### CodeId generic over the Contract type

```diff
-let code_id: CodeId<
-    SvCustomMsg,
-    SvCustomMsg,
-    _,
-> = CodeId::store_code(&app);
+let code_id: CodeId<
+    GenericContract<
+        SvCustomMsg,
+        SvCustomMsg,
+    >,
+    _,
+> = CodeId::store_code(&app);
```

### Lifetime elision in a contract's impl block not supported

```diff
-#[contract]
-impl Cw1SubkeysContract<'_> {
-    // [...]
-}
+#[contract]
+impl<'a> Cw1SubkeysContract<'a> {
+    // [...]
+}
```

## 0.10.0 -> 1.0.0

### Update deps to 2.0.0

In Cargo.toml:

```diff
-cosmwasm-schema = "1.5.0"
-cosmwasm-std = "1.5.0"
-cw-multi-test = "0.20.0"
-cw-storage-plus = "1.2.0"
-cw-utils = "1.0.2"
-cw2 = "1.1.2"
+cosmwasm-schema = "2.0.0"
+cosmwasm-std = "2.0.0"
+cw-multi-test = "2.0.0"
+cw-storage-plus = "2.0.0"
+cw-utils = "2.0.0"
+cw2 = "2.0.0"
```

In a multi-test code:

```diff
-let addr = Addr::unchecked("addr0001");
+use cw_multi_test::IntoBech32;
+let addr = "addr0001".into_bech32();
let contract = code_id
    .instantiate(vec![owner.to_owned()], false)
    .with_label("Sublist contract")
-    .call(addr)
+    .call(&addr)
    .unwrap();
```

In the contract's code:

```diff
struct Contract<'a> {
-    data: Item<'static, ContractData>,
-    admins: Map<'static, &'a Addr, Empty>,
+    data: Item<ContractData>,
+    admins: Map<&'a Addr, Empty>,
}
```

## 0.9.3 -> 0.10.0

### Querier in Multitest App

```diff
let version: ContractVersion =
-        query_contract_info(&app.app_mut().wrap(), contract.contract_addr.to_string()).unwrap();
+        query_contract_info(&app.querier(), contract.contract_addr.to_string()).unwrap();
```

### Multitest module name

```diff
-use contract::multitest_utils::Group;
+use contract::mt::Group;
```

### BoundQuerier improve

```diff
let querier = sylvia::types::BoundQuerier::<
    _,
-    std::marker::PhantomData<(
-        SvCustomMsg,
-        SvCustomMsg,
-        SvCustomMsg,
-        SvCustomMsg,
-    )>,
+    &dyn super::CustomAndGeneric<
+        RetT = SvCustomMsg,
+        Exec = SvCustomMsg,
+        Query = SvCustomMsg,
+        Sudo = SvCustomMsg,
+        Error = (),
+        ExecC = (),
+        QueryC = (),
+    >,
>::borrowed(&contract, &querier_wrapper);
```

### Remove `#[contract(module=...)]` support

There is no need to provide any additional data to an interface implementation on a contract.

```diff
-#[contract(module=crate::contract)]
-#[sv::messages(cw1 as Cw1)]
impl Cw1 for CustomContract {
    type Error = StdError;

    #[sv::msg(exec)]
    fn execute(&self, _ctx: ExecCtx, _msgs: Vec<CosmosMsg>) -> StdResult<Response> {
        Ok(Response::new())
    }
}
```

### Sylvia attributes

Each sylvia attribute that is used by `#[contract]` and `#[interface]` macro needs to be prefixed with `sv::`, for e.g.:

```diff
#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
-#[messages(cw1 as Cw1: custom(msg, query))]
+#[sv::messages(cw1 as Cw1: custom(msg, query))]
#[sv::custom(query=CounterQuery, msg=CounterMsg)]
impl CustomContract {
    pub const fn new() -> Self {
        Self {
            sudo_counter: Item::new("sudo_counter"),
        }
    }

-    #[msg(instantiate)]
+    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx<CounterQuery>,
    ) -> StdResult<Response<CounterMsg>> {
        self.sudo_counter.save(ctx.deps.storage, &0)?;
        Ok(Response::default())
    }
}
```

### Multitest proxy

Since `0.10.0` Sylvia won't generate multitest Proxy types for the `Interface` macro call. Instead, all the methods from interfaces are directly implemented on the contract's multitest proxy.
To use methods from an implemented interface like before, the user has to import the multitest trait from the module in which the interface is implemented.

```diff
-let resp = contract
-   .cw1_proxy()
-   .can_execute()
-   .unwrap();
+let resp = contract
+   .can_execute()
+   .unwrap();
```

### Associated types in generics

`Sylvia` interface is meant to be implemented on contract only once. Because of that, we decided to remove support for generics in interfaces.
Instead, when migrating from `0.9.3` to `0.10.0`, the user must replace generics with associated types.

## 0.8.1 -> 0.9.0

### `sv` module

Sylvia from now on will generate all the code in the `sv` module. This means that you will need to update your imports to use the new module.

### Implementing non-custom interface on custom contract

In Sylvia `0.8.x` there was missing check for the `,` in `#[messages(cw1 as Cw1: custom(msg query))]`.
Since `0.9.0` Sylvia expects user to split `msg` and `query` with `,` as such `#[messages(cw1 as Cw1: custom(msg, query))]`.

## 0.5.0 -> 0.6.0

### Context distinction

```
error[E0308]: mismatched types
  --> contracts/cw1-subkeys/src/contract.rs:56:49
   |
56 |         let result = self.whitelist.instantiate(ctx.branch(), admins, mutable)?;
   |                                     ----------- ^^^^^^^^^^^^ expected `ExecCtx<'_>`, found `InstantiateCtx<'_>`
   |                                     |
   |                                     arguments to this method are incorrect
   |
   = note: expected struct `ExecCtx<'_>`
              found struct `InstantiateCtx<'_>`
```

`InstantiateCtx` and `ExecCtx` could be previously used interchangeably. They are currently separate
  types. The same applies to `ReplyCtx`/`MigrateCtx` pair.

## 0.4.2 -> 0.5.0

### `module` attribute repurpose

```diff
-#[contract(module=some_contract)]
-impl SomeContract {
-    ...
-}
+mod some_contract {
+   #[contract]
+   impl SomeContract {
+       ...
+   }
+}
```

`module` attr for macro contract should now point to your contract implementation
