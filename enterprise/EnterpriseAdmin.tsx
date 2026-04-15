// ═══════════════════════════════════════════════════════════════════════
//  EnterpriseAdmin.tsx — Panel de Administración Enterprise
//  QuantumEnergyOS — Dashboard centralizado de gestión empresarial
// ═══════════════════════════════════════════════════════════════════════

import React, { useState, useEffect } from "react";
import { 
  Users, Shield, FileText, Database, Activity, 
  Settings, BarChart3, Clock, CheckCircle, XCircle,
  AlertTriangle, Download, RefreshCw, Plus, Trash2,
  Edit, Eye, Play, Pause, Save, Search, Filter
} from "lucide-react";

// Importar módulos enterprise (simulated imports - in production these would be actual imports)
// Using any types for demo since zustand might not be in scope
const useEnterpriseAuth: any = null;
const useEnterpriseAudit: any = null;
const useEnterpriseBackup: any = null;
const useEnterpriseReports: any = null;

// ═══════════════════════════════════════════════════════════════════════
//  Tipos
// ═══════════════════════════════════════════════════════════════════════

type AdminTab = "users" | "roles" | "audit" | "backups" | "reports" | "settings";

// ═══════════════════════════════════════════════════════════════════════
//  Componentes del Panel
// ═══════════════════════════════════════════════════════════════════════

// ── Header del Panel Enterprise ─────────────────────────────────────────

function EnterpriseHeader({ user, onLogout }: { user: any; onLogout: () => void }) {
  return (
    <div className="enterprise-header">
      <div className="enterprise-logo">
        <Shield size={28} className="logo-icon" />
        <div className="logo-text">
          <h1>QuantumEnergyOS</h1>
          <span>Enterprise Admin</span>
        </div>
      </div>
      
      <div className="enterprise-user">
        <div className="user-info">
          <span className="user-name">{user?.name || "Admin"}</span>
          <span className="user-role">{user?.role || "admin"}</span>
        </div>
        <button className="logout-btn" onClick={onLogout}>
          Cerrar sesión
        </button>
      </div>
    </div>
  );
}

// ── Navegación del Panel ─────────────────────────────────────────────────

function EnterpriseNav({ activeTab, onTabChange }: { activeTab: AdminTab; onTabChange: (tab: AdminTab) => void }) {
  const tabs: { id: AdminTab; label: string; icon: React.ReactNode }[] = [
    { id: "users", label: "Usuarios", icon: <Users size={18} /> },
    { id: "roles", label: "Roles", icon: <Shield size={18} /> },
    { id: "audit", label: "Auditoría", icon: <Activity size={18} /> },
    { id: "backups", label: "Backups", icon: <Database size={18} /> },
    { id: "reports", label: "Reportes", icon: <BarChart3 size={18} /> },
    { id: "settings", label: "Configuración", icon: <Settings size={18} /> },
  ];

  return (
    <nav className="enterprise-nav">
      {tabs.map(tab => (
        <button
          key={tab.id}
          className={`nav-tab ${activeTab === tab.id ? "active" : ""}`}
          onClick={() => onTabChange(tab.id)}
        >
          {tab.icon}
          <span>{tab.label}</span>
        </button>
      ))}
    </nav>
  );
}

// ── Gestión de Usuarios ─────────────────────────────────────────────────

function UsersPanel() {
  const [searchQuery, setSearchQuery] = useState("");
  
  // Demo users data
  const demoUsers = [
    { id: "user_001", name: "Gio Corpus", email: "admin@quantumenergy.os", role: "admin", department: "IT", isActive: true, lastLogin: "2026-04-03T10:30:00Z" },
    { id: "user_002", name: "Eliot Hernandez", email: "manager@quantumenergy.os", role: "manager", department: "Operaciones", isActive: true, lastLogin: "2026-04-02T14:22:00Z" },
    { id: "user_003", name: "Carlos Mendez", email: "operator@quantumenergy.os", role: "operator", department: "Ingeniería", isActive: true, lastLogin: "2026-04-03T08:15:00Z" },
    { id: "user_004", name: "Ana Laura", email: "auditor@quantumenergy.os", role: "auditor", department: "Cumplimiento", isActive: true, lastLogin: "2026-04-01T16:45:00Z" },
    { id: "user_005", name: "Roberto Sánchez", email: "viewer@quantumenergy.os", role: "viewer", department: "Ventas", isActive: false, lastLogin: "2026-03-28T09:00:00Z" },
  ];

  const filteredUsers = demoUsers.filter(u => 
    u.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    u.email.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const roleColors: Record<string, string> = {
    admin: "#ef4444",
    manager: "#f59e0b",
    operator: "#3b82f6",
    viewer: "#6b7280",
    auditor: "#8b5cf6",
  };

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <h2><Users size={20} /> Gestión de Usuarios</h2>
        <button className="primary-btn">
          <Plus size={16} /> Nuevo Usuario
        </button>
      </div>

      <div className="search-bar">
        <Search size={18} />
        <input 
          type="text" 
          placeholder="Buscar usuarios..." 
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      <div className="users-table">
        <table>
          <thead>
            <tr>
              <th>Usuario</th>
              <th>Email</th>
              <th>Rol</th>
              <th>Departamento</th>
              <th>Estado</th>
              <th>Último Acceso</th>
              <th>Acciones</th>
            </tr>
          </thead>
          <tbody>
            {filteredUsers.map(user => (
              <tr key={user.id}>
                <td className="user-name-cell">
                  <div className="avatar">{user.name.charAt(0)}</div>
                  {user.name}
                </td>
                <td>{user.email}</td>
                <td>
                  <span className="role-badge" style={{ backgroundColor: roleColors[user.role] }}>
                    {user.role}
                  </span>
                </td>
                <td>{user.department}</td>
                <td>
                  {user.isActive ? (
                    <span className="status-active"><CheckCircle size={14} /> Activo</span>
                  ) : (
                    <span className="status-inactive"><XCircle size={14} /> Inactivo</span>
                  )}
                </td>
                <td className="date-cell">
                  {new Date(user.lastLogin).toLocaleDateString("es-MX")}
                </td>
                <td className="actions-cell">
                  <button className="icon-btn-small" title="Ver"><Eye size={14} /></button>
                  <button className="icon-btn-small" title="Editar"><Edit size={14} /></button>
                  <button className="icon-btn-small danger" title="Eliminar"><Trash2 size={14} /></button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// ── Gestión de Roles y Permisos ─────────────────────────────────────────

function RolesPanel() {
  const roles = [
    { 
      id: "admin", 
      name: "Administrador", 
      description: "Acceso completo al sistema",
      permissions: ["users:*", "roles:*", "audit:*", "backup:*", "reports:*", "settings:*", "grid:*", "quartz:*", "solar:*"],
      usersCount: 1,
      color: "#ef4444",
    },
    { 
      id: "manager", 
      name: "Gerente", 
      description: "Gestión de operaciones y reportes",
      permissions: ["grid:*", "quartz:*", "solar:read", "reports:*", "audit:read"],
      usersCount: 2,
      color: "#f59e0b",
    },
    { 
      id: "operator", 
      name: "Operador", 
      description: "Operaciones de red y sistema cuántico",
      permissions: ["grid:read", "grid:write", "grid:execute", "quartz:read", "quartz:execute", "solar:read"],
      usersCount: 3,
      color: "#3b82f6",
    },
    { 
      id: "viewer", 
      name: "Visor", 
      description: "Solo lectura del sistema",
      permissions: ["grid:read", "quartz:read", "solar:read"],
      usersCount: 15,
      color: "#6b7280",
    },
    { 
      id: "auditor", 
      name: "Auditor", 
      description: "Acceso a logs y compliance",
      permissions: ["audit:read", "reports:read", "reports:export"],
      usersCount: 2,
      color: "#8b5cf6",
    },
  ];

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <h2><Shield size={20} /> Roles y Permisos</h2>
        <button className="primary-btn">
          <Plus size={16} /> Nuevo Rol
        </button>
      </div>

      <div className="roles-grid">
        {roles.map(role => (
          <div key={role.id} className="role-card">
            <div className="role-header" style={{ borderLeftColor: role.color }}>
              <h3>{role.name}</h3>
              <span className="users-count">{role.usersCount} usuarios</span>
            </div>
            <p className="role-description">{role.description}</p>
            <div className="role-permissions">
              <h4>Permisos ({role.permissions.length})</h4>
              <div className="permissions-list">
                {role.permissions.slice(0, 6).map((perm, i) => (
                  <span key={i} className="permission-tag">{perm}</span>
                ))}
                {role.permissions.length > 6 && (
                  <span className="permission-more">+{role.permissions.length - 6} más</span>
                )}
              </div>
            </div>
            <div className="role-actions">
              <button className="text-btn">Editar</button>
              <button className="text-btn">Ver usuarios</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ── Panel de Auditoría ─────────────────────────────────────────────────

function AuditPanel() {
  const [filterType, setFilterType] = useState("all");
  const [filterSeverity, setFilterSeverity] = useState("all");

  const auditEvents = [
    { id: "audit_001", timestamp: "2026-04-03T10:30:00Z", eventType: "user_login", severity: "info", user: "admin@quantumenergy.os", description: "Inicio de sesión exitoso" },
    { id: "audit_002", timestamp: "2026-04-03T09:45:00Z", eventType: "grid_execution", severity: "info", user: "operator@quantumenergy.os", description: "Ejecución de QAOA para balanceo de red" },
    { id: "audit_003", timestamp: "2026-04-03T09:15:00Z", eventType: "solar_alert", severity: "warning", user: "system", description: "Alerta de actividad solar media" },
    { id: "audit_004", timestamp: "2026-04-03T08:22:00Z", eventType: "permission_denied", severity: "warning", user: "viewer@quantumenergy.os", description: "Intento de acceso a configuración de red" },
    { id: "audit_005", timestamp: "2026-04-02T16:30:00Z", eventType: "backup_create", severity: "info", user: "admin@quantumenergy.os", description: "Copia de seguridad completada" },
    { id: "audit_006", timestamp: "2026-04-02T14:00:00Z", eventType: "user_create", severity: "info", user: "admin@quantumenergy.os", description: "Nuevo usuario creado" },
    { id: "audit_007", timestamp: "2026-04-02T10:15:00Z", eventType: "quartz_operation", severity: "info", user: "manager@quantumenergy.os", description: "Predicción de red ejecutada" },
    { id: "audit_008", timestamp: "2026-04-01T22:45:00Z", eventType: "system_error", severity: "error", user: "system", description: "Error de conexión con IBM Quantum" },
  ];

  const severityColors: Record<string, string> = {
    info: "#3b82f6",
    warning: "#f59e0b",
    error: "#ef4444",
    critical: "#dc2626",
  };

  const filteredEvents = auditEvents.filter(e => {
    if (filterType !== "all" && e.eventType !== filterType) return false;
    if (filterSeverity !== "all" && e.severity !== filterSeverity) return false;
    return true;
  });

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <h2><Activity size={20} /> Registro de Auditoría</h2>
        <button className="secondary-btn">
          <Download size={16} /> Exportar CSV
        </button>
      </div>

      <div className="audit-filters">
        <select value={filterType} onChange={(e) => setFilterType(e.target.value)}>
          <option value="all">Todos los eventos</option>
          <option value="user_login">Inicio de sesión</option>
          <option value="grid_execution">Ejecución de red</option>
          <option value="backup_create">Backup</option>
          <option value="user_create">Crear usuario</option>
          <option value="system_error">Error del sistema</option>
        </select>
        <select value={filterSeverity} onChange={(e) => setFilterSeverity(e.target.value)}>
          <option value="all">Todas las severidades</option>
          <option value="info">Información</option>
          <option value="warning">Advertencia</option>
          <option value="error">Error</option>
          <option value="critical">Crítico</option>
        </select>
      </div>

      <div className="audit-stats">
        <div className="stat-card">
          <span className="stat-value">45,230</span>
          <span className="stat-label">Total Eventos</span>
        </div>
        <div className="stat-card">
          <span className="stat-value" style={{ color: "#3b82f6" }}>42,000</span>
          <span className="stat-label">Info</span>
        </div>
        <div className="stat-card">
          <span className="stat-value" style={{ color: "#f59e0b" }}>2,800</span>
          <span className="stat-label">Warnings</span>
        </div>
        <div className="stat-card">
          <span className="stat-value" style={{ color: "#ef4444" }}>420</span>
          <span className="stat-label">Errores</span>
        </div>
        <div className="stat-card">
          <span className="stat-value" style={{ color: "#dc2626" }}>10</span>
          <span className="stat-label">Críticos</span>
        </div>
      </div>

      <div className="audit-table">
        <table>
          <thead>
            <tr>
              <th>Fecha/Hora</th>
              <th>Tipo de Evento</th>
              <th>Severidad</th>
              <th>Usuario</th>
              <th>Descripción</th>
            </tr>
          </thead>
          <tbody>
            {filteredEvents.map(event => (
              <tr key={event.id}>
                <td className="date-cell">
                  {new Date(event.timestamp).toLocaleString("es-MX")}
                </td>
                <td className="event-type">{event.eventType.replace(/_/g, " ")}</td>
                <td>
                  <span 
                    className="severity-badge" 
                    style={{ backgroundColor: severityColors[event.severity] }}
                  >
                    {event.severity}
                  </span>
                </td>
                <td>{event.user}</td>
                <td className="description-cell">{event.description}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// ── Panel de Backups ────────────────────────────────────────────────────

function BackupsPanel() {
  const [isCreating, setIsCreating] = useState(false);

  const backupJobs = [
    { id: "backup_001", configName: "Backup Completo Diario", type: "full", status: "completed", startTime: "2026-04-03T02:00:00Z", sizeBytes: 2_456_789_012, tables: 7 },
    { id: "backup_002", configName: "Backup Incremental Horario", type: "incremental", status: "completed", startTime: "2026-04-03T10:00:00Z", sizeBytes: 125_678_901, tables: 2 },
    { id: "backup_003", configName: "Backup Completo Diario", type: "full", status: "completed", startTime: "2026-04-02T02:00:00Z", sizeBytes: 2_389_456_123, tables: 7 },
    { id: "backup_004", configName: "Backup Completo Semanal", type: "full", status: "completed", startTime: "2026-03-30T02:00:00Z", sizeBytes: 2_890_123_456, tables: 8 },
    { id: "backup_005", configName: "Backup Incremental Horario", type: "incremental", status: "failed", startTime: "2026-04-03T09:00:00Z", sizeBytes: 0, tables: 0, error: "Connection timeout" },
  ];

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <h2><Database size={20} /> Gestión de Backups</h2>
        <button 
          className="primary-btn"
          onClick={() => setIsCreating(!isCreating)}
          disabled={isCreating}
        >
          {isCreating ? <RefreshCw size={16} className="spin" /> : <Plus size={16} />}
          {isCreating ? "Creando..." : "Nuevo Backup"}
        </button>
      </div>

      <div className="backup-stats">
        <div className="stat-card">
          <span className="stat-value">4</span>
          <span className="stat-label">Backups Completados</span>
        </div>
        <div className="stat-card">
          <span className="stat-value">8.16 GB</span>
          <span className="stat-label">Espacio Usado</span>
        </div>
        <div className="stat-card">
          <span className="stat-value">98.2%</span>
          <span className="stat-label">Tasa de Éxito</span>
        </div>
        <div className="stat-card">
          <span className="stat-value">2026-04-04 02:00</span>
          <span className="stat-label">Próximo Backup</span>
        </div>
      </div>

      <div className="backup-configs">
        <h3>Configuraciones</h3>
        <div className="configs-list">
          <div className="config-item">
            <div className="config-info">
              <span className="config-name">Backup Completo Diario</span>
              <span className="config-schedule">Diario a las 2:00 AM</span>
            </div>
            <span className="config-status active">Activo</span>
            <button className="icon-btn-small"><Play size={14} /></button>
          </div>
          <div className="config-item">
            <div className="config-info">
              <span className="config-name">Backup Incremental Horario</span>
              <span className="config-schedule">Cada hora</span>
            </div>
            <span className="config-status active">Activo</span>
            <button className="icon-btn-small"><Play size={14} /></button>
          </div>
          <div className="config-item">
            <div className="config-info">
              <span className="config-name">Backup Completo Semanal</span>
              <span className="config-schedule">Domingo 2:00 AM</span>
            </div>
            <span className="config-status active">Activo</span>
            <button className="icon-btn-small"><Play size={14} /></button>
          </div>
        </div>
      </div>

      <div className="backup-history">
        <h3>Historial de Backups</h3>
        <table>
          <thead>
            <tr>
              <th>Configuración</th>
              <th>Tipo</th>
              <th>Estado</th>
              <th>Fecha</th>
              <th>Tamaño</th>
              <th>Tablas</th>
              <th>Acciones</th>
            </tr>
          </thead>
          <tbody>
            {backupJobs.map(job => (
              <tr key={job.id}>
                <td>{job.configName}</td>
                <td><span className="type-badge">{job.type}</span></td>
                <td>
                  {job.status === "completed" ? (
                    <span className="status-completed"><CheckCircle size={14} /> Completado</span>
                  ) : (
                    <span className="status-failed"><XCircle size={14} /> Fallido</span>
                  )}
                </td>
                <td className="date-cell">{new Date(job.startTime).toLocaleString("es-MX")}</td>
                <td>{formatBytes(job.sizeBytes)}</td>
                <td>{job.tables}</td>
                <td className="actions-cell">
                  <button className="icon-btn-small" title="Restaurar"><Download size={14} /></button>
                  <button className="icon-btn-small" title="Eliminar"><Trash2 size={14} /></button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// ── Panel de Reportes ───────────────────────────────────────────────────

function ReportsPanel() {
  const reportTypes = [
    { id: "grid_performance", name: "Rendimiento de Red", description: "Análisis de la red eléctrica", icon: "⚡" },
    { id: "energy_savings", name: "Ahorro Energético", description: "Métricas de energía saved", icon: "💰" },
    { id: "user_activity", name: "Actividad de Usuarios", description: "Uso del sistema por usuarios", icon: "👥" },
    { id: "system_health", name: "Salud del Sistema", description: "Métricas de disponibilidad", icon: "❤️" },
    { id: "audit_summary", name: "Resumen de Auditoría", description: "Eventos de auditoría", icon: "📋" },
    { id: "quartz_operations", name: "Operaciones Cuarzo 4D", description: "Rendimiento cuántico", icon: "💎" },
  ];

  const generatedReports = [
    { id: "report_001", name: "Rendimiento de Red", type: "grid_performance", generatedAt: "2026-04-03T08:00:00Z", format: "PDF", size: "2.3 MB" },
    { id: "report_002", name: "Ahorro Energético", type: "energy_savings", generatedAt: "2026-04-01T09:00:00Z", format: "PDF", size: "1.9 MB" },
    { id: "report_003", name: "Salud del Sistema", type: "system_health", generatedAt: "2026-04-03T06:00:00Z", format: "JSON", size: "125 KB" },
    { id: "report_004", name: "Resumen de Auditoría", type: "audit_summary", generatedAt: "2026-04-01T08:00:00Z", format: "PDF", size: "3.5 MB" },
  ];

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <h2><BarChart3 size={20} /> Reportes Empresariales</h2>
        <button className="primary-btn">
          <Plus size={16} /> Nuevo Reporte
        </button>
      </div>

      <div className="report-types">
        <h3>Tipos de Reporte</h3>
        <div className="types-grid">
          {reportTypes.map(type => (
            <div key={type.id} className="report-type-card">
              <span className="type-icon">{type.icon}</span>
              <div className="type-info">
                <h4>{type.name}</h4>
                <p>{type.description}</p>
              </div>
              <button className="generate-btn">Generar</button>
            </div>
          ))}
        </div>
      </div>

      <div className="reports-history">
        <h3>Reportes Generados</h3>
        <table>
          <thead>
            <tr>
              <th>Nombre</th>
              <th>Tipo</th>
              <th>Fecha</th>
              <th>Formato</th>
              <th>Tamaño</th>
              <th>Acciones</th>
            </tr>
          </thead>
          <tbody>
            {generatedReports.map(report => (
              <tr key={report.id}>
                <td>{report.name}</td>
                <td>{report.type.replace(/_/g, " ")}</td>
                <td className="date-cell">{new Date(report.generatedAt).toLocaleString("es-MX")}</td>
                <td><span className="format-badge">{report.format}</span></td>
                <td>{report.size}</td>
                <td className="actions-cell">
                  <button className="icon-btn-small" title="Descargar"><Download size={14} /></button>
                  <button className="icon-btn-small" title="Ver"><Eye size={14} /></button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// ── Configuración del Sistema ───────────────────────────────────────────

function SettingsPanel() {
  const [activeSection, setActiveSection] = useState("general");

  const sections = [
    { id: "general", label: "General", icon: <Settings size={16} /> },
    { id: "security", label: "Seguridad", icon: <Shield size={16} /> },
    { id: "notifications", label: "Notificaciones", icon: <Activity size={16} /> },
    { id: "integrations", label: "Integraciones", icon: <Database size={16} /> },
    { id: "api", label: "API", icon: <FileText size={16} /> },
  ];

  return (
    <div className="admin-panel settings-panel">
      <div className="panel-header">
        <h2><Settings size={20} /> Configuración del Sistema</h2>
        <button className="primary-btn">
          <Save size={16} /> Guardar Cambios
        </button>
      </div>

      <div className="settings-layout">
        <div className="settings-nav">
          {sections.map(section => (
            <button
              key={section.id}
              className={`settings-nav-item ${activeSection === section.id ? "active" : ""}`}
              onClick={() => setActiveSection(section.id)}
            >
              {section.icon}
              <span>{section.label}</span>
            </button>
          ))}
        </div>

        <div className="settings-content">
          {activeSection === "general" && (
            <div className="settings-section">
              <h3>Configuración General</h3>
              <div className="setting-item">
                <label>Nombre de la Organización</label>
                <input type="text" defaultValue="QuantumEnergyOS" />
              </div>
              <div className="setting-item">
                <label>Zona Horaria</label>
                <select defaultValue="America/Tijuana">
                  <option value="America/Tijuana">America/Tijuana (UTC-7)</option>
                  <option value="America/Mexico_City">America/Mexico_City (UTC-6)</option>
                </select>
              </div>
              <div className="setting-item">
                <label>Idioma</label>
                <select defaultValue="es-MX">
                  <option value="es-MX">Español (México)</option>
                  <option value="en-US">English (US)</option>
                </select>
              </div>
              <div className="setting-item">
                <label>Moneda</label>
                <select defaultValue="MXN">
                  <option value="MXN">Peso Mexicano (MXN)</option>
                  <option value="USD">Dólar Estadounidense (USD)</option>
                </select>
              </div>
            </div>
          )}

          {activeSection === "security" && (
            <div className="settings-section">
              <h3>Configuración de Seguridad</h3>
              <div className="setting-item">
                <label>Autenticación de Dos Factores (2FA)</label>
                <label className="toggle">
                  <input type="checkbox" defaultChecked />
                  <span className="slider"></span>
                </label>
              </div>
              <div className="setting-item">
                <label>Sesión expira después de (minutos)</label>
                <input type="number" defaultValue={60} />
              </div>
              <div className="setting-item">
                <label>Requerir cambio de contraseña cada</label>
                <select defaultValue="90">
                  <option value="30">30 días</option>
                  <option value="60">60 días</option>
                  <option value="90">90 días</option>
                  <option value="180">180 días</option>
                </select>
              </div>
              <div className="setting-item">
                <label>Registro de IP de usuarios</label>
                <label className="toggle">
                  <input type="checkbox" defaultChecked />
                  <span className="slider"></span>
                </label>
              </div>
            </div>
          )}

          {activeSection === "notifications" && (
            <div className="settings-section">
              <h3>Notificaciones</h3>
              <div className="setting-item">
                <label>Notificaciones por email</label>
                <label className="toggle">
                  <input type="checkbox" defaultChecked />
                  <span className="slider"></span>
                </label>
              </div>
              <div className="setting-item">
                <label>Alertas de seguridad</label>
                <label className="toggle">
                  <input type="checkbox" defaultChecked />
                  <span className="slider"></span>
                </label>
              </div>
              <div className="setting-item">
                <label>Reportes programados</label>
                <label className="toggle">
                  <input type="checkbox" defaultChecked />
                  <span className="slider"></span>
                </label>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════════
//  Componente Principal del Panel Enterprise
// ═══════════════════════════════════════════════════════════════════════

interface EnterpriseAdminProps {
  user?: {
    name: string;
    email: string;
    role: string;
  };
  onLogout?: () => void;
}

export default function EnterpriseAdmin({ 
  user = { name: "Gio Corpus", email: "admin@quantumenergy.os", role: "admin" },
  onLogout = () => console.log("Logout")
}: EnterpriseAdminProps) {
  const [activeTab, setActiveTab] = useState<AdminTab>("users");

  return (
    <div className="enterprise-admin">
      <EnterpriseHeader user={user} onLogout={onLogout} />
      <EnterpriseNav activeTab={activeTab} onTabChange={setActiveTab} />
      
      <main className="enterprise-content">
        {activeTab === "users" && <UsersPanel />}
        {activeTab === "roles" && <RolesPanel />}
        {activeTab === "audit" && <AuditPanel />}
        {activeTab === "backups" && <BackupsPanel />}
        {activeTab === "reports" && <ReportsPanel />}
        {activeTab === "settings" && <SettingsPanel />}
      </main>
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════════
//  Estilos CSS para el Panel Enterprise (inline para demo)
// ═══════════════════════════════════════════════════════════════════════

const enterpriseStyles = `
.enterprise-admin {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #0f172a;
  min-height: 100vh;
  color: #e2e8f0;
}

.enterprise-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  background: #1e293b;
  border-bottom: 1px solid #334155;
}

.enterprise-logo {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.logo-icon {
  color: #3b82f6;
}

.logo-text h1 {
  font-size: 1.25rem;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.logo-text span {
  font-size: 0.75rem;
  color: #94a3b8;
}

.enterprise-user {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.user-info {
  text-align: right;
}

.user-name {
  display: block;
  font-weight: 600;
  font-size: 0.875rem;
}

.user-role {
  display: block;
  font-size: 0.75rem;
  color: #94a3b8;
  text-transform: capitalize;
}

.logout-btn {
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid #475569;
  color: #94a3b8;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 0.875rem;
}

.logout-btn:hover {
  background: #334155;
  color: #f8fafc;
}

.enterprise-nav {
  display: flex;
  gap: 0.25rem;
  padding: 0.75rem 1.5rem;
  background: #1e293b;
  border-bottom: 1px solid #334155;
  overflow-x: auto;
}

.nav-tab {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1rem;
  background: transparent;
  border: none;
  color: #94a3b8;
  border-radius: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  white-space: nowrap;
}

.nav-tab:hover {
  background: #334155;
  color: #f8fafc;
}

.nav-tab.active {
  background: #3b82f6;
  color: #f8fafc;
}

.enterprise-content {
  padding: 1.5rem;
}

.admin-panel {
  background: #1e293b;
  border-radius: 0.75rem;
  padding: 1.5rem;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.panel-header h2 {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0;
}

.primary-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1rem;
  background: #3b82f6;
  border: none;
  color: #f8fafc;
  border-radius: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
}

.primary-btn:hover {
  background: #2563eb;
}

.primary-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.secondary-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1rem;
  background: transparent;
  border: 1px solid #475569;
  color: #94a3b8;
  border-radius: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
}

.secondary-btn:hover {
  background: #334155;
  color: #f8fafc;
}

/* Users Table */
.search-bar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background: #0f172a;
  border-radius: 0.5rem;
  margin-bottom: 1.5rem;
}

.search-bar input {
  flex: 1;
  background: transparent;
  border: none;
  color: #f8fafc;
  font-size: 0.875rem;
  outline: none;
}

.users-table table,
.audit-table table,
.backup-history table,
.reports-history table {
  width: 100%;
  border-collapse: collapse;
}

.users-table th,
.audit-table th,
.backup-history th,
.reports-history th {
  text-align: left;
  padding: 0.75rem 1rem;
  background: #0f172a;
  color: #94a3b8;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.users-table td,
.audit-table td,
.backup-history td,
.reports-history td {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #334155;
  font-size: 0.875rem;
}

.user-name-cell {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.avatar {
  width: 2rem;
  height: 2rem;
  background: #3b82f6;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 0.875rem;
}

.role-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: capitalize;
  color: #f8fafc;
}

.status-active,
.status-completed {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: #22c55e;
  font-size: 0.875rem;
}

.status-inactive,
.status-failed {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: #ef4444;
  font-size: 0.875rem;
}

.date-cell {
  color: #94a3b8;
  font-size: 0.875rem;
}

.actions-cell {
  display: flex;
  gap: 0.25rem;
}

.icon-btn-small {
  padding: 0.375rem;
  background: transparent;
  border: 1px solid #475569;
  color: #94a3b8;
  border-radius: 0.25rem;
  cursor: pointer;
}

.icon-btn-small:hover {
  background: #334155;
  color: #f8fafc;
}

.icon-btn-small.danger:hover {
  background: #dc2626;
  border-color: #dc2626;
  color: #f8fafc;
}

/* Audit Panel */
.audit-filters {
  display: flex;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.audit-filters select {
  padding: 0.5rem 1rem;
  background: #0f172a;
  border: 1px solid #475569;
  color: #f8fafc;
  border-radius: 0.375rem;
  font-size: 0.875rem;
}

.audit-stats {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.stat-card {
  background: #0f172a;
  padding: 1rem;
  border-radius: 0.5rem;
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 1.5rem;
  font-weight: 700;
  color: #f8fafc;
}

.stat-label {
  display: block;
  font-size: 0.75rem;
  color: #94a3b8;
  margin-top: 0.25rem;
}

.severity-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: uppercase;
  color: #f8fafc;
}

.event-type {
  text-transform: capitalize;
}

.description-cell {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Backups Panel */
.backup-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.backup-configs {
  margin-bottom: 1.5rem;
}

.backup-configs h3,
.backup-history h3,
.report-types h3,
.reports-history h3 {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 1rem;
}

.configs-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.config-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1rem;
  background: #0f172a;
  border-radius: 0.5rem;
}

.config-info {
  flex: 1;
}

.config-name {
  display: block;
  font-weight: 500;
}

.config-schedule {
  display: block;
  font-size: 0.75rem;
  color: #94a3b8;
}

.config-status {
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 500;
}

.config-status.active {
  background: #22c55e;
  color: #f8fafc;
}

.type-badge {
  padding: 0.25rem 0.5rem;
  background: #475569;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  text-transform: capitalize;
}

.format-badge {
  padding: 0.25rem 0.5rem;
  background: #3b82f6;
  border-radius: 0.25rem;
  font-size: 0.75rem;
}

/* Reports Panel */
.report-types {
  margin-bottom: 1.5rem;
}

.types-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.report-type-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  background: #0f172a;
  border-radius: 0.5rem;
}

.type-icon {
  font-size: 1.5rem;
}

.type-info {
  flex: 1;
}

.type-info h4 {
  font-size: 0.875rem;
  font-weight: 600;
  margin: 0;
}

.type-info p {
  font-size: 0.75rem;
  color: #94a3b8;
  margin: 0;
}

.generate-btn {
  padding: 0.5rem 1rem;
  background: #3b82f6;
  border: none;
  color: #f8fafc;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 0.75rem;
}

.generate-btn:hover {
  background: #2563eb;
}

/* Settings Panel */
.settings-layout {
  display: flex;
  gap: 1.5rem;
}

.settings-nav {
  width: 200px;
  flex-shrink: 0;
}

.settings-nav-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.75rem 1rem;
  background: transparent;
  border: none;
  color: #94a3b8;
  border-radius: 0.5rem;
  cursor: pointer;
  text-align: left;
  font-size: 0.875rem;
}

.settings-nav-item:hover {
  background: #334155;
  color: #f8fafc;
}

.settings-nav-item.active {
  background: #3b82f6;
  color: #f8fafc;
}

.settings-content {
  flex: 1;
  background: #0f172a;
  border-radius: 0.5rem;
  padding: 1.5rem;
}

.settings-section h3 {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 1.5rem;
}

.setting-item {
  margin-bottom: 1.5rem;
}

.setting-item label {
  display: block;
  font-size: 0.875rem;
  color: #94a3b8;
  margin-bottom: 0.5rem;
}

.setting-item input[type="text"],
.setting-item input[type="number"],
.setting-item select {
  width: 100%;
  padding: 0.625rem 1rem;
  background: #1e293b;
  border: 1px solid #475569;
  color: #f8fafc;
  border-radius: 0.375rem;
  font-size: 0.875rem;
}

.toggle {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 24px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #475569;
  transition: 0.3s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background: #f8fafc;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle input:checked + .slider {
  background: #3b82f6;
}

.toggle input:checked + .slider:before {
  transform: translateX(24px);
}

/* Roles Grid */
.roles-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.role-card {
  background: #0f172a;
  border-radius: 0.5rem;
  padding: 1rem;
}

.role-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-left: 3px solid;
  padding-left: 0.75rem;
  margin-bottom: 0.75rem;
}

.role-header h3 {
  font-size: 1rem;
  font-weight: 600;
  margin: 0;
}

.users-count {
  font-size: 0.75rem;
  color: #94a3b8;
}

.role-description {
  font-size: 0.875rem;
  color: #94a3b8;
  margin-bottom: 1rem;
}

.role-permissions h4 {
  font-size: 0.75rem;
  color: #94a3b8;
  margin-bottom: 0.5rem;
}

.permissions-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.permission-tag {
  padding: 0.25rem 0.5rem;
  background: #334155;
  border-radius: 0.25rem;
  font-size: 0.625rem;
  color: #f8fafc;
}

.permission-more {
  padding: 0.25rem 0.5rem;
  background: #475569;
  border-radius: 0.25rem;
  font-size: 0.625rem;
  color: #f8fafc;
}

.role-actions {
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #334155;
}

.text-btn {
  background: transparent;
  border: none;
  color: #3b82f6;
  font-size: 0.875rem;
  cursor: pointer;
}

.text-btn:hover {
  color: #2563eb;
  text-decoration: underline;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Responsive */
@media (max-width: 1200px) {
  .roles-grid,
  .types-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .audit-stats,
  .backup-stats {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .roles-grid,
  .types-grid {
    grid-template-columns: 1fr;
  }
  
  .settings-layout {
    flex-direction: column;
  }
  
  .settings-nav {
    width: 100%;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
}
`;

// Inject styles
if (typeof document !== "undefined") {
  const styleSheet = document.createElement("style");
  styleSheet.textContent = enterpriseStyles;
  document.head.appendChild(styleSheet);
}
