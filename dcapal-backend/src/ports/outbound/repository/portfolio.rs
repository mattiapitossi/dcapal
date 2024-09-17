use crate::app::domain::entity::{Portfolio, PortfolioHoldings};
use crate::error::Result;
use bigdecimal::BigDecimal;
use uuid::Uuid;

#[derive(Clone)]
pub struct PortfolioRepository {
    pub postgres: sqlx::PgPool,
}

impl PortfolioRepository {
    pub fn new(postgres: sqlx::PgPool) -> Self {
        Self { postgres }
    }

    pub async fn save(&self, user_id: Uuid, portfolio: Portfolio) -> Result<Portfolio> {
        let mut tx = self.postgres.begin().await?;

        // Insert the portfolio
        let portfolio_row = sqlx::query!(
            r#"
        INSERT INTO "portfolios" (id, user_id, name, description, currency)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, name, description, currency
        "#,
            portfolio.id,
            user_id,
            portfolio.name,
            portfolio.description,
            portfolio.currency
        )
        .fetch_one(&mut *tx)
        .await?;

        // Insert the holdings
        for holding in &portfolio.assets {
            sqlx::query!(
                r#"
            INSERT INTO "portfolio_holdings" (portfolio_id, symbol, name, quantity, weight, total, price, average_buy_price)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
                portfolio_row.id,
                holding.symbol,
                holding.name,
                holding.quantity,
                holding.weight,
                holding.total,
                holding.price,
                holding.average_buy_price
            )
            .execute(&mut *tx)
            .await?;
        }

        // Commit the transaction
        tx.commit().await?;

        // Construct and return the Portfolio struct
        Ok(Portfolio {
            id: portfolio_row.id,
            name: portfolio_row.name,
            description: portfolio_row.description,
            currency: portfolio_row.currency,
            assets: portfolio.assets,
        })
    }

    pub async fn find_all(&self, user_id: Uuid) -> Result<Vec<Portfolio>> {
        let mut tx = self.postgres.begin().await?;
        let portfolios = sqlx::query!(
            r#"
        SELECT id, name, description, currency
        FROM "portfolios"
        WHERE user_id = $1
        "#,
            user_id
        )
        .fetch_all(&mut *tx)
        .await?;

        let mut result = Vec::new();
        for portfolio in portfolios {
            let holdings = sqlx::query!(
                r#"
                SELECT 
                    symbol, 
                    quantity,
                    average_buy_price,
                    name,
                    price,
                    weight,
                    total
                FROM "portfolio_holdings"
                WHERE portfolio_id = $1
                "#,
                portfolio.id
            )
            .fetch_all(&mut *tx)
            .await?;

            let assets = holdings
                .into_iter()
                .map(|holding| PortfolioHoldings {
                    symbol: holding.symbol,
                    quantity: holding.quantity,
                    price: holding.price,
                    average_buy_price: holding.average_buy_price,
                    weight: holding.weight,
                    name: holding.name,
                    total: BigDecimal::from(0), //TODO: set the proper total
                })
                .collect();

            result.push(Portfolio {
                id: portfolio.id,
                name: portfolio.name,
                description: portfolio.description,
                currency: portfolio.currency,
                assets,
            });
        }

        tx.commit().await?;
        Ok(result)
    }
}
