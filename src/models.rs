use serde::de;
use derivative::Derivative;
use ordered_float::OrderedFloat;
use serde::{Serialize, Deserialize, Deserializer};

pub enum OrderType {
    Buy = 0,
    Sell = 1,
}

#[derive(Debug)]
pub struct OrderBook {
    pub bids: Vec<LimitPrice>,
    pub asks: Vec<LimitPrice>,
}

#[derive(Derivative)]
#[derivative(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct LimitPrice {
    pub price: OrderedFloat<f64>,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub size: OrderedFloat<f64>,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Data {
    Trade(Trade),
    Order(Order),
    None {},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Msg {
    pub channel: String,
    pub event: String,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    pub id: u64,
    pub amount: f64,
    pub amount_str: String,
    pub buy_order_id: u64,
    pub microtimestamp: String,
    pub price: f64,
    pub price_str: String,
    pub sell_order_id: u64,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub _type: u8,
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Order {
    pub id: u64,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub id_str: String,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub order_type: u8,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub datetime: String,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub microtimestamp: String,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub amount: f64,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub amount_str: String,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub price: f64,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        Ord = "ignore",
        PartialOrd = "ignore"
    )]
    pub price_str: String,
}



#[derive(Debug, Deserialize)]
pub struct OfferData {
    #[serde(deserialize_with = "de_float_from_str")]
    pub price: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub size: f32,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepthStreamData {
    pub last_update_id: usize,
    pub bids: Vec<OfferData>,
    pub asks: Vec<OfferData>,
}
pub fn de_float_from_str<'a, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;
    str_val.parse::<f32>().map_err(de::Error::custom)
}

/// The request with a id of the book
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBookRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
/// The response details of a book
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBookResponse {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub author: ::prost::alloc::string::String,
    #[prost(int32, tag="4")]
    pub year: i32,
}
/// Generated client implementations.
pub mod bookstore_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// The book store service definition.
    #[derive(Debug, Clone)]
    pub struct BookstoreClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BookstoreClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> BookstoreClient<T>
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
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> BookstoreClient<InterceptedService<T, F>>
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
            BookstoreClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// Retrieve a book
        pub async fn get_book(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBookRequest>,
        ) -> Result<tonic::Response<super::GetBookResponse>, tonic::Status> {
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
                "/bookstore.Bookstore/GetBook",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod bookstore_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with BookstoreServer.
    #[async_trait]
    pub trait Bookstore: Send + Sync + 'static {
        /// Retrieve a book
        async fn get_book(
            &self,
            request: tonic::Request<super::GetBookRequest>,
        ) -> Result<tonic::Response<super::GetBookResponse>, tonic::Status>;
    }
    /// The book store service definition.
    #[derive(Debug)]
    pub struct BookstoreServer<T: Bookstore> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Bookstore> BookstoreServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
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
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for BookstoreServer<T>
    where
        T: Bookstore,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/bookstore.Bookstore/GetBook" => {
                    #[allow(non_camel_case_types)]
                    struct GetBookSvc<T: Bookstore>(pub Arc<T>);
                    impl<T: Bookstore> tonic::server::UnaryService<super::GetBookRequest>
                    for GetBookSvc<T> {
                        type Response = super::GetBookResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBookRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_book(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBookSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
    impl<T: Bookstore> Clone for BookstoreServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Bookstore> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Bookstore> tonic::transport::NamedService for BookstoreServer<T> {
        const NAME: &'static str = "bookstore.Bookstore";
    }
}
