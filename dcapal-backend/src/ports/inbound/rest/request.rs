use crate::app::infra::claim::Claims;
use crate::ports::inbound::rest::FeeStructure;
use crate::{AppContext, DateTime};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncPortfoliosRequest {
    pub portfolios: Vec<PortfolioRequest>,
    pub deleted_portfolios: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioRequest {
    pub id: Uuid,
    pub name: String,
    pub quote_ccy: String,
    pub fees: Option<TransactionFeesRequest>,
    pub assets: Vec<PortfolioAssetRequest>,
    pub last_updated_at: DateTime,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioAssetRequest {
    pub symbol: String,
    pub name: String,
    pub aclass: String,
    pub base_ccy: String,
    pub provider: String,
    pub qty: Decimal,
    pub target_weight: Decimal,
    pub price: Decimal,
    pub fees: Option<TransactionFeesRequest>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFeesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_impact: Option<Decimal>,
    pub fee_structure: FeeStructure,
}

pub async fn sync_portfolios(
    State(ctx): State<AppContext>,
    claims: Claims,
    Json(req): Json<SyncPortfoliosRequest>,
) -> crate::error::Result<Response> {
    info!("Syncing portfolios for user_id: {}.", claims.sub);
    match &ctx
        .services
        .portfolio
        .sync_portfolios(claims.sub, req)
        .await
    {
        Ok(resp) => {
            info!(
                "Successfully synced portfolios for user_id: {}.",
                claims.sub
            );
            Ok(Json(resp).into_response())
        }
        Err(e) => {
            error!("Failed to sync portfolios: {} due to: {}.", claims.sub, e);
            Ok(StatusCode::BAD_REQUEST.into_response())
        }
    }
}

#[test]
fn test_fee_structure_deserialization() {
    let json = r#"{
        "feeStructure": {
            "type": "variable",
            "feeRate": 0.19,
            "minFee": 2.95
        },
        "maxFeeImpact": 0.5
    }"#;

    let fees: TransactionFeesRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(fees.fee_structure, FeeStructure::Variable { .. }));
}
