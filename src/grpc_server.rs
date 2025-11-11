use std::pin::Pin;
use futures_core::Stream as FutStream;
use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::wrappers::BroadcastStream;
use futures_util::StreamExt;

pub mod transaction {
    tonic::include_proto!("transaction");
}

use transaction::transaction_service_server::{TransactionService, TransactionServiceServer};
use transaction::{StreamTransactionsRequest, TransactionResponse};

type ResponseStream = Pin<Box<dyn FutStream<Item = Result<TransactionResponse, Status>> + Send + Sync>>;

#[derive(Clone)]
pub struct MyTransactionService {
    pub tx: tokio::sync::broadcast::Sender<(String, u64)>,
}

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    type StreamTransactionsStream = ResponseStream;

    async fn stream_transactions(
        &self,
        _request: Request<StreamTransactionsRequest>,
    ) -> Result<Response<Self::StreamTransactionsStream>, Status> {
        let rx = self.tx.subscribe();

        let stream = BroadcastStream::new(rx).filter_map(|result| async move {
            match result {
                Ok((json, timestamp)) => Some(Ok(TransactionResponse {
                    transaction_json: json,
                    timestamp,
                })),
                Err(_) => None,
            }
        });

        Ok(Response::new(Box::pin(stream) as Self::StreamTransactionsStream))
    }
}

pub async fn serve_grpc(
    addr: std::net::SocketAddr,
    tx: tokio::sync::broadcast::Sender<(String, u64)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = MyTransactionService { tx };
    Server::builder()
        .add_service(TransactionServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}