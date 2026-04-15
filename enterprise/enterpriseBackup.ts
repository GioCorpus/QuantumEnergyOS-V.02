// ═══════════════════════════════════════════════════════════════════════
//  enterpriseBackup.ts — Sistema de Backups Automatizados Enterprise
//  QuantumEnergyOS — Backup, restauración y recuperación de datos
// ═══════════════════════════════════════════════════════════════════════

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

// ═══════════════════════════════════════════════════════════════════════
//  Tipos de Backup
// ═══════════════════════════════════════════════════════════════════════

export type BackupStatus = "pending" | "in_progress" | "completed" | "failed";
export type BackupType = "full" | "incremental" | "differential";
export type BackupSchedule = "manual" | "hourly" | "daily" | "weekly" | "monthly";

export interface BackupConfig {
  id: string;
  name: string;
  type: BackupType;
  schedule: BackupSchedule;
  enabled: boolean;
  retentionDays: number;
  includeTables: string[];
  compress: boolean;
  encrypt: boolean;
}

export interface BackupJob {
  id: string;
  configId: string;
  configName: string;
  type: BackupType;
  status: BackupStatus;
  startTime: string;
  endTime: string | null;
  sizeBytes: number;
  filePath: string | null;
  errorMessage: string | null;
  tablesBackedUp: string[];
  recordsCount: Record<string, number>;
}

export interface RestorePoint {
  id: string;
  backupId: string;
  timestamp: string;
  sizeBytes: number;
  tables: string[];
  checksum: string;
}

export interface BackupStats {
  totalBackups: number;
  totalSize: number;
  lastBackup: string | null;
  nextScheduled: string | null;
  successRate: number;
  storageUsed: number;
}

// ═══════════════════════════════════════════════════════════════════════
//  Utilidades
// ═══════════════════════════════════════════════════════════════════════

function generateBackupId(): string {
  return `backup_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

function calculateChecksum(data: string): string {
  let hash = 0;
  for (let i = 0; i < data.length; i++) {
    const char = data.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash).toString(16).padStart(8, "0");
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function estimateNextBackup(schedule: BackupSchedule): string {
  const now = new Date();
  const next = new Date(now);

  switch (schedule) {
    case "hourly":
      next.setHours(next.getHours() + 1);
      break;
    case "daily":
      next.setDate(next.getDate() + 1);
      next.setHours(2, 0, 0, 0);
      break;
    case "weekly":
      next.setDate(next.getDate() + (7 - next.getDay()));
      next.setHours(2, 0, 0, 0);
      break;
    case "monthly":
      next.setMonth(next.getMonth() + 1);
      next.setDate(1);
      next.setHours(2, 0, 0, 0);
      break;
    default:
      return "Manual";
  }

  return next.toISOString();
}

// ═══════════════════════════════════════════════════════════════════════
//  Configuraciones de Backup Predefinidas
// ═══════════════════════════════════════════════════════════════════════

const DEFAULT_BACKUP_CONFIGS: BackupConfig[] = [
  {
    id: "config_full_daily",
    name: "Backup Completo Diario",
    type: "full",
    schedule: "daily",
    enabled: true,
    retentionDays: 30,
    includeTables: ["users", "roles", "permissions", "audit_logs", "grid_history", "quartz_data", "solar_data"],
    compress: true,
    encrypt: true,
  },
  {
    id: "config_incremental_hourly",
    name: "Backup Incremental Horario",
    type: "incremental",
    schedule: "hourly",
    enabled: true,
    retentionDays: 7,
    includeTables: ["audit_logs", "grid_history"],
    compress: true,
    encrypt: false,
  },
  {
    id: "config_weekly_full",
    name: "Backup Completo Semanal",
    type: "full",
    schedule: "weekly",
    enabled: true,
    retentionDays: 90,
    includeTables: ["users", "roles", "permissions", "audit_logs", "grid_history", "quartz_data", "solar_data", "system_config"],
    compress: true,
    encrypt: true,
  },
];

// ═══════════════════════════════════════════════════════════════════════
//  Backups Demo
// ═══════════════════════════════════════════════════════════════════════

const DEMO_BACKUP_JOBS: BackupJob[] = [
  {
    id: "backup_001",
    configId: "config_full_daily",
    configName: "Backup Completo Diario",
    type: "full",
    status: "completed",
    startTime: "2026-04-03T02:00:00Z",
    endTime: "2026-04-03T02:01:30Z",
    sizeBytes: 2_456_789_012,
    filePath: "/backups/2026-04-03/full_daily_001.qeosbak",
    errorMessage: null,
    tablesBackedUp: ["users", "roles", "permissions", "audit_logs", "grid_history", "quartz_data", "solar_data"],
    recordsCount: {
      users: 156,
      roles: 5,
      permissions: 28,
      audit_logs: 45_230,
      grid_history: 12_450,
      quartz_data: 8_920,
      solar_data: 3_240,
    },
  },
  {
    id: "backup_002",
    configId: "config_incremental_hourly",
    configName: "Backup Incremental Horario",
    type: "incremental",
    status: "completed",
    startTime: "2026-04-03T10:00:00Z",
    endTime: "2026-04-03T10:00:25Z",
    sizeBytes: 125_678_901,
    filePath: "/backups/2026-04-03/incremental_hourly_042.qeosbak",
    errorMessage: null,
    tablesBackedUp: ["audit_logs", "grid_history"],
    recordsCount: {
      audit_logs: 1_230,
      grid_history: 450,
    },
  },
  {
    id: "backup_003",
    configId: "config_full_daily",
    configName: "Backup Completo Diario",
    type: "full",
    status: "completed",
    startTime: "2026-04-02T02:00:00Z",
    endTime: "2026-04-02T02:01:45Z",
    sizeBytes: 2_389_456_123,
    filePath: "/backups/2026-04-02/full_daily_001.qeosbak",
    errorMessage: null,
    tablesBackedUp: ["users", "roles", "permissions", "audit_logs", "grid_history", "quartz_data", "solar_data"],
    recordsCount: {
      users: 153,
      roles: 5,
      permissions: 28,
      audit_logs: 42_100,
      grid_history: 12_000,
      quartz_data: 8_800,
      solar_data: 3_100,
    },
  },
  {
    id: "backup_004",
    configId: "config_weekly_full",
    configName: "Backup Completo Semanal",
    type: "full",
    status: "completed",
    startTime: "2026-03-30T02:00:00Z",
    endTime: "2026-03-30T02:03:20Z",
    sizeBytes: 2_890_123_456,
    filePath: "/backups/2026-03-30/weekly_full_001.qeosbak",
    errorMessage: null,
    tablesBackedUp: ["users", "roles", "permissions", "audit_logs", "grid_history", "quartz_data", "solar_data", "system_config"],
    recordsCount: {
      users: 150,
      roles: 5,
      permissions: 28,
      audit_logs: 38_900,
      grid_history: 11_500,
      quartz_data: 8_600,
      solar_data: 2_900,
      system_config: 45,
    },
  },
  {
    id: "backup_005",
    configId: "config_incremental_hourly",
    configName: "Backup Incremental Horario",
    type: "incremental",
    status: "failed",
    startTime: "2026-04-03T09:00:00Z",
    endTime: "2026-04-03T09:00:15Z",
    sizeBytes: 0,
    filePath: null,
    errorMessage: "Connection timeout - database not reachable",
    tablesBackedUp: [],
    recordsCount: {},
  },
];

// ═══════════════════════════════════════════════════════════════════════
//  Store de Backups
// ═══════════════════════════════════════════════════════════════════════

interface BackupStore {
  configs: BackupConfig[];
  jobs: BackupJob[];
  isRunning: boolean;
  
  // Acciones
  createBackup: (configId: string) => Promise<BackupJob>;
  restoreBackup: (backupId: string) => Promise<boolean>;
  createConfig: (config: Omit<BackupConfig, "id">) => Promise<BackupConfig>;
  updateConfig: (configId: string, updates: Partial<BackupConfig>) => Promise<boolean>;
  deleteConfig: (configId: string) => Promise<boolean>;
  toggleConfig: (configId: string) => Promise<boolean>;
  getBackupById: (backupId: string) => BackupJob | undefined;
  getStats: () => BackupStats;
  getRestorePoints: (backupId?: string) => RestorePoint[];
  cleanupOldBackups: () => void;
}

// ═══════════════════════════════════════════════════════════════════════

export const useEnterpriseBackup = create<BackupStore>()(
  persist(
    (set, get) => ({
      configs: DEFAULT_BACKUP_CONFIGS,
      jobs: DEMO_BACKUP_JOBS,
      isRunning: false,

      createBackup: async (configId: string): Promise<BackupJob> => {
        const config = get().configs.find(c => c.id === configId);
        if (!config) {
          throw new Error("Configuración de backup no encontrada");
        }

        const job: BackupJob = {
          id: generateBackupId(),
          configId: config.id,
          configName: config.name,
          type: config.type,
          status: "in_progress",
          startTime: new Date().toISOString(),
          endTime: null,
          sizeBytes: 0,
          filePath: null,
          errorMessage: null,
          tablesBackedUp: config.includeTables,
          recordsCount: {},
        };

        set(state => ({
          jobs: [job, ...state.jobs],
          isRunning: true,
        }));

        // Simular proceso de backup
        try {
          await new Promise(resolve => setTimeout(resolve, 2000));
          
          // Calcular tamaño basado en tablas
          const estimatedSize = config.includeTables.length * 150_000_000;
          
          const completedJob: BackupJob = {
            ...job,
            status: "completed",
            endTime: new Date().toISOString(),
            sizeBytes: estimatedSize,
            filePath: `/backups/${new Date().toISOString().split("T")[0]}/${config.type}_${config.schedule}_${job.id.split("_")[1]}.qeosbak`,
            recordsCount: config.includeTables.reduce((acc, table) => {
              acc[table] = Math.floor(Math.random() * 10000) + 1000;
              return acc;
            }, {} as Record<string, number>),
          };

          set(state => ({
            jobs: state.jobs.map(j => j.id === job.id ? completedJob : j),
            isRunning: false,
          }));

          return completedJob;
        } catch (error) {
          const failedJob: BackupJob = {
            ...job,
            status: "failed",
            endTime: new Date().toISOString(),
            errorMessage: String(error),
          };

          set(state => ({
            jobs: state.jobs.map(j => j.id === job.id ? failedJob : j),
            isRunning: false,
          }));

          return failedJob;
        }
      },

      restoreBackup: async (backupId: string): Promise<boolean> => {
        const backup = get().jobs.find(j => j.id === backupId);
        if (!backup) {
          throw new Error("Backup no encontrado");
        }

        if (backup.status !== "completed") {
          throw new Error("Solo se pueden restaurar backups completados");
        }

        // Simular restauración
        await new Promise(resolve => setTimeout(resolve, 3000));

        return true;
      },

      createConfig: async (configData: Omit<BackupConfig, "id">): Promise<BackupConfig> => {
        const newConfig: BackupConfig = {
          ...configData,
          id: `config_${generateBackupId()}`,
        };

        set(state => ({
          configs: [...state.configs, newConfig],
        }));

        return newConfig;
      },

      updateConfig: async (configId: string, updates: Partial<BackupConfig>): Promise<boolean> => {
        set(state => ({
          configs: state.configs.map(c => 
            c.id === configId ? { ...c, ...updates } : c
          ),
        }));

        return true;
      },

      deleteConfig: async (configId: string): Promise<boolean> => {
        set(state => ({
          configs: state.configs.filter(c => c.id !== configId),
        }));

        return true;
      },

      toggleConfig: async (configId: string): Promise<boolean> => {
        set(state => ({
          configs: state.configs.map(c => 
            c.id === configId ? { ...c, enabled: !c.enabled } : c
          ),
        }));

        return true;
      },

      getBackupById: (backupId: string): BackupJob | undefined => {
        return get().jobs.find(j => j.id === backupId);
      },

      getStats: (): BackupStats => {
        const jobs = get().jobs;
        const completedJobs = jobs.filter(j => j.status === "completed");
        const totalSize = completedJobs.reduce((sum, j) => sum + j.sizeBytes, 0);
        
        const lastBackup = completedJobs.length > 0 
          ? completedJobs[0].endTime 
          : null;

        // Calcular siguiente backup programado
        const enabledConfigs = get().configs.filter(c => c.enabled);
        const nextSchedules = enabledConfigs.map(c => estimateNextBackup(c.schedule));
        const nextScheduled = nextSchedules.length > 0 
          ? nextSchedules.sort()[0] 
          : null;

        const successRate = jobs.length > 0
          ? (completedJobs.length / jobs.length) * 100
          : 100;

        return {
          totalBackups: completedJobs.length,
          totalSize,
          lastBackup,
          nextScheduled,
          successRate,
          storageUsed: totalSize,
        };
      },

      getRestorePoints: (backupId?: string): RestorePoint[] => {
        const completedJobs = get().jobs
          .filter(j => j.status === "completed")
          .sort((a, b) => new Date(b.endTime!).getTime() - new Date(a.endTime!).getTime());

        if (backupId) {
          const job = completedJobs.find(j => j.id === backupId);
          if (!job) return [];

          return [{
            id: `restore_${job.id}`,
            backupId: job.id,
            timestamp: job.endTime!,
            sizeBytes: job.sizeBytes,
            tables: job.tablesBackedUp,
            checksum: calculateChecksum(job.id + job.sizeBytes.toString()),
          }];
        }

        return completedJobs.map(job => ({
          id: `restore_${job.id}`,
          backupId: job.id,
          timestamp: job.endTime!,
          sizeBytes: job.sizeBytes,
          tables: job.tablesBackedUp,
          checksum: calculateChecksum(job.id + job.sizeBytes.toString()),
        }));
      },

      cleanupOldBackups: () => {
        const configs = get().configs;
        
        set(state => ({
          jobs: state.jobs.filter(job => {
            const config = configs.find(c => c.id === job.configId);
            if (!config) return false;
            
            const jobDate = new Date(job.startTime);
            const cutoffDate = new Date();
            cutoffDate.setDate(cutoffDate.getDate() - config.retentionDays);
            
            return jobDate >= cutoffDate;
          }),
        }));
      },
    }),
    {
      name: "enterprise-backup-storage",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ 
        configs: state.configs, 
        jobs: state.jobs 
      }),
    }
  )
);

// ═══════════════════════════════════════════════════════════════════════
//  Hooks de Backup
// ═══════════════════════════════════════════════════════════════════════

export function useBackup() {
  const { 
    configs, 
    jobs, 
    isRunning,
    createBackup, 
    restoreBackup,
    createConfig,
    updateConfig,
    deleteConfig,
    toggleConfig,
    getBackupById,
    getStats,
    getRestorePoints,
    cleanupOldBackups,
  } = useEnterpriseBackup();

  return {
    configs,
    jobs,
    isRunning,
    createBackup,
    restoreBackup,
    createConfig,
    updateConfig,
    deleteConfig,
    toggleConfig,
    getBackupById,
    getStats,
    getRestorePoints,
    cleanupOldBackups,
    formatBytes,
  };
}

export function useBackupStats() {
  const { getStats } = useEnterpriseBackup();
  return getStats();
}

export function useBackupHistory(limit = 10) {
  const { jobs } = useEnterpriseBackup();
  return jobs.slice(0, limit);
}