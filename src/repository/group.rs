use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use madtofan_microservice_common::repository::connection_pool::ServiceConnectionPool;
use mockall::automock;
use sqlx::{query, query_as, types::time::OffsetDateTime, FromRow};

use madtofan_microservice_common::email::groups_response::Group;

#[derive(FromRow)]
pub struct GroupEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub name: String,
    pub description: String,
}

impl GroupEntity {
    pub fn into_group_response(self) -> Group {
        Group {
            name: self.name,
            description: self.description,
        }
    }
}

#[automock]
#[async_trait]
pub trait GroupRepositoryTrait {
    async fn list_groups(&self) -> anyhow::Result<Vec<GroupEntity>>;
    async fn list_groups_by_sub(
        &self,
        email: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> anyhow::Result<Vec<GroupEntity>>;
    async fn get_groups_by_sub_count(&self, email: &str) -> anyhow::Result<i64>;
    async fn get_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>>;
    async fn add_group(&self, name: &str, description: &str) -> anyhow::Result<GroupEntity>;
    async fn remove_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>>;
}

pub type DynGroupRepositoryTrait = Arc<dyn GroupRepositoryTrait + Send + Sync>;

#[derive(Clone)]
pub struct GroupRepository {
    pool: ServiceConnectionPool,
}

impl GroupRepository {
    pub fn new(pool: ServiceConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepositoryTrait for GroupRepository {
    async fn list_groups(&self) -> anyhow::Result<Vec<GroupEntity>> {
        query_as!(
            GroupEntity,
            r#"
                select
                    id,
                    name,
                    description,
                    created_at,
                    updated_at
                from subscription_group
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while obtaining for group list")
    }

    async fn list_groups_by_sub(
        &self,
        email: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> anyhow::Result<Vec<GroupEntity>> {
        let limit_string = format!(" limit {}", limit.unwrap_or_default());
        let offset_string = format!(" offset {}", offset.unwrap_or_default());
        let query_string = format!(
            r#"
                select
                    sg.id as id,
                    sg.name as name,
                    sg.description as description,
                    sg.created_at as created_at,
                    sg.updated_at as updated_at
                from subscription_group as sg
                join subscriber as s
                on sg.id = s.group_id
                where s.email = '{}'
                {}
                {}
            "#,
            email,
            match limit {
                Some(_) => &limit_string,
                None => "",
            },
            match offset {
                Some(_) => &offset_string,
                None => "",
            }
        );

        sqlx::query_as::<_, GroupEntity>(&query_string)
            .fetch_all(&self.pool)
            .await
            .context("an unexpected error occured while obtaining for group list")
    }

    async fn get_groups_by_sub_count(&self, email: &str) -> anyhow::Result<i64> {
        let count_result = query!(
            r#"
                select
                    count(*)
                from subscriber
                where email = $1::varchar
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count_result.count.unwrap())
    }

    async fn get_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>> {
        query_as!(
            GroupEntity,
            r#"
                select
                    id,
                    name,
                    description,
                    created_at,
                    updated_at
                from subscription_group
                where name = $1::varchar
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while searching for group")
    }

    async fn add_group(&self, name: &str, description: &str) -> anyhow::Result<GroupEntity> {
        query_as!(
            GroupEntity,
            r#"
                insert into subscription_group (
                        name,
                        description
                    )
                values (
                        $1::varchar,
                        $2::varchar
                    )
                returning *
            "#,
            name,
            description,
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating the subscription group")
    }

    async fn remove_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>> {
        query_as!(
            GroupEntity,
            r#"
                delete from subscription_group 
                where name = $1::varchar
                returning *
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while removing the subscription group")
    }
}
