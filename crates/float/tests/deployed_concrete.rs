use alloy::hex;
use alloy::json_abi::JsonAbi;
use alloy::primitives::{keccak256, Address, Bytes, B256};
use alloy::sol_types::SolInterface;
use rain_math_float::DecimalFloat;
use revm::context::result::ExecutionResult;
use revm::database::InMemoryDB;
use revm::state::{AccountInfo, Bytecode};
use revm::{Context, MainBuilder, MainContext, SystemCallEvm};

const CRATE_CALLS: [(&str, &str); 29] = [
    ("maxPositiveValue()", "bytes32"),
    ("minPositiveValue()", "bytes32"),
    ("maxNegativeValue()", "bytes32"),
    ("minNegativeValue()", "bytes32"),
    ("zero()", "bytes32"),
    ("parse(string)", "bytes4,bytes32"),
    ("format(bytes32,bool)", "string"),
    ("add(bytes32,bytes32)", "bytes32"),
    ("sub(bytes32,bytes32)", "bytes32"),
    ("mul(bytes32,bytes32)", "bytes32"),
    ("div(bytes32,bytes32)", "bytes32"),
    ("minus(bytes32)", "bytes32"),
    ("abs(bytes32)", "bytes32"),
    ("inv(bytes32)", "bytes32"),
    ("eq(bytes32,bytes32)", "bool"),
    ("lt(bytes32,bytes32)", "bool"),
    ("gt(bytes32,bytes32)", "bool"),
    ("lte(bytes32,bytes32)", "bool"),
    ("gte(bytes32,bytes32)", "bool"),
    ("integer(bytes32)", "bytes32"),
    ("frac(bytes32)", "bytes32"),
    ("floor(bytes32)", "bytes32"),
    ("min(bytes32,bytes32)", "bytes32"),
    ("max(bytes32,bytes32)", "bytes32"),
    ("isZero(bytes32)", "bool"),
    ("fromFixedDecimalLossless(uint256,uint8)", "bytes32"),
    ("toFixedDecimalLossless(bytes32,uint8)", "uint256"),
    ("fromFixedDecimalLossy(uint256,uint8)", "bytes32,bool"),
    ("toFixedDecimalLossy(bytes32,uint8)", "uint256,bool"),
];

const REMOVED: [&str; 2] = ["format(bytes32,bytes32,bytes32)", "format(bytes32)"];

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn committed_abi() -> JsonAbi {
    let json = std::fs::read_to_string(root().join("crates/float/abi/DecimalFloat.json")).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&json).unwrap();
    serde_json::from_value(artifact["abi"].clone()).unwrap()
}

fn pinned(name: &str) -> Bytes {
    let source = std::fs::read_to_string(root().join("src/generated/candidate/DecimalFloat.sol")).unwrap();
    let start = source.find(&format!("{name} =")).unwrap_or_else(|| panic!("no {name}"));
    let rest = &source[start..];
    let open = rest.find("hex\"").unwrap() + 4;
    let close = open + rest[open..].find('"').unwrap();
    Bytes::from(hex::decode(&rest[open..close]).unwrap())
}

fn selector(signature: &str) -> [u8; 4] {
    keccak256(signature)[..4].try_into().unwrap()
}

/// Runs `calldata` against the crate's runtime bytecode on a fresh EVM.
fn run(calldata: Vec<u8>) -> ExecutionResult {
    let address = Address::repeat_byte(0x11);
    let mut db = InMemoryDB::default();
    db.insert_account_info(
        address,
        AccountInfo::default().with_code(Bytecode::new_legacy(DecimalFloat::DEPLOYED_BYTECODE.clone())),
    );
    let mut evm = Context::mainnet().with_db(db).build_mainnet();
    evm.system_call(address, Bytes::from(calldata)).unwrap().result
}

#[test]
fn crate_bytecode_is_the_pinned_deploy() {
    assert_eq!(DecimalFloat::DEPLOYED_BYTECODE, pinned("RUNTIME_CODE"));
    assert_eq!(DecimalFloat::BYTECODE, pinned("CREATION_CODE"));
    let hash: B256 = {
        let source = std::fs::read_to_string(root().join("src/generated/candidate/DecimalFloat.sol")).unwrap();
        let at = source.find("BYTECODE_HASH = bytes32(0x").unwrap() + "BYTECODE_HASH = bytes32(".len();
        source[at..at + 66].parse().unwrap()
    };
    assert_eq!(keccak256(&DecimalFloat::DEPLOYED_BYTECODE), hash);
}

#[test]
fn committed_abi_has_every_call_the_crate_makes() {
    let abi = committed_abi();
    for (signature, outputs) in CRATE_CALLS {
        let name = &signature[..signature.find('(').unwrap()];
        let function = abi
            .function(name)
            .and_then(|fs| fs.iter().find(|f| f.signature() == signature))
            .unwrap_or_else(|| panic!("{signature} is not in the committed ABI"));
        let got: Vec<String> = function.outputs.iter().map(|o| o.ty.clone()).collect();
        assert_eq!(got.join(","), outputs, "{signature} outputs");
    }
}

#[test]
fn removed_formats_are_absent() {
    let abi = committed_abi();
    let formats: Vec<String> = abi.function("format").unwrap().iter().map(|f| f.signature()).collect();
    assert_eq!(formats, vec!["format(bytes32,bool)".to_string()]);
    for signature in REMOVED {
        match run(selector(signature).into_iter().chain([0u8; 96]).collect()) {
            ExecutionResult::Revert { output, .. } => assert!(output.is_empty(), "{signature}: {output}"),
            other => panic!("{signature} dispatched: {other:?}"),
        }
    }
}

#[test]
fn concrete_dispatches_every_call_the_crate_makes() {
    for (signature, _) in CRATE_CALLS {
        let mut calldata = selector(signature).to_vec();
        if signature == "parse(string)" {
            calldata.extend([0u8; 31].iter().chain(&[0x20]).chain(&[0u8; 32]));
        } else {
            calldata.extend([0u8; 64]);
        }
        match run(calldata) {
            ExecutionResult::Success { output, .. } => assert!(!output.data().is_empty(), "{signature}"),
            ExecutionResult::Revert { output, .. } => assert!(
                DecimalFloat::DecimalFloatErrors::abi_decode(&output).is_ok(),
                "{signature} reverted without a DecimalFloat error: {output}"
            ),
            other => panic!("{signature} halted: {other:?}"),
        }
    }
}
