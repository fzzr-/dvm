use byteorder::{LittleEndian, ByteOrder};

use libra::libra_types;
use libra_types::account_address::AccountAddress;

use dvm_test_kit::*;
use dvm::vm::native::{Reg, dbg};
use dvm::vm::native::oracle::PriceOracle;
use dvm::compiled_protos::vm_grpc::{VmTypeTag, VmArgs};
use lang::{banch32::bech32_into_libra, compiler::str_xxhash};
use data_source::MockDataSource;

#[test]
fn test_create_account() {
    let test_kit = TestKit::new();
    let create_account = "\
        import 0x0.Account;
        main(fresh_address: address) {
          Account.create_account(move(fresh_address));
          return;
        }
    ";
    let bech32_sender_address = "wallets196udj7s83uaw2u4safcrvgyqc0sc3flxuherp6";
    let account_address = account(bech32_sender_address);

    let args = vec![VmArgs {
        r#type: VmTypeTag::Address as i32,
        value: bech32_sender_address.to_string(),
    }];
    let res = test_kit.execute_script(create_account, meta(&account_address), args);
    test_kit.assert_success(&res);
    assert!(!res.executions[0].write_set.is_empty());
    test_kit.merge_result(&res);
}

#[test]
fn test_native_func() {
    dbg::PrintByteArray {}.reg_function();

    let test_kit = TestKit::new();

    test_kit.add_std_module(include_str!("./resources/dbg.mvir"));

    let script = "\
        import 0x0.Dbg;
        main(data: bytearray) {
          Dbg.print_byte_array(move(data));
          return;
        }
    ";
    let args = vec![VmArgs {
        r#type: VmTypeTag::ByteArray as i32,
        value: "b\"C001C00D\"".to_string(),
    }];

    let account_address = account("wallets196udj7s83uaw2u4safcrvgyqc0sc3flxuherp6");
    let res = test_kit.execute_script(script, meta(&account_address), args);
    test_kit.assert_success(&res);
}

#[test]
fn test_oracle() {
    let ds = MockDataSource::new();
    let mut price = vec![0; 8];
    LittleEndian::write_u64(&mut price, 13);

    ds.insert(PriceOracle::make_path(str_xxhash("usdbtc")).unwrap(), price);
    PriceOracle::new(Box::new(ds.clone())).reg_function();
    let dump = dbg::DumpU64::new();
    dump.clone().reg_function();

    let test_kit = TestKit::new();
    test_kit.add_std_module(include_str!("./resources/dbg.mvir"));

    let script = "
        import 0x0.Dbg;
        import 0x0.Oracle;

        main() {
          Dbg.dump_u64(Oracle.get_price(#\"USDBTC\"));
          return;
        }
    ";

    let account_address = account("wallets196udj7s83uaw2u4safcrvgyqc0sc3flxuherp6");

    let res = test_kit.execute_script(script, meta(&account_address), vec![]);
    test_kit.assert_success(&res);
    assert_eq!(dump.get(), Some(13));

    let mut price = vec![0; 8];
    LittleEndian::write_u64(&mut price, 1313);
    ds.insert(PriceOracle::make_path(str_xxhash("usdxrp")).unwrap(), price);

    let script = "
        use 0x0::Dbg;
        use 0x0::Oracle;

        fun main() {
          Dbg::dump_u64(Oracle::get_price(#\"USDxrp\"));
        }
    ";
    let res = test_kit.execute_script(script, meta(&account_address), vec![]);
    test_kit.assert_success(&res);
    assert_eq!(dump.get(), Some(1313));
}

#[test]
fn test_account_event() {
    let test_kit = TestKit::new();
    let bech32_sender_address = "wallet14ng6lzsvyy26sxmujmjthvrjde8x6gkk2gzeft";
    let account_address = AccountAddress::from_hex_literal(&format!(
        "0x{}",
        bech32_into_libra(bech32_sender_address).unwrap()
    ))
    .unwrap();

    let script = "\
        import 0x0.Account;
        import 0x0.Coins;

        main(recipient: address, amount: u128, denom: bytearray) {
            let coin: Coins.Coin;
            coin = Account.withdraw_from_sender(move(amount), move(denom));

            Account.deposit(move(recipient), move(coin));
            return;
        }
    ";

    let args = vec![
        VmArgs {
            r#type: VmTypeTag::Address as i32,
            value: "wallets1y6pk6wjmvjm7hn79mpam5wcnsxy2z2dqmvj80p".to_string(),
        },
        VmArgs {
            r#type: VmTypeTag::U128 as i32,
            value: "10".to_string(),
        },
        VmArgs {
            r#type: VmTypeTag::ByteArray as i32,
            value: "b\"646669\"".to_string(),
        },
    ];
    let res = test_kit.execute_script(script, meta(&account_address), args);
    test_kit.assert_success(&res);
}

fn account(bech32: &str) -> AccountAddress {
    AccountAddress::from_hex_literal(&format!("0x{}", bech32_into_libra(bech32).unwrap())).unwrap()
}
