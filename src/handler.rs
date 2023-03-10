use tonic::{Request, Response, Status};

use crate::{
    email::{
        email_server::Email, groups_response::Group, subscribers_response::Subscriber,
        AddGroupRequest, AddSubscriberRequest, BlastEmailRequest, EmailResponse,
        GetSubscriberGroupsRequest, GetSubscribersRequest, GroupsResponse, RemoveGroupRequest,
        RemoveSubscriberRequest, SendEmailRequest, SubscribersResponse,
    },
    service::{
        email::DynEmailServiceTrait, group::DynGroupServiceTrait,
        subscriber::DynSubscriberServiceTrait,
    },
};

pub struct RequestHandler {
    subscriber_service: DynSubscriberServiceTrait,
    group_service: DynGroupServiceTrait,
    email_service: DynEmailServiceTrait,
}

impl RequestHandler {
    pub fn new(
        subscriber_service: DynSubscriberServiceTrait,
        group_service: DynGroupServiceTrait,
        email_service: DynEmailServiceTrait,
    ) -> Self {
        Self {
            subscriber_service,
            group_service,
            email_service,
        }
    }
}

#[tonic::async_trait]
impl Email for RequestHandler {
    async fn send_email(
        &self,
        request: Request<SendEmailRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.email_service
            .send_email(req.email, req.title, req.body)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Success sending email!"),
        }))
    }

    async fn blast_email(
        &self,
        request: Request<BlastEmailRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        let subscribers = self.subscriber_service.list_sub_by_group(req.group).await?;

        let addresses = subscribers
            .into_iter()
            .map(|subscriber| subscriber.email)
            .collect::<Vec<String>>();

        self.email_service
            .blast_email(addresses, req.title, req.body)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Success blasting email!"),
        }))
    }

    async fn add_subscriber(
        &self,
        request: Request<AddSubscriberRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.subscriber_service
            .add_subscriber(req.email, req.group)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully add subscriber!"),
        }))
    }

    async fn remove_subscriber(
        &self,
        request: Request<RemoveSubscriberRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.subscriber_service
            .remove_subscriber_from_group(req.email, req.group)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully removed subscriber!"),
        }))
    }

    async fn add_group(
        &self,
        request: Request<AddGroupRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.group_service
            .add_group(req.name, req.description)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully add group!"),
        }))
    }

    async fn remove_group(
        &self,
        request: Request<RemoveGroupRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.group_service.remove_group(req.name).await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully removed group!"),
        }))
    }

    async fn get_subscribers(
        &self,
        request: Request<GetSubscribersRequest>,
    ) -> Result<Response<SubscribersResponse>, Status> {
        let req = request.into_inner();

        let subscriber_entity = self.subscriber_service.list_sub_by_group(req.group).await?;

        let subscribers = subscriber_entity
            .into_iter()
            .map(|sub| sub.into_subscriber_response())
            .collect::<Vec<Subscriber>>();

        Ok(Response::new(SubscribersResponse { subscribers }))
    }

    async fn get_subscriber_groups(
        &self,
        request: Request<GetSubscriberGroupsRequest>,
    ) -> Result<Response<GroupsResponse>, Status> {
        let req = request.into_inner();

        let groups_entity = self.group_service.list_group_from_sub(req.email).await?;

        let groups = groups_entity
            .into_iter()
            .map(|group| group.into_group_response())
            .collect::<Vec<Group>>();

        Ok(Response::new(GroupsResponse { groups }))
    }
}
