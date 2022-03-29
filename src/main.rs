#[macro_use] extern crate rocket;

use std::path::{Path, PathBuf};
use std::collections::HashMap;

// use std::ffi::CString;
// use libc::chown;

use rocket::form::{Form, Contextual, FromForm};
use rocket::fs::{NamedFile, TempFile, relative};

use regex::Regex;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(relative!("/static/fox.png")).await.ok()
}

#[get("/files/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    let mut path = Path::new(relative!("static/files")).join(file);
    if !path.exists() {
        path = Path::new("/drives/LinuxData0/aikufurr/Pictures/c2mec31oelg61.png").to_path_buf();
    }
    NamedFile::open(&path).await.ok()
}

#[get("/upload")]
async fn upload_get() -> Option<NamedFile> {
    NamedFile::open(Path::new(relative!("static/upload.html"))).await.ok()
}


#[derive(Debug, FromForm)]
struct FileStruct<'v> {
    file: TempFile<'v>,
}

#[post("/upload", data = "<form>")]
async fn upload_post<'r>(mut form: Form<Contextual<'r, FileStruct<'r>>>) -> String {
    let exit_code: String = match form.value {
        Some(ref mut submission) => {
            let file = &mut submission.file;

            let upload_path = format!("./static/{}", "files");
            let file_r = Regex::new(r"(?m)([A-z]*)$").unwrap();
            let unsafe_name = file.raw_name().unwrap().dangerous_unsafe_unsanitized_raw().as_str();
            let ext: &str;
            match file_r.captures(&unsafe_name) {
                Some(v) => ext = v.get(1).unwrap().as_str(),
                None => ext = "",
            }
            let file_name = file.name().unwrap();
            if !Path::new(&upload_path).exists() {
                std::fs::create_dir_all(&upload_path).unwrap();
            }
            let filename = format!("{}_{}.{}", uuid::Uuid::new_v4().to_string().replace("-", ""), file_name, ext);
            let pth = format!("{}/{}", upload_path, &filename);
            let filepath = Path::new(&pth);
            file.copy_to(filepath).await.expect("file upload error!");
            
            // let file_as_cstring = CString::new(&*filepath.to_str().unwrap()).expect("CString::new failed");
            // let ptr = file_as_cstring.as_ptr();

            // unsafe {
            //     chown(ptr, 1000, 1000);
            // }

            filename
        }
        None => "error".to_owned(),
    };

    exit_code
}


#[get("/presence?<cmus_base>&<cmus_artist>&<cmus_album>")]
async fn presence(cmus_base: Option<&str>, cmus_artist: Option<&str>, cmus_album: Option<&str>) -> Option<NamedFile> {
    let mut base: String;
    match cmus_base {
        Some(arg) => base = arg.replace("%20", " "),
        None => base = "".to_owned()
    }
    let artist: String;
    match cmus_artist {
        Some(arg) => artist = arg.replace("%20", " "),
        None => artist = "".to_owned()
    }
    let album: String;
    match cmus_album {
        Some(arg) => album = arg.replace("%20", " "),
        None => album = "".to_owned()
    }

    let mut path: String = relative!("/static/fox.png").to_owned();

    if !artist.is_empty() && !album.is_empty() {
        if !base.is_empty() {
            base = base + "/";
        }
        let dir = "/home/aikufurr/Music/".to_owned() + &base + &artist + "/" + &album;
        if Path::new(&format!("{}/cover.png", dir)).exists() {
            path = format!("{}/cover.png", dir);
        } else if Path::new(&format!("{}/cover.jpg", dir)).exists() {
            path = format!("{}/cover.jpg", dir);
        } else if Path::new(&format!("{}/cover0.png", dir)).exists() {
            path = format!("{}/cover0.png", dir);
        } else if Path::new(&format!("{}/cover0.jpg", dir)).exists() {
            path = format!("{}/cover0.jpg", dir);
        } else if Path::new(&format!("{}/cover1.png", dir)).exists() {
            path = format!("{}/cover1.png", dir);
        } else if Path::new(&format!("{}/cove1r.jpg", dir)).exists() {
            path = format!("{}/cover1.jpg", dir);
        }
    }

    NamedFile::open(&path).await.ok()
}

#[get("/wordle")]
async fn wordle() -> Option<NamedFile> {
    let mut path = Path::new(relative!("static/wordle/wordle.html"));
    if !path.exists() {
        path = Path::new(relative!("/static/fox.png"));
    }
    NamedFile::open(&path).await.ok()
}

#[catch(404)]
async fn not_found() -> Option<NamedFile> {
    NamedFile::open("/drives/LinuxData0/aikufurr/Pictures/c2mec31oelg61.png").await.ok()
}

#[derive(Debug, FromForm, UriDisplayQuery)]
struct EmailWebhook {
    recipient: String,
    subject: String,
}

fn db_make_email(email: &EmailWebhook) -> String {
    let conn = sqlite::open("database.db").unwrap();

    let id: String = uuid::Uuid::new_v4().to_string().replace("-", "") + ".png";

    let mut statement = conn.prepare("INSERT INTO emails (ID, RECIPIENT, SUBJECT) VALUES (?,?,?)").unwrap();
    statement.bind(1, &*id).unwrap();
    statement.bind(2, &*email.recipient).unwrap();
    statement.bind(3, &*email.subject).unwrap();
    
    statement.next().unwrap();
    
    id
}

fn db_get_email(id: &str) -> EmailWebhook {
    let conn = sqlite::open("database.db").unwrap();

    let mut statement = conn.prepare("SELECT RECIPIENT, SUBJECT FROM emails WHERE ID = ?").unwrap();
    statement.bind(1, id).unwrap();

    let mut eml = EmailWebhook {
        recipient: "".to_owned(),
        subject: "".to_owned()
    };

    eml.recipient = "".to_owned();

    while let sqlite::State::Row = statement.next().unwrap() {
        eml.recipient = statement.read::<String>(0).unwrap();
        eml.subject = statement.read::<String>(1).unwrap();
    }

    eml
}

// fn db_del_email(id: &str) {
//     let conn = sqlite::open("database.db").unwrap();

//     let mut statement = conn.prepare("DELETE FROM emails WHERE ID = ?").unwrap();
//     statement.bind(1, id).unwrap();
//     statement.next().unwrap();
// }

#[get("/email?<email..>")]
async fn email_make(email: EmailWebhook) -> String {

    db_make_email(&email)
}

#[get("/email/<id>")]
async fn email(id: String) -> Option<NamedFile> {
    let details = db_get_email(&id);

    let mut map = HashMap::new();
    map.insert("content", "<@308681202548604938>, your email to `".to_owned() + &details.recipient + "` about `" + &details.subject + "` was read!");

    let client = reqwest::Client::new();
    let response = client.post("https://discord.com/api/webhooks/x/y")
        .json(&map)
        .send()
        .await;

    match response {
        Ok(_) => {},
        Err(_) => {}
    };

    // db_del_email(&id);

    NamedFile::open(relative!("/static/1x1.png")).await.ok()
}

fn init_db() {
    let conn = sqlite::open("database.db").unwrap();

    let mut statement = conn.prepare("SELECT count(name) FROM sqlite_master WHERE type='table' AND name = ?").unwrap();
    statement.bind(1, "emails").unwrap();
    while let sqlite::State::Row = statement.next().unwrap() {
        if statement.read::<i64>(0).unwrap() == 0 {
            statement = conn.prepare(r#"CREATE TABLE "emails" (
                "ID"	TEXT NOT NULL UNIQUE,
                "RECIPIENT"	TEXT,
                "SUBJECT"	TEXT,
                PRIMARY KEY("ID")
            )"#).unwrap();
            statement.next().unwrap();
        }
        break;
    }
}

#[launch]
fn rocket() -> _ {
    init_db();
    rocket::build().mount("/", routes![index, files, upload_get, upload_post, presence, wordle, email_make, email]).register("/", catchers![not_found])
}
