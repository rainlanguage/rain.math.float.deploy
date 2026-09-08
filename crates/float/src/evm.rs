use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolInterface;
use revm::context::result::{ExecutionResult, Output, SuccessReason};
use revm::context::{BlockEnv, CfgEnv, TxEnv};
use revm::database::InMemoryDB;
use revm::primitives::address;
use revm::state::{AccountInfo, Bytecode};
use revm::{Context, MainBuilder, MainContext, MainnetEvm, SystemCallEvm};
use std::cell::RefCell;

use crate::{DecimalFloat, FloatError};

#[cfg(any(test, feature = "test-harness"))]
use crate::TestDecimalFloat;

/// Fixed address where the DecimalFloat contract is deployed in the in-memory EVM.
/// This arbitrary address is used consistently across all Calculator instances.
pub(crate) const FLOAT_ADDRESS: Address = address!("00000000000000000000000000000000000f10a2");

#[cfg(any(test, feature = "test-harness"))]
/// Fixed address where the TestDecimalFloat contract is deployed in the in-memory EVM.
pub(crate) const TEST_FLOAT_ADDRESS: Address = address!("00000000000000000000000000000000000f10a3");

type EvmContext = Context<BlockEnv, TxEnv, CfgEnv, InMemoryDB>;
type LocalEvm = MainnetEvm<EvmContext>;

thread_local! {
    pub(crate) static LOCAL_EVM: RefCell<LocalEvm> = RefCell::new(build_evm());
}

/// The in-memory EVM every call runs against, with the contracts in place.
fn build_evm() -> LocalEvm {
    let mut db = InMemoryDB::default();
    place(
        &mut db,
        FLOAT_ADDRESS,
        "RAIN_MATH_FLOAT_ARTIFACT",
        &DecimalFloat::BYTECODE,
        &DecimalFloat::DEPLOYED_BYTECODE,
    );
    #[cfg(any(test, feature = "test-harness"))]
    place(
        &mut db,
        TEST_FLOAT_ADDRESS,
        "RAIN_MATH_FLOAT_TEST_ARTIFACT",
        &TestDecimalFloat::BYTECODE,
        &TestDecimalFloat::DEPLOYED_BYTECODE,
    );
    Context::mainnet().with_db(db).build_mainnet()
}

/// Places the contract compiled into this crate at `address`, as deployed.
#[cfg(not(feature = "test-harness"))]
fn place(
    db: &mut InMemoryDB,
    address: Address,
    _artifact_var: &str,
    _creation: &Bytes,
    runtime: &Bytes,
) {
    put(db, address, runtime.clone());
}

/// Places a contract at `address`. The bytecode is the one compiled into this
/// crate unless the environment variable named by `artifact_var` points at a
/// forge artifact of another build of the same ABI, such as a test concrete
/// compiled from the library source. `RAIN_MATH_FLOAT_DEPLOY_MODE=create`
/// runs the creation bytecode instead of placing the runtime bytecode, for a
/// concrete whose constructor sets immutables or deploys what it depends on.
#[cfg(feature = "test-harness")]
fn place(
    db: &mut InMemoryDB,
    address: Address,
    artifact_var: &str,
    creation: &Bytes,
    runtime: &Bytes,
) {
    let (creation, runtime) = match std::env::var_os(artifact_var) {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            let json = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{artifact_var}={}: {e}", path.display()));
            let artifact: alloy::json_abi::ContractObject = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("{artifact_var}={}: {e}", path.display()));
            (
                artifact
                    .bytecode
                    .unwrap_or_else(|| panic!("{artifact_var}={}: no bytecode", path.display())),
                artifact.deployed_bytecode.unwrap_or_else(|| {
                    panic!("{artifact_var}={}: no deployedBytecode", path.display())
                }),
            )
        }
        None => (creation.clone(), runtime.clone()),
    };
    match std::env::var("RAIN_MATH_FLOAT_DEPLOY_MODE") {
        Ok(mode) if mode == "create" => create(db, address, creation),
        Ok(mode) if mode == "runtime" => put(db, address, runtime),
        Err(std::env::VarError::NotPresent) => put(db, address, runtime),
        other => panic!("RAIN_MATH_FLOAT_DEPLOY_MODE must be `runtime` or `create`: {other:?}"),
    }
}

/// Writes `runtime` as the code at `address`.
fn put(db: &mut InMemoryDB, address: Address, runtime: Bytes) {
    db.insert_account_info(
        address,
        AccountInfo::default().with_code(Bytecode::new_legacy(runtime)),
    );
}

/// Runs `creation` as `address` so its constructor executes, keeps whatever
/// state that produced, and writes the runtime code it returned at `address`.
#[cfg(feature = "test-harness")]
fn create(db: &mut InMemoryDB, address: Address, creation: Bytes) {
    use revm::DatabaseCommit;

    put(db, address, creation);
    let mut evm = Context::mainnet().with_db(db.clone()).build_mainnet();
    let result_and_state = evm
        .system_call(address, Bytes::new())
        .expect("creation code ran");
    let runtime = match result_and_state.result {
        ExecutionResult::Success {
            output: Output::Call(output),
            ..
        } => output,
        other => panic!("creation code at {address} did not return runtime code: {other:?}"),
    };
    db.commit(result_and_state.state);
    put(db, address, runtime);
}

pub(crate) fn execute_call<F, T>(calldata: Bytes, process_output: F) -> Result<T, FloatError>
where
    F: FnOnce(Bytes) -> Result<T, FloatError>,
{
    execute_call_at_address(FLOAT_ADDRESS, calldata, process_output)
}

#[cfg(any(test, feature = "test-harness"))]
pub(crate) fn execute_test_call<F, T>(calldata: Bytes, process_output: F) -> Result<T, FloatError>
where
    F: FnOnce(Bytes) -> Result<T, FloatError>,
{
    execute_call_at_address(TEST_FLOAT_ADDRESS, calldata, process_output)
}

fn execute_call_at_address<F, T>(
    address: Address,
    calldata: Bytes,
    process_output: F,
) -> Result<T, FloatError>
where
    F: FnOnce(Bytes) -> Result<T, FloatError>,
{
    let result = LOCAL_EVM.try_with(|evm| {
        let evm = &mut *evm.borrow_mut();
        let result_and_state = evm.system_call(address, calldata)?;

        Ok::<_, FloatError>(result_and_state.result)
    })??;

    match result {
        ExecutionResult::Success {
            reason: SuccessReason::Return,
            output: Output::Call(output),
            ..
        } => process_output(output),
        ExecutionResult::Success { reason, output, .. } => {
            Err(FloatError::UnexpectedSuccess(reason, output))
        }
        ExecutionResult::Revert { output, .. } => {
            if let Ok(error) = DecimalFloat::DecimalFloatErrors::abi_decode(output.as_ref()) {
                return Err(FloatError::DecimalFloat(Box::new(error)));
            }

            Err(FloatError::Revert(output))
        }
        ExecutionResult::Halt { reason, .. } => Err(FloatError::Halt(reason)),
    }
}
