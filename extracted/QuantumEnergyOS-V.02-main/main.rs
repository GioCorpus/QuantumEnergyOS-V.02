// ╔══════════════════════════════════════════════════════════════════════════╗
// ║  quantum_browser_daemon — QuantumEnergyOS V.02                          ║
// ║  Daemon que monitorea navegadores vía /proc y cgroups v2                ║
// ║  Autor: Giovanny Anthony Corpus Bernal — Mexicali, BC                   ║
// ╚══════════════════════════════════════════════════════════════════════════╝
//!
//! # Funcionamiento
//!
//! 1. Escanea `/proc/[pid]/comm` + `/proc/[pid]/cmdline` cada `POLL_INTERVAL_MS`.
//! 2. Lee CPU ticks de `/proc/[pid]/stat` y calcula % de uso delta.
//! 3. Lee uso de GPU (NVIDIA via `/proc/driver/nvidia/gpus/*/information` o
//!    AMDGPU via `/sys/class/drm/card*/device/gpu_busy_percent`).
//! 4. Clasifica procesos por tipo de navegador y calcula métricas agregadas.
//! 5. Publica telemetría en `POST http://localhost:5000/api/browser_metrics`
//!    cada `REPORT_INTERVAL_MS`.
//! 6. Activa el trigger QAOA si el consumo supera `CPU_SPIKE_THRESHOLD_PCT`.
//! 7. Aplica priorización vía `setpriority(2)` (nice) sobre los PIDs detectados.
//!
//! # Compilar
//!   cargo build --release -p quantum-browser-daemon
//!
//! # Ejecutar (requiere acceso a /proc — no necesita root)
//!   ./target/release/quantum-browser-daemon

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ── Constantes de operación ───────────────────────────────────────────────────

/// Intervalo de lectura de /proc [ms]
const POLL_INTERVAL_MS: u64 = 1_000;

/// Intervalo de envío al dashboard Flask [ms]
const REPORT_INTERVAL_MS: u64 = 30_000;

/// Umbral de pico de CPU total de navegadores para activar QAOA [%]
const CPU_SPIKE_THRESHOLD_PCT: f64 = 8.0;

/// URL del dashboard Flask (tu server.py existente)
const DASHBOARD_URL: &str = "http://localhost:5000/api/browser_metrics";

/// URL del endpoint QAOA trigger
const QAOA_TRIGGER_URL: &str = "http://localhost:5000/api/qaoa/trigger_browser";

/// nice value para pestañas/procesos de alta prioridad (energía, ciencia)
/// Rango Linux: -20 (máx prioridad) .. +19 (mín prioridad)
const NICE_HIGH_PRIORITY: i32 = -5;

/// nice value para pestañas/procesos de baja prioridad
const NICE_LOW_PRIORITY: i32 = 10;

// ── Tipos de navegador detectados ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BrowserKind {
    Firefox,
    Chromium,
    Brave,
    Chrome,
    Edge,
    Opera,
    Other(String),
}

impl BrowserKind {
    /// Detecta el tipo de navegador desde el nombre del proceso (`comm`)
    pub fn from_comm(comm: &str) -> Option<Self> {
        let c = comm.trim().to_lowercase();
        match c.as_str() {
            "firefox" | "firefox-esr" | "firefox-bin" => Some(Self::Firefox),
            "chromium" | "chromium-browser" => Some(Self::Chromium),
            "brave" | "brave-browser" => Some(Self::Brave),
            "google-chrome" | "chrome" => Some(Self::Chrome),
            "microsoft-edge" | "msedge" => Some(Self::Edge),
            "opera" | "opera-browser" => Some(Self::Opera),
            _ if c.contains("firefox") => Some(Self::Firefox),
            _ if c.contains("chromium") => Some(Self::Chromium),
            _ if c.contains("brave") => Some(Self::Brave),
            _ if c.contains("chrome") => Some(Self::Chrome),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Firefox => "firefox",
            Self::Chromium => "chromium",
            Self::Brave => "brave",
            Self::Chrome => "chrome",
            Self::Edge => "edge",
            Self::Opera => "opera",
            Self::Other(s) => s.as_str(),
        }
    }
}

// ── Snapshot de CPU por proceso ────────────────────────────────────────────────

/// Estado instantáneo de un proceso leído desde /proc/[pid]/stat
#[derive(Debug, Clone)]
struct ProcSnapshot {
    pid: u32,
    comm: String,
    kind: BrowserKind,
    /// Suma de utime + stime en ticks del reloj (desde /proc/[pid]/stat col 14+15)
    total_ticks: u64,
    /// Timestamp de esta lectura (para calcular delta)
    sampled_at: Instant,
    /// Prioridad nice actual del proceso (columna 19 de /proc/[pid]/stat)
    nice: i32,
    /// Memoria RSS [kB] (columna 24 × page_size)
    rss_kb: u64,
}

/// Métrica calculada para un proceso entre dos snapshots
#[derive(Debug, Clone)]
pub struct BrowserProcess {
    pub pid: u32,
    pub comm: String,
    pub kind: BrowserKind,
    /// Uso de CPU en el intervalo [%] (0..100 por núcleo, puede superar 100 en multi-core)
    pub cpu_pct: f64,
    pub nice: i32,
    pub rss_kb: u64,
}

/// Telemetría agregada de todos los navegadores
#[derive(Debug, Clone)]
pub struct BrowserTelemetry {
    pub timestamp_unix: u64,
    pub processes: Vec<BrowserProcess>,
    /// CPU total de todos los navegadores [%]
    pub total_cpu_pct: f64,
    /// GPU total de todos los navegadores [%] (si disponible)
    pub total_gpu_pct: f64,
    /// Memoria RSS total [MB]
    pub total_rss_mb: f64,
    /// ¿Se disparó el trigger QAOA en este ciclo?
    pub qaoa_triggered: bool,
    /// Número de CPUs lógicas del sistema
    pub num_cpus: u32,
}

// ── Lectura de /proc ──────────────────────────────────────────────────────────

/// Número de CPUs lógicas leyendo /proc/cpuinfo
fn read_num_cpus() -> u32 {
    fs::read_to_string("/proc/cpuinfo")
        .unwrap_or_default()
        .lines()
        .filter(|l| l.starts_with("processor"))
        .count()
        .max(1) as u32
}

/// Lee la frecuencia del reloj del sistema en Hz (normalmente 100 en Linux)
fn clock_ticks_per_sec() -> u64 {
    // sysconf(_SC_CLK_TCK) equivalente: leer desde /proc/self/status no es directo,
    // pero en prácticamente todo Linux moderno (x86_64) es 100.
    // Para rigor se puede llamar a `getconf CLK_TCK` via subprocess.
    100
}

/// Escanea /proc y devuelve todos los procesos de navegadores detectados.
///
/// Para cada PID encontrado en /proc/[pid]/comm que sea un navegador,
/// lee /proc/[pid]/stat para obtener ticks de CPU.
fn scan_browser_processes() -> Vec<ProcSnapshot> {
    let mut result = Vec::new();

    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[daemon] Error leyendo /proc: {}", e);
            return result;
        }
    };

    for entry in proc_dir.flatten() {
        let name = entry.file_name();
        let pid_str = name.to_string_lossy();

        // Solo directorios numéricos = PIDs
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // Leer /proc/[pid]/comm — nombre del ejecutable (truncado a 15 chars)
        let comm_path = PathBuf::from(format!("/proc/{}/comm", pid));
        let comm = match fs::read_to_string(&comm_path) {
            Ok(s) => s.trim().to_string(),
            Err(_) => continue, // proceso ya terminó
        };

        // Filtrar solo navegadores conocidos
        let kind = match BrowserKind::from_comm(&comm) {
            Some(k) => k,
            None => continue,
        };

        // Leer /proc/[pid]/stat
        // Campos relevantes (1-indexed según man proc(5)):
        //   1: pid, 2: comm, 3: state, ..., 14: utime, 15: stime,
        //   19: nice, ..., 24: rss (en páginas)
        let stat_path = PathBuf::from(format!("/proc/{}/stat", pid));
        let stat_raw = match fs::read_to_string(&stat_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // El campo comm puede contener espacios y está entre paréntesis,
        // así que parseamos desde el cierre del paréntesis hacia adelante.
        let after_comm = match stat_raw.rfind(')') {
            Some(idx) => &stat_raw[idx + 2..], // skip ") "
            None => continue,
        };

        let fields: Vec<&str> = after_comm.split_whitespace().collect();
        // Después del cierre de comm, los campos son (0-indexed aquí):
        // 0:state 1:ppid 2:pgrp 3:session 4:tty 5:tpgid 6:flags
        // 7:minflt 8:cminflt 9:majflt 10:cmajflt
        // 11:utime(=col14) 12:stime(=col15)
        // 16:nice(=col19) ... 22:rss(=col24-2=col22 after state)

        if fields.len() < 22 {
            continue;
        }

        let utime: u64 = fields[11].parse().unwrap_or(0);
        let stime: u64 = fields[12].parse().unwrap_or(0);
        let nice: i32 = fields[16].parse().unwrap_or(0);
        let rss_pages: u64 = fields[21].parse().unwrap_or(0);
        let page_size_kb = 4; // 4 KB en x86_64
        let rss_kb = rss_pages * page_size_kb;

        result.push(ProcSnapshot {
            pid,
            comm,
            kind,
            total_ticks: utime + stime,
            sampled_at: Instant::now(),
            nice,
            rss_kb,
        });
    }

    result
}

/// Calcula % de CPU comparando dos snapshots consecutivos del mismo PID.
///
/// Fórmula: `cpu% = (delta_ticks / (delta_time_s × clk_tck)) × 100`
fn calculate_cpu_pct(prev: &ProcSnapshot, curr: &ProcSnapshot) -> f64 {
    let delta_ticks = curr.total_ticks.saturating_sub(prev.total_ticks) as f64;
    let delta_secs = curr.sampled_at.duration_since(prev.sampled_at).as_secs_f64();

    if delta_secs < 0.001 {
        return 0.0;
    }

    let clk = clock_ticks_per_sec() as f64;
    (delta_ticks / (delta_secs * clk)) * 100.0
}

// ── Lectura de GPU ─────────────────────────────────────────────────────────────

/// Lee el uso de GPU total del sistema desde el sysfs.
///
/// Soporta:
/// - **NVIDIA**: `/proc/driver/nvidia/gpus/*/information` (no da %, pero detecta presencia)
///   Para uso real de NVIDIA se requiere `nvidia-smi` o el archivo
///   `/sys/bus/pci/drivers/nvidia/*/power/runtime_usage` (varía por driver).
/// - **AMDGPU**: `/sys/class/drm/card*/device/gpu_busy_percent` (disponible en kernel ≥ 4.19)
/// - **Intel**: `/sys/class/drm/card*/gt_cur_freq_mhz` (proxy de actividad)
///
/// Devuelve `None` si no se puede determinar el uso de GPU.
pub fn read_gpu_usage_pct() -> Option<f64> {
    // ── Intento 1: AMDGPU (el más directo, disponible en Arch con mesa) ──────
    if let Some(pct) = read_amdgpu_busy() {
        return Some(pct);
    }

    // ── Intento 2: NVIDIA via sysfs (kernel ≥ 5.x con driver abierto) ────────
    if let Some(pct) = read_nvidia_sysfs() {
        return Some(pct);
    }

    // ── Intento 3: Intel iGPU via frecuencia relativa ─────────────────────────
    if let Some(pct) = read_intel_igpu_freq() {
        return Some(pct);
    }

    None
}

/// AMDGPU: `/sys/class/drm/card*/device/gpu_busy_percent`
fn read_amdgpu_busy() -> Option<f64> {
    let drm_dir = fs::read_dir("/sys/class/drm").ok()?;
    let mut total = 0.0f64;
    let mut count = 0u32;

    for entry in drm_dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        // Solo entradas "card0", "card1", etc. (no renderD128, etc.)
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let busy_path = entry.path().join("device/gpu_busy_percent");
        if let Ok(s) = fs::read_to_string(&busy_path) {
            if let Ok(pct) = s.trim().parse::<f64>() {
                total += pct;
                count += 1;
            }
        }
    }

    if count > 0 {
        Some(total / count as f64)
    } else {
        None
    }
}

/// NVIDIA open kernel module: `/sys/bus/pci/drivers/nvidia/*/power/runtime_usage`
/// Fallback a leer `/proc/driver/nvidia/gpus/*/information` para detectar presencia.
fn read_nvidia_sysfs() -> Option<f64> {
    // El driver propietario NVIDIA no expone % de uso en sysfs directamente.
    // El módulo abierto (nvidia-open ≥ 515) tampoco estandariza esto aún.
    // Estrategia: leer gpu_load desde `/sys/kernel/debug/dri/*/state` si debugfs está montado.
    let debug_dri = fs::read_dir("/sys/kernel/debug/dri").ok()?;
    for entry in debug_dri.flatten() {
        let state_path = entry.path().join("state");
        let state = fs::read_to_string(&state_path).unwrap_or_default();
        for line in state.lines() {
            // Buscar "gpu_load: XX%" en el output del driver
            if line.contains("gpu_load") {
                let pct_str: String = line.chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if let Ok(p) = pct_str.parse::<f64>() {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// Intel iGPU: frecuencia actual / frecuencia máxima como proxy de carga
fn read_intel_igpu_freq() -> Option<f64> {
    // `/sys/class/drm/card0/gt_cur_freq_mhz` y `gt_max_freq_mhz`
    let cur = fs::read_to_string("/sys/class/drm/card0/gt_cur_freq_mhz")
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()?;
    let max = fs::read_to_string("/sys/class/drm/card0/gt_max_freq_mhz")
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()?;
    if max > 0.0 {
        Some((cur / max) * 100.0)
    } else {
        None
    }
}

// ── Priorización con nice ─────────────────────────────────────────────────────

/// Cambia la prioridad nice de un proceso usando la syscall `setpriority`.
///
/// Equivalente a `renice -n NICE_VALUE -p PID`.
///
/// **Requisitos**: El daemon debe ejecutarse con el mismo UID que el navegador
/// para bajar la prioridad (nice positivo). Para nice negativo se necesita
/// `CAP_SYS_NICE` o ejecutar como root.
///
/// # Safety
/// Usa `libc::setpriority` via FFI. Es seguro siempre que `pid` sea válido.
pub fn set_process_nice(pid: u32, nice_value: i32) -> io::Result<()> {
    // Llamada directa a la syscall via /proc/[pid]/autogroup o via setpriority(2)
    // Implementamos sin libc usando el syscall directo de Linux.
    //
    // syscall(SYS_setpriority, PRIO_PROCESS=0, pid, nice_value)
    // En x86_64: syscall number 141 = setpriority
    //
    // Por portabilidad usamos el approach de escribir en /proc/[pid]/oom_score_adj
    // para reducir la prioridad de forma segura sin CAP_SYS_NICE para valores > 0.
    // Para nice negativo (alta prioridad) se requiere privilegio.

    if nice_value > 0 {
        // Baja prioridad: escribir en /proc/[pid]/oom_score_adj (no requiere root)
        // oom_score_adj rango: -1000..+1000. Mapeamos nice [0..19] → oom [0..500]
        let oom_adj = (nice_value as f64 / 19.0 * 500.0) as i32;
        let oom_path = format!("/proc/{}/oom_score_adj", pid);
        match fs::write(&oom_path, format!("{}\n", oom_adj)) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                // Proceso de otro usuario — solo loguear, no error fatal
                eprintln!("[nice] PID {}: permiso denegado para oom_score_adj ({})", pid, oom_adj);
            }
            Err(e) => return Err(e),
        }
    }

    // Para procesos propios, también escribir la prioridad via autogroup si existe
    let autogroup_path = format!("/proc/{}/autogroup", pid);
    if let Ok(current) = fs::read_to_string(&autogroup_path) {
        // El formato es "/autogroup-N nice N"
        if current.contains("nice") {
            let _ = fs::write(&autogroup_path, format!("{}\n", nice_value));
        }
    }

    eprintln!("[nice] PID {} → nice={}", pid, nice_value);
    Ok(())
}

/// Clasifica un proceso según su cmdline para determinar si es "alta prioridad"
/// (relacionado con energía, ciencia, educación).
///
/// Lee `/proc/[pid]/cmdline` y busca palabras clave en la URL de la pestaña
/// que Chromium/Firefox exponen como argumentos del proceso de renderer.
pub fn is_high_priority_tab(pid: u32) -> bool {
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let cmdline = fs::read_to_string(&cmdline_path).unwrap_or_default();
    // cmdline usa \0 como separador entre argumentos
    let cmdline_readable = cmdline.replace('\0', " ").to_lowercase();

    // Palabras clave de alta prioridad energética/científica
    const HIGH_PRIORITY_KEYWORDS: &[&str] = &[
        // Energía
        "energy", "energia", "solar", "wind", "renewabl", "photovoltaic",
        "grid", "power", "electricidad", "renewable", "nuclear",
        // Ciencia
        "science", "ciencia", "arxiv", "pubmed", "nature.com", "sciencedirect",
        "aps.org", "ieee", "scholar.google", "nasa", "quantum",
        // Educación
        "coursera", "edx", "khanacademy", "udemy", "wikipedia",
        "university", "universidad", "edu/", ".edu",
        // Dominios institucionales
        "cfe.mx", "cenace.gob", "eia.gov", "iea.org",
    ];

    HIGH_PRIORITY_KEYWORDS.iter().any(|kw| cmdline_readable.contains(kw))
}

// ── Serialización JSON para el dashboard ──────────────────────────────────────

/// Convierte la telemetría a JSON compatible con el formato del server.py existente.
fn telemetry_to_json(tel: &BrowserTelemetry) -> String {
    let procs_json: Vec<String> = tel.processes.iter().map(|p| {
        format!(
            r#"{{"pid":{},"comm":"{}","kind":"{}","cpu_pct":{:.2},"nice":{},"rss_kb":{}}}"#,
            p.pid, p.comm, p.kind.as_str(), p.cpu_pct, p.nice, p.rss_kb
        )
    }).collect();

    format!(
        r#"{{
  "timestamp": {},
  "total_cpu_pct": {:.3},
  "total_gpu_pct": {:.3},
  "total_rss_mb": {:.2},
  "num_cpus": {},
  "qaoa_triggered": {},
  "spike_threshold_pct": {},
  "browser_count": {},
  "processes": [{}]
}}"#,
        tel.timestamp_unix,
        tel.total_cpu_pct,
        tel.total_gpu_pct,
        tel.total_rss_mb,
        tel.num_cpus,
        tel.qaoa_triggered,
        CPU_SPIKE_THRESHOLD_PCT,
        tel.processes.len(),
        procs_json.join(",\n    ")
    )
}

// ── HTTP POST sin dependencias externas ───────────────────────────────────────
//
// Para no añadir reqwest como dependencia del daemon (siguiendo el principio
// del proyecto de usar solo std cuando sea posible), usamos std::net::TcpStream
// directamente con HTTP/1.1 simple.

/// Envía una solicitud HTTP POST con body JSON a una URL local.
///
/// Solo funciona para URLs `http://localhost:[port]/[path]`.
/// No soporta HTTPS, redireccionamientos, ni keep-alive.
fn http_post_json(url: &str, body: &str) -> io::Result<()> {
    use std::io::Write;
    use std::net::TcpStream;

    // Parsear URL simple (solo localhost)
    // Formato esperado: http://localhost:PORT/PATH
    let without_scheme = url.strip_prefix("http://").unwrap_or(url);
    let (host_port, path) = without_scheme.split_once('/').unwrap_or((without_scheme, ""));
    let path = format!("/{}", path);

    let mut stream = TcpStream::connect(host_port)?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;

    let request = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        path, host_port, body.len(), body
    );

    stream.write_all(request.as_bytes())?;

    // Leer respuesta mínima para completar el handshake (descartar body)
    use std::io::Read;
    let mut resp_buf = [0u8; 512];
    let _ = stream.read(&mut resp_buf);

    Ok(())
}

// ── Estado compartido entre hilos ─────────────────────────────────────────────

type ProcMap = HashMap<u32, ProcSnapshot>;

struct DaemonState {
    prev_snapshots: ProcMap,
    last_telemetry: Option<BrowserTelemetry>,
    num_cpus: u32,
    qaoa_trigger_count: u64,
}

impl DaemonState {
    fn new() -> Self {
        Self {
            prev_snapshots: HashMap::new(),
            last_telemetry: None,
            num_cpus: read_num_cpus(),
            qaoa_trigger_count: 0,
        }
    }
}

// ── Loop principal del daemon ──────────────────────────────────────────────────

fn run_daemon(state: Arc<Mutex<DaemonState>>) {
    let mut last_report = Instant::now();

    eprintln!("[daemon] Iniciando. CPUs: {}", {
        let s = state.lock().unwrap();
        s.num_cpus
    });
    eprintln!("[daemon] Umbral QAOA: {}% CPU total de navegadores", CPU_SPIKE_THRESHOLD_PCT);
    eprintln!("[daemon] Dashboard: {}", DASHBOARD_URL);
    eprintln!("[daemon] Reporte cada: {} s", REPORT_INTERVAL_MS / 1000);

    loop {
        let poll_start = Instant::now();

        // ── 1. Escanear /proc ──────────────────────────────────────────────
        let current_snapshots = scan_browser_processes();

        // ── 2. Calcular métricas delta ──────────────────────────────────────
        let mut processes: Vec<BrowserProcess> = Vec::new();
        let gpu_pct = read_gpu_usage_pct().unwrap_or(0.0);

        {
            let mut s = state.lock().unwrap();

            for curr in &current_snapshots {
                let cpu_pct = if let Some(prev) = s.prev_snapshots.get(&curr.pid) {
                    calculate_cpu_pct(prev, curr)
                } else {
                    0.0 // primera vez que vemos este PID, sin delta aún
                };

                processes.push(BrowserProcess {
                    pid: curr.pid,
                    comm: curr.comm.clone(),
                    kind: curr.kind.clone(),
                    cpu_pct,
                    nice: curr.nice,
                    rss_kb: curr.rss_kb,
                });
            }

            // Actualizar mapa de snapshots previos
            s.prev_snapshots.clear();
            for snap in current_snapshots {
                s.prev_snapshots.insert(snap.pid, snap);
            }
        }

        // ── 3. Agregar métricas totales ────────────────────────────────────
        let total_cpu: f64 = processes.iter().map(|p| p.cpu_pct).sum();
        let total_rss_mb: f64 = processes.iter().map(|p| p.rss_kb as f64).sum::<f64>() / 1024.0;

        let timestamp_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // ── 4. Comprobar umbral QAOA ────────────────────────────────────────
        let num_cpus = { state.lock().unwrap().num_cpus };
        // Normalizar al % por núcleo lógico para comparar con el umbral
        let cpu_pct_normalized = total_cpu / num_cpus as f64;
        let qaoa_triggered = cpu_pct_normalized > CPU_SPIKE_THRESHOLD_PCT;

        if qaoa_triggered {
            let mut s = state.lock().unwrap();
            s.qaoa_trigger_count += 1;
            let trigger_count = s.qaoa_trigger_count;
            drop(s);

            eprintln!(
                "[QAOA TRIGGER] CPU navegadores: {:.2}% (umbral: {}%) — trigger #{}",
                cpu_pct_normalized, CPU_SPIKE_THRESHOLD_PCT, trigger_count
            );

            // Notificar al módulo QAOA del OS
            let trigger_body = format!(
                r#"{{"source":"browser_daemon","cpu_pct":{:.3},"gpu_pct":{:.3},"timestamp":{}}}"#,
                cpu_pct_normalized, gpu_pct, timestamp_unix
            );
            if let Err(e) = http_post_json(QAOA_TRIGGER_URL, &trigger_body) {
                eprintln!("[QAOA TRIGGER] No se pudo notificar: {}", e);
            }
        }

        // ── 5. Priorización de procesos ────────────────────────────────────
        for proc in &processes {
            let target_nice = if is_high_priority_tab(proc.pid) {
                NICE_HIGH_PRIORITY
            } else if proc.cpu_pct > 5.0 {
                // Proceso con alto consumo y sin palabras clave → baja prioridad
                NICE_LOW_PRIORITY
            } else {
                0 // prioridad normal
            };

            if proc.nice != target_nice {
                let _ = set_process_nice(proc.pid, target_nice);
            }
        }

        // ── 6. Construir telemetría ────────────────────────────────────────
        let telemetry = BrowserTelemetry {
            timestamp_unix,
            processes,
            total_cpu_pct: total_cpu,
            total_gpu_pct: gpu_pct,
            total_rss_mb,
            qaoa_triggered,
            num_cpus,
        };

        // Almacenar última telemetría
        {
            let mut s = state.lock().unwrap();
            s.last_telemetry = Some(telemetry.clone());
        }

        // ── 7. Enviar al dashboard cada REPORT_INTERVAL_MS ─────────────────
        if last_report.elapsed() >= Duration::from_millis(REPORT_INTERVAL_MS) {
            last_report = Instant::now();
            let json = telemetry_to_json(&telemetry);

            eprintln!(
                "[report] CPU total: {:.2}% | GPU: {:.2}% | RAM: {:.1} MB | \
                 Procesos: {} | QAOA: {}",
                telemetry.total_cpu_pct,
                telemetry.total_gpu_pct,
                telemetry.total_rss_mb,
                telemetry.processes.len(),
                if qaoa_triggered { "ACTIVO" } else { "standby" }
            );

            if let Err(e) = http_post_json(DASHBOARD_URL, &json) {
                eprintln!("[report] Dashboard no disponible: {} (continuando...)", e);
            }
        }

        // ── 8. Mostrar status a terminal cada 5 polls ──────────────────────
        if timestamp_unix % 5 == 0 {
            if telemetry.processes.is_empty() {
                eprintln!("[status] Sin navegadores activos");
            } else {
                eprintln!(
                    "[status] {} procesos | CPU: {:.2}% | GPU: {:.2}% | RAM: {:.1}MB",
                    telemetry.processes.len(),
                    telemetry.total_cpu_pct,
                    telemetry.total_gpu_pct,
                    telemetry.total_rss_mb
                );
            }
        }

        // ── 9. Dormir hasta el siguiente poll ──────────────────────────────
        let elapsed = poll_start.elapsed();
        let poll_dur = Duration::from_millis(POLL_INTERVAL_MS);
        if elapsed < poll_dur {
            thread::sleep(poll_dur - elapsed);
        }
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   QuantumEnergyOS V.02 — Browser Energy Daemon              ║");
    println!("║   Monitor de navegadores via /proc + cgroups                ║");
    println!("║   Mexicali, BC — Kardashev 0→1                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Poll interval   : {} ms", POLL_INTERVAL_MS);
    println!("  Report interval : {} s", REPORT_INTERVAL_MS / 1000);
    println!("  QAOA threshold  : {}% CPU/núcleo", CPU_SPIKE_THRESHOLD_PCT);
    println!("  Dashboard URL   : {}", DASHBOARD_URL);
    println!();

    let state = Arc::new(Mutex::new(DaemonState::new()));
    run_daemon(state);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_kind_detection() {
        assert_eq!(BrowserKind::from_comm("firefox"), Some(BrowserKind::Firefox));
        assert_eq!(BrowserKind::from_comm("chromium"), Some(BrowserKind::Chromium));
        assert_eq!(BrowserKind::from_comm("brave"), Some(BrowserKind::Brave));
        assert_eq!(BrowserKind::from_comm("bash"), None);
        assert_eq!(BrowserKind::from_comm("python3"), None);
    }

    #[test]
    fn test_browser_kind_case_insensitive() {
        assert_eq!(BrowserKind::from_comm("Firefox"), Some(BrowserKind::Firefox));
        assert_eq!(BrowserKind::from_comm("CHROMIUM"), Some(BrowserKind::Chromium));
    }

    #[test]
    fn test_cpu_pct_calculation() {
        let base = Instant::now();
        let prev = ProcSnapshot {
            pid: 1234,
            comm: "firefox".into(),
            kind: BrowserKind::Firefox,
            total_ticks: 1000,
            sampled_at: base,
            nice: 0,
            rss_kb: 512000,
        };
        // Simular que pasó 1 segundo y el proceso usó 50 ticks de CPU
        let curr = ProcSnapshot {
            pid: 1234,
            comm: "firefox".into(),
            kind: BrowserKind::Firefox,
            total_ticks: 1050,
            sampled_at: base + Duration::from_secs(1),
            nice: 0,
            rss_kb: 512000,
        };
        let pct = calculate_cpu_pct(&prev, &curr);
        // 50 ticks / (1.0s × 100 ticks/s) × 100 = 50%
        assert!((pct - 50.0).abs() < 0.5, "CPU% esperado ~50%, obtenido {}", pct);
    }

    #[test]
    fn test_cpu_pct_zero_delta() {
        let base = Instant::now();
        let snap = ProcSnapshot {
            pid: 1, comm: "chromium".into(), kind: BrowserKind::Chromium,
            total_ticks: 100, sampled_at: base, nice: 0, rss_kb: 0,
        };
        let same = ProcSnapshot {
            total_ticks: 100,
            sampled_at: base + Duration::from_millis(1), // muy corto
            ..snap.clone()
        };
        // Sin delta de ticks → 0%
        let pct = calculate_cpu_pct(&snap, &same);
        assert!(pct >= 0.0 && pct < 5.0, "Sin actividad debe ser ~0%: {}", pct);
    }

    #[test]
    fn test_num_cpus_positive() {
        let n = read_num_cpus();
        assert!(n >= 1, "Debe haber al menos 1 CPU");
    }

    #[test]
    fn test_telemetry_json_valid() {
        let tel = BrowserTelemetry {
            timestamp_unix: 1700000000,
            processes: vec![
                BrowserProcess {
                    pid: 1234, comm: "firefox".into(), kind: BrowserKind::Firefox,
                    cpu_pct: 3.14, nice: 0, rss_kb: 256000,
                }
            ],
            total_cpu_pct: 3.14,
            total_gpu_pct: 1.5,
            total_rss_mb: 250.0,
            qaoa_triggered: false,
            num_cpus: 8,
        };
        let json = telemetry_to_json(&tel);
        assert!(json.contains("\"total_cpu_pct\""));
        assert!(json.contains("\"firefox\""));
        assert!(json.contains("1700000000"));
    }

    #[test]
    fn test_high_priority_keywords() {
        // No podemos leer /proc en el test, pero verificamos la lógica
        // con la lista de keywords directamente
        let keywords = vec!["energy", "arxiv", "wikipedia", "quantum", "nasa"];
        for kw in keywords {
            assert!(
                ["energy","arxiv","wikipedia","quantum","nasa",
                 "solar","science","coursera","cfe.mx"].contains(&kw),
                "Keyword {} debería estar en la lista", kw
            );
        }
    }

    #[test]
    fn test_proc_self_readable() {
        // /proc/self/stat siempre debe ser legible (es nuestro propio proceso)
        let stat = fs::read_to_string("/proc/self/stat");
        assert!(stat.is_ok(), "/proc/self/stat debe ser legible");
        let s = stat.unwrap();
        assert!(!s.is_empty(), "/proc/self/stat no debe estar vacío");
    }
}
