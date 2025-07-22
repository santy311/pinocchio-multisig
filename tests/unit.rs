use litesvm::LiteSVM;
use pinocchio_multisig::{
    instruction::{
        add_member::AddMemberData, init_multisig::InitMultisigData,
        modify_config::ModifyConfigData, remove_member::RemoveMemberData,
    },
    state::{DataLen, Member, Multisig},
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    sysvar::rent,
    transaction::VersionedTransaction,
};
use std::str::FromStr;

fn setup_svm_and_program() -> (LiteSVM, Keypair, Pubkey, Pubkey, u8) {
    let mut svm = LiteSVM::new();
    let fee_payer = Keypair::new();

    svm.airdrop(&fee_payer.pubkey(), 100000000).unwrap();

    let program_id = Pubkey::from_str("7ut7NJGp4uGXavSNHxgdVDRyiEkVgk95uWMEcuNaDW7c").unwrap();
    svm.add_program_from_file(program_id, "./target/deploy/pinocchio_multisig.so")
        .unwrap();
    let (state_pda, bump) =
        Pubkey::find_program_address(&[b"multisig", fee_payer.pubkey().as_ref()], &program_id);
    (svm, fee_payer, program_id, state_pda, bump)
}

fn modify_config_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    bump: u8,
) -> Instruction {
    let binding = ModifyConfigData { threshold: 2, bump };
    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![2];
    ix_data_with_discriminator.extend_from_slice(&ix_data);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(rent::id(), false),
        ],
        data: ix_data_with_discriminator.try_into().unwrap(),
    }
}

fn create_initialize_multisig_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    bump: u8,
) -> Instruction {
    let binding = InitMultisigData {
        threshold: 1,
        num_members: 3,
        bump,
    };
    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![0];
    ix_data_with_discriminator.extend_from_slice(&ix_data);

    let members = [
        Member::new(
            Pubkey::from_str("3hPmQsxMb4buU1PozSqMS7wni14JoP5kmPA9UTpJnerb")
                .unwrap()
                .to_bytes(),
            1,
        ),
        Member::new(
            Pubkey::from_str("CzYQ2kFnBxsNEt9Zy34vQ3n5fSDhvA4o4XaTnq1rLvyr")
                .unwrap()
                .to_bytes(),
            0,
        ),
        Member::new(
            Pubkey::from_str("HYb56EtuoqU2dm4Mvr1ft9uj9kBdMYcvkLLLZ2DiB7wK")
                .unwrap()
                .to_bytes(),
            0,
        ),
    ];

    ix_data_with_discriminator.extend_from_slice(&members[0].to_bytes());
    ix_data_with_discriminator.extend_from_slice(&members[1].to_bytes());
    ix_data_with_discriminator.extend_from_slice(&members[2].to_bytes());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ix_data_with_discriminator.try_into().unwrap(),
    }
}

fn create_add_member_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    bump: u8,
) -> Instruction {
    let binding = AddMemberData {
        num_members: 2,
        bump,
    };
    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![1];
    ix_data_with_discriminator.extend_from_slice(&ix_data);

    let members = [
        Member::new(
            Pubkey::from_str("Aa4Uhe6hpCWZhf9EczEq6oyJjticWWuQ8aa29Z3U9VoS")
                .unwrap()
                .to_bytes(),
            1,
        ),
        Member::new(
            Pubkey::from_str("BaqtBjLTDHQvARvCDjGS6hKG45E2eUB7GwnGwrm5EZRP")
                .unwrap()
                .to_bytes(),
            0,
        ),
    ];

    ix_data_with_discriminator.extend_from_slice(&members[0].to_bytes());
    ix_data_with_discriminator.extend_from_slice(&members[1].to_bytes());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ix_data_with_discriminator.try_into().unwrap(),
    }
}

fn remove_member_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    bump: u8,
) -> Instruction {
    let binding = RemoveMemberData { member_id: 1, bump };
    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![3];
    ix_data_with_discriminator.extend_from_slice(&ix_data);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ix_data_with_discriminator.try_into().unwrap(),
    }
}

#[test]
fn test_initialize_and_add_mapping() {
    let (mut svm, fee_payer, program_id, state_pda, bump) = setup_svm_and_program();
    // Success: Initialize
    println!("=========== Initializing multisig ===========");
    let ix = create_initialize_multisig_ix(program_id, &fee_payer, state_pda, bump);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx);
    println!("result: {:?}", result);
    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    println!("multisig: {:?}", multisig);
    assert_eq!(multisig.creator, fee_payer.pubkey().to_bytes());
    assert_eq!(multisig.threshold, 1);
    assert_eq!(multisig.num_members, 3);
    assert_eq!(multisig.bump, bump);

    let members_data = &multisig_data[Multisig::LEN..];
    for member_data in members_data.chunks(Member::LEN) {
        let member = Member::from_bytes(member_data).unwrap();
        println!(
            "Member id {}: {:?} with admin role {}",
            member.member_id,
            Pubkey::new_from_array(member.pubkey),
            member.role == 1
        );
    }

    // Success: Add member
    println!("=========== Adding member ===========");

    let ix = create_add_member_ix(program_id, &fee_payer, state_pda, bump);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    let multisig_data = svm.get_account(&state_pda).unwrap().data;

    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.num_members, 5);

    let members_data = &multisig_data[Multisig::LEN..];
    for member_data in members_data.chunks(Member::LEN) {
        let member = Member::from_bytes(member_data).unwrap();
        println!(
            "Member id {}: {:?} with admin role {}",
            member.member_id,
            Pubkey::new_from_array(member.pubkey),
            member.role == 1
        );
    }

    // Success: Remove member
    println!("=========== Removing member ===========");
    let ix = remove_member_ix(program_id, &fee_payer, state_pda, bump);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let members_data = &multisig_data[Multisig::LEN..];
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.num_members, 4);

    for member_data in members_data.chunks(Member::LEN) {
        let member = Member::from_bytes(member_data).unwrap();
        println!(
            "Member id {}: {:?} with admin role {}",
            member.member_id,
            Pubkey::new_from_array(member.pubkey),
            member.role == 1
        );
    }

    // Failure: Remove member
    println!("=========== Removing non-existent member ===========");
    let ix = remove_member_ix(program_id, &fee_payer, state_pda, bump);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_err());

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let members_data = &multisig_data[Multisig::LEN..];
    for member_data in members_data.chunks(Member::LEN) {
        let member = Member::from_bytes(member_data).unwrap();
        println!(
            "Member id {}: {:?} with admin role {}",
            member.member_id,
            Pubkey::new_from_array(member.pubkey),
            member.role == 1
        );
    }

    println!("=========== Modifying config ===========");
    let ix = modify_config_ix(program_id, &fee_payer, state_pda, bump);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.threshold, 2);
}
