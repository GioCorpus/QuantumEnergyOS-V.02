// ═══════════════════════════════════════════════════════════════════════
//  Scheduler Híbrido — Cuántico + Clásico
//
//  El scheduler más complejo del kernel: prioriza tareas cuánticas
//  (simulación molecular VQE, QAOA de red eléctrica) sobre tareas
//  clásicas, y maneja la DECOHERENCIA como restricción de tiempo real.
//
//  Problema central: los qubits fotónicos pierden coherencia con el tiempo
//  (pérdida de fotones en waveguides). Si una tarea cuántica es interrumpida
//  más de T_coherence, el estado se destruye → hay que reiniciar.
//
//  Solución: el scheduler trata T_coherence como un DEADLINE HARD
//  (similar a RT scheduling) y ajusta dinámicamente el presupuesto.
//
//  Recalibración dinámica: cuando el chip detecta deriva > 10 mrad,
//  el scheduler pausa tareas cuánticas y lanza recalibración automática.
// ═══════════════════════════════════════════════════════════════════════

#![no_std]
extern crate alloc;

use alloc::collections::BinaryHeap;
use alloc::vec::Vec;
use core::cmp::{Ord, Ordering};
use heapless::Vec as HVec;
use spin::Mutex;

/// Tick del scheduler en nanosegundos
pub const SCHEDULER_TICK_NS:     u64 = 100_000; // 100 μs
/// Tiempo de coherencia típico en waveguide Si₃N₄ a temperatura ambiente
pub const WAVEGUIDE_T_COHERENCE_NS: u64 = 5_000_000; // 5 ms
/// Ventana de recalibración: máximo tiempo sin calibrar antes de pausa
pub const MAX_DRIFT_TIME_NS:     u64 = 1_000_000_000; // 1 s
/// Prioridad base de tareas cuánticas
pub const QUANTUM_PRIORITY_BASE: i32 = 100;

// ═══════════════════════════════════════════════════════════════════════
//  TIPOS DE TAREA
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    // ── Tareas cuánticas (prioridad alta + deadline de coherencia) ──
    QuantumVQE {
        n_modes:    u32,
        n_layers:   u32,
        molecule:   MoleculeTarget,
    },
    QuantumQAOA {
        n_nodes:    u32,
        p_layers:   u32,
        region:     GridRegion,
    },
    QuantumGKPCorrect {
        mode_idx:   u32,
        chip_id:    u8,
    },
    QuantumCalibrate {
        chip_id:    u8,
        full_sweep: bool,
    },
    QuantumMatMul {
        matrix_size: u32,
        chip_id:     u8,
    },
    // ── Tareas clásicas (prioridad normal) ──
    ClassicIO   { fd: u32, bytes: u64 },
    ClassicComp { priority: i32 },
    ClassicNet  { socket: u32 },
    // ── Tarea de sistema ──
    SystemIdle,
    SystemRecalibrate { chip_id: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoleculeTarget {
    H2,           // Hidrógeno molecular — benchmark
    H2O,          // Agua — aplicación energética
    N2,           // Nitrógeno — síntesis de amoniaco / fertilizantes
    FeS2,         // Pirita — catálisis energética
    Custom(u32),  // Molécula arbitraria (n_qubits)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridRegion {
    BajaCalifornia,  // Mexicali, Tijuana, Ensenada, Tecate
    Sonora,          // Hermosillo, Nogales, Ciudad Obregón
    Chihuahua,
    NorthwestMexico, // BC + Sonora + Chihuahua completo
    Custom(u32),     // N nodos arbitrarios
}

// ═══════════════════════════════════════════════════════════════════════
//  TAREA DEL SCHEDULER
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Task {
    pub id:          u64,
    pub task_type:   TaskType,
    pub priority:    i32,
    /// Deadline absoluto [ns desde boot] — None para tareas best-effort
    pub deadline_ns: Option<u64>,
    /// Coherencia restante [ns] — para tareas cuánticas
    pub coherence_budget_ns: Option<u64>,
    /// Tiempo de CPU acumulado [ns]
    pub cpu_time_ns: u64,
    /// Estado de la tarea
    pub state:       TaskState,
    /// Chip fotónico asignado
    pub chip_id:     Option<u8>,
    /// Prioridad dinámica (ajustada por scheduler)
    pub dynamic_priority: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    /// Bloqueada esperando chip fotónico disponible
    WaitingChip,
    /// Bloqueada esperando resultado homodyne
    WaitingMeasurement,
    /// Suspendida por deriva de fase — esperando recalibración
    SuspendedDrift,
    Completed,
    Failed { reason: &'static str },
}

impl Task {
    pub fn new_quantum_vqe(id: u64, molecule: MoleculeTarget, n_modes: u32, n_layers: u32) -> Self {
        let coherence_ns = WAVEGUIDE_T_COHERENCE_NS;
        // VQE tiene deadline = tiempo de coherencia
        let deadline = Some(coherence_ns * 10); // 10 ciclos de coherencia máximo

        Self {
            id,
            task_type: TaskType::QuantumVQE { n_modes, n_layers, molecule },
            priority:  QUANTUM_PRIORITY_BASE + 50,  // VQE = prioridad máxima
            deadline_ns: deadline,
            coherence_budget_ns: Some(coherence_ns),
            cpu_time_ns: 0,
            state:   TaskState::Ready,
            chip_id: None,
            dynamic_priority: QUANTUM_PRIORITY_BASE + 50,
        }
    }

    pub fn new_quantum_qaoa(id: u64, region: GridRegion, p_layers: u32) -> Self {
        let n_nodes = match &region {
            GridRegion::BajaCalifornia  => 6,
            GridRegion::Sonora          => 8,
            GridRegion::Chihuahua       => 7,
            GridRegion::NorthwestMexico => 21,
            GridRegion::Custom(n)       => *n,
        };
        Self {
            id,
            task_type: TaskType::QuantumQAOA { n_nodes, p_layers, region },
            priority:  QUANTUM_PRIORITY_BASE + 40,
            deadline_ns: Some(WAVEGUIDE_T_COHERENCE_NS * 5),
            coherence_budget_ns: Some(WAVEGUIDE_T_COHERENCE_NS),
            cpu_time_ns: 0,
            state:   TaskState::Ready,
            chip_id: None,
            dynamic_priority: QUANTUM_PRIORITY_BASE + 40,
        }
    }

    pub fn new_classic(id: u64, priority: i32) -> Self {
        Self {
            id,
            task_type: TaskType::ClassicComp { priority },
            priority,
            deadline_ns: None,
            coherence_budget_ns: None,
            cpu_time_ns: 0,
            state:   TaskState::Ready,
            chip_id: None,
            dynamic_priority: priority,
        }
    }

    pub fn new_calibration(id: u64, chip_id: u8) -> Self {
        Self {
            id,
            task_type: TaskType::QuantumCalibrate { chip_id, full_sweep: true },
            priority:  QUANTUM_PRIORITY_BASE + 80,  // Recalibración = prioridad máxima absoluta
            deadline_ns: Some(SCHEDULER_TICK_NS * 5), // Debe completar en 5 ticks
            coherence_budget_ns: None,
            cpu_time_ns: 0,
            state:   TaskState::Ready,
            chip_id: Some(chip_id),
            dynamic_priority: QUANTUM_PRIORITY_BASE + 80,
        }
    }

    pub fn is_quantum(&self) -> bool {
        matches!(self.task_type,
            TaskType::QuantumVQE { .. }  | TaskType::QuantumQAOA { .. }
            | TaskType::QuantumGKPCorrect { .. } | TaskType::QuantumCalibrate { .. }
            | TaskType::QuantumMatMul { .. }
        )
    }

    /// Reducir el presupuesto de coherencia por el tick del scheduler
    pub fn consume_coherence(&mut self, elapsed_ns: u64) -> CoherenceStatus {
        if let Some(ref mut budget) = self.coherence_budget_ns {
            if *budget <= elapsed_ns {
                *budget = 0;
                CoherenceStatus::Expired
            } else {
                *budget -= elapsed_ns;
                if *budget < SCHEDULER_TICK_NS * 3 {
                    CoherenceStatus::Critical { remaining_ns: *budget }
                } else {
                    CoherenceStatus::Ok { remaining_ns: *budget }
                }
            }
        } else {
            CoherenceStatus::NotApplicable
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CoherenceStatus {
    Ok { remaining_ns: u64 },
    Critical { remaining_ns: u64 },
    Expired,
    NotApplicable,
}

// Necesario para BinaryHeap
impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for Task {}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // Max-heap por prioridad dinámica, luego por deadline
        self.dynamic_priority.cmp(&other.dynamic_priority)
            .then_with(|| {
                match (self.deadline_ns, other.deadline_ns) {
                    (Some(a), Some(b)) => b.cmp(&a), // deadline más cercano = mayor prioridad
                    (Some(_), None)    => Ordering::Greater,
                    (None, Some(_))    => Ordering::Less,
                    (None, None)       => Ordering::Equal,
                }
            })
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  SCHEDULER HÍBRIDO
// ═══════════════════════════════════════════════════════════════════════

pub struct HybridScheduler {
    /// Cola de prioridad (max-heap)
    ready_queue:        Vec<Task>,
    /// Tarea en ejecución actualmente
    running:            Option<Task>,
    /// Tiempo actual del sistema [ns desde boot]
    pub current_time_ns: u64,
    /// Estado de los chips fotónicos
    chip_states:        HVec<ChipSchedulerState, 4>,
    /// Estadísticas
    pub stats:          SchedulerStats,
}

#[derive(Debug, Clone)]
pub struct ChipSchedulerState {
    pub chip_id:            u8,
    pub occupied:           bool,
    pub last_calibration_ns: u64,
    pub drift_mrad:         f64,
    pub temperature_k:      f64,
}

impl ChipSchedulerState {
    pub fn needs_recalibration(&self, current_time_ns: u64) -> bool {
        let time_since_calib = current_time_ns - self.last_calibration_ns;
        time_since_calib > MAX_DRIFT_TIME_NS || self.drift_mrad.abs() > 10.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct SchedulerStats {
    pub total_tasks_scheduled:     u64,
    pub quantum_tasks_completed:   u64,
    pub classic_tasks_completed:   u64,
    pub coherence_expirations:     u32,
    pub forced_recalibrations:     u32,
    pub avg_quantum_latency_ns:    f64,
    pub avg_classic_latency_ns:    f64,
}

impl HybridScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue:      Vec::new(),
            running:          None,
            current_time_ns:  0,
            chip_states:      HVec::new(),
            stats:            SchedulerStats::default(),
        }
    }

    pub fn register_chip(&mut self, chip_id: u8, temp_k: f64) {
        self.chip_states.push(ChipSchedulerState {
            chip_id,
            occupied: false,
            last_calibration_ns: 0,
            drift_mrad: 0.0,
            temperature_k: temp_k,
        }).ok();
    }

    /// Añadir tarea a la cola de listo
    pub fn submit(&mut self, mut task: Task) {
        // Ajustar prioridad dinámica para tareas con deadline inminente
        if let Some(deadline) = task.deadline_ns {
            let slack = deadline.saturating_sub(self.current_time_ns);
            if slack < SCHEDULER_TICK_NS * 5 {
                // Urgency boost: aumentar prioridad si el deadline es inminente
                task.dynamic_priority += 50;
            }
        }
        self.ready_queue.push(task);
        // Re-sort (simulating BinaryHeap behaviour without std)
        self.ready_queue.sort_by(|a, b| b.cmp(a));
        self.stats.total_tasks_scheduled += 1;
    }

    /// Tick del scheduler — ejecutar un paso de scheduling
    pub fn tick(&mut self) -> SchedulerDecision {
        self.current_time_ns += SCHEDULER_TICK_NS;

        // ── 1. Verificar coherencia de la tarea en ejecución ──────────
        if let Some(ref mut running) = self.running {
            let status = running.consume_coherence(SCHEDULER_TICK_NS);
            match status {
                CoherenceStatus::Expired => {
                    // ¡Coherencia perdida! La tarea cuántica debe reiniciarse
                    let task_id = running.id;
                    self.stats.coherence_expirations += 1;
                    let failed_task = self.running.take().unwrap();
                    return SchedulerDecision::TaskFailed {
                        task_id:      failed_task.id,
                        reason:       "coherencia perdida en waveguide",
                        reschedule:   failed_task.is_quantum(),
                    };
                }
                CoherenceStatus::Critical { remaining_ns } => {
                    // Advertencia: coherencia crítica — completar o interrumpir
                    if remaining_ns < SCHEDULER_TICK_NS {
                        let task = self.running.take().unwrap();
                        return SchedulerDecision::PreemptQuantum {
                            task_id: task.id,
                            reason:  "presupuesto de coherencia agotado",
                        };
                    }
                }
                _ => {}
            }
        }

        // ── 2. Verificar si algún chip necesita recalibración ─────────
        for chip in self.chip_states.iter() {
            if chip.needs_recalibration(self.current_time_ns) {
                // Crear tarea de calibración con máxima prioridad
                let calib_task = Task::new_calibration(
                    self.current_time_ns, // usar tiempo como ID
                    chip.chip_id,
                );
                self.stats.forced_recalibrations += 1;
                // Suspender tarea cuántica actual si está usando este chip
                if let Some(ref running) = self.running {
                    if running.chip_id == Some(chip.chip_id) && running.is_quantum() {
                        let task = self.running.take().unwrap();
                        self.ready_queue.push(task);
                    }
                }
                self.ready_queue.insert(0, calib_task); // Insertar al frente
                return SchedulerDecision::Recalibrate { chip_id: chip.chip_id };
            }
        }

        // ── 3. Si no hay tarea en ejecución, seleccionar la siguiente ──
        if self.running.is_none() && !self.ready_queue.is_empty() {
            let next = self.ready_queue.remove(0);

            // Para tareas cuánticas: verificar que el chip esté disponible
            if next.is_quantum() {
                let chip_available = self.chip_states.iter()
                    .any(|c| !c.occupied && next.chip_id.map(|id| id == c.chip_id).unwrap_or(true));

                if !chip_available {
                    // Chip ocupado: pasar a WaitingChip
                    let mut waiting = next;
                    waiting.state = TaskState::WaitingChip;
                    self.ready_queue.push(waiting);
                    return SchedulerDecision::Idle;
                }
            }

            let task_id   = next.id;
            let task_type = next.task_type.clone();
            let is_quantum = next.is_quantum();
            self.running  = Some(next);

            return SchedulerDecision::Schedule {
                task_id,
                task_type,
                is_quantum,
                estimated_duration_ns: if is_quantum { 500_000 } else { 100_000 },
            };
        }

        // ── 4. Tarea en ejecución → continuar ─────────────────────────
        if let Some(ref running) = self.running {
            SchedulerDecision::Continue {
                task_id:   running.id,
                is_quantum: running.is_quantum(),
            }
        } else {
            SchedulerDecision::Idle
        }
    }

    /// Notificar al scheduler que una tarea completó
    pub fn complete_task(&mut self, task_id: u64, success: bool) {
        if let Some(task) = self.running.take() {
            if task.id == task_id {
                if task.is_quantum() {
                    self.stats.quantum_tasks_completed += 1;
                } else {
                    self.stats.classic_tasks_completed += 1;
                }
                // Liberar el chip
                if let Some(chip_id) = task.chip_id {
                    if let Some(chip) = self.chip_states.iter_mut()
                                           .find(|c| c.chip_id == chip_id) {
                        chip.occupied = false;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum SchedulerDecision {
    Schedule {
        task_id:                u64,
        task_type:              TaskType,
        is_quantum:             bool,
        estimated_duration_ns:  u64,
    },
    Continue {
        task_id:   u64,
        is_quantum: bool,
    },
    PreemptQuantum {
        task_id: u64,
        reason:  &'static str,
    },
    TaskFailed {
        task_id:    u64,
        reason:     &'static str,
        reschedule: bool,
    },
    Recalibrate {
        chip_id: u8,
    },
    Idle,
}
