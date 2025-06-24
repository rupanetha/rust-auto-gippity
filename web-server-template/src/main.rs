use actix_web::{web, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FitnessProgress {
    id: u64,
    user_id: u64,
    workout: String,
    duration: u64, // in minutes
    timestamp: i64, // Unix timestamp
    timezone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    password: String,
    timezone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    fitness_progresses: HashMap<u64, FitnessProgress>,
    users: HashMap<u64, User>,
}

impl Database {
    fn new() -> Self {
        Self {
            fitness_progresses: HashMap::new(),
            users: HashMap::new(),
        }
    }

    fn insert_fitness_progress(&mut self, fitness_progress: FitnessProgress) {
        self.fitness_progresses.insert(fitness_progress.id, fitness_progress);
    }

    fn get_fitness_progress(&self, id: &u64) -> Option<&FitnessProgress> {
        self.fitness_progresses.get(id)
    }

    fn get_all_fitness_progresses(&self) -> Vec<&FitnessProgress> {
        self.fitness_progresses.values().collect()
    }

    fn delete_fitness_progress(&mut self, id: &u64) {
        self.fitness_progresses.remove(id);
    }

    fn update_fitness_progress(&mut self, fitness_progress: FitnessProgress) {
        self.fitness_progresses.insert(fitness_progress.id, fitness_progress);
    }

    fn insert_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u| u.username == username)
    }

    fn save_to_file(&self) -> std::io::Result<()> {
        let data: String = serde_json::to_string(&self)?;
        let mut file: fs::File = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file_content)?;
        Ok(db)
    }
}

struct AppState {
    db: Mutex<Database>,
}

async fn create_fitness_progress(app_state: web::Data<AppState>, fitness_progress: web::Json<FitnessProgress>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert_fitness_progress(fitness_progress.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn read_fitness_progress(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get_fitness_progress(&id.into_inner()) {
        Some(fitness_progress) => HttpResponse::Ok().json(fitness_progress),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn read_all_fitness_progresses(app_state: web::Data<AppState>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    let fitness_progresses = db.get_all_fitness_progresses();
    HttpResponse::Ok().json(fitness_progresses)
}

async fn update_fitness_progress(app_state: web::Data<AppState>, fitness_progress: web::Json<FitnessProgress>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.update_fitness_progress(fitness_progress.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn delete_fitness_progress(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.delete_fitness_progress(&id.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn register(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert_user(user.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn login(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get_user_by_name(&user.username) {
        Some(stored_user) if stored_user.password == user.password => {
            HttpResponse::Ok().body("Logged in")
        },
        _ => HttpResponse::BadRequest().body("Invalid username or password"),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let db = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new(),
    };

    let data: web::Data<AppState> = web::Data::new(AppState {
        db: Mutex::new(db),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/fitness_progress", web::post().to(create_fitness_progress))
            .route("/fitness_progress", web::get().to(read_all_fitness_progresses))
            .route("/fitness_progress/{id}", web::get().to(read_fitness_progress))
            .route("/fitness_progress", web::put().to(update_fitness_progress))
            .route("/fitness_progress/{id}", web::delete().to(delete_fitness_progress))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}