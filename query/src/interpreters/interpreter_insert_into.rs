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

use std::sync::Arc;

use common_exception::Result;
use common_planners::InsertIntoPlan;
use common_streams::DataBlockStream;
use common_streams::SendableDataBlockStream;

use crate::interpreters::Interpreter;
use crate::interpreters::InterpreterPtr;
use crate::sessions::DatabendQueryContextRef;

pub struct InsertIntoInterpreter {
    ctx: DatabendQueryContextRef,
    plan: InsertIntoPlan,
}

impl InsertIntoInterpreter {
    pub fn try_create(
        ctx: DatabendQueryContextRef,
        plan: InsertIntoPlan,
    ) -> Result<InterpreterPtr> {
        Ok(Arc::new(InsertIntoInterpreter { ctx, plan }))
    }
}

#[async_trait::async_trait]
impl Interpreter for InsertIntoInterpreter {
    fn name(&self) -> &str {
        "InsertIntoInterpreter"
    }

    async fn execute(&self) -> Result<SendableDataBlockStream> {
        let table = self
            .ctx
            .get_table(&self.plan.db_name, &self.plan.tbl_name)?;

        let io_ctx = self.ctx.get_cluster_table_io_context()?;
        let io_ctx = Arc::new(io_ctx);

        table.append_data(io_ctx, self.plan.clone()).await?;
        Ok(Box::pin(DataBlockStream::create(
            self.plan.schema(),
            None,
            vec![],
        )))
    }
}
