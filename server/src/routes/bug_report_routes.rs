use actix_web::web;

use crate::handlers::bug_report_handlers::BugReportHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/bug_report")
            .route("/", web::post().to(BugReportHandler::create_bug_report))
            .route("/{id}", web::put().to(BugReportHandler::update_bug_status))
            .service(
                web::resource("/")
                .wrap(jwt_middleware.clone())
                .route(web::get().to(BugReportHandler::get_all_bug_reports))
            )
        );
}