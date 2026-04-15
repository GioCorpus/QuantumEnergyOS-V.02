// ═══════════════════════════════════════════════════════════════════════
//  appStore.ts — Estado global Zustand + bridge Tauri ↔ Rust
// ═══════════════════════════════════════════════════════════════════════

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

// ── Tipos (espejo de los tipos Rust) ─────────────────────────────────
export interface QuartzStatus {
  initialized:       boolean;
  layers_active:     number;
  majorana_qubits:   number;
  coherence_time_ms: number;
  layer_names:       string[];
}

export interface GridNode {
  id:          number;
  name:        string;
  load_kw:     number;
  capacity_kw: number;
}

export interface GridBalanceResult {
  n_nodes:         number;
  energy_saved_kw: number;
  optimal_config:  number[];
  qaoa_energy:     number;
  quartz_hash:     string;
  execution_ms:    number;
  node_states:     NodeState[];
}

export interface NodeState {
  node_id:   number;
  active:    boolean;
  load_pct:  number;
  name:      string;
}

export interface BatteryState {
  level_pct:     number;
  charging:      boolean;
  voltage_v:     number | null;
  temperature_c: number | null;
  health:        string;
}

export interface SolarForecast {
  location:          string;
  risk_level:        "LOW" | "MEDIUM" | "HIGH" | "EXTREME";
  kp_index:          number;
  alert_message:     string;
  grid_impact_pct:   number;
  recommendation:    string;
  next_event_hours:  number | null;
}

export interface QuartzPrediction {
  hours_ahead:            number;
  n_nodes:                number;
  layers:                 QuartzLayer[];
  grid_efficiency:        number;
  braid_operations:       number;
  topological_protection: number;
}

export interface QuartzLayer {
  id:           number;
  name:         string;
  amplitude:    number;
  phase_rad:    number;
  entanglement: number;
  active:       boolean;
}

// ── Store ─────────────────────────────────────────────────────────────
interface AppStore {
  // Estado
  quartz:      QuartzStatus | null;
  battery:     BatteryState | null;
  solar:       SolarForecast | null;
  solarRisk:   string;
  gridResult:  GridBalanceResult | null;
  prediction:  QuartzPrediction | null;
  loading:     Record<string, boolean>;
  error:       string | null;

  // Acciones
  init:             () => Promise<void>;
  refreshBattery:   () => Promise<void>;
  refreshSolar:     () => Promise<void>;
  balanceGrid:      (nodes: GridNode[], layers?: number) => Promise<void>;
  predictGrid:      (hours: number, nodes: number) => Promise<void>;
  storeQuartz:      (key: string, data: unknown, layer: number) => Promise<string>;
  retrieveQuartz:   (key: string) => Promise<unknown>;
  autoRepair:       (nodeId: number) => Promise<void>;
  clearError:       () => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  quartz:     null,
  battery:    null,
  solar:      null,
  solarRisk:  "LOW",
  gridResult: null,
  prediction: null,
  loading:    {},
  error:      null,

  init: async () => {
    set({ loading: { init: true } });
    try {
      // Inicializar Cuarzo 4D
      const quartz = await invoke<QuartzStatus>("init_quartz4d");
      set({ quartz });

      // Leer batería y estado solar en paralelo
      await Promise.all([
        get().refreshBattery(),
        get().refreshSolar(),
      ]);
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: {} });
    }
  },

  refreshBattery: async () => {
    try {
      const battery = await invoke<BatteryState>("get_device_battery");
      set({ battery });
    } catch (e) {
      console.warn("Battery read failed:", e);
    }
  },

  refreshSolar: async () => {
    try {
      // Mexicali, BC — coordenadas base
      const solar = await invoke<SolarForecast>("predict_solar_event", {
        lat: 32.6245, lon: -115.4523,
      });
      set({ solar, solarRisk: solar.risk_level });
    } catch (e) {
      console.warn("Solar forecast failed:", e);
    }
  },

  balanceGrid: async (nodes, layers = 2) => {
    set(s => ({ loading: { ...s.loading, grid: true }, error: null }));
    try {
      const result = await invoke<GridBalanceResult>("balance_grid", {
        nodes: nodes.map(n => ({
          id: n.id, name: n.name,
          load_kw: n.load_kw, capacity_kw: n.capacity_kw,
        })),
        pLayers: layers,
      });
      set({ gridResult: result });

      // Persistir en SQLite offline
      await invoke("save_grid_history", { result }).catch(() => {});
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set(s => ({ loading: { ...s.loading, grid: false } }));
    }
  },

  predictGrid: async (hours, nodes) => {
    set(s => ({ loading: { ...s.loading, predict: true } }));
    try {
      const prediction = await invoke<QuartzPrediction>("predict_grid_state", {
        hoursAhead: hours,
        nNodes: nodes,
      });
      set({ prediction });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set(s => ({ loading: { ...s.loading, predict: false } }));
    }
  },

  storeQuartz: async (key, data, layer) => {
    const result = await invoke<{ hash: string }>("store_quartz4d", {
      key, data, layer,
    });
    return result.hash;
  },

  retrieveQuartz: async (key) => {
    return invoke("retrieve_quartz4d", { key });
  },

  autoRepair: async (nodeId) => {
    set(s => ({ loading: { ...s.loading, repair: true } }));
    try {
      await invoke("auto_repair_grid", { faultNodeId: nodeId });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set(s => ({ loading: { ...s.loading, repair: false } }));
    }
  },

  clearError: () => set({ error: null }),
}));

// ── Nodos predefinidos de Baja California ────────────────────────────
export const BC_NODES: GridNode[] = [
  { id: 0, name: "Mexicali",  load_kw: 85_000, capacity_kw: 120_000 },
  { id: 1, name: "Tijuana",   load_kw: 95_000, capacity_kw: 130_000 },
  { id: 2, name: "Ensenada",  load_kw: 42_000, capacity_kw: 65_000  },
  { id: 3, name: "Tecate",    load_kw: 18_000, capacity_kw: 30_000  },
  { id: 4, name: "San Felipe", load_kw: 8_500,  capacity_kw: 15_000  },
  { id: 5, name: "Rosarito",  load_kw: 22_000, capacity_kw: 35_000  },
];
