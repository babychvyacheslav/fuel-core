use super::{
    BlockImporterAdapter,
    P2PAdapter,
    VerifierAdapter,
};
use fuel_core_services::stream::BoxStream;
use fuel_core_sync::ports::{
    BlockImporterPort,
    ConsensusPort,
    PeerToPeerPort,
};
use fuel_core_types::{
    blockchain::{
        primitives::{
            BlockHeight,
            BlockId,
        },
        SealedBlock,
        SealedBlockHeader,
    },
    fuel_tx::Transaction,
    services::p2p::SourcePeer,
};

#[async_trait::async_trait]
impl PeerToPeerPort for P2PAdapter {
    fn height_stream(&self) -> BoxStream<BlockHeight> {
        use futures::StreamExt;
        fuel_core_services::stream::IntoBoxStream::into_boxed(
            tokio_stream::wrappers::BroadcastStream::new(
                self.service.subscribe_block_height(),
            )
            .filter_map(|r| futures::future::ready(r.ok().map(|r| r.block_height))),
        )
    }

    async fn get_sealed_block_header(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Option<SourcePeer<SealedBlockHeader>>> {
        Ok(self.service.get_sealed_block_header(height).await?.map(
            |(peer_id, header)| SourcePeer {
                peer_id: peer_id.into(),
                data: header,
            },
        ))
    }

    async fn get_transactions(
        &self,
        block: SourcePeer<BlockId>,
    ) -> anyhow::Result<Option<Vec<Transaction>>> {
        let SourcePeer {
            peer_id,
            data: block,
        } = block;
        self.service
            .get_transactions_from_peer(peer_id.into(), block)
            .await
    }
}

#[async_trait::async_trait]
impl BlockImporterPort for BlockImporterAdapter {
    fn committed_height_stream(&self) -> BoxStream<BlockHeight> {
        use futures::StreamExt;
        fuel_core_services::stream::IntoBoxStream::into_boxed(
            tokio_stream::wrappers::BroadcastStream::new(self.block_importer.subscribe())
                .filter_map(|r| {
                    futures::future::ready(
                        r.ok().map(|r| *r.sealed_block.entity.header().height()),
                    )
                }),
        )
    }
    async fn execute_and_commit(&self, block: SealedBlock) -> anyhow::Result<()> {
        self.execute_and_commit(block).await
    }
}

impl ConsensusPort for VerifierAdapter {
    fn check_sealed_header(&self, header: &SealedBlockHeader) -> anyhow::Result<bool> {
        Ok(self.block_verifier.verify_consensus(header))
    }
}