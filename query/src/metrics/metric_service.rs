// Copyright 2020 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::convert::Infallible;
use std::net::SocketAddr;

use axum::body::Bytes;
use axum::body::Full;
use axum::extract::Extension;
use axum::handler::get;
use axum::http::Response;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::AddExtensionLayer;
use axum::Router;
use common_base::tokio;
use common_exception::ErrorCode;
use common_exception::Result;
use common_metrics::PrometheusHandle;

use crate::common::service::HttpShutdownHandler;
use crate::servers::Server;
use crate::sessions::SessionManagerRef;

pub struct MetricService {
    shutdown_handler: HttpShutdownHandler,
}

pub struct MetricTemplate {
    prom: PrometheusHandle,
}

impl IntoResponse for MetricTemplate {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        Html(self.prom.render()).into_response()
    }
}

pub async fn metric_handler(prom_extension: Extension<PrometheusHandle>) -> MetricTemplate {
    let prom = prom_extension.0;
    MetricTemplate { prom }
}

// build axum router for metric server
macro_rules! build_router {
    ($prometheus: expr) => {
        Router::new()
            .route("/metrics", get(metric_handler))
            .layer(AddExtensionLayer::new($prometheus.clone()))
    };
}

impl MetricService {
    // TODO add session tls handler
    pub fn create(_sessions: SessionManagerRef) -> Box<MetricService> {
        Box::new(MetricService {
            shutdown_handler: HttpShutdownHandler::create("metric api".to_string()),
        })
    }

    async fn start_without_tls(&mut self, listening: SocketAddr) -> Result<SocketAddr> {
        let prometheus_handle = common_metrics::try_handle().ok_or_else(|| {
            ErrorCode::InitPrometheusFailure("Prometheus recorder has not been initialized yet.")
        })?;
        let app = build_router!(prometheus_handle);
        let server = axum_server::bind(listening.to_string())
            .handle(self.shutdown_handler.abort_handle.clone())
            .serve(app);
        self.shutdown_handler.try_listen(tokio::spawn(server)).await
    }
}

#[async_trait::async_trait]
impl Server for MetricService {
    async fn shutdown(&mut self, graceful: bool) {
        self.shutdown_handler.shutdown(graceful).await;
    }

    async fn start(&mut self, listening: SocketAddr) -> Result<SocketAddr> {
        self.start_without_tls(listening).await
    }
}
