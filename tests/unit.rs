use litesvm::LiteSVM;
use pinocchio_multisig::{
    instruction::{
        add_member::AddMemberData, init_multisig::InitMultisigData,
        modify_config::ModifyConfigData, remove_member::RemoveMemberData, CreateProposalData,
        VoteData,
    },
    state::{DataLen, Member, Multisig, Proposal},
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

fn setup_svm_and_program() -> (LiteSVM, Keypair, Keypair, Pubkey) {
    let mut svm = LiteSVM::new();
    let fee_payer = Keypair::new();

    svm.airdrop(&fee_payer.pubkey(), 100000000).unwrap();

    let program_id = Pubkey::from_str("7ut7NJGp4uGXavSNHxgdVDRyiEkVgk95uWMEcuNaDW7c").unwrap();
    svm.add_program_from_file(program_id, "./target/deploy/pinocchio_multisig.so")
        .unwrap();

    let second_admin = Keypair::new();
    svm.airdrop(&second_admin.pubkey(), 100000000).unwrap();

    (svm, fee_payer, second_admin, program_id)
}

fn setup_all_pdas(fee_payer: &Keypair, program_id: Pubkey) -> (Pubkey, Pubkey, u8, u8) {
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&[b"multisig", fee_payer.pubkey().as_ref()], &program_id);

    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", state_pda.as_ref()], &program_id);

    (state_pda, vault_pda, vault_bump, state_bump)
}

fn modify_config_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    state_bump: u8,
) -> Instruction {
    let binding = ModifyConfigData {
        threshold: 2,
        max_expiry_duration: 1 * 24 * 60 * 60,
        veto_threshold: 2,
        bump: state_bump,
    };
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
    vault_pda: Pubkey,
    vault_bump: u8,
    state_bump: u8,
) -> Instruction {
    let binding = InitMultisigData {
        threshold: 1,
        num_members: 3,
        max_expiry_duration: 3 * 24 * 60 * 60,
        veto_threshold: 1,
        seed: 1,
        bump: state_bump,
        vault_bump,
    };
    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![0];
    ix_data_with_discriminator.extend_from_slice(&ix_data);

    let members = [
        Member::new(fee_payer.pubkey().to_bytes(), 1),
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
            AccountMeta::new(vault_pda, false),
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
    state_bump: u8,
) -> Instruction {
    let binding = AddMemberData {
        num_members: 2,
        bump: state_bump,
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
    state_bump: u8,
) -> Instruction {
    let binding = RemoveMemberData {
        member_id: 1,
        bump: state_bump,
    };
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

fn create_create_proposal_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    proposal_acc: Pubkey,
    multisig_bump: u8,
    proposal_bump: u8,
) -> Instruction {
    let binding = CreateProposalData {
        multisig_bump,
        proposal_bump,
    };
    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![4];
    ix_data_with_discriminator.extend_from_slice(&ix_data);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new(proposal_acc, false),
            AccountMeta::new_readonly(rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ix_data_with_discriminator.try_into().unwrap(),
    }
}

fn create_vote_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    proposal_acc: Pubkey,
    state_bump: u8,
    proposal_id: u64,
    proposal_bump: u8,
) -> Instruction {
    let binding = VoteData {
        proposal_id,
        vote: 1,
        multisig_bump: state_bump,
        proposal_bump,
    };
    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![5];
    ix_data_with_discriminator.extend_from_slice(&ix_data);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new(proposal_acc, false),
            AccountMeta::new_readonly(rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ix_data_with_discriminator.try_into().unwrap(),
    }
}

#[test]
fn test_initialize_and_add_mapping() {
    let (mut svm, fee_payer, second_admin, program_id) = setup_svm_and_program();

    let (state_pda, vault_pda, vault_bump, state_bump) = setup_all_pdas(&fee_payer, program_id);

    // Success: Initialize
    println!("=========== Initializing multisig ===========");
    let ix = create_initialize_multisig_ix(
        program_id, &fee_payer, state_pda, vault_pda, vault_bump, state_bump,
    );
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.creator, fee_payer.pubkey().to_bytes());
    assert_eq!(multisig.threshold, 1);
    assert_eq!(multisig.num_members, 3);
    assert_eq!(multisig.bump, state_bump);
    assert_eq!(multisig.max_expiry_duration, 3 * 24 * 60 * 60);
    assert_eq!(multisig.veto_threshold, 1);
    assert_eq!(multisig.seed, 1);

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

    let ix = create_add_member_ix(program_id, &fee_payer, state_pda, state_bump);
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
    let ix = remove_member_ix(program_id, &fee_payer, state_pda, state_bump);
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
    let ix = remove_member_ix(program_id, &fee_payer, state_pda, state_bump);
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
    let ix = modify_config_ix(program_id, &fee_payer, state_pda, state_bump);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.threshold, 2);
    assert_eq!(multisig.max_expiry_duration, 1 * 24 * 60 * 60);
    assert_eq!(multisig.veto_threshold, 2);

    // Success: Create proposal
    println!("=========== Creating proposal ===========");
    let (proposal_acc, proposal_bump) = Pubkey::find_program_address(
        &[
            b"proposal",
            state_pda.as_ref(),
            &(multisig.proposal_counter).to_le_bytes(),
        ],
        &program_id,
    );

    println!(
        "Derived proposal PDA: {:?}, bump: {}",
        proposal_acc, proposal_bump
    );
    println!("bump: {:?}", proposal_bump.to_le_bytes());
    println!("pda: {:?}", proposal_acc.to_bytes());
    println!("id_seed: {:?}", multisig.proposal_counter.to_le_bytes());
    println!("owner: {:?}", state_pda.to_bytes());

    let ix = create_create_proposal_ix(
        program_id,
        &fee_payer,
        state_pda,
        proposal_acc,
        state_bump,
        proposal_bump,
    );
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.proposal_counter, 1);

    let proposal_data = svm.get_account(&proposal_acc).unwrap().data;
    let proposal_bytes = &proposal_data[..Proposal::LEN];
    let proposal = Proposal::from_bytes(proposal_bytes).unwrap();
    println!("proposal: {:?}", proposal);
    assert_eq!(proposal.multisig, state_pda.to_bytes());
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.creator, 0);
    assert_eq!(proposal.status, 0);

    svm.expire_blockhash();

    // Success: Vote
    println!("=========== Voting ===========");
    let ix = create_vote_ix(
        program_id,
        &fee_payer,
        state_pda,
        proposal_acc,
        state_bump,
        proposal.id,
        proposal_bump,
    );
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    let proposal_data = svm.get_account(&proposal_acc).unwrap().data;
    let proposal_bytes = &proposal_data[..Proposal::LEN];
    let proposal = Proposal::from_bytes(proposal_bytes).unwrap();
    println!("proposal: {:?}", proposal);
    assert_eq!(proposal.status, 0);
    let voter_list_data = &proposal_data[Proposal::LEN..];
    let voter_list = voter_list_data.chunks(1).map(|v| v[0]).collect::<Vec<u8>>();
    assert_eq!(voter_list, vec![0]);
}
