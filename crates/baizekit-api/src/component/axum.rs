use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::body::Body;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, Request};
use axum::Router;
use baizekit_app::anyhow::Context;
use baizekit_app::anyhow::Result;
use baizekit_app::application::ApplicationInner;
use baizekit_app::async_trait::async_trait;
use baizekit_app::component::Component;
use baizekit_app::config::Config;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
pub use tower_http::cors::AllowOrigin;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::trace;
use tower_http::trace::TraceLayer;
pub use tracing::Level;
use tracing::{info, Span};
use utoipa::openapi::path::Operation;
use utoipa::openapi::{Info, OpenApi, Paths};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Debug, Deserialize)]
pub struct AxumComponentConfig {
    /// 服务器监听地址,格式：IP: 0.0.0.0:8080 或 `[::1]:8080`
    #[serde(deserialize_with = "deserialize_socket_addr")]
    pub addr: SocketAddr,
}

fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let addr_str: String = String::deserialize(deserializer)?;

    SocketAddr::from_str(&addr_str)
        .map_err(|e| D::Error::custom(format!("invalid socket address '{}': {}", addr_str, e)))
}

impl Default for AxumComponentConfig {
    fn default() -> Self {
        Self { addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080) }
    }
}

pub struct AxumComponent {
    router: Router,
    openapi: OpenApi,
    config: AxumComponentConfig,
    shutdown_trigger: CancellationToken,
    shutdown_done: CancellationToken,
}

pub struct AxumServiceInfo {
    pub path: String,
    pub router: Router,
    pub openapi: OpenApi,
}

impl AxumServiceInfo {
    pub fn new(path: impl Into<String>, router: Router, openapi: OpenApi) -> Self {
        AxumServiceInfo { path: path.into(), router, openapi }
    }
}

pub struct AxumComponentBuilder {
    services: Vec<AxumServiceInfo>,
    default_health_route: bool,
    openapi_title: String,
    openapi_version: String,
    layers: Vec<Box<dyn Fn(Router) -> Router + Send + Sync + 'static>>,
}

impl AxumComponentBuilder {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            default_health_route: true,
            openapi_title: "App".to_string(),
            openapi_version: "0.1.0".to_string(),
            layers: Vec::new(),
        }
    }

    pub fn add_service(mut self, service: AxumServiceInfo) -> Self {
        self.services.push(service);
        self
    }

    pub fn add_services(mut self, services: Vec<AxumServiceInfo>) -> Self {
        self.services.extend(services);
        self
    }

    pub fn with_default_health_route(mut self, enable: bool) -> Self {
        self.default_health_route = enable;
        self
    }

    pub fn with_openapi_info(mut self, title: impl Into<String>, version: impl Into<String>) -> Self {
        self.openapi_title = title.into();
        self.openapi_version = version.into();
        self
    }

    /// 配置CORS
    /// allow_origin: 允许的源，None时默认使用Any
    pub fn with_cors(mut self, allow_origin: Option<AllowOrigin>) -> Self {
        let allow_origin = allow_origin.unwrap_or(AllowOrigin::any());

        self.layers.push(Box::new(move |router| {
            let cors_layer = CorsLayer::new()
                .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_origin(allow_origin.clone());

            router.layer(cors_layer)
        }));
        self
    }

    /// 配置请求追踪
    /// level: 追踪级别，None时默认使用INFO
    pub fn with_trace(mut self, level: Option<Level>) -> Self {
        let trace_level = level.unwrap_or(Level::INFO);

        self.layers.push(Box::new(move |router| {
            let mut router = router.layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<Body>| {
                        let x_request_id = request.extensions().get::<RequestId>();
                        tracing::debug_span!("request", ?x_request_id, method = %request.method(), uri = %request.uri())
                    })
                    .on_request(|r: &Request<Body>, _span: &Span| info!("request: {} {}", r.method(), r.uri().path()))
                    .on_response(trace::DefaultOnResponse::new().level(trace_level)),
            );

            router = router
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()))
                .layer(PropagateRequestIdLayer::x_request_id());

            router
        }));
        self
    }

    pub fn with_layer<F>(mut self, layer: F) -> Self
    where
        F: Fn(Router) -> Router + Send + Sync + 'static,
    {
        self.layers.push(Box::new(layer));
        self
    }

    pub async fn build(self, inner: Arc<ApplicationInner>, label: String) -> Result<AxumComponent> {
        let config = inner.config().await;
        let conf: AxumComponentConfig = config.get(format!("axum.{}", label).as_str())?;

        let shutdown_token = CancellationToken::new();

        let mut router = Router::new();
        let mut openapi = OpenApi::new(Info::new(&self.openapi_title, &self.openapi_version), Paths::new());

        for info in &self.services {
            router = router.nest(&info.path, info.router.clone());
            openapi = openapi.nest(&info.path, info.openapi.clone());
        }

        let router = if self.default_health_route {
            router.route("/health", axum::routing::get(|| async { "OK" }))
        } else {
            router
        };

        let mut router = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi.clone()));

        for layer in self.layers {
            router = layer(router);
        }

        Ok(AxumComponent {
            router,
            openapi,
            config: conf,
            shutdown_trigger: shutdown_token.child_token(),
            shutdown_done: shutdown_token,
        })
    }
}

#[async_trait]
impl Component for AxumComponent {
    async fn init(&mut self, _config: &Config, label: String) -> Result<()> {
        let listener = TcpListener::bind(self.config.addr).await.context("listener bind failed.")?;
        info!("[{}] Axum服务器绑定到: {}", label, self.config.addr);

        self.print_service_info();

        let shutdown_trigger = self.shutdown_trigger.clone();
        let shutdown_done = self.shutdown_done.clone();
        let router = self.router.clone();

        tokio::spawn(async move {
            let server = axum::serve(listener, router).with_graceful_shutdown(async move {
                shutdown_trigger.cancelled().await;
                info!("Axum服务器开始优雅关闭...");
                shutdown_done.cancel();
            });

            let _ = server.await;
            info!("Axum服务器已完全关闭");
        });

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!("收到关闭信号，通知Axum服务器关闭...");
        self.shutdown_trigger.cancel();
        self.shutdown_done.cancelled().await;
        info!("Axum组件已关闭");
        Ok(())
    }
}

impl AxumComponent {
    fn print_service_info(&self) {
        let print_operation = |method: &str, path: &str, operation: Option<Operation>| {
            let Some(operation) = operation else { return };
            info!("{:>6} - {}: {}", method, path, operation.summary.unwrap_or_default());
        };

        for (path, path_item) in self.openapi.paths.paths.clone() {
            print_operation("GET", &path, path_item.get);
            print_operation("POST", &path, path_item.post);
            print_operation("PUT", &path, path_item.put);
            print_operation("DELETE", &path, path_item.delete);
            print_operation("PATCH", &path, path_item.patch);
            print_operation("HEAD", &path, path_item.head);
            print_operation("OPTIONS", &path, path_item.options);
            print_operation("TRACE", &path, path_item.trace);
        }
    }
}
