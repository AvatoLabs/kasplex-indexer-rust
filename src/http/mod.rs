use axum::{Router, routing::get};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct HttpState {
    pub kaspa_rest_base_url: String,
}

pub fn build_router_with_state(state: HttpState) -> Router {
    Router::new().nest("/v1", v1_router()).with_state(state)
}

fn v1_router() -> Router<HttpState> {
    Router::new()
        .route("/info", get(handler_info))
        .route("/krc20/tokenlist", get(handler_krc20_tokenlist))
        .route("/krc20/token/{tick}", get(handler_krc20_token))
        .route(
            "/krc20/address/{address}/tokenlist",
            get(handler_krc20_address_tokenlist),
        )
        .route(
            "/krc20/address/{address}/token/{tick}",
            get(handler_krc20_address_token),
        )
        .route("/krc20/oplist", get(handler_krc20_oplist))
        .route("/krc20/op/{id}", get(handler_krc20_op))
        .route("/archive/vspc/{daascore}", get(handler_archive_vspc))
        .route("/archive/oplist/{oprange}", get(handler_archive_oplist))
        .route("/krc20/market/{tick}", get(handler_krc20_market))
        .route("/krc20/blacklist/{ca}", get(handler_krc20_blacklist))
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiOk<T> {
    data: T,
}

async fn handler_info(axum::extract::State(state): axum::extract::State<HttpState>) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({
            "version": crate::config::VERSION,
            "status": "ok",
            "kaspa_rest_base_url": state.kaspa_rest_base_url,
        }),
    })
}

async fn handler_krc20_tokenlist() -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({ "items": [], "hasMore": false }),
    })
}

async fn handler_krc20_token(
    axum::extract::Path((_tick,)): axum::extract::Path<(String,)>,
) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({}),
    })
}

async fn handler_krc20_address_tokenlist(
    axum::extract::Path((_address,)): axum::extract::Path<(String,)>,
) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({ "items": [], "hasMore": false }),
    })
}

async fn handler_krc20_address_token(
    axum::extract::Path((_address, _tick)): axum::extract::Path<(String, String)>,
) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({}),
    })
}

async fn handler_krc20_oplist() -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({ "items": [], "hasMore": false }),
    })
}

async fn handler_krc20_op(
    axum::extract::Path((_id,)): axum::extract::Path<(String,)>,
) -> axum::response::Result<axum::Json<ApiOk<serde_json::Value>>> {
    Ok(axum::Json(ApiOk {
        data: serde_json::json!({}),
    }))
}

async fn handler_archive_vspc(
    axum::extract::Path((_daa_score,)): axum::extract::Path<(u64,)>,
) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({ "vspc": [] }),
    })
}

async fn handler_archive_oplist(
    axum::extract::Path((_oprange,)): axum::extract::Path<(String,)>,
) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({ "items": [] }),
    })
}

async fn handler_krc20_market(
    axum::extract::Path((_tick,)): axum::extract::Path<(String,)>,
) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({ "items": [], "hasMore": false }),
    })
}

async fn handler_krc20_blacklist(
    axum::extract::Path((_ca,)): axum::extract::Path<(String,)>,
) -> axum::Json<ApiOk<serde_json::Value>> {
    axum::Json(ApiOk {
        data: serde_json::json!({ "blacklisted": false }),
    })
}
