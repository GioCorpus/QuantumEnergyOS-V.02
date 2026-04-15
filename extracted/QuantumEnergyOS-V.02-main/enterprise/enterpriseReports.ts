// ═══════════════════════════════════════════════════════════════════════
//  enterpriseReports.ts — Sistema de Reportes Empresariales
//  QuantumEnergyOS — Generación de reportes, métricas y analytics
// ═══════════════════════════════════════════════════════════════════════

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

// ═══════════════════════════════════════════════════════════════════════
//  Tipos de Reportes
// ═══════════════════════════════════════════════════════════════════════

export type ReportType = 
  | "grid_performance"
  | "energy_savings"
  | "user_activity"
  | "system_health"
  | "audit_summary"
  | "quartz_operations"
  | "solar_analysis"
  | "compliance";

export type ReportFormat = "json" | "csv" | "pdf" | "html";
export type ReportFrequency = "on_demand" | "daily" | "weekly" | "monthly";

export interface ReportConfig {
  id: string;
  name: string;
  type: ReportType;
  description: string;
  frequency: ReportFrequency;
  enabled: boolean;
  recipients: string[];
  filters: Record<string, unknown>;
  schedule?: string; // Cron expression
}

export interface ReportData {
  id: string;
  configId: string;
  configName: string;
  type: ReportType;
  generatedAt: string;
  period: {
    start: string;
    end: string;
  };
  format: ReportFormat;
  fileSize: number;
  filePath: string | null;
  status: "generating" | "completed" | "failed";
  errorMessage?: string;
  data: Record<string, unknown>;
  summary: {
    totalRecords: number;
    charts: number;
    pages: number;
  };
}

export interface ReportMetrics {
  totalReports: number;
  reportsThisMonth: number;
  avgGenerationTime: number;
  storageUsed: number;
  formatsDistribution: Record<ReportFormat, number>;
  typesDistribution: Record<ReportType, number>;
}

// ═══════════════════════════════════════════════════════════════════════
//  Utilidades
// ═══════════════════════════════════════════════════════════════════════

function generateReportId(): string {
  return `report_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  const minutes = seconds / 60;
  return `${minutes.toFixed(1)}min`;
}

// ═══════════════════════════════════════════════════════════════════════
//  Reportes Predefinidos
// ═══════════════════════════════════════════════════════════════════════

const DEFAULT_REPORT_CONFIGS: ReportConfig[] = [
  {
    id: "report_grid_perf",
    name: "Rendimiento de Red",
    type: "grid_performance",
    description: "Análisis de rendimiento de la red eléctrica de Baja California",
    frequency: "weekly",
    enabled: true,
    recipients: ["manager@quantumenergy.os", "operator@quantumenergy.os"],
    filters: {
      nodes: ["all"],
      includeCharts: true,
      comparePrevious: true,
    },
    schedule: "0 8 * * 1", // Lunes 8am
  },
  {
    id: "report_energy_savings",
    name: "Ahorro Energético",
    type: "energy_savings",
    description: "Reporte de energía saved mediante optimización cuántica",
    frequency: "monthly",
    enabled: true,
    recipients: ["admin@quantumenergy.os", "manager@quantumenergy.os"],
    filters: {
      includeProjections: true,
      currency: "MXN",
      compareYear: true,
    },
    schedule: "0 9 1 * *", // Primer día del mes 9am
  },
  {
    id: "report_user_activity",
    name: "Actividad de Usuarios",
    type: "user_activity",
    description: "Resumen de actividad de usuarios en el sistema",
    frequency: "weekly",
    enabled: true,
    recipients: ["admin@quantumenergy.os"],
    filters: {
      includeLoginStats: true,
      includeActionsSummary: true,
      groupByDepartment: true,
    },
    schedule: "0 7 * * 1", // Lunes 7am
  },
  {
    id: "report_system_health",
    name: "Salud del Sistema",
    type: "system_health",
    description: "Métricas de salud y disponibilidad del sistema",
    frequency: "daily",
    enabled: true,
    recipients: ["admin@quantumenergy.os", "operator@quantumenergy.os"],
    filters: {
      includeUptime: true,
      includeErrors: true,
      includePerformance: true,
    },
    schedule: "0 6 * * *", // Daily 6am
  },
  {
    id: "report_audit_summary",
    name: "Resumen de Auditoría",
    type: "audit_summary",
    description: "Resumen de eventos de auditoría y compliance",
    frequency: "monthly",
    enabled: true,
    recipients: ["auditor@quantumenergy.os", "admin@quantumenergy.os"],
    filters: {
      includeSecurityEvents: true,
      includeUserActions: true,
      complianceFormat: true,
    },
    schedule: "0 8 1 * *", // Primer día del mes 8am
  },
  {
    id: "report_quartz_ops",
    name: "Operaciones Cuarzo 4D",
    type: "quartz_operations",
    description: "Análisis de operaciones del sistema cuántico Cuarzo 4D",
    frequency: "weekly",
    enabled: true,
    recipients: ["operator@quantumenergy.os", "manager@quantumenergy.os"],
    filters: {
      includeCoherence: true,
      includePredictions: true,
      includeBraidOps: true,
    },
    schedule: "0 10 * * 5", // Viernes 10am
  },
];

// ═══════════════════════════════════════════════════════════════════════
//  Datos Demo para Reportes
// ═══════════════════════════════════════════════════════════════════════

const DEMO_REPORT_DATA: Record<ReportType, Record<string, unknown>> = {
  grid_performance: {
    totalNodes: 8,
    avgLoadFactor: 0.72,
    peakLoad: 95000,
    minLoad: 18500,
    energySaved: 456.7,
    downtime: 0.02,
    alertsTriggered: 12,
    optimizationRuns: 156,
    nodes: [
      { name: "Mexicali Centro", load: 0.85, status: "normal" },
      { name: "Mexicali Industrial", load: 0.72, status: "normal" },
      { name: "Tijuana Norte", load: 0.88, status: "warning" },
      { name: "Tijuana Este", load: 0.65, status: "normal" },
      { name: "Ensenada", load: 0.55, status: "normal" },
      { name: "Tecate", load: 0.45, status: "normal" },
      { name: "Rosarito", load: 0.60, status: "normal" },
      { name: "San Felipe", load: 0.38, status: "normal" },
    ],
  },
  energy_savings: {
    totalSavedKwh: 45678.9,
    savingsMXN: 82345.67,
    reductionPct: 12.3,
    peakShaving: 234.5,
    costAvoidance: 45678.9,
    carbonOffset: 23.4,
    efficiencyGain: 8.7,
    projections: {
      nextMonth: 48000,
      nextQuarter: 145000,
      nextYear: 580000,
    },
  },
  user_activity: {
    totalUsers: 156,
    activeUsers: 89,
    totalSessions: 2345,
    avgSessionDuration: 45,
    actionsPerformed: 15678,
    byDepartment: {
      IT: 45,
      Operaciones: 34,
      Ingeniería: 23,
      Cumplimiento: 12,
    },
    topActions: [
      { action: "grid_execution", count: 456 },
      { action: "quartz_predict", count: 234 },
      { action: "solar_check", count: 189 },
      { action: "report_view", count: 156 },
    ],
  },
  system_health: {
    uptime: 99.98,
    avgResponseTime: 125,
    errorRate: 0.02,
    apiCalls: 456789,
    cpuUsage: 34,
    memoryUsage: 56,
    diskUsage: 45,
    activeConnections: 23,
    services: [
      { name: "API Server", status: "healthy", uptime: 99.99 },
      { name: "Quantum Engine", status: "healthy", uptime: 99.95 },
      { name: "Grid Optimizer", status: "healthy", uptime: 100 },
      { name: "Solar Monitor", status: "healthy", uptime: 99.89 },
    ],
  },
  audit_summary: {
    totalEvents: 45230,
    securityEvents: 234,
    criticalAlerts: 3,
    userActions: 12345,
    systemEvents: 32885,
    bySeverity: {
      info: 42000,
      warning: 2800,
      error: 420,
      critical: 10,
    },
    byType: {
      user_login: 8900,
      grid_execution: 5600,
      permission_denied: 234,
      system_error: 45,
    },
    complianceScore: 98.5,
  },
  quartz_operations: {
    totalOperations: 1234,
    successfulOps: 1198,
    coherenceAvg: 1850,
    braidOperations: 45678,
    qubitsUtilized: 16,
    predictionsAccuracy: 87.3,
    layers: [
      { name: "Física", active: true, operations: 456 },
      { name: "Topológica", active: true, operations: 389 },
      { name: "Holográfica 4D", active: true, operations: 234 },
      { name: "Energética", active: true, operations: 119 },
    ],
  },
  solar_analysis: {
    alertsTriggered: 12,
    avgKpIndex: 2.3,
    maxKpIndex: 5.8,
    riskDistribution: {
      LOW: 18,
      MEDIUM: 4,
      HIGH: 1,
      EXTREME: 0,
    },
    gridImpact: 4.5,
    recommendations: [
      "Monitoreo activo durante风暴 solar",
      "Ejecutar QAOA preventivo",
      "Verificar topología de red",
    ],
  },
  compliance: {
    score: 98.5,
    frameworks: ["ISO27001", "SOC2", "GDPR"],
    findings: {
      critical: 0,
      high: 2,
      medium: 5,
      low: 12,
    },
    lastAudit: "2026-03-15",
    nextAudit: "2026-06-15",
    policiesCompliant: 45,
    policiesTotal: 47,
  },
};

// ═══════════════════════════════════════════════════════════════════════
//  Reportes Demo Generados
// ═══════════════════════════════════════════════════════════════════════

const DEMO_REPORTS: ReportData[] = [
  {
    id: "report_gen_001",
    configId: "report_grid_perf",
    configName: "Rendimiento de Red",
    type: "grid_performance",
    generatedAt: "2026-04-03T08:00:00Z",
    period: { start: "2026-03-27T00:00:00Z", end: "2026-04-03T00:00:00Z" },
    format: "pdf",
    fileSize: 2_345_678,
    filePath: "/reports/2026-04-03/grid_performance_weekly.pdf",
    status: "completed",
    data: DEMO_REPORT_DATA.grid_performance,
    summary: { totalRecords: 8, charts: 4, pages: 12 },
  },
  {
    id: "report_gen_002",
    configId: "report_energy_savings",
    configName: "Ahorro Energético",
    type: "energy_savings",
    generatedAt: "2026-04-01T09:00:00Z",
    period: { start: "2026-03-01T00:00:00Z", end: "2026-03-31T23:59:59Z" },
    format: "pdf",
    fileSize: 1_890_123,
    filePath: "/reports/2026-04-01/energy_savings_monthly.pdf",
    status: "completed",
    data: DEMO_REPORT_DATA.energy_savings,
    summary: { totalRecords: 156, charts: 6, pages: 18 },
  },
  {
    id: "report_gen_003",
    configId: "report_system_health",
    configName: "Salud del Sistema",
    type: "system_health",
    generatedAt: "2026-04-03T06:00:00Z",
    period: { start: "2026-04-02T00:00:00Z", end: "2026-04-03T00:00:00Z" },
    format: "json",
    fileSize: 125_678,
    filePath: "/reports/2026-04-03/system_health_daily.json",
    status: "completed",
    data: DEMO_REPORT_DATA.system_health,
    summary: { totalRecords: 24, charts: 2, pages: 3 },
  },
  {
    id: "report_gen_004",
    configId: "report_audit_summary",
    configName: "Resumen de Auditoría",
    type: "audit_summary",
    generatedAt: "2026-04-01T08:00:00Z",
    period: { start: "2026-03-01T00:00:00Z", end: "2026-03-31T23:59:59Z" },
    format: "pdf",
    fileSize: 3_456_789,
    filePath: "/reports/2026-04-01/audit_summary_monthly.pdf",
    status: "completed",
    data: DEMO_REPORT_DATA.audit_summary,
    summary: { totalRecords: 45230, charts: 5, pages: 24 },
  },
  {
    id: "report_gen_005",
    configId: "report_user_activity",
    configName: "Actividad de Usuarios",
    type: "user_activity",
    generatedAt: "2026-04-03T07:00:00Z",
    period: { start: "2026-03-27T00:00:00Z", end: "2026-04-03T00:00:00Z" },
    format: "csv",
    fileSize: 89_456,
    filePath: "/reports/2026-04-03/user_activity_weekly.csv",
    status: "completed",
    data: DEMO_REPORT_DATA.user_activity,
    summary: { totalRecords: 156, charts: 3, pages: 5 },
  },
];

// ═══════════════════════════════════════════════════════════════════════
//  Store de Reportes
// ═══════════════════════════════════════════════════════════════════════

interface ReportStore {
  configs: ReportConfig[];
  generatedReports: ReportData[];
  
  // Acciones
  generateReport: (configId: string, format?: ReportFormat) => Promise<ReportData>;
  getReportById: (reportId: string) => ReportData | undefined;
  getReportsByType: (type: ReportType) => ReportData[];
  getReportsByConfig: (configId: string) => ReportData[];
  createConfig: (config: Omit<ReportConfig, "id">) => ReportConfig;
  updateConfig: (configId: string, updates: Partial<ReportConfig>) => boolean;
  deleteConfig: (configId: string) => boolean;
  toggleConfig: (configId: string) => boolean;
  getMetrics: () => ReportMetrics;
  downloadReport: (reportId: string, format: ReportFormat) => Promise<string>;
}

// ═══════════════════════════════════════════════════════════════════════

export const useEnterpriseReports = create<ReportStore>()(
  persist(
    (set, get) => ({
      configs: DEFAULT_REPORT_CONFIGS,
      generatedReports: DEMO_REPORTS,

      generateReport: async (configId: string, format: ReportFormat = "pdf"): Promise<ReportData> => {
        const config = get().configs.find(c => c.id === configId);
        if (!config) {
          throw new Error("Configuración de reporte no encontrada");
        }

        const reportId = generateReportId();
        const now = new Date();
        const startDate = new Date(now);
        startDate.setDate(startDate.getDate() - (config.frequency === "daily" ? 1 : config.frequency === "weekly" ? 7 : 30));

        const newReport: ReportData = {
          id: reportId,
          configId: config.id,
          configName: config.name,
          type: config.type,
          generatedAt: now.toISOString(),
          period: {
            start: startDate.toISOString(),
            end: now.toISOString(),
          },
          format,
          fileSize: 0,
          filePath: null,
          status: "generating",
          data: {},
          summary: { totalRecords: 0, charts: 0, pages: 0 },
        };

        set(state => ({
          generatedReports: [newReport, ...state.generatedReports],
        }));

        try {
          // Simular generación de reporte
          await new Promise(resolve => setTimeout(resolve, 1500));

          const templateData = DEMO_REPORT_DATA[config.type];
          
          const completedReport: ReportData = {
            ...newReport,
            status: "completed",
            fileSize: Math.floor(Math.random() * 3000000) + 500000,
            filePath: `/reports/${now.toISOString().split("T")[0]}/${config.type}_${format}`,
            data: templateData || {},
            summary: {
              totalRecords: Math.floor(Math.random() * 500) + 50,
              charts: Math.floor(Math.random() * 8) + 2,
              pages: Math.floor(Math.random() * 20) + 3,
            },
          };

          set(state => ({
            generatedReports: state.generatedReports.map(r => 
              r.id === reportId ? completedReport : r
            ),
          }));

          return completedReport;
        } catch (error) {
          const failedReport: ReportData = {
            ...newReport,
            status: "failed",
            errorMessage: String(error),
          };

          set(state => ({
            generatedReports: state.generatedReports.map(r => 
              r.id === reportId ? failedReport : r
            ),
          }));

          return failedReport;
        }
      },

      getReportById: (reportId: string): ReportData | undefined => {
        return get().generatedReports.find(r => r.id === reportId);
      },

      getReportsByType: (type: ReportType): ReportData[] => {
        return get().generatedReports.filter(r => r.type === type);
      },

      getReportsByConfig: (configId: string): ReportData[] => {
        return get().generatedReports.filter(r => r.configId === configId);
      },

      createConfig: (configData: Omit<ReportConfig, "id">): ReportConfig => {
        const newConfig: ReportConfig = {
          ...configData,
          id: `report_${generateReportId()}`,
        };

        set(state => ({
          configs: [...state.configs, newConfig],
        }));

        return newConfig;
      },

      updateConfig: (configId: string, updates: Partial<ReportConfig>): boolean => {
        set(state => ({
          configs: state.configs.map(c => 
            c.id === configId ? { ...c, ...updates } : c
          ),
        }));

        return true;
      },

      deleteConfig: (configId: string): boolean => {
        set(state => ({
          configs: state.configs.filter(c => c.id !== configId),
        }));

        return true;
      },

      toggleConfig: (configId: string): boolean => {
        set(state => ({
          configs: state.configs.map(c => 
            c.id === configId ? { ...c, enabled: !c.enabled } : c
          ),
        }));

        return true;
      },

      getMetrics: (): ReportMetrics => {
        const reports = get().generatedReports.filter(r => r.status === "completed");
        const now = new Date();
        const thisMonth = reports.filter(r => {
          const reportDate = new Date(r.generatedAt);
          return reportDate.getMonth() === now.getMonth() && reportDate.getFullYear() === now.getFullYear();
        });

        const formatsDistribution: Record<ReportFormat, number> = {
          json: 0,
          csv: 0,
          pdf: 0,
          html: 0,
        };

        const typesDistribution: Record<ReportType, number> = {
          grid_performance: 0,
          energy_savings: 0,
          user_activity: 0,
          system_health: 0,
          audit_summary: 0,
          quartz_operations: 0,
          solar_analysis: 0,
          compliance: 0,
        };

        let totalSize = 0;
        let totalGenTime = 0;

        reports.forEach(r => {
          formatsDistribution[r.format]++;
          typesDistribution[r.type]++;
          totalSize += r.fileSize;
          totalGenTime += 1500; // Simulado
        });

        return {
          totalReports: reports.length,
          reportsThisMonth: thisMonth.length,
          avgGenerationTime: reports.length > 0 ? totalGenTime / reports.length : 0,
          storageUsed: totalSize,
          formatsDistribution,
          typesDistribution,
        };
      },

      downloadReport: async (reportId: string, format: ReportFormat): Promise<string> => {
        const report = get().getReportById(reportId);
        if (!report) {
          throw new Error("Reporte no encontrado");
        }

        if (report.status !== "completed") {
          throw new Error("El reporte aún no está disponible");
        }

        // Simular descarga
        await new Promise(resolve => setTimeout(resolve, 500));

        switch (format) {
          case "json":
            return JSON.stringify(report.data, null, 2);
          case "csv":
            return "Timestamp,Value\n" + Object.entries(report.data).map(([k, v]) => `${k},${JSON.stringify(v)}`).join("\n");
          case "html":
            return `<html><body><h1>${report.configName}</h1><pre>${JSON.stringify(report.data, null, 2)}</pre></body></html>`;
          case "pdf":
            return "PDF content would be generated here";
          default:
            return "";
        }
      },
    }),
    {
      name: "enterprise-reports-storage",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ 
        configs: state.configs, 
        generatedReports: state.generatedReports 
      }),
    }
  )
);

// ═══════════════════════════════════════════════════════════════════════
//  Hooks de Reportes
// ═══════════════════════════════════════════════════════════════════════

export function useReports() {
  const {
    configs,
    generatedReports,
    generateReport,
    getReportById,
    getReportsByType,
    getReportsByConfig,
    createConfig,
    updateConfig,
    deleteConfig,
    toggleConfig,
    getMetrics,
    downloadReport,
  } = useEnterpriseReports();

  return {
    configs,
    generatedReports,
    generateReport,
    getReportById,
    getReportsByType,
    getReportsByConfig,
    createConfig,
    updateConfig,
    deleteConfig,
    toggleConfig,
    getMetrics,
    downloadReport,
    formatBytes,
    formatDuration,
  };
}

export function useReportMetrics() {
  const { getMetrics } = useEnterpriseReports();
  return getMetrics();
}

export function useReportHistory(type?: ReportType, limit = 10) {
  const { generatedReports } = useEnterpriseReports();
  const filtered = type 
    ? generatedReports.filter(r => r.type === type)
    : generatedReports;
  return filtered.slice(0, limit);
}