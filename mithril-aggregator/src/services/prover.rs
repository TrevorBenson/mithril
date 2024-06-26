use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use async_trait::async_trait;

use mithril_common::{
    crypto_helper::MKTree,
    entities::{
        BlockRange, CardanoDbBeacon, CardanoTransaction, CardanoTransactionsSetProof,
        TransactionHash,
    },
    signable_builder::BlockRangeRootRetriever,
    StdResult,
};

/// Prover service is the cryptographic engine in charge of producing cryptographic proofs for transactions
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProverService: Sync + Send {
    /// Compute the cryptographic proofs for the given transactions
    async fn compute_transactions_proofs(
        &self,
        up_to: &CardanoDbBeacon,
        transaction_hashes: &[TransactionHash],
    ) -> StdResult<Vec<CardanoTransactionsSetProof>>;
}

/// Transactions retriever
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TransactionsRetriever: Sync + Send {
    /// Get all transactions up to given beacon using chronological order
    async fn get_up_to(&self, beacon: &CardanoDbBeacon) -> StdResult<Vec<CardanoTransaction>>;

    /// Get a list of transactions by hashes using chronological order
    async fn get_by_hashes(
        &self,
        hashes: Vec<TransactionHash>,
    ) -> StdResult<Vec<CardanoTransaction>>;

    /// Get by block ranges
    async fn get_by_block_ranges(
        &self,
        block_ranges: Vec<BlockRange>,
    ) -> StdResult<Vec<CardanoTransaction>>;
}

/// Mithril prover
pub struct MithrilProverService {
    transaction_retriever: Arc<dyn TransactionsRetriever>,
    block_range_root_retriever: Arc<dyn BlockRangeRootRetriever>,
}

impl MithrilProverService {
    /// Create a new Mithril prover
    pub fn new(
        transaction_retriever: Arc<dyn TransactionsRetriever>,
        block_range_root_retriever: Arc<dyn BlockRangeRootRetriever>,
    ) -> Self {
        Self {
            transaction_retriever,
            block_range_root_retriever,
        }
    }

    async fn get_block_ranges(
        &self,
        transaction_hashes: &[TransactionHash],
    ) -> StdResult<Vec<BlockRange>> {
        let transactions = self
            .transaction_retriever
            .get_by_hashes(transaction_hashes.to_vec())
            .await?;
        let block_ranges = transactions
            .iter()
            .map(|t| BlockRange::from_block_number(t.block_number))
            .collect::<BTreeSet<_>>();

        Ok(block_ranges.into_iter().collect::<Vec<_>>())
    }

    /// Get all the transactions of the block ranges
    async fn get_all_transactions_for_block_ranges(
        &self,
        block_ranges: &[BlockRange],
    ) -> StdResult<HashMap<BlockRange, Vec<CardanoTransaction>>> {
        let mut block_ranges_map = HashMap::new();
        let transactions = self
            .transaction_retriever
            .get_by_block_ranges(block_ranges.to_vec())
            .await?;
        for transaction in transactions {
            let block_range = BlockRange::from_block_number(transaction.block_number);
            let block_range_transactions: &mut Vec<_> =
                block_ranges_map.entry(block_range).or_insert(vec![]);
            block_range_transactions.push(transaction)
        }

        Ok(block_ranges_map)
    }
}

#[async_trait]
impl ProverService for MithrilProverService {
    async fn compute_transactions_proofs(
        &self,
        up_to: &CardanoDbBeacon,
        transaction_hashes: &[TransactionHash],
    ) -> StdResult<Vec<CardanoTransactionsSetProof>> {
        // 1 - Compute the set of block ranges with transactions to prove
        let block_ranges_transactions = self.get_block_ranges(transaction_hashes).await?;
        let block_range_transactions = self
            .get_all_transactions_for_block_ranges(&block_ranges_transactions)
            .await?;

        // 2 - Compute block ranges sub Merkle trees
        let mut mk_trees = BTreeMap::new();
        for (block_range, transactions) in block_range_transactions {
            let mk_tree = MKTree::new(&transactions)?;
            mk_trees.insert(block_range, mk_tree);
        }

        // 3 - Compute block range roots Merkle map
        let mut mk_map = self
            .block_range_root_retriever
            .compute_merkle_map_from_block_range_roots(up_to.immutable_file_number)
            .await?;

        // 4 - Enrich the Merkle map with the block ranges Merkle trees
        for (block_range, mk_tree) in mk_trees {
            mk_map.insert(block_range, mk_tree.into())?;
        }

        // 5 - Compute the proof for all transactions
        if let Ok(mk_proof) = mk_map.compute_proof(transaction_hashes) {
            let transaction_hashes_certified: Vec<TransactionHash> = transaction_hashes
                .iter()
                .filter(|hash| mk_proof.contains(&hash.as_str().into()).is_ok())
                .cloned()
                .collect();

            Ok(vec![CardanoTransactionsSetProof::new(
                transaction_hashes_certified,
                mk_proof,
            )])
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::max;

    use anyhow::anyhow;
    use mithril_common::crypto_helper::{MKMap, MKMapNode, MKTreeNode};
    use mithril_common::entities::{CardanoTransaction, ImmutableFileNumber};
    use mockall::mock;
    use mockall::predicate::eq;

    use super::*;

    mock! {
        pub BlockRangeRootRetrieverImpl { }

        #[async_trait]
        impl BlockRangeRootRetriever for BlockRangeRootRetrieverImpl {
            async fn retrieve_block_range_roots(
                &self,
                up_to_beacon: ImmutableFileNumber,
            ) -> StdResult<Box<dyn Iterator<Item = (BlockRange, MKTreeNode)>>>;

            async fn compute_merkle_map_from_block_range_roots(
                &self,
                up_to_beacon: ImmutableFileNumber,
            ) -> StdResult<MKMap<BlockRange, MKMapNode<BlockRange>>>;
        }
    }

    mod test_data {
        use super::*;

        // Generate transactions for 'total_block_ranges' consecutive block ranges,
        // with 'total_transactions_per_block_range' transactions per block range
        pub fn generate_transactions(
            total_block_ranges: usize,
            total_transactions_per_block_range: usize,
        ) -> Vec<CardanoTransaction> {
            let block_range_length = BlockRange::LENGTH as usize;
            let max_transaction_per_block_number =
                max(1, total_transactions_per_block_range / block_range_length);
            let mut transactions = vec![];

            for i in 0..total_block_ranges {
                let block_range = BlockRange::from_block_number((i * block_range_length) as u64);
                for j in 0..total_transactions_per_block_range {
                    let transaction_index = i * total_transactions_per_block_range + j;
                    let block_number =
                        block_range.start + (j / max_transaction_per_block_number) as u64;
                    let slot_number = 100 * block_number;
                    let immutable_file_number = block_number / 5;
                    let tx_hash = format!(
                        "tx-br-{}..{}-{}-idx-{}",
                        block_range.start, block_range.end, j, transaction_index
                    );
                    let block_hash = format!("block_hash-{block_number}");
                    transactions.push(CardanoTransaction::new(
                        &tx_hash,
                        block_number,
                        slot_number,
                        block_hash,
                        immutable_file_number,
                    ));
                }
            }

            transactions
        }

        pub fn filter_transactions_for_indices(
            indices: &[usize],
            transactions: &[CardanoTransaction],
        ) -> Vec<CardanoTransaction> {
            transactions
                .iter()
                .enumerate()
                .filter(|(i, _)| indices.contains(i))
                .map(|(_, t)| t.to_owned())
                .collect()
        }

        pub fn compute_transaction_hashes_from_transactions(
            transactions: &[CardanoTransaction],
        ) -> Vec<TransactionHash> {
            transactions
                .iter()
                .map(|t| t.transaction_hash.clone())
                .collect()
        }

        pub fn compute_block_ranges_map_from_transactions(
            transactions: &[CardanoTransaction],
        ) -> BTreeMap<BlockRange, Vec<CardanoTransaction>> {
            let mut block_ranges_map = BTreeMap::new();
            for transaction in transactions {
                let block_range = BlockRange::from_block_number(transaction.block_number);
                let block_range_transactions: &mut Vec<_> =
                    block_ranges_map.entry(block_range).or_insert(vec![]);
                block_range_transactions.push(transaction.to_owned())
            }

            block_ranges_map
        }

        pub fn filter_transactions_for_block_ranges(
            block_ranges: &[BlockRange],
            transactions: &[CardanoTransaction],
        ) -> Vec<CardanoTransaction> {
            transactions
                .iter()
                .filter(|t| block_ranges.contains(&BlockRange::from_block_number(t.block_number)))
                .map(|t| t.to_owned())
                .collect()
        }

        pub fn compute_mk_map_from_block_ranges_map(
            block_ranges_map: BTreeMap<BlockRange, Vec<CardanoTransaction>>,
        ) -> MKMap<BlockRange, MKMapNode<BlockRange>> {
            MKMap::new_from_iter(
                block_ranges_map
                    .into_iter()
                    .map(|(block_range, transactions)| {
                        (
                            block_range,
                            MKMapNode::TreeNode(
                                MKTree::new(&transactions)
                                    .unwrap()
                                    .compute_root()
                                    .unwrap()
                                    .clone(),
                            ),
                        )
                    }),
            )
            .unwrap()
        }

        pub fn compute_beacon_from_transactions(
            transactions: &[CardanoTransaction],
        ) -> CardanoDbBeacon {
            CardanoDbBeacon {
                immutable_file_number: transactions.last().unwrap().immutable_file_number,
                ..CardanoDbBeacon::default()
            }
        }

        pub struct TestData {
            pub transaction_hashes_to_prove: Vec<TransactionHash>,
            pub block_ranges_map: BTreeMap<BlockRange, Vec<CardanoTransaction>>,
            pub block_ranges_to_prove: Vec<BlockRange>,
            pub all_transactions_in_block_ranges_to_prove: Vec<CardanoTransaction>,
            pub beacon: CardanoDbBeacon,
        }

        pub fn build_test_data(
            transactions_to_prove: &[CardanoTransaction],
            transactions: &[CardanoTransaction],
        ) -> TestData {
            let transaction_hashes_to_prove =
                compute_transaction_hashes_from_transactions(transactions_to_prove);
            let block_ranges_map = compute_block_ranges_map_from_transactions(transactions);
            let block_ranges_map_to_prove =
                compute_block_ranges_map_from_transactions(transactions_to_prove);
            let block_ranges_to_prove = block_ranges_map_to_prove
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            let all_transactions_in_block_ranges_to_prove =
                filter_transactions_for_block_ranges(&block_ranges_to_prove, transactions);
            let beacon = compute_beacon_from_transactions(transactions);

            TestData {
                transaction_hashes_to_prove,
                block_ranges_map,
                block_ranges_to_prove,
                all_transactions_in_block_ranges_to_prove,
                beacon,
            }
        }
    }

    fn build_prover<F, G>(
        transaction_retriever_mock_config: F,
        block_range_root_retriever_mock_config: G,
    ) -> MithrilProverService
    where
        F: FnOnce(&mut MockTransactionsRetriever),
        G: FnOnce(&mut MockBlockRangeRootRetrieverImpl),
    {
        let mut transaction_retriever = MockTransactionsRetriever::new();
        transaction_retriever_mock_config(&mut transaction_retriever);
        let mut block_range_root_retriever = MockBlockRangeRootRetrieverImpl::new();
        block_range_root_retriever_mock_config(&mut block_range_root_retriever);

        MithrilProverService::new(
            Arc::new(transaction_retriever),
            Arc::new(block_range_root_retriever),
        )
    }

    #[tokio::test]
    async fn compute_proof_for_one_set_of_three_known_transactions() {
        let total_block_ranges = 5;
        let total_transactions_per_block_range = 3;
        let transactions = test_data::generate_transactions(
            total_block_ranges,
            total_transactions_per_block_range,
        );
        let transactions_to_prove =
            test_data::filter_transactions_for_indices(&[1, 2, 4], &transactions);
        let test_data = test_data::build_test_data(&transactions_to_prove, &transactions);
        let prover = build_prover(
            |retriever_mock| {
                let transaction_hashes_to_prove = test_data.transaction_hashes_to_prove.clone();
                let transactions_to_prove = transactions_to_prove.clone();
                retriever_mock
                    .expect_get_by_hashes()
                    .with(eq(transaction_hashes_to_prove))
                    .return_once(move |_| Ok(transactions_to_prove));

                let block_ranges_to_prove = test_data.block_ranges_to_prove.clone();
                let all_transactions_in_block_ranges_to_prove =
                    test_data.all_transactions_in_block_ranges_to_prove.clone();
                retriever_mock
                    .expect_get_by_block_ranges()
                    .with(eq(block_ranges_to_prove))
                    .return_once(move |_| Ok(all_transactions_in_block_ranges_to_prove));
            },
            |block_range_root_retriever_mock| {
                let block_ranges_map = test_data.block_ranges_map.clone();
                block_range_root_retriever_mock
                    .expect_compute_merkle_map_from_block_range_roots()
                    .return_once(|_| {
                        Ok(test_data::compute_mk_map_from_block_ranges_map(
                            block_ranges_map,
                        ))
                    });
            },
        );

        let transactions_set_proof = prover
            .compute_transactions_proofs(&test_data.beacon, &test_data.transaction_hashes_to_prove)
            .await
            .unwrap();

        assert_eq!(transactions_set_proof.len(), 1);
        assert_eq!(
            transactions_set_proof[0].transactions_hashes(),
            test_data.transaction_hashes_to_prove
        );
        transactions_set_proof[0].verify().unwrap();
    }

    #[tokio::test]
    async fn cant_compute_proof_for_unknown_transaction() {
        let total_block_ranges = 5;
        let total_transactions_per_block_range = 3;
        let transactions = test_data::generate_transactions(
            total_block_ranges,
            total_transactions_per_block_range,
        );
        let transactions_to_prove = test_data::filter_transactions_for_indices(&[], &transactions);
        let mut test_data = test_data::build_test_data(&transactions_to_prove, &transactions);
        test_data.transaction_hashes_to_prove = vec!["tx-unknown-123".to_string()];
        let prover = build_prover(
            |retriever_mock| {
                let transaction_hashes_to_prove = test_data.transaction_hashes_to_prove.clone();
                let transactions_to_prove = transactions_to_prove.clone();
                retriever_mock
                    .expect_get_by_hashes()
                    .with(eq(transaction_hashes_to_prove))
                    .return_once(move |_| Ok(transactions_to_prove));

                let block_ranges_to_prove = test_data.block_ranges_to_prove.clone();
                let all_transactions_in_block_ranges_to_prove =
                    test_data.all_transactions_in_block_ranges_to_prove.clone();
                retriever_mock
                    .expect_get_by_block_ranges()
                    .with(eq(block_ranges_to_prove))
                    .return_once(move |_| Ok(all_transactions_in_block_ranges_to_prove));
            },
            |block_range_root_retriever_mock| {
                let block_ranges_map = test_data.block_ranges_map.clone();
                block_range_root_retriever_mock
                    .expect_compute_merkle_map_from_block_range_roots()
                    .return_once(|_| {
                        Ok(test_data::compute_mk_map_from_block_ranges_map(
                            block_ranges_map,
                        ))
                    });
            },
        );

        let transactions_set_proof = prover
            .compute_transactions_proofs(&test_data.beacon, &test_data.transaction_hashes_to_prove)
            .await
            .unwrap();

        assert_eq!(transactions_set_proof.len(), 0);
    }

    #[tokio::test]
    async fn compute_proof_for_one_set_of_three_known_transactions_and_two_unknowns() {
        let total_block_ranges = 5;
        let total_transactions_per_block_range = 3;
        let transactions = test_data::generate_transactions(
            total_block_ranges,
            total_transactions_per_block_range,
        );
        let transactions_to_prove =
            test_data::filter_transactions_for_indices(&[1, 2, 4], &transactions);
        let transaction_hashes_unknown =
            vec!["tx-unknown-123".to_string(), "tx-unknown-456".to_string()];
        let mut test_data = test_data::build_test_data(&transactions_to_prove, &transactions);
        let transaction_hashes_known = test_data.transaction_hashes_to_prove.clone();
        test_data.transaction_hashes_to_prove = [
            test_data.transaction_hashes_to_prove.clone(),
            transaction_hashes_unknown,
        ]
        .concat();
        let prover = build_prover(
            |retriever_mock| {
                let transaction_hashes_to_prove = test_data.transaction_hashes_to_prove.clone();
                let transactions_to_prove = transactions_to_prove.clone();
                retriever_mock
                    .expect_get_by_hashes()
                    .with(eq(transaction_hashes_to_prove))
                    .return_once(move |_| Ok(transactions_to_prove));

                let block_ranges_to_prove = test_data.block_ranges_to_prove.clone();
                let all_transactions_in_block_ranges_to_prove =
                    test_data.all_transactions_in_block_ranges_to_prove.clone();
                retriever_mock
                    .expect_get_by_block_ranges()
                    .with(eq(block_ranges_to_prove))
                    .return_once(move |_| Ok(all_transactions_in_block_ranges_to_prove));
            },
            |block_range_root_retriever_mock| {
                let block_ranges_map = test_data.block_ranges_map.clone();
                block_range_root_retriever_mock
                    .expect_compute_merkle_map_from_block_range_roots()
                    .return_once(|_| {
                        Ok(test_data::compute_mk_map_from_block_ranges_map(
                            block_ranges_map,
                        ))
                    });
            },
        );

        let transactions_set_proof = prover
            .compute_transactions_proofs(&test_data.beacon, &test_data.transaction_hashes_to_prove)
            .await
            .unwrap();

        assert_eq!(transactions_set_proof.len(), 1);
        assert_eq!(
            transactions_set_proof[0].transactions_hashes(),
            transaction_hashes_known
        );
        transactions_set_proof[0].verify().unwrap();
    }

    #[tokio::test]
    async fn cant_compute_proof_if_transaction_retriever_fails() {
        let total_block_ranges = 5;
        let total_transactions_per_block_range = 3;
        let transactions = test_data::generate_transactions(
            total_block_ranges,
            total_transactions_per_block_range,
        );
        let transactions_to_prove =
            test_data::filter_transactions_for_indices(&[1, 2, 4], &transactions);
        let test_data = test_data::build_test_data(&transactions_to_prove, &transactions);
        let prover = build_prover(
            |retriever_mock| {
                retriever_mock
                    .expect_get_by_hashes()
                    .returning(|_| Err(anyhow!("Error")));
            },
            |block_range_root_retriever_mock| {
                block_range_root_retriever_mock
                    .expect_compute_merkle_map_from_block_range_roots()
                    .return_once(|_| MKMap::new(&[]));
            },
        );

        prover
            .compute_transactions_proofs(&test_data.beacon, &test_data.transaction_hashes_to_prove)
            .await
            .expect_err("Should have failed because of transaction retriever failure");
    }

    #[tokio::test]
    async fn cant_compute_proof_if_block_range_root_retriever_fails() {
        let total_block_ranges = 5;
        let total_transactions_per_block_range = 3;
        let transactions = test_data::generate_transactions(
            total_block_ranges,
            total_transactions_per_block_range,
        );
        let transactions_to_prove =
            test_data::filter_transactions_for_indices(&[1, 2, 4], &transactions);
        let test_data = test_data::build_test_data(&transactions_to_prove, &transactions);
        let prover = build_prover(
            |retriever_mock| {
                let transactions_to_prove = transactions_to_prove.clone();
                retriever_mock
                    .expect_get_by_hashes()
                    .return_once(move |_| Ok(transactions_to_prove));

                let all_transactions_in_block_ranges_to_prove =
                    test_data.all_transactions_in_block_ranges_to_prove.clone();
                retriever_mock
                    .expect_get_by_block_ranges()
                    .return_once(move |_| Ok(all_transactions_in_block_ranges_to_prove));
            },
            |block_range_root_retriever_mock| {
                block_range_root_retriever_mock
                    .expect_compute_merkle_map_from_block_range_roots()
                    .return_once(|_| Err(anyhow!("Error")));
            },
        );

        prover
            .compute_transactions_proofs(&test_data.beacon, &test_data.transaction_hashes_to_prove)
            .await
            .expect_err("Should have failed because of block range root retriever failure");
    }
}
