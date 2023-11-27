use std::env;

use super::Recipe;
use futures::TryStreamExt;
use mongodb::bson::Document;
use mongodb::{bson::doc, Client, Collection};

#[derive(Debug)]
pub struct DB {}

impl DB {
    pub fn new() -> Self {
        Self {}
    }

    #[tracing::instrument]
    pub async fn test(&self) -> mongodb::error::Result<()> {
        let uri = env::var("MONGODB_URI").unwrap();
        let client = Client::with_uri_str(uri).await?;
        let database: mongodb::Database = client.database("dsp");
        let recipes_coll: Collection<Document> = database.collection("recipes");
        // Find a movie based on the title value
        let recipe = recipes_coll
            .find_one(doc! { "title": "Title" }, None)
            .await?;
        // Print the document
        println!("Found a recipe:\n{:#?}", recipe);
        Ok(())
    }

    #[tracing::instrument]
    pub async fn get_recipes(&self) -> mongodb::error::Result<Vec<Recipe>> {
        let uri = env::var("MONGODB_URI").unwrap();
        let client = Client::with_uri_str(uri).await?;
        let database: mongodb::Database = client.database("dsp");
        let recipes_coll: Collection<Recipe> = database.collection("recipes");
        let recipes = recipes_coll.find(doc! {}, None).await?;
        let recipes = recipes.try_collect().await?;
        Ok(recipes)
    }

    #[tracing::instrument]
    pub async fn delete_recipes(&self) -> mongodb::error::Result<()> {
        let uri = env::var("MONGODB_URI").unwrap();
        let client = Client::with_uri_str(uri).await?;
        let database: mongodb::Database = client.database("dsp");
        let recipes_coll: Collection<Document> = database.collection("recipes");

        recipes_coll.delete_many(doc! {}, None).await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn save_recipes(&self, recipes: Vec<Recipe>) -> mongodb::error::Result<()> {
        let uri = env::var("MONGODB_URI").unwrap();
        let client = Client::with_uri_str(uri).await?;
        let database = client.database("dsp");
        let recipes_coll: Collection<Recipe> = database.collection("recipes");
        recipes_coll.insert_many(recipes, None).await?;
        Ok(())
    }

    #[tracing::instrument]
    pub async fn save_recipe(&self, recipe: Recipe) -> mongodb::error::Result<()> {
        let uri = env::var("MONGODB_URI").unwrap();
        let client = Client::with_uri_str(uri).await?;
        let database = client.database("dsp");
        let recipes_coll: Collection<Recipe> = database.collection("recipes");
        recipes_coll.insert_one(recipe, None).await?;
        Ok(())
    }
}
