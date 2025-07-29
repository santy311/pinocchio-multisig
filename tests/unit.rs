use litesvm::LiteSVM;
use pinocchio_multisig::{
    instruction::{
        add_member::AddMemberData, execute::ExecuteData, init_multisig::InitMultisigData,
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
use std::{collections::HashMap, str::FromStr};

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

fn setup_all_pdas(program_id: Pubkey, multisig_id: u64) -> (Pubkey, Pubkey, u8, u8) {
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&[b"multisig", &multisig_id.to_le_bytes()], &program_id);

    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", state_pda.as_ref()], &program_id);

    (state_pda, vault_pda, vault_bump, state_bump)
}

fn modify_config_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    state_bump: u8,
    multisig_id: u64,
) -> Instruction {
    let binding = ModifyConfigData {
        threshold: 2,
        max_expiry_duration: 1 * 24 * 60 * 60,
        veto_threshold: 2,
        bump: state_bump,
        multisig_id,
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
    multisig_id: u64,
) -> Instruction {
    let binding = InitMultisigData {
        threshold: 1,
        num_members: 3,
        max_expiry_duration: 3 * 24 * 60 * 60,
        veto_threshold: 1,
        multisig_id,
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
    multisig_id: u64,
) -> Instruction {
    let binding = AddMemberData {
        num_members: 2,
        bump: state_bump,
        multisig_id,
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
    multisig_id: u64,
) -> Instruction {
    let binding = RemoveMemberData {
        member_id: 1,
        bump: state_bump,
        multisig_id,
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
    multisig_id: u64,
) -> Instruction {
    let binding = CreateProposalData {
        multisig_bump,
        proposal_bump,
        multisig_id,
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
    vote: u8,
    proposal_bump: u8,
    multisig_id: u64,
) -> Instruction {
    let binding = VoteData {
        proposal_id,
        vote,
        multisig_bump: state_bump,
        proposal_bump,
        multisig_id,
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

fn create_execute_ix(
    program_id: Pubkey,
    fee_payer: &Keypair,
    state_pda: Pubkey,
    proposal_acc: Pubkey,
    state_bump: u8,
    proposal_bump: u8,
    multisig_id: u64,
) -> Instruction {
    let binding = ExecuteData {
        proposal_id: 0,
        multisig_id,
        bump: state_bump,
        vault_bump: 0,
        proposal_bump,
    };

    let ix_data = binding.to_bytes();
    let mut ix_data_with_discriminator = vec![6];
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

fn print_stats(
    svm: &LiteSVM,
    tx_result: &litesvm::types::TransactionMetadata,
    state_pda: &Pubkey,
    proposal_acc: &Option<Pubkey>,
) {
    println!("=========== Stats ===========");
    println!("cu units consumed: {:?}", tx_result.compute_units_consumed);
    println!(
        "multisig size: {:?}",
        svm.get_account(&state_pda).unwrap().data.len()
    );
    if let Some(proposal_acc) = proposal_acc {
        println!(
            "proposal size: {:?}",
            svm.get_account(&proposal_acc).unwrap().data.len()
        );
    }
}

#[test]
fn test_initialize_and_add_mapping() {
    let (mut svm, fee_payer, second_admin, program_id) = setup_svm_and_program();

    let multisig_id = 1;

    let (state_pda, vault_pda, vault_bump, state_bump) = setup_all_pdas(program_id, multisig_id);

    // Success: Initialize
    println!("=========== Initializing multisig ===========");
    let ix = create_initialize_multisig_ix(
        program_id,
        &fee_payer,
        state_pda,
        vault_pda,
        vault_bump,
        state_bump,
        multisig_id,
    );
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Initialize multisig");
    print_stats(&svm, &result, &state_pda, &None);

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.creator, fee_payer.pubkey().to_bytes());
    assert_eq!(multisig.threshold, 1);
    assert_eq!(multisig.num_members, 3);
    assert_eq!(multisig.bump, state_bump);
    assert_eq!(multisig.max_expiry_duration, 3 * 24 * 60 * 60);
    assert_eq!(multisig.veto_threshold, 1);
    assert_eq!(multisig.multisig_id, multisig_id);

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

    let ix = create_add_member_ix(program_id, &fee_payer, state_pda, state_bump, multisig_id);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Add member");
    print_stats(&svm, &result, &state_pda, &None);

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
    let ix = remove_member_ix(program_id, &fee_payer, state_pda, state_bump, multisig_id);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Remove member");
    print_stats(&svm, &result, &state_pda, &None);

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
    let ix = remove_member_ix(program_id, &fee_payer, state_pda, state_bump, multisig_id);
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

    // Success: Add member
    println!("=========== Adding member ===========");

    svm.expire_blockhash();

    let ix = create_add_member_ix(program_id, &fee_payer, state_pda, state_bump, multisig_id);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Add member");
    print_stats(&svm, &result, &state_pda, &None);

    let multisig_data = svm.get_account(&state_pda).unwrap().data;

    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.num_members, 6);

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
    let ix = modify_config_ix(program_id, &fee_payer, state_pda, state_bump, multisig_id);
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Modify config");
    print_stats(&svm, &result, &state_pda, &None);

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.threshold, 2);
    assert_eq!(multisig.max_expiry_duration, 1 * 24 * 60 * 60);
    assert_eq!(multisig.veto_threshold, 2);
    println!("multisig: {:?}", multisig);

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

    let ix = create_create_proposal_ix(
        program_id,
        &fee_payer,
        state_pda,
        proposal_acc,
        state_bump,
        proposal_bump,
        multisig_id,
    );
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Create proposal");
    print_stats(&svm, &result, &state_pda, &Some(proposal_acc));

    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig_bytes = &multisig_data[..Multisig::LEN];
    let multisig = Multisig::from_bytes(multisig_bytes).unwrap();
    assert_eq!(multisig.proposal_counter, 1);

    let proposal_data = svm.get_account(&proposal_acc).unwrap().data;
    let proposal_bytes = &proposal_data[..Proposal::LEN];
    let proposal = Proposal::from_bytes(proposal_bytes).unwrap();
    assert_eq!(proposal.multisig, state_pda.to_bytes());
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.creator, 0);
    assert_eq!(proposal.status, 0);

    println!("proposal: {:?}", proposal);

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
        1,
        proposal_bump,
        multisig_id,
    );
    let msg =
        v0::Message::try_compile(&fee_payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&fee_payer]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Vote");
    print_stats(&svm, &result, &state_pda, &Some(proposal_acc));

    let proposal_data = svm.get_account(&proposal_acc).unwrap().data;
    let proposal_bytes = &proposal_data[..Proposal::LEN];
    let proposal = Proposal::from_bytes(proposal_bytes).unwrap();
    assert_eq!(proposal.status, 0);
    assert_eq!(proposal.yes_votes, 1);
    assert_eq!(proposal.no_votes, 0);
    assert_eq!(proposal.veto_votes, 0);
    let voter_list_data = &proposal_data[Proposal::LEN..];
    let voter_list = voter_list_data.chunks(1).map(|v| v[0]).collect::<Vec<u8>>();
    assert_eq!(voter_list, vec![0]);
    println!("proposal: {:?}", proposal);

    println!("=========== End of test ===========");
}

#[test]
fn test_five_members_different_votes() {
    use solana_sdk::signer::Signer;
    let (mut svm, fee_payer, _second_admin, program_id) = setup_svm_and_program();
    let multisig_id = 2;

    let (state_pda, vault_pda, vault_bump, state_bump) = setup_all_pdas(program_id, multisig_id);

    // Create 5 unique member keypairs
    let member1 = fee_payer.insecure_clone(); // admin
    let member2 = Keypair::new();
    let member3 = Keypair::new();
    let member4 = Keypair::new();
    let member5 = Keypair::new();
    let members = [&member1, &member2, &member3, &member4, &member5];
    for m in &members[1..] {
        svm.airdrop(&m.pubkey(), 100000000).unwrap();
    }

    // Initialize multisig with 5 members
    let init_ix = {
        let binding = InitMultisigData {
            threshold: 2,
            num_members: 5,
            max_expiry_duration: 3 * 24 * 60 * 60,
            veto_threshold: 2,
            multisig_id,
            bump: state_bump,
            vault_bump,
        };
        let mut ix_data_with_discriminator = vec![0];
        ix_data_with_discriminator.extend_from_slice(&binding.to_bytes());
        for (i, m) in members.iter().enumerate() {
            let role = if i == 0 { 1 } else { 0 };
            let mut member = Member::new(m.pubkey().to_bytes(), role);
            member.member_id = i as u8;
            ix_data_with_discriminator.extend_from_slice(&member.to_bytes());
        }
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(member1.pubkey(), true),
                AccountMeta::new(state_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: ix_data_with_discriminator.try_into().unwrap(),
        }
    };
    let msg = v0::Message::try_compile(&member1.pubkey(), &[init_ix], &[], svm.latest_blockhash())
        .unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&member1]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Initialize multisig with 5 members");
    print_stats(&svm, &result, &state_pda, &None);

    // Create proposal
    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig = Multisig::from_bytes(&multisig_data[..Multisig::LEN]).unwrap();
    let (proposal_acc, proposal_bump) = Pubkey::find_program_address(
        &[
            b"proposal",
            state_pda.as_ref(),
            &(multisig.proposal_counter).to_le_bytes(),
        ],
        &program_id,
    );
    let create_proposal_ix = create_create_proposal_ix(
        program_id,
        &member1,
        state_pda,
        proposal_acc,
        state_bump,
        proposal_bump,
        multisig_id,
    );
    let msg = v0::Message::try_compile(
        &member1.pubkey(),
        &[create_proposal_ix],
        &[],
        svm.latest_blockhash(),
    )
    .unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&member1]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Create proposal");
    print_stats(&svm, &result, &state_pda, &Some(proposal_acc));

    // Each member votes: [yes, no, veto, yes, no]
    let votes = [1, 0, 2, 1, 0];
    for (i, (member, &vote)) in members.iter().zip(votes.iter()).enumerate() {
        let ix = create_vote_ix(
            program_id,
            member,
            state_pda,
            proposal_acc,
            state_bump,
            0, // proposal id
            vote,
            proposal_bump,
            multisig_id,
        );
        let msg =
            v0::Message::try_compile(&member.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
        let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[member]).unwrap();
        let result = svm.send_transaction(tx).unwrap();
        println!("Action: Vote (member {})", i);
        print_stats(&svm, &result, &state_pda, &Some(proposal_acc));
    }

    // Check proposal vote counts
    let proposal_data = svm.get_account(&proposal_acc).unwrap().data;
    let proposal = Proposal::from_bytes(&proposal_data[..Proposal::LEN]).unwrap();
    assert_eq!(proposal.yes_votes, 2);
    assert_eq!(proposal.no_votes, 2);
    assert_eq!(proposal.veto_votes, 1);
    // Check voter list order: yes, yes, no, no, veto (by member_id)
    let voter_list_data = &proposal_data[Proposal::LEN..];
    let voter_list = voter_list_data.chunks(1).map(|v| v[0]).collect::<Vec<u8>>();
    // The order is: [yes_votes...][no_votes...][veto_votes...]
    // So should be [0,3,1,4,2] (member_id for yes, yes, no, no, veto)
    assert_eq!(voter_list.len(), 5);
    assert_eq!(voter_list[0], 0); // member1 (yes)
    assert_eq!(voter_list[1], 3); // member4 (yes)
    assert_eq!(voter_list[2], 1); // member2 (no)
    assert_eq!(voter_list[3], 4); // member5 (no)
    assert_eq!(voter_list[4], 2); // member3 (veto)

    // Print all the voting info
    println!("=========== Voting info ===========");
    println!("proposal: {:?}", proposal);
    println!("voter_list: {:?}", voter_list);
    let members_data = &multisig_data[Multisig::LEN..];

    let mut hashmap = HashMap::new();

    for member_data in members_data.chunks(Member::LEN) {
        let member = Member::from_bytes(member_data).unwrap();
        println!(
            "Member id {}: {:?} with admin role {}",
            member.member_id,
            Pubkey::new_from_array(member.pubkey),
            member.role == 1
        );
        hashmap.insert(member.member_id, Pubkey::new_from_array(member.pubkey));
    }

    println!("voter list");
    println!("number of voters: {:?}", voter_list.len());

    println!("nay sayers");
    for i in 0..proposal.no_votes {
        println!("{:?}", hashmap.get(&voter_list[i as usize]).unwrap());
    }
    println!("yay sayers");
    for i in proposal.no_votes..proposal.no_votes + proposal.yes_votes {
        println!("{:?}", hashmap.get(&voter_list[i as usize]).unwrap());
    }

    println!("veto sayers");
    for i in proposal.no_votes + proposal.yes_votes
        ..proposal.no_votes + proposal.yes_votes + proposal.veto_votes
    {
        println!("{:?}", hashmap.get(&voter_list[i as usize]).unwrap());
    }

    // Execute proposal
    let ix = create_execute_ix(
        program_id,
        &member1,
        state_pda,
        proposal_acc,
        state_bump,
        proposal_bump,
        multisig_id,
    );
    let msg =
        v0::Message::try_compile(&member1.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&member1]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Execute proposal");
    print_stats(&svm, &result, &state_pda, &Some(proposal_acc));

    // Check proposal status
    let proposal_data = svm.get_account(&proposal_acc).unwrap().data;
    let proposal = Proposal::from_bytes(&proposal_data[..Proposal::LEN]).unwrap();
    assert_eq!(proposal.status, 1);
}

#[test]
fn test_max_members_and_votes() {
    use solana_sdk::signer::Signer;
    let (mut svm, fee_payer, _second_admin, program_id) = setup_svm_and_program();
    let multisig_id = 3;

    let (state_pda, vault_pda, vault_bump, state_bump) = setup_all_pdas(program_id, multisig_id);

    // Create 250 unique member keypairs
    let mut members = Vec::with_capacity(250);
    members.push(fee_payer.insecure_clone()); // admin
    for _ in 1..250 {
        let kp = Keypair::new();
        svm.airdrop(&kp.pubkey(), 100000000).unwrap();
        members.push(kp);
    }
    let member_refs: Vec<&Keypair> = members.iter().collect();

    // Initialize multisig with 250 members
    let init_ix = {
        let binding = InitMultisigData {
            threshold: 125, // arbitrary threshold
            num_members: 250,
            max_expiry_duration: 3 * 24 * 60 * 60,
            veto_threshold: 10,
            multisig_id,
            bump: state_bump,
            vault_bump,
        };
        let mut ix_data_with_discriminator = vec![0];
        ix_data_with_discriminator.extend_from_slice(&binding.to_bytes());
        for (i, m) in member_refs.iter().enumerate() {
            let role = if i == 0 { 1 } else { 0 };
            let mut member = Member::new(m.pubkey().to_bytes(), role);
            member.member_id = i as u8;
            ix_data_with_discriminator.extend_from_slice(&member.to_bytes());
        }
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(member_refs[0].pubkey(), true),
                AccountMeta::new(state_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: ix_data_with_discriminator.try_into().unwrap(),
        }
    };
    let msg = v0::Message::try_compile(
        &member_refs[0].pubkey(),
        &[init_ix],
        &[],
        svm.latest_blockhash(),
    )
    .unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[member_refs[0]]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Initialize multisig with 250 members");
    print_stats(&svm, &result, &state_pda, &None);

    // Create proposal
    let multisig_data = svm.get_account(&state_pda).unwrap().data;
    let multisig = Multisig::from_bytes(&multisig_data[..Multisig::LEN]).unwrap();
    let (proposal_acc, proposal_bump) = Pubkey::find_program_address(
        &[
            b"proposal",
            state_pda.as_ref(),
            &(multisig.proposal_counter).to_le_bytes(),
        ],
        &program_id,
    );
    let create_proposal_ix = create_create_proposal_ix(
        program_id,
        member_refs[0],
        state_pda,
        proposal_acc,
        state_bump,
        proposal_bump,
        multisig_id,
    );
    let msg = v0::Message::try_compile(
        &member_refs[0].pubkey(),
        &[create_proposal_ix],
        &[],
        svm.latest_blockhash(),
    )
    .unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[member_refs[0]]).unwrap();
    let result = svm.send_transaction(tx).unwrap();
    println!("Action: Create proposal");
    print_stats(&svm, &result, &state_pda, &Some(proposal_acc));

    // All 250 members vote yes (vote = 1)
    for (i, member) in member_refs.iter().enumerate() {
        let ix = create_vote_ix(
            program_id,
            member,
            state_pda,
            proposal_acc,
            state_bump,
            0, // proposal id
            1, // yes
            proposal_bump,
            multisig_id,
        );
        let msg =
            v0::Message::try_compile(&member.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
        let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[member]).unwrap();
        let result = svm.send_transaction(tx).unwrap();
        if (i + 1) % 25 == 0 {
            println!(
                "Action: Vote (member {}) - {} members have voted...",
                i,
                i + 1
            );
        }
    }

    // Check proposal vote counts
    let proposal_data = svm.get_account(&proposal_acc).unwrap().data;
    let proposal = Proposal::from_bytes(&proposal_data[..Proposal::LEN]).unwrap();
    assert_eq!(proposal.yes_votes, 250);
    assert_eq!(proposal.no_votes, 0);
    assert_eq!(proposal.veto_votes, 0);
    let voter_list_data = &proposal_data[Proposal::LEN..];
    let voter_list = voter_list_data.chunks(1).map(|v| v[0]).collect::<Vec<u8>>();
    assert_eq!(voter_list.len(), 250);
    println!(
        "Action: All 250 members voted yes. Proposal: {:?}",
        proposal
    );
    print_stats(&svm, &result, &state_pda, &Some(proposal_acc));
}
