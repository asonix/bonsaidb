use std::sync::Arc;

use async_trait::async_trait;
pub use bonsaidb_core::circulate::Relay;
use bonsaidb_core::{
    circulate,
    permissions::bonsai::{
        database_resource_name, pubsub_topic_resource_name, BonsaiAction, DatabaseAction,
        PubSubAction,
    },
    pubsub::{self, database_topic, PubSub},
    Error,
};

use crate::{backend, Database, DatabaseNonBlocking};

impl<Backend: backend::Backend> PubSub for super::Database<Backend> {
    type Subscriber = Subscriber<Backend>;

    fn create_subscriber(&self) -> Result<Self::Subscriber, bonsaidb_core::Error> {
        self.check_permission(
            database_resource_name(self.name()),
            &BonsaiAction::Database(DatabaseAction::PubSub(PubSubAction::CreateSuscriber)),
        )?;
        Ok(Subscriber {
            database: self.clone(),
            subscriber: self.data.storage.instance.relay().create_subscriber(),
        })
    }

    fn publish<S: Into<String> + Send, P: serde::Serialize + Sync>(
        &self,
        topic: S,
        payload: &P,
    ) -> Result<(), bonsaidb_core::Error> {
        let topic = topic.into();
        self.check_permission(
            pubsub_topic_resource_name(self.name(), &topic),
            &BonsaiAction::Database(DatabaseAction::PubSub(PubSubAction::Publish)),
        )?;
        self.data
            .storage
            .instance
            .relay()
            .publish(database_topic(&self.data.name, &topic), payload)?;
        Ok(())
    }

    fn publish_to_all<P: serde::Serialize + Sync>(
        &self,
        topics: Vec<String>,
        payload: &P,
    ) -> Result<(), bonsaidb_core::Error> {
        self.data.storage.instance.relay().publish_to_all(
            topics
                .iter()
                .map(|topic| {
                    self.check_permission(
                        pubsub_topic_resource_name(self.name(), topic),
                        &BonsaiAction::Database(DatabaseAction::PubSub(PubSubAction::Publish)),
                    )
                    .map(|_| database_topic(&self.data.name, topic))
                })
                .collect::<Result<_, _>>()?,
            payload,
        )?;
        Ok(())
    }

    fn publish_bytes<S: Into<String> + Send>(
        &self,
        topic: S,
        payload: Vec<u8>,
    ) -> Result<(), bonsaidb_core::Error> {
        let topic = topic.into();
        self.check_permission(
            pubsub_topic_resource_name(self.name(), &topic),
            &BonsaiAction::Database(DatabaseAction::PubSub(PubSubAction::Publish)),
        )?;
        self.data
            .storage
            .instance
            .relay()
            .publish_raw(database_topic(&self.data.name, &topic), payload);
        Ok(())
    }

    fn publish_bytes_to_all(
        &self,
        topics: Vec<String>,
        payload: Vec<u8>,
    ) -> Result<(), bonsaidb_core::Error> {
        self.data.storage.instance.relay().publish_raw_to_all(
            topics
                .iter()
                .map(|topic| {
                    self.check_permission(
                        pubsub_topic_resource_name(self.name(), topic),
                        &BonsaiAction::Database(DatabaseAction::PubSub(PubSubAction::Publish)),
                    )
                    .map(|_| database_topic(&self.data.name, topic))
                })
                .collect::<Result<_, _>>()?,
            payload,
        );
        Ok(())
    }
}

/// A subscriber for `PubSub` messages.
pub struct Subscriber<Backend: backend::Backend> {
    database: Database<Backend>,
    subscriber: circulate::Subscriber,
}

#[async_trait]
impl<Backend: backend::Backend> pubsub::Subscriber for Subscriber<Backend> {
    fn subscribe_to<S: Into<String> + Send>(&self, topic: S) -> Result<(), Error> {
        let topic = topic.into();
        self.database.check_permission(
            pubsub_topic_resource_name(self.database.name(), &topic),
            &BonsaiAction::Database(DatabaseAction::PubSub(PubSubAction::SubscribeTo)),
        )?;
        self.subscriber
            .subscribe_to(database_topic(self.database.name(), &topic));
        Ok(())
    }

    fn unsubscribe_from(&self, topic: &str) -> Result<(), Error> {
        self.database.check_permission(
            pubsub_topic_resource_name(self.database.name(), topic),
            &BonsaiAction::Database(DatabaseAction::PubSub(PubSubAction::UnsubscribeFrom)),
        )?;
        self.subscriber
            .unsubscribe_from(&database_topic(self.database.name(), topic));
        Ok(())
    }

    fn receiver(&self) -> &'_ flume::Receiver<Arc<circulate::Message>> {
        self.subscriber.receiver()
    }
}
