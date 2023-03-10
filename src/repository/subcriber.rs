use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use common::repository::connection_pool::ServiceConnectionPool;
use mockall::automock;
use sqlx::{query_as, types::time::OffsetDateTime, FromRow};

use crate::email::subscribers_response::Subscriber;

use super::group::GroupEntity;

#[derive(FromRow)]
pub struct SubscriberEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub email: String,
    pub group_id: i64,
}

impl SubscriberEntity {
    pub fn into_subscriber_response(self) -> Subscriber {
        Subscriber { email: self.email }
    }
}

#[automock]
#[async_trait]
pub trait SubscriberRepositoryTrait {
    async fn list_sub_by_group(&self, group: GroupEntity) -> anyhow::Result<Vec<SubscriberEntity>>;
    async fn add_subscriber(
        &self,
        email: &str,
        group: GroupEntity,
    ) -> anyhow::Result<SubscriberEntity>;
    async fn remove_subscriber_from_group(
        &self,
        email: &str,
        group: GroupEntity,
    ) -> anyhow::Result<Option<SubscriberEntity>>;
}

pub type DynSubscriberRepositoryTrait = Arc<dyn SubscriberRepositoryTrait + Send + Sync>;

#[derive(Clone)]
pub struct SubscriberRepository {
    pool: ServiceConnectionPool,
}

impl SubscriberRepository {
    pub fn new(pool: ServiceConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriberRepositoryTrait for SubscriberRepository {
    async fn list_sub_by_group(&self, group: GroupEntity) -> anyhow::Result<Vec<SubscriberEntity>> {
        query_as!(
            SubscriberEntity,
            r#"
                select
                    id,
                    email,
                    group_id,
                    created_at,
                    updated_at
                from subscriber
                where group_id = $1::bigint
            "#,
            group.id
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while search for subscribers by group")
    }

    async fn add_subscriber(
        &self,
        email: &str,
        group: GroupEntity,
    ) -> anyhow::Result<SubscriberEntity> {
        query_as!(
            SubscriberEntity,
            r#"
                insert into subscriber (
                        email,
                        group_id,
                        created_at,
                        updated_at
                    )
                values (
                        $1::varchar,
                        $2::bigint,
                        current_timestamp,
                        current_timestamp
                    )
                returning *
            "#,
            email,
            group.id,
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating the subscriber")
    }

    async fn remove_subscriber_from_group(
        &self,
        email: &str,
        group: GroupEntity,
    ) -> anyhow::Result<Option<SubscriberEntity>> {
        query_as!(
            SubscriberEntity,
            r#"
                delete from subscriber 
                where 
                    email = $1::varchar 
                    and group_id = $2::bigint
                returning *
            "#,
            email,
            group.id,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while removing the subscriber")
    }
}
