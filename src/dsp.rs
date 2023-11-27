use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    data::{self, Recipe},
    scrape::Scraper,
    timekeeper::TimeKeeper,
};

#[derive(Debug, Clone, Deserialize)]
pub struct ComputedRecipeRequest {
    pub name: String,
    pub rate: f64,
    pub requirements: RecipeRequirements,
}

pub type RecipeRequirements = HashMap<String, i64>;

#[derive(Debug, Clone, Serialize)]
pub struct ComputedRecipe {
    pub output_item: String,
    pub facility: String,
    pub num_facilities_needed: f64,
    pub items_consumed_per_sec: HashMap<String, f64>,
    pub seconds_spent_per_craft: f64,
    pub crafting_per_sec: f64,
    pub used_for: String,
    pub depth: Option<i64>,
    pub image: Option<String>,
}

#[tracing::instrument]
pub async fn load_recipes() -> HashMap<String, Vec<Recipe>> {
    let db = data::dsp::DB::new();
    let recipes = db.get_recipes().await.unwrap();

    let mut recipe_map = HashMap::new();

    for recipe in recipes {
        let entry = recipe_map
            .entry(recipe.output_item.clone().to_lowercase())
            .or_insert(vec![]);
        entry.push(recipe);
    }

    recipe_map
}

#[tracing::instrument]
pub async fn refresh_data() {
    let s = Scraper::new();
    // use empty vec to scrape all recipes
    let urls = vec![
        // "https://dsp-wiki.com/Stone_Brick".to_string(),
        // "https://dsp-wiki.com/Storage_Mk.I".to_string(),
        // "https://dsp-wiki.com/Storage_Mk.II".to_string(),
        // "https://dsp-wiki.com/Storage_Tank".to_string(),
        // "https://dsp-wiki.com/Strange_Matter".to_string(),
        // "https://dsp-wiki.com/Structure_Matrix".to_string(),
        // "https://dsp-wiki.com/Sulfuric_Acid".to_string(),
        // "https://dsp-wiki.com/Super-Magnetic_Ring".to_string(),
        // "https://dsp-wiki.com/Tesla_Tower".to_string(),
        // "https://dsp-wiki.com/Thermal_Power_Plant".to_string(),
        // "https://dsp-wiki.com/Thruster".to_string(),
        // "https://dsp-wiki.com/Titanium_Alloy".to_string(),
    ];
    let recipe_lists = s.scrape_dsp_data(urls).await;

    let db = data::dsp::DB::new();
    let _ = db.delete_recipes().await;

    let mut timekeeper = TimeKeeper::new();
    println!("Start Save Recipes {:?}", timekeeper.start());
    let flattened_recipe_lists: Vec<Recipe> = recipe_lists
        .into_iter()
        .flat_map(|list| list.into_iter())
        .collect();
    let _ = db.save_recipes(flattened_recipe_lists).await;
    println!("End Save Recipes {:?}", timekeeper.end());
}
