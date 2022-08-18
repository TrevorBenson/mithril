mod init;

use mithril_common::crypto_helper::tests_setup;
use mithril_common::entities::ProtocolMessagePartKey;
use mithril_common::entities::SignerWithStake;
use mithril_common::fake_data;

#[tokio::test]
async fn create_certificate() {
    // initialization
    let mut tester = init::RuntimeTester::build().await;

    // create signers & declare stake distribution
    let signers = tests_setup::setup_signers(2);
    let signers_with_stake: Vec<SignerWithStake> =
        signers.clone().into_iter().map(|s| s.into()).collect();
    tester
        .chain_observer
        .set_signers(signers_with_stake.clone())
        .await;
    tester
        .deps
        .simulate_genesis(
            signers_with_stake.clone(),
            signers_with_stake,
            fake_data::protocol_parameters(),
        )
        .await;

    // start the runtime state machine
    cycle!(tester, "signing");
    cycle!(tester, "signing");

    // register signers
    tester.register_signers(&signers).await.unwrap();
    cycle!(tester, "signing");

    // change the immutable number to alter the beacon
    tester.increase_immutable_number().await.unwrap();
    cycle!(tester, "idle");
    cycle!(tester, "signing");
    cycle!(tester, "signing");

    // signers send their single signature
    tester.send_single_signatures(&signers).await.unwrap();

    // The state machine should issue a multisignature
    cycle!(tester, "idle");
    let (last_certificates, snapshots) =
        tester.get_last_certificates_and_snapshots().await.unwrap();

    assert_eq!((1, 1), (last_certificates.len(), snapshots.len()));
    assert_eq!(
        (
            &last_certificates[0].hash,
            last_certificates[0]
                .protocol_message
                .get_message_part(&ProtocolMessagePartKey::SnapshotDigest)
                .unwrap()
        ),
        (&snapshots[0].certificate_hash, &snapshots[0].digest)
    );
    cycle!(tester, "idle");
}
