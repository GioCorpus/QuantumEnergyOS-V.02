// ═══════════════════════════════════════════════════════════════════════
//  enterpriseAudit.ts — Sistema de Auditoría y Logs Enterprise
//  QuantumEnergyOS — Registro de acciones, eventos y compliance
// ═══════════════════════════════════════════════════════════════════════

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

// ═══════════════════════════════════════════════════════════════════════
//  Tipos de Eventos de Auditoría
// ═══════════════════════════════════════════════════════════════════════

export type AuditEventType = 
  | "user_login" 
  | "user_logout"
  | "user_create"
  | "user_update"
  | "user_delete"
  | "user_role_change"
  | "permission_denied"
  | "grid_execution"
  | "quartz_operation"
  | "solar_alert"
  | "backup_create"
  | "backup_restore"
  | "settings_change"
  | "system_error"
  | "data_export";

export type AuditSeverity = "info" | "warning" | "error" | "critical";

export interface AuditEvent {
  id: string;
  timestamp: string;
  eventType: AuditEventType;
  severity: AuditSeverity;
  userId: string | null;
  userEmail: string | null;
  description: string;
  details: Record<string, unknown>;
  ipAddress?: string;
  userAgent?: string;
  resource?: string;
  action?: string;
}

export interface AuditFilters {
  startDate?: string;
  endDate?: string;
  eventTypes?: AuditEventType[];
  severities?: AuditSeverity[];
  userId?: string;
  searchQuery?: string;
}

export interface AuditStats {
  totalEvents: number;
  byType: Record<AuditEventType, number>;
  bySeverity: Record<AuditSeverity, number>;
  byUser: Record<string, number>;
  recentCritical: number;
  dailyTrend: { date: string; count: number }[];
}

// ═══════════════════════════════════════════════════════════════════════
//  Utilidades
// ═══════════════════════════════════════════════════════════════════════

function generateAuditId(): string {
  return `audit_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

function getCurrentTimestamp(): string {
  return new Date().toISOString();
}

// ═══════════════════════════════════════════════════════════════════════
//  Store de Auditoría
// ═══════════════════════════════════════════════════════════════════════

interface AuditStore {
  events: AuditEvent[];
  addEvent: (event: Omit<AuditEvent, "id" | "timestamp">) => void;
  getEvents: (filters?: AuditFilters) => AuditEvent[];
  getStats: () => AuditStats;
  clearOldEvents: (daysToKeep: number) => void;
  exportEvents: (filters?: AuditFilters) => string;
  getEventById: (id: string) => AuditEvent | undefined;
}

// ═══════════════════════════════════════════════════════════════════════
//  Eventos Demo
// ═══════════════════════════════════════════════════════════════════════

const DEMO_AUDIT_EVENTS: AuditEvent[] = [
  {
    id: "audit_001",
    timestamp: "2026-04-03T10:30:00Z",
    eventType: "user_login",
    severity: "info",
    userId: "user_admin_001",
    userEmail: "admin@quantumenergy.os",
    description: "Inicio de sesión exitoso",
    details: { method: "password", rememberMe: false },
    ipAddress: "192.168.1.100",
    resource: "/api/v1/auth/login",
  },
  {
    id: "audit_002",
    timestamp: "2026-04-03T09:45:00Z",
    eventType: "grid_execution",
    severity: "info",
    userId: "user_operator_001",
    userEmail: "operator@quantumenergy.os",
    description: "Ejecución de QAOA para balanceo de red",
    details: { nodes: 6, shots: 1024, energySaved: "450 kW" },
    resource: "/api/v1/grid/balance",
    action: "execute",
  },
  {
    id: "audit_003",
    timestamp: "2026-04-03T09:15:00Z",
    eventType: "solar_alert",
    severity: "warning",
    userId: null,
    userEmail: "system",
    description: "Alerta de actividad solar media",
    details: { kpIndex: 4.2, riskLevel: "MEDIUM", location: "Mexicali" },
    resource: "/api/v1/solar/forecast",
  },
  {
    id: "audit_004",
    timestamp: "2026-04-03T08:22:00Z",
    eventType: "permission_denied",
    severity: "warning",
    userId: "user_viewer_001",
    userEmail: "viewer@quantumenergy.os",
    description: "Intento de acceso a configuración de red",
    details: { requestedResource: "/api/v1/grid/config", requiredRole: "operator" },
    ipAddress: "192.168.1.45",
    resource: "/api/v1/grid/config",
    action: "write",
  },
  {
    id: "audit_005",
    timestamp: "2026-04-02T16:30:00Z",
    eventType: "backup_create",
    severity: "info",
    userId: "user_admin_001",
    userEmail: "admin@quantumenergy.os",
    description: "Copia de seguridad completada",
    details: { backupSize: "2.3 GB", duration: "45s", tables: ["users", "grid_history", "audit"] },
    resource: "/api/v1/backup",
    action: "create",
  },
  {
    id: "audit_006",
    timestamp: "2026-04-02T14:00:00Z",
    eventType: "user_create",
    severity: "info",
    userId: "user_admin_001",
    userEmail: "admin@quantumenergy.os",
    description: "Nuevo usuario creado",
    details: { newUserEmail: "eliot@quantumenergy.os", role: "operator", department: "Operaciones" },
    resource: "/api/v1/users",
    action: "create",
  },
  {
    id: "audit_007",
    timestamp: "2026-04-02T10:15:00Z",
    eventType: "quartz_operation",
    severity: "info",
    userId: "user_manager_001",
    userEmail: "manager@quantumenergy.os",
    description: "Predicción de red ejecutada",
    details: { hoursAhead: 24, nodes: 6, efficiency: 0.82 },
    resource: "/api/v1/quartz/predict",
    action: "execute",
  },
  {
    id: "audit_008",
    timestamp: "2026-04-01T22:45:00Z",
    eventType: "system_error",
    severity: "error",
    userId: null,
    userEmail: "system",
    description: "Error de conexión con IBM Quantum",
    details: { error: "Connection timeout", backend: "ibm_brisbane" },
    resource: "/api/v1/ibm/run",
  },
  {
    id: "audit_009",
    timestamp: "2026-04-01T18:30:00Z",
    eventType: "data_export",
    severity: "info",
    userId: "user_auditor_001",
    userEmail: "auditor@quantumenergy.os",
    description: "Exportación de reportes mensuales",
    details: { format: "CSV", records: 15420, dateRange: "2026-03-01 to 2026-03-31" },
    resource: "/api/v1/reports/export",
    action: "export",
  },
  {
    id: "audit_010",
    timestamp: "2026-04-01T12:00:00Z",
    eventType: "settings_change",
    severity: "warning",
    userId: "user_admin_001",
    userEmail: "admin@quantumenergy.os",
    description: "Cambio en configuración del sistema",
    details: { setting: "session_timeout", oldValue: "30min", newValue: "60min" },
    resource: "/api/v1/settings",
    action: "update",
  },
];

// ═══════════════════════════════════════════════════════════════════════
//  Implementación del Store
// ═══════════════════════════════════════════════════════════════════════

export const useEnterpriseAudit = create<AuditStore>()(
  persist(
    (set, get) => ({
      events: DEMO_AUDIT_EVENTS,

      addEvent: (eventData: Omit<AuditEvent, "id" | "timestamp">) => {
        const newEvent: AuditEvent = {
          ...eventData,
          id: generateAuditId(),
          timestamp: getCurrentTimestamp(),
        };

        set(state => ({
          events: [newEvent, ...state.events].slice(0, 10000), // Mantener max 10k eventos
        }));

        // También guardar en localStorage para persistencia
        const storedEvents = JSON.parse(localStorage.getItem("audit_events") || "[]");
        storedEvents.unshift(newEvent);
        localStorage.setItem("audit_events", JSON.stringify(storedEvents.slice(0, 10000)));
      },

      getEvents: (filters?: AuditFilters): AuditEvent[] => {
        let filtered = [...get().events];

        if (!filters) return filtered;

        if (filters.startDate) {
          filtered = filtered.filter(e => e.timestamp >= filters.startDate!);
        }

        if (filters.endDate) {
          filtered = filtered.filter(e => e.timestamp <= filters.endDate!);
        }

        if (filters.eventTypes && filters.eventTypes.length > 0) {
          filtered = filtered.filter(e => filters.eventTypes!.includes(e.eventType));
        }

        if (filters.severities && filters.severities.length > 0) {
          filtered = filtered.filter(e => filters.severities!.includes(e.severity));
        }

        if (filters.userId) {
          filtered = filtered.filter(e => e.userId === filters.userId);
        }

        if (filters.searchQuery) {
          const query = filters.searchQuery.toLowerCase();
          filtered = filtered.filter(e => 
            e.description.toLowerCase().includes(query) ||
            e.userEmail?.toLowerCase().includes(query) ||
            e.eventType.toLowerCase().includes(query)
          );
        }

        return filtered;
      },

      getStats: (): AuditStats => {
        const events = get().events;

        const byType: Record<AuditEventType, number> = {} as Record<AuditEventType, number>;
        const bySeverity: Record<AuditSeverity, number> = {} as Record<AuditSeverity, number>;
        const byUser: Record<string, number> = {};

        let recentCritical = 0;
        const thirtyDaysAgo = new Date();
        thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

        const dailyCount: Record<string, number> = {};

        events.forEach(event => {
          // Conteo por tipo
          byType[event.eventType] = (byType[event.eventType] || 0) + 1;

          // Conteo por severidad
          bySeverity[event.severity] = (bySeverity[event.severity] || 0) + 1;

          // Conteo por usuario
          if (event.userEmail) {
            byUser[event.userEmail] = (byUser[event.userEmail] || 0) + 1;
          }

          // Críticos en los últimos 30 días
          if (event.severity === "critical" && new Date(event.timestamp) >= thirtyDaysAgo) {
            recentCritical++;
          }

          // Tendencia diaria
          const dateKey = event.timestamp.split("T")[0];
          dailyCount[dateKey] = (dailyCount[dateKey] || 0) + 1;
        });

        // Convertir a array para tendencias
        const dailyTrend = Object.entries(dailyCount)
          .map(([date, count]) => ({ date, count }))
          .sort((a, b) => a.date.localeCompare(b.date))
          .slice(-14); // Últimas 2 semanas

        return {
          totalEvents: events.length,
          byType,
          bySeverity,
          byUser,
          recentCritical,
          dailyTrend,
        };
      },

      clearOldEvents: (daysToKeep: number) => {
        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - daysToKeep);

        set(state => ({
          events: state.events.filter(e => new Date(e.timestamp) >= cutoffDate),
        }));
      },

      exportEvents: (filters?: AuditFilters): string => {
        const events = get().getEvents(filters);
        const csv = [
          "ID,Timestamp,Event Type,Severity,User Email,Description,IP Address,Resource",
          ...events.map(e => 
            `${e.id},${e.timestamp},${e.eventType},${e.severity},"${e.userEmail || ""}","${e.description}","${e.ipAddress || ""}","${e.resource || ""}"`
          )
        ].join("\n");

        return csv;
      },

      getEventById: (id: string): AuditEvent | undefined => {
        return get().events.find(e => e.id === id);
      },
    }),
    {
      name: "enterprise-audit-storage",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ events: state.events }),
    }
  )
);

// ═══════════════════════════════════════════════════════════════════════
//  Hooks de Auditoría
// ═══════════════════════════════════════════════════════════════════════

export function useAudit() {
  const { events, addEvent, getEvents, getStats, clearOldEvents, exportEvents } = useEnterpriseAudit();
  
  return {
    events,
    addEvent,
    getEvents,
    getStats,
    clearOldEvents,
    exportEvents,
  };
}

export function useAuditLog(filters?: AuditFilters) {
  const { getEvents } = useEnterpriseAudit();
  return getEvents(filters);
}

export function useAuditStats() {
  const { getStats } = useEnterpriseAudit();
  return getStats();
}