use std::{env, fs::{self, File}, io::{copy, Write}};
use itertools::Itertools;
use serde::{Serialize,Deserialize};
use serde_json::Value;
use reqwest;

#[derive(Serialize,Deserialize,Debug)]
struct Config{
    admited_categories : Option<Vec<String>>,
    max_images_per_category: Option<u32>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Dataset{
    images : Vec<Image>,
    annotations : Vec<Annotation>,
    categories : Vec<Category>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Image {
    id: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    file_name: Option<String>,
    license: Option<String>,
    flickr_url: Option<String>,
    coco_url: Option<String>,
    date_captured: Option<String>,
    flickr_640_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Annotation {
    id: Option<u32>,
    image_id: Option<u32>,
    category_id: Option<u32>,
    segmentation: Option<Vec<Vec<Value>>>,
    area: Option<f32>,
    bbox: Option<[Value; 4]>,
    iscrowd: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Category {
    id: Option<u32>,
    name: Option<String>,
    supercategory: Option<String>,
}

fn main() {

    let cli_args : Vec<String>  = env::args().collect();

    if cli_args.len() == 0{
        println!("Por favor indique la ruta correspondiente al dataset");
        panic!();
    }

    let coco_dataset : Dataset = serde_json::from_str(fs::read_to_string(&cli_args[1]).unwrap().as_str()).unwrap();
    let self_config : Config = serde_json::from_str(fs::read_to_string(&cli_args[2]).unwrap().as_str()).unwrap();

    let target_categories : Vec<String> = self_config.admited_categories.unwrap_or(vec![]);
    let max_images_per_category : u32 = self_config.max_images_per_category.unwrap_or(u32::MAX);

    println!("\nCategorias = {}", target_categories.iter().join(", "));
    println!("Maximo de imagenes por categoria = {}\n", max_images_per_category);

    let current_categories = coco_dataset.categories
        .iter()
        .cloned()
        .filter(|cat| target_categories.contains(&cat.name.clone().unwrap()))
        .collect::<Vec<Category>>();

    let current_annotations = current_categories
        .iter()
        .map(|cat|{
            coco_dataset.annotations
                .iter()
                .cloned()
                .filter(|an| an.category_id.unwrap() == cat.id.unwrap())
                .sorted_by(|a,b| Ord::cmp(&a.iscrowd.unwrap(), &b.iscrowd.unwrap()))
                .take(max_images_per_category as usize)
            }
        ).flatten().collect::<Vec<Annotation>>();

    let current_images = coco_dataset.images
        .iter()
        .cloned()
        .filter(|im| current_annotations.iter().any(|an| an.image_id.unwrap() == im.id.unwrap()))
        .collect::<Vec<Image>>();

    current_images.iter().for_each(|x| {

        let url : String = x.flickr_url.clone().unwrap();

        println!("Descargando {}", url);

        let mut response = reqwest::blocking::get(url).unwrap();

        let file_path = format!("./data/{}",x.file_name.clone().unwrap());

        if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
            if let Err(err) = fs::create_dir_all(parent_dir) {
                eprintln!("Error al crear directorios: {}", err);
                return;
            }
        }

        let mut file = File::create(&file_path).unwrap();

        if response.status().is_success() {
            copy(&mut response, &mut file).expect("");
        }

    });

    let current_dataset = Dataset{categories : current_categories, annotations : current_annotations, images : current_images};

    let mut res = File::create("New_Annotations.json").unwrap();
    res.write_all(serde_json::to_string(&current_dataset).unwrap().as_bytes()).expect("");

}
