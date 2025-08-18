pub mod jsonrpc_client;
#[cfg(feature = "grpc")]
pub mod grpc_client;
#[cfg(feature = "grpc")]
pub mod grpc_wrapper;

pub use jsonrpc_client::{JsonRpcClient, GenerateResponse, ValidationResultResponse, PartialUpdateResultResponse, StreamEvent, ServiceInfoResponse};
#[cfg(feature = "grpc")]
pub use grpc_client::GrpcClient;
#[cfg(feature = "grpc")]
pub use grpc_wrapper::{GrpcClientWrapper, GrpcWrapperConfig, GrpcWrapperConfigBuilder};