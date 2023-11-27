use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::{self, Recipe};

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
