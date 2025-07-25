use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::body::Body;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, Request};
use axum::Router;
use baizekit_app::async_trait::async_trait;
use baizekit_app::component::Component;
use baizekit_app::config::Config;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::{info, Level, Span};
use utoipa::openapi::path::Operation;
use utoipa::openapi::{Info, OpenApi, Paths};
use utoipa_swagger_ui::SwaggerUi;
use baizekit_app::anyhow;
use baizekit_app::anyhow::Context;

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

#[derive(Debug, Deserialize)]
pub struct AxumComponentConfig {
    pub port: u16,
}

impl AxumComponentConfig {
    fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), self.port)
    }
}

// 依赖基础组件的高级组件
pub struct AxumComponent {
    router: Router,
    openapi: OpenApi,
    // 用于通知服务器关闭的信号
    shutdown_trigger: CancellationToken,
    shutdown_done: CancellationToken,
}

impl AxumComponent {
    pub async fn new(
        services: Vec<AxumServiceInfo>,
    ) -> anyhow::Result<Self> {
        let shutdown_token = CancellationToken::new();

        let mut router = Router::new();
        let mut openapi = OpenApi::new(Info::new("App", "0.1.0"), Paths::new());

        for info in services {
            router = router.nest(&info.path, info.router);
            openapi = openapi.nest(&info.path, info.openapi);
        }

        Ok(AxumComponent {
            router,
            openapi,
            shutdown_trigger: shutdown_token.child_token(),
            shutdown_done: shutdown_token,
        })
    }

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

#[async_trait]
impl Component for AxumComponent {
    async fn init(&self, config: &Config, label: String) -> anyhow::Result<()> {
        let conf: AxumComponentConfig = config.get("server")?;

        // 绑定地址
        let listener = TcpListener::bind(conf.socket_addr())
            .await.context("listener bind failed.")?;
        info!("[{}] Axum服务器绑定到: {}", label, conf.socket_addr());

        self.print_service_info();

        // 创建路由
        let router = self
            .router
            .clone()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", self.openapi.clone()))
            .layer(
                CorsLayer::new()
                    .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_origin(Any),
            )
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<Body>| {
                        let x_request_id = request.extensions().get::<RequestId>();
                        tracing::debug_span!("request", ?x_request_id, method = %request.method(), uri = %request.uri())
                    })
                    .on_request(|r: &Request<Body>, _span: &Span| info!("request: {} {}", r.method(), r.uri().path()))
                    .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
            )
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()))
            .layer(PropagateRequestIdLayer::x_request_id());

        let shutdown_trigger = self.shutdown_trigger.clone();
        let shutdown_done = self.shutdown_done.clone();
        // 启动服务器并保存任务句柄（不阻塞init方法）
        tokio::spawn(async move {
            // 启动Axum服务器，配置优雅关闭
            let server = axum::serve(listener, router).with_graceful_shutdown(async move {
                // 等待关闭通知
                shutdown_trigger.cancelled().await;
                info!("Axum服务器开始优雅关闭...");
                shutdown_done.cancel();
            });

            // 运行服务器，返回结果
            let _ = server.await;
            info!("Axum服务器已完全关闭");
        });

        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        info!("收到关闭信号，通知Axum服务器关闭...");
        // 发送关闭通知
        self.shutdown_trigger.cancel();
        // 等待服务关闭
        self.shutdown_done.cancelled().await;
        info!("Axum组件已关闭");
        Ok(())
    }
}
