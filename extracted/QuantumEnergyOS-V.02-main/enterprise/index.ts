// ═══════════════════════════════════════════════════════════════════════
//  enterprise/index.ts — Exportador de módulos Enterprise
//  QuantumEnergyOS — Punto de entrada para funcionalidades enterprise
// ═══════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════
//  Módulos Enterprise
// ═══════════════════════════════════════════════════════════════════════

// Autenticación y Gestión de Usuarios
export { 
  useEnterpriseAuth, 
  useAuth, 
  usePermissions,
  ROLE_PERMISSIONS,
  ROLE_LABELS,
  type EnterpriseUser,
  type UserRole,
  type LoginCredentials,
  type AuthState,
} from "./enterpriseAuth";

// Auditoría y Logs
export { 
  useEnterpriseAudit, 
  useAudit, 
  useAuditLog, 
  useAuditStats,
  type AuditEvent,
  type AuditEventType,
  type AuditSeverity,
  type AuditFilters,
  type AuditStats,
} from "./enterpriseAudit";

// Backups y Restauración
export { 
  useEnterpriseBackup, 
  useBackup, 
  useBackupStats, 
  useBackupHistory,
  type BackupConfig,
  type BackupJob,
  type BackupStatus,
  type BackupType,
  type BackupSchedule,
  type BackupStats,
  type RestorePoint,
} from "./enterpriseBackup";

// Reportes Empresariales
export { 
  useEnterpriseReports, 
  useReports, 
  useReportMetrics, 
  useReportHistory,
  type ReportConfig,
  type ReportData,
  type ReportType,
  type ReportFormat,
  type ReportFrequency,
  type ReportMetrics,
} from "./enterpriseReports";

// Panel de Administración (componente React)
export { default as EnterpriseAdmin } from "./EnterpriseAdmin";

// ═══════════════════════════════════════════════════════════════════════
//  Funciones utilitarias comunes
// ═══════════════════════════════════════════════════════════════════════

// Importar ROLE_PERMISSIONS desde enterpriseAuth
import { ROLE_PERMISSIONS } from "./enterpriseAuth";

/**
 * Verifica si el usuario tiene un permiso específico
 * @param userRole - Rol del usuario
 * @param permission - Permiso a verificar
 * @returns true si tiene el permiso
 */
export function checkPermission(userRole: string, permission: string): boolean {
  const permissions = ROLE_PERMISSIONS[userRole as keyof typeof ROLE_PERMISSIONS];
  return permissions?.includes(permission) || false;
}

/**
 * Obtiene el color asociado a un rol
 * @param role - Rol del usuario
 * @returns Color en formato hex
 */
export function getRoleColor(role: string): string {
  const colors: Record<string, string> = {
    admin: "#ef4444",
    manager: "#f59e0b",
    operator: "#3b82f6",
    viewer: "#6b7280",
    auditor: "#8b5cf6",
  };
  return colors[role] || "#6b7280";
}

/**
 * Formatea bytes a formato legible
 * @param bytes - Tamaño en bytes
 * @returns String formateado (ej: "1.5 GB")
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

/**
 * Formatea duración en milisegundos a formato legible
 * @param ms - Duración en milisegundos
 * @returns String formateado (ej: "1.5s" o "2min")
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  const minutes = seconds / 60;
  return `${minutes.toFixed(1)}min`;
}

/**
 * Genera un ID único
 * @returns ID único basado en timestamp
 */
export function generateUniqueId(): string {
  return `${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

// ═══════════════════════════════════════════════════════════════════════
//  Configuración Global Enterprise
// ═══════════════════════════════════════════════════════════════════════

export const ENTERPRISE_CONFIG = {
  // Configuración de auditoría
  audit: {
    maxEvents: 10000,
    retentionDays: 90,
    autoCleanup: true,
  },
  
  // Configuración de backups
  backup: {
    defaultRetentionDays: 30,
    maxBackupSize: 5 * 1024 * 1024 * 1024, // 5 GB
    compressDefault: true,
    encryptDefault: true,
  },
  
  // Configuración de reportes
  reports: {
    defaultFormat: "pdf" as const,
    maxFileSize: 10 * 1024 * 1024, // 10 MB
    autoSchedule: true,
  },
  
  // Configuración de sesión
  session: {
    timeoutMinutes: 60,
    maxConcurrentSessions: 5,
    require2FA: false,
  },
  
  // Configuración de la empresa
  company: {
    name: "QuantumEnergyOS",
    location: "Mexicali, Baja California, México",
    timezone: "America/Tijuana",
    currency: "MXN",
  },
};

// ═══════════════════════════════════════════════════════════════════════
//  Eventos del sistema enterprise (para logging)
// ═══════════════════════════════════════════════════════════════════════

export const ENTERPRISE_EVENTS = {
  // Tipos de eventos de autenticación
  AUTH: {
    LOGIN: "user_login",
    LOGOUT: "user_logout",
    LOGIN_FAILED: "user_login_failed",
    PASSWORD_CHANGE: "password_change",
    "2FA_ENABLE": "2fa_enable",
    "2FA_DISABLE": "2fa_disable",
    SESSION_EXPIRE: "session_expire",
  },
  
  // Tipos de eventos de usuario
  USER: {
    CREATE: "user_create",
    UPDATE: "user_update",
    DELETE: "user_delete",
    ROLE_CHANGE: "user_role_change",
    ACTIVATE: "user_activate",
    DEACTIVATE: "user_deactivate",
  },
  
  // Tipos de eventos de backup
  BACKUP: {
    START: "backup_start",
    COMPLETE: "backup_complete",
    FAIL: "backup_fail",
    RESTORE_START: "backup_restore_start",
    RESTORE_COMPLETE: "backup_restore_complete",
    RESTORE_FAIL: "backup_restore_fail",
  },
  
  // Tipos de eventos de reportes
  REPORT: {
    GENERATE: "report_generate",
    DOWNLOAD: "report_download",
    SCHEDULE: "report_schedule",
  },
  
  // Tipos de eventos del sistema
  SYSTEM: {
    CONFIG_CHANGE: "settings_change",
    PERMISSION_DENIED: "permission_denied",
    API_ACCESS: "api_access",
  },
} as const;

// ═══════════════════════════════════════════════════════════════════════
//  Constantes de permisos
// ═══════════════════════════════════════════════════════════════════════

export const PERMISSIONS = {
  // Permisos de usuario
  USERS_READ: "users:read",
  USERS_WRITE: "users:write",
  USERS_DELETE: "users:delete",
  
  // Permisos de roles
  ROLES_READ: "roles:read",
  ROLES_WRITE: "roles:write",
  
  // Permisos de auditoría
  AUDIT_READ: "audit:read",
  AUDIT_WRITE: "audit:write",
  AUDIT_DELETE: "audit:delete",
  
  // Permisos de backups
  BACKUP_READ: "backup:read",
  BACKUP_WRITE: "backup:write",
  BACKUP_RESTORE: "backup:restore",
  
  // Permisos de reportes
  REPORTS_READ: "reports:read",
  REPORTS_WRITE: "reports:write",
  REPORTS_EXPORT: "reports:export",
  
  // Permisos de configuración
  SETTINGS_READ: "settings:read",
  SETTINGS_WRITE: "settings:write",
  
  // Permisos de grid
  GRID_READ: "grid:read",
  GRID_WRITE: "grid:write",
  GRID_EXECUTE: "grid:execute",
  
  // Permisos de Quartz 4D
  QUARTZ_READ: "quartz:read",
  QUARTZ_WRITE: "quartz:write",
  QUARTZ_EXECUTE: "quartz:execute",
  
  // Permisos de Solar
  SOLAR_READ: "solar:read",
  SOLAR_WRITE: "solar:write",
} as const;

// ═══════════════════════════════════════════════════════════════════════
//  Versión del módulo enterprise
// ═══════════════════════════════════════════════════════════════════════

export const ENTERPRISE_VERSION = "1.0.0";
export const ENTERPRISE_BUILD = "20260403";

/**
 * Obtiene información de la versión enterprise
 * @returns Objeto con información de versión
 */
export function getEnterpriseInfo() {
  return {
    version: ENTERPRISE_VERSION,
    build: ENTERPRISE_BUILD,
    modules: [
      "enterpriseAuth",
      "enterpriseAudit", 
      "enterpriseBackup",
      "enterpriseReports",
      "EnterpriseAdmin",
    ],
    features: [
      "User Management",
      "Role-Based Access Control",
      "Audit Logging",
      "Automated Backups",
      "Report Generation",
      "Enterprise Dashboard",
    ],
  };
}

export default {
  version: ENTERPRISE_VERSION,
  config: ENTERPRISE_CONFIG,
  permissions: PERMISSIONS,
  events: ENTERPRISE_EVENTS,
  getEnterpriseInfo,
  checkPermission,
  getRoleColor,
  formatBytes,
  formatDuration,
  generateUniqueId,
};