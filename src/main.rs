use std::io::Write;
use std::fs::{create_dir_all, File};
use std::error::Error;
use std::vec;
use std::sync::Mutex;

use actix_files::NamedFile;
use serde::{Serialize, Deserialize};
use futures_util::stream::StreamExt;

use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer };

#[derive(Serialize, Deserialize, Debug)]
struct FileInfo {

    path: String,
    encrypt: bool,
    content: String,

}
struct State {

    last_file: Mutex<Option<(String, String)>>,
}

fn ceasar_cipher(content: &str, encrypt: bool) -> Result<String, Box<dyn Error>> {

    let shift = if encrypt { 3 } else { -3 };

    let res = content.chars()
        .map(|c| {

            if c.is_ascii_uppercase() {
                
                // 65 is ASCII value of 'A'
                (((c as u8 - 65) as i32 + shift).rem_euclid(26) as u8 + 65) as char
            }
            else if c.is_ascii_lowercase() {

                // 97 us ASCII value of 'a'
                (((c as u8 - 97) as i32 + shift).rem_euclid(26) as u8 + 97) as char
            }
            else if c.is_ascii_digit() {

                // 48 is ASCII value of '0'
                (((c as u8 - 48) as i32 + shift).rem_euclid(10) as u8 + 97) as char
            }
            else {

                c
            }
        })
        .collect();

    Ok(res)
}


async fn encrypt(file: web::Json<FileInfo>, data: web::Data<State>) -> Result<HttpResponse, actix_web::error::Error> {
   
    let encrypted_content = ceasar_cipher(&file.content, file.encrypt)?;

    let output_folder = if file.encrypt { "encrypted" } else { "decrypted" };
    let file_path = if file.encrypt { format!("encrypted_{}", file.path) } else { format!("decrypted_{}", file.path) };
    
    let output_path = format!("{}/{}", output_folder, file_path);

    if let Err(_) = create_dir_all(output_folder) {

        return Ok(HttpResponse::InternalServerError().body("Failed to create output directory"));
    }

    let mut output_file = match File::create(&output_path) {

        Ok(file) => file,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to create output file.")),
    };

    if let Err(_) = writeln!(output_file, "{}", encrypted_content) {
        
        return Ok(HttpResponse::InternalServerError().body("Failed to write to output file."));
    }

    let mut last_file = data.last_file.lock().unwrap();
    *last_file = Some((output_path.clone(), file_path.clone()));

    Ok(HttpResponse::Ok().body(encrypted_content))
}

async fn upload(mut payload: Multipart) -> Result<HttpResponse, actix_web::error::Error> {

    let mut uploaded_files = vec![];

    while let Some(item) = payload.next().await {

        let mut field = item?;

        let content_dispostion = field.content_disposition();
        let file_name = content_dispostion.get_filename().unwrap_or_default().to_string();

        let uploaded_dir = "uploaded";
        if let Err(_) = create_dir_all(uploaded_dir) {

            return Ok(HttpResponse::InternalServerError().body("Failed to create uploaded directory"));
        }

        let file_path = format!("{}/{}", uploaded_dir,file_name);
        let mut file = match File::create(&file_path) {

            Ok(file) => file,
            Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to save file.")),
        };

        while let Some(chunk) = field.next().await {
            
            let data = chunk?;
            if let Err(_) = file.write_all(&data) {

                return Ok(HttpResponse::InternalServerError().body("Failed to write file data."))
            }
        }

        uploaded_files.push(file_path);
    }

    if uploaded_files.is_empty() {

        return Ok(HttpResponse::BadRequest().body("No files uploaded."));
    }

    Ok(HttpResponse::Ok().body(format!("Files uploaded {:?}", uploaded_files)))
}


use actix_web::http::header::{ContentDisposition, DispositionType, DispositionParam};

async fn download(data: web::Data<State>) -> Result<NamedFile, actix_web::error::Error> {

    let last_file = data.last_file.lock().unwrap();

    if let Some((ref file_path, ref orig_file)) = *last_file {

        let file = NamedFile::open(file_path)?;

        Ok(file.set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![ DispositionParam::Filename(orig_file.clone().into()) ],
        }))
    }
    else {

        Err(actix_web::error::ErrorNotFound("No file for download"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let state = web::Data::new(State{

        last_file: Mutex::new(None),
    });

    HttpServer::new(move || {

        App::new()
            .app_data(state.clone())
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(|| async {

                HttpResponse::Found()
                    .append_header(("Location", "static/index.html"))
                    .finish()
            }))
            .route("/api/encrypt", web::post().to(encrypt))
            .route("/upload", web::post().to(upload))
            .route("/download", web::get().to(download))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
