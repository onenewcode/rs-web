mod flash;

use axum::{
    Router,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
};
use axum_service::{
    Mutation as MutationCore, Query as QueryCore,
    sea_orm::{Database, DatabaseConnection},
};
use entity::{banner_popup, post};
use flash::{PostResponse, get_flash_cookie, post_response};
use migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::env;
use tera::Tera;
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::services::ServeDir;
use tracing::*;

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    tracing_subscriber::fmt::init();

    match dotenvy::dotenv() {
        Ok(_) => info!("Successfully loaded .env file"),
        Err(e) => {
            error!("Failed to load .env file: {}", e);
            std::process::exit(1);
        }
    }

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
        
    // Try to run migrations, but handle cases where historical migrations are missing
    match migration::Migrator::up(&conn, None).await {
        Ok(_) => info!("Migrations completed successfully"),
        Err(e) => {
            warn!("Migration warning: {}", e);
            // Continue execution even if some historical migrations are missing
        }
    }
    
    // Run seeders after migrations
    match seeder::Migrator::up(&conn, None).await {
        Ok(_) => info!("Seeders completed successfully"),
        Err(e) => {
            warn!("Seeding warning: {}", e);
            // Continue execution even if seeding has warnings
        }
    }

    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");

    let state = AppState { templates, conn };

    let app = Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/posts/{id}", get(edit_post).post(update_post))
        .route("/posts/new", get(new_post))
        .route("/posts/delete/{id}", post(delete_post))
        .route(
            "/banner-popups",
            get(list_banner_popups).post(create_banner_popup),
        )
        .route(
            "/banner-popups/{id}",
            get(edit_banner_popup).post(update_banner_popup),
        )
        .route("/banner-popups/new", get(new_banner_popup))
        .route("/banner-popups/delete/{id}", post(delete_banner_popup))
        .nest_service(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            .handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    templates: Tera,
    conn: DatabaseConnection,
}

#[derive(Deserialize)]
struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

async fn list_posts(
    state: State<AppState>,
    Query(params): Query<Params>,
    cookies: Cookies,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(5);

    let (posts, num_pages) = QueryCore::find_posts_in_page(&state.conn, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn new_post(state: State<AppState>) -> Result<Html<String>, (StatusCode, &'static str)> {
    let ctx = tera::Context::new();
    let body = state
        .templates
        .render("new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn create_post(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<post::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_post(&state.conn, form)
        .await
        .expect("could not insert post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post successfully added".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn edit_post(
    state: State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let post: post::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    form: Form<post::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
    let form = form.0;

    MutationCore::update_post_by_id(&state.conn, id, form)
        .await
        .expect("could not edit post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post successfully updated".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn delete_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_post(&state.conn, id)
        .await
        .expect("could not delete post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post successfully deleted".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

// Banner Popup handlers

async fn list_banner_popups(
    state: State<AppState>,
    Query(params): Query<Params>,
    cookies: Cookies,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let items_per_page = params.posts_per_page.unwrap_or(5); // Reuse posts_per_page parameter

    let (banner_popups, num_pages) =
        QueryCore::find_banner_popups_in_page(&state.conn, page, items_per_page)
            .await
            .expect("Cannot find banner popups in page");

    let mut ctx = tera::Context::new();
    ctx.insert("banner_popups", &banner_popups);
    ctx.insert("page", &page);
    ctx.insert("items_per_page", &items_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("banner_popups/index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn new_banner_popup(
    state: State<AppState>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let ctx = tera::Context::new();
    let body = state
        .templates
        .render("banner_popups/new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn create_banner_popup(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<banner_popup::Model>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_banner_popup(&state.conn, form)
        .await
        .expect("could not insert banner popup");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Banner popup successfully added".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn edit_banner_popup(
    state: State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let banner_popup: banner_popup::Model = QueryCore::find_banner_popup_by_id(&state.conn, id)
        .await
        .expect("could not find banner popup")
        .unwrap_or_else(|| panic!("could not find banner popup with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("banner_popup", &banner_popup);

    let body = state
        .templates
        .render("banner_popups/edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_banner_popup(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    form: Form<banner_popup::Model>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::update_banner_popup_by_id(&state.conn, id, form)
        .await
        .expect("could not edit banner popup");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Banner popup successfully updated".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn delete_banner_popup(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_banner_popup(&state.conn, id)
        .await
        .expect("could not delete banner popup");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Banner popup successfully deleted".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
