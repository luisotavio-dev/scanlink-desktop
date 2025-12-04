use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::sync::{Arc, Mutex};

const SERVICE_TYPE: &str = "_scanlink._tcp.local.";

pub struct MdnsService {
    daemon: Arc<Mutex<Option<ServiceDaemon>>>,
    service_fullname: Arc<Mutex<Option<String>>>,
}

impl MdnsService {
    pub fn new() -> Self {
        Self {
            daemon: Arc::new(Mutex::new(None)),
            service_fullname: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

        *self.daemon.lock().unwrap() = Some(daemon);
        log::info!("mDNS daemon started");
        Ok(())
    }

    pub fn register(&self, port: u16, token_hint: &str) -> Result<(), String> {
        let daemon_lock = self.daemon.lock().unwrap();
        let daemon = daemon_lock.as_ref()
            .ok_or("mDNS daemon not started")?;

        let host = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "scanlink".to_string());

        let instance_name = format!("ScanLink ({})", host);

        // Get local IP
        let ip = local_ip_address::local_ip()
            .map_err(|e| format!("Failed to get local IP: {}", e))?;

        // Create properties with token hint for verification
        let properties = [
            ("version", "2.0"),
            ("hint", &token_hint.chars().take(8).collect::<String>()),
        ];

        let service_hostname = format!("{}.local.", host.replace(" ", "-").to_lowercase());

        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &instance_name,
            &service_hostname,
            ip,
            port,
            &properties[..],
        ).map_err(|e| format!("Failed to create service info: {}", e))?;

        let fullname = service_info.get_fullname().to_string();

        daemon.register(service_info)
            .map_err(|e| format!("Failed to register mDNS service: {}", e))?;

        *self.service_fullname.lock().unwrap() = Some(fullname);

        log::info!("mDNS service registered: {} on port {}", instance_name, port);

        Ok(())
    }

    pub fn unregister(&self) -> Result<(), String> {
        let fullname = self.service_fullname.lock().unwrap().take();

        if let Some(name) = fullname {
            if let Some(daemon) = self.daemon.lock().unwrap().as_ref() {
                daemon.unregister(&name)
                    .map_err(|e| format!("Failed to unregister mDNS service: {}", e))?;
                log::info!("mDNS service unregistered");
            }
        }

        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.unregister();

        if let Some(daemon) = self.daemon.lock().unwrap().take() {
            let _ = daemon.shutdown();
            log::info!("mDNS daemon stopped");
        }
    }
}

impl Clone for MdnsService {
    fn clone(&self) -> Self {
        Self {
            daemon: self.daemon.clone(),
            service_fullname: self.service_fullname.clone(),
        }
    }
}

impl Drop for MdnsService {
    fn drop(&mut self) {
        // Only cleanup if this is the last reference
        if Arc::strong_count(&self.daemon) == 1 {
            self.stop();
        }
    }
}
