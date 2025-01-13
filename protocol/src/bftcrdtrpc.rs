#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrSetNodeMessage {
    /// predecessor hashes
    #[prost(string, repeated, tag = "1")]
    pub predecessors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(oneof = "or_set_node_message::Operation", tags = "2, 3")]
    pub operation: ::core::option::Option<or_set_node_message::Operation>,
}
/// Nested message and enum types in `ORSetNodeMessage`.
pub mod or_set_node_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct AddMessage {
        /// elem for Add operation
        #[prost(int32, tag = "1")]
        pub elem: i32,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RemMessage {
        /// IDs to remove
        #[prost(string, repeated, tag = "1")]
        pub ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// elem to remove
        #[prost(int32, tag = "2")]
        pub elem: i32,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Operation {
        /// Add operation
        #[prost(message, tag = "2")]
        Add(AddMessage),
        /// Remove operation
        #[prost(message, tag = "3")]
        Rem(RemMessage),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrSetRequest {
    /// History
    #[prost(message, repeated, tag = "1")]
    pub nodes: ::prost::alloc::vec::Vec<OrSetNodeMessage>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrSetResponse {
    /// Map from int to list<String>
    #[prost(map = "int32, message", tag = "1")]
    pub result_map: ::std::collections::HashMap<i32, or_set_response::ElemIds>,
}
/// Nested message and enum types in `ORSetResponse`.
pub mod or_set_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ElemIds {
        /// List of strings
        #[prost(string, repeated, tag = "1")]
        pub elem_id: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RgaNodeMessage {
    /// predecessor hashes
    #[prost(string, repeated, tag = "1")]
    pub predecessors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(oneof = "rga_node_message::Operation", tags = "2, 3")]
    pub operation: ::core::option::Option<rga_node_message::Operation>,
}
/// Nested message and enum types in `RGANodeMessage`.
pub mod rga_node_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct InsertMessage {
        /// Integer value
        #[prost(int32, tag = "1")]
        pub value: i32,
        /// String id
        #[prost(string, tag = "2")]
        pub id: ::prost::alloc::string::String,
        /// (String, String) elem_id
        #[prost(message, optional, tag = "3")]
        pub elem_id: ::core::option::Option<ElemId>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct DeleteMessage {
        /// (String, String) elem_id
        #[prost(message, optional, tag = "1")]
        pub elem_id: ::core::option::Option<ElemId>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ElemId {
        /// First part of the tuple
        #[prost(string, tag = "1")]
        pub first: ::prost::alloc::string::String,
        /// Second part of the tuple
        #[prost(string, tag = "2")]
        pub second: ::prost::alloc::string::String,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Operation {
        /// Insert operation
        #[prost(message, tag = "2")]
        Insert(InsertMessage),
        /// Delete operation
        #[prost(message, tag = "3")]
        Delete(DeleteMessage),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RgaRequest {
    /// History
    #[prost(message, repeated, tag = "1")]
    pub nodes: ::prost::alloc::vec::Vec<RgaNodeMessage>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RgaResponse {
    /// Result of the operation
    #[prost(string, tag = "1")]
    pub result: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod bftcrdt_tester_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct BftcrdtTesterServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BftcrdtTesterServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> BftcrdtTesterServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> BftcrdtTesterServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            BftcrdtTesterServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn test_or_set_once(
            &mut self,
            request: impl tonic::IntoRequest<super::OrSetRequest>,
        ) -> std::result::Result<tonic::Response<super::OrSetResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bftcrdtrpc.BFTCRDTTesterService/testORSetOnce",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("bftcrdtrpc.BFTCRDTTesterService", "testORSetOnce"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn test_rga_once(
            &mut self,
            request: impl tonic::IntoRequest<super::RgaRequest>,
        ) -> std::result::Result<tonic::Response<super::RgaResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bftcrdtrpc.BFTCRDTTesterService/testRGAOnce",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("bftcrdtrpc.BFTCRDTTesterService", "testRGAOnce"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod bftcrdt_tester_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with BftcrdtTesterServiceServer.
    #[async_trait]
    pub trait BftcrdtTesterService: Send + Sync + 'static {
        async fn test_or_set_once(
            &self,
            request: tonic::Request<super::OrSetRequest>,
        ) -> std::result::Result<tonic::Response<super::OrSetResponse>, tonic::Status>;
        async fn test_rga_once(
            &self,
            request: tonic::Request<super::RgaRequest>,
        ) -> std::result::Result<tonic::Response<super::RgaResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct BftcrdtTesterServiceServer<T: BftcrdtTesterService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: BftcrdtTesterService> BftcrdtTesterServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for BftcrdtTesterServiceServer<T>
    where
        T: BftcrdtTesterService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/bftcrdtrpc.BFTCRDTTesterService/testORSetOnce" => {
                    #[allow(non_camel_case_types)]
                    struct testORSetOnceSvc<T: BftcrdtTesterService>(pub Arc<T>);
                    impl<
                        T: BftcrdtTesterService,
                    > tonic::server::UnaryService<super::OrSetRequest>
                    for testORSetOnceSvc<T> {
                        type Response = super::OrSetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OrSetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).test_or_set_once(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = testORSetOnceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bftcrdtrpc.BFTCRDTTesterService/testRGAOnce" => {
                    #[allow(non_camel_case_types)]
                    struct testRGAOnceSvc<T: BftcrdtTesterService>(pub Arc<T>);
                    impl<
                        T: BftcrdtTesterService,
                    > tonic::server::UnaryService<super::RgaRequest>
                    for testRGAOnceSvc<T> {
                        type Response = super::RgaResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RgaRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).test_rga_once(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = testRGAOnceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: BftcrdtTesterService> Clone for BftcrdtTesterServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: BftcrdtTesterService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: BftcrdtTesterService> tonic::server::NamedService
    for BftcrdtTesterServiceServer<T> {
        const NAME: &'static str = "bftcrdtrpc.BFTCRDTTesterService";
    }
}
