use sqlx::PgPool;

#[derive(Clone)]
pub struct MyState {
    pub pool: PgPool,
}
