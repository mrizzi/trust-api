use actix_web::web::Data;
use actix_web::{middleware::Logger, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;

use crate::index;
use crate::package;
use crate::vulnerability;
use crate::guac;

pub struct Server {
    bind: String,
    port: u16,
    guac_url: String,
}

#[derive(OpenApi)]
#[openapi(
        paths(
            package::get_package,
            package::query_package,
            package::query_package_dependencies,
            package::query_package_dependants,
            package::query_package_versions,
            vulnerability::query_vulnerability,
        ),
        components(
            schemas(package::Package, package::PackageList, package::PackageDependencies, package::PackageDependants, package::PackageRef, package::SnykData, package::VulnerabilityRef, vulnerability::Vulnerability)
        ),
        tags(
            (name = "package", description = "Package query endpoints."),
            (name = "vulnerability", description = "Vulnerability query endpoints")
        ),
    )]
pub struct ApiDoc;

impl Server {
    pub fn new(bind: String, port: u16, guac_url: String) -> Self {
        Self {
            bind,
            port,
            guac_url,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let openapi = ApiDoc::openapi();

        let guac = Arc::new(guac::Guac::new(&self.guac_url));

        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .app_data(Data::new(package::TrustedContent::new(guac.clone())))
                .app_data(Data::new(guac.clone()))
                .configure(package::configure())
                .configure(vulnerability::configure())
                .configure(index::configure())
                .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/openapi.json", openapi.clone()))
        })
        .bind((self.bind, self.port))?
        .run()
        .await?;
        Ok(())
    }
}
