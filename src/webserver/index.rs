use std::path::PathBuf;

use crate::valorant::AgentDataJSON;
use crate::valorant::RadarJSON;
use crate::valorant::WeaponDataJSON;
use actix::Actor;
use actix::AsyncContext;
use actix::Message;
use actix::StreamHandler;
use actix_files::NamedFile;
use actix_web::{http::KeepAlive, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;

// Implement a Message to send radar data updates to WebSocket actors
#[allow(dead_code)]
struct RadarDataUpdate(RadarJSON);
impl Message for RadarDataUpdate {
    type Result = ();
}

// WebSocket actor to handle WebSocket connections and data updates
struct RadarWs;

impl Actor for RadarWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        // For example, you could register the client here and start sending updates
        ctx.run_interval(std::time::Duration::from_micros(10), |_a, b| {
            b.text(RadarJSON::get().0.to_string())
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for RadarWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(msg) = msg {
            match msg {
                ws::Message::Text(_) => {
                    ctx.text(RadarJSON::get().0.to_string());
                }
                ws::Message::Ping(_) => ctx.pong("What are you doing here?".as_bytes()),
                ws::Message::Pong(_) => (),
                ws::Message::Close(reason) => {
                    ctx.close(reason);
                }
                _ => (),
            }
        }
    }
}

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

async fn assets(req: HttpRequest) -> impl Responder {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    //log::info!("path: {:?}", path);
    let file = NamedFile::open_async("./static/".to_string() + path.to_str().unwrap()).await;
    if let Ok(file) = file {
        return file.into_response(&req);
    }
    HttpResponse::Ok().body("404: Not Found")
}

async fn radar_data() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&RadarJSON::get().0).unwrap())
}

async fn radar_ws(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(RadarWs {}, &req, stream)
}

async fn actor_data() -> impl Responder {
    log::info!("actor_data");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&AgentDataJSON::get().0).unwrap())
}

async fn weapon_data() -> impl Responder {
    log::info!("weapon_data");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&WeaponDataJSON::get().0).unwrap())
}

#[actix_web::main]
pub async fn initialize() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(radar_ws)) // radar_data websocket
            .service(web::resource("/").to(index))
            //.service(web::resource("/data.json").to(radar_data))
            .service(web::resource("/actors.json").to(actor_data))
            .service(web::resource("/weapons.json").to(weapon_data))
            .service(web::resource("/radar.json").to(radar_data))
            .service(web::resource("/{filename:.*}").to(assets))
    })
    .keep_alive(KeepAlive::Os)
    .workers(4)
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
