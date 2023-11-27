use std::io::Write;
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs::File};

use crate::data::{Materials, Recipe};
use crate::timekeeper;

#[derive(Debug, Clone)]
pub struct MissingProductionTableError {
    pub url: String,
}

#[derive(Debug)]
pub enum RetryRequestError {
    MissingProductionTableError(MissingProductionTableError),
    ReqwestError(reqwest::Error),
}

pub struct RetryRequest<'a> {
    pub url: &'a str,
}

impl<'a> RetryRequest<'a> {
    pub fn new(url: &'a str) -> Self {
        Self { url }
    }

    pub async fn fetch(&self) -> Result<String, RetryRequestError> {
        let mut retries = 0;
        loop {
            let response = reqwest::get(self.url).await;
            match response {
                Ok(response) => {
                    let text_response = response.text().await.unwrap();
                    let document = scraper::Html::parse_document(&text_response);
                    let table_selector =
                        scraper::Selector::parse("table.pc_table:nth-of-type(1)").unwrap();
                    let table_elems: Vec<_> = document.select(&table_selector).collect();

                    if let Some(_table) = table_elems.first() {
                        return Ok(text_response);
                    } else {
                        println!("Unable to find production chain table for url {}", self.url);
                        retries += 1;
                        thread::sleep(Duration::from_millis(5000));
                        if retries > 3 {
                            return Err(RetryRequestError::MissingProductionTableError(
                                MissingProductionTableError {
                                    url: self.url.to_string(),
                                },
                            ));
                        }
                    }
                }
                Err(err) => {
                    println!("Error fetching url: {}", err);
                    retries += 1;
                    thread::sleep(Duration::from_millis(5000));
                    if retries > 3 {
                        return Err(RetryRequestError::ReqwestError(err));
                    }
                }
            }
        }
    }
}

pub struct Scraper {}

impl Scraper {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn scrape_dsp_data(&self, mut urls: Vec<String>) -> Vec<Vec<Recipe>> {
        let mut timekeeper = timekeeper::TimeKeeper::new();
        println!("Start {:?}", timekeeper.start());

        if urls.is_empty() {
            urls = self.get_urls().await;
            urls.iter().for_each(|item| println!("{}", item));
        }

        let mut recipe_lists = vec![];

        for (i, url) in urls.iter().enumerate() {
            println!(
                "[{}/{} {}%] {:?}",
                i + 1,
                urls.len(),
                (i + 1) * 100 / urls.len(),
                timekeeper.tick()
            );

            let recipe_list = self.scrape_url(url).await;
            recipe_lists.push(recipe_list);
            thread::sleep(Duration::from_millis(1000));
        }

        println!("End {:?}", timekeeper.end());

        // write recipe lists to json file
        let mut file = File::create("recipes.json").unwrap();
        let json = serde_json::to_string_pretty(&recipe_lists).unwrap();
        file.write_all(json.as_bytes()).unwrap();

        recipe_lists
    }

    pub async fn get_urls(&self) -> Vec<String> {
        let response = reqwest::get("https://dsp-wiki.com/Items")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let document = scraper::Html::parse_document(&response);
        let selector = scraper::Selector::parse("div.item_icon_container a[href]").unwrap();
        let urls = document
            .select(&selector)
            .map(|x| format!("https://dsp-wiki.com{}", x.attr("href").unwrap()));

        // uniqify
        let mut urls: Vec<String> = urls.collect();
        urls.sort();
        urls.dedup();

        println!("URL Count: {}", urls.len());
        urls
    }

    pub async fn scrape_url(&self, url: &str) -> Vec<Recipe> {
        println!("Scraping URL: {}", url);

        let mut recipes: Vec<Recipe> = vec![];

        let retry_request = RetryRequest::new(url);
        let response = retry_request.fetch().await;

        if response.is_err() {
            println!("Error fetching url: {:?}", response);
            return recipes;
        }

        let response = response.unwrap();

        let document = scraper::Html::parse_document(&response);
        let table_selector = scraper::Selector::parse("table.pc_table:nth-of-type(1)").unwrap();
        let table_elems: Vec<_> = document.select(&table_selector).collect();
        let row_selector = scraper::Selector::parse("tr:nth-of-type(n+1)").unwrap();
        if let Some(table) = table_elems.first() {
            let rows: Vec<_> = table.select(&row_selector).collect();
            for row in rows {
                let mut recipe = Recipe::new();

                // ------------------------------ MATERIALS ------------------------------
                let mut materials: Materials = HashMap::new();
                let material_selector = scraper::Selector::parse("div.tt_recipe_item").unwrap();
                let material_elems: Vec<_> = row.select(&material_selector).collect();

                for material_elem in material_elems {
                    let name_selector = scraper::Selector::parse("a").unwrap();
                    let name = &material_elem
                        .select(&name_selector)
                        .map(|x| x.attr("title"))
                        .next()
                        .unwrap_or_default();

                    if name.is_none() {
                        continue;
                    }

                    let count_selector = scraper::Selector::parse("div").unwrap();
                    let count_text = material_elem
                        .select(&count_selector)
                        .map(|x| x.text())
                        .next();

                    if count_text.is_none() {
                        continue;
                    }

                    let count = count_text.unwrap().next().unwrap_or("0");
                    materials.insert(name.unwrap().to_string(), count.parse::<f64>().unwrap());
                }

                recipe.materials = materials;

                // ------------------------------ TIME ------------------------------
                let time_selector = scraper::Selector::parse("div.tt_rec_arrow");
                let time_text = row.select(&time_selector.unwrap()).map(|x| x.text()).next();

                if time_text.is_none() {
                    continue;
                }

                let time = time_text.unwrap().next().unwrap_or("0");
                let re = regex::Regex::new(r"\d+\.*\d*").unwrap();
                let captures = re.captures(time);
                if let Some(captures) = captures {
                    let time_as_float = captures[0].parse::<f64>().unwrap();
                    recipe.time = time_as_float;
                }

                // ------------------------------ OUTPUT ITEM NAME, COUNT, & IMAGE ------------------------------

                let output_item_selector = scraper::Selector::parse("div.tt_output_item").unwrap();
                let output_items: Vec<_> = row.select(&output_item_selector).collect();
                for output_item in output_items {
                    // ------------------------------ OUTPUT ITEM NAME ------------------------------
                    let name_selector = scraper::Selector::parse("a").unwrap();
                    let name = &output_item
                        .select(&name_selector)
                        .map(|x| x.attr("title"))
                        .next()
                        .unwrap_or_default();

                    if name.is_none() {
                        continue;
                    }

                    // ------------------------------ OUTPUT ITEM COUNT ------------------------------
                    let count_selector = scraper::Selector::parse("div").unwrap();
                    let count_text = output_item.select(&count_selector).map(|x| x.text()).next();

                    if count_text.is_none() {
                        continue;
                    }

                    let count = count_text.unwrap().next().unwrap_or("0");
                    let re = regex::Regex::new(r"\d+\.*\d*").unwrap();
                    let captures = re.captures(count);
                    if let Some(captures) = captures {
                        let count_as_float = captures[0].parse::<f64>().unwrap();
                        recipe.output_item_count = count_as_float;
                    }

                    recipe.output_item = name.unwrap().to_string();

                    // ------------------------------ OUTPUT ITEM IMAGE ------------------------------
                    let image_selector = scraper::Selector::parse("img").unwrap();
                    let image = output_item
                        .select(&image_selector)
                        .map(|x| format!("https://dsp-wiki.com{}", x.attr("src").unwrap_or("/")))
                        .next()
                        .unwrap_or_default();

                    recipe.image = Some(image);
                }

                // ------------------------------ FACILITY ------------------------------
                let facilities_selector = scraper::Selector::parse("td:nth-of-type(2)").unwrap();
                let facility_elems: Vec<_> = row.select(&facilities_selector).collect();

                for facility_elem in facility_elems {
                    let facility_selector = scraper::Selector::parse("a:nth-of-type(1)").unwrap();
                    let facility = facility_elem
                        .select(&facility_selector)
                        .map(|x| x.attr("title").unwrap())
                        .next();

                    if let Some(facility) = facility {
                        recipe.facility = facility.to_string();
                    }
                }

                println!("{:?}", recipe);
                println!("-----------------------------------");

                recipes.push(recipe);
            }
        }

        recipes
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_get_urls() {
        let scraper = super::Scraper::new();
        let urls = scraper.get_urls().await;
        assert!(!urls.is_empty());
    }

    #[tokio::test]
    async fn test_scrape_url() {
        let scraper = super::Scraper::new();
        let url = "https://dsp-wiki.com/Iron_Ingot";
        let recipes = scraper.scrape_url(url).await;
        assert!(!recipes.is_empty());
        let recipe = recipes[0].clone();
        assert_eq!(recipe.output_item, "Iron Ingot");
        assert_eq!(recipe.output_item_count, 1.0);
        assert_eq!(recipe.facility, "Arc Smelter");
        assert_eq!(recipe.time, 1.0);
        assert_eq!(recipe.materials.len(), 1);
        assert_eq!(recipe.materials.get("Iron Ore"), Some(&1.0));
    }
}
