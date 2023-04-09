pub mod email;
pub mod group;
pub mod subscriber;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use clap::Parser;
    use sqlx::PgPool;

    use crate::{
        config::AppConfig,
        repository::{
            group::{DynGroupRepositoryTrait, GroupRepository},
            subcriber::{DynSubscriberRepositoryTrait, SubscriberRepository},
        },
        service::{
            email::{DynEmailServiceTrait, EmailService},
            group::{DynGroupServiceTrait, GroupService},
            subscriber::{DynSubscriberServiceTrait, SubscriberService},
        },
    };

    struct AllTraits {
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
        subscriber_service: DynSubscriberServiceTrait,
        group_service: DynGroupServiceTrait,
        email_service: DynEmailServiceTrait,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let config = Arc::new(AppConfig::parse());

        let subscriber_repository =
            Arc::new(SubscriberRepository::new(pool.clone())) as DynSubscriberRepositoryTrait;
        let group_repository =
            Arc::new(GroupRepository::new(pool.clone())) as DynGroupRepositoryTrait;
        let subscriber_service = Arc::new(SubscriberService::new(
            subscriber_repository.clone(),
            group_repository.clone(),
        )) as DynSubscriberServiceTrait;
        let group_service =
            Arc::new(GroupService::new(group_repository.clone())) as DynGroupServiceTrait;
        let email_service = Arc::new(EmailService::new(&config)) as DynEmailServiceTrait;

        AllTraits {
            subscriber_repository,
            subscriber_service,
            group_repository,
            group_service,
            email_service,
        }
    }

    #[sqlx::test]
    async fn add_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";
        let group_description = "group_description";

        traits
            .group_service
            .add_group(group_name.to_string(), group_description.to_string())
            .await?;

        let group = traits.group_repository.get_group(group_name).await?;

        assert_eq!(group.unwrap().description, group_description);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_to_remove_name = "group_to_remove";
        let group_to_remove_description = "group_to_remove_description";
        traits
            .group_repository
            .add_group("group1", "description1")
            .await?;
        traits
            .group_repository
            .add_group(group_to_remove_name, group_to_remove_description)
            .await?;

        let removed_group = traits
            .group_service
            .remove_group(group_to_remove_name.to_string())
            .await?;

        let groups_list = traits.group_repository.list_groups().await?;

        assert_eq!(groups_list.len(), 1);
        assert_eq!(
            removed_group.unwrap().description,
            group_to_remove_description
        );

        Ok(())
    }

    #[sqlx::test]
    async fn list_groups_by_sub_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group1_name = "group1_name";
        let group1_description = "group1_description";
        let group2_name = "group2_name";
        let group2_description = "group2_description";
        let group1 = traits
            .group_repository
            .add_group(group1_name, group1_description)
            .await?;
        traits
            .group_repository
            .add_group(group2_name, group2_description)
            .await?;

        let subscriber_email = "subscriber_email";
        traits
            .subscriber_repository
            .add_subscriber(subscriber_email, &group1)
            .await?;

        let groups_list = traits
            .group_service
            .list_groups_by_sub(subscriber_email.to_string())
            .await?;

        assert_eq!(groups_list.len(), 1);
        assert_eq!(groups_list.first().unwrap().description, group1_description);

        Ok(())
    }

    #[sqlx::test]
    async fn list_subs_by_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group1_name = "group1_name";
        let group1 = traits
            .group_repository
            .add_group(group1_name, "group1_description")
            .await?;
        let group2 = traits
            .group_repository
            .add_group("group2_name", "group2_description")
            .await?;

        let sub1_email = "sub1_email";
        traits
            .subscriber_repository
            .add_subscriber(sub1_email, &group1)
            .await?;
        let sub2_email = "sub2_email";
        traits
            .subscriber_repository
            .add_subscriber(sub2_email, &group2)
            .await?;

        let subs_list = traits
            .subscriber_service
            .list_subs_by_group(group1_name.to_string())
            .await?;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().email, sub1_email);

        Ok(())
    }

    #[sqlx::test]
    async fn add_subscriber_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";
        let group_description = "group_description";
        traits
            .group_repository
            .add_group(group_name, group_description)
            .await?;

        let sub_email = "sub_email";
        traits
            .subscriber_service
            .add_subscriber(sub_email.to_string(), group_name.to_string())
            .await?;

        let added_sub = traits
            .group_repository
            .list_groups_by_sub(sub_email)
            .await?;
        assert_eq!(added_sub.first().unwrap().description, group_description);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_subcriber_from_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";
        let group = traits
            .group_repository
            .add_group(group_name, "group_description")
            .await?;

        let sub1_email = "sub1_email";
        traits
            .subscriber_repository
            .add_subscriber(sub1_email, &group)
            .await?;
        let sub2_email = "sub2_email";
        traits
            .subscriber_repository
            .add_subscriber(sub2_email, &group)
            .await?;

        traits
            .subscriber_service
            .remove_subscriber_from_group(sub1_email.to_string(), group_name.to_string())
            .await?;
        let subs_list = traits
            .subscriber_repository
            .list_subs_by_group(&group)
            .await?;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().email, sub2_email);

        Ok(())
    }

    #[sqlx::test]
    async fn send_email_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        traits
            .email_service
            .send_email(
                "email@test.com".to_string(),
                "hello".to_string(),
                "this is a test".to_string(),
            )
            .await?;

        Ok(())
    }

    #[sqlx::test]
    #[ignore]
    async fn blast_email_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";
        let group = traits
            .group_repository
            .add_group(group_name, "group_description")
            .await?;

        let sub1_email = "sub1_email";
        traits
            .subscriber_repository
            .add_subscriber(sub1_email, &group)
            .await?;
        let sub2_email = "sub2_email";
        traits
            .subscriber_repository
            .add_subscriber(sub2_email, &group)
            .await?;

        let subs_list = traits
            .subscriber_repository
            .list_subs_by_group(&group)
            .await?;
        let email_list = subs_list
            .iter()
            .map(|entity| entity.email.clone())
            .collect::<Vec<String>>();

        traits
            .email_service
            .blast_email(
                email_list,
                "hello".to_string(),
                "this is a test".to_string(),
            )
            .await?;

        Ok(())
    }
}
