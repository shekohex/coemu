#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use tq_externalities::Externalities;

pub struct DefaultExtenernalities {
    pool: sqlx::AnyPool,
}

impl DefaultExtenernalities {
    pub fn new(pool: sqlx::AnyPool) -> Self { Self { pool } }
}

impl Externalities for DefaultExtenernalities {
    fn executer(&self) -> &sqlx::AnyPool { &self.pool }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(42)
            .min_connections(4)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let mut ext = DefaultExtenernalities::new(pool);

        async fn add_one(x: i32) -> i32 {
            tq_externalities::with_externalities(|ext| {
                sqlx::query("select 1 + ?1")
                    .bind(x)
                    .try_map(|row: sqlx::any::AnyRow| row.try_get::<i32, _>(0))
                    .fetch_one(ext.executer())
            })
            .unwrap()
            .await
            .unwrap()
        }

        let p =
            tq_externalities::set_and_run_with_externalities(&mut ext, || {});
    }
}
