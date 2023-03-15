use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use common::repository::connection_pool::ServiceConnectionPool;
use mockall::automock;
use sqlx::{query_as, types::time::OffsetDateTime, FromRow};

use crate::email::groups_response::Group;

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
    async fn list_groups_by_subcriber(&self, email: &str) -> anyhow::Result<Vec<GroupEntity>>;
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

    async fn list_groups_by_subcriber(&self, email: &str) -> anyhow::Result<Vec<GroupEntity>> {
        query_as!(
            GroupEntity,
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
                where s.email = $1::varchar
            "#,
            email
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while obtaining for group list")
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
                        description,
                        created_at,
                        updated_at
                    )
                values (
                        $1::varchar,
                        $2::varchar,
                        current_timestamp,
                        current_timestamp
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
