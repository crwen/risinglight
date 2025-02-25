// Copyright 2024 RisingLight Project Authors. Licensed under Apache-2.0.

use std::sync::Arc;

use super::*;
use crate::binder::CreateTable;
use crate::storage::Storage;

/// The executor of `create table` statement.
pub struct CreateTableExecutor<S: Storage> {
    pub plan: CreateTable,
    pub storage: Arc<S>,
}

impl<S: Storage> CreateTableExecutor<S> {
    #[try_stream(boxed, ok = DataChunk, error = ExecutorError)]
    pub async fn execute(self) {
        self.storage
            .create_table(
                self.plan.schema_id,
                &self.plan.table_name,
                &self.plan.columns,
                &self.plan.ordered_pk_ids,
            )
            .await?;

        let chunk = DataChunk::single(1);
        yield chunk
    }
}
