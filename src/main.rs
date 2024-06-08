use std::{env, fs::{self, File}, io::copy};
use serde::{Serialize,Deserialize};
use serde_json::Value;
use reqwest;

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

    let coco_json : String = fs::read_to_string(&cli_args[1]).unwrap();
    let coco_dataset : Dataset = serde_json::from_str(coco_json.as_str()).unwrap();

    let target_categories = ["Aluminium foil"];

    let current_categories = coco_dataset.categories
        .iter()
        .cloned()
        .filter(|cat| target_categories.contains(&cat.name.as_ref().unwrap().as_str()))
        .collect::<Vec<Category>>();

    let current_annotations = coco_dataset.annotations
        .iter()
        .cloned()
        .filter(|an| current_categories.iter().any(|cat| cat.id.unwrap() == an.category_id.unwrap()))
        .collect::<Vec<Annotation>>();

    let current_images = coco_dataset.images
        .iter()
        .cloned()
        .filter(|im| current_annotations.iter().any(|an| an.image_id.unwrap() == im.id.unwrap()))
        .collect::<Vec<Image>>();

    current_images.iter().for_each(|x| {

        let mut response = reqwest::blocking::get(x.flickr_url.clone().unwrap()).unwrap();

        let file_path = format!("/Volumes/Elements/{}",x.file_name.clone().unwrap());

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

    println!("{:?}", current_dataset.images.len());

}
