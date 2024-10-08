use std::error::Error;

use mongodb::bson::doc;
use mongodb::Database;
use serde::{Deserialize, Serialize};

use super::{collections::*, Mongo};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Counter {
    pub count: i64,
}

impl Counter {
    pub async fn bump_get(id: &str, _db: &Database) -> Result<i64, Box<dyn Error>> {
        let counters = unsafe { COUNTERS_DESER.get() }.unwrap();
        let filter = doc! {"_id": id};
        let update = doc! {"$inc": {"count": 1}};
        let exists = counters.find_one(filter.clone()).await?.is_some();
        Ok(if exists {
            // let options = mongodb::options::FindOneAndUpdateOptions::builder()
            //     .upsert(true)
            //     .build();
            counters
                .find_one_and_update(filter, update)
                .await?
                .unwrap()
                .count
        } else {
            unsafe { COUNTERS_SER.get() }
                .unwrap()
                .insert_one(doc! {"_id": id, "count": 2})
                .await?;
            1
        })
    }
}

impl Mongo {
    pub async fn new_id(collection: &str) -> i64 {
        Counter::bump_get(collection, unsafe { DATABASE.get() }.unwrap())
            .await
            .unwrap()
    }
}
