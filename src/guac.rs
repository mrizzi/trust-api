use crate::package::Package;
use crate::package::PackageDependencies;
use crate::package::PackageRef;
use crate::package::VulnerabilityRef;
use anyhow::anyhow;
use guac_rs::client::GuacClient;
use packageurl::PackageUrl;

#[derive(Clone)]
pub struct Guac {
    client: GuacClient,
}

impl Guac {
    pub fn new(url: &str) -> Self {
        let client = GuacClient::new(url.to_string());
        Self { client }
    }

    pub async fn get_packages(
        &self,
        purl: PackageUrl<'_>,
    ) -> Result<Vec<PackageRef>, anyhow::Error> {
        //strip version to search for all related packages
        let query_purl = format!(
            "pkg:{}/{}/{}",
            purl.ty(),
            purl.namespace().unwrap(),
            purl.name(),
        );

        let pkgs = self.client.get_packages(&query_purl).await.map_err(|e| {
            let e = format!("Error getting packages from GUAC: {:?}", e);
            log::warn!("{}", e);
            anyhow!(e)
        })?;
        let mut ret = Vec::new();
        for pkg in pkgs.iter() {
            let t = &pkg.type_;
            for namespace in pkg.namespaces.iter() {
                for name in namespace.names.iter() {
                    for version in name.versions.iter() {
                        let purl = format!(
                            "pkg:{}/{}/{}@{}",
                            t, namespace.namespace, name.name, version.version
                        );
                        let p = PackageRef {
                            purl: purl.clone(),
                            href: format!("/api/package?purl={}", &urlencoding::encode(&purl)),
                            trusted: Some(namespace.namespace == "redhat"),
                        };
                        ret.push(p);
                    }
                }
            }
        }
        Ok(ret)
    }

    pub async fn get_vulnerabilities(
        &self,
        purl: &str,
    ) -> Result<Vec<VulnerabilityRef>, anyhow::Error> {
        let vulns = self.client.certify_vuln(purl).await.map_err(|e| {
            let e = format!("Error getting vulnerabilities from GUAC: {:?}", e);
            log::warn!("{}", e);
            anyhow!(e)
        })?;

        let mut ret = Vec::new();
        for vuln in vulns.iter() {
            match &vuln.vulnerability {
                guac_rs::vuln::certify_vuln_q1::AllCertifyVulnTreeVulnerability::OSV(osv) => {
                    let id = osv.osv_id.clone();
                    let vuln_ref = VulnerabilityRef {
                        cve: id.clone(),
                        href: format!(
                            "{}/{}",
                            "https://osv.dev/vulnerability",
                            id.replace("ghsa", "GHSA")
                        ), //TODO fix guac id format
                    };
                    //TODO fix guac repeated entries
                    if !ret.contains(&vuln_ref) {
                        ret.push(vuln_ref);
                    }
                }
                guac_rs::vuln::certify_vuln_q1::AllCertifyVulnTreeVulnerability::CVE(id) => {
                    let vuln_ref = VulnerabilityRef {
                        cve: id.cve_id.clone(),
                        href: format!(
                            "https://access.redhat.com/security/cve/{}",
                            id.cve_id.to_lowercase()
                        ), //TODO fix guac id format
                    };
                    //TODO fix guac repeated entries
                    if !ret.contains(&vuln_ref) {
                        ret.push(vuln_ref);
                    }
                }
                _ => {}
            };
        }
        Ok(ret)
    }

    pub async fn get_dependencies(&self, purl: &str) -> Result<PackageDependencies, anyhow::Error> {
        let deps = self.client.get_dependencies(purl).await.map_err(|e| {
            let e = format!("Error getting dependencies from GUAC: {:?}", e);
            log::warn!("{}", e);
            anyhow!(e)
        })?;

        let mut ret = Vec::new();
        for dep in deps.iter() {
            let pkg = &dep.dependent_package;
            let t = &pkg.type_;
            for namespace in pkg.namespaces.iter() {
                for name in namespace.names.iter() {
                    for version in name.versions.iter() {
                        let purl = format!(
                            "pkg:{}/{}/{}@{}",
                            t, namespace.namespace, name.name, version.version
                        );
                        let p = PackageRef {
                            purl: purl.clone(),
                            href: format!("/api/package?purl={}", &urlencoding::encode(&purl)),
                            trusted: None,
                        };
                        ret.push(p);
                    }
                }
            }
        }
        Ok(PackageDependencies(ret))
    }

    pub async fn get_all_packages(&self) -> Result<Vec<Package>, anyhow::Error> {
        let all_packages = self.client.get_all_packages().await?;

        let mut all = Vec::new();
        for pkg in all_packages.iter() {
            let t = &pkg.type_;
            for namespace in pkg.namespaces.iter() {
                for name in namespace.names.iter() {
                    for version in name.versions.iter() {
                        let purl = format!(
                            "pkg:{}/{}/{}@{}",
                            t, namespace.namespace, name.name, version.version
                        );
                        let vulns = self.get_vulnerabilities(&purl).await?;
                        let p = Package {
                            purl: Some(purl.to_string()),
                            href: Some(format!(
                                "/api/package?purl={}",
                                &urlencoding::encode(&purl.to_string())
                            )),
                            trusted: Some(namespace.namespace == "redhat"),
                            trusted_versions: vec![],
                            snyk: None,
                            vulnerabilities: vulns,
                        };
                        all.push(p);
                    }
                }
            }
        }
        Ok(all)
    }

    pub async fn get_dependants(&self, purl: &str) -> Result<PackageDependencies, anyhow::Error> {
        let deps = self.client.is_dependent(purl).await.map_err(|e| {
            let e = format!("Error getting dependants from GUAC: {:?}", e);
            log::warn!("{}", e);
            anyhow!(e)
        })?;

        let mut ret = Vec::new();
        for dep in deps.iter() {
            let pkg = &dep.package;
            let t = &pkg.type_;
            for namespace in pkg.namespaces.iter() {
                for name in namespace.names.iter() {
                    for version in name.versions.iter() {
                        let purl = format!(
                            "pkg:{}/{}/{}@{}",
                            t, namespace.namespace, name.name, version.version
                        );
                        let p = PackageRef {
                            purl: purl.clone(),
                            href: format!("/api/package?purl={}", &urlencoding::encode(&purl)),
                            trusted: None,
                        };
                        ret.push(p);
                    }
                }
            }
        }
        Ok(PackageDependencies(ret))
    }
}
