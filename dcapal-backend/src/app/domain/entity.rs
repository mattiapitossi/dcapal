use crate::app::infra::utils::Expiring;
use crate::DateTime;
use bigdecimal::BigDecimal;
use chrono::{Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use time::Date;
use uuid::Uuid;

pub type AssetId = String;
pub type MarketId = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Crypto {
    pub id: AssetId,
    pub symbol: String,
}

impl Crypto {
    pub fn new_with_id(id: AssetId) -> Self {
        let symbol = id.to_uppercase();
        Self { id, symbol }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fiat {
    pub id: AssetId,
    pub symbol: String,
}

impl Fiat {
    pub fn new(id: AssetId, symbol: String) -> Self {
        Self { id, symbol }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Asset {
    Crypto(Crypto),
    Fiat(Fiat),
}

#[derive(Debug, Clone, Copy, strum_macros::Display)]
pub enum AssetKind {
    Crypto,
    Fiat,
}

impl Asset {
    pub fn id(&self) -> &AssetId {
        match self {
            Asset::Crypto(a) => &a.id,
            Asset::Fiat(a) => &a.id,
        }
    }

    pub fn is_fiat(&self) -> bool {
        matches!(self, Asset::Fiat(_))
    }

    pub fn kind(&self) -> AssetKind {
        match self {
            Asset::Crypto(_) => AssetKind::Crypto,
            Asset::Fiat(_) => AssetKind::Fiat,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Price {
    pub price: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime,
}

impl Price {
    const VALIDITY_MINS: u32 = 5;

    pub fn new(price: f64, ts: DateTime) -> Self {
        Self { price, ts }
    }
}

impl Expiring for Price {
    fn is_outdated(&self) -> bool {
        let now = Utc::now();
        if now.date_naive() > self.ts.date_naive() || now.time().hour() > self.ts.time().hour() {
            return true;
        }

        let now_mins = now.time().minute();
        let ts_mins = self.ts.time().minute();

        let now_validity_range = (now_mins / Self::VALIDITY_MINS) * Self::VALIDITY_MINS;
        let ts_validity_range = (ts_mins / Self::VALIDITY_MINS) * Self::VALIDITY_MINS;

        now_validity_range > ts_validity_range
    }

    fn time_to_live(&self) -> std::time::Duration {
        (self.ts + Duration::minutes(Self::VALIDITY_MINS as i64) - Utc::now())
            .to_std()
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
    }
}

#[derive(Debug, Clone, Copy, strum_macros::Display)]
pub enum OHLCFrequency {
    Minutes5,
    Daily,
}

impl OHLCFrequency {
    pub fn ohlc_range(&self, ts: DateTime) -> (DateTime, DateTime) {
        match self {
            OHLCFrequency::Minutes5 => Self::ohlc_range_minutes_5(ts),
            OHLCFrequency::Daily => Self::ohlc_range_daily(ts),
        }
    }

    fn ohlc_range_minutes_5(ts: DateTime) -> (DateTime, DateTime) {
        static MINS_5: u32 = 5;
        static N_PERIODS: i64 = 12;

        let ts_mins = ts.time().minute();
        let range_upper_mins = (ts_mins / MINS_5) * MINS_5;
        let range_upper = ts
            .with_minute(range_upper_mins)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        let range_low = range_upper - Duration::minutes(N_PERIODS * MINS_5 as i64);

        (range_low, range_upper)
    }

    fn ohlc_range_daily(ts: DateTime) -> (DateTime, DateTime) {
        static N_DAYS: i64 = 1;

        let ts_start_day = ts
            .with_hour(0)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap();

        let range_low = ts_start_day - Duration::days(N_DAYS);

        (range_low, ts)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Market {
    pub id: MarketId,
    pub pair: String,
    pub base: Asset,
    pub quote: Asset,
    #[serde(flatten)]
    price: Option<Price>,
}

impl Market {
    pub fn new(id: MarketId, base: Asset, quote: Asset, price: Option<Price>) -> Self {
        Self {
            id,
            pair: format!("{}/{}", base.id().to_uppercase(), quote.id().to_uppercase()),
            base,
            quote,
            price,
        }
    }

    pub fn price(&self) -> &Option<Price> {
        &self.price
    }

    pub fn set_price(&mut self, price: Price) {
        self.price.replace(price);
    }

    pub fn is_fiat(&self) -> bool {
        self.base.is_fiat() && self.quote.is_fiat()
    }

    pub fn is_price_outdated(&self) -> bool {
        let last_price = self.price();
        last_price.is_some() && last_price.as_ref().unwrap().is_outdated()
    }
}

impl Expiring for Market {
    fn is_outdated(&self) -> bool {
        self.is_price_outdated()
    }

    fn time_to_live(&self) -> std::time::Duration {
        self.price()
            .as_ref()
            .map(|p| p.time_to_live())
            .unwrap_or_else(|| std::time::Duration::from_secs(0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub name: Option<String>,
    pub email: String,
    pub birth_date: Option<Date>,
}
#[derive(sqlx::Type, Clone, Debug, Deserialize, Serialize)]
#[sqlx(type_name = "risk_tolerance", rename_all = "snake_case")]
pub enum RiskTolerance {
    Low,
    Medium,
    High,
}

impl FromStr for RiskTolerance {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Low" => Ok(RiskTolerance::Low),
            "Medium" => Ok(RiskTolerance::Medium),
            "High" => Ok(RiskTolerance::High),
            _ => Err(()),
        }
    }
}

impl fmt::Display for RiskTolerance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RiskTolerance::Low => "Low",
                RiskTolerance::Medium => "Medium",
                RiskTolerance::High => "High",
            }
        )
    }
}

#[derive(sqlx::Type, Clone, Debug, Deserialize, Serialize)]
#[sqlx(type_name = "investment_mode", rename_all = "snake_case")]
pub enum InvestmentMode {
    Standard,
    Expert,
}

impl FromStr for InvestmentMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Standard" => Ok(InvestmentMode::Standard),
            "Expert" => Ok(InvestmentMode::Expert),
            _ => Err(()),
        }
    }
}

impl fmt::Display for InvestmentMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InvestmentMode::Standard => "Standard",
                InvestmentMode::Expert => "Expert",
            }
        )
    }
}

#[derive(sqlx::Type, Clone, Debug, Deserialize, Serialize)]
#[sqlx(type_name = "investment_goal", rename_all = "snake_case")]
pub enum InvestmentGoal {
    Retirement,
    Education,
    WealthBuilding,
    Other,
}

impl FromStr for InvestmentGoal {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Retirement" => Ok(InvestmentGoal::Retirement),
            "Education" => Ok(InvestmentGoal::Education),
            "Wealth Building" => Ok(InvestmentGoal::WealthBuilding),
            "Other" => Ok(InvestmentGoal::Other),
            _ => Err(()),
        }
    }
}

impl fmt::Display for InvestmentGoal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InvestmentGoal::Retirement => "Retirement",
                InvestmentGoal::Education => "Education",
                InvestmentGoal::WealthBuilding => "Wealth Building",
                InvestmentGoal::Other => "Other",
            }
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvestmentPreferences {
    pub risk_tolerance: RiskTolerance,
    pub investment_horizon: i32,
    pub investment_mode: InvestmentMode,
    pub investment_goal: InvestmentGoal,
    pub ai_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ai {
    pub response: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Portfolio {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub assets: Vec<PortfolioHoldings>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortfolioHoldings {
    pub symbol: String,
    pub name: String,
    pub quantity: BigDecimal,
    pub price: BigDecimal,
    pub average_buy_price: BigDecimal,
    pub weight: BigDecimal,
    pub total: BigDecimal,
}
