// from https://github.com/gmornin/services/blob/master/src/traits/collection_item.rs

use mongodb::{
    bson::{doc, Bson},
    Collection,
};
use serde::{de::DeserializeOwned, Serialize};
use serenity::async_trait;

#[async_trait]
pub trait CollectionItem<D>
where
    Self: Sized + Clone + Send + Serialize + DeserializeOwned + Unpin + Sync,
    D: Into<Bson> + Send + 'static,
{
    /// update if exist, create or else
    async fn save_generic(
        &self,
        collection: &Collection<Self>,
    ) -> Result<(), mongodb::error::Error> {
        if collection
            .find_one_and_replace(doc! {"_id": self.id().into()}, self)
            .await?
            .is_none()
        {
            collection.insert_one(self).await?;
        };
        Ok(())
    }

    /// replaces item
    async fn save_replace(
        &self,
        collection: &Collection<Self>,
    ) -> Result<(), mongodb::error::Error> {
        collection
            .find_one_and_replace(doc! {"_id": self.id().into()}, self)
            .await?;
        Ok(())
    }

    /// creates item
    async fn save_create(
        &self,
        collection: &Collection<Self>,
    ) -> Result<(), mongodb::error::Error> {
        collection.insert_one(self).await?;
        Ok(())
    }

    /// deletes self
    async fn delete(&self, collection: &Collection<Self>) -> Result<(), mongodb::error::Error> {
        collection
            .find_one_and_delete(doc! {"_id": self.id().into()})
            .await?;
        Ok(())
    }

    async fn find_by_id(
        id: D,
        collection: &Collection<Self>,
    ) -> Result<Option<Self>, mongodb::error::Error> {
        collection.find_one(doc! {"_id": id.into()}).await
    }

    fn id(&self) -> D;
}
