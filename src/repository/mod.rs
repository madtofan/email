pub mod group;
pub mod subcriber;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use sqlx::PgPool;

    use crate::repository::{
        group::{DynGroupRepositoryTrait, GroupRepository},
        subcriber::DynSubscriberRepositoryTrait,
    };

    use super::subcriber::SubscriberRepository;

    struct AllTraits {
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let subscriber_repository =
            Arc::new(SubscriberRepository::new(pool.clone())) as DynSubscriberRepositoryTrait;
        let group_repository =
            Arc::new(GroupRepository::new(pool.clone())) as DynGroupRepositoryTrait;

        AllTraits {
            subscriber_repository,
            group_repository,
        }
    }

    #[sqlx::test]
    async fn remove_and_list_group_test(pool: PgPool) -> anyhow::Result<()> {
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
            .group_repository
            .remove_group(group_to_remove_name)
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
    async fn get_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_to_get_name = "group_to_get";
        let group_to_get_description = "group_to_get_description";
        traits
            .group_repository
            .add_group(group_to_get_name, group_to_get_description)
            .await?;

        let get_group = traits.group_repository.get_group(group_to_get_name).await?;

        assert_eq!(get_group.unwrap().description, group_to_get_description);

        Ok(())
    }

    #[sqlx::test]
    async fn list_groups_by_sub_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_to_subscribe_name = "group_to_subscribe_name";
        let group_to_subscribe_description = "group_to_subscribe_description";

        let group_to_subscribe = traits
            .group_repository
            .add_group(group_to_subscribe_name, group_to_subscribe_description)
            .await?;
        traits
            .group_repository
            .add_group("group1", "description1")
            .await?;

        let subscriber_email = "address@email.com";
        traits
            .subscriber_repository
            .add_subscriber(subscriber_email, &group_to_subscribe)
            .await?;

        let listed_group = traits
            .group_repository
            .list_groups_by_sub(subscriber_email, Some(0), Some(10))
            .await?;

        assert_eq!(listed_group.len(), 1);
        assert_eq!(
            listed_group.first().unwrap().description,
            group_to_subscribe_description
        );

        Ok(())
    }

    #[sqlx::test]
    async fn list_subs_by_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_1_name = "group_1_name";
        let group_1_description = "group_1_description";
        let group1 = traits
            .group_repository
            .add_group(group_1_name, group_1_description)
            .await?;

        let group_2_name = "group_2_name";
        let group_2_description = "group_2_description";
        let group2 = traits
            .group_repository
            .add_group(group_2_name, group_2_description)
            .await?;

        let sub_1_address = "sub_1_address@email.com";
        let sub1 = traits
            .subscriber_repository
            .add_subscriber(sub_1_address, &group1)
            .await?;

        let sub_2_address = "sub_2_address@email.com";
        traits
            .subscriber_repository
            .add_subscriber(sub_2_address, &group2)
            .await?;

        let subscribers_list = traits
            .subscriber_repository
            .list_subs_by_group(&group1, None, None)
            .await?;

        assert_eq!(subscribers_list.len(), 1);
        assert_eq!(subscribers_list.first().unwrap().email, sub1.email);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_subscriber_from_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";
        let group_description = "group_description";
        let group = traits
            .group_repository
            .add_group(group_name, group_description)
            .await?;

        let sub_1_address = "sub_1_address@email.com";
        traits
            .subscriber_repository
            .add_subscriber(sub_1_address, &group)
            .await?;

        let sub_2_address = "sub_2_address@email.com";
        let sub2 = traits
            .subscriber_repository
            .add_subscriber(sub_2_address, &group)
            .await?;

        traits
            .subscriber_repository
            .remove_subscriber_from_group(sub_1_address, &group)
            .await?;

        let subscribers_list = traits
            .subscriber_repository
            .list_subs_by_group(&group, None, None)
            .await?;

        assert_eq!(subscribers_list.len(), 1);
        assert_eq!(subscribers_list.first().unwrap().email, sub2.email);

        Ok(())
    }
}
