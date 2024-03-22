use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use madtofan_microservice_common::{
    email::subscribers_response::Subscriber, repository::connection_pool::ServiceConnectionPool,
};
use mockall::automock;
use sqlx::{query, query_as, types::time::OffsetDateTime, FromRow};

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
    async fn list_subs_by_group(
        &self,
        group: &GroupEntity,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> anyhow::Result<Vec<SubscriberEntity>>;
    async fn get_subs_by_group_count(&self, group: &GroupEntity) -> anyhow::Result<i64>;
    async fn add_subscriber(
        &self,
        email: &str,
        group: &GroupEntity,
    ) -> anyhow::Result<SubscriberEntity>;
    async fn remove_subscriber_from_group(
        &self,
        email: &str,
        group: &GroupEntity,
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
    async fn list_subs_by_group(
        &self,
        group: &GroupEntity,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> anyhow::Result<Vec<SubscriberEntity>> {
        let limit_string = format!(" limit {}", limit.unwrap_or_default());
        let offset_string = format!(" offset {}", offset.unwrap_or_default());
        let query_string = format!(
            r#"
                select
                    id,
                    email,
                    group_id,
                    created_at,
                    updated_at
                from subscriber
                where group_id = {}
                {}
                {}
            "#,
            group.id,
            match limit {
                Some(_) => &limit_string,
                None => "",
            },
            match offset {
                Some(_) => &offset_string,
                None => "",
            }
        );
        sqlx::query_as::<_, SubscriberEntity>(&query_string)
            .fetch_all(&self.pool)
            .await
            .context("an unexpected error occured while search for subscribers by group")
    }

    async fn get_subs_by_group_count(&self, group: &GroupEntity) -> anyhow::Result<i64> {
        let count_result = query!(
            r#"
                select
                    count(*)
                from subscriber
                where group_id = $1::bigint
            "#,
            group.id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count_result.count.unwrap())
    }

    async fn add_subscriber(
        &self,
        email: &str,
        group: &GroupEntity,
    ) -> anyhow::Result<SubscriberEntity> {
        query_as!(
            SubscriberEntity,
            r#"
                insert into subscriber (
                        email,
                        group_id
                    )
                values (
                        $1::varchar,
                        $2::bigint
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
        group: &GroupEntity,
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
